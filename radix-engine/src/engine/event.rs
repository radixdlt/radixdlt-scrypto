use crate::engine::SysCallTrace;
use sbor::{Decode, Encode, TypeId};
use transaction::model::Instruction;

pub enum ApplicationEvent<'a> {
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

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub enum OutputEvent {
    SysCallTrace(SysCallTrace),
}
