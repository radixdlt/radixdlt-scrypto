use crate::kernel::actor::Actor;
use crate::kernel::call_frame::CallFrameUpdate;
use crate::types::*;
use crate::{
    errors::RuntimeError,
    kernel::{kernel_api::KernelModuleApi, module::KernelModule},
    system::node_init::NodeInit,
};
use colored::Colorize;
use radix_engine_interface::api::substate_api::LockFlags;
use radix_engine_interface::types::{
    EntityType, InvocationDebugIdentifier, LockHandle, NodeId, SubstateKey, TypedModuleId,
};
use sbor::rust::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct KernelTraceModule {}

#[macro_export]
macro_rules! log {
    ( $api: expr, $msg: expr $( , $arg:expr )* ) => {
        #[cfg(not(feature = "alloc"))]
        println!("{}[{}] {}", "    ".repeat($api.kernel_get_current_depth()), $api.kernel_get_current_depth(), sbor::rust::format!($msg, $( $arg ),*));
    };
}

#[allow(unused_variables)] // for no_std
impl KernelModule for KernelTraceModule {
    fn before_invoke<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        identifier: &InvocationDebugIdentifier,
        input_size: usize,
    ) -> Result<(), RuntimeError> {
        let message = format!(
            "Invoking: fn = {:?}, input size = {}",
            identifier, input_size
        )
        .green();

        log!(api, "{}", message);
        Ok(())
    }

    fn before_push_frame<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        callee: &Actor,
        nodes_and_refs: &mut CallFrameUpdate,
        _args: &IndexedScryptoValue,
    ) -> Result<(), RuntimeError> {
        log!(api, "Sending nodes: {:?}", nodes_and_refs.nodes_to_move);
        log!(api, "Sending refs: {:?}", nodes_and_refs.node_refs_to_copy);
        Ok(())
    }

    fn on_execution_finish<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        caller: &Option<Actor>,
        nodes_and_refs: &CallFrameUpdate,
    ) -> Result<(), RuntimeError> {
        log!(api, "Returning nodes: {:?}", nodes_and_refs.nodes_to_move);
        log!(
            api,
            "Returning refs: {:?}",
            nodes_and_refs.node_refs_to_copy
        );
        Ok(())
    }

    fn after_invoke<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        output_size: usize,
    ) -> Result<(), RuntimeError> {
        log!(api, "Exiting: output size = {}", output_size);
        Ok(())
    }

    fn on_allocate_node_id<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        node_type: &EntityType,
    ) -> Result<(), RuntimeError> {
        log!(api, "Allocating node id: type = {:?}", node_type);
        Ok(())
    }

    fn before_create_node<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        node_id: &NodeId,
        node_init: &NodeInit,
        node_module_init: &BTreeMap<TypedModuleId, BTreeMap<SubstateKey, IndexedScryptoValue>>,
    ) -> Result<(), RuntimeError> {
        let message = format!(
            "Creating node: id = {:?}, type = {:?}",
            node_id,
            node_id.entity_type()
        )
        .red();
        log!(api, "{}", message);
        Ok(())
    }

    fn before_drop_node<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        node_id: &NodeId,
    ) -> Result<(), RuntimeError> {
        log!(api, "Dropping node: id = {:?}", node_id);
        Ok(())
    }

    fn before_lock_substate<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        node_id: &NodeId,
        module_id: &TypedModuleId,
        offset: &SubstateKey,
        flags: &LockFlags,
    ) -> Result<(), RuntimeError> {
        log!(
            api,
            "Locking substate: node id = {:?}, module_id = {:?}, substate_key = {:?}, flags = {:?}",
            node_id,
            module_id,
            offset,
            flags
        );
        Ok(())
    }

    fn after_lock_substate<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        handle: LockHandle,
        size: usize,
    ) -> Result<(), RuntimeError> {
        log!(api, "Substate locked: handle = {:?}", handle);
        Ok(())
    }

    fn on_read_substate<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        lock_handle: LockHandle,
        size: usize,
    ) -> Result<(), RuntimeError> {
        log!(
            api,
            "Reading substate: handle = {}, size = {:?}",
            lock_handle,
            size
        );
        Ok(())
    }

    fn on_write_substate<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        lock_handle: LockHandle,
        size: usize,
    ) -> Result<(), RuntimeError> {
        log!(
            api,
            "Writing substate: handle = {}, size = {:?}",
            lock_handle,
            size
        );
        Ok(())
    }

    fn on_drop_lock<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        lock_handle: LockHandle,
    ) -> Result<(), RuntimeError> {
        log!(api, "Dropping lock: handle = {} ", lock_handle);
        Ok(())
    }
}
