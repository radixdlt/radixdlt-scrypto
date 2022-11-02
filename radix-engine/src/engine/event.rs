use crate::engine::SysCallTrace;
use sbor::{Decode, Encode, TypeId};
use transaction::model::Instruction;

pub enum ApplicationEvent<'a> {
    PreExecuteInstruction { instruction: &'a Instruction },
    PostExecuteInstruction { instruction: &'a Instruction },
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub enum OutputEvent {
    SysCallTrace(SysCallTrace),
}
