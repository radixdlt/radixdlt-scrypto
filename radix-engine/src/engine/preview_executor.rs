use crate::fee::DEFAULT_MAX_TRANSACTION_COST;
use sbor::rust::marker::PhantomData;
use transaction::errors::TransactionValidationError;
use transaction::model::{PreviewFlags, PreviewIntent};
use transaction::validation::*;

use crate::engine::*;
use crate::ledger::*;
use crate::wasm::*;

#[derive(Debug)]
pub struct PreviewResult {
    pub intent: PreviewIntent,
    pub receipt: Receipt,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreviewError {
    TransactionValidationError(TransactionValidationError),
}

pub struct PreviewExecutor<'s, 'w, S, W, I>
where
    S: ReadableSubstateStore + WriteableSubstateStore,
    W: WasmEngine<I>,
    I: WasmInstance,
{
    substate_store: &'s mut S,
    wasm_engine: &'w mut W,
    wasm_instrumenter: &'w mut WasmInstrumenter,
    epoch_manager: &'w dyn EpochManager,
    intent_hash_manager: &'w dyn IntentHashManager,
    phantom: PhantomData<I>,
}

impl<'s, 'w, S, W, I> PreviewExecutor<'s, 'w, S, W, I>
where
    S: ReadableSubstateStore + WriteableSubstateStore,
    W: WasmEngine<I>,
    I: WasmInstance,
{
    pub fn new(
        substate_store: &'s mut S,
        wasm_engine: &'w mut W,
        wasm_instrumenter: &'w mut WasmInstrumenter,
        epoch_manager: &'w dyn EpochManager,
        intent_hash_manager: &'w dyn IntentHashManager,
    ) -> PreviewExecutor<'s, 'w, S, W, I> {
        PreviewExecutor {
            substate_store,
            wasm_engine,
            wasm_instrumenter,
            epoch_manager,
            intent_hash_manager,
            phantom: PhantomData,
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

        let mut executor = TransactionExecutor::new(
            self.substate_store,
            self.wasm_engine,
            self.wasm_instrumenter,
            (&preview_intent.flags).into(),
        );

        let receipt = executor.execute(&validated_preview_transaction);

        Ok(PreviewResult {
            intent: preview_intent,
            receipt: receipt,
        })
    }
}

impl Into<TransactionExecutorConfig> for &PreviewFlags {
    fn into(self) -> TransactionExecutorConfig {
        if self.unlimited_loan {
            TransactionExecutorConfig::new(
                false,
                TransactionCostCounterConfig::UnlimitedLoanAndMaxCost {
                    max_transaction_cost: DEFAULT_MAX_TRANSACTION_COST,
                },
            )
        } else {
            TransactionExecutorConfig::default(false)
        }
    }
}
