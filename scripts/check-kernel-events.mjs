import fs from "node:fs";
import Ajv2020 from "ajv/dist/2020.js";

// Consumes actual native Rust test output. This does not execute a JS simulator.
const artifact = JSON.parse(fs.readFileSync(process.argv[2], "utf8"));
const schema = JSON.parse(
  fs.readFileSync(
    new URL(
      "../packages/contracts/schema/contracts.schema.json",
      import.meta.url,
    ),
    "utf8",
  ),
);
const ajv = new Ajv2020({ allErrors: true, strict: true });
function integer(value, min, max, positive = false) {
  if (typeof value !== "string" || value === "-0") return false;
  const pattern = min < 0n ? /^-?(0|[1-9][0-9]*)$/ : /^(0|[1-9][0-9]*)$/;
  if (!pattern.test(value)) return false;
  const n = BigInt(value);
  return n >= min && n <= max && (!positive || n > 0n);
}
const max = 18_446_744_073_709_551_615n;
ajv.addFormat("uint64-decimal", {
  type: "string",
  validate: (v) => integer(v, 0n, max),
});
ajv.addFormat("positive-uint64-decimal", {
  type: "string",
  validate: (v) => integer(v, 0n, max, true),
});
ajv.addFormat("int64-decimal", {
  type: "string",
  validate: (v) =>
    integer(v, -9_223_372_036_854_775_808n, 9_223_372_036_854_775_807n),
});
ajv.addSchema(schema);
const eventSchema = ajv.compile({ $ref: `${schema.$id}#/$defs/event` });
function requireThat(condition, message) {
  if (!condition) throw new Error(message);
}
requireThat(
  Array.isArray(artifact.cases) && artifact.cases.length > 0,
  "No real kernel cases",
);
let eventCount = 0;
const kinds = new Set();
for (const { name, workload, events } of artifact.cases) {
  requireThat(events.length > 0, `${name}: no events`);
  const machine = workload.machine.machineId;
  const threads = new Map(workload.threads.map((t) => [t.threadId, t]));
  const blocks = new Map();
  function visit(list, ids) {
    for (const block of list) {
      ids.add(block.blockId);
      if (block.op === "repeat") visit(block.body, ids);
    }
  }
  for (const program of workload.programs) {
    const ids = new Set();
    visit(program.blocks, ids);
    blocks.set(program.programId, ids);
  }
  const previousThread = new Map();
  let previousEvent;
  let previousTick = 0n;
  for (let index = 0; index < events.length; index += 1) {
    const event = events[index];
    const context = `${name}: event ${index + 1} ${event.kind}`;
    requireThat(
      eventSchema(event),
      `${context}: ${ajv.errorsText(eventSchema.errors)}`,
    );
    requireThat(
      event.eventSequence === String(index + 1),
      `${context}: sequence gap`,
    );
    requireThat(
      event.eventId === `${machine}:event:${index + 1}`,
      `${context}: event identity`,
    );
    requireThat(
      BigInt(event.tick) >= previousTick,
      `${context}: time regressed`,
    );
    previousTick = BigInt(event.tick);
    const expectedEntities = [];
    let expectedParent = previousEvent;
    if (event.threadId !== undefined) {
      const thread = threads.get(event.threadId);
      requireThat(thread, `${context}: unknown thread`);
      expectedEntities.push(event.threadId);
      expectedParent = previousThread.get(event.threadId);
      if (event.blockId !== undefined) {
        requireThat(
          blocks.get(thread.programId).has(event.blockId),
          `${context}: foreign block`,
        );
      }
      previousThread.set(event.threadId, event.eventId);
    }
    if (event.payload.coreId !== undefined) {
      const core = event.payload.coreId;
      requireThat(core < workload.machine.cores, `${context}: unknown core`);
      let entity = `${machine}:core:${core}`;
      while (threads.has(entity)) entity += ":";
      expectedEntities.push(entity);
    }
    requireThat(
      JSON.stringify(event.entityIds) === JSON.stringify(expectedEntities),
      `${context}: entity order/identity`,
    );
    requireThat(
      JSON.stringify(event.causalParentIds) ===
        JSON.stringify(expectedParent ? [expectedParent] : []),
      `${context}: predecessor provenance`,
    );
    previousEvent = event.eventId;
    eventCount += 1;
    kinds.add(event.kind);
  }
}
for (const kind of [
  "arrival",
  "dispatchStarted",
  "dispatchCompleted",
  "computeStarted",
  "computeCompleted",
  "ioSubmitted",
  "ioCompleted",
  "yielded",
  "quantumExpired",
  "terminated",
  "runtimeError",
  "runCompleted",
]) {
  requireThat(
    kinds.has(kind),
    `Real kernel artifact lacks implemented event kind ${kind}`,
  );
}
console.log(
  `${artifact.cases.length} real kernel cases / ${eventCount} events conform to shared schema, source identity and canonical metadata (${[...kinds].sort().join(", ")})`,
);
