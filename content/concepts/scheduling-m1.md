# M1 scheduling model contract

Contract ID: `sched.m1`; revision: 1; status: author review pending independent tier-A review. This record defines the teaching model consumed by T02–T06. It is not a claim about a particular production OS scheduler.

## Learning objective and supported claims

A learner can predict how FCFS and round-robin move threads among not-arrived, ready, dispatching, running, blocked, and terminated states; explain how quantum and dispatch cost affect response and turnaround; and reconcile the timeline with ready waiting, blocked time, and per-core accounting.

Supported claims:

- FCFS selects the oldest eligible entry in the single global FIFO ready queue and does not preempt it merely because another thread becomes ready.
- RR selects from that same queue, allows at most one quantum of useful CPU service per dispatch, and appends an unfinished expired thread behind work made ready earlier at the same tick.
- A running thread that requests modeled I/O is blocked and consumes no CPU until the independent timer completes.
- Shorter RR quanta can improve initial response while increasing dispatch overhead when dispatch has a cost. No policy is universally more efficient for every workload or metric.
- M1's identical cores, global queue, abstract I/O timers, and deterministic tie-breaks are simplifications, not Linux-equivalent behavior.

## Source audit

Primary source: Remzi H. Arpaci-Dusseau and Andrea C. Arpaci-Dusseau, _Operating Systems: Three Easy Pieces_, free chapter PDFs, version 1.10.

| Source section       | Adopted foundation                                                                                                                          | Project-specific qualification                                                                                                   |
| -------------------- | ------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------- |
| Ch. 7 §7.1, pp. 1–2  | Workload assumptions must be explicit and relaxed deliberately.                                                                             | M1 allows staggered arrivals, I/O, preemption under RR, and multiple identical cores. Burst lengths are declared program inputs. |
| Ch. 7 §7.2, p. 2     | Turnaround is completion minus arrival.                                                                                                     | M1 termination is the completion point; unfinished threads are censored rather than assigned turnaround.                         |
| Ch. 7 §7.3, pp. 2–3  | FIFO/FCFS runs jobs in arrival order and is simple but can suffer a convoy effect.                                                          | Exact simultaneous-arrival order is declared workload order, then stable thread ID as a validation tie-break.                    |
| Ch. 7 §7.6, pp. 6–7  | Response is first scheduled service minus arrival.                                                                                          | M1 defines this as the tick useful user computation first begins, so dispatch overhead is included.                              |
| Ch. 7 §7.7, pp. 7–9  | RR runs for a time slice, rotates through the run queue, and trades response against turnaround; switch cost is amortized by longer slices. | M1 measures quantum in useful service ticks and charges configured dispatch cost on every dispatch, including from idle.         |
| Ch. 7 §7.8, pp. 9–10 | A thread doing I/O blocks; another can run; I/O completion returns it to ready.                                                             | M1 I/O is an independent deterministic timer with no device queue. Its same-tick ordering is a project convention below.         |
| Ch. 10               | Real multiprocessor scheduling raises cache affinity, synchronization, and queue-design concerns.                                           | M1 intentionally omits affinity/cache/migration costs and uses one global FIFO queue with deterministic ascending core IDs.      |

Links: <https://pages.cs.wisc.edu/~remzi/OSTEP/cpu-sched.pdf>, <https://pages.cs.wisc.edu/~remzi/OSTEP/cpu-sched-multi.pdf>, and the supplemental simulator index at <https://pages.cs.wisc.edu/~remzi/OSTEP/Homework/homework.html>. OSTEP asks readers to link to its chapters rather than redistribute them; no source text or exercise is copied into this repository.

The state names, event phases, stable tie-breaks, dispatch-from-idle charge, zero-cost behavior, and exact metric window below are OS Laboratory conventions selected for determinism and inspectability. They are not assertions that OSTEP or a real OS mandates those details.

## Time and interval rules

Ticks and event insertion sequences are checked unsigned 64-bit integers. Every duration is a positive integer except configured `switch_cost`, which may be zero. Arithmetic overflow is a run error; it never wraps.

All ledgers use half-open intervals `[start, end)`. A useful interval `[3, 5)` contributes two useful ticks and completes at tick 5. State changes at a tick consume no simulated time. A run's accounting window is `[0, final_tick)`, where `final_tick` is the tick at which all threads have terminated, or the reported quiescent/error boundary for an incomplete run. A zero-length window reports utilization as undefined rather than dividing by zero.

The quantum counts useful CPU service only. Dispatch overhead, blocked time, and idle time do not consume quantum. Each dispatch grants a fresh full quantum under RR. FCFS has no quantum.

## States, ownership, and transitions

At every stable boundary a thread has exactly one lifecycle state and cannot be owned by more than one queue/core.

| From                    | Cause                                             | To          | Required effect                                                                                                             |
| ----------------------- | ------------------------------------------------- | ----------- | --------------------------------------------------------------------------------------------------------------------------- |
| not-arrived             | arrival                                           | ready       | Append once to the global ready tail.                                                                                       |
| ready                   | scheduler selects it                              | dispatching | Remove the head and reserve exactly one idle core. Start a dispatch-overhead interval.                                      |
| dispatching             | switch completion                                 | running     | End overhead; begin/resume the current compute instruction. With zero cost this happens immediately in the dispatch phase.  |
| running                 | compute boundary followed by more compute/control | running     | Continue on the same core; FCFS keeps service, and RR keeps the remaining quantum.                                          |
| running                 | `yield`                                           | ready       | Release the core and append to the ready tail. Any pending completion/expiration carrying the old generation becomes stale. |
| running                 | `io_wait(d)` submission                           | blocked     | Release the core and schedule completion at current tick plus positive `d`.                                                 |
| blocked                 | I/O completion                                    | ready       | Append once to the ready tail.                                                                                              |
| running                 | applicable RR quantum expiration                  | ready       | Release the core and append after phase-2 ready events at this tick.                                                        |
| running                 | `end` after an instruction boundary               | terminated  | Release the core permanently.                                                                                               |
| any nonterminated state | validated program/overflow/budget error           | error       | Stop according to the future run-error contract; never fabricate successful completion.                                     |

`dispatching` owns a core but is neither ready nor running. A blocked thread owns no core. Termination is legal only at an instruction boundary. Zero-time control instructions execute in program order under T03's bounded control-operation budget. If they reach another compute instruction, the thread remains running as described above; if they reach yield, I/O, end, or error, that transition occurs at the same tick.

## Total event order and same-tick closure

The total key is `(tick, phase, insertion_sequence)`, ascending. Phase meanings are fixed:

1. useful-compute completions and switch completions already scheduled for this tick;
2. arrivals and I/O completions;
3. still-applicable RR quantum expirations;
4. scheduling dispatches onto idle cores.

Rules inside that order:

1. Initial arrivals are inserted in workload declaration order. Validation rejects duplicate IDs; stable thread ID is the documented fallback only for imported records that otherwise lack a declaration order.
2. Events created by cores at the same logical point are inserted in ascending core ID. Other equal-phase events retain their original insertion order. Host map/object iteration order is never semantic.
3. Phase 1 processes a compute boundary before phase 3 evaluates its quantum token. If zero-time continuation ends, blocks, yields, or errors, the old expiration is stale. If the thread remains running and has exhausted its quantum, phase 3 expires it.
4. Phase-2 arrivals and I/O completions append before phase-3 expired threads. Thus newly ready work precedes an expired thread at the same tick.
5. In phase 4, idle cores are considered in ascending ID. Each takes at most one ready head. Continue until either no idle core or no ready thread remains.
6. Positive switch cost `s` reserves the core over `[t, t+s)` and schedules switch completion at `t+s`. It does not allow replacement by later arrivals. Cost zero changes dispatching to running immediately during phase 4 at tick `t`; the first positive-duration compute completion is scheduled for a later tick.
7. A switch completion that reaches running at phase 1 may schedule a future useful completion/expiration but does not rerun an already passed same-tick phase. Zero-time program control is closed at that boundary before later phases.
8. Generation tokens attach to compute completion and quantum expiration. A token is applicable only if thread, core, dispatch generation, and current instruction still match. Ignoring a stale token causes no queue or metric change.
9. Dispatch runs after every phase-1/2/3 release at the tick. If no event exists but ready work and an idle core do, that state is invalid rather than silently waiting.

## Policy rules

FCFS never preempts a running or dispatching thread for arrival or wakeup. It releases a core only at yield, I/O submission, end, or error. A sequence of compute instructions separated only by zero-time repeat/control boundaries remains one FCFS possession.

RR grants `quantum` useful ticks at each transition from dispatching to running. If a compute instruction ends exactly at the quantum boundary, its instruction-boundary effects happen first. End, block, or yield wins and makes expiration stale; another compute instruction with the thread still running is then expired in phase 3. An early yield discards the unused grant. Blocking discards it; wakeup requires a new dispatch and fresh grant.

Both policies use the same global FIFO queue. Queue position is never changed by core affinity, priority, random choice, or predicted burst length in M1.

## Metrics and conservation checks

For a completed thread:

- response = first useful-compute start − arrival;
- turnaround = termination − arrival;
- ready wait = sum of ready intervals, ending when selected for dispatch (overhead is not ready wait);
- dispatch overhead = sum of its dispatching interval durations;
- blocked time = sum of blocked intervals;
- useful time = sum of useful-compute intervals.

For an unfinished thread, retain the accumulated components and report response only if it began useful work; turnaround is censored. For every completed thread with no unmodeled zero-time delay:

`turnaround = ready_wait + dispatch_overhead + blocked_time + useful_time`.

For each core over the run window, useful, overhead, and idle intervals are disjoint and exhaustive:

`useful + overhead + idle = final_tick`.

Useful/overhead/idle utilization uses `final_tick` as denominator and reports both counts and ratios. Completed throughput is `completed_threads / final_tick`; it is undefined for a zero-length window. These definitions do not add an energy or fairness score.

## Assumptions and omissions

M1 has identical cores, no affinities, no cache or migration cost, no priority, one global ready queue, deterministic inputs, no device contention, and no scheduler execution cost beyond configured dispatch overhead. `io_wait` models elapsed blocking only. Manual dispatch, synchronization dependencies, deadlock classification, interactive request markers, Linux scheduler profiles, and parallel processing of one event queue are later contracts.

## Visual mapping

Ready entries map to the global queue in exact order; dispatching maps to a reserved core with an overhead cue; running maps to useful activity and the active source block; blocked maps to its timer/dependency; terminated maps to an archival state. Text/shape labels must accompany color. Animation may interpolate only inside authoritative intervals and cannot create events.

## Independent cases

The normative machine-readable ledgers are in `tests/reference/scheduling/cases.json`; their format is test-only until T02 defines shared schemas. `tests/reference/scheduling/README.md` works every case arithmetically. A future engine must match their semantics, not merely their JSON spelling.
