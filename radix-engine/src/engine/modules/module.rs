use crate::engine::call_frame::RENodeLocation;
use crate::engine::*;
use crate::fee::FeeReserve;
use crate::model::Resource;
use crate::types::*;
use radix_engine_interface::api::types::{
    Level, LockHandle, RENodeId, SubstateId, SubstateOffset, VaultId,
};
use radix_engine_interface::data::IndexedScryptoValue;
use sbor::rust::fmt::Debug;

#[derive(Debug)]
pub enum InvocationInfo<'a> {
    Native(&'a NativeInvocationInfo),
    Scrypto(&'a ScryptoInvocation),
}

pub enum SysCallInput<'a> {
    Invoke {
        invocation: &'a dyn Debug,
        input_size: u32,
        value_count: u32,
        depth: usize,
    },
    ReadOwnedNodes,
    BorrowNode {
        node_id: &'a RENodeId,
    },
    DropNode {
        node_id: &'a RENodeId,
    },
    CreateNode {
        node: &'a RENode,
    },
    LockSubstate {
        node_id: &'a RENodeId,
        offset: &'a SubstateOffset,
        flags: &'a LockFlags,
    },
    GetRef {
        lock_handle: &'a LockHandle,
    },
    GetRefMut {
        lock_handle: &'a LockHandle,
    },
    DropLock {
        lock_handle: &'a LockHandle,
    },
    TakeSubstate {
        substate_id: &'a SubstateId,
    },
    ReadTransactionHash,
    ReadBlob {
        blob_hash: &'a Hash,
    },
    GenerateUuid,
    EmitLog {
        level: &'a Level,
        message: &'a String,
    },
    EmitEvent {
        event: &'a Event<'a>,
    },
}

#[derive(Debug)]
pub enum SysCallOutput<'a> {
    Invoke { rtn: &'a dyn Debug },
    ReadOwnedNodes,
    BorrowNode { node_pointer: &'a RENodeLocation },
    DropNode { node: &'a HeapRENode },
    CreateNode { node_id: &'a RENodeId },
    LockSubstate { lock_handle: LockHandle },
    GetRef { lock_handle: LockHandle },
    GetRefMut,
    DropLock,
    ReadTransactionHash { hash: &'a Hash },
    ReadBlob { blob: &'a [u8] },
    GenerateUuid { uuid: u128 },
    EmitLog,
    EmitEvent,
}

pub trait Module<R: FeeReserve> {
    fn pre_sys_call(
        &mut self,
        _call_frame: &CallFrame,
        _heap: &mut Heap,
        _track: &mut Track<R>,
        _input: SysCallInput,
    ) -> Result<(), ModuleError> {
        Ok(())
    }

    fn post_sys_call(
        &mut self,
        _call_frame: &CallFrame,
        _heap: &mut Heap,
        _track: &mut Track<R>,
        _output: SysCallOutput,
    ) -> Result<(), ModuleError> {
        Ok(())
    }

    fn pre_execute_invocation(
        &mut self,
        _actor: &REActor,
        _input: &IndexedScryptoValue,
        _call_frame: &CallFrame,
        _heap: &mut Heap,
        _track: &mut Track<R>,
    ) -> Result<(), ModuleError> {
        Ok(())
    }

    fn post_execute_invocation(
        &mut self,
        update: &CallFrameUpdate,
        call_frame: &CallFrame,
        heap: &mut Heap,
        track: &mut Track<R>,
    ) -> Result<(), ModuleError>;

    fn on_wasm_instantiation(
        &mut self,
        _call_frame: &CallFrame,
        _heap: &mut Heap,
        _track: &mut Track<R>,
        _code: &[u8],
    ) -> Result<(), ModuleError> {
        Ok(())
    }

    fn on_wasm_costing(
        &mut self,
        _call_frame: &CallFrame,
        _heap: &mut Heap,
        _track: &mut Track<R>,
        _units: u32,
    ) -> Result<(), ModuleError> {
        Ok(())
    }

    fn on_lock_fee(
        &mut self,
        _call_frame: &CallFrame,
        _eap: &mut Heap,
        _rack: &mut Track<R>,
        _ault_id: VaultId,
        fee: Resource,
        _ontingent: bool,
    ) -> Result<Resource, ModuleError> {
        Ok(fee)
    }

    fn on_finished_processing(
        &mut self,
        _heap: &mut Heap,
        _track: &mut Track<R>,
    ) -> Result<(), ModuleError> {
        Ok(())
    }
}
