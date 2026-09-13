# T03 independent implementation review packet

Baseline: merged T02 `e99d65a`; prerequisite contract clarifications `1f1adb1` and `323af26`. Candidate source checkpoint will be named in the review dispatch.

Reviewer: fresh GPT-5.6 Sol / xhigh, independent of author and contract auditor; root integrates findings. Escalation: GPT-6 Astra / xhigh only for unresolved competing derivations or semantic blockers.

Read AGENTS.md, OS-Laboratory-Spec.md, IMPLEMENTATION-TODO.md, docs/HANDOFF.md, docs/tasks/T03.md, both normative M1 concept contracts, T03 decisions/verification, and independently worked scheduling fixtures. Review all candidate diffs from e99d65a, including prerequisite contract corrections and pinned dependencies.

Review ownership is limited to `docs/reviews/T03-REVIEW-RESULT.md`; no implementation edits. Check event order and stable closure, generation/stale instruction applicability, checked arithmetic before mutation, immutable interpreter and per-tick repeat/control accounting, fatal cleanup/source state, canonical event source/causality/ID rules, Rust decoding parity including resource/version rejection and hostile boundaries, policy seam deferral, and test oracle independence. Confirm tests distinguish real integration from test policy harnesses. Run focused Cargo/check commands and targeted independent probes where useful; preserve exact results. All new test families need falsifiable evidence. Do not weaken assertions or infer universal correctness.

Report ACCEPT or CHANGES REQUIRED with concrete paths/lines, reproductions and expected/actual behavior. Flag material limitations explicitly. Core semantics cannot be accepted until blocking/significant findings are corrected and rerun/reviewed. No push, merge, app edits, or additional agents.
