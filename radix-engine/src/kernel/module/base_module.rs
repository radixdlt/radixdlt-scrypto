use crate::errors::ModuleError;
use crate::kernel::kernel_api::LockFlags;
use crate::kernel::*;
use crate::system::kernel_modules::fee::FeeReserve;
use crate::system::node::RENodeInit;
use crate::types::*;
use radix_engine_interface::api::types::*;
use radix_engine_interface::blueprints::resource::Resource;

#[derive(Clone)]
pub enum KernelApiCallInput<'a> {
    Invoke {
        fn_identifier: FnIdentifier,
        input_size: u32,
        depth: usize,
    },
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
}

#[derive(Debug, Clone)]
pub enum KernelApiCallOutput<'a> {
    Invoke { rtn: &'a dyn Debug },
    DropNode { node: &'a HeapRENode },
    CreateNode { node_id: &'a RENodeId },

    LockSubstate { lock_handle: LockHandle },
    GetRef { lock_handle: LockHandle },
    GetRefMut,
    DropLock,
}

pub trait BaseModule<R: FeeReserve> {
    fn pre_kernel_api_call(
        &mut self,
        _call_frame: &CallFrame,
        _heap: &mut Heap,
        _track: &mut Track<R>,
        _input: KernelApiCallInput,
    ) -> Result<(), ModuleError> {
        Ok(())
    }

    fn post_kernel_api_call(
        &mut self,
        _call_frame: &CallFrame,
        _heap: &mut Heap,
        _track: &mut Track<R>,
        _output: KernelApiCallOutput,
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
        _vault_id: VaultId,
        fee: Resource,
        _contingent: bool,
    ) -> Result<Resource, ModuleError> {
        Ok(fee)
    }
}
