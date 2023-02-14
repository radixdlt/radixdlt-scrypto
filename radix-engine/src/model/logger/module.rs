use crate::engine::*;
use crate::fee::FeeReserve;
use crate::model::LoggerSubstate;
use radix_engine_interface::api::types::{RENodeId, RENodeType};
use sbor::rust::vec::Vec;

pub struct LoggerModule;

#[macro_export]
macro_rules! log {
    ( $call_frame: expr, $msg: expr $( , $arg:expr )* ) => {
        #[cfg(not(feature = "alloc"))]
        println!("{}[{}] {}", "    ".repeat($call_frame.depth), $call_frame.depth, sbor::rust::format!($msg, $( $arg ),*));
    };
}

impl LoggerModule {
    pub fn initialize<Y: SystemApi>(api: &mut Y) -> Result<(), RuntimeError> {
        let logger = LoggerSubstate { logs: Vec::new() };
        let node_id = api.allocate_node_id(RENodeType::Logger)?;
        api.create_node(node_id, RENodeInit::Logger(logger))?;
        Ok(())
    }

    pub fn on_call_frame_enter<Y: SystemApi>(
        call_frame_update: &mut CallFrameUpdate,
        _actor: &ResolvedActor,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        let refed = api.get_visible_nodes()?;
        let maybe_id = refed.into_iter().find(|e| matches!(e, RENodeId::Logger));
        if let Some(logger_id) = maybe_id {
            call_frame_update.node_refs_to_copy.insert(logger_id);
        }

        Ok(())
    }
}

#[allow(unused_variables)] // for no_std
impl<R: FeeReserve> BaseModule<R> for LoggerModule {
    fn pre_sys_call(
        &mut self,
        call_frame: &CallFrame,
        _heap: &mut Heap,
        _track: &mut Track<R>,
        input: SysCallInput,
    ) -> Result<(), ModuleError> {
        match input {
            SysCallInput::Invoke { fn_identifier, .. } => {
                log!(call_frame, "Invoking: {}", fn_identifier);
            }
            SysCallInput::ReadOwnedNodes => {
                log!(call_frame, "Reading owned nodes");
            }
            SysCallInput::DropNode { node_id } => {
                log!(call_frame, "Dropping node: node_id = {:?}", node_id);
            }
            SysCallInput::CreateNode { node } => {
                log!(call_frame, "Creating node: node = {:?}", node);
            }
            SysCallInput::LockSubstate {
                node_id,
                offset,
                flags,
            } => {
                log!(
                    call_frame,
                    "Lock substate: node_id = {:?} offset = {:?} flags = {:?}",
                    node_id,
                    offset,
                    flags
                );
            }
            SysCallInput::GetRef { lock_handle } => {
                log!(
                    call_frame,
                    "Reading substate: lock_handle = {:?}",
                    lock_handle
                );
            }
            SysCallInput::GetRefMut { lock_handle } => {
                log!(call_frame, "Get Mut: lock_handle = {:?}", lock_handle);
            }
            SysCallInput::DropLock { lock_handle } => {
                log!(call_frame, "Drop Lock: lock_handle = {:?}", lock_handle);
            }
            SysCallInput::ReadBlob { blob_hash } => {
                log!(call_frame, "Reading blob: hash = {}", blob_hash);
            }
        }

        Ok(())
    }

    fn post_sys_call(
        &mut self,
        call_frame: &CallFrame,
        _heap: &mut Heap,
        _track: &mut Track<R>,
        output: SysCallOutput,
    ) -> Result<(), ModuleError> {
        match output {
            SysCallOutput::Invoke { rtn, .. } => {
                log!(call_frame, "Exiting invoke: output = {:?}", rtn);
            }
            SysCallOutput::ReadOwnedNodes { .. } => {}
            SysCallOutput::DropNode { .. } => {}
            SysCallOutput::CreateNode { .. } => {}
            SysCallOutput::LockSubstate { lock_handle } => {
                log!(
                    call_frame,
                    "Lock acquired: lock_handle = {:?} ",
                    lock_handle
                );
            }
            SysCallOutput::GetRef { .. } => {}
            SysCallOutput::GetRefMut { .. } => {}
            SysCallOutput::DropLock { .. } => {}
            SysCallOutput::ReadBlob { .. } => {}
        }

        Ok(())
    }
}
