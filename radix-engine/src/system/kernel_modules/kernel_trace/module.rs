use crate::{errors::ModuleError, kernel::*};

pub struct KernelTraceModule;

#[macro_export]
macro_rules! log {
    ( $call_frame: expr, $msg: expr $( , $arg:expr )* ) => {
        #[cfg(not(feature = "alloc"))]
        println!("{}[{}] {}", "    ".repeat($call_frame.depth), $call_frame.depth, sbor::rust::format!($msg, $( $arg ),*));
    };
}

#[allow(unused_variables)] // for no_std
impl BaseModule for KernelTraceModule {
    fn pre_kernel_api_call(
        &mut self,
        call_frame: &CallFrame,
        _heap: &mut Heap,
        _track: &mut Track,
        input: KernelApiCallInput,
    ) -> Result<(), ModuleError> {
        match input {
            KernelApiCallInput::Invoke { fn_identifier, .. } => {
                log!(call_frame, "Invoking: {:?}", fn_identifier);
            }
            KernelApiCallInput::DropNode { node_id } => {
                log!(call_frame, "Dropping node: node_id = {:?}", node_id);
            }
            KernelApiCallInput::CreateNode { node } => {
                log!(call_frame, "Creating node: node = {:?}", node);
            }
            KernelApiCallInput::LockSubstate {
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
            KernelApiCallInput::GetRef { lock_handle } => {
                log!(
                    call_frame,
                    "Reading substate: lock_handle = {:?}",
                    lock_handle
                );
            }
            KernelApiCallInput::GetRefMut { lock_handle } => {
                log!(call_frame, "Get Mut: lock_handle = {:?}", lock_handle);
            }
            KernelApiCallInput::DropLock { lock_handle } => {
                log!(call_frame, "Drop Lock: lock_handle = {:?}", lock_handle);
            }
        }

        Ok(())
    }

    fn post_kernel_api_call(
        &mut self,
        call_frame: &CallFrame,
        _heap: &mut Heap,
        _track: &mut Track,
        output: KernelApiCallOutput,
    ) -> Result<(), ModuleError> {
        match output {
            KernelApiCallOutput::Invoke { rtn, .. } => {
                log!(call_frame, "Exiting invoke: output = {:?}", rtn);
            }
            KernelApiCallOutput::DropNode { .. } => {}
            KernelApiCallOutput::CreateNode { .. } => {}
            KernelApiCallOutput::LockSubstate { lock_handle } => {
                log!(
                    call_frame,
                    "Lock acquired: lock_handle = {:?} ",
                    lock_handle
                );
            }
            KernelApiCallOutput::GetRef { .. } => {}
            KernelApiCallOutput::GetRefMut { .. } => {}
            KernelApiCallOutput::DropLock { .. } => {}
        }

        Ok(())
    }
}
