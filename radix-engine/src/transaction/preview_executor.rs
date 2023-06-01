use crate::transaction::TransactionReceipt;
use crate::transaction::*;
use crate::vm::wasm::WasmEngine;
use crate::vm::ScryptoVm;
use radix_engine_interface::network::NetworkDefinition;
use radix_engine_store_interface::interface::*;
use transaction::errors::TransactionValidationError;
use transaction::model::PreviewIntentV1;
use transaction::validation::NotarizedTransactionValidator;
use transaction::validation::ValidationConfig;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreviewError {
    TransactionValidationError(TransactionValidationError),
}

pub fn execute_preview<S: SubstateDatabase, W: WasmEngine>(
    substate_db: &S,
    scrypto_interpreter: &ScryptoVm<W>,
    network: &NetworkDefinition,
    preview_intent: PreviewIntentV1,
    with_kernel_trace: bool,
) -> Result<TransactionReceipt, PreviewError> {
    let validation_config = ValidationConfig::default(network.id);

    let validator = NotarizedTransactionValidator::new(validation_config);

    let validated = validator
        .validate_preview_intent_v1(preview_intent)
        .map_err(PreviewError::TransactionValidationError)?;

    Ok(execute_transaction(
        substate_db,
        scrypto_interpreter,
        &FeeReserveConfig::default(),
        &ExecutionConfig::for_preview().with_kernel_trace(with_kernel_trace),
        &validated.get_executable(),
    ))
}
