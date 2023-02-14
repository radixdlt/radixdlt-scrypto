use crate::engine::Track;
use crate::engine::*;
use crate::fee::{FeeReserve, FeeTable, SystemLoanFeeReserve};
use crate::ledger::{ReadableSubstateStore, WriteableSubstateStore};
use crate::model::*;
use crate::transaction::*;
use crate::types::*;
use crate::wasm::*;
use radix_engine_constants::{
    DEFAULT_COST_UNIT_PRICE, DEFAULT_MAX_CALL_DEPTH, DEFAULT_SYSTEM_LOAN,
};
use radix_engine_interface::api::Invokable;
use sbor::rust::borrow::Cow;
use transaction::model::*;

pub struct FeeReserveConfig {
    pub cost_unit_price: u128,
    pub system_loan: u32,
}

impl Default for FeeReserveConfig {
    fn default() -> Self {
        FeeReserveConfig::standard()
    }
}

impl FeeReserveConfig {
    pub fn standard() -> Self {
        Self {
            cost_unit_price: DEFAULT_COST_UNIT_PRICE,
            system_loan: DEFAULT_SYSTEM_LOAN,
        }
    }
}

pub struct ExecutionConfig {
    pub max_call_depth: usize,
    pub trace: bool,
    pub max_sys_call_trace_depth: usize,
    pub abort_when_loan_repaid: bool,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        ExecutionConfig::standard()
    }
}

impl ExecutionConfig {
    pub fn standard() -> Self {
        Self {
            max_call_depth: DEFAULT_MAX_CALL_DEPTH,
            trace: false,
            max_sys_call_trace_depth: 1,
            abort_when_loan_repaid: false,
        }
    }

    pub fn debug() -> Self {
        Self {
            trace: true,
            ..Self::default()
        }
    }

    pub fn with_tracing(trace: bool) -> Self {
        if trace {
            Self::debug()
        } else {
            Self::standard()
        }
    }

    pub fn up_to_loan_repayment() -> Self {
        Self {
            abort_when_loan_repaid: true,
            ..Self::default()
        }
    }

    pub fn up_to_loan_repayment_with_debug() -> Self {
        Self {
            abort_when_loan_repaid: true,
            trace: true,
            ..Self::default()
        }
    }
}

/// An executor that runs transactions.
/// This is no longer public -- it can be removed / merged into the exposed functions in a future small PR
/// But I'm not doing it in this PR to avoid merge conflicts in the body of execute_with_fee_reserve
struct TransactionExecutor<'s, 'w, S, W>
where
    S: ReadableSubstateStore,
    W: WasmEngine,
{
    substate_store: &'s S,
    scrypto_interpreter: &'w ScryptoInterpreter<W>,
}

impl<'s, 'w, S, W> TransactionExecutor<'s, 'w, S, W>
where
    S: ReadableSubstateStore,
    W: WasmEngine,
{
    pub fn new(substate_store: &'s S, scrypto_interpreter: &'w ScryptoInterpreter<W>) -> Self {
        Self {
            substate_store,
            scrypto_interpreter,
        }
    }

    pub fn execute(
        &mut self,
        transaction: &Executable,
        fee_reserve_config: &FeeReserveConfig,
        execution_config: &ExecutionConfig,
    ) -> TransactionReceipt {
        let fee_reserve = match transaction.fee_payment() {
            FeePayment::User {
                cost_unit_limit,
                tip_percentage,
            } => SystemLoanFeeReserve::new(
                fee_reserve_config.cost_unit_price,
                *tip_percentage,
                *cost_unit_limit,
                fee_reserve_config.system_loan,
                execution_config.abort_when_loan_repaid,
            ),
            FeePayment::NoFee => SystemLoanFeeReserve::no_fee(),
        };

        self.execute_with_fee_reserve(transaction, execution_config, fee_reserve)
    }

    fn execute_with_fee_reserve<R: FeeReserve>(
        &mut self,
        transaction: &Executable,
        execution_config: &ExecutionConfig,
        fee_reserve: R,
    ) -> TransactionReceipt {
        let transaction_hash = transaction.transaction_hash();
        let auth_zone_params = transaction.auth_zone_params();
        let pre_allocated_ids = transaction.pre_allocated_ids();
        let instructions = transaction.instructions();
        let blobs = transaction.blobs();

        #[cfg(not(feature = "alloc"))]
        if execution_config.trace {
            println!("{:-^80}", "Transaction Metadata");
            println!("Transaction hash: {}", transaction_hash);
            println!("Transaction auth zone params: {:?}", auth_zone_params);
            println!("Number of unique blobs: {}", blobs.len());

            println!("{:-^80}", "Engine Execution Log");
        }

        // Prepare state track and execution trace
        let track = Track::new(self.substate_store, fee_reserve, FeeTable::new());

        // Apply pre execution costing
        let pre_execution_result = track.apply_pre_execution_costs(transaction);
        let mut track = match pre_execution_result {
            Ok(track) => track,
            Err(err) => {
                return TransactionReceipt {
                    execution: TransactionExecution {
                        fee_summary: err.fee_summary,
                        events: vec![],
                    },
                    result: TransactionResult::Reject(RejectResult {
                        error: RejectionError::ErrorBeforeFeeLoanRepaid(RuntimeError::ModuleError(
                            ModuleError::CostingError(CostingError::FeeReserveError(err.error)),
                        )),
                    }),
                };
            }
        };

        // Invoke the function/method
        let track_receipt = {
            let mut module = KernelModule::new(execution_config);
            let mut id_allocator =
                IdAllocator::new(transaction_hash.clone(), pre_allocated_ids.clone());

            let mut kernel = Kernel::new(
                auth_zone_params.clone(),
                &mut id_allocator,
                &mut track,
                self.scrypto_interpreter,
                &mut module,
            );

            let invoke_result = kernel.invoke(TransactionProcessorRunInvocation {
                transaction_hash: transaction_hash.clone(),
                runtime_validations: Cow::Borrowed(transaction.runtime_validations()),
                instructions: match instructions {
                    InstructionList::Basic(instructions) => {
                        Cow::Owned(instructions.iter().map(|e| e.clone().into()).collect())
                    }
                    InstructionList::Any(instructions) => Cow::Borrowed(instructions),
                    InstructionList::AnyOwned(instructions) => Cow::Borrowed(instructions),
                },
                blobs: Cow::Borrowed(blobs),
            });

            let events = module.collect_events();
            track.finalize(invoke_result, events)
        };

        let receipt = TransactionReceipt {
            execution: TransactionExecution {
                fee_summary: track_receipt.fee_summary,
                events: track_receipt.events,
            },
            result: track_receipt.result,
        };
        #[cfg(not(feature = "alloc"))]
        if execution_config.trace {
            println!("{:-^80}", "Cost Analysis");
            let break_down = receipt
                .execution
                .fee_summary
                .execution_cost_unit_breakdown
                .iter()
                .collect::<BTreeMap<&String, &u32>>();
            for (k, v) in break_down {
                println!("{:<30}: {:>10}", k, v);
            }

            println!("{:-^80}", "Cost Totals");
            println!(
                "{:<30}: {:>10}",
                "Total Cost Units Consumed", receipt.execution.fee_summary.cost_unit_consumed
            );
            println!(
                "{:<30}: {:>10}",
                "Cost Unit Limit", receipt.execution.fee_summary.cost_unit_limit
            );
            // NB - we use "to_string" to ensure they align correctly
            println!(
                "{:<30}: {:>10}",
                "Execution XRD",
                receipt
                    .execution
                    .fee_summary
                    .total_execution_cost_xrd
                    .to_string()
            );
            println!(
                "{:<30}: {:>10}",
                "Royalty XRD",
                receipt
                    .execution
                    .fee_summary
                    .total_royalty_cost_xrd
                    .to_string()
            );

            match &receipt.result {
                TransactionResult::Commit(commit) => {
                    println!("{:-^80}", "Application Logs");
                    for (level, message) in &commit.application_logs {
                        println!("[{}] {}", level, message);
                    }
                    if commit.application_logs.is_empty() {
                        println!("None");
                    }
                }
                _ => {}
            }
        }
        receipt
    }
}

pub fn execute_and_commit_transaction<
    S: ReadableSubstateStore + WriteableSubstateStore,
    W: WasmEngine,
>(
    substate_store: &mut S,
    scrypto_interpreter: &ScryptoInterpreter<W>,
    fee_reserve_config: &FeeReserveConfig,
    execution_config: &ExecutionConfig,
    transaction: &Executable,
) -> TransactionReceipt {
    let receipt = execute_transaction(
        substate_store,
        scrypto_interpreter,
        fee_reserve_config,
        execution_config,
        transaction,
    );
    if let TransactionResult::Commit(commit) = &receipt.result {
        commit.state_updates.commit(substate_store);
    }
    receipt
}

pub fn execute_transaction<S: ReadableSubstateStore, W: WasmEngine>(
    substate_store: &S,
    scrypto_interpreter: &ScryptoInterpreter<W>,
    fee_reserve_config: &FeeReserveConfig,
    execution_config: &ExecutionConfig,
    transaction: &Executable,
) -> TransactionReceipt {
    TransactionExecutor::new(substate_store, scrypto_interpreter).execute(
        transaction,
        fee_reserve_config,
        execution_config,
    )
}
