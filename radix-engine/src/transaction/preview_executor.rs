use transaction::errors::TransactionValidationError;
use transaction::model::PreviewIntent;
use transaction::validation::TestEpochManager;
use transaction::validation::TestIntentHashManager;
use transaction::validation::TransactionValidator;

use crate::ledger::*;
use crate::transaction::TransactionReceipt;
use crate::transaction::*;
use crate::wasm::{DefaultWasmEngine, WasmInstrumenter};

#[derive(Debug)]
pub struct PreviewResult {
    pub intent: PreviewIntent,
    pub receipt: TransactionReceipt,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreviewError {
    TransactionValidationError(TransactionValidationError),
}

pub struct PreviewExecutor;

impl PreviewExecutor {
    pub fn execute_preview<S: ReadableSubstateStore + WriteableSubstateStore + 'static>(
        preview_intent: PreviewIntent,
        substate_store: S,
    ) -> Result<PreviewResult, PreviewError> {
        let epoch_manager = TestEpochManager::new(0);
        let intent_hash_manager = TestIntentHashManager::new();

        let validated_preview_transaction = TransactionValidator::validate_preview_intent(
            preview_intent.clone(),
            &intent_hash_manager,
            &epoch_manager,
        )
        .map_err(PreviewError::TransactionValidationError)?;

        let mut wasm_engine = DefaultWasmEngine::new();
        let mut wasm_instrumenter = WasmInstrumenter::new();
        let mut executor = TransactionExecutor::new(
            &substate_store,
            &mut wasm_engine,
            &mut wasm_instrumenter,
            false,
        );
        let receipt = executor.execute(&validated_preview_transaction);

        Ok(PreviewResult {
            intent: preview_intent,
            receipt: receipt,
        })
    }
}
