# T04 FCFS policy decision

Date: 2026-09-14. Contract: `sched.m1/revision-1`. Dependency: integrated T03 event kernel `1e1681d`.

## Decision

`FcfsPolicy` is a zero-state production implementation of the T03 `Policy` seam. It reports persisted `PolicyConfig::Fcfs`, reports no quantum, and returns a copied index from `ready.front()` without mutating the kernel-owned queue. The kernel remains solely responsible for removing the head, lifecycle ownership, phase ordering, core assignment, dispatch overhead, I/O readiness, yield requeueing, and terminal cleanup.

The constructor's existing configuration check rejects a decoded round-robin workload when paired with `FcfsPolicy`. This prevents the selected production policy from silently disagreeing with the persisted run input.

## Contract mapping

- `content/concepts/scheduling-m1.md` defines FCFS as the oldest eligible entry in one global FIFO, with no arrival/wakeup preemption and no quantum.
- Specification section 6 requires phase order, stable core traversal, dispatch cost on every dispatch, explicit-yield release, and same-tick phase-4 closure.
- The independently worked fixtures in `tests/reference/scheduling/` provide the expected cases used here: C01, C03, C05, C06, C07, C08, and C10.

The focused tests execute the production policy through `Kernel`; they do not copy scheduling logic into the policy or claim metrics aggregation. Exact core/thread intervals and dispatch/termination events are asserted where the reference case depends on them. C01 is executed twice and compares complete canonical events, intervals, and terminal state.

## Scope boundary

No kernel ordering, input schema, transport, metrics, retention, worker, Wasm, or UI code changes are part of T04. Round-robin remains T05. T04 establishes bounded FCFS evidence only; finite tests do not prove all possible workloads.
