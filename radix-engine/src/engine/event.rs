use transaction::model::Instruction;
use scrypto::engine::types::{BucketId, ProofId};
use crate::engine::{ProofSnapshot, REActor, TraceHeapSnapshot};
use crate::model::Resource;
use std::collections::HashMap;
use sbor::{TypeId, Encode, Decode};

pub enum ApplicationEvent<'a> {
    PreExecuteInstruction { instruction: &'a Instruction },
    PostExecuteInstruction { instruction: &'a Instruction },
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub enum OutputEvent {
    InstructionTraceV0(Instruction, TraceHeapSnapshot, TraceHeapSnapshot),
    InstructionTrace(Instruction, Vec<SysCallTrace>),
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct SysCallTrace {
    pub caller: REActor,
    pub depth: u32,
    pub input: SysCallTraceValue,
    pub output: SysCallTraceValue,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct SysCallTraceValue {
    pub buckets: HashMap<BucketId, Resource>,
    pub proofs: HashMap<ProofId, ProofSnapshot>
}
