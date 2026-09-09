# T00 toolchain baseline

Selected and checked on 2026-09-09. Exact npm versions are in package.json and package-lock.json; Rust selection is in rust-toolchain.toml and Cargo.lock. These are reproducible selections, not a claim that every component is the latest release.

| Component | Version | Source / basis |
|---|---|---|
| Node / npm | 22.20.0 / 10.9.3 | Installed host versions, pinned in .nvmrc and package.json; actual version commands verified by orchestrator. |
| Rust | 1.94.1 | [Official patch announcement](https://blog.rust-lang.org/2026/03/26/1.94.1-release/), verified by implementation agent; rustc/cargo versions and both native/Wasm targets independently checked by orchestrator. |
| React / React DOM | 19.2.8 | [Official release](https://github.com/react/react/releases/tag/v19.2.8), checked by implementation agent. |
| Vite | 8.2.2 | [Official release](https://github.com/vitejs/vite/releases/tag/v8.2.2), checked by implementation agent. |
| TypeScript | 5.9.3 | [Official release](https://github.com/microsoft/TypeScript/releases/tag/v5.9.3), checked by implementation agent. |
| Playwright | 1.63.0 | [Official release](https://github.com/microsoft/playwright/releases/tag/v1.63.0), checked by implementation agent. |
| React / React DOM type definitions | 19.2.18 / 19.2.7 | Publisher npm registry metadata checked by implementation agent. |
| Prettier | 3.9.6 | Publisher npm registry metadata checked by implementation agent. |

CI uses ubuntu-24.04, the pinned Rust/Node/npm versions and lockfile installs. GitHub action v4 refs were resolved through the official repositories' Git refs API: checkout at `11d5960a326750d5838078e36cf38b85af677262`, setup-node at `49933ea5288caeca8642d1e84afbd3f7d6820020`. The workflow pins these commits. Hosted runner images and browser OS dependencies may change; benchmark reports must record their actual environment later.

T00 uses a minimal raw Wasm export for an unsigned wrapping-add build probe, avoiding an additional binding generator before T02 settles transport. The probe does not define simulated time arithmetic (which must be checked u64 per the specification). PixiJS remains deferred to the rendering spike; T00 is not evidence that the proposed full rendering stack is accepted.

Vite must emit the Wasm file externally so the real fetch/failure path is testable. Its asset inline limit is zero. The Wasm build sets an explicit target directory so an ambient CARGO_TARGET_DIR cannot redirect the source artifact unexpectedly.
