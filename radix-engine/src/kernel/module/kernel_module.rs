use crate::errors::{ModuleError, RuntimeError};
use crate::kernel::*;
use crate::system::node::{RENodeInit, RENodeModuleInit};
use crate::system::node_modules::auth::AuthZoneStackSubstate;
use radix_engine_interface::api::{types::*, ClientActorApi};
use sbor::rust::collections::BTreeMap;

pub trait KernelModule {
    fn pre_kernel_invoke(
        &mut self,
        _current_frame: &CallFrame,
        _heap: &mut Heap,
        _track: &mut Track,
        _fn_identifier: &FnIdentifier,
        _input_size: usize,
    ) -> Result<(), ModuleError> {
        Ok(())
    }

    fn post_kernel_invoke(
        &mut self,
        _current_frame: &CallFrame,
        _heap: &mut Heap,
        _track: &mut Track,
        _output_size: usize,
    ) -> Result<(), ModuleError> {
        Ok(())
    }

    fn pre_kernel_execute(
        &mut self,
        _current_frame: &CallFrame,
        _heap: &mut Heap,
        _track: &mut Track,
        _callee: &ResolvedActor,
        _nodes_and_refs: &CallFrameUpdate,
    ) -> Result<(), ModuleError> {
        Ok(())
    }

    fn post_kernel_execute(
        &mut self,
        _current_frame: &CallFrame, // The callee frame
        _heap: &mut Heap,
        _track: &mut Track,
        _caller: &ResolvedActor,
        _nodes_and_refs: &CallFrameUpdate,
    ) -> Result<(), ModuleError> {
        Ok(())
    }

    fn on_allocate_node_id(
        &mut self,
        _current_frame: &CallFrame,
        _heap: &mut Heap,
        _track: &mut Track,
        _node_type: &RENodeType,
    ) -> Result<(), ModuleError> {
        Ok(())
    }

    fn pre_create_node(
        &mut self,
        _current_frame: &CallFrame,
        _heap: &mut Heap,
        _track: &mut Track,
        _node_id: &RENodeId,
        _node_init: &RENodeInit,
        _node_module_init: &BTreeMap<NodeModuleId, RENodeModuleInit>,
    ) -> Result<(), ModuleError> {
        Ok(())
    }

    fn post_create_node(
        &mut self,
        _current_frame: &CallFrame,
        _heap: &mut Heap,
        _track: &mut Track,
        _node_id: &RENodeId,
    ) -> Result<(), ModuleError> {
        Ok(())
    }

    fn pre_drop_node(
        &mut self,
        _current_frame: &CallFrame,
        _heap: &mut Heap,
        _track: &mut Track,
        _node_id: &RENodeId,
    ) -> Result<(), ModuleError> {
        Ok(())
    }

    fn post_drop_node(
        &mut self,
        _current_frame: &CallFrame,
        _heap: &mut Heap,
        _track: &mut Track,
    ) -> Result<(), ModuleError> {
        Ok(())
    }

    fn on_lock_substate(
        &mut self,
        _current_frame: &CallFrame,
        _heap: &mut Heap,
        _track: &mut Track,
        _node_id: &RENodeId,
        _module_id: &NodeModuleId,
        _offset: &SubstateOffset,
        _flags: &LockFlags,
    ) -> Result<(), ModuleError> {
        Ok(())
    }

    fn on_read_substate(
        &mut self,
        _current_frame: &CallFrame,
        _heap: &mut Heap,
        _track: &mut Track,
        _lock_handle: LockHandle,
        _size: usize,
    ) -> Result<(), ModuleError> {
        Ok(())
    }

    fn on_write_substate(
        &mut self,
        _current_frame: &CallFrame,
        _heap: &mut Heap,
        _track: &mut Track,
        _lock_handle: LockHandle,
        _size: usize,
    ) -> Result<(), ModuleError> {
        Ok(())
    }

    fn on_drop_lock(
        &mut self,
        _current_frame: &CallFrame,
        _heap: &mut Heap,
        _track: &mut Track,
        _lock_handle: LockHandle,
    ) -> Result<(), ModuleError> {
        Ok(())
    }

    fn on_wasm_instantiation(
        &mut self,
        _current_frame: &CallFrame,
        _heap: &mut Heap,
        _track: &mut Track,
        _code: &[u8],
    ) -> Result<(), ModuleError> {
        Ok(())
    }

    fn on_wasm_costing(
        &mut self,
        _current_frame: &CallFrame,
        _heap: &mut Heap,
        _track: &mut Track,
        _units: u32,
    ) -> Result<(), ModuleError> {
        Ok(())
    }
}

pub trait ApiBasedKernelModule {
    fn initialize<Y: KernelNodeApi + KernelSubstateApi>(&mut self, api: &mut Y);

    fn destroy<Y: KernelNodeApi + KernelSubstateApi>(
        &mut self,
        api: &mut Y,
    ) -> Result<AuthZoneStackSubstate, RuntimeError>;

    fn on_before_frame_start<Y>(
        &mut self,
        actor: &ResolvedActor,
        api: &mut Y,
    ) -> Result<(), RuntimeError>;

    fn on_call_frame_enter<Y: KernelNodeApi + KernelSubstateApi>(
        &mut self,
        call_frame_update: &mut CallFrameUpdate,
        actor: &ResolvedActor,
        api: &mut Y,
    ) -> Result<(), RuntimeError>;

    fn on_call_frame_exit<Y>(&mut self, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientActorApi<RuntimeError>;
}
