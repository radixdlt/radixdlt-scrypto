use crate::engine::*;
use crate::fee::FeeReserve;

pub struct LoggerModule;

#[macro_export]
macro_rules! log {
    ( $call_frame: expr, $msg: expr $( , $arg:expr )* ) => {
        #[cfg(not(feature = "alloc"))]
        println!("{}[{}] {}", "    ".repeat($call_frame.depth), $call_frame.depth, sbor::rust::format!($msg, $( $arg ),*));
    };
}

#[allow(unused_variables)] // for no_std
impl<R: FeeReserve> Module<R> for LoggerModule {
    fn pre_sys_call(
        &mut self,
        call_frame: &CallFrame,
        _heap: &mut Heap,
        _track: &mut Track<R>,
        input: SysCallInput,
    ) -> Result<(), ModuleError> {
        match input {
            SysCallInput::Invoke { invocation, .. } => {
                log!(call_frame, "Invoking: {:?}", invocation);
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
            SysCallInput::EmitLog { .. } => {
                log!(call_frame, "Emitting application log");
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
            SysCallOutput::GenerateUuid { .. } => {}
            SysCallOutput::EmitLog { .. } => {}
        }

        Ok(())
    }
}
