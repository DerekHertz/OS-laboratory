# T02 independent contract review result

Review target: `734666f0be6101bb42c8ebedbed32f5012a13e70` (`codex/t02-program-transport-contract`). Reviewer role: independent tier A. Date: 2026-09-10.

## Verdict

**CHANGES REQUIRED**

The checked schema and TypeScript unions are structurally coherent, and the exact integer formats are correct at both signed and unsigned endpoints. However, T02 does not yet give one implementable outcome for every required boundary. A malformed but structurally valid create input can crash the semantic validator; source-bearing events can omit all source/thread identity; unknown-run handling, compatibility error routing, resynchronization, checkpoint identity, and delta consistency remain underspecified. These are contract issues, not T03 implementation choices.

## Significant findings

### F1 — A malformed operand reference can crash workload validation instead of being rejected

Smallest complete counterexample (a structurally valid workload):

```json
{
  "schemaVersion": "os-lab-workload/1",
  "applicationVersion": "a",
  "modelVersion": "sched.m1/revision-1",
  "engineVersion": "sim-engine.0.1.0",
  "randomAlgorithmVersion": "none.v1",
  "seed": "0",
  "machine": {
    "machineId": "m",
    "cores": 1,
    "switchCost": "0",
    "controlOperationBudgetPerTick": "1"
  },
  "policy": { "kind": "fcfs" },
  "programs": [
    {
      "schemaVersion": "os-lab-program/1",
      "programId": "p",
      "name": "p",
      "parameters": [],
      "blocks": [
        { "blockId": "b", "op": "compute", "duration": { "parameter": "x" } },
        { "blockId": "e", "op": "end" }
      ]
    }
  ],
  "threads": [
    { "threadId": "m:t", "programId": "p", "arrival": "0", "parameters": {} }
  ]
}
```

`validateProgram` records `unknown parameter x`, but `validateWorkload` subsequently evaluates `declaration.default` when `declaration` is `undefined` (`scripts/validate-contracts.mjs`, lines 161–162). Competing behaviors are a clean pre-run validation rejection or an uncaught JavaScript exception. The contract requires the former and no ledger. Correction scope: guard the missing declaration before dereference, preserve the semantic error, and add this as an invalid workload/create fixture. Also ensure the production validation API returns a typed `invalidCommand`/validation error rather than leaking an exception.

### F2 — Event/source identity is optional where the trace contract needs it to be authoritative

This schema-valid event is enough to demonstrate the gap:

```json
{
  "eventId": "m:e",
  "eventSequence": "1",
  "tick": "0",
  "kind": "computeStarted",
  "entityIds": [],
  "causalParentIds": [],
  "payload": { "coreId": 0, "generation": "0" }
}
```

It has neither `threadId` nor `blockId`, yet it is a compute event that must support thread selection and source-block mapping. The current prose says those fields are optional for all events and only constrains a block ID if one happens to be present. Implementations may therefore omit both fields and still satisfy the written/schema contract, or may independently decide which event kinds require them. Expected outcome: define required identity per event kind (at minimum thread and active source block for block-executing events), and reject missing or cross-program block references. Affected areas: `program-transport-m1.md` “Events and source mapping”, schema `$defs.event`, the TypeScript event union, semantic validation, and fixtures. Correction should also state which events may legitimately lack a block (for example run completion) and how causal parents behave when a trace prefix is truncated.

### F3 — Unknown, obsolete, and not-yet-created run IDs do not have a total routing rule

Counterexample command:

```json
{
  "protocolVersion": "os-lab-protocol/1",
  "requestId": "q",
  "runId": "never-created",
  "kind": "start",
  "payload": {}
}
```

The prose defines “obsolete” run IDs as ignored, but does not define how a worker proves an ID obsolete, how a never-created ID differs, or whether the command above is ignored or receives `invalidState`. It also does not state whether run routing or protocol rejection wins when both the protocol and run ID are stale. These choices affect reply/no-reply behavior and sequence identity. Expected outcome: add a precedence table covering unsafe envelope, unsupported protocol, no active run, current run, known obsolete run, unknown run, duplicate create/run reuse, and malformed known-protocol command. Affected area: command protocol prose and protocol fixtures; schema shape need not change unless a distinct error code is selected.

### F4 — Pre-run compatibility failures are not mapped completely or consistently

Counterexample create fragment:

```json
{
  "randomAlgorithmVersion": "future.v2"
}
```

The schema correctly rejects it, but the error union has `unsupportedSchema`, `unsupportedModel`, and `unsupportedEngine` with no `unsupportedRandomAlgorithm`, even though random-algorithm compatibility is explicitly separate. `applicationVersion` is intentionally descriptive, but the remaining compatibility dimensions need distinct routing. More generally, the validator only forces terminality for three runtime codes and only forces sequence `"0"` for `unsupportedProtocol`; it accepts, for example, a pre-create `unsupportedEngine` error with `terminal: true` and sequence `"9"`. Competing interpretations are that `terminal` describes a rejected create request versus an existing run, and that unsupported randomness is `invalidCommand`, `unsupportedEngine`, or a missing dedicated code. Expected outcome: define each compatibility failure's code, sequence, and terminal meaning before and after ledger creation; add the missing random-algorithm outcome and negative/positive fixtures. Affected areas: version/error prose, reply schema, TypeScript projection, semantic validator, and fixtures.

### F5 — Snapshot ownership validation admits unowned queue entries, and delta consistency is not defined

Small snapshot counterexample (inside an otherwise ordinary state reply):

```json
{
  "machineId": "m",
  "sequence": "1",
  "tick": "0",
  "status": "running",
  "readyQueue": ["m:ghost"],
  "threads": [],
  "cores": [{ "coreId": 0, "state": "idle" }],
  "events": [],
  "trace": { "truncated": false }
}
```

The schema accepts this and `validateSnapshot` checks only whether listed threads occur in the ready set; it never checks that every ready-queue ID resolves to a listed ready thread. The same class of inconsistency can be emitted in a delta because delta operations are validated without a base snapshot or a normative final-state invariant. Competing interpretations for multiple operations touching the same entity are ordered last-write-wins, rejection, or transient inconsistency followed by repair. Expected outcome: reject unknown queue/core/thread references in full snapshots; state that a delta is applied atomically in listed order and that its final state must satisfy all snapshot ownership invariants (or define a different complete rule); provide base-plus-delta fixtures. Affected areas: snapshot/delta prose, semantic validator, and fixtures.

The accepted `V07-snapshot-with-typed-event` also contains ready work with an idle core. `sched.m1` says a non-error stable boundary cannot contain that pair after phase-4 closure. T02 must say whether snapshots can expose transient intra-tick states; if not, correct the fixture and enforce stable-boundary snapshots. T03 must not decide this independently.

### F6 — Gap recovery cannot unambiguously request the promised full snapshot

Counterexample sequence:

```json
{
  "clientSequence": "8",
  "receivedDelta": { "baseSequence": "7", "sequence": "9", "operations": [] },
  "inspectPayload": { "afterSequence": "8" }
}
```

The delta itself would also fail the current schema because operations must be nonempty; it is abbreviated here to isolate routing. The prose says a gap causes the client to send `inspect` and the worker answers with a full snapshot, while `inspect.afterSequence` is explicitly allowed and a normal state reply may contain either a snapshot or delta. There is no wire-level distinction between “resynchronize fully” and “send changes after 8.” Expected outcome: specify that a gap-recovery inspect omits `afterSequence`, add a `full`/`resync` mode, or require every inspect to return a full snapshot. Add a fixture that proves the selected path cannot return another unusable delta. Affected areas: command/reply prose, inspect schema/projection, and fixtures.

### F7 — Checkpoint IDs have no stated scope or non-reuse rule

Both replies below are independently schema-valid for the same run:

```json
[
  {
    "protocolVersion": "os-lab-protocol/1",
    "requestId": "q1",
    "runId": "r",
    "sequence": "3",
    "kind": "checkpoint",
    "payload": { "checkpointId": "c", "tick": "2" }
  },
  {
    "protocolVersion": "os-lab-protocol/1",
    "requestId": "q2",
    "runId": "r",
    "sequence": "6",
    "kind": "checkpoint",
    "payload": { "checkpointId": "c", "tick": "5" }
  }
]
```

“Stable checkpoint ID” does not say whether IDs are unique per run, globally namespaced, content-addressed, or replaceable by label. Reuse at two ticks makes later restore identity ambiguous. Expected outcome: define checkpoint-ID namespace and uniqueness/non-reuse, bind the checkpoint to run ID, engine/model versions, tick and state sequence, and distinguish user labels from identity. The reply need not duplicate every binding if the stored checkpoint envelope normatively contains it, but that envelope and failure behavior must be fixed before T13. Affected areas: checkpoint prose/projection/schema and fixtures.

## Boundary audit without additional findings

- `u64` accepts exactly `0` through `18446744073709551615`; positive `u64` excludes zero; `i64` accepts exactly `-9223372036854775808` through `9223372036854775807`. Canonical spelling rejects `-0`, signs, whitespace, exponents, leading zeroes, JSON numbers, and overflow. The schema formats and JavaScript `BigInt` checks agree.
- Command and reply top-level discriminants are closed and mutually exclusive. All required command and reply kinds are represented. The shared `simpleCommand` and completed/cancelled branches remain unambiguous because their `kind` values are disjoint.
- Repeat reachability is coherent for positive counts, empty bodies, and necessarily terminating nested bodies. Recursive block count and declared depth use the prose's top-level-depth-zero convention. F1 prevents acceptance of malformed references as a whole.
- Workload thread IDs and event/entity/causal IDs are checked against the machine prefix. Full snapshot thread/core bidirectional ownership is checked for listed entities, subject to F5. Program/block IDs are program-scoped; F2 must make the thread-to-program context mandatory wherever a block is reported.
- Kind-specific event payloads are closed, retained full-snapshot event sequences are checked for contiguity, and untruncated traces must begin at event sequence 1. Cross-reply/delta append continuity still depends on the final-state/delta rule requested in F5.
- Application, workload/program schema, protocol, model, engine, and random-algorithm fields are represented separately. Replay compatibility for saved/checkpointed data is not yet complete because of F4 and F7.

## Structural checks

All checks were run at the fixed author commit and passed:

```text
npm run contracts:check
  PASS — 21 contract cases; generated overflow, version, size, depth,
  block-count, and sequence-gap controls rejected

npm run typecheck
  PASS

npm run format:check
  PASS

git -c safe.directory=D:/codex/_projects/OS-laboratory diff --check
  PASS
```

These results are bounded structural evidence and do not resolve the counterexamples above. T03 must not start until the contract is corrected and independently re-reviewed.

## Re-review addendum — 2026-09-10

Re-review target: exact fixed head `702b2e01f584a78c341cc02a2de16f32c0677bbd`. I inspected both correction commits, `c894a6ac6924943641553d62bf74c10b4fe54afa` and `702b2e01f584a78c341cc02a2de16f32c0677bbd`, against the original packet and every finding above.

### Disposition of F1–F7

- **F1 resolved.** `validateWorkload` now guards a missing operand declaration instead of dereferencing `undefined`. Fixture `I13-unknown-operand-parameter-rejects-without-crash` reproduces the original complete workload and is rejected cleanly. Atomic create rejection remains explicit.
- **F2 resolved for canonical events.** Prose, schema, and the TypeScript discriminated union now require/forbid thread and block identity by event kind. Arithmetic/control-budget errors require the offending thread and block; internal-engine errors are either globally unattributed or carry both. Fixture `I14-compute-event-missing-source-identity` rejects the original source-less compute event. Causal-parent behavior after prefix truncation and cross-program producer validation are explicit.
- **F3 resolved.** The routing precedence table now distinguishes unrecoverable envelopes, unsupported protocols, malformed known-protocol envelopes/payloads, retired IDs, current IDs, unknown IDs, duplicate creates, and create while another run is current. Commit `702b2e0` further separates recoverable create discriminators from workload-body validation and gives deterministic compatibility-check order. Unsupported protocol wins before run lookup; only retired IDs are silently ignored.
- **F4 resolved.** `unsupportedRandomAlgorithm` exists in prose, schema, projection, and fixtures. Pre-ledger compatibility/validation/resource errors are sequence `"0"` and nonterminal; ledger-scoped recoverable errors use the current sequence and remain nonterminal; runtime errors are terminal. `receivedProtocolVersion` is limited to `unsupportedProtocol`. `I16` rejects contradictory pre-ledger terminal/sequence metadata.
- **F5 resolved.** Full-snapshot validation now rejects ghost/non-ready queue references and ready-plus-idle stable states. Snapshots are explicitly post-phase-closure stable boundaries. Deltas apply atomically in order, cannot add thread/core identities, and must produce a final state satisfying the full-snapshot, event, trace, and sequence invariants. `V05` and `I17` exercise valid and invalid base-plus-delta transitions; `702b2e0` closes the remaining new-identity behavior in the fixture applicator.
- **F6 resolved.** Inspect payloads are a closed union of `{ "mode": "full" }` and `{ "mode": "delta", "afterSequence": ... }`. Gap recovery explicitly sends full mode, which must return a snapshot and never another delta. `V10` covers the full-resynchronization request.
- **F7 resolved.** Checkpoint identity is now the non-reusable `(runId, checkpointId)` tuple, so opaque tokens may safely repeat across runs. The immutable envelope binds that tuple to model, engine, random algorithm, workload, interventions, tick, state sequence, and engine state; reply sequence must equal `stateSequence`. Commit `702b2e0` aligns the validator and evidence with tuple identity rather than incorrectly imposing a string-prefix convention.

### Remaining significant finding R1 — terminal error and snapshot source state are not inspectable by contract

The event-level part of F2 is corrected, but the original packet also requires pre-run versus runtime-error behavior, source-block stability, authoritative inspection, and an implementation-ready T03 boundary. Two independently schema-valid messages still expose a gap.

A running thread may omit its active block from an authoritative full snapshot:

```json
{
  "protocolVersion": "os-lab-protocol/1",
  "requestId": "q",
  "runId": "r",
  "sequence": "1",
  "kind": "state",
  "payload": {
    "snapshot": {
      "machineId": "m",
      "sequence": "1",
      "tick": "1",
      "status": "running",
      "readyQueue": [],
      "threads": [{ "threadId": "m:t", "status": "running", "coreId": 0 }],
      "cores": [{ "coreId": 0, "state": "running", "threadId": "m:t" }],
      "events": [],
      "trace": { "truncated": false }
    }
  }
}
```

This passes the schema and semantic ownership checks even though an authoritative running-state inspection cannot identify the active source block. The omission cannot reliably be reconstructed from events because detailed events may be truncated or absent in experiment mode.

A terminal attributable runtime error may likewise omit both its source and its final snapshot:

```json
{
  "protocolVersion": "os-lab-protocol/1",
  "requestId": "q",
  "runId": "r",
  "sequence": "5",
  "kind": "error",
  "payload": {
    "code": "controlBudgetExceeded",
    "message": "budget",
    "terminal": true
  }
}
```

This also passes the schema and semantic reply validation. Yet T01 requires control-budget exhaustion to report the offending thread/block and to leave an inspectable run-fatal boundary. T02 retires a terminally errored run, after which an `inspect` for that run is silently ignored. The contract does not require a final state reply before retirement and the error reply cannot carry a snapshot, so conforming implementations can expose neither the required final error/censored states nor the source location.

Competing interpretations are: `blockId` is optional presentation detail versus mandatory authoritative current-block state; and a terminal error is preceded by an unstated state reply, remains inspectable despite retirement, or needs to carry its final snapshot. T03/T07 must not choose among these.

Expected correction: define the meaning and required presence of `threadState.blockId` by lifecycle state (at minimum require the active block for running/blocked states and enough current/next-block state for authoritative inspection), with schema/projection/fixtures that reject a blockless running snapshot. For terminal runtime errors, require the offending `threadId` and `blockId` for `arithmeticOverflow` and `controlBudgetExceeded`, and guarantee transport of the authoritative final snapshot—either in a distinct runtime-error reply variant or in a mandatory ordered state/error sequence before retirement. Global `internalEngine` may remain unattributed. Add a fixture proving the final error/censored snapshot and source survive trace truncation. Affected areas: `program-transport-m1.md` “Replies, snapshots, deltas, and errors”; `$defs.threadState` and the error reply union in the schema; the TypeScript projection; semantic validation; and reference fixtures.

An independent direct schema probe at `702b2e0` returned `true` for both counterexamples, with no validation errors.

### Re-review structural checks

All prescribed commands passed at exact head `702b2e01f584a78c341cc02a2de16f32c0677bbd`:

```text
npm run contracts:check
  PASS — 28 contract cases; generated overflow, version, random-algorithm,
  size, depth, block-count, and sequence-gap controls rejected

npm run typecheck
  PASS

npm run format:check
  PASS

git -c safe.directory=D:/codex/_projects/OS-laboratory diff --check
  PASS
```

### Final re-review verdict

**CHANGES REQUIRED**

F1–F7 are resolved, but R1 remains significant under the original acceptance rule: authoritative snapshot/source state and the inspectable runtime-error boundary still admit incompatible outcomes. T03 must not start until R1 is corrected and independently re-reviewed.

## Final re-review addendum — 2026-09-10

Final review target: exact head `117f729ff82843d97868bf849a4e0b162baa798d` (`T02: preserve authoritative fatal inspection`). I reviewed the correction from `702b2e01f584a78c341cc02a2de16f32c0677bbd`, reran the R1 counterexamples unchanged, and re-audited F1–F7 and the original packet boundaries.

### R1 disposition

**Resolved.** The prose now defines `blockId` for every lifecycle state: first block before arrival, next/current block while ready/dispatching/running, the active I/O block while blocked, the executed `end` after termination, the offending block after error, and next/current block when censored. The schema makes `threadState.blockId` required, the TypeScript projection agrees, and `I18-running-snapshot-missing-authoritative-block` rejects the original blockless-running-snapshot counterexample.

Terminal runtime errors now carry their authoritative final snapshot before retirement. The schema and TypeScript error union require a snapshot for every runtime error, require source identity for arithmetic/control-budget errors, and allow `internalEngine` to be either globally unattributed or attributed with both thread and block. Semantic validation requires a matching reply/snapshot sequence, snapshot status `error`, empty ready queue, idle cores, the attributed thread in `error` at the same block, every other unfinished thread in `censored`, and previously terminated threads unchanged. `V06` demonstrates the complete fatal boundary with a truncated trace; `I19-terminal-runtime-error-missing-source-and-final-state` rejects the original incomplete terminal error.

Direct schema probes at exact head `117f729` produced:

```text
source-less terminal controlBudgetExceeded reply: rejected
blockless authoritative running snapshot: rejected
```

Both probes returned validation errors at the newly required fields/unions. This closes the ambiguity between retirement, later inspection, and same-reply final-state delivery.

### Regression audit

- **F1 remains resolved:** malformed operand references reject cleanly without a missing-declaration dereference.
- **F2 remains resolved:** canonical event identity rules remain closed by kind; the R1 correction additionally closes authoritative snapshot and terminal-error source identity.
- **F3 remains resolved:** envelope/protocol/run routing and create-body compatibility precedence are unchanged and deterministic.
- **F4 remains resolved:** application/schema/protocol/model/engine/random compatibility and pre-ledger versus ledger/runtime error metadata remain separate.
- **F5 remains resolved:** stable snapshot ownership, ghost rejection, atomic ordered deltas, existing-identity updates, and final-state validation remain intact. Mandatory thread block IDs apply equally to full snapshots and `setThread` delta operations.
- **F6 remains resolved:** `{ "mode": "full" }` is still an unconditional snapshot resynchronization path, distinct from delta inspection.
- **F7 remains resolved:** checkpoint identity remains the immutable, non-reused `(runId, checkpointId)` tuple with exact compatibility and state-sequence bindings.

The exact `u64/i64` formats, command/reply discriminants, repeat reachability and limits, machine namespaces, typed event payloads, causal/event ordering, and separate compatibility dimensions showed no regression. T03 has a defined outcome for every packet boundary reviewed here. Rust decoding/native-Wasm equivalence, stateful worker routing, and checkpoint-ledger non-reuse remain explicitly assigned verification work for T03/T07/T13 rather than unresolved T02 semantics.

### Final structural checks

All prescribed checks passed at exact head `117f729ff82843d97868bf849a4e0b162baa798d`:

```text
npm run contracts:check
  PASS — 30 contract cases; generated overflow, version, random-algorithm,
  size, depth, block-count, and sequence-gap controls rejected

npm run typecheck
  PASS

npm run format:check
  PASS

git -c safe.directory=D:/codex/_projects/OS-laboratory diff --check
  PASS
```

These finite checks are bounded evidence, supplemented by the direct counterexample probes above.

### Final verdict

**ACCEPT**

All original significant findings F1–F7 and residual finding R1 are resolved with agreement across normative prose, schema, TypeScript projection, semantic validator, and fixtures. No blocking or significant finding remains within T02 scope. This verdict approves the fixed contract head for the orchestrator's integration process; it does not itself merge T02 or start T03.
