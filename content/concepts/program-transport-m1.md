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

An exact unknown schema or protocol version is rejected; it is never coerced. A workload naming an unsupported model, engine, or random algorithm is rejected before execution with the corresponding `unsupportedModel`, `unsupportedEngine`, or `unsupportedRandomAlgorithm` code. `applicationVersion` is descriptive and does not gate execution. Saved runs retain every compatibility field. Schema migration changes stored shape explicitly; it never claims engine replay compatibility. An archived result may be viewed without being replayable.

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
- `inspect` uses a closed payload: `{ mode: "full" }` requests an unconditional full snapshot, while `{ mode: "delta", afterSequence }` requests changes after the client's last contiguous sequence and may fall back to a full snapshot.
- `checkpoint` requests an engine checkpoint with an optional text label.
- `cancel` requests terminal cancellation.
- `intervene` revision 1 supports only `setTraceMode`. This changes retention/transfer detail, not simulated outcomes, and is recorded at the next event boundary.

Routing is deterministic and uses this precedence. Request IDs do not provide scheduling order; the worker processes accepted commands serially.

| Step | Condition | Outcome |
| --- | --- | --- |
| 1 | The envelope cannot recover lexically valid `protocolVersion`, `requestId`, and `runId` strings | Discard it without a reply. |
| 2 | The recovered protocol is unsupported | Reply `unsupportedProtocol` using the worker's supported protocol, the recovered IDs, sequence `"0"`, `terminal: false`, and the received version. This precedes run lookup. |
| 3 | The known-protocol command envelope or kind-selected payload is structurally malformed | Reply `invalidCommand`, sequence `"0"`, `terminal: false`. For `create`, this step validates the command envelope and the presence/type of workload compatibility discriminators; step 8 validates the workload body. This precedes run lookup. |
| 4 | The run ID is in the worker-session retired-ID set | Ignore the command without a reply, including a later `create`; retired IDs are the only obsolete IDs and are never reused. |
| 5 | The run ID is current | Apply a non-`create` command to that ledger. A duplicate `create` receives nonterminal `invalidState` at the current ledger sequence. |
| 6 | The run ID is unknown and the command is not `create` | Reply nonterminal `invalidState` at sequence `"0"`; the ID is not thereby reserved or retired. |
| 7 | The run ID is unknown, the command is `create`, and another run is current | Reply nonterminal `invalidState` at sequence `"0"`; creating a replacement first requires cancel/restart retirement of the current run. |
| 8 | The run ID is unknown, the command is `create`, and no run is current | Validate atomically. Size/resource limits are checked first; non-version workload shape errors produce `invalidCommand`. If the compatibility discriminator strings are structurally recoverable, unsupported values are checked in schema, model, engine, then random-algorithm order and produce their dedicated code even though the closed shared schema also rejects the value. Success creates the sole current ledger and reserves the ID permanently for this worker session. |

Clients likewise discard replies for any noncurrent run ID. Cancel and terminal completion/error retire the current ID before another create may succeed; a UI restart is cancel/retire followed by create with a fresh ID.

## Replies, snapshots, deltas, and errors

Every emitted reply names the protocol used to encode the reply and echoes request/run identity as defined above. It has one discriminated kind:

- `ack` confirms acceptance, not completion;
- `progress` reports bounded work progress and the latest sequence;
- `checkpoint` reports the stable checkpoint ID plus its run, model, engine, random-algorithm, tick, and state-sequence bindings after checkpoint creation succeeds;
- `state` carries exactly one full snapshot or delta;
- `completed` carries terminal outcome and final snapshot;
- `cancelled` carries the last confirmed snapshot;
- `error` carries a stable error code, safe message, optional source location, and whether the run is terminal.

Snapshots are authoritative and carry a sequence. They are emitted only at stable boundaries after all four T01 phases for a tick have closed; they never expose an intra-phase state. Consequently, a nonempty ready queue cannot coexist with an idle core. Every queue and core reference resolves to exactly one listed thread, and the bidirectional status/ownership constraints hold.

Deltas carry `baseSequence` and `sequence` and apply only when `baseSequence` equals the client's last contiguous sequence. Otherwise the client discards the delta and sends `inspect` with `{ mode: "full" }`; that request must return a full snapshot, never a delta. Delta operations are an ordered, closed union: replace run summary, set one existing thread, set one existing core, replace the ready queue, append canonical events, or replace trace-retention metadata. Because snapshots enumerate every run thread and core, `setThread` and `setCore` cannot introduce a new identity. A delta is applied atomically in listed order; later operations replace earlier writes to the same run/thread/core/queue/trace target. Appended event batches concatenate in operation order. Intermediate states need not satisfy ownership rules, but the final state must satisfy every full-snapshot invariant, have sequence equal to the delta sequence, and have event/trace continuity. Failure rejects the entire delta and triggers a full resynchronization; no prefix is committed. Unknown operations are invalid rather than ignored.

Errors distinguish validation/protocol failure, unsupported schema/model/engine/random algorithm, resource limits, arithmetic overflow, control-budget exhaustion, invalid state transitions, checkpoint failure, and internal engine failure. Pre-ledger `invalidCommand`, `unsupportedSchema`, `unsupportedModel`, `unsupportedEngine`, `unsupportedRandomAlgorithm`, and `resourceLimit` failures use sequence `"0"` and `terminal: false` because no run exists. `unsupportedProtocol` follows routing step 2. Ledger-scoped `invalidState` and `checkpointFailed` use the current ledger sequence and are nonterminal. Runtime `arithmeticOverflow`, `controlBudgetExceeded`, and `internalEngine` use the current ledger sequence, are terminal, and use T01's run-fatal accounting. `terminal` always describes an existing run, never merely a rejected request; transport errors do not silently turn into completed runs. `receivedProtocolVersion` appears only on `unsupportedProtocol`.

## Events and source mapping

Canonical M1 events carry machine-namespaced `eventId`, contiguous run-local `eventSequence`, `tick`, `kind`, ordered `entityIds`, ordered causal parent IDs, and a kind-specific payload. Event sequence begins at `"1"`; retained event arrays are ascending but may begin later after declared truncation. Causal parent IDs name earlier canonical events in the same run; after trace-prefix truncation they may name a no-longer-retained event, so absence from the retained array is not invalid.

Identity is required by kind. `arrival`, both dispatch events, both compute events, I/O submission/completion, yield, quantum expiration, and termination require `threadId`. Compute start/completion, I/O submission/completion, yield, quantum expiration, and termination also require the active source `blockId`. `arrival` and dispatch events have no block ID. An `arithmeticOverflow` or `controlBudgetExceeded` runtime error requires both the offending thread and block; a global `internalEngine` error has neither, while a thread-attributed one has both. Intervention and run completion have neither. A thread-scoped event includes its thread ID in `entityIds`; core-scoped events also include the machine-namespaced core entity ID chosen by T03. Given the persisted workload, each event thread must exist and every block must belong to that thread's program; producers must reject rather than emit a cross-program mapping.

The closed revision-1 kinds are arrival, dispatch start/completion, compute start/completion, I/O submission/completion, yield, quantum expiration, termination, runtime error, intervention, and run completion. Source-block IDs always refer to the immutable program copy persisted with the run.

Checkpoint IDs are opaque and generated by the engine, never derived from or replaced by the optional user label. Their identity is the tuple `(runId, checkpointId)`; that tuple is never reused, while the same opaque checkpoint token may occur in a different run without collision. The checkpoint envelope is immutable and binds that identity to the exact model/engine/random-algorithm versions, tick, state sequence, complete workload, ordered interventions through that sequence, and engine state. Reusing a tuple for different content is `checkpointFailed`; loading any mismatched binding is rejected as the applicable unsupported-version or checkpoint failure rather than silently replayed. The checkpoint reply sequence equals its `stateSequence` and repeats the identity and compatibility bindings needed to select a compatible checkpoint; T13 defines serialization without weakening them.

Events are batched in replies. Retention metadata states whether the detailed prefix was truncated and the first/last retained event IDs. Trace mode may change storage/detail only; canonical terminal state and metrics must remain identical.

## Persistence and untrusted input

Imports are validated by byte size before JSON parsing where possible, then by schema and semantic limits. Unknown properties are rejected. Text is length-bounded and rendered without HTML interpretation. Revision 1 contains no external URL field. A failed transactional save preserves the previous valid record and the in-memory result. Export records the workload, all compatibility versions, seed/random-algorithm version, ordered interventions, outcome, and trace-retention metadata; T12 will define its storage envelope without changing these fields.

## Verification boundary

`packages/contracts/schema/contracts.schema.json` is the shared machine-readable source. `packages/contracts/src/index.ts` is its TypeScript projection. `scripts/validate-contracts.mjs` compiles the schema with Ajv, checks fixture expectations plus cross-field semantic rules, and executes deliberate negative controls for `u64` overflow and sequence-gap handling. Passing fixtures show bounded agreement, not proof that all malformed inputs are rejected. T03 must add Rust decoding/equivalence tests before claiming native/Wasm compatibility.
