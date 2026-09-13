# Initial model routing

Use the session's inherited model for this initial bootstrap. Do not change model settings silently. Tier B responsibilities: bounded T00 scaffolding. Orchestrator responsibilities: governance, integration review and verification audit. Tier A responsibilities, when assigned: scheduling contracts and independent semantic review. Tier C work has not been assigned.

Capability tiers describe assignments, not evidence of correctness. Record each actual agent assignment and review result with the task. The initial implementation agent is `t00_scaffold`; its model settings are inherited from the orchestrator. No unverified exact model identity is asserted here.

T01 contract authoring is assigned to the primary orchestrator as tier A / high responsibility using the session's inherited settings. The required independent tier-A semantic reviewer has not yet been assigned; T01 cannot be accepted and T03 cannot start until that review is recorded.

Review routing is portable and capability-based; model names are dated mappings rather than requirements. The reviewer uses a fresh context, a flagship complex-reasoning model, and `xhigh` effort when supported. As of 2026-09-09, the economical first-pass mapping is GPT-5.6 Sol at `xhigh`; GPT-6 Astra at `xhigh` is reserved for targeted escalation when competing derivations, source conflicts, or blocking findings remain unresolved. `max` is not the default. See `docs/reviews/T01-REVIEW-PACKET.md` for the complete policy and current official references.

T01's focused re-review accepted the corrected contract. T02 program/transport contract authoring is assigned to the primary orchestrator as tier A / high responsibility using inherited settings. Its independent review uses the same capability-based tier-A policy; see `docs/reviews/T02-REVIEW-PACKET.md`.

## T03 routing — 2026-09-11

The user explicitly requested model-appropriate subagents with the primary agent orchestrating. Available tool model metadata is the capability basis for this assignment, not a benchmark claim. `t03_kernel` uses GPT-6 Astra / high for the event kernel and interpreter (tier A). `t03_acceptance` uses GPT-5.6 Sol / xhigh in a fresh context for independent contract/acceptance preparation. A fresh GPT-5.6 Sol / xhigh reviewer will audit the implementation checkpoint; unresolved semantic findings escalate to GPT-6 Astra / xhigh. Root owns readiness, bounded contract decisions, task packet, final verification, and handoff. T03 implementation uses its own worktree and branch; the acceptance auditor is read-only. No other implementation task is dispatched before this bounded unit is reviewed.

## T03 resumed review — 2026-09-13

The prior implementation review stopped at the shared usage limit without a verdict. The user requested resumption with model-appropriate delegation. `t03_kernel_review` and `t03_input_review` now use fresh GPT-5.6 Sol / xhigh contexts for separate, bounded tier-A reviews of core semantics and decoding/transport compatibility. Each owns only its named report and saves partial coverage before substantial probing; both read the same unchanged author candidate. Root handles review integration, final checks and handoff. Routine fixes will use a bounded coding assignment; undefined semantics or competing derivations require targeted tier-A resolution. Do not infer live account allowance from historical user readings.
