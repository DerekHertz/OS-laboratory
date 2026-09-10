# T01 independent semantic review result

## Reviewer metadata

- Date: 2026-09-09
- Reviewed commit: `098bb5417e871a9d28f415e4c389d59d80d186f4` on `codex/t01-scheduling-contract`. The scheduling contract and reference cases entered at author commit `aed2efa`; the later reviewed commits add verification and review-routing material.
- Actual model and reasoning effort: Codex based on GPT-5. The exact routed model ID and reasoning-effort setting are not exposed to this reviewer, so neither is guessed.
- Repository sources inspected completely: `AGENTS.md`; `OS-Laboratory-Spec.md`, including sections 6, 9, and 11; `IMPLEMENTATION-TODO.md`, including T01 and the T02-T05 gates; `docs/tasks/T01.md`; `content/concepts/scheduling-m1.md`; `tests/reference/scheduling/README.md`; `tests/reference/scheduling/cases.json`; `docs/verification/T01.md`; and `docs/reviews/T01-REVIEW-PACKET.md`.
- Primary sources inspected directly: OSTEP v1.10 chapter 7 sections 7.1-7.3 and 7.6-7.8 in <https://pages.cs.wisc.edu/~remzi/OSTEP/cpu-sched.pdf>; OSTEP v1.10 chapter 10 in <https://pages.cs.wisc.edu/~remzi/OSTEP/cpu-sched-multi.pdf>; and the supplemental homework index at <https://pages.cs.wisc.edu/~remzi/OSTEP/Homework/homework.html>.
- Independence statement: This review started from the packet and repository evidence in a fresh reviewer conversation. It did not use the author's conversation, scratch work, a production scheduler, Linux, host-language iteration behavior, or the OSTEP homework simulator as an oracle. Expected schedules were derived before comparison with the authored README and JSON.

Notation below is `R/T/W/O/B/U` for response, turnaround, ready wait, dispatch overhead, blocked time, and useful time. Intervals are half-open.

## Case recomputation

| Case | Independent transitions, queue order, and core ledger | Independent thread metrics and arithmetic | Comparison |
| --- | --- | --- | --- |
| C01 | At tick 0 phase 2, declaration order makes the ready queue `[A,B,C]`; phase 4 selects A. Core 0 is `A useful [0,2)`, `B useful [2,3)`, `C useful [3,4)`. Queue heads at dispatch boundaries are A at 0, B at 2, C at 3. Final tick 4; core totals U/O/I = `4/0/0`. | A `0/2/0/0/0/2` (`2=0+0+0+2`); B `2/3/2/0/0/1` (`3=2+0+0+1`); C `3/4/3/0/0/1` (`4=3+0+0+1`). | Exact match to README and JSON. |
| C02 | Tick 0 queue `[A,B]`; A runs `[0,2)`. At tick 2 phase 1, A's compute boundary reaches `end`, so A terminates and its phase-3 expiration is stale. Phase 4 selects B, which runs `[2,3)`. Final tick 3; core totals `3/0/0`. | A `0/2/0/0/0/2` (`2=0+0+0+2`); B `2/3/2/0/0/1` (`3=2+0+0+1`). | Exact match. The authored stale-expiration assertions follow from phase 1 preceding phase 3. |
| C03 | At 0, A is removed from ready and reserves core 0: A overhead `[0,1)`, useful `[1,3)`. B arrives at 1 and waits `[1,3)`. At 3 B is selected: overhead `[3,4)`, useful `[4,5)`. Final tick 5; core totals `3/2/0`. | A `1/3/0/1/0/2` (`3=0+1+0+2`); B `3/4/2/1/0/1` (`4=2+1+0+1`). B response is `4-1=3`, not its ready wait. | Exact match. |
| C04 | Tick 0 queue `[A,B]`; A runs `[0,1)`, submits I/O at 1, and is blocked `[1,3)`. B runs `[1,3)`. At tick 3, phase 2 appends A before phase 3 appends expired B, producing `[A,B]`. A runs `[3,4)`; B resumes `[4,6)`. Final tick 6; core totals `6/0/0`. | A `0/4/0/0/2/2` (`4=0+0+2+2`); B `1/6/2/0/0/4` (`6=2+0+0+4`), with B ready intervals `[0,1)` and `[3,4)`. | Exact match. |
| C05 | Both cores idle `[0,2)`. A arrives at 2; ascending core order selects core 0. Core 0: idle `[0,2)`, A overhead `[2,3)`, A useful `[3,5)`; totals `2/1/2`. Core 1: idle `[0,5)`; totals `0/0/5`. Final tick 5. | A `1/3/0/1/0/2` (`3=0+1+0+2`). | Exact match. |
| C06 | Tick 0 queue `[A,B,C]`; phase 4 gives A to core 0 and B to core 1. Each has overhead `[0,1)` and useful `[1,3)`. At tick 3 phase 1, core 0's A completion precedes core 1's B completion; phase 4 gives queued C to core 0. Core 0 then has C overhead `[3,4)` and useful `[4,6)`, totals `4/2/0`. Core 1 is idle `[3,6)`, totals `2/1/3`. Final tick 6. | A `1/3/0/1/0/2`; B `1/3/0/1/0/2`; C `4/6/3/1/0/2`. Each turnaround respectively equals `0+1+0+2`, `0+1+0+2`, and `3+1+0+2`. | Exact match. |
| C07 | Tick 0 queue `[A,B]`; A runs `[0,1)`. At tick 1 A yields and appends behind B, giving `[B,A]`. B runs `[1,3)` and terminates; A resumes `[3,4)`. Final tick 4; core totals `4/0/0`. | A `0/4/2/0/0/2` (`4=2+0+0+2`); B `1/3/1/0/0/2` (`3=1+0+0+2`). | Exact match. |

Across C01-C07, every core ledger is contiguous from 0 to `finalTick`; core useful + overhead + idle equals `finalTick`; and total thread useful service equals total core useful service. README and JSON encode the same policies, core counts, switch costs, arrival/program order, intervals, metrics, final ticks, and assertions. The source audit is also accurate: OSTEP supplies the stated FCFS, turnaround, response, RR, switching-cost tradeoff, I/O-blocking, and multiprocessor concern foundations; declaration-order ties, event phases, useful-only quantum, dispatch-from-idle charging, deterministic core order, and abstract independent timers are clearly labeled OS Laboratory conventions.

## Adversarial cases

### A1 - positive switch completion and new arrival at the same tick

Input: one core, FCFS, switch cost 2; A arrives at 0 with `compute 1; end`; B arrives at 2 with `compute 1; end`.

Independent derivation: A overhead `[0,2)`. At tick 2, phase 1 completes A's switch and begins A useful service before phase 2 appends B. A uses `[2,3)` and ends. B waits `[2,3)`, has overhead `[3,5)`, uses `[5,6)`, and ends. Core totals are useful 2, overhead 4, idle 0 at final tick 6. A is `2/3/0/2/0/1`; B is `3/4/1/2/0/1`.

Outcome: exactly one timeline. The arrival cannot replace reserved A.

### A2 - compute ends exactly at quantum exhaustion, followed by compute

Input: one core, RR quantum 2, switch cost 0; A is `compute 2; compute 1; end`; B is `compute 1; end`; both arrive at 0 in A, B order.

Independent derivation: A uses `[0,2)`. Phase 1 advances A to its next compute while its remaining grant is zero; phase 3 expires A, producing queue `[B,A]`. B uses `[2,3)` and ends; A uses `[3,4)` and ends. A is `0/4/1/0/0/3`; B is `2/3/2/0/0/1`; core totals are `4/0/0`.

Outcome: exactly one timeline. The next compute receives no accidental service before expiration.

### A3 - compute ends exactly at quantum exhaustion, followed by yield, I/O, or end

Input: one core, RR quantum 2, switch cost 0; B is `compute 1; end`, waiting behind A at tick 0.

- Yield variant: A is `compute 2; yield; compute 1; end`. A uses `[0,2)`, yields in phase 1 and appends behind B, and the old expiration is stale. B uses `[2,3)`; A uses `[3,4)`. A is `0/4/1/0/0/3`; B is `2/3/2/0/0/1`.
- I/O variant: A is `compute 2; io_wait 1; compute 1; end`. A blocks `[2,3)` in phase 1 and the old expiration is stale; B uses `[2,3)`. At tick 3 B ends in phase 1, then A wakes in phase 2 and uses `[3,4)`. A is `0/4/0/0/1/3`; B is `2/3/2/0/0/1`.
- End variant: A is `compute 2; end`. A terminates in phase 1 at 2; B uses `[2,3)`. A is `0/2/0/0/0/2`; B is `2/3/2/0/0/1`.

Outcome: exactly one timeline in each variant. The phase-1 boundary action wins and invalidates phase-3 expiration.

### A4 - two core completions plus one I/O wakeup with ready work queued

Input: two cores, FCFS, switch cost 0. At tick 0, declaration order is D (`compute 1; io_wait 2; compute 1; end`), A (`compute 3; end`), C (`compute 2; end`); B (`compute 2; end`) arrives at tick 1.

Independent derivation: tick 0 assigns D to core 0 and A to core 1, leaving `[C]`. At tick 1 D blocks until 3; B arrives, so phase 4 assigns C to core 0 and leaves `[B]`. At tick 3, core 0 completes C before core 1 completes A; phase 2 then appends waking D, producing `[B,D]`. Phase 4 assigns B to core 0 and D to core 1. Core 0 is D `[0,1)`, C `[1,3)`, B `[3,5)`; core 1 is A `[0,3)`, D `[3,4)`, idle `[4,5)`. Final tick 5. D is `0/4/0/0/2/2`; A `0/3/0/0/0/3`; C `1/3/1/0/0/2`; B `2/4/2/0/0/2`.

Outcome: exactly one timeline; phase order and ascending core dispatch produce B on core 0 and D on core 1.

### A5 - zero-cost dispatch and zero-time control before compute or end

Input 1: one core, FCFS, switch cost 0; A arrives at 0 and executes a finite zero-time control sequence before `compute 1; end`.

Independent derivation: phase 4 changes A from dispatching to running at tick 0, closes the finite controls, begins useful `[0,1)`, and terminates A at 1. This path is unique.

Input 2 (minimal counterexample): one core, FCFS, switch cost 0; A is `end`; B is `compute 1; end`; both arrive at 0 in A, B order.

Competing derivations: literal rule 5 lets the core take at most A during phase 4. A immediately ends and releases it, leaving B ready and the core idle, after which rule 9 calls the state invalid and no future event advances time. A same-tick closure interpretation instead reruns phase 4, dispatches B at tick 0, gives B useful `[0,1)`, and finishes at 1. Deferring B to tick 1 would add an idle tick without a scheduling event or stated basis.

Outcome: the compute path has one outcome; the end/release path does not. This is finding F01.

### A6 - stale completion and expiration after state or dispatch changes

Input family: RR with a previously scheduled completion or expiration token from dispatch generation `g`; before that token is considered, the thread blocks, yields, terminates, or is expired and later redispatched as generation `g+1`. The redispatch variant deliberately keeps the same thread, core, and partially consumed compute instruction so generation is the distinguishing field.

Independent derivation: in all variants the old token fails at least one of lifecycle state, core, dispatch generation, or current-instruction identity. It causes no queue insertion, state change, or metric increment. Only the matching current-generation token can complete/expire useful service. A consumed compute-boundary event that leads immediately to block, yield, or end similarly makes its old quantum-expiration token stale.

Outcome: exactly one outcome. Rule 8 supplies all necessary applicability fields, including dispatch generation for same-thread/same-core redispatch.

### A7 - delayed first arrival, multiple idle cores, nonzero switch cost

Input: three cores, FCFS, switch cost 2; A arrives at tick 3 with `compute 1; end`.

Independent derivation: all cores idle `[0,3)`. At tick 3 core 0 reserves A, has overhead `[3,5)`, then useful `[5,6)`. Cores 1 and 2 remain idle `[0,6)`. Final tick 6. A is `2/3/0/2/0/1`; core 0 totals `1/2/3`, and cores 1 and 2 each total `0/0/6`.

Outcome: exactly one timeline. The run window begins at zero and ascending core order selects core 0.

## Findings

### F01 - Blocking - phase 4 cannot close after zero-time release

1. **Severity and title:** Blocking - phase 4 cannot close after zero-time release.
2. **File and section:** `content/concepts/scheduling-m1.md`, **Total event order and same-tick closure**, rules 5, 6, and 9; also **States, ownership, and transitions** for same-tick `end`, yield, I/O, and error.
3. **Violated invariant or ambiguous rule:** A ready thread and idle core must not be left without an eligible dispatch, and same-tick closure must determine one stable boundary. Rule 5 says each core takes at most one ready head in phase 4, while zero-cost rule 6 permits that selected thread to release the core immediately during phase 4. Rule 9 only promises dispatch after phase-1/2/3 releases and labels the resulting ready-plus-idle state invalid; it does not define how to reach a valid state.
4. **Minimal input/counterexample:** One core, FCFS, switch cost 0; A `end`, B `compute 1; end`; both arrive at tick 0 in A, B declaration order.
5. **Independently expected transitions, intervals, or metrics:** The coherent work-conserving interpretation is phase 2 queue `[A,B]`; phase 4 dispatches A, A ends at 0, phase 4 reaches closure again, dispatches B, B uses `[0,1)`, and B ends at 1. Core useful/overhead/idle is `1/0/0`. A has turnaround 0 and no useful service; B is `0/1/0/0/0/1`.
6. **Competing authored result or interpretation:** Literal rule 5 prohibits the core from taking B during that phase, leaving invalid state `(B ready, core idle)` with no future event. An inferred phase-4 fixpoint produces the expected `[0,1)` service, but that fixpoint contradicts “each takes at most one ready head” unless the unit of repetition is clarified.
7. **Recommended contract-level correction:** Define phase 4 as a deterministic fixpoint. After any zero-time phase-4 dispatch reaches yield, block, end, or error and releases a core, repeat an ascending-core scheduling scan at the same tick until no ready/idle pair remains or the run enters a defined error boundary. State how the control-operation budget spans repeated same-tick dispatches so zero-time programs cannot prevent closure. Replace the per-core “at most one” wording with a bound per scan or per dispatch opportunity.
8. **Existing fixture detection:** No. Every existing zero-cost dispatch reaches positive compute before releasing its core.

### F02 - Blocking - error transition delegates run ownership and accounting semantics

1. **Severity and title:** Blocking - error transition delegates run ownership and accounting semantics.
2. **File and section:** `content/concepts/scheduling-m1.md`, **States, ownership, and transitions**, row `any nonterminated state -> error`; **Time and interval rules**, definition of the error boundary.
3. **Violated invariant or ambiguous rule:** Lifecycle/core/queue ownership must be exclusive at every stable boundary, and T02/T03 must not invent whether an execution error is thread-local or run-fatal. “Stop according to the future run-error contract” leaves unspecified whether the failing thread is removed from ready/blocked ownership or releases a reserved/running core; whether other threads continue; which pending tokens are invalidated; and how `final_tick`, core intervals, and censored thread metrics are closed.
4. **Minimal input/counterexample:** One core, switch cost 0; A begins a zero-time loop and exhausts the bounded control-operation budget at tick 0 while B is ready with `compute 1; end`.
5. **Independently expected transitions, intervals, or metrics:** No single expectation can be derived. A run-fatal rule would stop at error tick 0, close every thread as incomplete/censored, invalidate events, and define ownership cleanup at an error boundary. A thread-local rule would move A to error, release the core, and allow B to use `[0,1)` under same-tick closure. These produce different terminal states, `final_tick`, throughput, and core ledgers.
6. **Competing authored result or interpretation:** The authored row permits both interpretations by deferring the choice. It gives no required ownership effect analogous to the other transition rows.
7. **Recommended contract-level correction:** Select and state run-fatal or thread-local error semantics. Define queue/core removal, generation invalidation, disposition of other threads/events, same-tick dispatch behavior, final-tick selection, and completed/censored accounting. T02 may then encode the decision without creating scheduling semantics.
8. **Existing fixture detection:** No existing case enters `error`.

### F03 - Blocking - response is undefined for a valid completed thread with no compute

1. **Severity and title:** Blocking - response is undefined for a valid completed thread with no compute.
2. **File and section:** `content/concepts/scheduling-m1.md`, **Metrics and conservation checks**, response definitions; `OS-Laboratory-Spec.md` sections 7 and 9.
3. **Violated invariant or ambiguous rule:** The required response metric must have one representation for every reportable thread status. M1 permits programs composed of `io_wait`, zero-time control, yield, and `end` without stating that a compute block is mandatory. For a completed thread that never begins useful compute, “first useful-compute start - arrival” has no left operand. The contract only makes response conditional for an unfinished thread.
4. **Minimal input/counterexample:** One core, FCFS, switch cost 0; A arrives at 0 with `io_wait 1; end` (or simply `end`).
5. **Independently expected transitions, intervals, or metrics:** For `io_wait 1; end`, A blocks `[0,1)`, wakes and ends at tick 1, so turnaround 1, blocked 1, and useful 0. There is no first useful-compute tick, so response cannot be numerically derived. The same issue occurs at final tick 0 for `end`.
6. **Competing authored result or interpretation:** A consumer could omit response, encode `null`/undefined, or infer 0 from first dispatch. Those choices are observably different and only the last changes the contract's stated “first useful computation” meaning.
7. **Recommended contract-level correction:** Explicitly define response as absent/undefined for every thread, completed or unfinished, that never begins useful computation, and require T02's schema to encode that state; alternatively prohibit no-compute programs through an explicit validation rule. The former preserves the current program language and section 9 meaning.
8. **Existing fixture detection:** No. Every reference thread performs useful compute.

No significant or editorial findings were identified beyond the blocking findings above.

## Acceptance checklist

- **Fail - lifecycle transitions and queue/core ownership:** ordinary transitions are complete and exclusive, but error ownership/cleanup is delegated (F02), and zero-time phase-4 release lacks a closure transition (F01).
- **Fail - deterministic same-tick ordering:** phases, insertion sequence, same-phase ordering, stable core generation, and normal dispatch are deterministic, but a phase-4 zero-time release does not have exactly one next action (F01).
- **Fail - FCFS/RR coverage of yield, block, quantum equality, switch cost, and multicore behavior:** all positive-time and existing-case paths are determined; the zero-cost dispatch path that immediately yields, blocks, ends, or errors is not closed (F01), and error semantics remain open (F02).
- **Pass - at least six independently worked required cases:** seven are present and cover simultaneous arrival, burst/quantum equality, nonzero switching, I/O wakeup, idle cores, stable two-core behavior, and yield.
- **Pass - exact case intervals and calculations:** C01-C07 independently reproduce exactly; all thread and core conservation checks hold, and README/JSON agree.
- **Pass - source/convention/omission boundary:** direct review of OSTEP chapter 7 and chapter 10 supports the claimed foundations; project-specific choices are labeled accurately and do not claim Linux equivalence.
- **Fail - no unresolved semantics / independent approval before T03:** F01-F03 would require T02 or T03 to invent observable behavior or encoding, so independent approval is withheld.

Verification executed during review:

- `node -e "const c=JSON.parse(require('fs').readFileSync('tests/reference/scheduling/cases.json','utf8')); console.log('valid JSON',c.cases.length)"` - passed; printed `valid JSON 7`.
- `npm run format:check` - passed; Prettier reported all configured files matched.
- `git -c safe.directory=D:/codex/_projects/OS-laboratory diff --check` - passed with no output. The command-local safe-directory setting was necessary because the sandbox user differs from the workspace owner; no Git configuration or branch history was changed.

## Verdict

**CHANGES REQUIRED**

Minimal correction scope: define deterministic same-tick phase-4 redispatch after zero-time release; settle error ownership, stopping, invalidation, and accounting; and define response encoding for threads that never begin useful compute. Required re-review is limited to those corrected clauses plus small fixtures covering (1) zero-cost immediate end with another ready thread, (2) a bounded-control/overflow error with other work present, and (3) a completed no-compute program. C01-C07 do not need wholesale re-review unless their normative expectations change.

---

## Focused re-review addendum

Reviewer metadata:

- Date: 2026-09-09.
- Reviewed state: commit `098bb5417e871a9d28f415e4c389d59d80d186f4` plus the uncommitted T01-FIX changes to `content/concepts/scheduling-m1.md`, `tests/reference/scheduling/README.md`, and `tests/reference/scheduling/cases.json`.
- Actual model and reasoning effort: Codex based on GPT-5. The exact routed model ID and reasoning-effort setting are not exposed to this reviewer, so neither is guessed.
- Scope: the corrected clauses for F01-F03 and new cases C08-C10. C01-C07 were confirmed unchanged and were not wholesale re-reviewed.
- Sources inspected: the required repository files in `docs/reviews/T01-REVIEW-PACKET.md`; OSTEP v1.10 chapter 7 sections 7.1-7.3 and 7.6-7.8; OSTEP v1.10 chapter 10 for the unchanged multiprocessor qualification; and the supplemental homework index.
- Independence statement: the corrected outcomes below were derived from the contract and case inputs before comparison with the authored README and JSON. No production scheduler, host-language ordering, Linux trace, or homework simulator was used as an oracle.

### Corrective case recomputation

| Case | Independent derivation | Metrics and encoding | Result |
| --- | --- | --- | --- |
| C08 | Tick 0 phase 2 produces `[A,B]`. Phase-4 scan 1 dispatches A; zero-cost transition reaches `end`, terminating A at tick 0 and releasing core 0. The ready/idle pair requires scan 2, which dispatches B at tick 0. B uses `[0,1)` and terminates at 1. | A has response `null`, turnaround 0, and zero component totals. B is `0/1/0/0/0/1`. Core 0 is useful `[0,1)`, totals `1/0/0`; final tick 1; completed throughput 2. README and JSON match. | F01 closed. Rule 5 defines a deterministic ascending-core fixpoint, rule 9 forbids a stranded ready/idle pair, and the run-wide budget bounds repeated zero-time scans. |
| C09 | Tick 0 phase 2 produces `[A,B]`. A is dispatched and consumes budget units 1-3 on its first three empty-repeat iteration evaluations. Evaluation 4 is the first disallowed operation and does not execute. The attributable fatal boundary occurs at tick 0 before B dispatches. | A is `error`; B is `censored`; both have response and turnaround `null` and zero accumulated components. The `[0,0)` core ledger is empty with zero totals; utilization and throughput are `null`; completed count is zero. README and JSON match. | F02 closed. The contract fixes run-wide budget scope, failing-operation behavior, queue/core cleanup, token/event invalidation, other-thread disposition, final tick, censored accounting, and prohibition on same-tick continuation. |
| C10 | A dispatches at tick 0, submits I/O, blocks on `[0,1)`, wakes in phase 2 at tick 1, and dispatches to `end` in phase 4 at tick 1 without useful service. | A is completed with response `null`, turnaround 1, ready/overhead/useful 0, and blocked 1. Core 0 is idle `[0,1)`, totals `0/0/1`; final tick 1; throughput 1. README and JSON match. | F03 closed. The contract assigns JSON `null` to absent response for both completed and unfinished threads and distinguishes it from numeric zero. |

### Adversarial follow-ups

- Multiple zero-time releases across multiple cores remain deterministic: each scan observes cores once in ascending ID, queue mutations take effect immediately, and another scan begins only if a ready/idle pair remains.
- A fatal error after positive-time execution closes active ready, blocked, dispatch, running, useful, overhead, and idle accounting at the error tick; it does not count error/censored threads as completed and it invalidates all future work.
- A thread that dispatches or blocks but never starts useful compute still has response `null`; dispatch overhead and blocked time remain separately measurable.
- If one thread terminates earlier in event order at the same tick that another thread later causes a fatal error, the surrounding rules preserve the already-terminated thread and include it in completed counts because it terminated before the error. The narrower sentence “Threads terminated before `t` remain completed” is therefore redundant rather than outcome-changing.

### Findings

#### E01 - Editorial - error-boundary completion wording uses two notions of “before”

`content/concepts/scheduling-m1.md`, **States, ownership, and transitions**, says both that every thread “not already terminated” is censored and that threads terminated “before `t`” remain completed. A thread may terminate earlier in event order at the same tick `t` that another thread causes an error. The later rule that completed counts include threads terminated “before the error” resolves the outcome: the first thread remains completed. Replacing “before `t`” with “before the error boundary” would remove the redundant narrower phrasing. This cannot change an implementation that follows the complete paragraph, so it is editorial and does not block acceptance. No existing fixture exercises this exact same-tick sequence.

No blocking or significant finding remains in the focused correction scope.

### Acceptance checklist

- **Pass - F01 / lifecycle and deterministic same-tick closure:** phase-4 scans reach a bounded deterministic fixpoint and cannot strand eligible work.
- **Pass - F02 / error ownership and accounting:** attributable and non-attributable fatal outcomes, lifecycle disposition, queue/core cleanup, interval closure, invalidation, final tick, and completed/censored metrics are defined.
- **Pass - F03 / response representation:** no-compute threads use JSON `null` for response, distinct from response zero.
- **Pass - corrective fixture accuracy:** C08-C10 independently reproduce exactly; README and JSON agree, thread/core conservation holds, and the zero-length error window is represented consistently.
- **Pass - source/convention boundary:** OSTEP supports the cited general response, turnaround, FCFS, RR, switching, and I/O foundations; the corrected fixpoint, fatal-error, budget, and encoding rules are clearly project conventions.
- **Pass - T02/T03 readiness for corrected scope:** the fixes no longer require downstream tasks to choose between competing scheduling outcomes.

Focused verification executed:

- `npm run format:check` - passed.
- `git -c safe.directory=D:/codex/_projects/OS-laboratory diff --check` - passed with no output.
- Manual independent arithmetic for C08-C10 - matched README and JSON, including the zero-tick error ledger and `null` metrics.

### Focused re-review verdict

**ACCEPT**

F01-F03 are closed. The corrected scheduling semantics and C08-C10 are approved for T01's independent-review gate, subject to the orchestrator's remaining commit, protected remote-CI, and integration steps. This review does not mark the PR ready, merge it, or start T02/T03.
