use crate::engine::*;
use crate::types::*;

pub struct CostingModule {}

impl CostingModule {
    pub fn new() -> Self {
        Self {}
    }
}

impl Module for CostingModule {
    fn pre_sys_call(
        &mut self,
        _heap: &mut Vec<CallFrame>,
        input: SysCallInput,
    ) -> Result<(), ModuleError> {
        match input {
            SysCallInput::InvokeFunction {
                fn_identifier,
                input,
            } => todo!(),
            SysCallInput::InvokeMethod {
                receiver,
                fn_identifier,
                input,
            } => todo!(),
            SysCallInput::BorrowNode { node_id } => todo!(),
            SysCallInput::DropNode { node_id } => todo!(),
            SysCallInput::CreateNode { node } => todo!(),
            SysCallInput::GlobalizeNode { node_id } => todo!(),
            SysCallInput::BorrowSubstateMut { substate_id } => todo!(),
            SysCallInput::ReturnSubstateMut { substate_ref } => todo!(),
            SysCallInput::ReadSubstate { substate_id } => todo!(),
            SysCallInput::WriteSubstate { substate_id, value } => todo!(),
            SysCallInput::TakeSubstate { substate_id } => todo!(),
            SysCallInput::ReadTransactionHash => todo!(),
            SysCallInput::GenerateUuid => todo!(),
            SysCallInput::EmitLog { level, message } => todo!(),
            SysCallInput::CheckAccessRule {
                access_rule,
                proof_ids,
            } => todo!(),
        }

        Ok(())
    }

    fn post_sys_call(
        &mut self,
        _heap: &mut Vec<CallFrame>,
        output: SysCallOutput,
    ) -> Result<(), ModuleError> {
        match output {
            SysCallOutput::InvokeFunction { output } => todo!(),
            SysCallOutput::InvokeMethod { output } => todo!(),
            SysCallOutput::BorrowNode { node_pointer } => todo!(),
            SysCallOutput::DropNode { node } => todo!(),
            SysCallOutput::CreateNode { node_id } => todo!(),
            SysCallOutput::GlobalizeNode => todo!(),
            SysCallOutput::BorrowSubstateMut { substate_ref } => todo!(),
            SysCallOutput::ReturnSubstateMut => todo!(),
            SysCallOutput::ReadSubstate { value } => todo!(),
            SysCallOutput::WriteSubstate => todo!(),
            SysCallOutput::TakeSubstate { value } => todo!(),
            SysCallOutput::ReadTransactionHash { hash } => todo!(),
            SysCallOutput::GenerateUuid { uuid } => todo!(),
            SysCallOutput::EmitLog => todo!(),
            SysCallOutput::CheckAccessRule { result } => todo!(),
        }

        Ok(())
    }
}
