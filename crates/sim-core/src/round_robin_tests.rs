use super::RoundRobinPolicy;
use crate::input::{PolicyConfig, Workload};
use crate::kernel::{
    CoreStatus, Interval, IntervalOwner, Kernel, Policy, RunStatus, State, ThreadStatus,
};
use serde_json::{Value, json};
use std::collections::VecDeque;
use std::num::NonZeroU64;

fn policy(quantum: u64) -> RoundRobinPolicy {
    RoundRobinPolicy::new(NonZeroU64::new(quantum).unwrap())
}

fn workload(
    programs: Vec<Vec<Value>>,
    arrivals: &[u64],
    cores: usize,
    cost: u64,
    quantum: Option<u64>,
) -> Workload {
    let policy = quantum.map_or_else(
        || json!({"kind":"fcfs"}),
        |value| json!({"kind":"roundRobin","quantum":value.to_string()}),
    );
    let value = json!({
        "schemaVersion":"os-lab-workload/1", "applicationVersion":"app.test",
        "modelVersion":"sched.m1/revision-1", "engineVersion":"sim-engine.0.1.0",
        "randomAlgorithmVersion":"none.v1", "seed":"1",
        "machine":{"machineId":"m","cores":cores,"switchCost":cost.to_string(),"controlOperationBudgetPerTick":"100"},
        "policy":policy,
        "programs": programs.iter().enumerate().map(|(i, blocks)| json!({
            "schemaVersion":"os-lab-program/1","programId":format!("p{i}"),"name":"test","parameters":[],"blocks":blocks
        })).collect::<Vec<_>>(),
        "threads": arrivals.iter().enumerate().map(|(i, arrival)| json!({
            "threadId":format!("m:t{i}"),"programId":format!("p{i}"),"arrival":arrival.to_string(),"parameters":{}
        })).collect::<Vec<_>>()
    });
    Workload::decode(&serde_json::to_vec(&value).unwrap()).unwrap()
}

fn compute(id: &str, duration: u64) -> Value {
    json!({"blockId":id,"op":"compute","duration":{"literal":duration.to_string()}})
}

fn io_wait(id: &str, duration: u64) -> Value {
    json!({"blockId":id,"op":"ioWait","duration":{"literal":duration.to_string()}})
}

fn yield_now(id: &str) -> Value {
    json!({"blockId":id,"op":"yield"})
}

fn end(id: &str) -> Value {
    json!({"blockId":id,"op":"end"})
}

fn run(mut kernel: Kernel<RoundRobinPolicy>) -> (Vec<Value>, Vec<Interval>, State) {
    let mut events = Vec::new();
    let mut intervals = Vec::new();
    for _ in 0..100 {
        let boundary = kernel.advance_tick();
        events.extend(boundary.events);
        intervals.extend(boundary.intervals);
        if !matches!(
            boundary.state.status,
            RunStatus::Created | RunStatus::Running
        ) {
            return (events, intervals, boundary.state);
        }
    }
    panic!("round-robin fixture did not terminate")
}

fn core_intervals(
    intervals: &[Interval],
    core: usize,
) -> Vec<(u64, u64, CoreStatus, Option<String>)> {
    intervals
        .iter()
        .filter_map(|interval| match &interval.owner {
            IntervalOwner::Core {
                core_id,
                status,
                thread_id,
            } if *core_id == core => {
                Some((interval.start, interval.end, *status, thread_id.clone()))
            }
            _ => None,
        })
        .collect()
}

fn thread_intervals(intervals: &[Interval], thread: &str) -> Vec<(u64, u64, ThreadStatus)> {
    intervals
        .iter()
        .filter_map(|interval| match &interval.owner {
            IntervalOwner::Thread { thread_id, status } if thread_id == thread => {
                Some((interval.start, interval.end, *status))
            }
            _ => None,
        })
        .collect()
}

fn dispatches(events: &[Value]) -> Vec<(u64, String, usize)> {
    events
        .iter()
        .filter(|event| event["kind"] == "dispatchStarted")
        .map(|event| {
            (
                event["tick"].as_str().unwrap().parse().unwrap(),
                event["threadId"].as_str().unwrap().to_owned(),
                event["payload"]["coreId"].as_u64().unwrap() as usize,
            )
        })
        .collect()
}

fn event_count(events: &[Value], kind: &str) -> usize {
    events.iter().filter(|event| event["kind"] == kind).count()
}

#[test]
fn policy_preserves_quantum_and_selects_fifo_head_without_mutation() {
    let queue = VecDeque::from([4, 1, 9]);
    let before = queue.clone();
    let mut scheduler = policy(3);
    assert_eq!(scheduler.select(&queue), Some(4));
    assert_eq!(queue, before);
    assert_eq!(
        scheduler.configuration(),
        PolicyConfig::RoundRobin { quantum: 3 }
    );
    assert_eq!(scheduler.quantum(), NonZeroU64::new(3));
}

#[test]
fn c02_completion_at_quantum_boundary_wins_and_is_deterministic() {
    let input = workload(
        vec![
            vec![compute("a-cpu", 2), end("a-end")],
            vec![compute("b-cpu", 1), end("b-end")],
        ],
        &[0, 0],
        1,
        0,
        Some(2),
    );
    let first = run(Kernel::new(input.clone(), policy(2)).unwrap());
    let second = run(Kernel::new(input, policy(2)).unwrap());
    assert_eq!(first, second);
    assert_eq!(first.2.tick, 3);
    assert_eq!(first.2.status, RunStatus::Completed);
    assert_eq!(event_count(&first.0, "quantumExpired"), 0);
    assert_eq!(
        dispatches(&first.0),
        vec![(0, "m:t0".into(), 0), (2, "m:t1".into(), 0)]
    );
    assert_eq!(
        core_intervals(&first.1, 0),
        vec![
            (0, 2, CoreStatus::Running, Some("m:t0".into())),
            (2, 3, CoreStatus::Running, Some("m:t1".into()))
        ]
    );
}

#[test]
fn c04_io_wakeup_precedes_same_tick_expiration_requeue() {
    let input = workload(
        vec![
            vec![
                compute("a-first", 1),
                io_wait("a-io", 2),
                compute("a-second", 1),
                end("a-end"),
            ],
            vec![compute("b-cpu", 4), end("b-end")],
        ],
        &[0, 0],
        1,
        0,
        Some(2),
    );
    let (events, intervals, state) = run(Kernel::new(input, policy(2)).unwrap());
    assert_eq!(state.tick, 6);
    assert_eq!(
        dispatches(&events),
        vec![
            (0, "m:t0".into(), 0),
            (1, "m:t1".into(), 0),
            (3, "m:t0".into(), 0),
            (4, "m:t1".into(), 0)
        ]
    );
    let tick_three = events
        .iter()
        .filter(|event| event["tick"] == "3")
        .map(|event| event["kind"].as_str().unwrap())
        .collect::<Vec<_>>();
    let wake = tick_three
        .iter()
        .position(|kind| *kind == "ioCompleted")
        .unwrap();
    let expire = tick_three
        .iter()
        .position(|kind| *kind == "quantumExpired")
        .unwrap();
    assert!(wake < expire);
    assert_eq!(
        core_intervals(&intervals, 0),
        vec![
            (0, 1, CoreStatus::Running, Some("m:t0".into())),
            (1, 3, CoreStatus::Running, Some("m:t1".into())),
            (3, 4, CoreStatus::Running, Some("m:t0".into())),
            (4, 6, CoreStatus::Running, Some("m:t1".into()))
        ]
    );
    assert_eq!(
        thread_intervals(&intervals, "m:t0"),
        vec![
            (0, 1, ThreadStatus::Running),
            (1, 3, ThreadStatus::Blocked),
            (3, 4, ThreadStatus::Running)
        ]
    );
}

#[test]
fn yield_discards_grant_and_requeues_at_tail() {
    let input = workload(
        vec![
            vec![
                compute("a-first", 1),
                yield_now("a-yield"),
                compute("a-second", 3),
                end("a-end"),
            ],
            vec![compute("b-cpu", 2), end("b-end")],
        ],
        &[0, 0],
        1,
        0,
        Some(2),
    );
    let (events, intervals, state) = run(Kernel::new(input, policy(2)).unwrap());
    assert_eq!(state.tick, 6);
    assert_eq!(event_count(&events, "quantumExpired"), 1);
    assert_eq!(
        dispatches(&events),
        vec![
            (0, "m:t0".into(), 0),
            (1, "m:t1".into(), 0),
            (3, "m:t0".into(), 0),
            (5, "m:t0".into(), 0)
        ]
    );
    assert_eq!(
        core_intervals(&intervals, 0),
        vec![
            (0, 1, CoreStatus::Running, Some("m:t0".into())),
            (1, 3, CoreStatus::Running, Some("m:t1".into())),
            (3, 5, CoreStatus::Running, Some("m:t0".into())),
            (5, 6, CoreStatus::Running, Some("m:t0".into()))
        ]
    );
}

#[test]
fn blocking_discards_unused_grant_and_wakeup_gets_a_fresh_quantum() {
    let input = workload(
        vec![
            vec![
                compute("a-first", 1),
                io_wait("a-io", 1),
                compute("a-second", 3),
                end("a-end"),
            ],
            vec![compute("b-cpu", 1), end("b-end")],
        ],
        &[0, 0],
        1,
        0,
        Some(2),
    );
    let (events, intervals, state) = run(Kernel::new(input, policy(2)).unwrap());
    assert_eq!(state.tick, 5);
    let expirations = events
        .iter()
        .filter(|event| event["kind"] == "quantumExpired")
        .map(|event| {
            (
                event["tick"].as_str().unwrap(),
                event["threadId"].as_str().unwrap(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(expirations, vec![("4", "m:t0")]);
    assert_eq!(
        dispatches(&events),
        vec![
            (0, "m:t0".into(), 0),
            (1, "m:t1".into(), 0),
            (2, "m:t0".into(), 0),
            (4, "m:t0".into(), 0)
        ]
    );
    assert_eq!(
        thread_intervals(&intervals, "m:t0"),
        vec![
            (0, 1, ThreadStatus::Running),
            (1, 2, ThreadStatus::Blocked),
            (2, 4, ThreadStatus::Running),
            (4, 5, ThreadStatus::Running)
        ]
    );
}

#[test]
fn dispatch_overhead_does_not_consume_useful_quantum() {
    let input = workload(
        vec![
            vec![compute("a-cpu", 2), end("a-end")],
            vec![compute("b-cpu", 1), end("b-end")],
        ],
        &[0, 0],
        1,
        1,
        Some(1),
    );
    let (events, intervals, state) = run(Kernel::new(input, policy(1)).unwrap());
    assert_eq!(state.tick, 6);
    assert_eq!(event_count(&events, "quantumExpired"), 1);
    assert_eq!(
        core_intervals(&intervals, 0),
        vec![
            (0, 1, CoreStatus::Dispatching, Some("m:t0".into())),
            (1, 2, CoreStatus::Running, Some("m:t0".into())),
            (2, 3, CoreStatus::Dispatching, Some("m:t1".into())),
            (3, 4, CoreStatus::Running, Some("m:t1".into())),
            (4, 5, CoreStatus::Dispatching, Some("m:t0".into())),
            (5, 6, CoreStatus::Running, Some("m:t0".into()))
        ]
    );
    assert_eq!(
        thread_intervals(&intervals, "m:t0"),
        vec![
            (0, 1, ThreadStatus::Dispatching),
            (1, 2, ThreadStatus::Running),
            (2, 4, ThreadStatus::Ready),
            (4, 5, ThreadStatus::Dispatching),
            (5, 6, ThreadStatus::Running)
        ]
    );
}

#[test]
fn multicore_expiration_and_redispatch_order_is_stable() {
    let input = workload(
        vec![
            vec![compute("a-cpu", 2), end("a-end")],
            vec![compute("b-cpu", 2), end("b-end")],
            vec![compute("c-cpu", 1), end("c-end")],
        ],
        &[0, 0, 0],
        2,
        0,
        Some(1),
    );
    let (events, intervals, state) = run(Kernel::new(input, policy(1)).unwrap());
    assert_eq!(state.tick, 3);
    assert_eq!(event_count(&events, "quantumExpired"), 2);
    assert_eq!(
        dispatches(&events),
        vec![
            (0, "m:t0".into(), 0),
            (0, "m:t1".into(), 1),
            (1, "m:t2".into(), 0),
            (1, "m:t0".into(), 1),
            (2, "m:t1".into(), 0)
        ]
    );
    assert_eq!(
        core_intervals(&intervals, 0),
        vec![
            (0, 1, CoreStatus::Running, Some("m:t0".into())),
            (1, 2, CoreStatus::Running, Some("m:t2".into())),
            (2, 3, CoreStatus::Running, Some("m:t1".into()))
        ]
    );
    assert_eq!(
        core_intervals(&intervals, 1),
        vec![
            (0, 1, CoreStatus::Running, Some("m:t1".into())),
            (1, 2, CoreStatus::Running, Some("m:t0".into())),
            (2, 3, CoreStatus::Idle, None)
        ]
    );
}

#[test]
fn constructor_rejects_fcfs_and_different_quantum_workloads() {
    let program = vec![vec![end("done")]];
    let fcfs = workload(program.clone(), &[0], 1, 0, None);
    assert!(Kernel::new(fcfs, policy(1)).is_err());
    let quantum_two = workload(program, &[0], 1, 0, Some(2));
    assert!(Kernel::new(quantum_two, policy(1)).is_err());
}
