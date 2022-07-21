use scrypto::prelude::Network;
use transaction::errors::TransactionValidationError;
use transaction::model::PreviewIntent;
use transaction::validation::TestIntentHashStore;
use transaction::validation::TransactionValidator;
use transaction::validation::ValidationParameters;

use crate::engine::*;
use crate::ledger::*;
use crate::wasm::{DefaultWasmEngine, WasmInstrumenter};

#[derive(Debug)]
pub struct PreviewResult {
    pub intent: PreviewIntent,
    pub receipt: Receipt,
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
        let intent_hash_store = TestIntentHashStore::new();
        let parameters: ValidationParameters = ValidationParameters {
            network: Network::LocalSimulator,
            current_epoch: 1,
            max_cost_unit_limit: 10_000_000,
            min_tip_bps: 0,
        }; // TODO: construct validation parameters based on current world state

        let validated_preview_transaction = TransactionValidator::validate_preview_intent(
            preview_intent.clone(),
            &intent_hash_store,
            &parameters,
        )
        .map_err(PreviewError::TransactionValidationError)?;

        let mut wasm_engine = DefaultWasmEngine::new();
        let mut wasm_instrumenter = WasmInstrumenter::new();
        let mut executor = TransactionExecutor::new(
            substate_store,
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
