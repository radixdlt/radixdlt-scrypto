use scrypto::prelude::Network;
use transaction::errors::TransactionValidationError;
use transaction::model::PreviewIntent;
use transaction::validation::TestIntentHashStore;
use transaction::validation::TransactionValidator;
use transaction::validation::ValidationParameters;

use crate::constants::DEFAULT_MAX_COST_UNIT_LIMIT;
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
    pub fn execute_preview<'s, S: ReadableSubstateStore + WriteableSubstateStore>(
        preview_intent: PreviewIntent,
        substate_store: &'s mut S,
    ) -> Result<PreviewResult, PreviewError> {
        // TODO: construct validation parameters based on current world state
        let intent_hash_store = TestIntentHashStore::new();
        let validation_params: ValidationParameters = ValidationParameters {
            network: Network::LocalSimulator,
            current_epoch: 1,
            max_cost_unit_limit: DEFAULT_MAX_COST_UNIT_LIMIT,
            min_tip_bps: 0,
        };
        let execution_params: ExecutionParameters = ExecutionParameters::default();

        // validate
        let validated_preview_transaction = TransactionValidator::validate_preview_intent(
            preview_intent.clone(),
            &intent_hash_store,
            &validation_params,
        )
        .map_err(PreviewError::TransactionValidationError)?;

        // execute
        let mut wasm_engine = DefaultWasmEngine::new();
        let mut wasm_instrumenter = WasmInstrumenter::new();
        let mut executor =
            TransactionExecutor::new(substate_store, &mut wasm_engine, &mut wasm_instrumenter);
        let receipt =
            executor.execute_and_commit(&validated_preview_transaction, &execution_params);

        Ok(PreviewResult {
            intent: preview_intent,
            receipt: receipt,
        })
    }
}
