# OS Laboratory — Agent Implementation Backlog

Read OS-Laboratory-Spec.md first. Status: M0 started; T00–T03 integrated; T04 independently accepted locally and awaiting protected integration; later tasks pending. See docs/tasks/, docs/verification/, and docs/HANDOFF.md for current execution state. Implement M0 then M1; later milestones are planned work requiring additional contracts. Task assignments below are recommendations, not automatic agent launches.

## Assignment and execution policy

| Tier | Suitable work | Suggested reasoning |
|---|---|---|
| A: strongest reasoning | Ambiguous contracts, simulation correctness, architecture, independent semantic review | High; increase only for unresolved complexity |
| B: general coding | Bounded feature development against settled interfaces | Medium |
| C: smaller coding | Small explicit changes with direct acceptance checks | Low/medium |

Map these tiers to available models at implementation time and record the mapping in docs/decisions/model-routing.md. Specific names are intentionally not permanent requirements. Evaluate assignments against results and reassign if needed. Independent review of core semantics uses tier A. UI work can also require tier A when design or interaction remains ambiguous.

Every task packet must identify its ID, deliverable, owned paths, prerequisite IDs, relevant spec sections, exact source sections where applicable, tier/reasoning, acceptance criteria, verification commands, reviewer, and escalation conditions. Before coding, expand any missing exact paths/commands from the initialized repository. Record completion evidence and residual limits. Do not mark an entire milestone complete because one task passes.

Escalate to contract resolution if sources conflict, a state transition is undefined, a public interface must change, replay diverges, or a benchmark cannot meet a budget after a targeted diagnosis. Do not let an implementation model silently invent OS semantics. Use a bounded follow-up task. Routine implementation choices within a settled contract do not need user approval.

Parallel work is eligible only after shared contracts land, with disjoint ownership. Integration has a named owner and its own task. The handoff does not instruct agents to launch automatically.

## M0 — contracts and feasibility

### T00 — initialize repository and build paths

- Tier B / medium. Dependencies: none. Own workspace manifests, build scripts, minimal app and Rust crate scaffolds.
- Read spec sections 5, 10. Resolve proposed stack versions from official documentation and record them.
- Deliver browser shell, native runner, Wasm build, and one documented clean-checkout build procedure.
- Acceptance: clean builds; worker loads a trivial Wasm function; no simulation rules in adapters. Reviewer B.

### T01 — scheduling model contract and reference cases

- Tier A / high. Dependencies: none. Own content/concepts scheduling records and reference fixture definitions.
- Read sections 6, 9, 11 and relevant OSTEP scheduling chapters. Audit exact source sections; distinguish sourced algorithms from our tie-breaking conventions.
- Deliver state machine, simultaneous-event rules, dispatch-cost accounting, FCFS/RR queue rules and at least six independently worked cases.
- Cases must cover simultaneous arrivals, quantum/burst equality, nonzero switching, I/O wakeup, idle cores, and two-core dispatch.
- Acceptance: exact timelines and metric calculations; no unresolved semantics. Independent tier-A review before T03.

### T02 — program and transport contracts

- Tier A / high. Dependencies: T01. Own schema definitions and contract generation.
- Read sections 6–8, 10. Define versions, IDs, integer encoding, commands, errors, deltas, source-block mapping, program limits and validation behavior.
- Acceptance: valid/invalid fixtures; generated Rust/TypeScript compatibility or a checked shared schema; no precision loss for large ticks; explicit unknown-version rejection. Reviewer A.

### T03 — event kernel and interpreter

- Tier A / high. Dependencies: T00–T02. Own sim-core event queue, lifecycle and M1 interpreter.
- Deliver checked integer time, stable ordering, stale-event invalidation, compute/I/O/yield/repeat/end, and bounded zero-time execution.
- Acceptance: lifecycle invariants and interpreter fixtures; same seed/input order produces same canonical events. No UI dependencies. Reviewer A.

### T04 — FCFS policy

- Tier B / medium. Dependencies: T03. Own FCFS policy only.
- Deliver policy against T01 contract; no modifications to global ordering without contract review.
- Acceptance: applicable independent reference fixtures pass; switch overhead and ready waiting are separate. Reviewer A.

### T05 — round-robin policy

- Tier B / medium. Dependencies: T03. Own RR policy only.
- Acceptance: quantum expiration, completion tie, yield, and multicore reference fixtures pass; blocked threads cannot dispatch. Reviewer A.

### T06 — metrics and trace retention

- Tier B / medium. Dependencies: T04, T05. Own metrics and retention modules.
- Read section 9. Implement online metrics, explicit unfinished statuses, bounded detailed trace and truncation metadata.
- Acceptance: independently calculated totals; instrumentation mode does not change outcomes; long-run retention reaches a stable bound. Reviewer A for metric definitions, B for implementation.

### T07 — worker bridge and cancellation

- Tier B / medium. Dependencies: T02, T06. Own sim-wasm adapter and worker host.
- Deliver bounded stepping chunks, batched updates, pause/cancel, obsolete-run filtering and state resynchronization.
- Acceptance: protocol integration tests, large-integer transport, responsive cancellation under load and simulated worker failure. Reviewer B.

### T08 — benchmark and architecture gate

- Tier A / high. Dependencies: T07. Own sim-cli benchmarks and report.
- Run scheduling envelope and small reference cases in native and Wasm. Measure section 9 targets with declared environment and limitations.
- Acceptance: report exact workload, throughput, retention, memory method, cancellation and determinism results. Resolve failures before full world implementation. Change a target only through an explicit evidence-backed decision; do not relabel failure as success. Reviewer A.

## M1 — complete playable scheduling laboratory

### T09 — world rendering and interaction prototype

- Tier B / medium; tier A if visual layout remains unresolved. Dependencies: T00, T02, T08. Own world renderer and movement.
- Read section 2. Deliver CPU cutaway, character, stations, pan/zoom, fast navigation and fixed fixture-driven overlays.
- Acceptance: keyboard and pointer interaction; reduced motion; readable at 1280×720 and 1920×1080 proposed viewports; no semantic events invented by renderer. Visual review required.

### T09a — production character sprites and emote controls

- Tier B / medium for animation integration; art authoring requires pixel-art production and visual review. Dependencies: T09. Own character assets, animation controller and emote picker.
- Read visual-design/CHARACTER-EMOTES.md and CAT-EMOTE.md. Pixel Circuit and all 15 expressions are approved. Author consistent sprite frames from the reference boards rather than slicing the boards into game assets.
- Deliver idle/traversal/interaction animations, all 15 emotes, keyboard-accessible picker, movement interruption and reduced-motion static poses. Finalize frame grid, anchors and timing in a short asset contract before authoring frames.
- Acceptance: consistent silhouette and foot anchors; every emote selectable; cosmetic reactions never alter simulation state/time; no movement lock; readable at actual game scale. Automatic triggers remain optional proposed behavior. Visual review required.

### T10 — authoritative inspection and x-ray integration

- Tier B / medium. Dependencies: T07, T09. Own selection, inspector and event-to-world mapping.
- Acceptance: selecting a thread synchronizes block, core and queue; state survives pause; causal links refer to actual events; DOM table works without canvas navigation. Review against recorded trace by A.

### T11 — block editor and presets

- Tier B / medium. Dependencies: T02. Own editor and M1 program fixtures.
- Deliver all M1 blocks, typed parameter controls, pseudocode, preset-copy editing, instance count and arrival controls.
- Acceptance: preset and equivalent constructed program compile identically modulo presentation IDs; malformed input rejected; trace IDs map to blocks. Reviewer B.

### T12 — save/load and import/export

- Tier B / medium. Dependencies: T02, T11. Own persistence and migrations.
- Acceptance: round-trip programs/configuration/notebook; quota failure preserves work; invalid imports fail safely; migrations have fixtures; no arbitrary code execution. Reviewer B.

### T13 — replay and checkpoints

- Tier A / high. Dependencies: T06, T07, T12. Own replay/checkpoint implementation.
- Acceptance: initial replay matches canonical trace; checkpoint replay matches uninterrupted state/events; retention bounds enforced; engine mismatch reported; interventions replay in order. Native/Wasm equivalence required. Reviewer A.

### T14 — timeline and comparison dashboard

- Tier B / medium. Dependencies: T06, T10, T13. Own timeline and comparison UI.
- Acceptance: filters and selected-event inspection; seek uses validated replay; truncated history labeled; identical-config comparison and changed-config labeling; incomplete runs not counted as completed samples. Reviewer B.

### T15 — source inspector and experiment content

- Tier B / medium for experiment predicates; C / low for settled text/link components. Dependencies: T01, T10, T11.
- Own M1 lesson cards and source UI. Deliver CPU competition, interactive-style compute/I/O, and quantum tradeoff scenarios with predictions and reference explanations.
- Acceptance: each claim maps to concept/source IDs, every objective is reachable, assumptions visible; Linux comparison uses checked references without claiming Linux-equivalent simulation. Semantic review A.

### T16 — independent parameter sweeps

- Tier B / medium. Dependencies: T08, T12, T14. Own batch runner orchestration.
- Acceptance: bounded pool, progress/cancel, incremental results, deterministic per-run seeds, equivalent results to serial runs, summary rendering for 10,000-thread workload. Reviewer B.

### T17 — release integration and review

- Tier A / high for semantic audit; B / medium for browser integration. Dependencies: T09–T16, including T09a.
- Own end-to-end tests, release report, documentation, deployment configuration.
- Acceptance: every section 13 criterion demonstrated, source gaps resolved, clean build reproducible, browser baseline documented, all performance limitations reported. Review code and UI with real execution; fixture-only animation is insufficient. Publish/deploy only within user authorization.

## Later milestones — expand before assigning implementation

Each row is a contract task followed by bounded engine, UI, content and integration tasks. A row is not one giant implementation TODO. Exact primitive semantics, reference cases and ownership must be delivered by the contract task first.

| ID | Dependencies | Contract task, tier A/high | Subsequent implementation and acceptance |
|---|---|---|---|
| T18 / M2 | T17 | Define typed shared state, atomicity, mutex/condition/semaphore rules, spawn/join, manual scheduling | B tasks per primitive; A independent interleavings and deadlock review; x-ray must show actual owner/wait chains |
| T19 / M3 | T17 | Define page model, translation profile, TLB/replacement, faults, copy-on-write, sparse limits | B tasks per mechanism; A known address traces and accounting; verify fault-to-I/O-to-scheduling causal path |
| T20 / M4 | T19 | Define files/cache/device queues, persistence guarantees and crash model | B tasks per mechanism; A independently derived crash outcomes; no directory durability implied by file flush |
| T21 / M5a | T19 | Define host/guest clocks, vCPU state and service accounting, shares and overcommit | B implementation; A guest service cannot exceed allocated vCPU service; inspect both scheduling layers |
| T22 / M5b–c | T20, T21 | Define nested mappings and virtual device paths with explicit costs | B implementation; A composed-translation cases and I/O accounting review |
| T23 / Linux profiles | Relevant module | Pin release/source and distinguish contract from approximation | Implement only selected supported behavior; version appears in model inspector and saved runs |
| T24 / future coding adapter | Product released; separate scope decision | Define observations, missing data, clock mapping, privacy and adapter capabilities | Separate plan for IDE/pet and trace collection; no promise of simulator-quality replay for real programs |

## Handoff prompt

“Read OS-Laboratory-Spec.md and this backlog. Execute the next dependency-ready task in scope. Before coding, identify its model contract, owned files and acceptance checks. Keep simulation semantics in sim-core and expose actual state to the UI. Record source references and verification evidence. If a required semantic rule is missing, resolve it as a bounded contract task rather than guessing. Do not expand later milestones into the first release. Finish with changed files, checks, limitations and the next dependency-ready task.”
