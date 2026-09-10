# T01 independent semantic review packet

Status: ready for independent execution. Date prepared: 2026-09-09. Review target: draft PR #2 on branch `codex/t01-scheduling-contract`.

## Purpose and independence boundary

Independently audit the M1 scheduling contract and its reference cases before T01 is accepted. This is a semantic review, not an implementation task.

Start from a fresh conversation or agent context. Read repository sources directly; do not inherit the author's conversation, scratch reasoning, or unpublished explanations. The reviewer must not edit the contract, fixtures, task status, application, engine, PR state, or branch history. The only permitted repository write is the review report described below.

The reviewer reports findings. The primary orchestrator owns corrections, verification, PR readiness, and integration. After corrections, the same reviewer may receive a narrow follow-up to confirm whether each finding was resolved.

## Portable model policy

The normative assignment is capability-based:

- Role: independent tier-A semantic reviewer.
- Required capability: a flagship model suitable for difficult multi-step reasoning and coding review.
- Reasoning effort: `xhigh` when supported; otherwise the highest available effort below a materially more expensive maximum mode.
- Context: fresh/no inherited turns. The repository and this packet supply the context.
- The reviewer records the actual model and effort when known. It must not guess unavailable model metadata.

Model names are dated routing examples, not permanent requirements. As of 2026-09-09:

1. Default economical route: `gpt-5.6-sol` at `xhigh`.
2. Escalation route: `gpt-6-astra` at `xhigh`, limited to unresolved contradictions, multiple plausible timelines, source/contract conflicts, or a focused re-review of blocking findings.
3. Do not repeat the entire review with the escalation model merely for reassurance. Give it the smallest disputed rule, counterexample, and competing derivations that preserve independence.
4. Do not use `max` by default. Raise effort only for a bounded issue that remains unresolved at `xhigh`.

If those model IDs later disappear, select their current functional equivalents using official model guidance: a cost-conscious flagship for the first pass and the most capable complex-reasoning model only for escalation. Review acceptance depends on evidence and reproducibility, not the brand name of the reviewer.

Current routing references: <https://developers.openai.com/api/docs/models/gpt-5.6-sol>, <https://developers.openai.com/api/docs/models/gpt-6-astra>, and <https://developers.openai.com/api/docs/models>.

## Required reading

Read these repository files completely:

1. `AGENTS.md`
2. `OS-Laboratory-Spec.md`, especially sections 6, 9, and 11
3. `IMPLEMENTATION-TODO.md`, T01 and the dependency gates around T02–T05
4. `docs/tasks/T01.md`
5. `content/concepts/scheduling-m1.md`
6. `tests/reference/scheduling/README.md`
7. `tests/reference/scheduling/cases.json`
8. `docs/verification/T01.md`, treating author-side checks as claims to audit rather than semantic proof

Audit the cited primary source sections directly:

- OSTEP v1.10 chapter 7 §§7.1–7.3 and 7.6–7.8: <https://pages.cs.wisc.edu/~remzi/OSTEP/cpu-sched.pdf>
- OSTEP v1.10 chapter 10 only for multiprocessor concerns and omissions: <https://pages.cs.wisc.edu/~remzi/OSTEP/cpu-sched-multi.pdf>
- The homework index is supplemental, not normative: <https://pages.cs.wisc.edu/~remzi/OSTEP/Homework/homework.html>

Do not treat similarity to Linux, Rust behavior, JavaScript ordering, a production scheduler, or an OSTEP homework simulator as an oracle. Identify which rules come from sources and which are deliberate OS Laboratory conventions.

## Review method

### 1. Derive before comparing

For each reference case, first use only its policy, core count, switch cost, arrival declarations, and programs to derive:

- ordered state/event transitions;
- ready-queue order at every scheduling boundary;
- half-open useful, overhead, and idle intervals per core;
- response, turnaround, ready wait, dispatch overhead, blocked time, and useful time per thread;
- the final accounting tick.

Preserve that derivation in the report or private scratch notes before comparing it with the authored expected ledger. Do not run future production scheduler code or mechanically restate `cases.json` as the oracle.

Then compare the independent result with both `README.md` and `cases.json`. Report every mismatch, even when the two authored files agree with each other.

### 2. Audit the normative contract

Challenge at least these areas:

- lifecycle states are complete, exclusive, and have unambiguous queue/core ownership;
- `(tick, phase, insertion_sequence)` is a total deterministic order;
- same-tick closure cannot skip eligible dispatch or repeat a passed phase incorrectly;
- zero-cost dispatch has defined behavior without an accidental extra tick;
- positive switch cost, including dispatch from idle, is reserved and accounted exactly once;
- FCFS behavior is defined for arrival, wakeup, compute continuation, yield, block, end, and error;
- RR quantum counts the intended service, resets only at a new dispatch, and handles exact compute/quantum equality;
- consecutive compute blocks at a quantum boundary cannot receive accidental extra service;
- block, yield, termination, and redispatch invalidate stale completion/expiration tokens safely;
- simultaneous arrival, I/O completion, quantum expiration, switch completion, and dispatch have one outcome;
- multicore completion generation and dispatch order are stable and do not depend on host iteration order;
- the global FIFO ready queue cannot contain a thread twice or dispatch blocked/dispatching work;
- response, turnaround, ready-wait, overhead, blocked, useful, idle, utilization, throughput, and censored status are mutually consistent;
- T02 can encode the contract without inventing or changing a semantic rule.

### 3. Construct adversarial cases

Independently derive at least these additional cases. They do not have to become fixtures unless a finding shows that coverage is needed.

1. A positive switch completion and a new arrival at the same tick.
2. A compute block ending exactly at quantum exhaustion followed by another compute block.
3. A compute block ending exactly at quantum exhaustion followed by `yield`, `io_wait`, and `end` variants.
4. Two core completions and one I/O wakeup at the same tick with ready work already queued.
5. Zero-cost dispatch followed by zero-time repeat/control operations before compute or end.
6. Stale completion and expiration events after block, yield, termination, and redispatch.
7. A delayed first arrival with multiple idle cores and nonzero switch cost.

For each, state whether the existing contract determines exactly one timeline. If it does not, provide the smallest counterexample and the competing valid interpretations.

### 4. Check calculations and encoding

Recompute the conservation identities rather than trusting the author's script:

- completed-thread turnaround equals ready wait plus dispatch overhead plus blocked plus useful;
- each core's useful plus overhead plus idle equals `finalTick`;
- total useful service agrees between thread and core ledgers.

Confirm that Markdown and JSON encode the same programs, policies, intervals, metrics, and assertions. Encoding/style defects are editorial unless they could change a consumer's semantic interpretation.

## Finding severity

- **Blocking:** nondeterminism, contradiction, incorrect normative outcome, missing ownership/transition, wrong metric definition, source misrepresentation, or a rule T02–T05 would have to invent.
- **Significant:** a reproducible coverage or clarity defect likely to cause divergent implementations, even if one intended answer can be inferred.
- **Editorial:** wording, formatting, or nonsemantic metadata that cannot change an implementation outcome.

Every non-editorial finding must contain:

1. severity and short title;
2. exact file and section;
3. violated invariant or ambiguous rule;
4. minimal input/counterexample;
5. independently expected transitions, intervals, or metrics;
6. competing authored result or interpretation;
7. recommended contract-level correction;
8. whether an existing fixture detects the defect.

Do not silently repair a finding. Do not weaken an expected result merely to make it consistent with the contract.

## Required report

Write only `docs/reviews/T01-REVIEW-RESULT.md`. If repository writing is unavailable, return the same content in the final response for the orchestrator to record. Use this structure:

```text
# T01 independent semantic review result

Reviewer metadata
- Date
- Reviewed commit
- Actual model and reasoning effort, if known
- Sources inspected
- Independence statement

Case recomputation
- One row per C01–C07 with independently derived core ledger,
  thread metrics, match/difference, and supporting arithmetic

Adversarial cases
- Input, independent derivation, whether the contract has one outcome

Findings
- Ordered by severity using the required finding fields

Acceptance checklist
- One explicit pass/fail/not-established result for every T01 acceptance item

Verdict
- ACCEPT | CHANGES REQUIRED | CONTRACT BLOCKED
```

Verdict rules:

- `ACCEPT`: all seven cases were independently reproduced, every required semantic area has exactly one outcome, source/convention boundaries are accurate, and no blocking or significant finding remains.
- `CHANGES REQUIRED`: at least one correctable blocking/significant defect exists; include the minimal correction and required re-review scope.
- `CONTRACT BLOCKED`: authoritative sources or requirements conflict, or more than one product decision remains valid and the reviewer cannot resolve it without owner direction.

Editorial findings may accompany `ACCEPT` only when they cannot alter behavior. The reviewer must not mark the PR ready, merge it, accept T01 in task records, or begin T02/T03.

## Prompt for a fresh review chat

Use this concise prompt after selecting a model under the portable policy:

> Act as the independent tier-A semantic reviewer for T01. Read and execute `docs/reviews/T01-REVIEW-PACKET.md` exactly. Start from repository evidence rather than prior conversation. Do not edit the contract, fixtures, task status, application, engine, PR, or branch history. Write only `docs/reviews/T01-REVIEW-RESULT.md`, with independently recomputed cases, adversarial cases, findings, acceptance checklist, and the required verdict. Stop after the report; do not implement fixes or merge anything.
