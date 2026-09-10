import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import Ajv2020 from "ajv/dist/2020.js";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const schemaPath = path.join(
  root,
  "packages/contracts/schema/contracts.schema.json",
);
const casesPath = path.join(root, "tests/reference/contracts/cases.json");
const schema = JSON.parse(fs.readFileSync(schemaPath, "utf8"));
const fixture = JSON.parse(fs.readFileSync(casesPath, "utf8"));
const U64_MAX = 18_446_744_073_709_551_615n;
const I64_MIN = -9_223_372_036_854_775_808n;
const I64_MAX = 9_223_372_036_854_775_807n;

function canonicalInteger(value, minimum, maximum, positive = false) {
  if (typeof value !== "string") return false;
  const pattern = minimum < 0n ? /^-?(0|[1-9][0-9]*)$/ : /^(0|[1-9][0-9]*)$/;
  if (!pattern.test(value) || value === "-0") return false;
  const parsed = BigInt(value);
  return parsed >= minimum && parsed <= maximum && (!positive || parsed > 0n);
}

const ajv = new Ajv2020({ allErrors: true, strict: true });
ajv.addFormat("uint64-decimal", {
  type: "string",
  validate: (value) => canonicalInteger(value, 0n, U64_MAX),
});
ajv.addFormat("positive-uint64-decimal", {
  type: "string",
  validate: (value) => canonicalInteger(value, 0n, U64_MAX, true),
});
ajv.addFormat("int64-decimal", {
  type: "string",
  validate: (value) => canonicalInteger(value, I64_MIN, I64_MAX),
});
ajv.addSchema(schema);

const validators = new Map(
  ["program", "workload", "command", "reply"].map((target) => [
    target,
    ajv.compile({ $ref: `${schema.$id}#/$defs/${target}` }),
  ]),
);

function validateProgram(program) {
  const errors = [];
  const encodedBytes = Buffer.byteLength(JSON.stringify(program), "utf8");
  if (encodedBytes > 1_048_576) errors.push("program exceeds byte limit");
  const parameters = new Map();
  for (const parameter of program.parameters) {
    if (parameters.has(parameter.id))
      errors.push(`duplicate parameter ${parameter.id}`);
    const minimum = BigInt(parameter.minimum);
    const defaultValue = BigInt(parameter.default);
    const maximum = BigInt(parameter.maximum);
    if (!(minimum <= defaultValue && defaultValue <= maximum)) {
      errors.push(`invalid range for ${parameter.id}`);
    }
    parameters.set(parameter.id, parameter);
  }

  const blockIds = new Set();
  const parameterOperands = [];
  let blockCount = 0;
  function visit(blocks, depth, topLevel = false) {
    if (depth > 64) errors.push("program nesting exceeds 64");
    let necessarilyTerminates = false;
    blocks.forEach((block, index) => {
      blockCount += 1;
      if (blockIds.has(block.blockId))
        errors.push(`duplicate block ${block.blockId}`);
      blockIds.add(block.blockId);
      if (necessarilyTerminates)
        errors.push(`unreachable block ${block.blockId} at index ${index}`);
      for (const key of ["duration", "count"]) {
        const operand = block[key];
        if (operand?.parameter) parameterOperands.push(operand.parameter);
      }
      if (block.op === "end") necessarilyTerminates = true;
      if (block.op === "repeat" && visit(block.body, depth + 1)) {
        necessarilyTerminates = true;
      }
    });
    if (topLevel && !necessarilyTerminates)
      errors.push("top-level path lacks end");
    return necessarilyTerminates;
  }
  visit(program.blocks, 0, true);
  if (blockCount > 10_000) errors.push("program exceeds block limit");
  for (const id of parameterOperands) {
    const parameter = parameters.get(id);
    if (!parameter) errors.push(`unknown parameter ${id}`);
    else if (BigInt(parameter.default) <= 0n) {
      errors.push(`nonpositive default for operand ${id}`);
    }
  }
  return errors;
}

function operandsIn(program) {
  const operands = [];
  function visit(blocks) {
    for (const block of blocks) {
      if (block.duration) operands.push(block.duration);
      if (block.count) operands.push(block.count);
      if (block.body) visit(block.body);
    }
  }
  visit(program.blocks);
  return operands;
}

function validateWorkload(workload) {
  const errors = [];
  if (Buffer.byteLength(JSON.stringify(workload), "utf8") > 1_048_576) {
    errors.push("workload exceeds byte limit");
  }
  const programs = new Map();
  for (const program of workload.programs) {
    if (programs.has(program.programId))
      errors.push(`duplicate program ${program.programId}`);
    programs.set(program.programId, program);
    errors.push(...validateProgram(program));
  }
  const threadIds = new Set();
  const machinePrefix = `${workload.machine.machineId}:`;
  for (const thread of workload.threads) {
    if (!thread.threadId.startsWith(machinePrefix)) {
      errors.push(`thread outside machine namespace ${thread.threadId}`);
    }
    if (threadIds.has(thread.threadId))
      errors.push(`duplicate thread ${thread.threadId}`);
    threadIds.add(thread.threadId);
    const program = programs.get(thread.programId);
    if (!program) {
      errors.push(`unknown program ${thread.programId}`);
      continue;
    }
    const declarations = new Map(
      program.parameters.map((item) => [item.id, item]),
    );
    for (const [id, value] of Object.entries(thread.parameters)) {
      const declaration = declarations.get(id);
      if (!declaration) {
        errors.push(`unknown override ${id}`);
        continue;
      }
      const parsed = BigInt(value);
      if (
        parsed < BigInt(declaration.minimum) ||
        parsed > BigInt(declaration.maximum)
      ) {
        errors.push(`override outside range ${id}`);
      }
    }
    for (const operand of operandsIn(program)) {
      if (!operand.parameter) continue;
      const declaration = declarations.get(operand.parameter);
      const value = thread.parameters[operand.parameter] ?? declaration.default;
      if (BigInt(value) <= 0n)
        errors.push(`nonpositive runtime operand ${operand.parameter}`);
    }
  }
  return errors;
}

function validateReply(reply) {
  const errors = [];
  const snapshot = reply.payload?.snapshot;
  if (snapshot) {
    errors.push(...validateSnapshot(snapshot));
    if (reply.sequence !== snapshot.sequence)
      errors.push("reply/snapshot sequence mismatch");
  }
  if (reply.kind === "completed" && snapshot?.status !== "completed") {
    errors.push("completed reply lacks completed snapshot");
  }
  if (reply.kind === "cancelled" && snapshot?.status !== "cancelled") {
    errors.push("cancelled reply lacks cancelled snapshot");
  }
  if (reply.kind === "state" && reply.payload.delta) {
    const delta = reply.payload.delta;
    if (BigInt(delta.sequence) <= BigInt(delta.baseSequence)) {
      errors.push("delta sequence does not advance");
    }
    if (reply.sequence !== delta.sequence)
      errors.push("reply/delta sequence mismatch");
  }
  if (reply.kind === "error") {
    const runtimeCodes = new Set([
      "arithmeticOverflow",
      "controlBudgetExceeded",
      "internalEngine",
    ]);
    if (runtimeCodes.has(reply.payload.code) && !reply.payload.terminal) {
      errors.push("runtime error is not terminal");
    }
    if (
      reply.payload.code === "unsupportedProtocol" &&
      (reply.sequence !== "0" || !reply.payload.receivedProtocolVersion)
    ) {
      errors.push("unsupported protocol error lacks pre-run metadata");
    }
  }
  return errors;
}

function validateSnapshot(snapshot) {
  const errors = [];
  const machinePrefix = `${snapshot.machineId}:`;
  const threads = new Map();
  for (const thread of snapshot.threads) {
    if (!thread.threadId.startsWith(machinePrefix)) {
      errors.push(
        `snapshot thread outside machine namespace ${thread.threadId}`,
      );
    }
    if (threads.has(thread.threadId))
      errors.push(`duplicate snapshot thread ${thread.threadId}`);
    threads.set(thread.threadId, thread);
  }
  const cores = new Map();
  for (const core of snapshot.cores) {
    if (cores.has(core.coreId))
      errors.push(`duplicate snapshot core ${core.coreId}`);
    cores.set(core.coreId, core);
    if (core.state === "idle" && core.threadId !== undefined) {
      errors.push(`idle core ${core.coreId} owns a thread`);
    }
    if (core.state !== "idle" && core.threadId === undefined) {
      errors.push(`busy core ${core.coreId} lacks a thread`);
    }
  }
  const ready = new Set(snapshot.readyQueue);
  for (const thread of snapshot.threads) {
    const isReady = thread.status === "ready";
    if (ready.has(thread.threadId) !== isReady) {
      errors.push(`ready ownership mismatch ${thread.threadId}`);
    }
    const ownsCore =
      thread.status === "dispatching" || thread.status === "running";
    if (ownsCore !== (thread.coreId !== undefined)) {
      errors.push(`core ownership mismatch ${thread.threadId}`);
      continue;
    }
    if (ownsCore) {
      const core = cores.get(thread.coreId);
      const expectedState =
        thread.status === "running" ? "running" : "dispatching";
      if (core?.threadId !== thread.threadId || core?.state !== expectedState) {
        errors.push(`thread/core cross-reference mismatch ${thread.threadId}`);
      }
    }
  }
  for (const core of snapshot.cores) {
    if (core.threadId === undefined) continue;
    const thread = threads.get(core.threadId);
    const expectedStatus = core.state === "running" ? "running" : "dispatching";
    if (thread?.coreId !== core.coreId || thread?.status !== expectedStatus) {
      errors.push(`core/thread cross-reference mismatch ${core.coreId}`);
    }
  }
  const eventIds = new Set();
  let previousEventSequence;
  for (const event of snapshot.events) {
    if (!event.eventId.startsWith(machinePrefix)) {
      errors.push(`event outside machine namespace ${event.eventId}`);
    }
    for (const entityId of event.entityIds) {
      if (!entityId.startsWith(machinePrefix)) {
        errors.push(`entity outside machine namespace ${entityId}`);
      }
    }
    for (const parentId of event.causalParentIds) {
      if (!parentId.startsWith(machinePrefix)) {
        errors.push(`causal parent outside machine namespace ${parentId}`);
      }
    }
    if (event.threadId && !event.threadId.startsWith(machinePrefix)) {
      errors.push(`event thread outside machine namespace ${event.threadId}`);
    }
    if (eventIds.has(event.eventId))
      errors.push(`duplicate event ${event.eventId}`);
    eventIds.add(event.eventId);
    const eventSequence = BigInt(event.eventSequence);
    if (
      previousEventSequence !== undefined &&
      eventSequence !== previousEventSequence + 1n
    ) {
      errors.push(`noncontiguous event sequence ${event.eventSequence}`);
    }
    previousEventSequence = eventSequence;
  }
  if (snapshot.events.length) {
    if (!snapshot.trace.truncated && snapshot.events[0].eventSequence !== "1") {
      errors.push("untruncated trace does not begin at event sequence 1");
    }
    if (snapshot.trace.firstEventId !== snapshot.events[0].eventId) {
      errors.push("trace first event mismatch");
    }
    if (snapshot.trace.lastEventId !== snapshot.events.at(-1).eventId) {
      errors.push("trace last event mismatch");
    }
  } else if (snapshot.trace.firstEventId || snapshot.trace.lastEventId) {
    errors.push("empty trace names retained events");
  }
  return errors;
}

function semanticErrors(target, value) {
  if (target === "program") return validateProgram(value);
  if (target === "workload") return validateWorkload(value);
  if (target === "command" && value.kind === "create") {
    return validateWorkload(value.payload.workload);
  }
  if (target === "reply") return validateReply(value);
  return [];
}

let failed = 0;
for (const testCase of fixture.cases) {
  const validate = validators.get(testCase.target);
  if (!validate) throw new Error(`Unknown fixture target ${testCase.target}`);
  const structural = validate(testCase.value);
  const semantic = structural
    ? semanticErrors(testCase.target, testCase.value)
    : [];
  const actual = structural && semantic.length === 0;
  if (actual !== testCase.valid) {
    failed += 1;
    const detail = structural
      ? semantic.join("; ")
      : ajv.errorsText(validate.errors, { separator: "; " });
    console.error(
      `${testCase.id}: expected ${testCase.valid}, got ${actual}: ${detail}`,
    );
  }
}

const maxCase = structuredClone(
  fixture.cases.find((item) => item.id === "V02-workload-preserves-u64"),
);
maxCase.value.threads[0].arrival = "18446744073709551616";
if (validators.get("workload")(maxCase.value)) {
  failed += 1;
  console.error("negative control: u64 overflow was accepted");
}

const engineCase = structuredClone(
  fixture.cases.find((item) => item.id === "V02-workload-preserves-u64"),
);
engineCase.value.engineVersion = "sim-engine.9.9.9";
if (validators.get("workload")(engineCase.value)) {
  failed += 1;
  console.error("negative control: unsupported engine was accepted");
}

let nestedBlock = { blockId: "depth.end", op: "end" };
for (let depth = 65; depth >= 0; depth -= 1) {
  nestedBlock = {
    blockId: `depth.repeat.${depth}`,
    op: "repeat",
    count: { literal: "1" },
    body: [nestedBlock],
  };
}
const deepProgram = {
  schemaVersion: "os-lab-program/1",
  programId: "program.deep",
  name: "Depth negative control",
  parameters: [],
  blocks: [nestedBlock],
};
if (!validateProgram(deepProgram).includes("program nesting exceeds 64")) {
  failed += 1;
  console.error("negative control: excessive nesting was accepted");
}

const manyBody = Array.from({ length: 9_999 }, (_, index) => ({
  blockId: `many.yield.${index}`,
  op: "yield",
}));
manyBody.push({ blockId: "many.end", op: "end" });
const manyBlocksProgram = {
  schemaVersion: "os-lab-program/1",
  programId: "program.many",
  name: "Block-count negative control",
  parameters: [],
  blocks: [
    {
      blockId: "many.repeat",
      op: "repeat",
      count: { literal: "1" },
      body: manyBody,
    },
  ],
};
if (
  !validateProgram(manyBlocksProgram).includes("program exceeds block limit")
) {
  failed += 1;
  console.error("negative control: excessive block count was accepted");
}

const largeBlocks = Array.from({ length: 8_000 }, (_, index) => ({
  blockId: `large.${index}.${"x".repeat(100)}`,
  op: "yield",
}));
largeBlocks.push({ blockId: "large.end", op: "end" });
const largeProgram = {
  schemaVersion: "os-lab-program/1",
  programId: "program.large",
  name: "Byte-size negative control",
  parameters: [],
  blocks: largeBlocks,
};
if (!validateProgram(largeProgram).includes("program exceeds byte limit")) {
  failed += 1;
  console.error("negative control: excessive byte size was accepted");
}

function canApplyDelta(currentSequence, delta) {
  return currentSequence === delta.baseSequence;
}
if (canApplyDelta("8", { baseSequence: "7", sequence: "9" })) {
  failed += 1;
  console.error("negative control: sequence gap was accepted");
}

if (failed) process.exit(1);
console.log(
  `${fixture.cases.length} contract cases passed; overflow, version, size, depth, block-count, and sequence-gap negative controls rejected`,
);
