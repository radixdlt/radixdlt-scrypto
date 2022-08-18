use super::Module;
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
        println!("{}{}", "  ".repeat($self.depth), sbor::rust::format!($msg, $( $arg ),*));
    };
}

impl Module for LoggerModule {
    fn pre_sys_call(
        &mut self,
        _heap: &mut Vec<super::CallFrame>,
        sys_call: super::SysCall,
    ) -> Result<(), super::ModuleError> {
        match sys_call {
            super::SysCall::InvokeFunction {
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
            super::SysCall::InvokeMethod {
                function, input, ..
            } => {
                log!(
                    self,
                    "Invoking method: fn = {:?}, buckets = {:?}, proofs = {:?}",
                    function,
                    input.bucket_ids,
                    input.proof_ids
                );

                self.depth = self.depth + 1;
            }
            super::SysCall::BorrowNode { node_id } => {
                log!(self, "Borrowing node: node_id = {:?}", node_id);
            }
            super::SysCall::DropNode { node_id } => {
                log!(self, "Dropping node: node_id = {:?}", node_id);
            }
            super::SysCall::CreateNode { node } => {
                log!(self, "Creating node: node_id = {:?}", node);
            }
            super::SysCall::GlobalizeNode { node_id } => {
                log!(self, "Globalizing node: node_id = {:?}", node_id);
            }
            super::SysCall::BorrowSubstateMut { substate_id } => {
                log!(self, "Borrowing substate: substate_id = {:?}", substate_id);
            }
            super::SysCall::ReturnSubstateMut { substate_ref } => {
                log!(
                    self,
                    "Borrowing substate: substate_ref = {:?}",
                    substate_ref
                );
            }
            super::SysCall::ReadSubstate { substate_id } => {
                log!(self, "Borrowing substate: substate_id = {:?}", substate_id);
            }
            super::SysCall::WriteSubstate { substate_id, value } => {
                log!(
                    self,
                    "Borrowing substate: substate_id = {:?}, value = {:?}",
                    substate_id,
                    value
                );
            }
            super::SysCall::TakeSubstate { substate_id } => {
                log!(self, "Borrowing substate: substate_id = {:?}", substate_id);
            }
            super::SysCall::ReadTransactionHash => {
                log!(self, "Reading transaction hash");
            }
            super::SysCall::GenerateUuid => {
                log!(self, "Generating UUID");
            }
            super::SysCall::EmitLog { .. } => {
                log!(self, "Emitting application log");
            }
            super::SysCall::CheckAccessRule { .. } => {
                log!(self, "Checking access rule");
            }
        }

        Ok(())
    }

    fn post_sys_call(
        &mut self,
        _heap: &mut Vec<super::CallFrame>,
        sys_call: super::SysCall,
    ) -> Result<(), super::ModuleError> {
        match sys_call {
            super::SysCall::InvokeFunction { fn_identifier, .. } => {
                log!(self, "Exiting function: fn = {:?}", fn_identifier);

                self.depth = self.depth - 1;
            }
            super::SysCall::InvokeMethod { function, .. } => {
                log!(self, "Exiting method: fn = {:?}", function);

                self.depth = self.depth - 1;
            }
            super::SysCall::BorrowNode { .. } => {}
            super::SysCall::DropNode { .. } => {}
            super::SysCall::CreateNode { .. } => {}
            super::SysCall::GlobalizeNode { .. } => {}
            super::SysCall::BorrowSubstateMut { .. } => {}
            super::SysCall::ReturnSubstateMut { .. } => {}
            super::SysCall::ReadSubstate { .. } => {}
            super::SysCall::WriteSubstate { .. } => {}
            super::SysCall::TakeSubstate { .. } => {}
            super::SysCall::ReadTransactionHash => {}
            super::SysCall::GenerateUuid => {}
            super::SysCall::EmitLog { .. } => {}
            super::SysCall::CheckAccessRule { .. } => {}
        }

        Ok(())
    }
}
