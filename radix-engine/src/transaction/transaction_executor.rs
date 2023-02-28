use crate::blueprints::transaction_processor::TransactionProcessorRunInvocation;
use crate::errors::*;
use crate::kernel::id_allocator::IdAllocator;
use crate::kernel::interpreters::ScryptoInterpreter;
use crate::kernel::kernel::Kernel;
use crate::kernel::kernel_api::Invokable;
use crate::kernel::module_mixer::KernelModuleMixer;
use crate::kernel::track::{PreExecutionError, Track};
use crate::ledger::{ReadableSubstateStore, WriteableSubstateStore};
use crate::system::kernel_modules::costing::*;
use crate::transaction::*;
use crate::types::*;
use crate::wasm::*;
use radix_engine_constants::*;
use sbor::rust::borrow::Cow;
use transaction::data::manifest_encode;
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
    pub debug: bool,
    pub max_call_depth: usize,
    pub max_kernel_call_depth_traced: Option<usize>,
    pub abort_when_loan_repaid: bool,
    pub max_wasm_mem_per_transaction: usize,
    pub max_wasm_mem_per_call_frame: usize,
    pub max_substate_reads_per_transaction: usize,
    pub max_substate_writes_per_transaction: usize,
    pub max_substate_size: usize,
    pub max_invoke_input_size: usize,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        ExecutionConfig::standard()
    }
}

impl ExecutionConfig {
    pub fn standard() -> Self {
        Self {
            debug: false,
            max_call_depth: DEFAULT_MAX_CALL_DEPTH,
            max_kernel_call_depth_traced: Some(1),
            abort_when_loan_repaid: false,
            max_wasm_mem_per_transaction: DEFAULT_MAX_WASM_MEM_PER_TRANSACTION,
            max_wasm_mem_per_call_frame: DEFAULT_MAX_WASM_MEM_PER_CALL_FRAME,
            max_substate_reads_per_transaction: DEFAULT_MAX_SUBSTATE_READS_PER_TRANSACTION,
            max_substate_writes_per_transaction: DEFAULT_MAX_SUBSTATE_WRITES_PER_TRANSACTION,
            max_substate_size: DEFAULT_MAX_SUBSTATE_SIZE,
            max_invoke_input_size: DEFAULT_MAX_INVOKE_INPUT_SIZE,
        }
    }

    pub fn debug() -> Self {
        Self {
            debug: true,
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
            debug: true,
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

        self.execute_with_fee_reserve(transaction, execution_config, fee_reserve, FeeTable::new())
    }

    fn apply_pre_execution_costs(
        mut fee_reserve: SystemLoanFeeReserve,
        fee_table: &FeeTable,
        executable: &Executable,
    ) -> Result<SystemLoanFeeReserve, PreExecutionError> {
        let result = fee_reserve
            .consume_deferred(fee_table.tx_base_fee(), 1, CostingReason::TxBaseCost)
            .and_then(|()| {
                fee_reserve.consume_deferred(
                    fee_table.tx_payload_cost_per_byte(),
                    executable.payload_size(),
                    CostingReason::TxPayloadCost,
                )
            })
            .and_then(|()| {
                fee_reserve.consume_deferred(
                    fee_table.tx_signature_verification_per_sig(),
                    executable.auth_zone_params().initial_proofs.len(),
                    CostingReason::TxSignatureVerification,
                )
            });

        match result {
            Ok(_) => Ok(fee_reserve),
            Err(e) => Err(PreExecutionError {
                fee_summary: fee_reserve.finalize(),
                error: e,
            }),
        }
    }

    fn execute_with_fee_reserve(
        &mut self,
        executable: &Executable,
        execution_config: &ExecutionConfig,
        fee_reserve: SystemLoanFeeReserve,
        fee_table: FeeTable,
    ) -> TransactionReceipt {
        let transaction_hash = executable.transaction_hash();

        #[cfg(not(feature = "alloc"))]
        if execution_config.debug {
            println!("{:-^80}", "Transaction Metadata");
            println!("Transaction hash: {}", transaction_hash);
            println!(
                "Transaction auth zone params: {:?}",
                executable.pre_allocated_ids()
            );
            println!("Number of unique blobs: {}", executable.blobs().len());

            println!("{:-^80}", "Engine Execution Log");
        }

        // Start resources usage measurement
        #[cfg(all(target_os = "linux", feature = "std", feature = "cpu_ram_metrics"))]
        let mut resources_tracker =
            crate::kernel::resources_tracker::ResourcesTracker::start_measurement();

        // Apply pre execution costing
        let pre_execution_result =
            Self::apply_pre_execution_costs(fee_reserve, &fee_table, executable);
        let fee_reserve = match pre_execution_result {
            Ok(fee_reserve) => fee_reserve,
            Err(err) => {
                return TransactionReceipt {
                    execution: TransactionExecution {
                        fee_summary: err.fee_summary,
                        events: vec![],
                        resources_usage: ResourcesUsage::default(),
                    },
                    result: TransactionResult::Reject(RejectResult {
                        error: RejectionError::ErrorBeforeFeeLoanRepaid(RuntimeError::ModuleError(
                            ModuleError::CostingError(CostingError::FeeReserveError(err.error)),
                        )),
                    }),
                };
            }
        };

        // Prepare state track and execution trace
        let mut track = Track::new(self.substate_store);

        // Invoke the function/method
        let track_receipt = {
            let mut id_allocator = IdAllocator::new(
                transaction_hash.clone(),
                executable.pre_allocated_ids().clone(),
            );

            // Create kernel
            let modules = KernelModuleMixer::standard(
                execution_config.debug,
                transaction_hash.clone(),
                executable.auth_zone_params().clone(),
                fee_reserve,
                fee_table,
                execution_config,
            );
            let mut kernel = Kernel::new(
                &mut id_allocator,
                &mut track,
                self.scrypto_interpreter,
                modules,
            );

            // Initialize
            kernel.initialize().expect("Failed to initialize kernel");

            // Invoke transaction processor
            let invoke_result = kernel.kernel_invoke(TransactionProcessorRunInvocation {
                transaction_hash: transaction_hash.clone(),
                runtime_validations: Cow::Borrowed(executable.runtime_validations()),
                instructions: Cow::Owned(manifest_encode(executable.instructions()).unwrap()),
                blobs: Cow::Borrowed(executable.blobs()),
            });

            // Teardown
            let (modules, invoke_result) = kernel.teardown(invoke_result);
            let fee_reserve = modules.costing.take_fee_reserve();
            let (vault_ops, events) = modules.execution_trace.collect_events();

            // Finalize track
            track.finalize(invoke_result, fee_reserve, vault_ops, events)
        };

        // Finish resources usage measurement and get results
        let resources_usage = match () {
            #[cfg(not(all(target_os = "linux", feature = "std", feature = "cpu_ram_metrics")))]
            () => ResourcesUsage::default(),
            #[cfg(all(target_os = "linux", feature = "std", feature = "cpu_ram_metrics"))]
            () => resources_tracker.end_measurement(),
        };

        let receipt = TransactionReceipt {
            execution: TransactionExecution {
                fee_summary: track_receipt.fee_summary,
                events: track_receipt.events,
                resources_usage,
            },
            result: track_receipt.result,
        };
        #[cfg(not(feature = "alloc"))]
        if execution_config.debug {
            println!("{:-^80}", "Cost Analysis");
            let break_down = receipt
                .execution
                .fee_summary
                .execution_cost_unit_breakdown
                .iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect::<BTreeMap<String, &u32>>();
            for (k, v) in break_down {
                println!("{:<30}: {:>10}", k, v);
            }

            println!("{:-^80}", "Cost Totals");
            println!(
                "{:<30}: {:>10}",
                "Total Cost Units Consumed",
                receipt.execution.fee_summary.total_cost_units_consumed
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
                TransactionResult::Reject(e) => {
                    println!("{:-^80}", "Transaction Rejected");
                    println!("{:?}", e.error);
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
