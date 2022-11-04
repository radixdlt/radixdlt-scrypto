use crate::engine::SysCallTrace;
use sbor::{Decode, Encode, TypeId};
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

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub enum TrackedEvent {
    Native(NativeEvent),
    Scrypto(Vec<u8>),
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub enum NativeEvent {
    SysCallTrace(SysCallTrace),
}
