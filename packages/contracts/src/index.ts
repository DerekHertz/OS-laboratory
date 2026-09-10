export const PROGRAM_SCHEMA_VERSION = "os-lab-program/1" as const;
export const WORKLOAD_SCHEMA_VERSION = "os-lab-workload/1" as const;
export const PROTOCOL_VERSION = "os-lab-protocol/1" as const;
export const MODEL_VERSION = "sched.m1/revision-1" as const;
export const ENGINE_VERSION = "sim-engine.0.1.0" as const;
export const RANDOM_ALGORITHM_VERSION = "none.v1" as const;

export type Uint64String = string & { readonly __uint64: unique symbol };
export type Int64String = string & { readonly __int64: unique symbol };
export type ContractId = string & { readonly __contractId: unique symbol };

export type Operand = { literal: Uint64String } | { parameter: ContractId };

export interface ParameterDefinition {
  id: ContractId;
  minimum: Int64String;
  default: Int64String;
  maximum: Int64String;
}

export type ProgramBlock =
  | { blockId: ContractId; op: "compute"; duration: Operand }
  | { blockId: ContractId; op: "ioWait"; duration: Operand }
  | { blockId: ContractId; op: "yield" }
  | {
      blockId: ContractId;
      op: "repeat";
      count: Operand;
      body: ProgramBlock[];
    }
  | { blockId: ContractId; op: "end" };

export interface ProgramDefinition {
  schemaVersion: typeof PROGRAM_SCHEMA_VERSION;
  programId: ContractId;
  name: string;
  parameters: ParameterDefinition[];
  blocks: ProgramBlock[];
}

export type SchedulingPolicy =
  { kind: "fcfs" } | { kind: "roundRobin"; quantum: Uint64String };

export interface WorkloadDefinition {
  schemaVersion: typeof WORKLOAD_SCHEMA_VERSION;
  applicationVersion: string;
  modelVersion: typeof MODEL_VERSION;
  engineVersion: typeof ENGINE_VERSION;
  randomAlgorithmVersion: typeof RANDOM_ALGORITHM_VERSION;
  seed: Uint64String;
  machine: {
    machineId: ContractId;
    cores: number;
    switchCost: Uint64String;
    controlOperationBudgetPerTick: Uint64String;
  };
  policy: SchedulingPolicy;
  programs: ProgramDefinition[];
  threads: Array<{
    threadId: ContractId;
    programId: ContractId;
    arrival: Uint64String;
    parameters: Record<string, Int64String>;
  }>;
}

export type TraceMode = "inspection" | "experiment";

interface CommandIdentity {
  protocolVersion: typeof PROTOCOL_VERSION;
  requestId: ContractId;
  runId: ContractId;
}

export type Command =
  | (CommandIdentity & {
      kind: "create";
      payload: { workload: WorkloadDefinition; traceMode: TraceMode };
    })
  | (CommandIdentity & { kind: "start" | "pause" | "cancel"; payload: {} })
  | (CommandIdentity & {
      kind: "advance";
      payload:
        | { mode: "globalEvent" | "activeTick" }
        | { mode: "threadInstruction"; threadId: ContractId };
    })
  | (CommandIdentity & {
      kind: "inspect";
      payload: { afterSequence?: Uint64String };
    })
  | (CommandIdentity & {
      kind: "checkpoint";
      payload: { label?: string };
    })
  | (CommandIdentity & {
      kind: "intervene";
      payload: { kind: "setTraceMode"; traceMode: TraceMode };
    });

export type ThreadStatus =
  | "notArrived"
  | "ready"
  | "dispatching"
  | "running"
  | "blocked"
  | "terminated"
  | "error"
  | "censored";

interface EventBase {
  eventId: ContractId;
  eventSequence: Uint64String;
  tick: Uint64String;
  entityIds: ContractId[];
  threadId?: ContractId;
  blockId?: ContractId;
  causalParentIds: ContractId[];
}

type CoreGenerationPayload = { coreId: number; generation: Uint64String };

export type CanonicalEvent =
  | (EventBase & {
      kind: "arrival";
      payload: { declarationIndex: number };
    })
  | (EventBase & {
      kind:
        | "dispatchStarted"
        | "dispatchCompleted"
        | "computeStarted"
        | "computeCompleted"
        | "quantumExpired";
      payload: CoreGenerationPayload;
    })
  | (EventBase & {
      kind: "ioSubmitted";
      payload: { duration: Uint64String; completionTick: Uint64String };
    })
  | (EventBase & {
      kind: "ioCompleted" | "yielded" | "terminated";
      payload: {};
    })
  | (EventBase & {
      kind: "runtimeError";
      payload: {
        code: "arithmeticOverflow" | "controlBudgetExceeded" | "internalEngine";
      };
    })
  | (EventBase & {
      kind: "intervention";
      payload: { kind: "setTraceMode"; traceMode: TraceMode };
    })
  | (EventBase & {
      kind: "runCompleted";
      payload: {
        completedThreads: number;
        erroredThreads: number;
        censoredThreads: number;
      };
    });

export interface Snapshot {
  machineId: ContractId;
  sequence: Uint64String;
  tick: Uint64String;
  status:
    "created" | "running" | "paused" | "completed" | "cancelled" | "error";
  readyQueue: ContractId[];
  threads: Array<{
    threadId: ContractId;
    status: ThreadStatus;
    coreId?: number;
    blockId?: ContractId;
  }>;
  cores: Array<{
    coreId: number;
    state: "idle" | "dispatching" | "running";
    threadId?: ContractId;
  }>;
  events: CanonicalEvent[];
  trace: {
    truncated: boolean;
    firstEventId?: ContractId;
    lastEventId?: ContractId;
  };
}

export type DeltaOperation =
  | { op: "replaceRun"; status: Snapshot["status"]; tick: Uint64String }
  | { op: "setThread"; thread: Snapshot["threads"][number] }
  | { op: "setCore"; core: Snapshot["cores"][number] }
  | { op: "replaceReadyQueue"; threadIds: ContractId[] }
  | { op: "appendEvents"; events: CanonicalEvent[] }
  | { op: "replaceTrace"; trace: Snapshot["trace"] };

export interface Delta {
  baseSequence: Uint64String;
  sequence: Uint64String;
  operations: DeltaOperation[];
}

interface ReplyIdentity extends CommandIdentity {
  sequence: Uint64String;
}

export type Reply =
  | (ReplyIdentity & { kind: "ack"; payload: { commandKind: Command["kind"] } })
  | (ReplyIdentity & {
      kind: "progress";
      payload: { completedUnits: Uint64String; totalUnits?: Uint64String };
    })
  | (ReplyIdentity & {
      kind: "checkpoint";
      payload: { checkpointId: ContractId; tick: Uint64String };
    })
  | (ReplyIdentity & {
      kind: "state";
      payload: { snapshot: Snapshot } | { delta: Delta };
    })
  | (ReplyIdentity & {
      kind: "completed";
      payload: { snapshot: Snapshot };
    })
  | (ReplyIdentity & {
      kind: "cancelled";
      payload: { snapshot: Snapshot };
    })
  | (ReplyIdentity & {
      kind: "error";
      payload: {
        code:
          | "unsupportedProtocol"
          | "invalidCommand"
          | "unsupportedSchema"
          | "unsupportedModel"
          | "unsupportedEngine"
          | "resourceLimit"
          | "arithmeticOverflow"
          | "controlBudgetExceeded"
          | "invalidState"
          | "checkpointFailed"
          | "internalEngine";
        message: string;
        terminal: boolean;
        receivedProtocolVersion?: string;
        threadId?: ContractId;
        blockId?: ContractId;
      };
    });

export function canApplyDelta(
  currentSequence: Uint64String,
  delta: Delta,
): boolean {
  return currentSequence === delta.baseSequence;
}
