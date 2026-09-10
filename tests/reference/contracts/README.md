# T02 contract fixtures

`cases.json` contains test-only valid/invalid examples for `program-transport.m1/revision-1`. The JSON Schema checks closed structural unions; `scripts/validate-contracts.mjs` adds cross-field rules that JSON Schema cannot conveniently express, including ID uniqueness, program reachability, parameter references/ranges, snapshot ownership, event continuity, and byte/depth/block limits.

Valid cases independently exercise all M1 blocks, a maximum `u64` tick without JSON-number conversion, create and selected-thread advance commands, a large-sequence delta, a source-attributed runtime error, a typed event snapshot, checkpoint completion, and an unsupported-protocol error encoded in the supported response protocol.

Invalid cases challenge unknown schema/protocol versions, JSON-number ticks, `u64` overflow, duplicate block IDs, invalid parameter ranges, missing termination, unknown program references, ambiguous command payloads, unknown delta operations, mismatched event payloads, and broken thread/core ownership. Additional generated negative controls challenge unsupported engine versions, nesting depth 65, 10,001 recursively counted blocks, documents over 1 MiB, and sequence gaps.

Expected validity is derived from the public schema and normative prose rather than a future engine. A production decoder must match these outcomes and preserve the large decimal strings byte-for-byte.
