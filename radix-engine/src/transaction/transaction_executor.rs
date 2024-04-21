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
    #[cfg(all(not(feature = "coverage"), not(feature = "wasm_fuzzing")))]
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
    #[cfg(any(feature = "coverage", feature = "wasm_fuzzing"))]
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

impl<C: SystemCallbackObject> WrappedSystem<C> for System<C> {
    type Init = ();

    fn create(config: System<C>, _: ()) -> Self {
        config
    }

    fn system_mut(&mut self) -> &mut System<C> {
        self
    }

    fn to_system(self) -> System<C> {
        self
    }
}

pub struct SubstateBootStore<'a, S: SubstateDatabase> {
    boot_store: &'a S,
}

impl<'a, S: SubstateDatabase> BootStore for SubstateBootStore<'a, S> {
    fn read_substate(
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

struct TransactionExecutor<'s, S, V: SystemCallbackObject>
where
    S: SubstateDatabase,
{
    substate_db: &'s S,
    system_init: SystemInit<V::InitInput>,
    phantom: PhantomData<V>,
}

impl<'s, S, V> TransactionExecutor<'s, S, V>
where
    S: SubstateDatabase,
    V: SystemCallbackObject,
{
    pub fn new(substate_db: &'s S, system_init: SystemInit<V::InitInput>) -> Self {
        Self {
            substate_db,
            system_init,
            phantom: PhantomData::default(),
        }
    }

    pub fn execute<T: WrappedSystem<V>>(
        &mut self,
        executable: &Executable,
        init: T::Init,
    ) -> TransactionReceipt {
        // Dump executable
        #[cfg(not(feature = "alloc"))]
        if self.system_init.enable_kernel_trace {
            Self::print_executable(&executable);
        }

        // Start hardware resource usage tracker
        #[cfg(all(target_os = "linux", feature = "std", feature = "cpu_ram_metrics"))]
        let mut resources_tracker =
            crate::kernel::resources_tracker::ResourcesTracker::start_measurement();

        let boot_store = SubstateBootStore {
            boot_store: self.substate_db,
        };

        let system_boot_result = System::init(&boot_store, executable, self.system_init.clone());

        // Create a track
        let mut track = Track::<_, SpreadPrefixKeyMapper>::new(self.substate_db);

        let validation_result = match system_boot_result {
            Ok(system) => {
                // Perform runtime validation.
                // TODO: the following assumptions can be removed with better interface.
                // We are assuming that intent hash store is ready when epoch manager is ready.
                let current_epoch = Self::read_epoch(&mut track);
                if let Some(current_epoch) = current_epoch {
                    if let Some(range) = executable.epoch_range() {
                        Self::validate_epoch_range(
                            current_epoch,
                            range.start_epoch_inclusive,
                            range.end_epoch_exclusive,
                        )
                        .and_then(|_| {
                            Self::validate_intent_hash(
                                &mut track,
                                executable.intent_hash().to_hash(),
                                range.end_epoch_exclusive,
                            )
                        })
                        .and_then(|_| Ok(system))
                    } else {
                        Ok(system)
                    }
                } else {
                    Ok(system)
                }
            }
            Err(e) => Err(RejectionReason::BootloadingError(e)),
        };

        // Run manifest
        let (costing_parameters, fee_summary, fee_details, result) = match validation_result {
            Ok(system) => {
                let (
                    interpretation_result,
                    (mut costing_module, runtime_module, execution_trace_module),
                ) = self.interpret_manifest::<T>(&mut track, system, init, executable);

                #[cfg(not(feature = "alloc"))]
                if self.system_init.enable_kernel_trace {
                    println!("{:-^120}", "Interpretation Results");
                    println!("{:?}", interpretation_result);
                }

                let costing_parameters = costing_module.fee_reserve.costing_parameters();

                let fee_details = if let Some(cost_breakdown) = costing_module.cost_breakdown {
                    let execution_cost_breakdown = cost_breakdown
                        .execution_cost_breakdown
                        .into_iter()
                        .map(|(k, v)| (k.to_string(), v))
                        .collect();
                    let finalization_cost_breakdown = cost_breakdown
                        .finalization_cost_breakdown
                        .into_iter()
                        .map(|(k, v)| (k.to_string(), v))
                        .collect();
                    Some(TransactionFeeDetails {
                        execution_cost_breakdown,
                        finalization_cost_breakdown,
                    })
                } else {
                    None
                };

                // Panic if an error is encountered in the system layer or below. The following code
                // is only enabled when compiling with the standard library since the panic catching
                // machinery and `SystemPanic` errors are only implemented in `std`.
                #[cfg(feature = "std")]
                if let Err(TransactionExecutionError::RuntimeError(RuntimeError::SystemError(
                    SystemError::SystemPanic(..),
                ))) = interpretation_result
                {
                    panic!("An error has occurred in the system layer or below and thus the transaction executor has panicked. Error: \"{interpretation_result:?}\"")
                }

                let result_type = Self::determine_result_type(
                    interpretation_result,
                    &mut costing_module.fee_reserve,
                );
                match result_type {
                    TransactionResultType::Commit(outcome) => {
                        let is_success = outcome.is_ok();

                        // Commit/revert
                        if !is_success {
                            costing_module.fee_reserve.revert_royalty();
                            track.revert_non_force_write_changes();
                        }

                        // Distribute fees
                        let (fee_reserve_finalization, paying_vaults, finalization_events) =
                            Self::finalize_fees(
                                &mut track,
                                costing_module.fee_reserve,
                                is_success,
                                executable.costing_parameters().free_credit_in_xrd,
                            );
                        let fee_destination = FeeDestination {
                            to_proposer: fee_reserve_finalization.to_proposer_amount(),
                            to_validator_set: fee_reserve_finalization.to_validator_set_amount(),
                            to_burn: fee_reserve_finalization.to_burn_amount(),
                            to_royalty_recipients: fee_reserve_finalization
                                .royalty_cost_breakdown
                                .clone(),
                        };

                        // Update intent hash status
                        if let Some(next_epoch) = Self::read_epoch(&mut track) {
                            Self::update_transaction_tracker(
                                &mut track,
                                next_epoch,
                                executable.intent_hash(),
                                is_success,
                            );
                        }

                        // Finalize events and logs
                        let (mut application_events, application_logs) =
                            runtime_module.finalize(is_success);
                        application_events.extend(finalization_events);

                        // Finalize execution trace
                        let execution_trace =
                            execution_trace_module.finalize(&paying_vaults, is_success);

                        // Finalize track
                        let tracked_substates = {
                            match track.finalize() {
                                Ok(result) => result,
                                Err(TrackFinalizeError::TransientSubstateOwnsNode) => {
                                    panic!("System invariants should prevent transient substate from owning nodes");
                                }
                            }
                        };

                        // Generate state updates from tracked substates
                        // Note that this process will prune invalid reads
                        let (new_node_ids, state_updates) =
                            to_state_updates::<SpreadPrefixKeyMapper>(tracked_substates);

                        // Summarizes state updates
                        let system_structure = SystemStructure::resolve(
                            self.substate_db,
                            &state_updates,
                            &application_events,
                        );
                        let state_update_summary =
                            StateUpdateSummary::new(self.substate_db, new_node_ids, &state_updates);

                        // Resource reconciliation does not currently work in preview mode
                        if executable.costing_parameters().free_credit_in_xrd.is_zero() {
                            let system_reader = SystemDatabaseReader::new_with_overlay(
                                self.substate_db,
                                &state_updates,
                            );
                            reconcile_resource_state_and_events(
                                &state_update_summary,
                                &application_events,
                                system_reader,
                            );
                        }

                        (
                            costing_parameters,
                            fee_reserve_finalization.into(),
                            fee_details,
                            TransactionResult::Commit(CommitResult {
                                state_updates,
                                state_update_summary,
                                fee_source: FeeSource { paying_vaults },
                                fee_destination,
                                outcome: match outcome {
                                    Ok(o) => TransactionOutcome::Success(o),
                                    Err(e) => TransactionOutcome::Failure(e),
                                },
                                application_events,
                                application_logs,
                                system_structure,
                                execution_trace: if self.system_init.execution_trace.is_some() {
                                    Some(execution_trace)
                                } else {
                                    None
                                },
                            }),
                        )
                    }
                    TransactionResultType::Reject(reason) => (
                        costing_parameters,
                        costing_module.fee_reserve.finalize().into(),
                        fee_details,
                        TransactionResult::Reject(RejectResult { reason }),
                    ),
                    TransactionResultType::Abort(reason) => (
                        costing_parameters,
                        costing_module.fee_reserve.finalize().into(),
                        fee_details,
                        TransactionResult::Abort(AbortResult { reason }),
                    ),
                }
            }
            Err(reason) => (
                // No execution is done, so add empty fee summary and details
                CostingParameters::babylon_genesis(),
                TransactionFeeSummary::default(),
                if self.system_init.enable_cost_breakdown {
                    Some(TransactionFeeDetails::default())
                } else {
                    None
                },
                TransactionResult::Reject(RejectResult { reason }),
            ),
        };

        // Stop hardware resource usage tracker
        let resources_usage = match () {
            #[cfg(not(all(target_os = "linux", feature = "std", feature = "cpu_ram_metrics")))]
            () => None,
            #[cfg(all(target_os = "linux", feature = "std", feature = "cpu_ram_metrics"))]
            () => Some(resources_tracker.end_measurement()),
        };

        // Produce final receipt
        let receipt = TransactionReceipt {
            costing_parameters,
            transaction_costing_parameters: executable.costing_parameters().clone(),
            fee_summary,
            fee_details,
            result,
            resources_usage,
        };

        // Dump summary
        #[cfg(not(feature = "alloc"))]
        if self.system_init.enable_kernel_trace {
            Self::print_execution_summary(&receipt);
        }

        receipt
    }

    fn read_epoch(track: &mut Track<S, SpreadPrefixKeyMapper>) -> Option<Epoch> {
        // TODO - Instead of doing a check of the exact epoch, we could do a check in range [X, Y]
        //        Which could allow for better caching of transaction validity over epoch boundaries
        match track.read_substate(
            CONSENSUS_MANAGER.as_node_id(),
            MAIN_BASE_PARTITION,
            &ConsensusManagerField::State.into(),
        ) {
            Some(x) => {
                let substate: FieldSubstate<ConsensusManagerStateFieldPayload> =
                    x.as_typed().unwrap();
                Some(substate.into_payload().into_latest().epoch)
            }
            None => None,
        }
    }

    fn validate_epoch_range(
        current_epoch: Epoch,
        start_epoch_inclusive: Epoch,
        end_epoch_exclusive: Epoch,
    ) -> Result<(), RejectionReason> {
        if current_epoch < start_epoch_inclusive {
            return Err(RejectionReason::TransactionEpochNotYetValid {
                valid_from: start_epoch_inclusive,
                current_epoch,
            });
        }
        if current_epoch >= end_epoch_exclusive {
            return Err(RejectionReason::TransactionEpochNoLongerValid {
                valid_until: end_epoch_exclusive.previous(),
                current_epoch,
            });
        }

        Ok(())
    }

    fn validate_intent_hash(
        track: &mut Track<S, SpreadPrefixKeyMapper>,
        intent_hash: Hash,
        expiry_epoch: Epoch,
    ) -> Result<(), RejectionReason> {
        let substate: FieldSubstate<TransactionTrackerSubstate> = track
            .read_substate(
                TRANSACTION_TRACKER.as_node_id(),
                MAIN_BASE_PARTITION,
                &TransactionTrackerField::TransactionTracker.into(),
            )
            .unwrap()
            .as_typed()
            .unwrap();

        let partition_number = substate
            .into_payload()
            .v1()
            .partition_for_expiry_epoch(expiry_epoch)
            .expect("Transaction tracker should cover all valid epoch ranges");

        let substate = track.read_substate(
            TRANSACTION_TRACKER.as_node_id(),
            PartitionNumber(partition_number),
            &SubstateKey::Map(scrypto_encode(&intent_hash).unwrap()),
        );

        match substate {
            Some(value) => {
                let substate: KeyValueEntrySubstate<TransactionStatus> = value.as_typed().unwrap();
                match substate.into_value() {
                    Some(status) => match status.into_v1() {
                        TransactionStatusV1::CommittedSuccess
                        | TransactionStatusV1::CommittedFailure => {
                            return Err(RejectionReason::IntentHashPreviouslyCommitted);
                        }
                        TransactionStatusV1::Cancelled => {
                            return Err(RejectionReason::IntentHashPreviouslyCancelled);
                        }
                    },
                    None => {}
                }
            }
            None => {}
        }

        Ok(())
    }

    fn interpret_manifest<T: WrappedSystem<V>>(
        &self,
        track: &mut Track<S, SpreadPrefixKeyMapper>,
        system: System<V>,
        init: T::Init,
        executable: &Executable,
    ) -> (
        Result<Vec<InstructionOutput>, TransactionExecutionError>,
        (
            CostingModule,
            TransactionRuntimeModule,
            ExecutionTraceModule,
        ),
    ) {
        let mut id_allocator = IdAllocator::new(executable.intent_hash().to_hash());

        let mut wrapped_system = T::create(system, init);

        let kernel_boot = BootLoader {
            id_allocator: &mut id_allocator,
            callback: &mut wrapped_system,
            store: track,
        };

        let interpretation_result = kernel_boot
            .execute(
                executable.encoded_instructions(),
                executable.pre_allocated_addresses(),
                executable.references(),
                executable.blobs(),
            )
            .and_then(|x| {
                let system = wrapped_system.system_mut();

                // Note that if a transactions fails during this phase, the costing is
                // done as if it would succeed.

                /* finalization costs: computation on Node side */
                let info = track.get_commit_info();
                for store_commit in &info {
                    system
                        .modules
                        .apply_finalization_cost(FinalizationCostingEntry::CommitStateUpdates {
                            store_commit,
                        })
                        .map_err(|e| {
                            TransactionExecutionError::RuntimeError(
                                RuntimeError::FinalizationCostingError(e),
                            )
                        })?;
                }
                system
                    .modules
                    .apply_finalization_cost(FinalizationCostingEntry::CommitEvents {
                        events: &system.modules.events().clone(),
                    })
                    .map_err(|e| {
                        TransactionExecutionError::RuntimeError(
                            RuntimeError::FinalizationCostingError(e),
                        )
                    })?;
                system
                    .modules
                    .apply_finalization_cost(FinalizationCostingEntry::CommitLogs {
                        logs: &system.modules.logs().clone(),
                    })
                    .map_err(|e| {
                        TransactionExecutionError::RuntimeError(
                            RuntimeError::FinalizationCostingError(e),
                        )
                    })?;

                /* state storage costs */
                for store_commit in &info {
                    system
                        .modules
                        .apply_storage_cost(StorageType::State, store_commit.len_increase())
                        .map_err(|e| {
                            TransactionExecutionError::RuntimeError(
                                RuntimeError::FinalizationCostingError(e),
                            )
                        })?;
                }

                /* archive storage costs */
                let total_event_size = system.modules.events().iter().map(|x| x.len()).sum();
                system
                    .modules
                    .apply_storage_cost(StorageType::Archive, total_event_size)
                    .map_err(|e| {
                        TransactionExecutionError::RuntimeError(
                            RuntimeError::FinalizationCostingError(e),
                        )
                    })?;

                let total_log_size = system.modules.logs().iter().map(|x| x.1.len()).sum();
                system
                    .modules
                    .apply_storage_cost(StorageType::Archive, total_log_size)
                    .map_err(|e| {
                        TransactionExecutionError::RuntimeError(
                            RuntimeError::FinalizationCostingError(e),
                        )
                    })?;

                Ok(x)
            })
            .or_else(|e| {
                // State updates are reverted

                // Events are reverted

                // Logs are NOT reverted (This is not ideal, as it means logs are free if the transaction fails)

                Err(e)
            })
            .map(|rtn| {
                let output: Vec<InstructionOutput> = scrypto_decode(&rtn).unwrap();
                output
            });

        let system = wrapped_system.to_system();
        (interpretation_result, system.modules.unpack())
    }

    fn determine_result_type(
        interpretation_result: Result<Vec<InstructionOutput>, TransactionExecutionError>,
        fee_reserve: &mut SystemLoanFeeReserve,
    ) -> TransactionResultType {
        // A `SuccessButFeeLoanNotRepaid` error is issued if a transaction finishes before
        // the SYSTEM_LOAN_AMOUNT is reached (which trigger a repay event) and even though
        // enough fee has been locked.
        //
        // Do another `repay` try during finalization to remedy it.
        let final_repay_result = fee_reserve.repay_all();

        match interpretation_result {
            Ok(output) => match final_repay_result {
                Ok(_) => TransactionResultType::Commit(Ok(output)), // success and system loan repaid fully
                Err(e) => {
                    if let Some(abort_reason) = e.abortion() {
                        TransactionResultType::Abort(abort_reason.clone())
                    } else {
                        TransactionResultType::Reject(RejectionReason::SuccessButFeeLoanNotRepaid)
                    }
                }
            },
            Err(e) => match e {
                TransactionExecutionError::BootloadingError(e) => {
                    TransactionResultType::Reject(RejectionReason::BootloadingError(e))
                }
                TransactionExecutionError::RuntimeError(e) => {
                    if let Some(abort_reason) = e.abortion() {
                        TransactionResultType::Abort(abort_reason.clone())
                    } else {
                        if fee_reserve.fully_repaid() {
                            TransactionResultType::Commit(Err(e))
                        } else {
                            TransactionResultType::Reject(
                                RejectionReason::ErrorBeforeLoanAndDeferredCostsRepaid(e),
                            )
                        }
                    }
                }
            },
        }
    }

    fn finalize_fees(
        track: &mut Track<S, SpreadPrefixKeyMapper>,
        fee_reserve: SystemLoanFeeReserve,
        is_success: bool,
        free_credit: Decimal,
    ) -> (
        FeeReserveFinalizationSummary,
        IndexMap<NodeId, Decimal>,
        Vec<(EventTypeIdentifier, Vec<u8>)>,
    ) {
        let mut events = Vec::<(EventTypeIdentifier, Vec<u8>)>::new();

        // Distribute royalty
        for (recipient, amount) in fee_reserve.royalty_cost_breakdown().clone() {
            let node_id = recipient.vault_id();
            let substate_key = FungibleVaultField::Balance.into();
            let mut vault_balance = track
                .read_substate(&node_id, MAIN_BASE_PARTITION, &substate_key)
                .unwrap()
                .as_typed::<FungibleVaultBalanceFieldSubstate>()
                .unwrap()
                .into_payload()
                .into_latest();
            vault_balance.put(LiquidFungibleResource::new(amount));
            let updated_substate_content =
                FungibleVaultBalanceFieldPayload::from_content_source(vault_balance)
                    .into_unlocked_substate();
            track
                .set_substate(
                    node_id,
                    MAIN_BASE_PARTITION,
                    substate_key,
                    IndexedScryptoValue::from_typed(&updated_substate_content),
                    &mut |_| -> Result<(), ()> { Ok(()) },
                )
                .unwrap();
            events.push((
                EventTypeIdentifier(
                    Emitter::Method(node_id, ModuleId::Main),
                    DepositEvent::EVENT_NAME.to_string(),
                ),
                scrypto_encode(&DepositEvent { amount }).unwrap(),
            ));
        }

        // Take fee payments
        let fee_reserve_finalization = fee_reserve.finalize();
        let mut fee_payments: IndexMap<NodeId, Decimal> = index_map_new();
        let mut required = fee_reserve_finalization.total_cost();
        let mut collected_fees = LiquidFungibleResource::new(Decimal::ZERO);
        for (vault_id, mut locked, contingent) in
            fee_reserve_finalization.locked_fees.iter().cloned().rev()
        {
            let amount = if contingent {
                if is_success {
                    Decimal::min(locked.amount(), required)
                } else {
                    Decimal::zero()
                }
            } else {
                Decimal::min(locked.amount(), required)
            };

            // NOTE: Decimal arithmetic operation safe unwrap.
            // No chance to overflow considering current costing parameters

            // Take fees
            collected_fees.put(locked.take_by_amount(amount).unwrap());
            required = required.checked_sub(amount).unwrap();

            // Refund overpayment
            let mut vault_balance = track
                .read_substate(
                    &vault_id,
                    MAIN_BASE_PARTITION,
                    &FungibleVaultField::Balance.into(),
                )
                .unwrap()
                .as_typed::<FungibleVaultBalanceFieldSubstate>()
                .unwrap()
                .into_payload()
                .into_latest();
            vault_balance.put(locked);
            let updated_substate_content =
                FungibleVaultBalanceFieldPayload::from_content_source(vault_balance)
                    .into_unlocked_substate();
            track
                .set_substate(
                    vault_id,
                    MAIN_BASE_PARTITION,
                    FungibleVaultField::Balance.into(),
                    IndexedScryptoValue::from_typed(&updated_substate_content),
                    &mut |_| -> Result<(), ()> { Ok(()) },
                )
                .unwrap();

            // Record final payments
            let entry = fee_payments.entry(vault_id).or_default();
            *entry = entry.checked_add(amount).unwrap();

            events.push((
                EventTypeIdentifier(
                    Emitter::Method(vault_id, ModuleId::Main),
                    PayFeeEvent::EVENT_NAME.to_string(),
                ),
                scrypto_encode(&PayFeeEvent { amount }).unwrap(),
            ));
        }
        // Free credit is locked first and thus used last
        if free_credit.is_positive() {
            let amount = Decimal::min(free_credit, required);
            collected_fees.put(LiquidFungibleResource::new(amount));
            required = required.checked_sub(amount).unwrap();
        }

        let to_proposer = fee_reserve_finalization.to_proposer_amount();
        let to_validator_set = fee_reserve_finalization.to_validator_set_amount();
        let to_burn = fee_reserve_finalization.to_burn_amount();

        // Sanity checks
        assert!(
            fee_reserve_finalization.total_bad_debt_in_xrd == Decimal::ZERO,
            "Bad debt is non-zero: {}",
            fee_reserve_finalization.total_bad_debt_in_xrd
        );
        assert!(
            required == Decimal::ZERO,
            "Locked fee does not cover transaction cost: {} required",
            required
        );
        let remaining_collected_fees = collected_fees.amount().checked_sub(fee_reserve_finalization.total_royalty_cost_in_xrd /* royalty already distributed */).unwrap();
        let to_distribute = to_proposer
            .checked_add(to_validator_set)
            .unwrap()
            .checked_add(to_burn)
            .unwrap();
        assert!(
            remaining_collected_fees  == to_distribute,
            "Remaining collected fee isn't equal to amount to distribute (proposer/validator set/burn): {} != {}",
            remaining_collected_fees,
            to_distribute,
        );

        if !to_proposer.is_zero() || !to_validator_set.is_zero() {
            // Fetch current leader
            // TODO: maybe we should move current leader into validator rewards?
            let substate: FieldSubstate<ConsensusManagerStateFieldPayload> = track
                .read_substate(
                    CONSENSUS_MANAGER.as_node_id(),
                    MAIN_BASE_PARTITION,
                    &ConsensusManagerField::State.into(),
                )
                .unwrap()
                .as_typed()
                .unwrap();
            let current_leader = substate.into_payload().into_latest().current_leader;

            // Update validator rewards
            let substate: FieldSubstate<ConsensusManagerValidatorRewardsFieldPayload> = track
                .read_substate(
                    CONSENSUS_MANAGER.as_node_id(),
                    MAIN_BASE_PARTITION,
                    &ConsensusManagerField::ValidatorRewards.into(),
                )
                .unwrap()
                .as_typed()
                .unwrap();

            let mut rewards = substate.into_payload().into_latest();

            if let Some(current_leader) = current_leader {
                let entry = rewards.proposer_rewards.entry(current_leader).or_default();
                *entry = entry.checked_add(to_proposer).unwrap()
            } else {
                // If there is no current leader, the rewards go to the pool
            };
            let vault_node_id = rewards.rewards_vault.0 .0;

            track
                .set_substate(
                    CONSENSUS_MANAGER.into_node_id(),
                    MAIN_BASE_PARTITION,
                    ConsensusManagerField::ValidatorRewards.into(),
                    IndexedScryptoValue::from_typed(&FieldSubstate::new_unlocked_field(
                        ConsensusManagerValidatorRewardsFieldPayload::from_content_source(rewards),
                    )),
                    &mut |_| -> Result<(), ()> { Ok(()) },
                )
                .unwrap();

            // Put validator rewards into the vault
            let total_amount = to_proposer.checked_add(to_validator_set).unwrap();
            let mut vault_balance = track
                .read_substate(
                    &vault_node_id,
                    MAIN_BASE_PARTITION,
                    &FungibleVaultField::Balance.into(),
                )
                .unwrap()
                .as_typed::<FungibleVaultBalanceFieldSubstate>()
                .unwrap()
                .into_payload()
                .into_latest();
            vault_balance.put(collected_fees.take_by_amount(total_amount).unwrap());
            let updated_substate_content =
                FungibleVaultBalanceFieldPayload::from_content_source(vault_balance)
                    .into_unlocked_substate();
            track
                .set_substate(
                    vault_node_id,
                    MAIN_BASE_PARTITION,
                    FungibleVaultField::Balance.into(),
                    IndexedScryptoValue::from_typed(&updated_substate_content),
                    &mut |_| -> Result<(), ()> { Ok(()) },
                )
                .unwrap();

            events.push((
                EventTypeIdentifier(
                    Emitter::Method(vault_node_id, ModuleId::Main),
                    DepositEvent::EVENT_NAME.to_string(),
                ),
                scrypto_encode(&DepositEvent {
                    amount: total_amount,
                })
                .unwrap(),
            ));
        }

        if to_burn.is_positive() {
            events.push((
                EventTypeIdentifier(
                    Emitter::Method(XRD.into_node_id(), ModuleId::Main),
                    "BurnFungibleResourceEvent".to_string(),
                ),
                scrypto_encode(&BurnFungibleResourceEvent { amount: to_burn }).unwrap(),
            ));
        }

        (fee_reserve_finalization, fee_payments, events)
    }

    fn update_transaction_tracker(
        track: &mut Track<S, SpreadPrefixKeyMapper>,
        next_epoch: Epoch,
        intent_hash: &TransactionIntentHash,
        is_success: bool,
    ) {
        // Read the intent hash store
        let transaction_tracker = track
            .read_substate(
                TRANSACTION_TRACKER.as_node_id(),
                MAIN_BASE_PARTITION,
                &TransactionTrackerField::TransactionTracker.into(),
            )
            .unwrap()
            .as_typed::<FieldSubstate<TransactionTrackerSubstate>>()
            .unwrap()
            .into_payload();

        let mut transaction_tracker = transaction_tracker.into_v1();

        // Update the status of the intent hash
        if let TransactionIntentHash::ToCheck {
            expiry_epoch,
            intent_hash,
        } = intent_hash
        {
            if let Some(partition_number) =
                transaction_tracker.partition_for_expiry_epoch(*expiry_epoch)
            {
                track
                    .set_substate(
                        TRANSACTION_TRACKER.into_node_id(),
                        PartitionNumber(partition_number),
                        SubstateKey::Map(scrypto_encode(intent_hash).unwrap()),
                        IndexedScryptoValue::from_typed(&KeyValueEntrySubstate::V1(
                            KeyValueEntrySubstateV1 {
                                value: Some(if is_success {
                                    TransactionStatus::V1(TransactionStatusV1::CommittedSuccess)
                                } else {
                                    TransactionStatus::V1(TransactionStatusV1::CommittedFailure)
                                }),
                                // TODO: maybe make it immutable, but how does this affect partition deletion?
                                lock_status: LockStatus::Unlocked,
                            },
                        )),
                        &mut |_| -> Result<(), ()> { Ok(()) },
                    )
                    .unwrap();
            } else {
                panic!("No partition for an expiry epoch")
            }
        }

        // Check if all intent hashes in the first epoch have expired, based on the `next_epoch`.
        //
        // In this particular implementation, because the transaction tracker coverage is greater than
        // the max epoch range in transaction header, we must check epoch range first to
        // ensure we don't store intent hash too far into the future.
        //
        // Also, we need to make sure epoch doesn't jump by a large distance.
        if next_epoch.number()
            >= transaction_tracker.start_epoch + transaction_tracker.epochs_per_partition
        {
            let discarded_partition = transaction_tracker.advance();
            track.delete_partition(
                TRANSACTION_TRACKER.as_node_id(),
                PartitionNumber(discarded_partition),
            );
        }
        track
            .set_substate(
                TRANSACTION_TRACKER.into_node_id(),
                MAIN_BASE_PARTITION,
                TransactionTrackerField::TransactionTracker.into(),
                IndexedScryptoValue::from_typed(&FieldSubstate::new_unlocked_field(
                    TransactionTrackerSubstate::V1(transaction_tracker),
                )),
                &mut |_| -> Result<(), ()> { Ok(()) },
            )
            .unwrap();
    }

    #[cfg(not(feature = "alloc"))]
    fn print_executable(executable: &Executable) {
        println!("{:-^120}", "Executable");
        println!("Intent hash: {}", executable.intent_hash().as_hash());
        println!("Payload size: {}", executable.payload_size());
        println!(
            "Transaction costing parameters: {:?}",
            executable.costing_parameters()
        );
        println!(
            "Pre-allocated addresses: {:?}",
            executable.pre_allocated_addresses()
        );
        println!("Blobs: {:?}", executable.blobs().keys());
        println!("References: {:?}", executable.references());
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
    T: WrappedSystem<V>,
>(
    substate_db: &S,
    vms: V::InitInput,
    execution_config: &ExecutionConfig,
    transaction: &Executable,
    init: T::Init,
) -> TransactionReceipt {
    let mut executor = TransactionExecutor::new(
        substate_db,
        SystemInit {
            enable_kernel_trace: execution_config.enable_kernel_trace,
            enable_cost_breakdown: execution_config.enable_cost_breakdown,
            execution_trace: execution_config.execution_trace,
            callback_init: vms,
            system_overrides: execution_config.system_overrides.clone(),
        },
    );

    executor.execute::<T>(transaction, init)
}

pub fn execute_transaction<'s, S: SubstateDatabase, W: WasmEngine, E: NativeVmExtension>(
    substate_db: &S,
    vm_init: VmInit<'s, W, E>,
    execution_config: &ExecutionConfig,
    transaction: &Executable,
) -> TransactionReceipt {
    execute_transaction_with_configuration::<S, Vm<'s, W, E>, System<Vm<'s, W, E>>>(
        substate_db,
        vm_init,
        execution_config,
        transaction,
        (),
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
    let receipt = execute_transaction_with_configuration::<S, Vm<'s, W, E>, System<Vm<'s, W, E>>>(
        substate_db,
        vms,
        execution_config,
        transaction,
        (),
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

enum TransactionResultType {
    Commit(Result<Vec<InstructionOutput>, RuntimeError>),
    Reject(RejectionReason),
    Abort(AbortReason),
}

pub trait WrappedSystem<C: SystemCallbackObject>: KernelCallbackObject {
    type Init;

    fn create(config: System<C>, init: Self::Init) -> Self;
    fn system_mut(&mut self) -> &mut System<C>;
    fn to_system(self) -> System<C>;
}
