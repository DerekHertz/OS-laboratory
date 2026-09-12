//! Atomic decoding of the T02 workload/program subset; worker envelopes are T07.
use serde::Deserialize;
use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet};

pub const MAX_BYTES: usize = 1_048_576;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputError {
    InvalidCommand,
    ResourceLimit,
    UnsupportedSchema,
    UnsupportedModel,
    UnsupportedEngine,
    UnsupportedRandomAlgorithm,
}
type Result<T> = std::result::Result<T, InputError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyConfig {
    Fcfs,
    RoundRobin { quantum: u64 },
}
impl PolicyConfig {
    pub fn quantum(&self) -> Option<std::num::NonZeroU64> {
        match self {
            Self::Fcfs => None,
            Self::RoundRobin { quantum } => std::num::NonZeroU64::new(*quantum),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum Operand {
    Literal(u64),
    Parameter(String),
}
#[derive(Debug, Clone)]
pub(crate) enum Op {
    Compute(Operand),
    Io(Operand),
    Yield,
    Repeat { count: Operand, after: usize },
    RepeatNext { start: usize },
    End,
}
#[derive(Debug, Clone)]
pub(crate) struct Instruction {
    pub id: String,
    pub op: Op,
}
#[derive(Debug, Clone)]
pub struct Program {
    pub(crate) id: String,
    pub(crate) instructions: Vec<Instruction>,
    parameters: BTreeMap<String, (i64, i64, i64)>,
}
#[derive(Debug, Clone)]
pub(crate) struct ThreadInput {
    pub id: String,
    pub program: usize,
    pub arrival: u64,
    pub parameters: BTreeMap<String, i64>,
}
/// Constructible only through validation. Programs and compatibility metadata stay immutable.
#[derive(Debug, Clone)]
pub struct Workload {
    pub(crate) source: Value,
    pub(crate) machine: String,
    pub(crate) cores: usize,
    pub(crate) switch_cost: u64,
    pub(crate) control_budget: u64,
    pub(crate) programs: Vec<Program>,
    pub(crate) threads: Vec<ThreadInput>,
    policy: PolicyConfig,
    seed: u64,
}
impl Workload {
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        Self::from_value(parse(bytes)?)
    }
    pub fn policy(&self) -> &PolicyConfig {
        &self.policy
    }
    pub fn seed(&self) -> u64 {
        self.seed
    }
    pub fn source(&self) -> &Value {
        &self.source
    }
    pub fn arrival(&self, index: usize) -> Option<u64> {
        self.threads.get(index).map(|t| t.arrival)
    }
    fn from_value(v: Value) -> Result<Self> {
        let o = object(
            &v,
            &[
                "schemaVersion",
                "applicationVersion",
                "modelVersion",
                "engineVersion",
                "randomAlgorithmVersion",
                "seed",
                "machine",
                "policy",
                "programs",
                "threads",
            ],
        )?;
        // Limit checks precede compatibility discrimination; malformed non-version fields
        // are rejected before unsupported values, as in create routing step 8.
        let machine = object(
            &o["machine"],
            &[
                "machineId",
                "cores",
                "switchCost",
                "controlOperationBudgetPerTick",
            ],
        )?;
        let machine_id = id(&machine["machineId"])?;
        id(&o["applicationVersion"])?;
        let cores = machine["cores"]
            .as_f64()
            .filter(|n| n.fract() == 0.0 && (1.0..=64.0).contains(n))
            .ok_or(InputError::InvalidCommand)? as usize;
        let switch_cost = unsigned(&machine["switchCost"], false)?;
        let control_budget = unsigned(&machine["controlOperationBudgetPerTick"], true)?;
        let seed = unsigned(&o["seed"], false)?;
        let policy = match o["policy"].get("kind").and_then(Value::as_str) {
            Some("fcfs") => {
                object(&o["policy"], &["kind"])?;
                PolicyConfig::Fcfs
            }
            Some("roundRobin") => {
                let p = object(&o["policy"], &["kind", "quantum"])?;
                PolicyConfig::RoundRobin {
                    quantum: unsigned(&p["quantum"], true)?,
                }
            }
            _ => return Err(InputError::InvalidCommand),
        };
        let raw_programs = array(&o["programs"], 1, 10_000)?;
        let raw_threads = array(&o["threads"], 1, 10_000)?;
        let mut programs = Vec::new();
        let mut program_ids = BTreeMap::new();
        let mut unsupported_program = false;
        for p in raw_programs {
            let program = Program::from_value(p, false)?;
            unsupported_program |= p["schemaVersion"] != "os-lab-program/1";
            if program_ids
                .insert(program.id.clone(), programs.len())
                .is_some()
            {
                return Err(InputError::InvalidCommand);
            }
            programs.push(program);
        }
        let mut threads = Vec::new();
        let mut thread_ids = BTreeSet::new();
        for t in raw_threads {
            let t = object(t, &["threadId", "programId", "arrival", "parameters"])?;
            let thread_id = id(&t["threadId"])?;
            if !thread_id.starts_with(&format!("{machine_id}:"))
                || !thread_ids.insert(thread_id.clone())
            {
                return Err(InputError::InvalidCommand);
            }
            let program_index = *program_ids
                .get(&id(&t["programId"])?)
                .ok_or(InputError::InvalidCommand)?;
            let program = &programs[program_index];
            let overrides = t["parameters"]
                .as_object()
                .ok_or(InputError::InvalidCommand)?;
            if overrides.len() > 128 {
                return Err(InputError::ResourceLimit);
            }
            let mut parameters: BTreeMap<_, _> = program
                .parameters
                .iter()
                .map(|(id, (_, d, _))| (id.clone(), *d))
                .collect();
            for (key, value) in overrides {
                let (minimum, _, maximum) = program
                    .parameters
                    .get(key)
                    .ok_or(InputError::InvalidCommand)?;
                let value = signed(value)?;
                if value < *minimum || value > *maximum {
                    return Err(InputError::InvalidCommand);
                }
                parameters.insert(key.clone(), value);
            }
            check_operands(&program.instructions, &parameters)?;
            threads.push(ThreadInput {
                id: thread_id,
                program: program_index,
                arrival: unsigned(&t["arrival"], false)?,
                parameters,
            });
        }
        for name in [
            "schemaVersion",
            "modelVersion",
            "engineVersion",
            "randomAlgorithmVersion",
        ] {
            if !o[name].is_string() {
                return Err(InputError::InvalidCommand);
            }
        }
        version(
            &o["schemaVersion"],
            "os-lab-workload/1",
            InputError::UnsupportedSchema,
        )?;
        if unsupported_program {
            return Err(InputError::UnsupportedSchema);
        }
        version(
            &o["modelVersion"],
            "sched.m1/revision-1",
            InputError::UnsupportedModel,
        )?;
        version(
            &o["engineVersion"],
            "sim-engine.0.1.0",
            InputError::UnsupportedEngine,
        )?;
        version(
            &o["randomAlgorithmVersion"],
            "none.v1",
            InputError::UnsupportedRandomAlgorithm,
        )?;
        Ok(Self {
            source: v,
            machine: machine_id,
            cores,
            switch_cost,
            control_budget,
            programs,
            threads,
            policy,
            seed,
        })
    }
}
impl Program {
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        Self::from_value(&parse(bytes)?, true)
    }
    fn from_value(v: &Value, check_version: bool) -> Result<Self> {
        let o = object(
            v,
            &["schemaVersion", "programId", "name", "parameters", "blocks"],
        )?;
        let name = o["name"].as_str().ok_or(InputError::InvalidCommand)?;
        if !(1..=128).contains(&name.chars().count()) {
            return Err(InputError::InvalidCommand);
        }
        let mut parameters = BTreeMap::new();
        for p in array(&o["parameters"], 0, 128)? {
            let p = object(p, &["id", "minimum", "default", "maximum"])?;
            let (min, default, max) = (
                signed(&p["minimum"])?,
                signed(&p["default"])?,
                signed(&p["maximum"])?,
            );
            if min > default
                || default > max
                || parameters
                    .insert(id(&p["id"])?, (min, default, max))
                    .is_some()
            {
                return Err(InputError::InvalidCommand);
            }
        }
        let mut instructions = Vec::new();
        let mut ids = BTreeSet::new();
        if !blocks(&o["blocks"], 0, &mut instructions, &mut ids)? {
            return Err(InputError::InvalidCommand);
        }
        check_operands(
            &instructions,
            &parameters
                .iter()
                .map(|(id, (_, d, _))| (id.clone(), *d))
                .collect(),
        )?;
        if !o["schemaVersion"].is_string() {
            return Err(InputError::InvalidCommand);
        }
        if check_version {
            version(
                &o["schemaVersion"],
                "os-lab-program/1",
                InputError::UnsupportedSchema,
            )?;
        }
        Ok(Self {
            id: id(&o["programId"])?,
            instructions,
            parameters,
        })
    }
}
fn blocks(
    v: &Value,
    depth: usize,
    code: &mut Vec<Instruction>,
    ids: &mut BTreeSet<String>,
) -> Result<bool> {
    if depth > 64 {
        return Err(InputError::ResourceLimit);
    }
    let list = array(v, usize::from(depth == 0), 10_000)?;
    let mut ends = false;
    for b in list {
        if ends {
            return Err(InputError::InvalidCommand);
        }
        let block_id = id(b.get("blockId").ok_or(InputError::InvalidCommand)?)?;
        if !ids.insert(block_id.clone()) {
            return Err(InputError::InvalidCommand);
        }
        if ids.len() > 10_000 {
            return Err(InputError::ResourceLimit);
        }
        let op = b
            .get("op")
            .and_then(Value::as_str)
            .ok_or(InputError::InvalidCommand)?;
        let instruction = match op {
            "compute" | "ioWait" => {
                let o = object(b, &["blockId", "op", "duration"])?;
                if op == "compute" {
                    Op::Compute(operand(&o["duration"])?)
                } else {
                    Op::Io(operand(&o["duration"])?)
                }
            }
            "yield" | "end" => {
                object(b, &["blockId", "op"])?;
                if op == "end" {
                    ends = true;
                    Op::End
                } else {
                    Op::Yield
                }
            }
            "repeat" => {
                let o = object(b, &["blockId", "op", "count", "body"])?;
                let count = operand(&o["count"])?;
                let start = code.len();
                code.push(Instruction {
                    id: block_id.clone(),
                    op: Op::Repeat {
                        count: count.clone(),
                        after: 0,
                    },
                });
                ends = blocks(&o["body"], depth + 1, code, ids)?;
                code.push(Instruction {
                    id: block_id,
                    op: Op::RepeatNext { start },
                });
                code[start].op = Op::Repeat {
                    count,
                    after: code.len(),
                };
                continue;
            }
            _ => return Err(InputError::InvalidCommand),
        };
        code.push(Instruction {
            id: block_id,
            op: instruction,
        });
    }
    Ok(ends)
}
fn check_operands(code: &[Instruction], params: &BTreeMap<String, i64>) -> Result<()> {
    for i in code {
        let operand = match &i.op {
            Op::Compute(o) | Op::Io(o) | Op::Repeat { count: o, .. } => Some(o),
            _ => None,
        };
        if let Some(Operand::Parameter(p)) = operand
            && params.get(p).is_none_or(|v| *v <= 0)
        {
            return Err(InputError::InvalidCommand);
        }
    }
    Ok(())
}
fn operand(v: &Value) -> Result<Operand> {
    if v.get("literal").is_some() {
        let o = object(v, &["literal"])?;
        Ok(Operand::Literal(unsigned(&o["literal"], true)?))
    } else {
        let o = object(v, &["parameter"])?;
        Ok(Operand::Parameter(id(&o["parameter"])?))
    }
}
fn version(v: &Value, expected: &str, error: InputError) -> Result<()> {
    if v.as_str().ok_or(InputError::InvalidCommand)? == expected {
        Ok(())
    } else {
        Err(error)
    }
}
fn object<'a>(v: &'a Value, keys: &[&str]) -> Result<&'a Map<String, Value>> {
    let o = v.as_object().ok_or(InputError::InvalidCommand)?;
    if o.len() != keys.len() || keys.iter().any(|key| !o.contains_key(*key)) {
        return Err(InputError::InvalidCommand);
    }
    Ok(o)
}
fn array(v: &Value, minimum: usize, maximum: usize) -> Result<&Vec<Value>> {
    let a = v.as_array().ok_or(InputError::InvalidCommand)?;
    if a.len() > maximum {
        return Err(InputError::ResourceLimit);
    }
    if a.len() < minimum {
        return Err(InputError::InvalidCommand);
    }
    Ok(a)
}
fn id(v: &Value) -> Result<String> {
    let s = v.as_str().ok_or(InputError::InvalidCommand)?;
    if s.is_empty()
        || s.len() > 128
        || !s.as_bytes()[0].is_ascii_alphabetic()
        || !s
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"._:~-".contains(&c))
    {
        return Err(InputError::InvalidCommand);
    }
    Ok(s.to_owned())
}
fn digits(s: &str) -> bool {
    !s.is_empty() && (s == "0" || !s.starts_with('0')) && s.bytes().all(|c| c.is_ascii_digit())
}
fn unsigned(v: &Value, positive: bool) -> Result<u64> {
    let s = v.as_str().ok_or(InputError::InvalidCommand)?;
    if !digits(s) {
        return Err(InputError::InvalidCommand);
    }
    let n = s.parse::<u64>().map_err(|_| InputError::InvalidCommand)?;
    if positive && n == 0 {
        return Err(InputError::InvalidCommand);
    }
    Ok(n)
}
fn signed(v: &Value) -> Result<i64> {
    let s = v.as_str().ok_or(InputError::InvalidCommand)?;
    if s == "-0" || !digits(s.strip_prefix('-').unwrap_or(s)) {
        return Err(InputError::InvalidCommand);
    }
    s.parse().map_err(|_| InputError::InvalidCommand)
}
fn parse(bytes: &[u8]) -> Result<Value> {
    if bytes.len() > MAX_BYTES {
        return Err(InputError::ResourceLimit);
    }
    // Bound hostile JSON recursion before disabling serde's default 128-level limit:
    // a valid 64-deep repeat AST needs more than 128 JSON object/array levels.
    let (mut depth, mut string, mut escaped) = (0usize, false, false);
    for &b in bytes {
        if string {
            if escaped {
                escaped = false;
            } else if b == b'\\' {
                escaped = true;
            } else if b == b'"' {
                string = false;
            }
        } else {
            match b {
                b'"' => string = true,
                b'{' | b'[' => {
                    depth += 1;
                    if depth > 160 {
                        return Err(InputError::ResourceLimit);
                    }
                }
                b'}' | b']' => depth = depth.saturating_sub(1),
                _ => {}
            }
        }
    }
    let mut decoder = serde_json::Deserializer::from_slice(bytes);
    decoder.disable_recursion_limit();
    let v = Value::deserialize(&mut decoder).map_err(|_| InputError::InvalidCommand)?;
    decoder.end().map_err(|_| InputError::InvalidCommand)?;
    Ok(v)
}

#[cfg(test)]
#[path = "input_tests.rs"]
mod tests;
