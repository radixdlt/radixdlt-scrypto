use crate::internal_prelude::*;
use crate::kernel::call_frame::CallFrameMessage;
use crate::kernel::kernel_api::{KernelInternalApi, KernelInvocation};
use crate::kernel::kernel_callback_api::{
    CloseSubstateEvent, CreateNodeEvent, DropNodeEvent, OpenSubstateEvent, ReadSubstateEvent,
    WriteSubstateEvent,
};
use crate::system::actor::Actor;
use crate::system::module::{InitSystemModule, SystemModule};
use crate::system::system_callback::System;
use crate::system::system_callback_api::SystemCallbackObject;
use crate::{errors::RuntimeError, kernel::kernel_api::KernelApi};
use colored::Colorize;
use radix_engine_interface::types::SubstateKey;
use sbor::rust::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct KernelTraceModule;

#[macro_export]
macro_rules! log {
    ( $api: expr, $msg: expr $( , $arg:expr )* ) => {
        #[cfg(not(feature = "alloc"))]
        println!("{}[{}] {}", "    ".repeat($api.kernel_get_current_depth()), $api.kernel_get_current_depth(), sbor::rust::format!($msg, $( $arg ),*));
    };
}

impl InitSystemModule for KernelTraceModule {
    #[cfg(feature = "resource_tracker")]
    fn init(&mut self) -> Result<(), BootloadingError> {
        panic!("KernelTraceModule should be disabled for feature resource_tracker!")
    }
}

#[allow(unused_variables)] // for no_std
impl<V: SystemCallbackObject> SystemModule<System<V>> for KernelTraceModule {
    fn before_invoke<Y: KernelApi<System<V>>>(
        api: &mut Y,
        invocation: &KernelInvocation<Actor>,
    ) -> Result<(), RuntimeError> {
        let message = format!(
            "Invoking: fn = {:?}, input size = {}",
            invocation.call_frame_data,
            invocation.len(),
        )
        .green();

        log!(api, "{}", message);
        log!(api, "Sending nodes: {:?}", invocation.args.owned_nodes());
        log!(api, "Sending refs: {:?}", invocation.args.references());
        Ok(())
    }

    fn on_execution_finish<Y: KernelApi<System<V>>>(
        api: &mut Y,
        message: &CallFrameMessage,
    ) -> Result<(), RuntimeError> {
        log!(api, "Returning nodes: {:?}", message.move_nodes);
        log!(api, "Returning refs: {:?}", message.copy_global_references);
        Ok(())
    }

    fn after_invoke<Y: KernelApi<System<V>>>(
        api: &mut Y,
        output: &IndexedScryptoValue,
    ) -> Result<(), RuntimeError> {
        log!(api, "Exiting: output size = {}", output.payload_len());
        Ok(())
    }

    fn on_allocate_node_id<Y: KernelApi<System<V>>>(
        api: &mut Y,
        entity_type: EntityType,
    ) -> Result<(), RuntimeError> {
        log!(api, "Allocating node id: entity_type = {:?}", entity_type);
        Ok(())
    }

    fn on_create_node<Y: KernelInternalApi<System<V>>>(
        api: &mut Y,
        event: &CreateNodeEvent,
    ) -> Result<(), RuntimeError> {
        match event {
            CreateNodeEvent::Start(node_id, node_module_init) => {
                let mut module_substate_keys =
                    BTreeMap::<&PartitionNumber, Vec<&SubstateKey>>::new();
                for (module_id, m) in *node_module_init {
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
            }
            _ => {}
        }

        Ok(())
    }

    fn on_drop_node<Y: KernelInternalApi<System<V>>>(
        api: &mut Y,
        event: &DropNodeEvent,
    ) -> Result<(), RuntimeError> {
        match event {
            DropNodeEvent::Start(node_id) => {
                log!(api, "Dropping node: id = {:?}", node_id);
            }
            _ => {}
        }
        Ok(())
    }

    fn on_open_substate<Y: KernelInternalApi<System<V>>>(
        api: &mut Y,
        event: &OpenSubstateEvent,
    ) -> Result<(), RuntimeError> {
        match event {
            OpenSubstateEvent::Start {
                node_id,
                partition_num,
                substate_key,
                flags,
            } => {
                log!(
                    api,
                    "Locking substate: node id = {:?}, partition_num = {:?}, substate_key = {:?}, flags = {:?}",
                    node_id,
                    partition_num,
                    substate_key,
                    flags
                );
            }
            OpenSubstateEvent::IOAccess(..) => {}
            OpenSubstateEvent::End {
                handle,
                node_id,
                size,
            } => {
                log!(
                    api,
                    "Substate locked: node id = {:?}, handle = {:?}",
                    node_id,
                    handle
                );
            }
        }

        Ok(())
    }

    fn on_read_substate<Y: KernelInternalApi<System<V>>>(
        api: &mut Y,
        event: &ReadSubstateEvent,
    ) -> Result<(), RuntimeError> {
        match event {
            ReadSubstateEvent::OnRead {
                handle,
                value,
                device,
            } => {
                log!(
                    api,
                    "Reading substate: handle = {}, size = {}, device = {:?}",
                    handle,
                    value.payload_len(),
                    device
                );
            }
            ReadSubstateEvent::IOAccess(_) => {}
        }

        Ok(())
    }

    fn on_write_substate<Y: KernelInternalApi<System<V>>>(
        api: &mut Y,
        event: &WriteSubstateEvent,
    ) -> Result<(), RuntimeError> {
        match event {
            WriteSubstateEvent::Start { handle, value } => {
                log!(
                    api,
                    "Writing substate: handle = {}, size = {}",
                    handle,
                    value.payload_len()
                );
            }
            _ => {}
        }

        Ok(())
    }

    fn on_close_substate<Y: KernelInternalApi<System<V>>>(
        api: &mut Y,
        event: &CloseSubstateEvent,
    ) -> Result<(), RuntimeError> {
        match event {
            CloseSubstateEvent::Start(lock_handle) => {
                log!(api, "Substate close: handle = {} ", lock_handle);
            }
        }
        Ok(())
    }
}
