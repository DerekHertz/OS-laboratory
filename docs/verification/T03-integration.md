# T03 orchestrator integration verification

Date: 2026-09-11. Branch: codex/t03-event-kernel. Recovery checkpoint44f94a9; candidate source pending. Windows, Node22.20.0/npm10.9.3. Root owns this record; author owns T03.md.

| Family / command | Null hypothesis | Independently expected result | Actual | Issue response |
|---|---|---|---|---|
| Shared contracts (`npm run contracts:check`) | Generated-ID correction or baseline contracts regress. | Original30 fixtures and10 generated-ID checks satisfy declared bounds; deliberate invalid controls rejected. | PASS after recovery; prerequisite mutation evidence in T03-contract-corrections.md. | Correct contract/projection divergence and rerun. |
| TS/format (`npm run typecheck`, `npm run format:check`) | Harness wiring breaks TypeScript or formatting. | Both return0. | PASS after recovery. | Fix and rerun. |
| Native producer integration (`npm run kernel:check`) | Rust produces events that fail the shared schema, source mapping, canonical identity, entity ordering or predecessor rules. | Cargo workspace tests execute real kernel runs, produce a fresh artifact containing every implemented event kind, then independent Ajv validation and metadata checks pass. Missing artifact, empty cases or missing kinds fail. | NOT RUN: awaiting author artifact reconciliation. | Fix producer/schema divergence; never use fabricated output or stale artifact. |
| Full workspace (`npm run check`) | Kernel integration breaks native/Wasm builds, browser probe or required checks. | Formatting/contracts/types/Rustfmt/Clippy/tests/native release/Wasm+web/browser stages all pass. | NOT RUN on candidate. | Diagnose actual stage and rerun after correction. |

PowerShell npm/npx wrapper startup was slow during resumption; npm.cmd with login:false responds promptly. Use npm.cmd on this host as needed; the package scripts and CI remain portable. No pending root command session remains from the focused checks. No remote T03 CI is claimed.
