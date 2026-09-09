# Repository and orchestration policy

Accepted by the user on 2026-09-09: short task branches, protected buildable main, reviewed integration, separate simulation compatibility versions, reproducible builds and recorded evidence. User additionally requires approximately 60% context wind-down and falsifiable tests with concise null hypothesis/expected/actual/remediation reports.

The initial T00 bootstrap uses one branch with disjoint ownership: the implementation agent owns code, manifests, scripts, tests and CI; the orchestrator owns repository instructions and documentation. Future concurrent implementation uses separate worktrees. This avoids introducing multiple competing workspace scaffolds before contracts exist.

Initial remote inspection on 2026-09-09 found no branch protection or rulesets. After PR #1 merged with two successful `build-and-test` runs, `docs/decisions/main-protection.json` was applied and immediately read back through the GitHub API. Main now requires an up-to-date `build-and-test` check and pull requests, linear history, resolved review conversations, and enforcement for administrators; force pushes and deletion are disabled. No release or deployment is implied by a successful build.

Applied configuration: main-protection.json requires an up-to-date `build-and-test` check, pull requests, linear history and resolved review conversations, including for administrators; it disallows force pushes and deletion. Required GitHub approval count is zero because implementation and orchestration currently use the same repository owner's credentials, which cannot approve their own PR. Independent semantic review is recorded as evidence; it is not represented as an independent GitHub identity. Add required human approval when another reviewer participates.
