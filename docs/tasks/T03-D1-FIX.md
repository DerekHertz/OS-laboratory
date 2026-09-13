# T03-D1 — resource-limit precedence fix

Date: 2026-09-13. Dependency: T03 author candidate `e269ee1` plus independent D-review finding D1. Branch/worktree unchanged. Root integrates; `t03_input_review` re-reviews.

- Tier B bounded correction: GPT-5.6 Luna / high. Agent `t03_d1_fix`.
- Owned paths: `crates/sim-core/src/input.rs`, `crates/sim-core/src/input_tests.rs`, and a concise D1 entry appended to `docs/verification/T03.md`. Do not edit kernel, contracts, reviewer reports, task/handoff/routing files, manifests or lockfile.
- Contract: `program-transport.m1` create routing step 8 requires size/resource limits before non-version workload shape errors, while compatibility discriminator checks retain schema/model/engine/random order after recoverable shape.
- Reproduction: 129 parameters returns `ResourceLimit`; adding an invalid machine ID currently returns `InvalidCommand`. Deliver a bounded whole-input resource-limit preflight so any detectable byte/depth/count limit wins over unrelated shape failures. Avoid implementing command routing or silently changing which violations are resource limits.
- Acceptance: mixed invalid-machine plus 129-parameter input returns `ResourceLimit`; current valid/invalid fixture parity remains; exact count/depth/byte boundaries remain; version ordering remains. Add independently expected mixed-failure regression and a meaningful negative control when practical.
- Checks: `cargo fmt --all -- --check`; `cargo clippy -p sim-core --all-targets --locked -- -D warnings`; `cargo test -p sim-core input::tests --locked`. Do not run full browser/build suites. Commit the bounded fix and report exact results.
- Escalate if enforcing global precedence requires changing accepted resource classifications/schema or unbounded traversal; checkpoint before expanding scope.
