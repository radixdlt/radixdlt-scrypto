use sbor::rust::marker::PhantomData;
use transaction::errors::TransactionValidationError;
use transaction::model::PreviewIntent;
use transaction::validation::*;

use crate::fee::{SystemLoanCostUnitCounter, UnlimitedLoanCostUnitCounter};
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

pub struct PreviewExecutor<'s, 'w, S, W, I, EM, IHM>
where
    S: ReadableSubstateStore + WriteableSubstateStore,
    W: WasmEngine<I>,
    I: WasmInstance,
    EM: EpochManager,
    IHM: IntentHashManager,
{
    substate_store: &'s mut S,
    wasm_engine: &'w mut W,
    wasm_instrumenter: &'w mut WasmInstrumenter,
    epoch_manager: &'w EM,
    intent_hash_manager: &'w IHM,
    phantom1: PhantomData<I>,
}

impl<'s, 'w, S, W, I, EM, IHM> PreviewExecutor<'s, 'w, S, W, I, EM, IHM>
where
    S: ReadableSubstateStore + WriteableSubstateStore,
    W: WasmEngine<I>,
    I: WasmInstance,
    EM: EpochManager,
    IHM: IntentHashManager,
{
    pub fn new(
        substate_store: &'s mut S,
        wasm_engine: &'w mut W,
        wasm_instrumenter: &'w mut WasmInstrumenter,
        epoch_manager: &'w EM,
        intent_hash_manager: &'w IHM,
    ) -> Self {
        PreviewExecutor {
            substate_store,
            wasm_engine,
            wasm_instrumenter,
            epoch_manager,
            intent_hash_manager,
            phantom1: PhantomData,
        }
    }

    pub fn execute(
        &mut self,
        preview_intent: PreviewIntent,
    ) -> Result<PreviewResult, PreviewError> {
        let validated_preview_transaction = TransactionValidator::validate_preview_intent(
            preview_intent.clone(),
            self.intent_hash_manager,
            self.epoch_manager,
        )
        .map_err(PreviewError::TransactionValidationError)?;

        let mut transaction_executor = TransactionExecutor::new(
            self.substate_store,
            self.wasm_engine,
            self.wasm_instrumenter,
            TransactionExecutorConfig::new(false),
        );

        let receipt = if preview_intent.flags.unlimited_loan {
            transaction_executor.execute_with_cost_unit_counter(
                &validated_preview_transaction,
                UnlimitedLoanCostUnitCounter::default(),
            )
        } else {
            transaction_executor.execute_with_cost_unit_counter(
                &validated_preview_transaction,
                SystemLoanCostUnitCounter::default(),
            )
        };

        Ok(PreviewResult {
            intent: preview_intent,
            receipt: receipt,
        })
    }
}
