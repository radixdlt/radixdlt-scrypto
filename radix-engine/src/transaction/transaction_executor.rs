use crate::blueprints::consensus_manager::{
    ConsensusManagerField, ConsensusManagerStateFieldPayload,
    ConsensusManagerValidatorRewardsFieldPayload,
};
use crate::blueprints::models::FieldPayload;
use crate::blueprints::resource::{
    fungible_vault::DepositEvent, fungible_vault::PayFeeEvent, BurnFungibleResourceEvent,
    FungibleVaultBalanceFieldPayload, FungibleVaultBalanceFieldSubstate, FungibleVaultField,
};
use crate::blueprints::transaction_tracker::{
    TransactionStatus, TransactionStatusV1, TransactionTrackerSubstate,
};
use crate::errors::*;
use crate::internal_prelude::KeyValueEntrySubstateV1;
use crate::internal_prelude::*;
use crate::kernel::id_allocator::IdAllocator;
use crate::kernel::kernel::BootLoader;
use crate::kernel::kernel_callback_api::*;
use crate::system::system_callback::{System, SystemInit};
use crate::system::system_callback_api::SystemCallbackObject;
use crate::system::system_db_reader::SystemDatabaseReader;
use crate::system::system_modules::costing::*;
use crate::system::system_modules::execution_trace::ExecutionTraceModule;
use crate::system::system_modules::transaction_runtime::TransactionRuntimeModule;
use crate::system::system_substates::KeyValueEntrySubstate;
use crate::system::system_substates::{FieldSubstate, LockStatus};
use crate::track::interface::CommitableSubstateStore;
use crate::track::{to_state_updates, BootStore, Track, TrackFinalizeError};
use crate::transaction::*;
use crate::vm::wasm::WasmEngine;
use crate::vm::{NativeVmExtension, Vm, VmInit};
use radix_common::constants::*;
use radix_engine_interface::api::ModuleId;
use radix_engine_interface::blueprints::resource::LiquidFungibleResource;
use radix_engine_interface::blueprints::transaction_processor::InstructionOutput;
use radix_substate_store_interface::db_key_mapper::DatabaseKeyMapper;
use radix_substate_store_interface::{db_key_mapper::SpreadPrefixKeyMapper, interface::*};
use radix_transactions::model::*;
use crate::system::system_modules::EnabledModules;

/// Protocol-defined costing parameters
#[derive(Debug, Copy, Clone, ScryptoSbor, PartialEq, Eq)]
pub struct CostingParameters {
    /// The price of execution cost unit in XRD.
    pub execution_cost_unit_price: Decimal,
    /// The max number execution cost units to consume.
    pub execution_cost_unit_limit: u32,
    /// The number of execution cost units loaned from system.
    pub execution_cost_unit_loan: u32,

    /// The price of finalization cost unit in XRD.
    pub finalization_cost_unit_price: Decimal,
    /// The max number finalization cost units to consume.
    pub finalization_cost_unit_limit: u32,

    /// The price of USD in xrd
    pub usd_price: Decimal,
    /// The price of state storage in xrd
    pub state_storage_price: Decimal,
    /// The price of archive storage in xrd
    pub archive_storage_price: Decimal,
}

impl CostingParameters {
    #[cfg(not(feature = "coverage"))]
    pub fn babylon_genesis() -> Self {
        Self {
            execution_cost_unit_price: EXECUTION_COST_UNIT_PRICE_IN_XRD.try_into().unwrap(),
            execution_cost_unit_limit: EXECUTION_COST_UNIT_LIMIT,
            execution_cost_unit_loan: EXECUTION_COST_UNIT_LOAN,
            finalization_cost_unit_price: FINALIZATION_COST_UNIT_PRICE_IN_XRD.try_into().unwrap(),
            finalization_cost_unit_limit: FINALIZATION_COST_UNIT_LIMIT,
            usd_price: USD_PRICE_IN_XRD.try_into().unwrap(),
            state_storage_price: STATE_STORAGE_PRICE_IN_XRD.try_into().unwrap(),
            archive_storage_price: ARCHIVE_STORAGE_PRICE_IN_XRD.try_into().unwrap(),
        }
    }
    #[cfg(feature = "coverage")]
    pub fn babylon_genesis() -> Self {
        Self {
            execution_cost_unit_price: Decimal::zero(),
            execution_cost_unit_limit: u32::MAX,
            execution_cost_unit_loan: u32::MAX,
            finalization_cost_unit_price: Decimal::zero(),
            finalization_cost_unit_limit: u32::MAX,
            usd_price: USD_PRICE_IN_XRD.try_into().unwrap(),
            state_storage_price: Decimal::zero(),
            archive_storage_price: Decimal::zero(),
        }
    }

    pub fn with_execution_cost_unit_limit(mut self, limit: u32) -> Self {
        self.execution_cost_unit_limit = limit;
        self
    }

    pub fn with_finalization_cost_unit_limit(mut self, limit: u32) -> Self {
        self.finalization_cost_unit_limit = limit;
        self
    }
}

#[derive(Debug, Copy, Clone, ScryptoSbor, PartialEq, Eq)]
pub struct LimitParameters {
    pub max_call_depth: usize,
    pub max_heap_substate_total_bytes: usize,
    pub max_track_substate_total_bytes: usize,
    pub max_substate_key_size: usize,
    pub max_substate_value_size: usize,
    pub max_invoke_input_size: usize,
    pub max_event_size: usize,
    pub max_log_size: usize,
    pub max_panic_message_size: usize,
    pub max_number_of_logs: usize,
    pub max_number_of_events: usize,
}

impl LimitParameters {
    pub fn babylon_genesis() -> Self {
        Self {
            max_call_depth: MAX_CALL_DEPTH,
            max_heap_substate_total_bytes: MAX_HEAP_SUBSTATE_TOTAL_BYTES,
            max_track_substate_total_bytes: MAX_TRACK_SUBSTATE_TOTAL_BYTES,
            max_substate_key_size: MAX_SUBSTATE_KEY_SIZE,
            max_substate_value_size: MAX_SUBSTATE_VALUE_SIZE,
            max_invoke_input_size: MAX_INVOKE_PAYLOAD_SIZE,
            max_event_size: MAX_EVENT_SIZE,
            max_log_size: MAX_LOG_SIZE,
            max_panic_message_size: MAX_PANIC_MESSAGE_SIZE,
            max_number_of_logs: MAX_NUMBER_OF_LOGS,
            max_number_of_events: MAX_NUMBER_OF_EVENTS,
        }
    }

    pub fn for_genesis_transaction() -> Self {
        Self {
            max_heap_substate_total_bytes: 512 * 1024 * 1024,
            max_track_substate_total_bytes: 512 * 1024 * 1024,
            max_number_of_events: 1024 * 1024,
            ..Self::babylon_genesis()
        }
    }
}

#[derive(Debug, Clone)]
pub struct SystemOverrides {
    pub disable_costing: bool,
    pub disable_limits: bool,
    pub disable_auth: bool,
    /// This is required for pre-bottlenose testnets which need to override
    /// the default Mainnet network definition
    pub network_definition: Option<NetworkDefinition>,
    pub costing_parameters: Option<CostingParameters>,
    pub limit_parameters: Option<LimitParameters>,
}

impl Default for SystemOverrides {
    fn default() -> Self {
        Self {
            disable_costing: false,
            disable_limits: false,
            disable_auth: false,
            network_definition: None,
            costing_parameters: None,
            limit_parameters: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExecutionConfig {
    // These parameters do not affect state execution but only affect side effects
    pub enable_kernel_trace: bool,
    pub enable_cost_breakdown: bool,
    pub execution_trace: Option<usize>,

    pub network_definition: Option<NetworkDefinition>,
    pub system_overrides: Option<SystemOverrides>,
}

impl ExecutionConfig {
    /// Creates an `ExecutionConfig` using default configurations.
    /// This is internal. Clients should use `for_xxx` constructors instead.
    fn default(network_definition: NetworkDefinition) -> Self {
        Self {
            network_definition: Some(network_definition),
            enable_kernel_trace: false,
            enable_cost_breakdown: false,
            execution_trace: None,
            system_overrides: None,
        }
    }

    pub fn for_genesis_transaction(network_definition: NetworkDefinition) -> Self {
        Self {
            system_overrides: Some(SystemOverrides {
                disable_costing: true,
                disable_limits: true,
                disable_auth: true,
                ..Default::default()
            }),
            ..Self::default(network_definition)
        }
    }

    pub fn for_system_transaction(network_definition: NetworkDefinition) -> Self {
        Self {
            system_overrides: Some(SystemOverrides {
                disable_costing: true,
                disable_limits: true,
                ..Default::default()
            }),
            ..Self::default(network_definition)
        }
    }

    pub fn for_notarized_transaction(network_definition: NetworkDefinition) -> Self {
        Self {
            ..Self::default(network_definition)
        }
    }

    pub fn for_test_transaction() -> Self {
        Self {
            enable_kernel_trace: true,
            enable_cost_breakdown: true,
            ..Self::default(NetworkDefinition::simulator())
        }
    }

    pub fn for_preview(network_definition: NetworkDefinition) -> Self {
        Self {
            enable_cost_breakdown: true,
            execution_trace: Some(MAX_EXECUTION_TRACE_DEPTH),
            ..Self::default(network_definition)
        }
    }

    pub fn for_preview_no_auth(network_definition: NetworkDefinition) -> Self {
        Self {
            system_overrides: Some(SystemOverrides {
                disable_auth: true,
                ..Default::default()
            }),
            enable_cost_breakdown: true,
            execution_trace: Some(MAX_EXECUTION_TRACE_DEPTH),
            ..Self::default(network_definition)
        }
    }

    pub fn with_kernel_trace(mut self, enabled: bool) -> Self {
        self.enable_kernel_trace = enabled;
        self
    }

    pub fn with_cost_breakdown(mut self, enabled: bool) -> Self {
        self.enable_cost_breakdown = enabled;
        self
    }
}

pub struct SubstateBootStore<'a, S: SubstateDatabase> {
    boot_store: &'a S,
}

impl<'a, S: SubstateDatabase> BootStore for SubstateBootStore<'a, S> {
    fn read_boot_substate(
        &self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        substate_key: &SubstateKey,
    ) -> Option<IndexedScryptoValue> {
        let db_partition_key = SpreadPrefixKeyMapper::to_db_partition_key(node_id, partition_num);
        let db_sort_key = SpreadPrefixKeyMapper::to_db_sort_key(&substate_key);
        self.boot_store
            .get_substate(&db_partition_key, &db_sort_key)
            .map(|v| IndexedScryptoValue::from_vec(v.to_vec()).unwrap())
    }
}

pub struct TransactionExecutor<'s, S, V: KernelCallbackObject>
    where
        S: SubstateDatabase,
{
    substate_db: &'s S,
    system_init: V::InitInput,
    phantom: PhantomData<V>,
}

impl<'s, S, V> TransactionExecutor<'s, S, V>
    where
        S: SubstateDatabase,
        V: KernelCallbackObject,
{
    pub fn new(substate_db: &'s S, system_init: V::InitInput) -> Self {
        Self {
            substate_db,
            system_init,
            phantom: PhantomData::default(),
        }
    }

    pub fn execute(
        &mut self,
        executable: &Executable,
    ) -> V::Receipt {

        let kernel_boot = BootLoader {
            id_allocator: IdAllocator::new(executable.intent_hash().to_hash()),
            store: Track::<_, SpreadPrefixKeyMapper>::new(self.substate_db),
            init: self.system_init.clone(),
            phantom: PhantomData::<V>::default()
        };

        kernel_boot.execute(executable)

        // Dump summary
        /*
        #[cfg(not(feature = "alloc"))]
        if self.system_init.enable_kernel_trace {
            Self::print_execution_summary(&receipt);
        }
         */
    }


    #[cfg(not(feature = "alloc"))]
    fn print_execution_summary(receipt: &TransactionReceipt) {
        // NB - we use "to_string" to ensure they align correctly

        if let Some(fee_details) = &receipt.fee_details {
            println!("{:-^120}", "Execution Cost Breakdown");
            for (k, v) in &fee_details.execution_cost_breakdown {
                println!("{:<75}: {:>25}", k, v.to_string());
            }

            println!("{:-^120}", "Finalization Cost Breakdown");
            for (k, v) in &fee_details.finalization_cost_breakdown {
                println!("{:<75}: {:>25}", k, v.to_string());
            }
        }

        println!("{:-^120}", "Fee Summary");
        println!(
            "{:<40}: {:>25}",
            "Execution Cost Units Consumed",
            receipt
                .fee_summary
                .total_execution_cost_units_consumed
                .to_string()
        );
        println!(
            "{:<40}: {:>25}",
            "Finalization Cost Units Consumed",
            receipt
                .fee_summary
                .total_finalization_cost_units_consumed
                .to_string()
        );
        println!(
            "{:<40}: {:>25}",
            "Execution Cost in XRD",
            receipt.fee_summary.total_execution_cost_in_xrd.to_string()
        );
        println!(
            "{:<40}: {:>25}",
            "Finalization Cost in XRD",
            receipt
                .fee_summary
                .total_finalization_cost_in_xrd
                .to_string()
        );
        println!(
            "{:<40}: {:>25}",
            "Tipping Cost in XRD",
            receipt.fee_summary.total_tipping_cost_in_xrd.to_string()
        );
        println!(
            "{:<40}: {:>25}",
            "Storage Cost in XRD",
            receipt.fee_summary.total_storage_cost_in_xrd.to_string()
        );
        println!(
            "{:<40}: {:>25}",
            "Royalty Costs in XRD",
            receipt.fee_summary.total_royalty_cost_in_xrd.to_string()
        );

        match &receipt.result {
            TransactionResult::Commit(commit) => {
                println!("{:-^120}", "Application Logs");
                for (level, message) in &commit.application_logs {
                    println!("[{}] {}", level, message);
                }

                println!("{:-^120}", "Outcome");
                println!(
                    "{}",
                    match &commit.outcome {
                        TransactionOutcome::Success(_) => "Success".to_string(),
                        TransactionOutcome::Failure(error) => format!("Failure: {:?}", error),
                    }
                );
            }
            TransactionResult::Reject(e) => {
                println!("{:-^120}", "Transaction Rejected");
                println!("{:?}", e.reason);
            }
            TransactionResult::Abort(e) => {
                println!("{:-^120}", "Transaction Aborted");
                println!("{:?}", e);
            }
        }
        println!("{:-^120}", "Finish");
    }
}

pub fn execute_transaction_with_configuration<
    S: SubstateDatabase,
    V: SystemCallbackObject,
>(
    substate_db: &S,
    vms: V::InitInput,
    execution_config: &ExecutionConfig,
    transaction: &Executable,
) -> TransactionReceipt {
    let mut executor = TransactionExecutor::<_, System<V>>::new(
        substate_db,
        SystemInit {
            enable_kernel_trace: execution_config.enable_kernel_trace,
            enable_cost_breakdown: execution_config.enable_cost_breakdown,
            execution_trace: execution_config.execution_trace,
            callback_init: vms,
            system_overrides: execution_config.system_overrides.clone(),
        },
    );

    executor.execute(transaction)
}

pub fn execute_transaction<'s, S: SubstateDatabase, W: WasmEngine, E: NativeVmExtension>(
    substate_db: &S,
    vm_init: VmInit<'s, W, E>,
    execution_config: &ExecutionConfig,
    transaction: &Executable,
) -> TransactionReceipt {
    execute_transaction_with_configuration::<S, Vm<'s, W, E>>(
        substate_db,
        vm_init,
        execution_config,
        transaction,
    )
}

pub fn execute_and_commit_transaction<
    's,
    S: SubstateDatabase + CommittableSubstateDatabase,
    W: WasmEngine,
    E: NativeVmExtension,
>(
    substate_db: &mut S,
    vms: VmInit<'s, W, E>,
    execution_config: &ExecutionConfig,
    transaction: &Executable,
) -> TransactionReceipt {
    let receipt = execute_transaction_with_configuration::<S, Vm<'s, W, E>>(
        substate_db,
        vms,
        execution_config,
        transaction,
    );
    if let TransactionResult::Commit(commit) = &receipt.result {
        substate_db.commit(
            &commit
                .state_updates
                .create_database_updates::<SpreadPrefixKeyMapper>(),
        );
    }
    receipt
}

pub enum TransactionResultType {
    Commit(Result<Vec<InstructionOutput>, RuntimeError>),
    Reject(RejectionReason),
    Abort(AbortReason),
}