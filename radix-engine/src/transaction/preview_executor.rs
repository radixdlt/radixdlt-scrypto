use scrypto::core::NetworkDefinition;
use transaction::errors::TransactionValidationError;
use transaction::model::PreviewIntent;
use transaction::validation::IntentHashManager;
use transaction::validation::TransactionValidator;
use transaction::validation::ValidationConfig;

use crate::constants::DEFAULT_MAX_COST_UNIT_LIMIT;
use crate::constants::PREVIEW_CREDIT;
use crate::fee::SystemLoanFeeReserve;
use crate::ledger::*;
use crate::transaction::TransactionReceipt;
use crate::transaction::*;
use crate::types::*;
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

pub struct PreviewExecutor<'s, 'w, 'n, S, W, I, IHM>
where
    S: ReadableSubstateStore,
    W: WasmEngine<I>,
    I: WasmInstance,
    IHM: IntentHashManager,
{
    substate_store: &'s mut S,
    wasm_engine: &'w mut W,
    wasm_instrumenter: &'w mut WasmInstrumenter,
    intent_hash_manager: &'w IHM,
    network: &'n NetworkDefinition,
    phantom1: PhantomData<I>,
}

impl<'s, 'w, 'n, S, W, I, IHM> PreviewExecutor<'s, 'w, 'n, S, W, I, IHM>
where
    S: ReadableSubstateStore,
    W: WasmEngine<I>,
    I: WasmInstance,
    IHM: IntentHashManager,
{
    pub fn new(
        substate_store: &'s mut S,
        wasm_engine: &'w mut W,
        wasm_instrumenter: &'w mut WasmInstrumenter,
        intent_hash_manager: &'w IHM,
        network: &'n NetworkDefinition,
    ) -> Self {
        PreviewExecutor {
            substate_store,
            wasm_engine,
            wasm_instrumenter,
            intent_hash_manager,
            network,
            phantom1: PhantomData,
        }
    }

    pub fn execute(
        &mut self,
        preview_intent: PreviewIntent,
    ) -> Result<PreviewResult, PreviewError> {
        // TODO: construct validation config based on current world state
        let validation_config = ValidationConfig {
            network_id: self.network.id,
            current_epoch: 1,
            max_cost_unit_limit: DEFAULT_MAX_COST_UNIT_LIMIT,
            min_tip_percentage: 0,
        };
        let execution_params = ExecutionConfig::default();
        let validator = TransactionValidator::new(validation_config, false);

        let validated_preview_transaction = validator
            .validate_preview_intent(preview_intent.clone(), self.intent_hash_manager)
            .map_err(PreviewError::TransactionValidationError)?;

        let mut transaction_executor = TransactionExecutor::new(
            self.substate_store,
            self.wasm_engine,
            self.wasm_instrumenter,
        );

        let mut fee_reserve = SystemLoanFeeReserve::default();
        if preview_intent.flags.unlimited_loan {
            fee_reserve.credit(PREVIEW_CREDIT);
        }
        let receipt = transaction_executor.execute_with_fee_reserve(
            &validated_preview_transaction,
            &execution_params,
            SystemLoanFeeReserve::default(),
        );

        Ok(PreviewResult {
            intent: preview_intent,
            receipt,
        })
    }
}
