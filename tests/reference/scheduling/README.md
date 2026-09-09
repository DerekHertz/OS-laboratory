# Independently worked M1 scheduling cases

These cases were calculated from `sched.m1` before the event kernel or policies exist. Intervals are half-open. Thread metrics are ordered as response, turnaround, ready wait, dispatch overhead, blocked time, useful time. They are normative semantic expectations; `cases.json` is a test-only encoding pending T02.

## C01 — simultaneous FCFS arrivals

One core, FCFS, switch cost 0. A, B, C are declared in that order at tick 0 with compute lengths 2, 1, 1.

- Tick 0 phase 2 appends A, B, C. Phase 4 dispatches A immediately.
- A uses `[0,2)` and terminates at 2; B uses `[2,3)` and terminates at 3; C uses `[3,4)` and terminates at 4.
- A = `(0,2,0,0,0,2)`; B = `(2,3,2,0,0,1)`; C = `(3,4,3,0,0,1)`.
- Core 0 over `[0,4)`: useful 4, overhead 0, idle 0. Conservation: each turnaround is its ready wait plus useful time.

This fixes declaration order as the simultaneous-arrival tie-break.

## C02 — RR compute completion equals quantum

One core, RR quantum 2, switch cost 0. A (`compute 2; end`) then B (`compute 1; end`) arrive at 0.

- A uses `[0,2)`. At tick 2 its compute boundary and `end` occur in phase 1, so its phase-3 expiration is stale; A is never requeued.
- B uses `[2,3)` and terminates.
- A = `(0,2,0,0,0,2)`; B = `(2,3,2,0,0,1)`.
- Core 0 over `[0,3)`: useful 3, overhead 0, idle 0.

This distinguishes “burst completed” from “unfinished thread expired.”

## C03 — nonzero dispatch cost, including from idle

One core, FCFS, switch cost 1. A arrives at 0 with compute 2; B arrives at 1 with compute 1.

- Tick 0 reserves A: overhead `[0,1)`, useful `[1,3)`, termination at 3.
- B waits ready from `[1,3)`, then has overhead `[3,4)`, useful `[4,5)`, termination at 5.
- A = `(1,3,0,1,0,2)`; B = `(3,4,2,1,0,1)`.
- Core 0 over `[0,5)`: useful 3, overhead 2, idle 0.

For B, response is `4−1=3`; dispatching time is not ready wait.

## C04 — I/O wakeup precedes same-tick RR requeue

One core, RR quantum 2, switch cost 0. At tick 0, A (`compute 1; io_wait 2; compute 1; end`) is declared before B (`compute 4; end`).

- A uses `[0,1)`, submits I/O at 1, and is blocked `[1,3)`.
- B uses `[1,3)`. At tick 3, A's phase-2 completion appends A before B's phase-3 expiration appends B.
- A therefore uses `[3,4)` and terminates; B resumes for `[4,6)` and terminates at its compute/quantum boundary without requeue.
- A = `(0,4,0,0,2,2)`; B = `(1,6,2,0,0,4)`. B's ready intervals are `[0,1)` and `[3,4)`.
- Core 0 over `[0,6)`: useful 6, overhead 0, idle 0.

This fixes phase-2 readiness ahead of phase-3 expiration.

## C05 — idle cores and delayed arrival

Two cores, FCFS, switch cost 1. A arrives at tick 2 with compute 2.

- Both cores are idle `[0,2)`. At tick 2, ascending core order reserves A on core 0.
- Core 0 has overhead `[2,3)`, useful `[3,5)`; A terminates at 5. Core 1 stays idle `[0,5)`.
- A = `(1,3,0,1,0,2)`.
- Over `[0,5)`, core 0 = useful 2, overhead 1, idle 2; core 1 = useful 0, overhead 0, idle 5.

The run window begins at zero even before the first arrival.

## C06 — stable two-core dispatch and completion

Two cores, FCFS, switch cost 1. A, B, C arrive in that order at 0; each computes for 2 then ends.

- Tick 0: core 0 reserves A and core 1 reserves B. Both have overhead `[0,1)` and useful `[1,3)`.
- Tick 3 phase 1 handles A/core 0 completion before B/core 1. Both terminate. In phase 4, core 0 reserves queue head C; core 1 finds no ready work.
- C has overhead `[3,4)`, useful `[4,6)`, and terminates at 6.
- A = `(1,3,0,1,0,2)`; B = `(1,3,0,1,0,2)`; C = `(4,6,3,1,0,2)`.
- Over `[0,6)`, core 0 = useful 4, overhead 2, idle 0; core 1 = useful 2, overhead 1, idle 3.

This fixes ascending core IDs for both simultaneous completion generation and dispatch.

## C07 — explicit yield goes to the queue tail

One core, FCFS, switch cost 0. At tick 0, A (`compute 1; yield; compute 1; end`) is declared before B (`compute 2; end`).

- A uses `[0,1)`, yields at tick 1, and appends behind waiting B.
- B uses `[1,3)` and terminates. A waits `[1,3)`, resumes on `[3,4)`, and terminates.
- A = `(0,4,2,0,0,2)`; B = `(1,3,1,0,0,2)`.
- Core 0 over `[0,4)`: useful 4, overhead 0, idle 0.

FCFS is nonpreemptive, but explicit yield relinquishes its possession.

## Cross-case checks

For every completed thread, the six component ledger satisfies `turnaround = ready + overhead + blocked + useful`. For every core, `useful + overhead + idle = final_tick`. The JSON repeats these values explicitly so tests can report which independently derived expectation diverged.
