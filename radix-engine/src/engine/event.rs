use transaction::model::Instruction;
use scrypto::engine::types::{BucketId, ProofId};
use crate::engine::{ProofSnapshot, REActor};
use crate::model::Resource;
use scrypto::values::ScryptoValue;
use std::collections::HashMap;

pub enum ApplicationEvent<'a> {
    PreExecuteInstruction { instruction: &'a Instruction },
    PostExecuteInstruction { instruction: &'a Instruction },
}

pub enum OutputEvent {
    InstructionTrace(Instruction, Vec<SysCallTrace>),
}

pub struct SysCallTrace {
    pub caller: REActor,
    pub depth: u32,
    pub input: SysCallTraceValue,
    pub output: SysCallTraceValue,
}

pub struct SysCallTraceValue {
    pub raw: ScryptoValue,
    pub buckets: HashMap<BucketId, Resource>,
    pub proofs: HashMap<ProofId, ProofSnapshot>
}
