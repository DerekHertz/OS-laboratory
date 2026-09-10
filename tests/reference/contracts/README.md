# T02 contract fixtures

`cases.json` contains test-only valid/invalid examples for `program-transport.m1/revision-1`. The JSON Schema checks closed structural unions; `scripts/validate-contracts.mjs` adds cross-field rules that JSON Schema cannot conveniently express, including ID uniqueness, program reachability, parameter references/ranges, snapshot ownership, event continuity, and byte/depth/block limits.

Valid cases independently exercise all M1 blocks, a maximum `u64` tick without JSON-number conversion, create and selected-thread advance commands, a base-plus-large-sequence delta, a source-attributed runtime error, a stable-boundary typed event snapshot, bound checkpoint completion, explicit full resynchronization, and unsupported-protocol/random-algorithm errors encoded in the supported response protocol.

Invalid cases challenge unknown schema/protocol versions, JSON-number ticks, `u64` overflow, duplicate block IDs, invalid parameter ranges, missing termination, unknown program/operand references, ambiguous command payloads, unknown delta operations, mismatched or source-less events, ghost queue entries, bad pre-ledger error metadata, broken thread/core ownership, and a base-plus-delta result that violates stable-boundary ownership. Additional generated negative controls challenge unsupported engine/random-algorithm versions, nesting depth 65, 10,001 recursively counted blocks, documents over 1 MiB, and sequence gaps.

Expected validity is derived from the public schema and normative prose rather than a future engine. A production decoder must match these outcomes and preserve the large decimal strings byte-for-byte.
