use crate::engine::*;
use crate::fee::FeeReserve;
use crate::model::ResourceContainer;
use crate::types::*;

pub struct LoggerModule {
    depth: usize,
}

impl LoggerModule {
    pub fn new() -> Self {
        Self { depth: 0 }
    }
}

#[macro_export]
macro_rules! log {
    ( $self: expr, $msg: expr $( , $arg:expr )* ) => {
        #[cfg(not(feature = "alloc"))]
        println!("{}[{}] {}", "    ".repeat($self.depth), $self.depth, sbor::rust::format!($msg, $( $arg ),*));
    };
}

#[allow(unused_variables)] // for no_std
impl<R: FeeReserve> Module<R> for LoggerModule {
    fn pre_sys_call(
        &mut self,
        _track: &mut Track<R>,
        _heap: &mut Vec<CallFrame>,
        input: SysCallInput,
    ) -> Result<(), ModuleError> {
        match input {
            SysCallInput::InvokeFunction {
                fn_identifier,
                input,
            } => {
                log!(
                    self,
                    "Invoking function: fn = {:?}, buckets = {:?}, proofs = {:?}",
                    fn_identifier,
                    input.bucket_ids,
                    input.proof_ids
                );

                self.depth = self.depth + 1;
            }
            SysCallInput::InvokeMethod {
                fn_identifier,
                input,
                ..
            } => {
                log!(
                    self,
                    "Invoking method: fn = {:?}, buckets = {:?}, proofs = {:?}",
                    fn_identifier,
                    input.bucket_ids,
                    input.proof_ids
                );

                self.depth = self.depth + 1;
            }
            SysCallInput::BorrowNode { node_id } => {
                log!(self, "Borrowing node: node_id = {:?}", node_id);
            }
            SysCallInput::DropNode { node_id } => {
                log!(self, "Dropping node: node_id = {:?}", node_id);
            }
            SysCallInput::CreateNode { node } => {
                log!(self, "Creating node: node = {:?}", node);
            }
            SysCallInput::GlobalizeNode { node_id } => {
                log!(self, "Globalizing node: node_id = {:?}", node_id);
            }
            SysCallInput::BorrowSubstateMut { substate_id } => {
                log!(self, "Borrowing substate: substate_id = {:?}", substate_id);
            }
            SysCallInput::ReturnSubstateMut { substate_ref } => {
                log!(
                    self,
                    "Borrowing substate: substate_ref = {:?}",
                    substate_ref
                );
            }
            SysCallInput::ReadSubstate { substate_id } => {
                log!(self, "Borrowing substate: substate_id = {:?}", substate_id);
            }
            SysCallInput::WriteSubstate { substate_id, value } => {
                log!(
                    self,
                    "Borrowing substate: substate_id = {:?}, value = {:?}",
                    substate_id,
                    value
                );
            }
            SysCallInput::TakeSubstate { substate_id } => {
                log!(self, "Borrowing substate: substate_id = {:?}", substate_id);
            }
            SysCallInput::ReadTransactionHash => {
                log!(self, "Reading transaction hash");
            }
            SysCallInput::ReadBlob { blob_hash } => {
                log!(self, "Reading blob: {}", blob_hash);
            }
            SysCallInput::GenerateUuid => {
                log!(self, "Generating UUID");
            }
            SysCallInput::EmitLog { .. } => {
                log!(self, "Emitting application log");
            }
            SysCallInput::CheckAccessRule { .. } => {
                log!(self, "Checking access rule");
            }
        }

        Ok(())
    }

    fn post_sys_call(
        &mut self,
        _track: &mut Track<R>,
        _heap: &mut Vec<CallFrame>,
        output: SysCallOutput,
    ) -> Result<(), ModuleError> {
        match output {
            SysCallOutput::InvokeFunction { output, .. } => {
                self.depth = self.depth - 1;
                log!(self, "Exiting function: output = {:?}", output);
            }
            SysCallOutput::InvokeMethod { output, .. } => {
                self.depth = self.depth - 1;
                log!(self, "Exiting method: output = {:?}", output);
            }
            SysCallOutput::BorrowNode { .. } => {}
            SysCallOutput::DropNode { .. } => {}
            SysCallOutput::CreateNode { .. } => {}
            SysCallOutput::GlobalizeNode { .. } => {}
            SysCallOutput::BorrowSubstateMut { .. } => {}
            SysCallOutput::ReturnSubstateMut { .. } => {}
            SysCallOutput::ReadSubstate { .. } => {}
            SysCallOutput::WriteSubstate { .. } => {}
            SysCallOutput::TakeSubstate { .. } => {}
            SysCallOutput::ReadTransactionHash { .. } => {}
            SysCallOutput::ReadBlob { .. } => {}
            SysCallOutput::GenerateUuid { .. } => {}
            SysCallOutput::EmitLog { .. } => {}
            SysCallOutput::CheckAccessRule { .. } => {}
        }

        Ok(())
    }

    fn on_wasm_instantiation(
        &mut self,
        _track: &mut Track<R>,
        _heap: &mut Vec<CallFrame>,
        _code: &[u8],
    ) -> Result<(), ModuleError> {
        Ok(())
    }

    fn on_wasm_costing(
        &mut self,
        _track: &mut Track<R>,
        _heap: &mut Vec<CallFrame>,
        _units: u32,
    ) -> Result<(), ModuleError> {
        Ok(())
    }

    fn on_lock_fee(
        &mut self,
        _track: &mut Track<R>,
        _heap: &mut Vec<CallFrame>,
        _vault_id: VaultId,
        fee: ResourceContainer,
        _contingent: bool,
    ) -> Result<ResourceContainer, ModuleError> {
        Ok(fee)
    }
}
