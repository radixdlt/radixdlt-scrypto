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
                #[cfg(not(feature = "alloc"))]
                println!(
                    "{}Invoking function: fn = {:?}, buckets = {:?}, proofs = {:?}",
                    "  ".repeat(self.depth),
                    fn_identifier,
                    input.bucket_ids,
                    input.proof_ids
                );

                self.depth = self.depth + 1;
            }
            super::SysCall::InvokeMethod {
                receiver,
                function,
                input,
            } => todo!(),
            super::SysCall::BorrowNode { node_id } => todo!(),
            super::SysCall::DropNode { node_id } => todo!(),
            super::SysCall::CreateNode { re_node } => todo!(),
            super::SysCall::GlobalizeNode { node_id } => todo!(),
            super::SysCall::BorrowSubstateMut { substate_id } => todo!(),
            super::SysCall::ReturnSubstateMut { val_ref } => todo!(),
            super::SysCall::ReadSubstate { substate_id } => todo!(),
            super::SysCall::WriteSubstate { substate_id, value } => todo!(),
            super::SysCall::TakeSubstate { substate_id } => todo!(),
            super::SysCall::ReadTransactionHash => todo!(),
            super::SysCall::GenerateUuid => todo!(),
            super::SysCall::EmitLog { level, message } => todo!(),
            super::SysCall::CheckAccessRule {
                access_rule,
                proof_ids,
            } => todo!(),
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
                #[cfg(not(feature = "alloc"))]
                println!(
                    "{}Exiting function: fn = {:?}",
                    "  ".repeat(self.depth),
                    fn_identifier
                );

                self.depth = self.depth - 1;
            }
            super::SysCall::InvokeMethod {
                receiver,
                function,
                input,
            } => todo!(),
            super::SysCall::BorrowNode { node_id } => todo!(),
            super::SysCall::DropNode { node_id } => todo!(),
            super::SysCall::CreateNode { re_node } => todo!(),
            super::SysCall::GlobalizeNode { node_id } => todo!(),
            super::SysCall::BorrowSubstateMut { substate_id } => todo!(),
            super::SysCall::ReturnSubstateMut { val_ref } => todo!(),
            super::SysCall::ReadSubstate { substate_id } => todo!(),
            super::SysCall::WriteSubstate { substate_id, value } => todo!(),
            super::SysCall::TakeSubstate { substate_id } => todo!(),
            super::SysCall::ReadTransactionHash => todo!(),
            super::SysCall::GenerateUuid => todo!(),
            super::SysCall::EmitLog { level, message } => todo!(),
            super::SysCall::CheckAccessRule {
                access_rule,
                proof_ids,
            } => todo!(),
        }

        Ok(())
    }
}
