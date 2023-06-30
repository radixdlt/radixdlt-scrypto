use crate::kernel::actor::Actor;
use crate::kernel::call_frame::Message;
use crate::kernel::kernel_api::KernelInvocation;
use crate::system::module::SystemModule;
use crate::system::system_callback::SystemConfig;
use crate::system::system_callback_api::SystemCallbackObject;
use crate::track::interface::StoreAccessInfo;
use crate::types::*;
use crate::{errors::RuntimeError, kernel::kernel_api::KernelApi};
use colored::Colorize;
use radix_engine_interface::api::field_lock_api::LockFlags;
use radix_engine_interface::types::{LockHandle, NodeId, SubstateKey};
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
impl<V: SystemCallbackObject> SystemModule<SystemConfig<V>> for KernelTraceModule {
    fn before_invoke<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        invocation: &KernelInvocation,
    ) -> Result<(), RuntimeError> {
        let message = format!(
            "Invoking: fn = {:?}, input size = {}",
            invocation.actor,
            invocation.len(),
        )
        .green();

        log!(api, "{}", message);
        Ok(())
    }

    fn before_push_frame<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        callee: &Actor,
        message: &mut Message,
        _args: &IndexedScryptoValue,
    ) -> Result<(), RuntimeError> {
        log!(api, "Sending nodes: {:?}", message.move_nodes);
        log!(api, "Sending refs: {:?}", message.copy_references);
        Ok(())
    }

    fn on_execution_finish<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        message: &Message,
    ) -> Result<(), RuntimeError> {
        log!(api, "Returning nodes: {:?}", message.move_nodes);
        log!(api, "Returning refs: {:?}", message.copy_references);
        Ok(())
    }

    fn after_invoke<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        output_size: usize,
    ) -> Result<(), RuntimeError> {
        log!(api, "Exiting: output size = {}", output_size);
        Ok(())
    }

    fn on_allocate_node_id<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        entity_type: EntityType,
    ) -> Result<(), RuntimeError> {
        log!(api, "Allocating node id: entity_type = {:?}", entity_type);
        Ok(())
    }

    fn before_create_node<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        node_id: &NodeId,
        node_module_init: &BTreeMap<PartitionNumber, BTreeMap<SubstateKey, IndexedScryptoValue>>,
    ) -> Result<(), RuntimeError> {
        let mut module_substate_keys = BTreeMap::<&PartitionNumber, Vec<&SubstateKey>>::new();
        for (module_id, m) in node_module_init {
            for (substate_key, _) in m {
                module_substate_keys
                    .entry(module_id)
                    .or_default()
                    .push(substate_key);
            }
        }
        let message = format!(
            "Creating node: id = {:?}, type = {:?}, substates = {:?}, module 0 = {:?}",
            node_id,
            node_id.entity_type(),
            module_substate_keys,
            node_module_init.get(&PartitionNumber(0))
        )
        .red();
        log!(api, "{}", message);
        Ok(())
    }

    fn before_drop_node<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        node_id: &NodeId,
    ) -> Result<(), RuntimeError> {
        log!(api, "Dropping node: id = {:?}", node_id);
        Ok(())
    }

    fn before_open_substate<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        node_id: &NodeId,
        module_id: &PartitionNumber,
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

    fn after_open_substate<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        handle: LockHandle,
        node_id: &NodeId,
        _store_access: &StoreAccessInfo,
        size: usize,
    ) -> Result<(), RuntimeError> {
        log!(
            api,
            "Substate locked: node id = {:?}, handle = {:?}",
            node_id,
            handle
        );
        Ok(())
    }

    fn on_read_substate<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        lock_handle: LockHandle,
        value_size: usize,
        store_access: &StoreAccessInfo,
    ) -> Result<(), RuntimeError> {
        log!(
            api,
            "Reading substate: handle = {}, size = {}",
            lock_handle,
            value_size
        );
        Ok(())
    }

    fn on_write_substate<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        lock_handle: LockHandle,
        value_size: usize,
        store_access: &StoreAccessInfo,
    ) -> Result<(), RuntimeError> {
        log!(
            api,
            "Writing substate: handle = {}, size = {}",
            lock_handle,
            value_size
        );
        Ok(())
    }

    fn on_close_substate<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        lock_handle: LockHandle,
        _store_access: &StoreAccessInfo,
    ) -> Result<(), RuntimeError> {
        log!(api, "Dropping lock: handle = {} ", lock_handle);
        Ok(())
    }
}
