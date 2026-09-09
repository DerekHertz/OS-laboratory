# Contributing

The accepted product and architecture are in OS-Laboratory-Spec.md. Task status lives in IMPLEMENTATION-TODO.md. Agent operating rules live in AGENTS.md.

Use a short branch such as `codex/t00-build-scaffold` from current main. One bounded task should have one PR with a clear problem, resulting behavior, verification evidence and remaining limitations. Use coherent commits with the task ID; squash when integrating. Keep unrelated dependency upgrades separate.

Use independent worktrees for concurrent implementation and assign exact paths. Shared interfaces must be settled before dependent work starts. The orchestrator integrates and updates the task record after review. Required CI checks and branch protection must be configured before relying on them as an enforced gate.

## Version policy

Application releases use `v0.x.y` during development. M0 is a foundation checkpoint; the first playable release requires all M1 acceptance criteria. Agree a stable baseline before v1.0.0. Never move published tags.

Save, program and worker protocol formats carry explicit versions. Test supported migrations and reject unsupported versions clearly. Exported runs record the engine compatibility version and source commit. A scheduling, ordering or metric correction requires a recorded compatibility decision even if it is described as a bug fix.

Commit dependency lockfiles and pin toolchains. Keep generated build output, caches, local results and secrets out of Git. Commit source assets and independent reference fixtures. Check generated contracts for drift once generation exists.

## Verification and maintenance

Follow the evidence format in docs/verification/TEMPLATE.md. Run checks relevant to the change and report the exact outcomes. Failed tests must retain their original expectation unless an independently reviewed contract correction justifies a change.

Upgrade dependencies in separate PRs. Record architectural decisions under docs/decisions and keep changed contracts synchronized with the specification.

After integration, remove merged task branches and worktrees after verifying that their work is retained. Revert broken integrations through a new change; do not rewrite shared history.

## Build procedure (verified locally for T00)

Install Node 22.20.0 with npm 10.9.3, and Rust through rustup. On Windows, install Visual Studio C++ build tools and a Windows SDK; ensure cargo is on PATH in the terminal running npm. rust-toolchain.toml selects Rust 1.94.1, rustfmt, clippy and the wasm32-unknown-unknown target.

From the repository root:

```text
rustup toolchain install 1.94.1 --profile minimal --component rustfmt,clippy --target wasm32-unknown-unknown
npm ci --include=optional
npx playwright install chromium
npm run check
```

On this Windows host, npm's existing `os` configuration reports `linux`. Replace the install command with `npm ci --include=optional --os=win32`; this overrides that setting only for the command and does not change user configuration. On ordinary hosts, npm's configured OS should match `node -p process.platform`.

On Linux, use `npx playwright install --with-deps chromium` to install browser OS dependencies too. The check runs formatting, type checking, Rust lint/tests, native release and Wasm/browser builds, and real browser tests. Check docs/verification/T00.md for observed results and limitations; these instructions alone are not verification evidence.

After the Wasm build, `npm run dev` starts the development server; use the URL printed in the terminal. Rebuild Rust changes with `npm run build:wasm`. `cargo run -p sim-cli -- 17 25` exercises the native build probe and should print `42`. This arithmetic probe is infrastructure only; it does not implement scheduling or the future simulation protocol.
