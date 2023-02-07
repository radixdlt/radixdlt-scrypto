use crate::{
    errors::ModuleError,
    kernel::*,
    system::node::{RENodeInit, RENodeModuleInit},
};
use radix_engine_interface::api::types::{
    FnIdentifier, LockHandle, NodeModuleId, RENodeId, RENodeType, SubstateOffset,
};
use sbor::rust::collections::BTreeMap;

pub struct KernelTraceModule;

#[macro_export]
macro_rules! log {
    ( $call_frame: expr, $msg: expr $( , $arg:expr )* ) => {
        #[cfg(not(feature = "alloc"))]
        println!("{}[{}] {}", "    ".repeat($call_frame.depth), $call_frame.depth, sbor::rust::format!($msg, $( $arg ),*));
    };
}

#[allow(unused_variables)] // for no_std
impl KernelModule for KernelTraceModule {
    fn pre_kernel_invoke(
        &mut self,
        current_frame: &CallFrame,
        heap: &mut Heap,
        track: &mut Track,
        fn_identifier: &FnIdentifier,
        input_size: usize,
    ) -> Result<(), ModuleError> {
        log!(
            current_frame,
            "Invoking: fn = {:?}, input size = {}",
            fn_identifier,
            input_size
        );
        Ok(())
    }

    fn post_kernel_invoke(
        &mut self,
        current_frame: &CallFrame,
        heap: &mut Heap,
        track: &mut Track,
        output_size: usize,
    ) -> Result<(), ModuleError> {
        log!(current_frame, "Exiting: output size = {}", output_size);
        Ok(())
    }

    fn pre_kernel_execute(
        &mut self,
        current_frame: &CallFrame,
        heap: &mut Heap,
        track: &mut Track,
        callee: &ResolvedActor,
        nodes_and_refs: &CallFrameUpdate,
    ) -> Result<(), ModuleError> {
        log!(
            current_frame,
            "Sending nodes: {:?}",
            nodes_and_refs.nodes_to_move
        );
        log!(
            current_frame,
            "Sending refs: {:?}",
            nodes_and_refs.node_refs_to_copy
        );
        Ok(())
    }

    fn post_kernel_execute(
        &mut self,
        current_frame: &CallFrame,
        heap: &mut Heap,
        track: &mut Track,
        nodes_and_refs: &CallFrameUpdate,
    ) -> Result<(), ModuleError> {
        log!(
            current_frame,
            "Received nodes: {:?}",
            nodes_and_refs.nodes_to_move
        );
        log!(
            current_frame,
            "Received refs: {:?}",
            nodes_and_refs.node_refs_to_copy
        );
        Ok(())
    }

    fn on_allocate_node_id(
        &mut self,
        current_frame: &CallFrame,
        heap: &mut Heap,
        track: &mut Track,
        node_type: &RENodeType,
    ) -> Result<(), ModuleError> {
        log!(current_frame, "Allocating node id: type = {:?}", node_type);
        Ok(())
    }

    fn pre_create_node(
        &mut self,
        current_frame: &CallFrame,
        heap: &mut Heap,
        track: &mut Track,
        node_id: &RENodeId,
        node_init: &RENodeInit,
        node_module_init: &BTreeMap<NodeModuleId, RENodeModuleInit>,
    ) -> Result<(), ModuleError> {
        log!(
            current_frame,
            "Creating node: id = {:?}, init = {:?}, module_init = {:?}",
            node_id,
            node_init,
            node_module_init
        );
        Ok(())
    }

    fn pre_drop_node(
        &mut self,
        current_frame: &CallFrame,
        heap: &mut Heap,
        track: &mut Track,
        node_id: &RENodeId,
    ) -> Result<(), ModuleError> {
        log!(current_frame, "Dropping node: id = {:?}", node_id);
        Ok(())
    }

    fn on_lock_substate(
        &mut self,
        current_frame: &CallFrame,
        heap: &mut Heap,
        track: &mut Track,
        node_id: &RENodeId,
        module_id: &NodeModuleId,
        offset: &SubstateOffset,
        flags: &LockFlags,
    ) -> Result<(), ModuleError> {
        log!(
            current_frame,
            "Locking substate: node id = {:?}, module_id = {:?}, offset = {:?}, flags = {:?}",
            node_id,
            module_id,
            offset,
            flags
        );
        Ok(())
    }

    fn on_read_substate(
        &mut self,
        current_frame: &CallFrame,
        heap: &mut Heap,
        track: &mut Track,
        lock_handle: LockHandle,
        size: usize,
    ) -> Result<(), ModuleError> {
        log!(
            current_frame,
            "Reading substate: handle = {}, size = {:?}",
            lock_handle,
            size
        );
        Ok(())
    }

    fn on_write_substate(
        &mut self,
        current_frame: &CallFrame,
        heap: &mut Heap,
        track: &mut Track,
        lock_handle: LockHandle,
        size: usize,
    ) -> Result<(), ModuleError> {
        log!(
            current_frame,
            "Writing substate: handle = {}, size = {:?}",
            lock_handle,
            size
        );
        Ok(())
    }

    fn on_drop_lock(
        &mut self,
        current_frame: &CallFrame,
        heap: &mut Heap,
        track: &mut Track,
        lock_handle: LockHandle,
    ) -> Result<(), ModuleError> {
        log!(current_frame, "Dropping lock: handle = {} ", lock_handle);
        Ok(())
    }
}
