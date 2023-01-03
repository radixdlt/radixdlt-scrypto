use radix_engine_interface::node::NetworkDefinition;
use transaction::errors::TransactionValidationError;
use transaction::model::PreviewIntent;
use transaction::validation::IntentHashManager;
use transaction::validation::NotarizedTransactionValidator;
use transaction::validation::ValidationConfig;

use crate::engine::ScryptoInterpreter;
use crate::ledger::*;
use crate::transaction::TransactionReceipt;
use crate::transaction::*;
use crate::wasm::WasmEngine;

#[derive(Debug)]
pub struct PreviewResult {
    pub intent: PreviewIntent,
    pub receipt: TransactionReceipt,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreviewError {
    TransactionValidationError(TransactionValidationError),
}

pub fn execute_preview<S: ReadableSubstateStore, W: WasmEngine, IHM: IntentHashManager>(
    substate_store: &S,
    scrypto_interpreter: &ScryptoInterpreter<W>,
    intent_hash_manager: &IHM,
    network: &NetworkDefinition,
    preview_intent: PreviewIntent,
) -> Result<PreviewResult, PreviewError> {
    let validation_config = ValidationConfig::default(network.id);

    let validator = NotarizedTransactionValidator::new(validation_config);

    let receipt = {
        let executable = validator
            .validate_preview_intent(&preview_intent, intent_hash_manager)
            .map_err(PreviewError::TransactionValidationError)?;

        execute_transaction(
            substate_store,
            scrypto_interpreter,
            &FeeReserveConfig::default(),
            &ExecutionConfig::default(),
            &executable,
        )
    };

    Ok(PreviewResult {
        intent: preview_intent,
        receipt,
    })
}
