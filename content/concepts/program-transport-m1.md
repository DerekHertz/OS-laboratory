# M1 program and transport contract

Contract ID: `program-transport.m1`; revision: 1; status: authoring. This contract encodes `sched.m1/revision-1` for worker and persistence boundaries without adding simulation semantics.

## Version and compatibility boundary

Compatibility fields are recorded independently:

- `applicationVersion` identifies the browser/application build that created the input or saved run.
- `schemaVersion` identifies stored program/workload shape: `os-lab-program/1` and `os-lab-workload/1`.
- `protocolVersion` identifies command/reply meaning: `os-lab-protocol/1`.
- `modelVersion` identifies scheduling outcomes: `sched.m1/revision-1`.
- `engineVersion` is the exact compatible engine contract, initially `sim-engine.0.1.0`, and is persisted with a run.
- `randomAlgorithmVersion`, initially `none.v1`, and unsigned 64-bit `seed` are always persisted, even when the selected M1 policy does not consume randomness.

An exact unknown schema or protocol version is rejected; it is never coerced. A workload naming an unsupported model or engine is rejected before execution. Saved runs retain all four values. Schema migration changes stored shape explicitly; it never claims engine replay compatibility. An archived result may be viewed without being replayable.

## Exact integers and identifiers

Ticks, durations, event/inspection sequences, quantum, switch cost, and control-operation budgets cross JSON boundaries as canonical base-10 strings. Unsigned values match `0|[1-9][0-9]*` and must fit `u64`; signed program parameter values use optional `-` and must fit `i64`. Leading zeroes, plus signs, whitespace, exponents, JSON numbers, and values outside the corresponding Rust range are invalid. Counts that are deliberately bounded below 2^53, such as core count and array lengths, remain JSON integers.

IDs are Unicode-free protocol tokens matching `[A-Za-z][A-Za-z0-9._:~-]{0,127}`. IDs are unique in their declared namespace. A workload defines a machine ID plus program and thread IDs; a program defines parameter and source-block IDs. Each thread ID begins with its machine ID and `:`, and run-scoped event IDs use the same machine prefix. This is the M1 form of the machine/guest namespace required by later models. User-facing names and labels are separate bounded text fields and must be rendered as text, never HTML.

## Program language

A program is immutable during one run and contains a nonempty top-level block list. Every block has a stable `blockId` used by events and inspection. The only revision-1 operations are:

- `compute(duration)` with a positive literal or positive integer parameter;
- `ioWait(duration)` with the same operand rule;
- zero-time `yield`;
- bounded `repeat(count, body)`, where count is a positive literal/parameter and body may be empty;
- zero-time `end`.

Execution never evaluates JavaScript, WebAssembly, host expressions, URLs, or native code. Operands are either canonical decimal literals or parameter references. Parameter declarations have signed `i64` minimum/default/maximum values with `minimum <= default <= maximum`; execution-time operands must additionally satisfy the operation's positive requirement. Overrides must name a declared parameter and lie within its range.

Validation limits are constants, not implementation accidents: each encoded program or workload document is at most 1,048,576 UTF-8 bytes; 128 parameters; 10,000 blocks after recursively counting repeat blocks; nesting depth at most 64 with the top-level list at depth zero; user-facing name at most 128 Unicode scalar values; label at most 256; and at most 10,000 thread instances per workload. Duplicate block, parameter, program, or thread IDs are rejected. The top-level flow must necessarily reach `end`. Because repeat counts are positive at execution, a repeat whose body necessarily reaches `end` also necessarily terminates the thread; any block following an already-terminating block or repeat is rejected as unreachable. A finite repeat without a terminating body falls through after its declared count and remains subject to the run-wide per-tick control-operation budget in `sched.m1`.

## Workload and create boundary

A workload records its application/schema/model/engine/random-algorithm versions, seed, machine, policy, program definitions, and thread instances. Cores are 1–64. FCFS has no quantum; RR has one positive useful-service quantum. Switch cost is unsigned and may be zero. The control-operation budget is positive. Each thread references one program, has an unsigned arrival tick, and supplies zero or more checked parameter overrides.

Create validation is atomic and pre-run: any structural error, unknown program/parameter, invalid positive operand, limit breach, or unsupported version rejects the whole create command and produces no run ledger. Valid programs that later block indefinitely or exhaust the runtime control budget are not rejected merely for that behavior.

## Command protocol

Every command has `protocolVersion`, `requestId`, `runId`, `kind`, and the payload selected by `kind`. Request IDs are unique per client session. Run IDs identify one created run and are never reused after cancel/restart.

- `create` supplies the complete workload and trace mode.
- `start` and `pause` request lifecycle transitions.
- `advance` selects the next global event, next active tick, or next instruction boundary for a named thread.
- `inspect` requests a full snapshot, optionally noting the last contiguous sequence the client holds.
- `checkpoint` requests an engine checkpoint with an optional text label.
- `cancel` requests terminal cancellation.
- `intervene` revision 1 supports only `setTraceMode`. This changes retention/transfer detail, not simulated outcomes, and is recorded at the next event boundary.

Commands for an unknown protocol receive `unsupportedProtocol`. That error uses the worker's supported `protocolVersion`, echoes lexically valid request/run IDs, records the received version in the error payload, and uses sequence `"0"` because no run ledger exists. An envelope whose IDs cannot be safely recovered is discarded. Malformed known-protocol commands receive `invalidCommand`, also with sequence `"0"` when creation never produced a ledger. A command for an obsolete run ID is ignored with no reply so that delayed worker messages cannot mutate the active run. Request IDs do not provide scheduling order; the worker processes accepted commands serially.

## Replies, snapshots, deltas, and errors

Every emitted reply names the protocol used to encode the reply and echoes request/run identity as defined above. It has one discriminated kind:

- `ack` confirms acceptance, not completion;
- `progress` reports bounded work progress and the latest sequence;
- `checkpoint` reports the stable checkpoint ID and tick after checkpoint creation succeeds;
- `state` carries exactly one full snapshot or delta;
- `completed` carries terminal outcome and final snapshot;
- `cancelled` carries the last confirmed snapshot;
- `error` carries a stable error code, safe message, optional source location, and whether the run is terminal.

Snapshots are authoritative and carry a sequence. Deltas carry `baseSequence` and `sequence` and apply only when `baseSequence` equals the client's last contiguous sequence. Otherwise the client discards the delta and sends `inspect`; the worker answers with a full snapshot. Delta operations are an ordered, closed union: replace run summary, set one thread, set one core, replace the ready queue, append canonical events, or replace trace-retention metadata. Unknown operations are invalid rather than ignored.

Errors distinguish validation/protocol failure, unsupported schema/model/engine, resource limits, arithmetic overflow, control-budget exhaustion, invalid state transitions, checkpoint failure, and internal engine failure. Runtime overflow and control-budget exhaustion use T01's run-fatal accounting; transport errors do not silently turn into completed runs.

## Events and source mapping

Canonical M1 events carry machine-namespaced `eventId`, contiguous run-local `eventSequence`, `tick`, `kind`, ordered `entityIds`, optional `threadId`, optional `blockId`, ordered causal parent IDs, and a kind-specific payload. Event sequence begins at `"1"`; retained event arrays are ascending but may begin later after declared truncation. The closed revision-1 kinds are arrival, dispatch start/completion, compute start/completion, I/O submission/completion, yield, quantum expiration, termination, runtime error, intervention, and run completion. Source-block IDs always refer to the immutable program copy persisted with the run.

Events are batched in replies. Retention metadata states whether the detailed prefix was truncated and the first/last retained event IDs. Trace mode may change storage/detail only; canonical terminal state and metrics must remain identical.

## Persistence and untrusted input

Imports are validated by byte size before JSON parsing where possible, then by schema and semantic limits. Unknown properties are rejected. Text is length-bounded and rendered without HTML interpretation. Revision 1 contains no external URL field. A failed transactional save preserves the previous valid record and the in-memory result. Export records the workload, all compatibility versions, seed/random-algorithm version, ordered interventions, outcome, and trace-retention metadata; T12 will define its storage envelope without changing these fields.

## Verification boundary

`packages/contracts/schema/contracts.schema.json` is the shared machine-readable source. `packages/contracts/src/index.ts` is its TypeScript projection. `scripts/validate-contracts.mjs` compiles the schema with Ajv, checks fixture expectations plus cross-field semantic rules, and executes deliberate negative controls for `u64` overflow and sequence-gap handling. Passing fixtures show bounded agreement, not proof that all malformed inputs are rejected. T03 must add Rust decoding/equivalence tests before claiming native/Wasm compatibility.
