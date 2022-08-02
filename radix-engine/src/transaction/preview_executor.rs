use sbor::rust::marker::PhantomData;
use scrypto::prelude::Network;
use transaction::errors::TransactionValidationError;
use transaction::model::PreviewIntent;
use transaction::validation::IntentHashManager;
use transaction::validation::TransactionValidator;
use transaction::validation::ValidationParameters;

use crate::constants::DEFAULT_MAX_COST_UNIT_LIMIT;
use crate::fee::SystemLoanCostUnitCounter;
use crate::fee::UnlimitedLoanCostUnitCounter;
use crate::ledger::*;
use crate::transaction::TransactionReceipt;
use crate::transaction::*;
use crate::wasm::{WasmEngine, WasmInstance, WasmInstrumenter};

#[derive(Debug)]
pub struct PreviewResult {
    pub intent: PreviewIntent,
    pub receipt: TransactionReceipt,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreviewError {
    TransactionValidationError(TransactionValidationError),
}

pub struct PreviewExecutor<'s, 'w, S, W, I, IHM>
where
    S: ReadableSubstateStore + WriteableSubstateStore,
    W: WasmEngine<I>,
    I: WasmInstance,
    IHM: IntentHashManager,
{
    substate_store: &'s mut S,
    wasm_engine: &'w mut W,
    wasm_instrumenter: &'w mut WasmInstrumenter,
    intent_hash_manager: &'w IHM,
    phantom1: PhantomData<I>,
}

impl<'s, 'w, S, W, I, IHM> PreviewExecutor<'s, 'w, S, W, I, IHM>
where
    S: ReadableSubstateStore + WriteableSubstateStore,
    W: WasmEngine<I>,
    I: WasmInstance,
    IHM: IntentHashManager,
{
    pub fn new(
        substate_store: &'s mut S,
        wasm_engine: &'w mut W,
        wasm_instrumenter: &'w mut WasmInstrumenter,
        intent_hash_manager: &'w IHM,
    ) -> Self {
        PreviewExecutor {
            substate_store,
            wasm_engine,
            wasm_instrumenter,
            intent_hash_manager,
            phantom1: PhantomData,
        }
    }

    pub fn execute(
        &mut self,
        preview_intent: PreviewIntent,
    ) -> Result<PreviewResult, PreviewError> {
        // TODO: construct validation parameters based on current world state
        let validation_params = ValidationParameters {
            network: Network::LocalSimulator,
            current_epoch: 1,
            max_cost_unit_limit: DEFAULT_MAX_COST_UNIT_LIMIT,
            min_tip_percentage: 0,
        };
        let execution_params = ExecutionParameters::default();

        let validated_preview_transaction = TransactionValidator::validate_preview_intent(
            preview_intent.clone(),
            self.intent_hash_manager,
            &validation_params,
        )
        .map_err(PreviewError::TransactionValidationError)?;

        let mut transaction_executor = TransactionExecutor::new(
            self.substate_store,
            self.wasm_engine,
            self.wasm_instrumenter,
        );

        let receipt = if preview_intent.flags.unlimited_loan {
            transaction_executor.execute_with_cost_unit_counter(
                &validated_preview_transaction,
                &execution_params,
                UnlimitedLoanCostUnitCounter::default(),
            )
        } else {
            transaction_executor.execute_with_cost_unit_counter(
                &validated_preview_transaction,
                &execution_params,
                SystemLoanCostUnitCounter::default(),
            )
        };

        Ok(PreviewResult {
            intent: preview_intent,
            receipt: receipt,
        })
    }
}
