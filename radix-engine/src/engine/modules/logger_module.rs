use crate::engine::*;
use crate::fee::FeeReserve;
use crate::model::Resource;
use radix_engine_interface::api::types::VaultId;
use radix_engine_interface::data::IndexedScryptoValue;

pub struct LoggerModule {}

impl LoggerModule {
    pub fn new() -> Self {
        Self {}
    }
}

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
            SysCallInput::Invoke { info, .. } => {
                log!(call_frame, "Invoking: {:?}", info);
            }
            SysCallInput::ReadOwnedNodes => {
                log!(call_frame, "Reading owned nodes");
            }
            SysCallInput::BorrowNode { node_id } => {
                log!(call_frame, "Borrowing node: node_id = {:?}", node_id);
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
            SysCallInput::TakeSubstate { substate_id } => {
                log!(
                    call_frame,
                    "Taking substate: substate_id = {:?}",
                    substate_id
                );
            }
            SysCallInput::ReadTransactionHash => {
                log!(call_frame, "Reading transaction hash");
            }
            SysCallInput::ReadBlob { blob_hash } => {
                log!(call_frame, "Reading blob: hash = {}", blob_hash);
            }
            SysCallInput::GenerateUuid => {
                log!(call_frame, "Generating UUID");
            }
            SysCallInput::EmitLog { .. } => {
                log!(call_frame, "Emitting application log");
            }
            SysCallInput::EmitEvent { .. } => {
                log!(call_frame, "Emitting an event");
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
            SysCallOutput::BorrowNode { .. } => {}
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
            SysCallOutput::ReadTransactionHash { .. } => {}
            SysCallOutput::ReadBlob { .. } => {}
            SysCallOutput::GenerateUuid { .. } => {}
            SysCallOutput::EmitLog { .. } => {}
            SysCallOutput::EmitEvent { .. } => {}
        }

        Ok(())
    }

    fn pre_execute_invocation(
        &mut self,
        _actor: &REActor,
        _input: &IndexedScryptoValue,
        _call_frame: &CallFrame,
        _heap: &mut Heap,
        _track: &mut Track<R>,
    ) -> Result<(), ModuleError> {
        Ok(())
    }

    fn post_execute_invocation(
        &mut self,
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
        _heap: &mut Heap,
        _track: &mut Track<R>,
        _vault_id: VaultId,
        fee: Resource,
        _contingent: bool,
    ) -> Result<Resource, ModuleError> {
        Ok(fee)
    }

    fn on_finished_processing(
        &mut self,
        _heap: &mut Heap,
        _track: &mut Track<R>,
    ) -> Result<(), ModuleError> {
        Ok(())
    }
}
