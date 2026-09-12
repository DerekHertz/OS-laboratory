# T03 independent acceptance preparation

Date: 2026-09-11. Agent: `t03_acceptance`, GPT-5.6 Sol / xhigh, fresh context. Read-only audit; no implementation edits. Root preserved this report from the final agent response. This prepares acceptance and is not final code review.

The committed repeat/generated-ID corrections (`1f1adb1`, `323af26`) resolve the identified contradictions. The recorded schema mutation detects the original generated-ID defect. Canonical metadata and reserved counter rules were subsequently accepted by root and recorded in the task packet; the kernel decision document must specify them as well.

## Acceptance checklist

- Production FCFS/RR remain T04/T05. Parsing policy configuration and exposing hooks is T03; test policy drivers must be explicit.
- Preserve build_probe and keep browser, worker, metrics, retention, checkpoint and replay behavior outside T03.
- At stable boundaries: each ready thread occurs once in queue; dispatching/running owns exactly one core; other statuses own none; no ready work coexists with an idle core.
- Complete canonical streams and terminal state repeat identically. Source blocks, entity ordering, parents and generation values follow the agreed rules.
- Check tick/insertion/generation/event counters before mutation. Reserve canonical MAX for the sole terminal error. Attribute arithmeticOverflow only when a thread/block exists; otherwise use global internalEngine.
- Rust program/workload fixture parity must cover canonical integers, u64MAX, compatibility versions, unknown fields, duplicate IDs, namespaces, parameter overrides, reachability, depth64/65, blocks10000/10001 and1MiB bytes. Compile sim-core for wasm32-unknown-unknown; actual browser engine invocation remains T07/T08.
- Check actual generated IDs: machine m with thread m:core:0 yields core entity m:core:0:, and full machine/u64 suffixes remain representable.

## Independently worked cases

1. One core, zero switch, compute1;ioWait2;yield;end: useful[0,1), blocked[1,3); at3 wake, dispatch, yield, redispatch, terminate. Dispatch generations1,2,3 and corresponding block sources.
2. Test quantum2, A=compute2;compute1;end, B arrival2: A completes first compute and enters second in phase1; B arrives phase2; A expires phase3; B precedes A.
3. repeat3{};end: budget4 completes tick0; budget3 consumes three iteration charges and fails at end. C09 repeat4{} budget3 instead fails at the repeat block before evaluation4.
4. repeat3{compute1};end budget1: repeat charges at ticks0,1,2, free final exit, end at tick3 using reset budget; completes3.
5. A=end, B=end, budget1 tick0: A terminates; B fails; preserve A terminated, B error, empty queue and idle cores, final0.
6. A=compute1;repeat2{};end, B arrival1, budget1: A fails in phase1 tick1; B arrival in phase2 never executes, B censored, all pending work discarded.
7. Test quantum2, compute5;end: generations1–3; old completions at5 from generations1/2 stale; generation3 completes once.
8. Test quantum2, compute1;ioWait1;compute1;end: I/O wakes phase2 tick2; old phase3 expiration is stale and cannot affect the new dispatch.
9. Inject last canonical sequence MAX-1: attempted ordinary event absent, terminal error gets MAX. Separate near-limit tests check tick/insertion/generation cannot wrap.
10. Mutate phase order, budget reset-per-dispatch, or generation check; the corresponding independently expected case must fail, then restore/rerun.

No unresolved semantic blocker remains once agreed canonical metadata/counter decisions are recorded. Fresh implementation review is still required.
