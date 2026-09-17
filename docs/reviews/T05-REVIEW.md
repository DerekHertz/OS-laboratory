# T05 round-robin independent review

Review date: 2026-09-17. Contract: `sched.m1/revision-1`.

## Verdict

**ACCEPT** exact candidate `ba8fa68cef0738535abe7ef4b3f4e43740620df2`, descended from task-packet commit `091de7f` and integrated T04 baseline `90835140d92ed271009435f16d088e033c11223c`.

I found no round-robin contract violation or integration-blocking defect. The production change is confined to the round-robin module, its focused tests, the minimal module export, and T05 decision/verification records. It does not change the authoritative T03 kernel, input validation, schema, transport, metrics, adapters, or UI.

## Independent semantic review

`RoundRobinPolicy` is the required production implementation of the existing kernel `Policy` seam. It stores an exact positive `NonZeroU64` quantum, returns the matching `PolicyConfig::RoundRobin`, exposes the same quantum through `Policy::quantum`, and returns a copied `ready.front()` without mutating the kernel-owned queue. The unchanged kernel verifies persisted configuration and quantum agreement before a run, rejects a non-head policy choice, removes the selected head itself, grants a fresh quantum on each transition to running, and remains solely responsible for useful-time accounting, event phases, generation tokens, lifecycle transitions, queue-tail insertion, stable core scans, events, and raw intervals.

| Contract behavior | Independently expected result | Reviewed evidence at `ba8fa68` |
| --- | --- | --- |
| Policy seam | Exact positive persisted quantum; same runtime quantum; oldest global FIFO entry; no policy-side queue mutation. | Direct policy test asserts configuration, quantum 3, selection of 4 from `[4,1,9]`, and an unchanged queue. The constructor type makes a zero quantum unrepresentable. |
| C02 completion/quantum tie | A two-tick burst under quantum 2 completes in phase 1; its phase-3 expiration is stale; B then runs; final tick 3. | No `quantumExpired` event is applied, dispatches are A at 0 and B at 2, exact useful intervals are `[0,2)` and `[2,3)`, and the terminal state is completed at tick 3. Two complete executions compare equal in events, intervals, and state. |
| C04 wakeup before expiration | A phase-2 I/O completion at tick 3 enters the ready queue before B's phase-3 expiration requeue. | Tick-3 `ioCompleted` precedes `quantumExpired`; dispatch order is A/B/A/B and exact core/thread intervals match the worked case through final tick 6. |
| Yield resets a grant | Yield releases immediately, appends behind older ready work, and discards unused quantum. | Dispatch order A/B/A/A, one later expiration, and exact useful intervals show B precedes yielded A and A receives a fresh grant. |
| Blocking resets a grant | A blocked thread owns no core and cannot dispatch before wakeup; its later dispatch receives a full quantum. | A is blocked on `[1,2)`, B runs during that interval, A next dispatches at tick 2, and its only applied expiration is at tick 4 after two fresh useful ticks. |
| Dispatch overhead | Switch cost is separate from useful service and does not consume quantum. | With quantum 1 and switch cost 1, three dispatching intervals and three useful intervals remain distinct; only unfinished A expires and the final tick is 6. |
| Stable multicore RR | Simultaneous expirations append in stable core order and ascending-core scans consume the FIFO deterministically. | Dispatches are A@0/B@1, then C@0/A@1, then B@0; both exact core ledgers and two expirations match the independently derived rotation through tick 3. |
| Persisted-policy mismatch | RR must reject an FCFS workload and a workload with a different RR quantum. | Both mismatches are rejected through the real kernel constructor guards. |

The tests exercise decoded workloads and the production policy through the real T03 kernel rather than duplicating scheduler mechanics. Their expectations agree with the accepted scheduling contract and independently worked C02/C04 cases. The additional yield, block, overhead, and multicore expectations follow directly from the contract's lifecycle, quantum, phase, and stable-core rules. The decision record accurately preserves the scope boundary, and the verification record supplies falsifiable failure claims, independently calculated expected outcomes, observed results, and a meaningful restored quantum negative control.

## Exact checks and limits

The following checks were run against an isolated archive of exact commit `ba8fa68cef0738535abe7ef4b3f4e43740620df2`:

- `cargo fmt --all -- --check` exited 0.
- `cargo clippy -p sim-core --all-targets --locked -- -D warnings` exited 0.
- `cargo test -p sim-core round_robin --locked` exited 0: 9 passed, 0 failed, 0 ignored, 26 filtered out. Eight tests belong to T05; one pre-existing FCFS constructor test matches the filter name.
- `git diff-tree --check ba8fa68^ ba8fa68` exited 0.
- `git merge-base --is-ancestor 9083514 ba8fa68` exited 0.

No optional targeted probe was run because contract inspection and the focused real-kernel cases left no concrete unresolved concern. I did not rerun the author's recorded mutation. These are bounded finite-case results: T06 metrics/retention, native/Wasm equivalence, browser behavior, remote CI, push, and integration are outside this verdict.
