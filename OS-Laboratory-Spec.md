# OS Laboratory — Product and Technical Specification

Status: planning baseline, revision 1. Date: 2026-09-08.

This document and IMPLEMENTATION-TODO.md form the agent handoff. No application has been implemented or benchmarked. Requirements marked **proposed default** resolve implementation details that were not explicitly selected during discussion. Change these through a recorded design decision when evidence warrants it; preserve the accepted product requirements.

## 1. Product

A browser-based, explorable operating-systems laboratory for a learner working at master's-program depth. A miniature character explores a side-on cutaway computer, manipulates policies and programs, and sees execution through an x-ray overlay. The learner improves a computer for chosen workloads, investigates tradeoffs, and compares reproducible experiments.

Accepted decisions:

- Open laboratory with optional guided experiments and unrestricted exploration of implemented mechanisms.
- General OS principles, with clearly identified Linux-specific extensions.
- Editable predefined programs and Scratch-style program construction sharing one execution model.
- Small, deeply inspectable runs and large experiments.
- TypeScript application and Rust simulation engine compiled to WebAssembly, subject to an early feasibility benchmark.
- Source-backed model contracts, tests, and visual mappings for every concept.
- Simulated virtual machines as a later module, separate from virtual memory.
- Future coding visualization through an IDE plugin or desktop pet, after the game.
- Implementation tasks assigned by model capability and bounded scope.

Efficiency is workload-dependent. Do not award a universal score that treats maximum utilization as universally optimal. Benchmark cards define objectives and constraints, such as minimizing response latency while meeting a throughput floor. Show regressions alongside improvements. Do not add energy metrics before defining an explicit energy model.

## 2. Experience and visual rules

The central canvas shows component platforms and connecting passages. CPU bays, RAM frames, storage, and device interfaces provide geography. The kernel is an overlay across managed resources, not a physical room to which processes travel. Virtual address spaces are logical views linked to physical mappings.

Selected art direction: C / Pixel Circuit (see visual-design/C-pixel-circuit.png): midnight hardware, restrained mint activity, pixel-art world and cream CRT service bot with orange backpack, paired with readable modern UI panels. All 15 emotes are approved, including Meow, Paw Paw and Cat Nap; see visual-design/CHARACTER-EMOTES.md, C-character-emotes.png and C-cat-emote.png. Animation timing and automatic triggers remain proposed; production sprites still require authoring. Mockup text and timelines are illustrative, not authoritative simulation data. Prioritize legibility over decorative density.

Layout:

- Top: simulation state, workload name, pause/run, stepping, speed, restart.
- Center: world with pan/zoom, character, stations, and activity overlays.
- Right: selected entity, authoritative state, relationships, controls, model and sources.
- Bottom: expandable timeline, thread lanes, filters, causal event details.
- Program editor and comparison notebook open as resizable panels.

Proposed default movement: left/right or A/D, up/down at lifts, E to interact; clickable stations and direct navigation provide equivalent access. No precision jumping, combat, failure from movement, or mandatory repeated walking. Simulation pause does not pause character exploration. Unimplemented areas are labeled future modules and contain no invented activity.

Selecting a thread highlights its active block, CPU assignment, queue membership, and dependencies. Selecting a resource reveals related work. Follow mode tracks a thread or request and may be canceled without affecting simulation. Every animated semantic signal is derived from engine state/events. Decorative effects must be visually distinct.

Running, runnable, blocked, and terminated states have text/shape cues in addition to color. Provide reduced motion, keyboard-operable lab controls, adjustable text size, and a DOM-based state table as an alternative to the canvas. Target desktop browsers initially; mobile play is out of first-release scope.

## 3. Laboratory workflow

1. Choose a preset, construct a program, or load an experiment card.
2. Inspect its source-backed assumptions and choose a machine configuration.
3. Optionally record a prediction.
4. Run, pause, step, inspect, or intervene.
5. Save the result and compare against a baseline using identical workload parameters.
6. Record an explanation and reveal the reference discussion.

Objective checks evaluate measurable state or trace predicates. Free-text reasoning is stored without mandatory AI grading. Progress records completed experiments and personal benchmarks. No accounts or AI service dependency in the first release. Cosmetic progression may follow; functional sandbox controls are not gated by achievements.

Changing configuration while paused is allowed only for explicitly supported mutable properties. Each change records an intervention at an event boundary. Other edits create a new run. Preserve the previous run for comparison.

## 4. Curriculum and delivery boundaries

| Milestone | Mechanisms | Representative experiments |
|---|---|---|
| M0: foundation | Engine contracts, reference traces, Wasm bridge, benchmarks | Determinism and large scheduling workloads |
| M1: playable lab | Thread lifecycle; FCFS and round-robin; modeled context-switch costs; abstract blocking I/O | Quantum tradeoffs, interactive versus CPU workloads, two-core contention |
| M2: concurrency | Mutexes, condition variables, semaphores, spawn/join, deadlock | Lost updates, producer/consumer, lock ordering; priority inversion after priority scheduling exists |
| M3: virtual memory | Address spaces, paging, TLB model, demand paging, replacement, copy-on-write | Working sets, thrashing, page-fault scheduling interactions |
| M4: storage and I/O | File descriptors, cache, queues, modeled persistence and crashes | Visibility versus durability, journaling, backpressure |
| M5: virtualization | Guest kernels, vCPU scheduling, quotas/contention; then nested translation and virtual devices | Noisy neighbors, overcommitment, guest/host interactions |

Linux extensions attach to individual mechanisms with pinned references. M1 needs a Linux comparison note, not a full Linux scheduler. Advanced memory ordering, NUMA, RCU, and alternative filesystem designs require separate model contracts and are later depth extensions, not implicit M1 commitments.

First-release completion means M1 is complete end to end, including saves, replay, block construction, experiments, correctness tests, and bounded large runs. M0 alone is not a playable release. Later milestones require their own detailed semantics before feature implementation begins.

## 5. Architecture

Proposed default frontend: React with TypeScript for DOM panels, PixiJS for the cutaway, Vite for browser builds. A rendering spike must confirm accessibility integration, asset loading, and worker communication before these choices are locked. Library versions must be pinned after checking supported official releases during implementation.

Suggested repository boundaries:

```text
apps/web/                 TypeScript UI, world, worker host, persistence
crates/sim-core/          Pure Rust state, event queue, interpreter, policies
crates/sim-wasm/          Browser adapter; no simulation rules
crates/sim-cli/           Native runner, batch experiments, benchmarks
packages/contracts/      Generated TypeScript transport definitions
content/concepts/         Model contracts and source records
content/experiments/      Versioned programs, objectives, explanations
tests/reference/         Independently derived expected scenarios
docs/decisions/           Resolved design choices and benchmark reports
```

The Rust engine is authoritative. UI frameworks, browser APIs, animation, storage, and wall-clock time cannot determine model behavior. The interpreter, scheduler, and resource modules operate on shared explicitly owned state. A guest has its own execution context and resource namespace; avoid global assumptions that there is only one scheduler or address space.

A dedicated Web Worker runs one simulation instance. It advances in bounded chunks so pause/cancel requests can be processed. A bounded pool may execute independent batch runs. Do not introduce parallel event processing within a single simulation in M1.

## 6. Initial execution contract — proposed default

Use integer simulated ticks, never browser elapsed time. Tick-to-time conversion is a declared experiment parameter. Tick and sequence counters use checked 64-bit arithmetic internally; transport them as decimal strings where JSON cannot represent them exactly. Reject overflow rather than wrapping.

Events use a total order of `(tick, phase, insertion_sequence)`. At each tick process: (1) completion of already-running work and switch overhead, (2) arrivals and I/O completions, (3) quantum expiration still applicable after completion processing, (4) scheduling dispatch. Process cores in ascending stable ID for otherwise simultaneous completion generation. This ordering is a teaching-model choice, not an OS guarantee. Use generation tokens to ignore stale expiration/completion events.

M1 uses identical cores, a single global FIFO ready queue, and one runnable thread at most once in that queue. FCFS is nonpreemptive except for explicit yield or blocking. Round-robin expiration moves an unfinished thread to the ready-queue tail. A finished compute burst at its quantum boundary completes before expiration is evaluated. Newly ready work precedes expired-thread requeue at the same tick under the phase ordering above.

M1 programs have immutable instruction structure during a run. Compute bursts consume ticks and are preemptible at quantum boundaries. `io_wait(duration)` submits an abstract independent timer and blocks; it does not imply a modeled storage controller. `yield` relinquishes the core; `end` terminates. Branch/loop evaluation is a control operation at the instruction boundary. Place a bounded control-operation budget between time-advancing instructions to detect zero-time loops and report a program error with the offending block.

Switch cost is configured in ticks; M1 charges it on every dispatch, including dispatch from idle. A selected thread is reserved in a `dispatching` state until overhead finishes. This simplifying convention must be visible in the model inspector. No implicit migration/cache penalty in M1. Later cost models must be explicit additions.

Persist the program, full configuration, engine/model version, random algorithm version, seed, and ordered interventions. Identical inputs and versions must produce identical canonical events and terminal state in native and Wasm builds. No determinism promise across engine versions. Unsupported saved-run versions may be viewed as archived results or rejected with a clear explanation, never silently replayed differently.

Stepping controls: next global event; next instruction boundary of the selected thread (other necessary events still execute); next tick with activity. Selected-thread stepping must stop or explain termination, deadlock, or budget exhaustion. Manual dispatch override is a labeled experimental mode and records each choice; introduce it with M2.

## 7. Programs and editor

Presets and the block editor compile to one versioned structured intermediate representation. Store stable source-block IDs so traces map back to blocks. Never evaluate supplied JavaScript or native code.

M1 blocks: compute, abstract I/O wait, yield, bounded repeat, end. Expose named integer parameters with declared ranges, units, and defaults. Workload definitions specify instance counts and arrival times; multiple programs can populate one run. Read-only pseudocode reflects the same representation. Editing a preset creates a user copy.

M2 adds shared/local variables, expressions, spawn/join and synchronization. Define types, integer overflow, operation atomicity, and shared-memory semantics before implementation. Start with an explicitly labeled sequentially consistent model. Weak memory requires a separately selected model, not behavior inherited from Rust or the host CPU.

Reject malformed references, impossible parameter values, excessive nesting and oversized imports before execution. Allow valid programs that race, deadlock, or perform poorly. An empty runnable/event set with unfinished blocked work must produce an inspectable quiescent state; label deadlock only when the implemented dependency model supports that conclusion.

## 8. Events, state, and integrations

Commands carry `protocolVersion`, `requestId`, and `runId`. Required commands: create, start, pause, advance, inspect, checkpoint, cancel, and supported intervention. Replies distinguish acknowledgment, progress, state update, completion, cancellation, and error. Ignore messages belonging to an obsolete run; reject incompatible protocol versions.

Events include event ID, tick, kind, involved entity IDs, optional thread/block IDs, causal parent IDs, and a typed payload. IDs are stable within a run and namespaced by machine/guest. Inspection uses authoritative snapshots/deltas with sequence numbers. A sequence gap triggers resynchronization.

Batch event transfer and state updates; do not send one message per event or serialize full state every frame. Proposed default: at most 10 UI updates per second during unrestricted running, while paused queries return promptly. Selected detail may be more frequent within the same budget. Render density/utilization summaries for large populations.

Future real-code adapters declare capabilities, timestamp domains, trace gaps, and whether information is observed or inferred. They may reuse presentation contracts without pretending real execution supports simulator replay or complete state. Actual Linux booting, instrumentation collectors, IDE packaging, and desktop-pet packaging are excluded from M1–M5.

## 9. Metrics, large runs, and performance

For M1 report completed-thread throughput, initial response time (first user computation minus arrival), turnaround (termination minus arrival), ready-queue waiting, dispatch overhead, blocked time, and per-core useful/overhead/idle utilization. State the denominator and simulated interval. For unfinished work report censored status rather than treating it as completed. Interactive multi-request latency requires explicit request markers in a later program extension.

Inspection mode retains bounded detailed events. Experiment mode computes metrics online with selected tracing. Mode changes must not alter simulated outcomes. Retention limits must be visible; trace truncation cannot silently become an apparently complete timeline. For percentile estimates disclose the estimator or use exact samples within a declared sample limit.

Proposed benchmark envelope: 10,000 threads, 64 modeled cores, and 10 million events per long run. These are unvalidated targets. Do not allocate a sprite per thread or an object per simulated memory byte. Sparse/page-granular memory models are required when memory is introduced.

Proposed budgets to validate in M0: pause/cancel acknowledgment within 250 ms while running; worker live state plus buffers at most 256 MiB for the scheduling envelope; detailed in-memory trace cap 64 MiB. Checkpoints count against a separate declared cap (proposed 128 MiB). Record test hardware, OS, browser/version, build mode, workload, events/sec, peak memory measurement method, and cancellation latency. Establish a supported browser/device baseline from results. Do not claim precise cross-browser peak memory where tooling cannot measure it.

Batch sweeps have explicit concurrency and total-run limits, progress, cancellation, and incremental result saving. Default concurrency is conservatively capped at two workers until measurement supports more. Full replay can rerun from initial conditions; checkpoints accelerate seeking. Large-run trace windows can be regenerated with matching versions and recorded inputs.

## 10. Persistence and robustness

Local-first IndexedDB storage for programs, configuration, notebook entries, progress, and compact run results. Proposed format: JSON export for source experiments and metadata, optional chunked trace export later. Include schema and engine versions. Migrate saved content explicitly and test migrations on fixtures. Use transactional saves and preserve the prior valid record on failure.

Storage quota failures must preserve in-memory work and offer export. Browser data can be cleared or evicted; do not present local storage as a backup. Imported data is untrusted: size/depth limits, schema validation, safe text rendering, and URL protocol checks for links. No execution of imported scripts. On worker crash preserve last confirmed state and offer reproducible restart. Do not silently report a failed run as complete.

Static hosting requires correct Wasm content type and worker asset resolution. No application server or account system is required. Offline support is a later enhancement unless explicitly added; local simulation does not by itself guarantee offline startup.

## 11. Ground truth and sources

Every concept record has: ID, learning objective, precise supported claims, authoritative section references, source edition/commit, model rules, assumptions/omissions, visual mapping, independent expected cases, and verification status. Every lesson refers to concept IDs. Every engine rule has a model-contract reference. Source presence alone does not demonstrate correctness.

Source register:

| ID | Public source | Intended use |
|---|---|---|
| S1 | https://pages.cs.wisc.edu/~remzi/OSTEP/ | Foundations; scheduling chapters 7–10, concurrency 26–32, memory 13–23, I/O and persistence 36–45 |
| S2 | https://pages.cs.wisc.edu/~remzi/OSTEP/Homework/homework.html | Independent algorithm examples, only under reconciled assumptions |
| S3 | https://pdos.csail.mit.edu/6.828/2025/xv6.html | Teaching kernel book and source; pin selected revision |
| S4 | https://www.kernel.org/doc/html/latest/scheduler/ | Locate Linux scheduler references; replace latest links with release/commit references in implemented contracts |
| S5 | https://man7.org/linux/man-pages/man2/fsync.2.html | File synchronization and directory durability distinctions |
| S6 | https://docs.riscv.org/reference/isa/priv/priv-index.html | Locate explicit architecture profiles; pin edition before modeling hardware-specific behavior |

Initial sources were located during planning; complete section-level audits remain implementation prerequisites. POSIX interface requirements and research-paper extensions need additional primary references before their modules are implemented. Do not describe generic toy translation as x86 or RISC-V without the corresponding architecture contract.

Author original explanations and art, linking to source material. Check reuse licenses before copying exercises, code, text, or assets. OSTEP requests links to its public chapters rather than local chapter copies.

Tests include independently calculated tiny scenarios, invariants/property tests, native/Wasm equivalence, replay checks, and UI-state correspondence. Linux observations validate only the tested environment and behavior. Do not require nondeterministic real Linux scheduling traces to match teaching-model schedules exactly.

## 12. Virtualization contract boundary

M5 simulates guests executing the same program language, not full guest binaries. A guest scheduler chooses a guest thread for a vCPU; host scheduling determines when that vCPU receives physical execution. A guest thread receives no CPU service while its vCPU is descheduled. Track host time separately from guest CPU service and explicitly define guest timer behavior before implementation.

M5a covers vCPU overcommitment, resource shares, and guest/host accounting. M5b adds guest-virtual to guest-physical to host-physical mappings after M3 semantics exist. M5c adds explicit virtual device paths after M4. Memory ballooning and host swapping are optional extensions requiring distinct rules. Report modeled overhead rather than suggesting measured hypervisor performance.

## 13. Release acceptance

M1 acceptance requires a user to construct and save a workload, walk or navigate to a station, run FCFS and round-robin, inspect matching block/core/queue state, replay a run, compare metrics, and read the applicable assumptions and sources. Large benchmark runs must finish or cancel without unbounded trace growth or UI lockup on the declared baseline environment.

Required evidence: engine/reference tests; native/Wasm determinism fixtures; program import rejection cases; save/load round trip; worker cancellation and obsolete-message handling; UI integration test; visual review at declared desktop viewport sizes; keyboard and reduced-motion checks; benchmark report. No test can be replaced by a model's confidence statement.

Unresolved choices are bounded: final art assets/name, exact dependency versions, supported browser baseline and measured throughput, later-module semantics, and particular Linux release profiles. Resolve technical choices in their prerequisite tasks. They do not authorize expanding the first release.
