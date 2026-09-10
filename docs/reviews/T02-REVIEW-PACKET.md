# T02 independent contract review packet

Status: ready after author checkpoint. Review target: branch `codex/t02-program-transport-contract`.

Act as an independent tier-A reviewer. Read `AGENTS.md`, specification sections 6–8 and 10, `content/concepts/scheduling-m1.md`, `docs/tasks/T02.md`, `content/concepts/program-transport-m1.md`, the shared schema, TypeScript projection, fixtures, validator, and `docs/verification/T02.md`. Treat author checks as claims, not proof.

Do not edit contracts, fixtures, task state, application, engine, branch, or PR. Write only `docs/reviews/T02-REVIEW-RESULT.md`.

Independently challenge: exact `u64/i64` boundaries; all command/reply discriminants; unknown/obsolete run and protocol behavior; pre-run versus runtime errors; source-block stability; repeat reachability and validation limits; machine namespaces; event payload and sequence ordering; snapshot queue/core ownership; delta gap recovery; checkpoint identity; separate application/schema/protocol/model/engine/random compatibility; and whether T03 can implement without inventing behavior.

For each significant finding provide the smallest JSON counterexample, competing interpretations, expected rejection/transition, affected file/section, and correction scope. Re-run `npm run contracts:check`, `npm run typecheck`, `npm run format:check`, and `git diff --check` only as structural evidence.

Verdict is `ACCEPT`, `CHANGES REQUIRED`, or `CONTRACT BLOCKED`. Acceptance requires one outcome for every challenged boundary, exact agreement between prose/schema/projection/fixtures, no precision loss, and no blocking/significant finding. Do not start T03 or merge T02.
