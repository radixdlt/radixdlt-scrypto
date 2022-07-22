use core::str::FromStr;

use scrypto::math::Decimal;
use scrypto::prelude::Network;
use transaction::errors::TransactionValidationError;
use transaction::model::PreviewIntent;
use transaction::validation::TestIntentHashStore;
use transaction::validation::TransactionValidator;
use transaction::validation::ValidationParameters;

use crate::constants::DEFAULT_MAX_CALL_DEPTH;
use crate::constants::DEFAULT_MAX_COST_UNIT_LIMIT;
use crate::constants::DEFAULT_SYSTEM_LOAN;
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
        // TODO: construct validation parameters based on current world state
        let intent_hash_store = TestIntentHashStore::new();
        let parameters: ValidationParameters = ValidationParameters {
            network: Network::LocalSimulator,
            current_epoch: 1,
            max_cost_unit_limit: DEFAULT_MAX_COST_UNIT_LIMIT,
            min_tip_bps: 0,
        };
        let cost_unit_price = Decimal::from_str("0.000001").unwrap();
        let max_call_depth = DEFAULT_MAX_CALL_DEPTH;
        let system_loan = DEFAULT_SYSTEM_LOAN;

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
            cost_unit_price,
            max_call_depth,
            system_loan,
            false,
            false,
        );

        let receipt = executor.execute(&validated_preview_transaction);

        Ok(PreviewResult {
            intent: preview_intent,
            receipt: receipt,
        })
    }
}
