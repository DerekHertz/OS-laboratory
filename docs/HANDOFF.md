# Implementation handoff

Date: 2026-09-09. Active branch: `codex/t01-scheduling-contract`. Main baseline: `fcd8159`.

T00 is complete and integrated. PR #1 merged as `fcd8159`; two remote `build-and-test` checks passed on its exact branch head. Main protection was applied from `docs/decisions/main-protection.json` and read back successfully. Required settings include strict `build-and-test`, pull requests, linear history, resolved conversations, admin enforcement, and no force-push/deletion.

T01 is in review on draft PR #2. The author commit is `aed2efa`; both remote `build-and-test` runs passed in 55 and 57 seconds. The contract is `content/concepts/scheduling-m1.md`; seven independent worked ledgers are in `tests/reference/scheduling/`. Author-side JSON, interval, metric-conservation, formatting, whitespace, and full workspace checks pass. No engine or transport code has been added. A separate tier-A semantic review is still required before T01 acceptance and before T03. See `docs/tasks/T01.md` and `docs/verification/T01.md`.

Source audit used OSTEP v1.10 chapter 7 §§7.1–7.3 and 7.6–7.8 for workload assumptions, metrics, FCFS, RR, switch-cost tradeoffs, and I/O blocking; chapter 10 establishes real multiprocessor concerns that M1 explicitly omits. Project-specific event phases, queue tie-breaks, idle-dispatch charges, and stable core ordering are labeled as teaching-model conventions.

Next actions:

1. Start a fresh review chat and execute `docs/reviews/T01-REVIEW-PACKET.md`. Use its portable tier-A routing: currently GPT-5.6 Sol at `xhigh` for the economical first pass, escalating only disputed issues to GPT-6 Astra at `xhigh`. The reviewer writes only `docs/reviews/T01-REVIEW-RESULT.md`.
2. After review, update the evidence, mark PR #2 ready, require protected remote CI on the final head, and integrate only if it passes.
3. After T01 is accepted, expand and execute T02 program/transport contracts. T03 remains blocked on T00–T02; do not add kernel behavior early.

No agents or background commands are running. Ignored `node_modules/`, `target/`, generated Wasm/browser output, Playwright results, and `.verification/` are local artifacts, not source inputs.
