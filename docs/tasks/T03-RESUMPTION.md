# T03 resumption — 2026-09-13

Root is readiness, review integration, and handoff owner. Dependencies T00–T02 are integrated at main `e99d65a`. Author code remains `e269ee1`; current checkpoint `09383b6` preserves the interrupted handoff. Existing test evidence remains evidence for that unchanged source, not a substitute for review.

## K-review: event kernel

- Agent `t03_kernel_review`, GPT-5.6 Sol / xhigh, tier A, fresh context.
- Read scope: `crates/sim-core/src/kernel.rs`, `kernel_tests.rs`, input/policy interfaces as necessary; `sched.m1` and `program-transport.m1` concept contracts; T03 decisions; independent scheduling fixtures and author evidence. Read repository instructions, spec, TODO and handoff first.
- Owned output: `docs/reviews/T03-KERNEL-REVIEW.md` only. No source edits; temporary independent probes under `.verification/review-kernel/` only.
- Acceptance: independently assess phase ordering, stable closure, control-budget scope, repeat sources, stale instruction/dispatch tokens, checked counters/deadlines, fatal cleanup and raw accounting intervals. Give ACCEPT or CHANGES REQUIRED with reproducible concrete findings. Review test-only policy boundaries; do not claim T04/T05 complete.
- Checks: inspect complete kernel diff from `e99d65a`; run focused existing tests and at most a small set of independent probes addressing concrete suspected defects. Record exact commands and observed/expected behavior. Do not rerun browser/build suites.
- Escalate undefined contract behavior, conflicting derivations, or unsafe remaining context by checkpointing a partial report immediately. Root resolves findings; targeted Astra / xhigh escalation only if unresolved.

## D-review: decoding and transport compatibility

- Agent `t03_input_review`, GPT-5.6 Sol / xhigh, tier A, fresh context.
- Read scope: `crates/sim-core/src/input.rs`, `input_tests.rs`, pinned manifests/lockfile; shared schema/TS and validators; T03 generated-ID correction, event-check scripts, and relevant concept contracts/fixtures. Read repository instructions, spec, TODO and handoff first. Kernel internals belong to K-review; inspect only producer interfaces if required.
- Owned output: `docs/reviews/T03-INPUT-REVIEW.md` only. No source edits; temporary independent probes under `.verification/review-input/` only.
- Acceptance: check valid/invalid workload parity, byte/depth/count and numeric boundaries, version precedence, immutable source mapping, generated-ID correction and actual-event validator claims. Give ACCEPT or CHANGES REQUIRED with concrete reproductions and residual limits.
- Checks: inspect scoped diff from `e99d65a`; focused decoder/contracts checks and small targeted probes only. Do not rerun browser/build suites.
- Escalation/reviewer integration owner: root, as for K-review.

## Root integration and usage discipline

Both reviewers are read-only except disjoint reports, so they share the existing implementation worktree. Root owns HANDOFF, routing, combined review result, and verification status. Save initial partial review coverage before substantial probing and update after each bounded unit. No new implementation task starts before T03 acceptance/integration. No live account-quota tool is available; prior percentages are historical, not current readings. Do not burn another window repeating completed verification. Checkpoint any unresolved work honestly.
