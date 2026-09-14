# T04 FCFS independent review

## Verdict

**ACCEPT** candidate `71604d26fc346e50981506ff05bd54c29742e002` and its T04 ancestry from checkpoint `a7b9206` against integrated T03 baseline `1e1681d`.

I found no FCFS contract violation or integration-blocking defect. The production change is confined to the FCFS module, its focused tests, and the minimal module export; no T03 kernel ordering, lifecycle, input, schema, or transport source changed.

## Independent semantic review

`FcfsPolicy` is the required zero-state implementation of the existing T03 `Policy` seam: `configuration()` returns `PolicyConfig::Fcfs`, `quantum()` returns `None`, and `select()` returns a copied `ready.front()` without mutating the kernel-owned queue. The unchanged kernel verifies persisted-policy/quantum agreement, invokes the policy only for a nonempty ready queue and idle core, rejects a non-head choice, removes the head itself, and owns dispatch scans, lifecycle changes, queue-tail insertion, events, and raw intervals. Consequently arrivals and wakeups cannot preempt an FCFS possession; only yield, I/O, end, or a fatal error releases the core.

| Contract/case | Independently expected result | Reviewed evidence at `71604d2` |
| --- | --- | --- |
| Policy seam | Persisted FCFS, no quantum, oldest global FIFO entry, no policy-side mutation. | Direct policy test asserts all four properties; implementation is exactly `ready.front().copied()` with no state. |
| C01 | Declaration-order A/B/C, useful core intervals `[0,2)`, `[2,3)`, `[3,4)`, final tick 4, deterministic repeat. | Dispatch order and exact core intervals match; two complete executions compare equal in events, intervals, and terminal state. |
| C03 | Dispatch overhead remains separate from ready waiting; B is ready `[1,3)`, dispatching `[3,4)`, running `[4,5)`. | Exact core ledger and B thread intervals match through final tick 5. A's uninterrupted `[1,3)` service also demonstrates no arrival preemption. |
| C05 | Pre-arrival idle time is retained and the sole delayed thread uses core 0. | Exact core 0 and core 1 ledgers, dispatch assignment, and final tick 5 match. |
| C06 | Ascending-core stable assignment A/0, B/1, then C/0. | Dispatch sequence and both exact core ledgers match through final tick 6. |
| C07 | Explicit yield appends A behind the older waiter B. | Dispatch sequence A/B/A, exact useful intervals, and A's ready interval `[1,3)` match. |
| C08 | Zero-cost `end` releases and redispatches in the same phase-4 fixpoint. | A termination is observed at tick 0, B is the second same-tick dispatch, only B owns useful `[0,1)`, and the run completes at tick 1. |
| C10 | No-compute work blocks `[0,1)`, leaves the core idle, then terminates without a useful interval. | Exact blocked and idle intervals, two dispatches, tick-1 termination, completed state, and absence of an A running interval match. |
| Mismatched input | `Kernel<FcfsPolicy>` must reject persisted RR configuration. | Focused constructor test observes rejection through the existing T03 guard. |

The test expectations agree with the independently worked `tests/reference/scheduling/README.md` and `cases.json`; they exercise the real decoded workload and T03 kernel rather than a copied scheduler. Ready waiting and dispatch overhead are evidenced as distinct raw thread states, while metrics aggregation remains correctly deferred to T06. The decision record accurately preserves the scope boundary. The verification record states falsifiable failure claims, independent expectations, observed results, and correction responses; its front-to-back selection mutation is meaningful and was recorded as failing before restoration.

## Exact checks and limits

- `cargo test -p sim-core fcfs --locked` (using `C:\Users\Derek\.cargo\bin\cargo.exe`) exited 0: 9 passed, 0 failed, 0 ignored, 18 filtered out.
- `cargo fmt --all -- --check` exited 0.
- `cargo clippy -p sim-core --all-targets --locked -- -D warnings` exited 0.
- `git diff --check 1e1681d..71604d2` exited 0.
- The candidate ancestry check `git merge-base --is-ancestor a7b9206 71604d2` exited 0. The only post-checkpoint commit is `71604d2`; the complete T04 source delta from `1e1681d` adds `fcfs.rs`, `fcfs_tests.rs`, and the single `pub mod fcfs` export.

No optional targeted probe was run because code inspection and the focused real-kernel cases left no concrete unresolved concern. I did not rerun the author's recorded mutation. These are bounded finite-case results; T05 round-robin, T06 metrics/retention, native/Wasm equivalence, browser behavior, remote CI, push, and integration are outside this verdict.
