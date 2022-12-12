use crate::model::SysCallTrace;
use crate::types::*;
use transaction::model::Instruction;

#[derive(Debug)]
pub enum Event<'a> {
    Runtime(RuntimeEvent<'a>),
    Tracked(TrackedEvent),
}

#[derive(Debug)]
pub enum RuntimeEvent<'a> {
    PreExecuteManifest,
    PreExecuteInstruction {
        instruction_index: usize,
        instruction: &'a Instruction,
    },
    PostExecuteInstruction {
        instruction_index: usize,
        instruction: &'a Instruction,
    },
    PostExecuteManifest,
}

#[derive(Debug, Clone)]
#[scrypto(TypeId, Encode, Decode)]
pub enum TrackedEvent {
    SysCallTrace(SysCallTrace),
}
