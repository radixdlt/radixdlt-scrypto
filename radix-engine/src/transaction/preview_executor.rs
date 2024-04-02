use crate::system::system_callback_api::SystemCallbackObject;
use crate::transaction::TransactionReceipt;
use crate::transaction::*;
use radix_common::network::NetworkDefinition;
use radix_substate_store_interface::interface::*;
use radix_transactions::errors::TransactionValidationError;
use radix_transactions::model::PreviewIntentV1;
use radix_transactions::validation::NotarizedTransactionValidator;
use radix_transactions::validation::ValidationConfig;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreviewError {
    TransactionValidationError(TransactionValidationError),
}

pub fn execute_preview<S: SubstateDatabase, V: SystemCallbackObject + Clone>(
    substate_db: &S,
    vm: V,
    network: &NetworkDefinition,
    preview_intent: PreviewIntentV1,
    with_kernel_trace: bool,
) -> Result<TransactionReceipt, PreviewError> {
    let validation_config = ValidationConfig::default(network.id);

    let validator = NotarizedTransactionValidator::new(validation_config);

    let mut execution_config = if preview_intent.flags.disable_auth {
        ExecutionConfig::for_preview_no_auth(network.clone())
    } else {
        ExecutionConfig::for_preview(network.clone())
    };
    execution_config = execution_config.with_kernel_trace(with_kernel_trace);

    let validated = validator
        .validate_preview_intent_v1(preview_intent)
        .map_err(PreviewError::TransactionValidationError)?;

    Ok(execute_transaction(
        substate_db,
        vm,
        &execution_config,
        &validated.get_executable(),
    ))
}
