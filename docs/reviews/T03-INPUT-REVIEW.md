# T03 decoding and transport compatibility review

Status: **ACCEPT**

Reviewed candidate `e269ee1` against baseline `e99d65a`, the D1 correction in `a9cf705` against `d8b8f09`, and final D1-R1 closure `ec2598a` against `a9cf705`. Review ownership was limited to this report, with one independent probe under ignored `.verification/review-input/`. Kernel scheduling internals were not reviewed.

## Resolved finding

### D1 — resource-limit precedence was field-order dependent

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

Correction `a9cf705` adds a bounded whole-workload resource preflight before any required field is dereferenced. It covers program/thread counts, per-program parameter counts, per-thread override counts, recursively counted blocks, per-list block counts, and repeat depth. Ordinary decoding remains authoritative for shape and compatibility errors.

The new regression independently reconstructs D1 and tests the exact boundary. With 129 valid parameter declarations plus `machineId: "1invalid"`, expected and actual are `ResourceLimit`. With 128 declarations and the same malformed ID, expected and actual are `InvalidCommand`. The correction therefore removes field-order dependence for D1 without moving the valid boundary.

### D1-R1 — preflight treated `body` on a non-repeat block as program nesting

The correction covers every `ResourceLimit` path that detailed `Workload` decoding can reach: raw byte and structural depth checks already run before parsing; the new preflight covers root program/thread arrays, program parameters, thread overrides, repeat depth, each block-list length, and the recursively accumulated block count. Standalone `Program::decode` is outside create-routing step 8 and unchanged.

At `a9cf705`, `preflight_blocks` recursed through every object property named `body` without checking that `op` was `repeat`. Starting from a valid workload, a top-level `{"blockId":"bad","op":"yield","body":[null, ... 10,001 entries]}` followed by `end` remained below the byte limit. Expected was `InvalidCommand` because the values were behind an illegal non-repeat body; actual was `ResourceLimit` from the unconditional preflight.

Closure `ec2598a` guards traversal on a string `op` exactly equal to `repeat`. Its focused regression constructs the 10,001-null fake body, independently confirms the encoded workload is below `MAX_BYTES`, and now gets the expected `InvalidCommand`. The same test retains the accepted 129/128 mixed-failure assertions, so the fix does not regress resource-first precedence.

## Accepted scoped observations

- The decoder rejects unknown fields and malformed operands, directly parses canonical decimal strings to `u64`/`i64`, checks positive runtime operands, preserves the parsed workload source, and retains original source-block IDs in immutable compiled instructions.
- Raw input is bounded at 1,048,576 bytes before parsing; JSON structural depth is capped before disabling serde's default recursion limit; semantic repeat depth and recursively counted block limits are independently enforced. Existing boundary cases cover depth 64/65, 10,000/10,001 blocks, byte limit/+1, Unicode scalar name length, core count, parameter count, and numeric lexical/range edges.
- Unsupported workload discriminators are checked in schema, model, engine, random-algorithm order after ordinary body validation. Nested program schema incompatibility is folded into the schema result. D1 and D1-R1 are covered by focused regressions.
- The generated-ID correction consistently keeps authored IDs at 128 characters and uses the 160-character generated type for event IDs, causal parents, mixed entity arrays, and trace bounds in the JSON schema and TypeScript projection. The boundary checks cover 160 accepted/161 rejected and authored 128 accepted/129 rejected.
- `scripts/check-kernel.mjs` deletes the prior artifact, runs actual Rust workspace tests with the producer environment path, and only then validates the newly written artifact. `check-kernel-events.mjs` validates every emitted event against the shared schema and checks contiguous IDs/sequences, nondecreasing ticks, declared thread/program/block mapping, deterministic core collision handling, entity order, predecessor links, and presence of all 12 T03-produced kinds. Intervention remains outside the T03 producer set. I inspected this path but did not repeat the already recorded full kernel/browser builds.
- Direct Rust dependencies are exact-pinned (`serde = 1.0.228`, `serde_json = 1.0.149`), the lockfile records transitive versions/checksums, and the repository separately pins Rust 1.94.1 with rustfmt, clippy, and the Wasm target.

## Verification

- `cargo test -p sim-core --locked input::tests -- --nocapture` with `C:\Users\Derek\.cargo\bin` prepended to `PATH`: **PASS**, 4 passed, 13 filtered out.
- `npm.cmd run contracts:check`: **PASS**, 30 contract cases and 10 generated-ID boundary checks; named negative controls rejected.
- Probe setup `cargo run --quiet --locked`: **not run to completion** because the new isolated probe had no lockfile and restricted network access prevented an index fetch. No product conclusion relies on this attempt.
- `cargo run --quiet --offline` in `.verification/review-input`: **PASS as a reproduction**, exit 0 with the two exact decoder results shown above.
- Re-review command `cargo test -p sim-core input::tests --locked` with `C:\Users\Derek\.cargo\bin` prepended to `PATH`: **PASS**, 5 passed, 13 filtered out. This includes `resource_limit_preflight_precedes_unrelated_shape_errors` and all previously accepted decoder families, but has no malformed non-repeat-body case.
- Final closure command `cargo test -p sim-core resource_limit_preflight_precedes_unrelated_shape_errors --locked` with the same `PATH`: **PASS**, 1 passed, 17 filtered out. Expected/actual: 129 parameters plus malformed machine ID is `ResourceLimit`; 128 plus malformed ID is `InvalidCommand`; oversized fake body on `yield` below the raw byte cap is `InvalidCommand`.

D1 and D1-R1 are corrected, and no residual blocking finding remains in the assigned decoding and transport scope. Deferred kernel, worker, policy, replay, and browser claims remain outside this review.
