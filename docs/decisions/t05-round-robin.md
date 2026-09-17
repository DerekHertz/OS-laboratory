# T05 round-robin policy decision

Date: 2026-09-14. Contract: `sched.m1/revision-1`. Dependency: integrated T04 `9083514`; authoritative event kernel remains T03.

## Decision

`RoundRobinPolicy` stores a `NonZeroU64` useful-service quantum. Its production `Policy` implementation returns the same value in both `PolicyConfig::RoundRobin` and `Policy::quantum`, and returns a copied index from `ready.front()` without mutating kernel ownership. Requiring `NonZeroU64` makes an invalid zero-quantum policy unrepresentable after construction.

The kernel remains solely responsible for useful-time accounting, expiration tokens, completion-before-expiration ordering, stale-token rejection, ready-tail insertion, fresh grants on dispatch, core scans, switch overhead, lifecycle ownership, and fatal cleanup. T05 does not duplicate or modify those mechanisms.

## Contract mapping

- `content/concepts/scheduling-m1.md` defines a quantum as useful service only, grants a fresh full quantum on every dispatch, and puts unfinished expired work behind phase-2 arrivals and I/O completions.
- C02 independently fixes completion at a quantum boundary ahead of stale expiration; C04 fixes I/O wakeup ahead of same-tick expiration requeue.
- The focused suite also derives bounded cases for explicit yield, blocked-grant reset, nonzero dispatch overhead, and two-core FIFO rotation directly from the accepted phase and lifecycle rules. C06/C07/C10 supply applicable stable-core, yield-tail, and no-running-while-blocked invariants, without relabeling their FCFS-specific timelines as RR fixtures.
- The existing kernel constructor checks both persisted policy identity and exact quantum, so an RR policy cannot silently run FCFS or a differently configured RR workload.

## Scope boundary

No kernel ordering, schema, transport, metric aggregation, trace retention, worker, Wasm, or UI code changes are part of T05. Finite focused cases are bounded evidence, not a proof for all workloads. Independent semantic review and protected integration remain separate gates.
