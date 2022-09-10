use transaction::model::*;
use transaction::validation::TransactionValidator;

use crate::constants::{DEFAULT_COST_UNIT_PRICE, DEFAULT_MAX_CALL_DEPTH, DEFAULT_SYSTEM_LOAN};
use crate::engine::Track;
use crate::engine::*;
use crate::fee::{FeeReserve, FeeTable, SystemLoanFeeReserve, UnlimitedLoanFeeReserve};
use crate::ledger::{ReadableSubstateStore, WriteableSubstateStore};
use crate::model::*;
use crate::transaction::*;
use crate::types::*;
use crate::wasm::*;

pub struct ExecutionConfig {
    pub cost_unit_price: Decimal,
    pub system_loan: u32,
    pub max_call_depth: usize,
    pub execution_privilege: ExecutionPrivilege,
    pub trace: bool,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        ExecutionConfig::standard()
    }
}

impl ExecutionConfig {
    pub fn standard() -> Self {
        Self {
            cost_unit_price: DEFAULT_COST_UNIT_PRICE
                .parse()
                .expect("Invalid cost unit price"),
            system_loan: DEFAULT_SYSTEM_LOAN,
            max_call_depth: DEFAULT_MAX_CALL_DEPTH,
            execution_privilege: ExecutionPrivilege::User,
            trace: false,
        }
    }

    pub fn debug() -> Self {
        Self {
            cost_unit_price: DEFAULT_COST_UNIT_PRICE
                .parse()
                .expect("Invalid cost unit price"),
            system_loan: DEFAULT_SYSTEM_LOAN,
            max_call_depth: DEFAULT_MAX_CALL_DEPTH,
            execution_privilege: ExecutionPrivilege::User,
            trace: true,
        }
    }
}

/// An executor that runs transactions.
pub struct TransactionExecutor<'s, 'w, S, W, I>
where
    S: ReadableSubstateStore,
    W: WasmEngine<I>,
    I: WasmInstance,
{
    substate_store: &'s mut S,
    wasm_engine: &'w mut W,
    wasm_instrumenter: &'w mut WasmInstrumenter,
    phantom: PhantomData<I>,
}

impl<'s, 'w, S, W, I> TransactionExecutor<'s, 'w, S, W, I>
where
    S: ReadableSubstateStore,
    W: WasmEngine<I>,
    I: WasmInstance,
{
    pub fn new(
        substate_store: &'s mut S,
        wasm_engine: &'w mut W,
        wasm_instrumenter: &'w mut WasmInstrumenter,
    ) -> Self {
        Self {
            substate_store,
            wasm_engine,
            wasm_instrumenter,
            phantom: PhantomData,
        }
    }

    pub fn execute_system_transaction(
        &mut self,
        transaction: SystemTransaction,
    ) -> TransactionReceipt {
        let mut execution_trace = ExecutionTrace::new();
        let mut track = Track::new(self.substate_store, UnlimitedLoanFeeReserve::default());

        let mut kernel = Kernel::new(
            Hash([0u8; Hash::LENGTH]),
            vec![],
            ExecutionPrivilege::System,
            DEFAULT_MAX_CALL_DEPTH,
            &mut track,
            self.wasm_engine,
            self.wasm_instrumenter,
            WasmMeteringParams::new(InstructionCostRules::tiered(1, 5, 10, 5000), 512),
            &mut execution_trace,
            vec![],
        );

        let instructions = TransactionValidator::validate_manifest(&transaction.manifest).unwrap();

        let invoke_result = {
            let result = kernel.invoke_function(
                FnIdentifier::Native(NativeFnIdentifier::TransactionProcessor(
                    TransactionProcessorFnIdentifier::Run,
                )),
                ScryptoValue::from_typed(&TransactionProcessorRunInput {
                    instructions: instructions.clone(),
                }),
            );
            result.map(|o| {
                scrypto_decode::<Vec<Vec<u8>>>(&o.raw)
                    .expect("TransactionProcessor returned data of unexpected type")
            })
        };

        let execution_trace_receipt = execution_trace.to_receipt();
        let track_receipt = track.finalize(invoke_result, execution_trace_receipt.resource_changes);
        let receipt = TransactionReceipt {
            contents: TransactionContents { instructions },
            execution: TransactionExecution {
                fee_summary: track_receipt.fee_summary,
                application_logs: track_receipt.application_logs,
            },
            result: track_receipt.result,
        };

        receipt
    }

    pub fn execute<T: ExecutableTransaction>(
        &mut self,
        transaction: &T,
        execution_config: &ExecutionConfig,
    ) -> TransactionReceipt {
        let fee_reserve = SystemLoanFeeReserve::new(
            transaction.cost_unit_limit(),
            transaction.tip_percentage(),
            execution_config.cost_unit_price,
            execution_config.system_loan,
        );

        self.execute_with_fee_reserve(transaction, execution_config, fee_reserve)
    }

    pub fn execute_with_fee_reserve<T: ExecutableTransaction, R: FeeReserve>(
        &mut self,
        transaction: &T,
        execution_config: &ExecutionConfig,
        mut fee_reserve: R,
    ) -> TransactionReceipt {
        let transaction_hash = transaction.transaction_hash();
        let signer_public_keys = transaction.signer_public_keys().to_vec();
        let instructions = transaction.instructions().to_vec();

        #[cfg(not(feature = "alloc"))]
        if execution_config.trace {
            println!("{:-^80}", "Transaction Metadata");
            println!("Transaction hash: {}", transaction_hash);
            println!("Transaction signers: {:?}", signer_public_keys);

            println!("{:-^80}", "Engine Execution Log");
        }

        // Prepare state track and execution trace
        let fee_table = FeeTable::new();
        CostingModule::apply_static_fees(&mut fee_reserve, &fee_table, transaction);
        let mut track = Track::new(self.substate_store, fee_reserve);
        let mut execution_trace = ExecutionTrace::new();

        // Invoke the function/method
        let invoke_result = {
            let mut modules = Vec::<Box<dyn Module<R>>>::new();
            if execution_config.trace {
                modules.push(Box::new(LoggerModule::new()));
            }
            modules.push(Box::new(CostingModule::new(fee_table)));

            let mut kernel = Kernel::new(
                transaction_hash,
                signer_public_keys,
                execution_config.execution_privilege,
                execution_config.max_call_depth,
                &mut track,
                self.wasm_engine,
                self.wasm_instrumenter,
                WasmMeteringParams::new(InstructionCostRules::tiered(1, 5, 10, 5000), 512), // TODO: add to ExecutionConfig
                &mut execution_trace,
                modules,
            );
            let result = kernel.invoke_function(
                FnIdentifier::Native(NativeFnIdentifier::TransactionProcessor(
                    TransactionProcessorFnIdentifier::Run,
                )),
                ScryptoValue::from_typed(&TransactionProcessorRunInput {
                    instructions: instructions.clone(),
                }),
            );
            result.map(|o| {
                scrypto_decode::<Vec<Vec<u8>>>(&o.raw)
                    .expect("TransactionProcessor returned data of unexpected type")
            })
        };

        // Produce the final transaction receipt
        let execution_trace_receipt = execution_trace.to_receipt();
        let track_receipt = track.finalize(invoke_result, execution_trace_receipt.resource_changes);
        let receipt = TransactionReceipt {
            contents: TransactionContents { instructions },
            execution: TransactionExecution {
                fee_summary: track_receipt.fee_summary,
                application_logs: track_receipt.application_logs,
            },
            result: track_receipt.result,
        };
        #[cfg(not(feature = "alloc"))]
        if execution_config.trace {
            println!("{:-^80}", "Cost Analysis");
            for (k, v) in &receipt.execution.fee_summary.cost_breakdown {
                println!("{:<30}: {:>8}", k, v);
            }

            println!("{:-^80}", "Application Logs");
            for (level, message) in &receipt.execution.application_logs {
                println!("[{}] {}", level, message);
            }
            if receipt.execution.application_logs.is_empty() {
                println!("None");
            }
        }
        receipt
    }
}

impl<'s, 'w, S, W, I> TransactionExecutor<'s, 'w, S, W, I>
where
    S: ReadableSubstateStore + WriteableSubstateStore,
    W: WasmEngine<I>,
    I: WasmInstance,
{
    pub fn execute_and_commit<T: ExecutableTransaction>(
        &mut self,
        transaction: &T,
        execution_config: &ExecutionConfig,
    ) -> TransactionReceipt {
        let receipt = self.execute(transaction, execution_config);
        if let TransactionResult::Commit(commit) = &receipt.result {
            commit.state_updates.commit(self.substate_store);
        }
        receipt
    }
}
