use super::FcfsPolicy;
use crate::input::Workload;
use crate::kernel::{
    CoreStatus, Interval, IntervalOwner, Kernel, Policy, RunStatus, State, ThreadStatus,
};
use serde_json::{Value, json};
use std::collections::VecDeque;

fn workload(programs: Vec<Vec<Value>>, arrivals: &[u64], cores: usize, cost: u64) -> Workload {
    let value = json!({
        "schemaVersion":"os-lab-workload/1", "applicationVersion":"app.test",
        "modelVersion":"sched.m1/revision-1", "engineVersion":"sim-engine.0.1.0",
        "randomAlgorithmVersion":"none.v1", "seed":"1",
        "machine":{"machineId":"m","cores":cores,"switchCost":cost.to_string(),"controlOperationBudgetPerTick":"100"},
        "policy":{"kind":"fcfs"},
        "programs": programs.iter().enumerate().map(|(i, blocks)| json!({
            "schemaVersion":"os-lab-program/1","programId":format!("p{i}"),"name":"test","parameters":[],"blocks":blocks
        })).collect::<Vec<_>>(),
        "threads": arrivals.iter().enumerate().map(|(i, arrival)| json!({
            "threadId":format!("m:t{i}"),"programId":format!("p{i}"),"arrival":arrival.to_string(),"parameters":{}
        })).collect::<Vec<_>>()
    });
    Workload::decode(&serde_json::to_vec(&value).unwrap()).unwrap()
}

fn compute(n: u64) -> Value {
    json!({"blockId":"compute","op":"compute","duration":{"literal":n.to_string()}})
}

fn compute_named(id: &str, n: u64) -> Value {
    json!({"blockId":id,"op":"compute","duration":{"literal":n.to_string()}})
}

fn io_wait(n: u64) -> Value {
    json!({"blockId":"io","op":"ioWait","duration":{"literal":n.to_string()}})
}

fn end() -> Value {
    json!({"blockId":"end","op":"end"})
}

fn run(mut kernel: Kernel<FcfsPolicy>) -> (Vec<Value>, Vec<Interval>, State) {
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
    panic!("FCFS fixture did not terminate")
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

fn dispatches(events: &[Value]) -> Vec<(String, usize)> {
    events
        .iter()
        .filter(|event| event["kind"] == "dispatchStarted")
        .map(|event| {
            (
                event["threadId"].as_str().unwrap().to_owned(),
                event["payload"]["coreId"].as_u64().unwrap() as usize,
            )
        })
        .collect()
}

#[test]
fn policy_selects_fifo_head_without_mutating_queue() {
    let queue = VecDeque::from([4, 1, 9]);
    let before = queue.clone();
    let selected = FcfsPolicy.select(&queue);
    assert_eq!(selected, Some(4));
    assert_eq!(queue, before);
    assert_eq!(FcfsPolicy.configuration(), crate::input::PolicyConfig::Fcfs);
    assert_eq!(FcfsPolicy.quantum(), None);
}

#[test]
fn c01_simultaneous_arrivals_run_in_declaration_order_deterministically() {
    let input = workload(
        vec![
            vec![compute(2), end()],
            vec![compute(1), end()],
            vec![compute(1), end()],
        ],
        &[0, 0, 0],
        1,
        0,
    );
    let first = run(Kernel::new(input.clone(), FcfsPolicy).unwrap());
    let second = run(Kernel::new(input, FcfsPolicy).unwrap());
    assert_eq!(first, second);
    assert_eq!(first.2.tick, 4);
    assert_eq!(
        dispatches(&first.0),
        vec![("m:t0".into(), 0), ("m:t1".into(), 0), ("m:t2".into(), 0)]
    );
    assert_eq!(
        core_intervals(&first.1, 0),
        vec![
            (0, 2, CoreStatus::Running, Some("m:t0".into())),
            (2, 3, CoreStatus::Running, Some("m:t1".into())),
            (3, 4, CoreStatus::Running, Some("m:t2".into()))
        ]
    );
}

#[test]
fn c03_dispatch_overhead_is_distinct_from_ready_waiting() {
    let (_, intervals, state) = run(Kernel::new(
        workload(
            vec![vec![compute(2), end()], vec![compute(1), end()]],
            &[0, 1],
            1,
            1,
        ),
        FcfsPolicy,
    )
    .unwrap());
    assert_eq!(state.tick, 5);
    assert_eq!(
        core_intervals(&intervals, 0),
        vec![
            (0, 1, CoreStatus::Dispatching, Some("m:t0".into())),
            (1, 3, CoreStatus::Running, Some("m:t0".into())),
            (3, 4, CoreStatus::Dispatching, Some("m:t1".into())),
            (4, 5, CoreStatus::Running, Some("m:t1".into()))
        ]
    );
    assert_eq!(
        thread_intervals(&intervals, "m:t1"),
        vec![
            (1, 3, ThreadStatus::Ready),
            (3, 4, ThreadStatus::Dispatching),
            (4, 5, ThreadStatus::Running)
        ]
    );
}

#[test]
fn c05_delayed_arrival_preserves_idle_ledgers_and_uses_lowest_core() {
    let (events, intervals, state) = run(Kernel::new(
        workload(vec![vec![compute(2), end()]], &[2], 2, 1),
        FcfsPolicy,
    )
    .unwrap());
    assert_eq!(state.tick, 5);
    assert_eq!(dispatches(&events), vec![("m:t0".into(), 0)]);
    assert_eq!(
        core_intervals(&intervals, 0),
        vec![
            (0, 2, CoreStatus::Idle, None),
            (2, 3, CoreStatus::Dispatching, Some("m:t0".into())),
            (3, 5, CoreStatus::Running, Some("m:t0".into()))
        ]
    );
    assert_eq!(
        core_intervals(&intervals, 1),
        vec![(0, 5, CoreStatus::Idle, None)]
    );
}

#[test]
fn c06_two_core_dispatch_is_stable() {
    let (events, intervals, state) = run(Kernel::new(
        workload(
            vec![
                vec![compute(2), end()],
                vec![compute(2), end()],
                vec![compute(2), end()],
            ],
            &[0, 0, 0],
            2,
            1,
        ),
        FcfsPolicy,
    )
    .unwrap());
    assert_eq!(state.tick, 6);
    assert_eq!(
        dispatches(&events),
        vec![("m:t0".into(), 0), ("m:t1".into(), 1), ("m:t2".into(), 0)]
    );
    assert_eq!(
        core_intervals(&intervals, 0),
        vec![
            (0, 1, CoreStatus::Dispatching, Some("m:t0".into())),
            (1, 3, CoreStatus::Running, Some("m:t0".into())),
            (3, 4, CoreStatus::Dispatching, Some("m:t2".into())),
            (4, 6, CoreStatus::Running, Some("m:t2".into()))
        ]
    );
    assert_eq!(
        core_intervals(&intervals, 1),
        vec![
            (0, 1, CoreStatus::Dispatching, Some("m:t1".into())),
            (1, 3, CoreStatus::Running, Some("m:t1".into())),
            (3, 6, CoreStatus::Idle, None)
        ]
    );
}

#[test]
fn c07_yield_requeues_at_fifo_tail() {
    let (events, intervals, state) = run(Kernel::new(
        workload(
            vec![
                vec![
                    compute_named("first", 1),
                    json!({"blockId":"yield","op":"yield"}),
                    compute_named("second", 1),
                    end(),
                ],
                vec![compute(2), end()],
            ],
            &[0, 0],
            1,
            0,
        ),
        FcfsPolicy,
    )
    .unwrap());
    assert_eq!(state.tick, 4);
    assert_eq!(
        dispatches(&events),
        vec![("m:t0".into(), 0), ("m:t1".into(), 0), ("m:t0".into(), 0)]
    );
    assert_eq!(
        core_intervals(&intervals, 0),
        vec![
            (0, 1, CoreStatus::Running, Some("m:t0".into())),
            (1, 3, CoreStatus::Running, Some("m:t1".into())),
            (3, 4, CoreStatus::Running, Some("m:t0".into()))
        ]
    );
    assert_eq!(
        thread_intervals(&intervals, "m:t0"),
        vec![
            (0, 1, ThreadStatus::Running),
            (1, 3, ThreadStatus::Ready),
            (3, 4, ThreadStatus::Running)
        ]
    );
}

#[test]
fn c08_immediate_end_reaches_same_tick_dispatch_fixpoint() {
    let (events, intervals, state) = run(Kernel::new(
        workload(vec![vec![end()], vec![compute(1), end()]], &[0, 0], 1, 0),
        FcfsPolicy,
    )
    .unwrap());
    assert_eq!(state.tick, 1);
    assert_eq!(state.status, RunStatus::Completed);
    assert_eq!(
        dispatches(&events),
        vec![("m:t0".into(), 0), ("m:t1".into(), 0)]
    );
    assert!(events.iter().any(|event| {
        event["kind"] == "terminated" && event["threadId"] == "m:t0" && event["tick"] == "0"
    }));
    assert_eq!(
        core_intervals(&intervals, 0),
        vec![(0, 1, CoreStatus::Running, Some("m:t1".into()))]
    );
    assert!(thread_intervals(&intervals, "m:t0").is_empty());
}

#[test]
fn c10_no_compute_program_blocks_then_completes_without_running_interval() {
    let (events, intervals, state) = run(Kernel::new(
        workload(vec![vec![io_wait(1), end()]], &[0], 1, 0),
        FcfsPolicy,
    )
    .unwrap());
    assert_eq!(state.tick, 1);
    assert_eq!(state.status, RunStatus::Completed);
    assert_eq!(dispatches(&events).len(), 2);
    assert!(events.iter().any(|event| {
        event["kind"] == "terminated" && event["threadId"] == "m:t0" && event["tick"] == "1"
    }));
    assert_eq!(
        thread_intervals(&intervals, "m:t0"),
        vec![(0, 1, ThreadStatus::Blocked)]
    );
    assert_eq!(
        core_intervals(&intervals, 0),
        vec![(0, 1, CoreStatus::Idle, None)]
    );
    assert!(!intervals.iter().any(|interval| matches!(
        &interval.owner,
        IntervalOwner::Thread {
            thread_id,
            status: ThreadStatus::Running
        } if thread_id == "m:t0"
    )));
}

#[test]
fn constructor_rejects_persisted_round_robin_for_fcfs() {
    let mut value = serde_json::to_value(workload(vec![vec![end()]], &[0], 1, 0).source()).unwrap();
    value["policy"] = json!({"kind":"roundRobin","quantum":"1"});
    let input = Workload::decode(&serde_json::to_vec(&value).unwrap()).unwrap();
    assert!(Kernel::new(input, FcfsPolicy).is_err());
}
