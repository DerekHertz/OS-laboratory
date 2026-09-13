# T03 decoding and transport compatibility review

Status: **CHANGES REQUIRED**

Reviewed candidate `e269ee1` against baseline `e99d65a`; checkpoint `09383b6` contains documentation only. Review ownership was limited to this report, with one independent probe under ignored `.verification/review-input/`. Kernel scheduling internals were not reviewed.

## Blocking finding

### D1 — resource-limit precedence is field-order dependent

`content/concepts/program-transport-m1.md` routing step 8 requires size/resource limits to be checked first, before non-version workload shape failures. `Workload::from_value` instead validates `machineId` at `crates/sim-core/src/input.rs:121`, before it reaches the 128-parameter resource limit at `crates/sim-core/src/input.rs:263` through `Program::from_value`. A workload containing both defects therefore returns `InvalidCommand`; the same oversized parameter list with an otherwise valid machine returns `ResourceLimit`.

Minimal independent reproduction:

```text
cd .verification/review-input
$env:PATH='C:\Users\Derek\.cargo\bin;'+$env:PATH
cargo run --quiet --offline

resource-only=ResourceLimit
resource-plus-shape=InvalidCommand
```

The probe starts from `V02-workload-preserves-u64`, replaces the first program's declarations with 129 individually valid parameters, then changes `machineId` to the invalid authored ID `1invalid`. Expected for both inputs: `ResourceLimit`, because the resource breach must win routing step 8. Actual for the combined input: `InvalidCommand`.

This is observable create-error behavior defined by the accepted transport contract. Fix by performing a complete resource-limit preflight before ordinary shape validation, while retaining the separately tested schema/model/engine/random compatibility order. Add a mixed-failure regression case; then rerun the focused decoder and contract checks. Other early ordinary-shape validations should be audited against later depth, block-count, program/thread-count, and override-count limits so the result does not depend on field order.

## Accepted scoped observations

- The decoder rejects unknown fields and malformed operands, directly parses canonical decimal strings to `u64`/`i64`, checks positive runtime operands, preserves the parsed workload source, and retains original source-block IDs in immutable compiled instructions.
- Raw input is bounded at 1,048,576 bytes before parsing; JSON structural depth is capped before disabling serde's default recursion limit; semantic repeat depth and recursively counted block limits are independently enforced. Existing boundary cases cover depth 64/65, 10,000/10,001 blocks, byte limit/+1, Unicode scalar name length, core count, parameter count, and numeric lexical/range edges.
- Unsupported workload discriminators are checked in schema, model, engine, random-algorithm order after ordinary body validation. Nested program schema incompatibility is folded into the schema result. D1 is the uncovered resource-versus-shape case.
- The generated-ID correction consistently keeps authored IDs at 128 characters and uses the 160-character generated type for event IDs, causal parents, mixed entity arrays, and trace bounds in the JSON schema and TypeScript projection. The boundary checks cover 160 accepted/161 rejected and authored 128 accepted/129 rejected.
- `scripts/check-kernel.mjs` deletes the prior artifact, runs actual Rust workspace tests with the producer environment path, and only then validates the newly written artifact. `check-kernel-events.mjs` validates every emitted event against the shared schema and checks contiguous IDs/sequences, nondecreasing ticks, declared thread/program/block mapping, deterministic core collision handling, entity order, predecessor links, and presence of all 12 T03-produced kinds. Intervention remains outside the T03 producer set. I inspected this path but did not repeat the already recorded full kernel/browser builds.
- Direct Rust dependencies are exact-pinned (`serde = 1.0.228`, `serde_json = 1.0.149`), the lockfile records transitive versions/checksums, and the repository separately pins Rust 1.94.1 with rustfmt, clippy, and the Wasm target.

## Verification

- `cargo test -p sim-core --locked input::tests -- --nocapture` with `C:\Users\Derek\.cargo\bin` prepended to `PATH`: **PASS**, 4 passed, 13 filtered out.
- `npm.cmd run contracts:check`: **PASS**, 30 contract cases and 10 generated-ID boundary checks; named negative controls rejected.
- Probe setup `cargo run --quiet --locked`: **not run to completion** because the new isolated probe had no lockfile and restricted network access prevented an index fetch. No product conclusion relies on this attempt.
- `cargo run --quiet --offline` in `.verification/review-input`: **PASS as a reproduction**, exit 0 with the two exact decoder results shown above.

The passing finite suites support the covered boundaries but do not override D1. T03 decoding/transport compatibility should be re-reviewed after the precedence fix and focused regression test pass.
