# T03 prerequisite: generated identifiers and repeat charging

Date: 2026-09-11. Owner: root. Independent contract auditor: `t03_acceptance` (GPT-5.6 Sol / xhigh).

The T02 shared ID limit of 128 cannot encode every valid machine prefix plus a generated event sequence. Rejecting a previously valid workload or hashing its namespace would hide the contradiction. The pre-implementation correction separates authored IDs (unchanged 128-character limit) from generated IDs (same alphabet, maximum 160). Event IDs, causal parents, trace bounds, and mixed entity arrays use the generated bound; thread/block/request/run/machine IDs remain authored IDs.

T03 event IDs are `<machineId>:event:<decimal event sequence>`, at most 155 characters (128 + 7 + 20). Core entity IDs start as `<machineId>:core:<decimal core index>`, at most 136 characters (128 + 6 + 2). If a candidate equals a declared thread ID, append `:` repeatedly until it is distinct from all declared thread IDs. This terminates by length 129 for short base candidates because authored thread IDs have maximum length 128; candidates already longer than 128 cannot collide. Core indices remain distinguishable. Generation is deterministic from the immutable workload, without probabilistic aliases. Event and entity identities occupy separate typed namespaces.

This is a pre-release contract correction, retaining the revision-1 protocol and engine versions: T02 has only schema/fixture consumers, and no simulation producer or saved-run implementation has shipped. Update the schema, TypeScript projection and checks together. Old T02 validators reject some newly permitted generated outputs and must be updated; this is not a general promise of backward compatibility. After a released producer/consumer or saved run exists, an equivalent incompatible output change requires a new compatibility version and explicit rejection/migration. Authored workload acceptance is unchanged.

The separate scheduling clarification specifies one control unit immediately before each repeat iteration body, with no extra exit charge. Empty count N costs N units; nested body operations cost their own units. This preserves C09's explicit four iteration evaluations at count 4. Initial-entry-plus-exit charging was rejected by independent audit because it would add an unspecified fifth operation.

Acceptance evidence is in `docs/verification/T03-contract-corrections.md`. Kernel tests separately verify the actual ID producer and repeat interpreter, so schema fixtures alone do not establish execution correctness.
