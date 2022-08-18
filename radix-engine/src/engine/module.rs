use super::{CallFrame, HeapRENode, NativeSubstateRef};
use crate::types::*;

pub enum SysCall<'a> {
    InvokeFunction {
        fn_identifier: &'a FnIdentifier,
        input: &'a ScryptoValue,
    },
    InvokeMethod {
        receiver: &'a Receiver,
        fn_identifier: &'a FnIdentifier,
        input: &'a ScryptoValue,
    },
    BorrowNode {
        node_id: &'a RENodeId,
    },

    DropNode {
        node_id: &'a RENodeId,
    },

    CreateNode {
        node: &'a HeapRENode,
    },

    GlobalizeNode {
        node_id: &'a RENodeId,
    },

    BorrowSubstateMut {
        substate_id: &'a SubstateId,
    },

    ReturnSubstateMut {
        substate_ref: &'a NativeSubstateRef,
    },

    ReadSubstate {
        substate_id: &'a SubstateId,
    },

    WriteSubstate {
        substate_id: &'a SubstateId,
        value: &'a ScryptoValue,
    },

    TakeSubstate {
        substate_id: &'a SubstateId,
    },
    ReadTransactionHash,
    GenerateUuid,
    EmitLog {
        level: &'a Level,
        message: &'a String,
    },
    CheckAccessRule {
        access_rule: &'a AccessRule,
        proof_ids: &'a Vec<ProofId>,
    },
}

#[derive(Debug)]
pub struct ModuleError(String);

pub trait Module {
    fn pre_sys_call(
        &mut self,
        heap: &mut Vec<CallFrame>,
        sys_call: SysCall,
    ) -> Result<(), ModuleError>;

    fn post_sys_call(
        &mut self,
        heap: &mut Vec<CallFrame>,
        sys_call: SysCall,
    ) -> Result<(), ModuleError>;
}
