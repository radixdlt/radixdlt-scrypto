use crate::engine::*;
use crate::types::*;

pub enum SysCallInput<'a> {
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

pub enum SysCallOutput<'a> {
    InvokeFunction { output: &'a ScryptoValue },
    InvokeMethod { output: &'a ScryptoValue },
    BorrowNode { node_pointer: &'a RENodePointer },
    DropNode { node: &'a HeapRootRENode },
    CreateNode { node_id: &'a RENodeId },
    GlobalizeNode,
    BorrowSubstateMut { substate_ref: &'a NativeSubstateRef },
    ReturnSubstateMut,
    ReadSubstate { value: &'a ScryptoValue },
    WriteSubstate,
    TakeSubstate { value: &'a ScryptoValue },
    ReadTransactionHash { hash: &'a Hash },
    GenerateUuid { uuid: u128 },
    EmitLog,
    CheckAccessRule { result: bool },
}

pub trait Module {
    fn pre_sys_call(
        &mut self,
        heap: &mut Vec<CallFrame>,
        input: SysCallInput,
    ) -> Result<(), ModuleError>;

    fn post_sys_call(
        &mut self,
        heap: &mut Vec<CallFrame>,
        output: SysCallOutput,
    ) -> Result<(), ModuleError>;

    fn on_wasm_instantiation(
        &mut self,
        heap: &mut Vec<CallFrame>,
        code: &[u8],
    ) -> Result<(), ModuleError>;

    fn on_wasm_costing(&mut self, heap: &mut Vec<CallFrame>, units: u32)
        -> Result<(), ModuleError>;
}
