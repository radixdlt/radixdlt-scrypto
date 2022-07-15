use transaction::model::PreviewIntent;
use transaction::validation::TestEpochManager;
use transaction::validation::TestIntentHashManager;
use transaction::validation::TransactionValidator;

use crate::engine::*;
use crate::ledger::*;
use crate::wasm::{DefaultWasmEngine, WasmInstrumenter};

#[derive(Debug)]
pub struct PreviewResult {
    pub intent: PreviewIntent,
    pub receipt: Receipt,
}

pub struct PreviewExecutor;

impl PreviewExecutor {
    pub fn execute_preview<S: ReadableSubstateStore + WriteableSubstateStore>(
        preview_intent: PreviewIntent,
        substate_store: &mut S,
    ) -> PreviewResult {
        let epoch_manager = TestEpochManager::new(0);
        let intent_hash_manager = TestIntentHashManager::new();

        let validated_preview_transaction = TransactionValidator::validate_preview_intent(
            preview_intent.clone(),
            &intent_hash_manager,
            &epoch_manager,
        );

        let mut wasm_engine = DefaultWasmEngine::new();
        let mut wasm_instrumenter = WasmInstrumenter::new();
        let mut executor = TransactionExecutor::new(
            substate_store,
            &mut wasm_engine,
            &mut wasm_instrumenter,
            false,
        );

        let receipt = executor.execute(&validated_preview_transaction.unwrap());

        PreviewResult {
            intent: preview_intent,
            receipt: receipt,
        }
    }
}
