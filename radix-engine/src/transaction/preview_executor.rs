use crate::transaction::TransactionReceipt;
use crate::transaction::*;
use crate::vm::VmInitialize;
use radix_common::network::NetworkDefinition;
use radix_substate_store_interface::interface::*;
use radix_transactions::errors::TransactionValidationError;
use radix_transactions::model::PreviewIntentV1;
use radix_transactions::validation::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreviewError {
    TransactionValidationError(TransactionValidationError),
}

impl From<TransactionValidationError> for PreviewError {
    fn from(value: TransactionValidationError) -> Self {
        Self::TransactionValidationError(value)
    }
}

pub fn execute_preview(
    substate_db: &impl SubstateDatabase,
    vm_modules: &impl VmInitialize,
    network: &NetworkDefinition,
    preview_intent: PreviewIntentV1,
    with_kernel_trace: bool,
) -> Result<TransactionReceipt, PreviewError> {
    let validator = TransactionValidator::new(substate_db, network);

    let mut execution_config = if preview_intent.flags.disable_auth {
        ExecutionConfig::for_preview_no_auth(network.clone())
    } else {
        ExecutionConfig::for_preview(network.clone())
    };
    execution_config = execution_config.with_kernel_trace(with_kernel_trace);

    let validated = validator.validate_preview_intent_v1(preview_intent)?;

    Ok(execute_transaction(
        substate_db,
        vm_modules,
        &execution_config,
        validated.create_executable(),
    ))
}
