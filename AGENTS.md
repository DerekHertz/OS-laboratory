# Implementation operating rules

Read OS-Laboratory-Spec.md, IMPLEMENTATION-TODO.md and docs/HANDOFF.md before work.

- The orchestrator owns task readiness, assignment, review and integration. Every task packet names owned paths, dependencies, contract references, acceptance checks, reviewer and escalation conditions.
- Use short `codex/<task-id>-<description>` branches and separate worktrees for concurrent implementation. Do not let agents edit the same paths. Initial repository scaffolding may share one branch with an explicitly recorded, disjoint documentation/code split.
- Land shared contracts before dependent parallel implementation. Keep simulation semantics in sim-core. Resolve undefined behavior through a documented contract decision.
- Keep main buildable. Integrate reviewed task PRs with required checks; squash with task ID. Revert faulty integrations without rewriting shared history. Do not claim remote protection exists without checking it.
- Pin dependencies/toolchains and commit lockfiles. Record application, schema, protocol and engine compatibility separately; never silently replay incompatible saved runs.

## Context and handoff

At approximately 60% context usage, stop assigning or starting tasks. Finish the current bounded unit when feasible, run its relevant checks, collect agent results and prepare a safe handoff. If finishing would exceed safe capacity, checkpoint explicitly as incomplete. Exact context telemetry may be unavailable: use a conservative estimate, never invent a percentage. This rule applies to every agent as well as the orchestrator.

Update docs/HANDOFF.md with branch and commit, dirty files, completed work, exact verification commands/results, blockers, outstanding agents, and the next dependency-ready task. Leave no untracked background activity unexplained. Do not start a new session automatically.

## Falsifiable verification

For every test or clearly identified parameterized test family, record concise English evidence in docs/verification/: null hypothesis (the failure claim being challenged), what is exercised, expected result, actual result, and how failures will be addressed. Here “null hypothesis” is a testable engineering claim, not a claim of statistical significance.

Expected outcomes must come from independently worked cases, public contracts or general invariants, not the production function under test. Use varied inputs, boundary cases and failure paths. Where meaningful, deliberately break a behavior or remove a dependency and confirm the relevant test fails; restore it and rerun. Never substitute a hardcoded success value, mock, or snapshot for the real integration being claimed. Distinguish not-run and blocked from passing. Passing finite tests is bounded evidence, not a proof of universal correctness.

Core semantics require independent review before integration. Test failures remain open until corrected and rerun, or explicitly recorded as a blocker; do not weaken assertions to pass.
