use super::*;
use std::collections::BTreeSet;
use std::num::NonZeroU64;

/// Explicitly test-only driver of the kernel's FIFO/quantum mechanisms. Passing
/// these tests does not claim a production T04/T05 policy is implemented.
struct TestPolicy(PolicyConfig);
impl Policy for TestPolicy {
    fn configuration(&self) -> PolicyConfig {
        self.0.clone()
    }
    fn quantum(&self) -> Option<NonZeroU64> {
        self.0.quantum()
    }
    fn select(&mut self, ready: &VecDeque<usize>) -> Option<usize> {
        ready.front().copied()
    }
}
fn compute(id: &str, n: u64) -> Value {
    json!({"blockId": id, "op": "compute", "duration": {"literal": n.to_string()}})
}
fn end(id: &str) -> Value {
    json!({"blockId": id, "op": "end"})
}
fn repeat(id: &str, n: u64, body: Vec<Value>) -> Value {
    json!({"blockId": id, "op": "repeat", "count": {"literal": n.to_string()}, "body": body})
}
fn workload(
    programs: Vec<Vec<Value>>,
    arrivals: &[u64],
    cores: usize,
    cost: u64,
    budget: u64,
    quantum: Option<u64>,
) -> Value {
    json!({"schemaVersion": "os-lab-workload/1", "applicationVersion": "app.test", "modelVersion": "sched.m1/revision-1", "engineVersion": "sim-engine.0.1.0", "randomAlgorithmVersion": "none.v1", "seed": "42",
        "machine": {"machineId": "m", "cores": cores, "switchCost": cost.to_string(), "controlOperationBudgetPerTick": budget.to_string()},
        "policy": quantum.map_or_else(|| json!({"kind":"fcfs"}), |q| json!({"kind":"roundRobin","quantum":q.to_string()})),
        "programs": programs.into_iter().enumerate().map(|(i, blocks)| json!({"schemaVersion":"os-lab-program/1","programId":format!("p{i}"),"name":"Test","parameters":[],"blocks":blocks})).collect::<Vec<_>>(),
        "threads": arrivals.iter().enumerate().map(|(i, arrival)| json!({"threadId":format!("m:t{i}"),"programId":format!("p{i}"),"arrival":arrival.to_string(),"parameters":{}})).collect::<Vec<_>>()})
}
fn kernel(v: &Value) -> Kernel<TestPolicy> {
    let workload = Workload::decode(&serde_json::to_vec(v).unwrap()).unwrap();
    let policy = TestPolicy(workload.policy().clone());
    Kernel::new(workload, policy).unwrap()
}
fn invariant(state: &State) {
    let ready: BTreeSet<_> = state.ready_queue.iter().collect();
    assert_eq!(ready.len(), state.ready_queue.len());
    assert!(ready.is_empty() || state.cores.iter().all(|c| c.status != CoreStatus::Idle));
    for t in &state.threads {
        assert_eq!(
            ready.contains(&t.thread_id),
            t.status == ThreadStatus::Ready
        );
        assert_eq!(
            t.core_id.is_some(),
            matches!(t.status, ThreadStatus::Dispatching | ThreadStatus::Running)
        );
        if let Some(core) = t.core_id {
            let c = &state.cores[core];
            assert_eq!(c.thread_id.as_ref(), Some(&t.thread_id));
            assert_eq!(
                c.status == CoreStatus::Running,
                t.status == ThreadStatus::Running
            );
        }
    }
    for c in &state.cores {
        assert_eq!(c.thread_id.is_none(), c.status == CoreStatus::Idle);
        if let Some(thread) = &c.thread_id {
            assert_eq!(
                state
                    .threads
                    .iter()
                    .find(|t| &t.thread_id == thread)
                    .unwrap()
                    .core_id,
                Some(c.core_id)
            );
        }
    }
}
fn finish(k: &mut Kernel<TestPolicy>) -> Boundary {
    let (mut events, mut intervals) = (Vec::new(), Vec::new());
    for _ in 0..1000 {
        let boundary = k.advance_tick();
        invariant(&boundary.state);
        events.extend(boundary.events);
        intervals.extend(boundary.intervals);
        if !matches!(
            boundary.state.status,
            RunStatus::Created | RunStatus::Running
        ) {
            return Boundary {
                state: boundary.state,
                events,
                intervals,
            };
        }
    }
    panic!("test failed to terminate within independent 1000-tick boundary cap")
}
fn kinds_at(events: &[Value], tick: u64) -> Vec<(String, String)> {
    let tick = tick.to_string();
    events
        .iter()
        .filter(|e| e["tick"] == tick)
        .map(|e| {
            (
                e["kind"].as_str().unwrap().to_owned(),
                e["threadId"].as_str().unwrap_or("").to_owned(),
            )
        })
        .collect()
}

#[test]
fn reference_lifecycle_intervals_with_test_only_policy() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../tests/reference/scheduling/cases.json"
    ))
    .unwrap();
    fn translate(list: &[Value], prefix: &str) -> Vec<Value> {
        list.iter()
            .enumerate()
            .map(|(i, v)| {
                let id = v
                    .get("blockId")
                    .and_then(Value::as_str)
                    .map(str::to_owned)
                    .unwrap_or_else(|| format!("{prefix}.b{i}"));
                if let Some(n) = v.get("compute") {
                    compute(&id, n.as_u64().unwrap())
                } else if let Some(n) = v.get("ioWait") {
                    json!({"blockId":id,"op":"ioWait","duration":{"literal":n.to_string()}})
                } else if let Some(r) = v.get("repeat") {
                    repeat(
                        &id,
                        r["count"].as_u64().unwrap(),
                        translate(r["body"].as_array().unwrap(), &id),
                    )
                } else {
                    json!({"blockId":id,"op":v.as_str().unwrap()})
                }
            })
            .collect()
    }
    for case in fixture["cases"].as_array().unwrap() {
        let threads = case["threads"].as_array().unwrap();
        let programs = threads
            .iter()
            .map(|t| translate(t["program"].as_array().unwrap(), t["id"].as_str().unwrap()))
            .collect();
        let arrivals = threads
            .iter()
            .map(|t| t["arrival"].as_u64().unwrap())
            .collect::<Vec<_>>();
        let v = workload(
            programs,
            &arrivals,
            case["cores"].as_u64().unwrap() as usize,
            case["switchCost"].as_u64().unwrap(),
            case["controlOperationBudgetPerTick"]
                .as_u64()
                .unwrap_or(100),
            case["policy"]["quantum"].as_u64(),
        );
        let b = finish(&mut kernel(&v));
        assert_eq!(
            b.state.tick,
            case["finalTick"].as_u64().unwrap(),
            "{}",
            case["id"]
        );
        let error = case["runOutcome"]["status"] == "error";
        assert_eq!(
            b.state.status,
            if error {
                RunStatus::Error
            } else {
                RunStatus::Completed
            }
        );
        for ledger in case["coreLedgers"].as_array().unwrap() {
            let core = ledger["core"].as_u64().unwrap() as usize;
            let actual: Vec<Value> = b
                .intervals
                .iter()
                .filter_map(|i| {
                    if let IntervalOwner::Core {
                        core_id,
                        status,
                        thread_id,
                    } = &i.owner
                    {
                        if *core_id != core {
                            return None;
                        }
                        let name = thread_id
                            .as_ref()
                            .map(|id| {
                                threads
                                        [id.strip_prefix("m:t").unwrap().parse::<usize>().unwrap()]
                                        ["id"]
                                        .clone()
                            })
                            .unwrap_or(Value::Null);
                        Some(json!([
                            i.start,
                            i.end,
                            match status {
                                CoreStatus::Idle => "idle",
                                CoreStatus::Running => "useful",
                                CoreStatus::Dispatching => "overhead",
                            },
                            name
                        ]))
                    } else {
                        None
                    }
                })
                .collect();
            assert_eq!(
                json!(actual),
                ledger["intervals"],
                "{} core {core}",
                case["id"]
            );
        }
        assert_eq!(
            b,
            finish(&mut kernel(&v)),
            "repeat determinism {}",
            case["id"]
        );
    }
}

#[test]
fn repeat_charge_boundaries_and_source_identity() {
    for (count, budget, expected, block) in [
        (3, 4, RunStatus::Completed, "done"),
        (3, 3, RunStatus::Error, "done"),
        (4, 3, RunStatus::Error, "loop"),
    ] {
        let b = finish(&mut kernel(&workload(
            vec![vec![repeat("loop", count, vec![]), end("done")]],
            &[0],
            1,
            0,
            budget,
            None,
        )));
        assert_eq!(b.state.tick, 0);
        assert_eq!(b.state.status, expected);
        assert_eq!(b.state.threads[0].block_id, block);
    }
    let v = workload(
        vec![vec![
            repeat("loop", 3, vec![compute("cpu", 1)]),
            end("done"),
        ]],
        &[0],
        1,
        0,
        1,
        None,
    );
    let b = finish(&mut kernel(&v));
    assert_eq!((b.state.status, b.state.tick), (RunStatus::Completed, 3));
    assert_eq!(
        b.events
            .iter()
            .filter(|e| e["kind"] == "computeCompleted" && e["blockId"] == "cpu")
            .count(),
        3
    );
    let nested = workload(
        vec![vec![
            repeat(
                "outer",
                2,
                vec![repeat("inner", 2, vec![compute("cpu", 1)])],
            ),
            end("done"),
        ]],
        &[0],
        1,
        0,
        2,
        None,
    );
    assert_eq!(finish(&mut kernel(&nested)).state.tick, 4);
    let terminating = workload(
        vec![vec![repeat("outer", u64::MAX, vec![end("inside")])]],
        &[0],
        1,
        0,
        2,
        None,
    );
    assert_eq!(
        finish(&mut kernel(&terminating)).state.threads[0].block_id,
        "inside"
    );
}

#[test]
fn fatal_budget_is_run_wide_and_stops_later_phases() {
    let b = finish(&mut kernel(&workload(
        vec![vec![end("a")], vec![end("b")]],
        &[0, 0],
        2,
        0,
        1,
        None,
    )));
    assert_eq!(
        b.state.threads.iter().map(|t| t.status).collect::<Vec<_>>(),
        [ThreadStatus::Terminated, ThreadStatus::Error]
    );
    assert_eq!(b.state.error.unwrap().block_id.as_deref(), Some("b"));
    let mut k = kernel(&workload(
        vec![
            vec![compute("cpu", 1), repeat("loop", 2, vec![]), end("a")],
            vec![end("b")],
        ],
        &[0, 1],
        2,
        0,
        1,
        None,
    ));
    let b = finish(&mut k);
    assert_eq!((b.state.status, b.state.tick), (RunStatus::Error, 1));
    assert!(
        !b.events
            .iter()
            .any(|e| e["kind"] == "arrival" && e["threadId"] == "m:t1")
    );
    assert!(k.pending.is_empty() && k.ready.is_empty());
    assert!(k.advance_tick().events.is_empty());
    assert_eq!(b.state.threads[1].status, ThreadStatus::Censored);
    let cycle = workload(
        vec![vec![
            repeat("loop", 100, vec![json!({"blockId":"give","op":"yield"})]),
            end("done"),
        ]],
        &[0],
        1,
        0,
        5,
        None,
    );
    let mut k = kernel(&cycle);
    let b = finish(&mut k);
    assert_eq!(k.controls, 5);
    assert_eq!(b.state.error.unwrap().block_id.as_deref(), Some("give"));
}

#[test]
fn phase_order_quantum_instruction_match_and_stale_tokens() {
    let b = finish(&mut kernel(&workload(
        vec![vec![compute("cpu", 5), end("done")]],
        &[0],
        1,
        0,
        100,
        Some(2),
    )));
    assert_eq!(b.state.tick, 5);
    assert_eq!(
        b.events
            .iter()
            .filter(|e| e["kind"] == "computeCompleted")
            .count(),
        1
    );
    assert_eq!(
        b.events
            .iter()
            .filter(|e| e["kind"] == "quantumExpired")
            .count(),
        2
    );
    assert_eq!(
        b.events
            .iter()
            .find(|e| e["kind"] == "computeCompleted")
            .unwrap()["payload"]["generation"],
        "3"
    );
    let v = workload(
        vec![
            vec![compute("a1", 2), compute("a2", 1), end("a")],
            vec![compute("b1", 1), end("b")],
        ],
        &[0, 2],
        1,
        0,
        100,
        Some(2),
    );
    let b = finish(&mut kernel(&v));
    assert_eq!(
        kinds_at(&b.events, 2),
        vec![
            ("computeCompleted".into(), "m:t0".into()),
            ("computeStarted".into(), "m:t0".into()),
            ("arrival".into(), "m:t1".into()),
            ("quantumExpired".into(), "m:t0".into()),
            ("dispatchStarted".into(), "m:t1".into()),
            ("dispatchCompleted".into(), "m:t1".into()),
            ("computeStarted".into(), "m:t1".into())
        ]
    );
    let v = workload(
        vec![vec![
            compute("cpu", 1),
            json!({"blockId":"io","op":"ioWait","duration":{"literal":"1"}}),
            compute("cpu2", 1),
            end("done"),
        ]],
        &[0],
        1,
        0,
        100,
        Some(2),
    );
    let b = finish(&mut kernel(&v));
    assert_eq!(b.state.tick, 3);
    assert!(!b.events.iter().any(|e| e["kind"] == "quantumExpired"));
    assert_eq!(
        b.events
            .iter()
            .find(|e| e["kind"] == "ioCompleted")
            .unwrap()["blockId"],
        "io"
    );
}

#[test]
fn phase_four_scans_finish_ascending_cores() {
    let b = finish(&mut kernel(&workload(
        vec![
            vec![end("a")],
            vec![compute("b", 1), end("be")],
            vec![compute("c", 1), end("ce")],
        ],
        &[0, 0, 0],
        2,
        0,
        100,
        None,
    )));
    let starts: Vec<_> = b
        .events
        .iter()
        .filter(|e| e["kind"] == "dispatchStarted")
        .map(|e| (e["threadId"].clone(), e["payload"]["coreId"].clone()))
        .collect();
    assert_eq!(
        starts,
        vec![
            (json!("m:t0"), json!(0)),
            (json!("m:t1"), json!(1)),
            (json!("m:t2"), json!(0))
        ]
    );
    // Equal-tick completion order is original insertion order, not a new core sort.
    let completed: Vec<_> = b
        .events
        .iter()
        .filter(|e| e["kind"] == "computeCompleted")
        .map(|e| e["payload"]["coreId"].clone())
        .collect();
    assert_eq!(completed, vec![json!(1), json!(0)]);
}

#[test]
fn arithmetic_failures_close_active_and_future_ownership() {
    for (cost, blocks) in [
        (1, vec![end("end")]),
        (0, vec![compute("cpu", 1), end("end")]),
        (
            0,
            vec![
                json!({"blockId":"io","op":"ioWait","duration":{"literal":"1"}}),
                end("end"),
            ],
        ),
    ] {
        let b = finish(&mut kernel(&workload(
            vec![blocks],
            &[u64::MAX],
            1,
            cost,
            100,
            None,
        )));
        assert_eq!(b.state.tick, u64::MAX);
        assert_eq!(b.state.error.unwrap().code, ErrorCode::ArithmeticOverflow);
        assert_eq!(
            b.intervals
                .iter()
                .filter_map(
                    |i| matches!(i.owner, IntervalOwner::Core { .. }).then_some(i.end - i.start)
                )
                .sum::<u64>(),
            u64::MAX
        );
        assert!(!b.events.iter().any(|e| e["kind"] == "terminated"));
    }
    let v = workload(
        vec![
            vec![compute("a", 1), repeat("loop", 2, vec![]), end("ae")],
            vec![compute("b", 5), end("be")],
            vec![end("ce")],
        ],
        &[0, 0, 2],
        2,
        0,
        1,
        None,
    );
    let b = finish(&mut kernel(&v));
    for core in 0..2 {
        let durations: u64 = b
            .intervals
            .iter()
            .filter_map(|i| {
                if matches!(i.owner, IntervalOwner::Core{core_id,..} if core_id==core) {
                    Some(i.end - i.start)
                } else {
                    None
                }
            })
            .sum();
        assert_eq!(durations, 1);
    }
    assert_eq!(
        b.state.threads.iter().map(|t| t.status).collect::<Vec<_>>(),
        vec![
            ThreadStatus::Error,
            ThreadStatus::Censored,
            ThreadStatus::Censored
        ]
    );
}

#[test]
fn checked_counter_overflow_and_reserved_terminal_event() {
    let v = workload(
        vec![vec![compute("cpu", 1), end("done")]],
        &[0],
        1,
        0,
        100,
        None,
    );
    for counter in ["insertion", "core", "instruction"] {
        let mut k = kernel(&v);
        match counter {
            "insertion" => k.insertion = u64::MAX,
            "core" => k.cores[0].generation = u64::MAX,
            _ => k.threads[0].instruction_generation = u64::MAX,
        }
        let b = finish(&mut k);
        assert_eq!(
            b.state.error.unwrap().code,
            ErrorCode::ArithmeticOverflow,
            "{counter}"
        );
        assert!(k.pending.is_empty());
    }
    let mut k = kernel(&v);
    k.event_sequence = u64::MAX - 1;
    let b = finish(&mut k);
    assert_eq!(b.events.len(), 1);
    assert_eq!(b.events[0]["eventSequence"], u64::MAX.to_string());
    assert_eq!(b.events[0]["kind"], "runtimeError");
    assert_eq!(k.insertion, 1); // arrival never made ready or dispatched
    assert_eq!(k.cores[0].generation, 0);
    let mut k = kernel(&workload(vec![vec![end("done")]], &[0], 1, 0, 100, None));
    k.event_sequence = u64::MAX - 5; // arrival, two dispatch events, end consume final ordinary slots
    let b = finish(&mut k);
    assert_eq!(b.state.error.unwrap().code, ErrorCode::InternalEngine);
    assert_eq!(b.state.threads[0].status, ThreadStatus::Terminated);
    assert_eq!(
        b.events.last().unwrap()["eventSequence"],
        u64::MAX.to_string()
    );
    assert!(b.events.last().unwrap().get("threadId").is_none());
}

#[test]
fn generated_ids_causal_predecessors_and_producer_artifact() {
    let mut batches = Vec::new();
    for (machine, thread) in [
        ("m".to_owned(), "m:core:0".to_owned()),
        ("m".repeat(127), format!("{}:", "m".repeat(127))),
    ] {
        let mut v = workload(
            vec![vec![compute("cpu", 1), end("end")]],
            &[0],
            1,
            0,
            100,
            None,
        );
        v["machine"]["machineId"] = json!(machine);
        v["threads"][0]["threadId"] = json!(thread);
        let b = finish(&mut kernel(&v));
        let mut previous = None;
        for (i, e) in b.events.iter().enumerate() {
            assert_eq!(e["eventSequence"], (i + 1).to_string());
            assert!(e["eventId"].as_str().unwrap().len() <= 160);
            let parents = e["causalParentIds"].as_array().unwrap();
            assert_eq!(parents, &previous.iter().cloned().collect::<Vec<Value>>());
            if let Some(entities) = e["entityIds"].as_array() {
                assert!(entities.iter().all(|id| id.as_str().unwrap().len() <= 160));
                if entities.len() == 2 {
                    assert_ne!(entities[0], entities[1]);
                }
            }
            previous = Some(e["eventId"].clone());
        }
        batches.push(
            json!({"name":format!("namespace-{}",machine.len()),"workload":v,"events":b.events}),
        );
    }
    let v = workload(
        vec![vec![
            compute("cpu", 3),
            json!({"blockId":"io","op":"ioWait","duration":{"literal":"1"}}),
            json!({"blockId":"give","op":"yield"}),
            end("done"),
        ]],
        &[0],
        1,
        1,
        100,
        Some(2),
    );
    batches.push(
        json!({"name":"all-operations","workload":v,"events":finish(&mut kernel(&v)).events}),
    );
    let v = workload(
        vec![vec![repeat("loop", 2, vec![]), end("done")]],
        &[0],
        1,
        0,
        1,
        None,
    );
    batches
        .push(json!({"name":"fatal-budget","workload":v,"events":finish(&mut kernel(&v)).events}));
    if let Some(path) = std::env::var_os("OS_LAB_KERNEL_EVENTS") {
        std::fs::write(
            path,
            serde_json::to_vec_pretty(&json!({"cases":batches})).unwrap(),
        )
        .unwrap();
    }
}

#[test]
fn insertion_preflight_rejects_before_operation_event() {
    for (cost, blocks, quantum, forbidden, slots) in [
        (1, vec![end("done")], None, "dispatchStarted", 0),
        (
            0,
            vec![compute("cpu", 1), end("done")],
            None,
            "computeStarted",
            0,
        ),
        (
            0,
            vec![compute("cpu", 2), end("done")],
            Some(2),
            "computeStarted",
            1,
        ),
        (
            0,
            vec![
                json!({"blockId":"io","op":"ioWait","duration":{"literal":"1"}}),
                end("done"),
            ],
            None,
            "ioSubmitted",
            0,
        ),
    ] {
        let mut k = kernel(&workload(vec![blocks], &[0], 1, cost, 100, quantum));
        k.insertion = u64::MAX - slots;
        let b = finish(&mut k);
        assert_eq!(b.state.error.unwrap().code, ErrorCode::ArithmeticOverflow);
        assert!(
            !b.events.iter().any(|e| e["kind"] == forbidden),
            "{forbidden}"
        );
        assert_eq!(k.insertion, u64::MAX - slots);
    }
}

#[test]
fn earliest_deadline_near_u64_limit() {
    for (arrival, quantum, duration, tick, status, expirations) in [
        (
            u64::MAX - 5,
            u64::MAX,
            2,
            u64::MAX - 3,
            RunStatus::Completed,
            0,
        ),
        (u64::MAX - 5, 2, 10, u64::MAX - 1, RunStatus::Error, 2),
        (u64::MAX - 2, 2, 2, u64::MAX, RunStatus::Completed, 0),
    ] {
        let b = finish(&mut kernel(&workload(
            vec![vec![compute("cpu", duration), end("done")]],
            &[arrival],
            1,
            0,
            100,
            Some(quantum),
        )));
        assert_eq!((b.state.tick, b.state.status), (tick, status));
        assert_eq!(
            b.events
                .iter()
                .filter(|e| e["kind"] == "quantumExpired")
                .count(),
            expirations
        );
    }
    // Equality reaches another compute with zero grant at MAX; phase 3 still
    // expires it before a fresh positive grant makes the real next deadline fail.
    let b = finish(&mut kernel(&workload(
        vec![vec![compute("a", 2), compute("b", 1), end("done")]],
        &[u64::MAX - 2],
        1,
        0,
        100,
        Some(2),
    )));
    assert_eq!(b.state.tick, u64::MAX);
    assert_eq!(
        b.events
            .iter()
            .filter(|e| e["kind"] == "quantumExpired")
            .count(),
        1
    );
}

#[test]
fn stale_tokens_do_not_advance_time_or_mutate_current_occurrence() {
    let v = workload(
        vec![vec![compute("cpu", 5), end("done")]],
        &[0],
        1,
        0,
        100,
        Some(2),
    );
    let mut clean = kernel(&v);
    let mut injected = kernel(&v);
    assert_eq!(clean.advance_tick(), injected.advance_tick());
    let old = injected.pending.peek().unwrap().0;
    assert_eq!(clean.advance_tick(), injected.advance_tick()); // generation 2 now owns core
    for kind in [Kind::Compute, Kind::Expire, Kind::Switch] {
        injected.insertion += 1;
        injected.pending.push(Reverse(Pending {
            tick: 3,
            phase: kind.phase(),
            sequence: injected.insertion,
            kind,
            ..old
        }));
    }
    injected.insertion += 1;
    injected.pending.push(Reverse(Pending {
        tick: u64::MAX,
        sequence: injected.insertion,
        ..old
    }));
    assert_eq!(finish(&mut clean), finish(&mut injected));
}

#[test]
fn policy_contract_rejects_mismatch_and_invalid_selection() {
    let v = workload(vec![vec![end("done")]], &[0], 1, 0, 100, None);
    let w = Workload::decode(&serde_json::to_vec(&v).unwrap()).unwrap();
    assert!(
        Kernel::new(
            w.clone(),
            TestPolicy(PolicyConfig::RoundRobin { quantum: 1 })
        )
        .is_err()
    );
    struct Bad;
    impl Policy for Bad {
        fn configuration(&self) -> PolicyConfig {
            PolicyConfig::Fcfs
        }
        fn quantum(&self) -> Option<NonZeroU64> {
            None
        }
        fn select(&mut self, _: &VecDeque<usize>) -> Option<usize> {
            Some(99)
        }
    }
    let mut k = Kernel::new(w, Bad).unwrap();
    let b = k.advance_tick();
    assert_eq!(b.state.error.unwrap().code, ErrorCode::InternalEngine);
    assert_eq!(b.state.threads[0].status, ThreadStatus::Censored);
}
