use crate::{
    errors::RuntimeError,
    kernel::*,
    system::node::{RENodeInit, RENodeModuleInit},
};
use radix_engine_interface::api::types::{
    FnIdentifier, LockHandle, NodeModuleId, RENodeId, RENodeType, SubstateOffset,
};
use radix_engine_interface::*;
use sbor::rust::collections::BTreeMap;

#[derive(ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct KernelDebugModule;

impl KernelModuleState for KernelDebugModule {
    const ID: u8 = KernelModuleId::KernelDebug as u8;
}

#[macro_export]
macro_rules! log {
    ( $api: expr, $msg: expr $( , $arg:expr )* ) => {
        if let Some(state) = $api.get_module_state::<KernelDebugModule>() {
            #[cfg(not(feature = "alloc"))]
            println!("{}[{}] {}", "    ".repeat($api.get_current_depth()), $api.get_current_depth(), sbor::rust::format!($msg, $( $arg ),*));
        }
    };
}

#[allow(unused_variables)] // for no_std
impl KernelModule for KernelDebugModule {
    fn pre_kernel_invoke<Y: KernelNodeApi + KernelSubstateApi>(
        api: &mut Y,
        fn_identifier: &FnIdentifier,
        input_size: usize,
    ) -> Result<(), RuntimeError> {
        log!(
            api,
            "Invoking: fn = {:?}, input size = {}",
            fn_identifier,
            input_size
        );
        Ok(())
    }

    fn post_kernel_invoke<Y: KernelNodeApi + KernelSubstateApi>(
        api: &mut Y,
        output_size: usize,
    ) -> Result<(), RuntimeError> {
        log!(api, "Exiting: output size = {}", output_size);
        Ok(())
    }

    fn pre_kernel_execute<Y: KernelNodeApi + KernelSubstateApi>(
        api: &mut Y,
        callee: &ResolvedActor,
        nodes_and_refs: &CallFrameUpdate,
    ) -> Result<(), RuntimeError> {
        log!(api, "Sending nodes: {:?}", nodes_and_refs.nodes_to_move);
        log!(api, "Sending refs: {:?}", nodes_and_refs.node_refs_to_copy);
        Ok(())
    }

    fn post_kernel_execute<Y: KernelNodeApi + KernelSubstateApi>(
        api: &mut Y,
        caller: &ResolvedActor,
        nodes_and_refs: &CallFrameUpdate,
    ) -> Result<(), RuntimeError> {
        log!(api, "Received nodes: {:?}", nodes_and_refs.nodes_to_move);
        log!(api, "Received refs: {:?}", nodes_and_refs.node_refs_to_copy);
        Ok(())
    }

    fn on_allocate_node_id<Y: KernelNodeApi + KernelSubstateApi>(
        api: &mut Y,
        node_type: &RENodeType,
    ) -> Result<(), RuntimeError> {
        log!(api, "Allocating node id: type = {:?}", node_type);
        Ok(())
    }

    fn pre_create_node<Y: KernelNodeApi + KernelSubstateApi>(
        api: &mut Y,
        node_id: &RENodeId,
        node_init: &RENodeInit,
        node_module_init: &BTreeMap<NodeModuleId, RENodeModuleInit>,
    ) -> Result<(), RuntimeError> {
        log!(
            api,
            "Creating node: id = {:?}, init = {:?}, module_init = {:?}",
            node_id,
            node_init,
            node_module_init
        );
        Ok(())
    }

    fn pre_drop_node<Y: KernelNodeApi + KernelSubstateApi>(
        api: &mut Y,
        node_id: &RENodeId,
    ) -> Result<(), RuntimeError> {
        log!(api, "Dropping node: id = {:?}", node_id);
        Ok(())
    }

    fn on_lock_substate<Y: KernelNodeApi + KernelSubstateApi>(
        api: &mut Y,
        node_id: &RENodeId,
        module_id: &NodeModuleId,
        offset: &SubstateOffset,
        flags: &LockFlags,
    ) -> Result<(), RuntimeError> {
        log!(
            api,
            "Locking substate: node id = {:?}, module_id = {:?}, offset = {:?}, flags = {:?}",
            node_id,
            module_id,
            offset,
            flags
        );
        Ok(())
    }

    fn on_read_substate<Y: KernelNodeApi + KernelSubstateApi>(
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

    fn on_write_substate<Y: KernelNodeApi + KernelSubstateApi>(
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

    fn on_drop_lock<Y: KernelNodeApi + KernelSubstateApi>(
        api: &mut Y,
        lock_handle: LockHandle,
    ) -> Result<(), RuntimeError> {
        log!(api, "Dropping lock: handle = {} ", lock_handle);
        Ok(())
    }
}
