use crate::engine::*;
use crate::fee::FeeReserve;
use crate::model::Resource;
use crate::types::*;
use radix_engine_interface::api::types::{LockHandle, RENodeId, SubstateOffset, VaultId};

#[derive(Clone)]
pub enum SysCallInput<'a> {
    Invoke {
        fn_identifier: String,
        input_size: u32,
        depth: usize,
    },
    ReadOwnedNodes,
    DropNode {
        node_id: &'a RENodeId,
    },
    CreateNode {
        node: &'a RENodeInit,
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
    ReadBlob {
        blob_hash: &'a Hash,
    },
}

#[derive(Debug, Clone)]
pub enum SysCallOutput<'a> {
    Invoke { rtn: &'a dyn Debug },
    ReadOwnedNodes,
    DropNode { node: &'a HeapRENode },
    CreateNode { node_id: &'a RENodeId },
    LockSubstate { lock_handle: LockHandle },
    GetRef { lock_handle: LockHandle },
    GetRefMut,
    DropLock,
    ReadBlob { blob: &'a [u8] },
}

pub trait BaseModule<R: FeeReserve> {
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
        _actor: &ResolvedActor,
        _call_frame_update: &CallFrameUpdate,
        _call_frame: &CallFrame,
        _heap: &mut Heap,
        _track: &mut Track<R>,
    ) -> Result<(), ModuleError> {
        Ok(())
    }

    fn post_execute_invocation(
        &mut self,
        _caller: &ResolvedActor,
        _update: &CallFrameUpdate,
        _call_frame: &CallFrame,
        _heap: &mut Heap,
        _track: &mut Track<R>,
    ) -> Result<(), ModuleError> {
        Ok(())
    }

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
}
