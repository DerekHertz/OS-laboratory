//! Single-threaded sched.m1 phase closure. No wall clock, worker, or UI dependencies.
use crate::input::{Op, Operand, PolicyConfig, Workload};
use serde_json::{Value, json};
use std::cmp::Reverse;
use std::collections::{BinaryHeap, VecDeque};

/// T04/T05 supply production policies. A policy may inspect the FIFO and select its
/// head, but cannot mutate lifecycle ownership or run time. A missing/wrong choice
/// while ready work exists is an engine error rather than an unstable boundary.
pub trait Policy {
    fn configuration(&self) -> PolicyConfig;
    fn quantum(&self) -> Option<std::num::NonZeroU64>;
    fn select(&mut self, ready: &VecDeque<usize>) -> Option<usize>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadStatus {
    NotArrived,
    Ready,
    Dispatching,
    Running,
    Blocked,
    Terminated,
    Error,
    Censored,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoreStatus {
    Idle,
    Dispatching,
    Running,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunStatus {
    Created,
    Running,
    Completed,
    Error,
    Quiescent,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    ArithmeticOverflow,
    ControlBudgetExceeded,
    InternalEngine,
}
impl ErrorCode {
    fn name(self) -> &'static str {
        match self {
            Self::ArithmeticOverflow => "arithmeticOverflow",
            Self::ControlBudgetExceeded => "controlBudgetExceeded",
            Self::InternalEngine => "internalEngine",
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeError {
    pub code: ErrorCode,
    pub tick: u64,
    pub thread_id: Option<String>,
    pub block_id: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThreadState {
    pub thread_id: String,
    pub status: ThreadStatus,
    pub core_id: Option<usize>,
    pub block_id: String,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreState {
    pub core_id: usize,
    pub status: CoreStatus,
    pub thread_id: Option<String>,
}
/// Stable kernel state. Protocol sequences, retention, deltas, and command routing
/// are deliberately left to T06/T07; this is not a T02 transport Snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct State {
    pub tick: u64,
    pub status: RunStatus,
    pub threads: Vec<ThreadState>,
    pub cores: Vec<CoreState>,
    pub ready_queue: Vec<String>,
    pub error: Option<RuntimeError>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntervalOwner {
    Thread {
        thread_id: String,
        status: ThreadStatus,
    },
    Core {
        core_id: usize,
        status: CoreStatus,
        thread_id: Option<String>,
    },
}
/// Raw half-open accounting facts, not aggregated metrics or retained trace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Interval {
    pub start: u64,
    pub end: u64,
    pub owner: IntervalOwner,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Boundary {
    pub state: State,
    pub events: Vec<Value>,
    pub intervals: Vec<Interval>,
}

#[derive(Debug)]
struct Thread {
    status: ThreadStatus,
    core: Option<usize>,
    pc: usize,
    loops: Vec<(usize, u64)>,
    remaining: Option<u64>,
    service_start: u64,
    quantum_left: Option<u64>,
    instruction_generation: u64,
    since: u64,
    last_event: Option<String>,
}
#[derive(Debug)]
struct Core {
    thread: Option<usize>,
    status: CoreStatus,
    generation: u64,
    since: u64,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Kind {
    Compute,
    Switch,
    Arrival,
    Io,
    Expire,
}
impl Kind {
    fn phase(self) -> u8 {
        match self {
            Self::Compute | Self::Switch => 1,
            Self::Arrival | Self::Io => 2,
            Self::Expire => 3,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Pending {
    tick: u64,
    phase: u8,
    sequence: u64,
    kind: Kind,
    thread: usize,
    core: usize,
    generation: u64,
    instruction: u64,
}

pub struct Kernel<P: Policy> {
    workload: Workload,
    policy: P,
    tick: u64,
    status: RunStatus,
    threads: Vec<Thread>,
    cores: Vec<Core>,
    core_entities: Vec<String>,
    quantum: Option<u64>,
    ready: VecDeque<usize>,
    pending: BinaryHeap<Reverse<Pending>>,
    insertion: u64,
    event_sequence: u64,
    controls: u64,
    events: Vec<Value>,
    intervals: Vec<Interval>,
    error: Option<RuntimeError>,
}
type Result<T> = std::result::Result<T, Fault>;
#[derive(Debug, Clone, Copy)]
struct Fault {
    code: ErrorCode,
    thread: Option<usize>,
}
impl Fault {
    fn overflow(thread: usize) -> Self {
        Self {
            code: ErrorCode::ArithmeticOverflow,
            thread: Some(thread),
        }
    }
    fn internal() -> Self {
        Self {
            code: ErrorCode::InternalEngine,
            thread: None,
        }
    }
}
impl<P: Policy> Kernel<P> {
    /// No production scheduler is supplied by T03. The injected policy must match
    /// the persisted configuration; a mismatch rejects construction atomically.
    pub fn new(workload: Workload, policy: P) -> std::result::Result<Self, &'static str> {
        if *workload.policy() != policy.configuration() {
            return Err("policy does not match persisted workload");
        }
        if workload.policy().quantum() != policy.quantum() {
            return Err("policy quantum does not match persisted workload");
        }
        let quantum = policy.quantum().map(std::num::NonZeroU64::get);
        let core_entities = (0..workload.cores)
            .map(|core| {
                let mut id = format!("{}:core:{core}", workload.machine);
                while workload.threads.iter().any(|thread| thread.id == id) {
                    id.push(':');
                }
                id
            })
            .collect();
        let threads = workload
            .threads
            .iter()
            .map(|_| Thread {
                status: ThreadStatus::NotArrived,
                core: None,
                pc: 0,
                loops: Vec::new(),
                remaining: None,
                service_start: 0,
                quantum_left: None,
                instruction_generation: 0,
                since: 0,
                last_event: None,
            })
            .collect();
        let cores = (0..workload.cores)
            .map(|_| Core {
                thread: None,
                status: CoreStatus::Idle,
                generation: 0,
                since: 0,
            })
            .collect();
        let mut kernel = Self {
            workload,
            policy,
            tick: 0,
            status: RunStatus::Created,
            threads,
            cores,
            core_entities,
            quantum,
            ready: VecDeque::new(),
            pending: BinaryHeap::new(),
            insertion: 0,
            event_sequence: 0,
            controls: 0,
            events: Vec::new(),
            intervals: Vec::new(),
            error: None,
        };
        for thread in 0..kernel.threads.len() {
            // <= 10,000 validated arrivals cannot exhaust a fresh u64 counter.
            kernel
                .schedule(
                    kernel.workload.threads[thread].arrival,
                    Kind::Arrival,
                    thread,
                    0,
                )
                .map_err(|_| "initial event counter overflow")?;
        }
        Ok(kernel)
    }
    pub fn workload(&self) -> &Workload {
        &self.workload
    }
    pub fn state(&self) -> State {
        State {
            tick: self.tick,
            status: self.status,
            threads: self
                .threads
                .iter()
                .enumerate()
                .map(|(i, t)| ThreadState {
                    thread_id: self.thread_id(i).to_owned(),
                    status: t.status,
                    core_id: t.core,
                    block_id: self.block_id(i).to_owned(),
                })
                .collect(),
            cores: self
                .cores
                .iter()
                .enumerate()
                .map(|(i, c)| CoreState {
                    core_id: i,
                    status: c.status,
                    thread_id: c.thread.map(|t| self.thread_id(t).to_owned()),
                })
                .collect(),
            ready_queue: self
                .ready
                .iter()
                .map(|t| self.thread_id(*t).to_owned())
                .collect(),
            error: self.error.clone(),
        }
    }
    /// Advance one complete active tick, including the phase-4 fixpoint. Stale
    /// heap entries neither advance simulated time nor produce events/intervals.
    /// Caller owns each returned batch; the kernel keeps no historical trace.
    pub fn advance_tick(&mut self) -> Boundary {
        if matches!(self.status, RunStatus::Created | RunStatus::Running) {
            if let Some(next) = self.pop_live() {
                if next.tick != self.tick {
                    self.controls = 0;
                }
                self.tick = next.tick;
                self.status = RunStatus::Running;
                let result = self.process_tick(next);
                if let Err(fault) = result {
                    self.fail(fault);
                }
            } else {
                self.status = RunStatus::Quiescent;
                self.close_all();
            }
        }
        Boundary {
            state: self.state(),
            events: std::mem::take(&mut self.events),
            intervals: std::mem::take(&mut self.intervals),
        }
    }
    fn process_tick(&mut self, first: Pending) -> Result<()> {
        self.apply(first)?;
        loop {
            while self.pending.peek().is_some_and(|e| !self.applicable(&e.0)) {
                self.pending.pop();
            }
            if self.pending.peek().is_none_or(|e| e.0.tick != self.tick) {
                break;
            }
            let next = self.pending.pop().expect("peeked pending event").0;
            self.apply(next)?;
        }
        self.dispatch()?;
        if self
            .threads
            .iter()
            .all(|t| t.status == ThreadStatus::Terminated)
        {
            self.emit("runCompleted", None, None, false, json!({"completedThreads": self.threads.len(), "erroredThreads": 0, "censoredThreads": 0}))?;
            self.status = RunStatus::Completed;
            self.pending.clear();
            self.close_all();
        }
        Ok(())
    }
    fn pop_live(&mut self) -> Option<Pending> {
        while let Some(Reverse(event)) = self.pending.pop() {
            if self.applicable(&event) {
                return Some(event);
            }
        }
        None
    }
    fn applicable(&self, e: &Pending) -> bool {
        let t = &self.threads[e.thread];
        match e.kind {
            Kind::Arrival => t.status == ThreadStatus::NotArrived,
            Kind::Io => {
                t.status == ThreadStatus::Blocked && t.instruction_generation == e.instruction
            }
            Kind::Switch | Kind::Compute | Kind::Expire => {
                let c = &self.cores[e.core];
                c.thread == Some(e.thread)
                    && t.core == Some(e.core)
                    && c.generation == e.generation
                    && if e.kind == Kind::Switch {
                        t.status == ThreadStatus::Dispatching
                    } else {
                        t.status == ThreadStatus::Running
                            && t.instruction_generation == e.instruction
                    }
            }
        }
    }
    fn schedule(&mut self, tick: u64, kind: Kind, thread: usize, core: usize) -> Result<()> {
        let sequence = self
            .insertion
            .checked_add(1)
            .ok_or(Fault::overflow(thread))?;
        self.insertion = sequence;
        self.pending.push(Reverse(Pending {
            tick,
            phase: kind.phase(),
            sequence,
            kind,
            thread,
            core,
            generation: self.cores[core].generation,
            instruction: self.threads[thread].instruction_generation,
        }));
        Ok(())
    }
    fn apply(&mut self, e: Pending) -> Result<()> {
        let (thread, core) = (e.thread, e.core);
        self.ensure_event(Some(thread))?;
        match e.kind {
            Kind::Arrival => {
                self.emit(
                    "arrival",
                    Some(thread),
                    None,
                    false,
                    json!({"declarationIndex": thread}),
                )?;
                self.make_ready(thread);
            }
            Kind::Io => {
                self.emit("ioCompleted", Some(thread), None, true, json!({}))?;
                self.threads[thread].pc += 1;
                self.make_ready(thread);
            }
            Kind::Switch => self.start_running(thread, core)?,
            Kind::Compute => {
                self.account_service(thread);
                self.emit_core("computeCompleted", thread, core, true)?;
                self.threads[thread].remaining = None;
                self.threads[thread].pc += 1;
                self.drive(thread, core)?;
            }
            Kind::Expire => {
                self.account_service(thread);
                self.emit_core("quantumExpired", thread, core, true)?;
                self.release(thread, core);
                self.make_ready(thread);
            }
        }
        Ok(())
    }
    fn dispatch(&mut self) -> Result<()> {
        while !self.ready.is_empty() && self.cores.iter().any(|c| c.status == CoreStatus::Idle) {
            for core in 0..self.cores.len() {
                if self.cores[core].status != CoreStatus::Idle || self.ready.is_empty() {
                    continue;
                }
                let thread = self
                    .policy
                    .select(&self.ready)
                    .ok_or_else(Fault::internal)?;
                if Some(&thread) != self.ready.front() {
                    return Err(Fault::internal());
                }
                self.ensure_event(Some(thread))?;
                let generation = self.cores[core]
                    .generation
                    .checked_add(1)
                    .ok_or(Fault::overflow(thread))?;
                let end = self
                    .tick
                    .checked_add(self.workload.switch_cost)
                    .ok_or(Fault::overflow(thread))?;
                self.ready.pop_front();
                self.set_thread_status(thread, ThreadStatus::Dispatching);
                self.set_core_status(core, CoreStatus::Dispatching);
                self.cores[core].thread = Some(thread);
                self.cores[core].generation = generation;
                self.threads[thread].core = Some(core);
                self.emit_core("dispatchStarted", thread, core, false)?;
                if self.workload.switch_cost == 0 {
                    self.start_running(thread, core)?;
                } else {
                    self.schedule(end, Kind::Switch, thread, core)?;
                }
            }
        }
        Ok(())
    }
    fn start_running(&mut self, thread: usize, core: usize) -> Result<()> {
        self.emit_core("dispatchCompleted", thread, core, false)?;
        self.set_thread_status(thread, ThreadStatus::Running);
        self.set_core_status(core, CoreStatus::Running);
        self.threads[thread].quantum_left = self.quantum;
        self.drive(thread, core)
    }
    fn drive(&mut self, thread: usize, core: usize) -> Result<()> {
        loop {
            let pc = self.threads[thread].pc;
            let op = self.workload.programs[self.workload.threads[thread].program]
                .instructions
                .get(pc)
                .ok_or_else(Fault::internal)?
                .op
                .clone();
            match op {
                Op::Compute(operand) => {
                    self.ensure_event(Some(thread))?;
                    let remaining = self.threads[thread]
                        .remaining
                        .unwrap_or_else(|| self.resolve(thread, &operand));
                    let completion = self
                        .tick
                        .checked_add(remaining)
                        .ok_or(Fault::overflow(thread))?;
                    let expiration = self.threads[thread]
                        .quantum_left
                        .map(|q| self.tick.checked_add(q).ok_or(Fault::overflow(thread)))
                        .transpose()?;
                    let instruction_generation = self.threads[thread]
                        .instruction_generation
                        .checked_add(1)
                        .ok_or(Fault::overflow(thread))?;
                    self.threads[thread].instruction_generation = instruction_generation;
                    self.threads[thread].remaining = Some(remaining);
                    self.threads[thread].service_start = self.tick;
                    self.emit_core("computeStarted", thread, core, true)?;
                    self.schedule(completion, Kind::Compute, thread, core)?;
                    if let Some(expiration) = expiration {
                        self.schedule(expiration, Kind::Expire, thread, core)?;
                    }
                    return Ok(());
                }
                Op::Io(operand) => {
                    self.ensure_event(Some(thread))?;
                    self.control(thread)?;
                    let duration = self.resolve(thread, &operand);
                    let completion = self
                        .tick
                        .checked_add(duration)
                        .ok_or(Fault::overflow(thread))?;
                    self.emit("ioSubmitted", Some(thread), None, true, json!({"duration": duration.to_string(), "completionTick": completion.to_string()}))?;
                    self.schedule(completion, Kind::Io, thread, core)?;
                    self.release(thread, core);
                    self.set_thread_status(thread, ThreadStatus::Blocked);
                    return Ok(());
                }
                Op::Yield => {
                    self.ensure_event(Some(thread))?;
                    self.control(thread)?;
                    self.emit("yielded", Some(thread), None, true, json!({}))?;
                    self.threads[thread].pc += 1;
                    self.release(thread, core);
                    self.make_ready(thread);
                    return Ok(());
                }
                Op::End => {
                    self.ensure_event(Some(thread))?;
                    self.control(thread)?;
                    self.emit("terminated", Some(thread), None, true, json!({}))?;
                    self.release(thread, core);
                    self.set_thread_status(thread, ThreadStatus::Terminated);
                    return Ok(());
                }
                Op::Repeat { count, .. } => {
                    self.control(thread)?;
                    let count = self.resolve(thread, &count);
                    self.threads[thread].loops.push((pc, count));
                    self.threads[thread].pc += 1;
                }
                Op::RepeatNext { start } => {
                    let (active, remaining) = *self.threads[thread]
                        .loops
                        .last()
                        .ok_or_else(Fault::internal)?;
                    if active != start {
                        return Err(Fault::internal());
                    }
                    if remaining > 1 {
                        self.control(thread)?;
                        self.threads[thread]
                            .loops
                            .last_mut()
                            .expect("checked loop")
                            .1 -= 1;
                        self.threads[thread].pc = start + 1;
                    } else {
                        self.threads[thread].loops.pop();
                        let Op::Repeat { after, .. } = self.workload.programs
                            [self.workload.threads[thread].program]
                            .instructions[start]
                            .op
                        else {
                            return Err(Fault::internal());
                        };
                        self.threads[thread].pc = after;
                    }
                }
            }
        }
    }
    fn control(&mut self, thread: usize) -> Result<()> {
        if self.controls >= self.workload.control_budget {
            return Err(Fault {
                code: ErrorCode::ControlBudgetExceeded,
                thread: Some(thread),
            });
        }
        self.controls = self
            .controls
            .checked_add(1)
            .ok_or(Fault::overflow(thread))?;
        Ok(())
    }
    fn account_service(&mut self, thread: usize) {
        let t = &mut self.threads[thread];
        let elapsed = self.tick - t.service_start;
        t.remaining = t.remaining.map(|n| n - elapsed);
        t.quantum_left = t.quantum_left.map(|n| n - elapsed);
        t.service_start = self.tick;
    }
    fn resolve(&self, thread: usize, operand: &Operand) -> u64 {
        match operand {
            Operand::Literal(n) => *n,
            Operand::Parameter(id) => self.workload.threads[thread].parameters[id] as u64,
        }
    }
    fn make_ready(&mut self, thread: usize) {
        self.set_thread_status(thread, ThreadStatus::Ready);
        self.ready.push_back(thread);
    }
    fn release(&mut self, thread: usize, core: usize) {
        self.set_core_status(core, CoreStatus::Idle);
        self.cores[core].thread = None;
        self.threads[thread].core = None;
    }
    fn set_thread_status(&mut self, thread: usize, status: ThreadStatus) {
        self.close_thread(thread);
        self.threads[thread].status = status;
    }
    fn set_core_status(&mut self, core: usize, status: CoreStatus) {
        self.close_core(core);
        self.cores[core].status = status;
    }
    fn close_thread(&mut self, thread: usize) {
        let t = &self.threads[thread];
        if t.since < self.tick
            && !matches!(
                t.status,
                ThreadStatus::NotArrived
                    | ThreadStatus::Terminated
                    | ThreadStatus::Error
                    | ThreadStatus::Censored
            )
        {
            self.intervals.push(Interval {
                start: t.since,
                end: self.tick,
                owner: IntervalOwner::Thread {
                    thread_id: self.thread_id(thread).to_owned(),
                    status: t.status,
                },
            });
        }
        self.threads[thread].since = self.tick;
    }
    fn close_core(&mut self, core: usize) {
        let c = &self.cores[core];
        if c.since < self.tick {
            self.intervals.push(Interval {
                start: c.since,
                end: self.tick,
                owner: IntervalOwner::Core {
                    core_id: core,
                    status: c.status,
                    thread_id: c.thread.map(|t| self.thread_id(t).to_owned()),
                },
            });
        }
        self.cores[core].since = self.tick;
    }
    fn close_all(&mut self) {
        for t in 0..self.threads.len() {
            self.close_thread(t);
        }
        for c in 0..self.cores.len() {
            self.close_core(c);
        }
    }
    fn fail(&mut self, fault: Fault) {
        let error = RuntimeError {
            code: fault.code,
            tick: self.tick,
            thread_id: fault.thread.map(|t| self.thread_id(t).to_owned()),
            block_id: fault.thread.map(|t| self.block_id(t).to_owned()),
        };
        // Fatal cleanup discards every queued token and clears all ownership.
        // Generations need not increment (and could themselves have overflowed):
        // the terminal state can never accept another event or dispatch.
        self.pending.clear();
        self.ready.clear();
        self.close_all();
        for (i, t) in self.threads.iter_mut().enumerate() {
            if t.status != ThreadStatus::Terminated {
                t.status = if fault.thread == Some(i) {
                    ThreadStatus::Error
                } else {
                    ThreadStatus::Censored
                };
            }
            t.core = None;
        }
        for c in &mut self.cores {
            c.thread = None;
            c.status = CoreStatus::Idle;
        }
        self.status = RunStatus::Error;
        self.error = Some(error);
        // emit reserves the final u64 sequence for a fatal report.
        let _ = self.emit(
            "runtimeError",
            fault.thread,
            None,
            fault.thread.is_some(),
            json!({"code": fault.code.name()}),
        );
    }
    fn thread_id(&self, thread: usize) -> &str {
        &self.workload.threads[thread].id
    }
    fn block_id(&self, thread: usize) -> &str {
        &self.workload.programs[self.workload.threads[thread].program].instructions
            [self.threads[thread].pc]
            .id
    }
    fn emit_core(&mut self, kind: &str, thread: usize, core: usize, block: bool) -> Result<()> {
        self.emit(
            kind,
            Some(thread),
            Some(core),
            block,
            json!({"coreId": core, "generation": self.cores[core].generation.to_string()}),
        )
    }
    fn emit(
        &mut self,
        kind: &str,
        thread: Option<usize>,
        core: Option<usize>,
        block: bool,
        payload: Value,
    ) -> Result<()> {
        if kind != "runtimeError" {
            self.ensure_event(thread)?;
        }
        let sequence = self
            .event_sequence
            .checked_add(1)
            .ok_or_else(Fault::internal)?;
        self.event_sequence = sequence;
        let event_id = format!("{}:event:{sequence}", self.workload.machine);
        let mut entities = Vec::new();
        let mut parents = Vec::new();
        if let Some(t) = thread {
            entities.push(self.thread_id(t).to_owned());
            if let Some(parent) = &self.threads[t].last_event {
                parents.push(parent.clone());
            }
        }
        if let Some(c) = core {
            entities.push(self.core_entities[c].clone());
        }
        if thread.is_none() && sequence > 1 {
            parents.push(format!("{}:event:{}", self.workload.machine, sequence - 1));
        }
        let mut event = json!({"eventId": event_id, "eventSequence": sequence.to_string(), "tick": self.tick.to_string(), "kind": kind, "entityIds": entities, "causalParentIds": parents, "payload": payload});
        if let Some(t) = thread {
            event["threadId"] = json!(self.thread_id(t));
            if block {
                event["blockId"] = json!(self.block_id(t));
            }
            self.threads[t].last_event = Some(event_id);
        }
        self.events.push(event);
        Ok(())
    }
    fn ensure_event(&self, thread: Option<usize>) -> Result<()> {
        if self.event_sequence >= u64::MAX - 1 {
            Err(thread.map(Fault::overflow).unwrap_or_else(Fault::internal))
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
#[path = "kernel_tests.rs"]
mod tests;
