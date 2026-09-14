use super::FcfsPolicy;
use crate::input::Workload;
use crate::kernel::{CoreStatus, IntervalOwner, Kernel, Policy, RunStatus, ThreadStatus};
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
fn end() -> Value {
    json!({"blockId":"end","op":"end"})
}
fn run(
    mut kernel: Kernel<FcfsPolicy>,
) -> (
    Vec<Value>,
    Vec<crate::kernel::Interval>,
    crate::kernel::State,
) {
    let mut events = Vec::new();
    let mut intervals = Vec::new();
    for _ in 0..100 {
        let b = kernel.advance_tick();
        events.extend(b.events);
        intervals.extend(b.intervals);
        if !matches!(b.state.status, RunStatus::Created | RunStatus::Running) {
            return (events, intervals, b.state);
        }
    }
    panic!("FCFS fixture did not terminate")
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
fn real_kernel_matches_fcfs_reference_edges_and_is_deterministic() {
    let w = workload(
        vec![vec![compute(1), end()], vec![compute(2), end()]],
        &[0, 0],
        1,
        0,
    );
    let first = run(Kernel::new(w.clone(), FcfsPolicy).unwrap());
    let second = run(Kernel::new(w, FcfsPolicy).unwrap());
    assert_eq!(first, second);
    assert_eq!(first.2.tick, 3);
    assert_eq!(
        first.2.threads.iter().map(|t| t.status).collect::<Vec<_>>(),
        vec![ThreadStatus::Terminated; 2]
    );
    let owners: Vec<_> = first
        .1
        .iter()
        .filter_map(|i| match &i.owner {
            IntervalOwner::Core {
                status: CoreStatus::Running,
                thread_id: Some(id),
                ..
            } => Some((i.start, i.end, id.clone())),
            _ => None,
        })
        .collect();
    assert_eq!(owners, vec![(0, 1, "m:t0".into()), (1, 3, "m:t1".into())]);
}

#[test]
fn dispatch_cost_yield_tail_and_immediate_end_use_kernel_phase_closure() {
    let w = workload(
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
        1,
    );
    let (events, intervals, state) = run(Kernel::new(w, FcfsPolicy).unwrap());
    assert_eq!(state.status, RunStatus::Completed);
    let useful: Vec<_> = intervals
        .iter()
        .filter_map(|i| match &i.owner {
            IntervalOwner::Core {
                status: CoreStatus::Running,
                thread_id: Some(id),
                ..
            } => Some((i.start, i.end, id.clone())),
            _ => None,
        })
        .collect();
    assert_eq!(
        useful,
        vec![
            (1, 2, "m:t0".into()),
            (3, 5, "m:t1".into()),
            (6, 7, "m:t0".into())
        ]
    );
    assert!(
        events
            .iter()
            .any(|e| e["kind"] == "threadTerminated" && e["threadId"] == "m:t1")
    );
}

#[test]
fn constructor_rejects_persisted_round_robin_for_fcfs() {
    let mut v = serde_json::to_value(workload(vec![vec![end()]], &[0], 1, 0).source()).unwrap();
    v["policy"] = json!({"kind":"roundRobin","quantum":"1"});
    let w = Workload::decode(&serde_json::to_vec(&v).unwrap()).unwrap();
    assert!(Kernel::new(w, FcfsPolicy).is_err());
}
