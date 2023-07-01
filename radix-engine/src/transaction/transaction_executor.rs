use crate::blueprints::consensus_manager::{ConsensusManagerSubstate, ValidatorRewardsSubstate};
use crate::blueprints::transaction_processor::TransactionProcessorError;
use crate::blueprints::transaction_tracker::{TransactionStatus, TransactionTrackerSubstate};
use crate::errors::*;
use crate::kernel::id_allocator::IdAllocator;
use crate::kernel::kernel::KernelBoot;
use crate::system::system::{KeyValueEntrySubstate, SubstateMutability};
use crate::system::system_callback::SystemConfig;
use crate::system::system_modules::costing::*;
use crate::system::system_modules::execution_trace::ExecutionTraceModule;
use crate::system::system_modules::transaction_runtime::TransactionRuntimeModule;
use crate::system::system_modules::{EnabledModules, SystemModuleMixer};
use crate::track::interface::SubstateStore;
use crate::track::{to_state_updates, Track};
use crate::transaction::*;
use crate::types::*;
use crate::vm::wasm::*;
use crate::vm::{ScryptoVm, Vm};
use radix_engine_constants::*;
use radix_engine_interface::api::LockFlags;
use radix_engine_interface::blueprints::resource::LiquidFungibleResource;
use radix_engine_interface::blueprints::transaction_processor::InstructionOutput;
use radix_engine_store_interface::{db_key_mapper::SpreadPrefixKeyMapper, interface::*};
use transaction::model::*;

#[derive(Debug, Clone)]
pub struct FeeReserveConfig {
    pub cost_unit_price: Decimal,
    pub usd_price: Decimal,
    pub state_expansion_price: Decimal,
    pub system_loan: u32,
}

impl Default for FeeReserveConfig {
    fn default() -> Self {
        Self {
            cost_unit_price: DEFAULT_COST_UNIT_PRICE_IN_XRD.try_into().unwrap(),
            usd_price: DEFAULT_USD_PRICE_IN_XRD.try_into().unwrap(),
            state_expansion_price: DEFAULT_STATE_EXPANSION_PRICE_IN_XRD.try_into().unwrap(),
            system_loan: DEFAULT_SYSTEM_LOAN,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExecutionConfig {
    pub enabled_modules: EnabledModules,
    pub max_execution_trace_depth: usize,
    pub max_call_depth: usize,
    pub cost_unit_limit: u32,
    pub abort_when_loan_repaid: bool,
    pub max_number_of_substates_in_track: usize,
    pub max_number_of_substates_in_heap: usize,
    pub max_substate_size: usize,
    pub max_invoke_input_size: usize,
    pub enable_cost_breakdown: bool,
    pub max_event_size: usize,
    pub max_log_size: usize,
    pub max_panic_message_size: usize,
    pub max_number_of_logs: usize,
    pub max_number_of_events: usize,
    pub max_per_function_royalty_in_xrd: Decimal,
}

impl ExecutionConfig {
    /// Creates an `ExecutionConfig` using default configurations.
    /// This is internal. Clients should use `for_xxx` constructors instead.
    fn default() -> Self {
        Self {
            enabled_modules: EnabledModules::for_notarized_transaction(),
            max_execution_trace_depth: DEFAULT_MAX_EXECUTION_TRACE_DEPTH,
            max_call_depth: DEFAULT_MAX_CALL_DEPTH,
            cost_unit_limit: DEFAULT_COST_UNIT_LIMIT,
            abort_when_loan_repaid: false,
            max_number_of_substates_in_track: DEFAULT_MAX_NUMBER_OF_SUBSTATES_IN_TRACK,
            max_number_of_substates_in_heap: DEFAULT_MAX_NUMBER_OF_SUBSTATES_IN_HEAP,
            max_substate_size: DEFAULT_MAX_SUBSTATE_SIZE,
            max_invoke_input_size: DEFAULT_MAX_INVOKE_INPUT_SIZE,
            enable_cost_breakdown: false,
            max_event_size: DEFAULT_MAX_EVENT_SIZE,
            max_log_size: DEFAULT_MAX_LOG_SIZE,
            max_panic_message_size: DEFAULT_MAX_PANIC_MESSAGE_SIZE,
            max_number_of_logs: DEFAULT_MAX_NUMBER_OF_LOGS,
            max_number_of_events: DEFAULT_MAX_NUMBER_OF_EVENTS,
            max_per_function_royalty_in_xrd: Decimal::try_from(
                DEFAULT_MAX_PER_FUNCTION_ROYALTY_IN_XRD,
            )
            .unwrap(),
        }
    }

    pub fn for_genesis_transaction() -> Self {
        Self {
            enabled_modules: EnabledModules::for_genesis_transaction(),
            max_number_of_substates_in_track: 50_000,
            max_number_of_substates_in_heap: 50_000,
            max_number_of_events: 1_000_000,
            ..Self::default()
        }
    }

    pub fn for_system_transaction() -> Self {
        Self {
            enabled_modules: EnabledModules::for_system_transaction(),
            ..Self::default()
        }
    }

    pub fn for_notarized_transaction() -> Self {
        Self {
            enabled_modules: EnabledModules::for_notarized_transaction(),
            ..Self::default()
        }
    }

    pub fn for_test_transaction() -> Self {
        Self {
            enabled_modules: EnabledModules::for_test_transaction(),
            enable_cost_breakdown: true,
            ..Self::default()
        }
    }

    pub fn for_preview() -> Self {
        Self {
            enabled_modules: EnabledModules::for_preview(),
            ..Self::default()
        }
    }

    pub fn with_kernel_trace(mut self, enabled: bool) -> Self {
        if enabled {
            self.enabled_modules.insert(EnabledModules::KERNEL_TRACE);
        } else {
            self.enabled_modules.remove(EnabledModules::KERNEL_TRACE);
        }
        self
    }

    pub fn with_cost_unit_limit(mut self, cost_unit_limit: u32) -> Self {
        self.cost_unit_limit = cost_unit_limit;
        self
    }

    pub fn up_to_loan_repayment(mut self, enabled: bool) -> Self {
        self.abort_when_loan_repaid = enabled;
        self
    }
}

/// An executor that runs transactions.
/// This is no longer public -- it can be removed / merged into the exposed functions in a future small PR
/// But I'm not doing it in this PR to avoid merge conflicts in the body of execute_with_fee_reserve
struct TransactionExecutor<'s, 'w, S, W>
where
    S: SubstateDatabase,
    W: WasmEngine,
{
    substate_db: &'s S,
    scrypto_vm: &'w ScryptoVm<W>,
}

impl<'s, 'w, S, W> TransactionExecutor<'s, 'w, S, W>
where
    S: SubstateDatabase,
    W: WasmEngine,
{
    pub fn new(substate_db: &'s S, scrypto_vm: &'w ScryptoVm<W>) -> Self {
        Self {
            substate_db,
            scrypto_vm,
        }
    }

    pub fn execute(
        &mut self,
        transaction: &Executable,
        fee_reserve_config: &FeeReserveConfig,
        execution_config: &ExecutionConfig,
    ) -> TransactionReceipt {
        let fee_reserve = SystemLoanFeeReserve::new(
            fee_reserve_config.cost_unit_price,
            fee_reserve_config.usd_price,
            fee_reserve_config.state_expansion_price,
            transaction.fee_payment().tip_percentage,
            execution_config.cost_unit_limit,
            fee_reserve_config.system_loan,
            execution_config.abort_when_loan_repaid,
        )
        .with_free_credit(transaction.fee_payment().free_credit_in_xrd);

        self.execute_with_fee_reserve(transaction, execution_config, fee_reserve, FeeTable::new())
    }

    fn execute_with_fee_reserve(
        &mut self,
        executable: &Executable,
        execution_config: &ExecutionConfig,
        fee_reserve: SystemLoanFeeReserve,
        fee_table: FeeTable,
    ) -> TransactionReceipt {
        // Dump executable
        #[cfg(not(feature = "alloc"))]
        if execution_config
            .enabled_modules
            .contains(EnabledModules::KERNEL_TRACE)
        {
            Self::print_executable(&executable);
        }

        // Start hardware resource usage tracker
        #[cfg(all(target_os = "linux", feature = "std", feature = "cpu_ram_metrics"))]
        let mut resources_tracker =
            crate::kernel::resources_tracker::ResourcesTracker::start_measurement();

        // Create a track
        let mut track = Track::<_, SpreadPrefixKeyMapper>::new(self.substate_db);

        // Perform runtime validation.
        // TODO: the following assumptions can be removed with better interface.
        // We are assuming that intent hash store is ready when epoch manager is ready.
        let current_epoch = Self::read_epoch(&mut track);
        let validation_result = if let Some(current_epoch) = current_epoch {
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
            } else {
                Ok(())
            }
        } else {
            Ok(())
        };

        // Run manifest
        let result = match validation_result {
            Ok(()) => {
                let (
                    interpretation_result,
                    (mut costing_module, runtime_module, execution_trace_module),
                ) = self.interpret_manifest(
                    &mut track,
                    executable,
                    execution_config,
                    fee_reserve,
                    fee_table,
                );

                #[cfg(not(feature = "alloc"))]
                if execution_config
                    .enabled_modules
                    .contains(EnabledModules::KERNEL_TRACE)
                {
                    println!("{:-^100}", "Interpretation Results");
                    println!("{:?}", interpretation_result);
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
                        let (mut fee_summary, fee_payments) =
                            Self::finalize_fees(&mut track, costing_module.fee_reserve, is_success);
                        fee_summary.execution_cost_breakdown = costing_module
                            .costing_traces
                            .into_iter()
                            .map(|(k, v)| (k.to_string(), v))
                            .collect();
                        fee_summary.fee_payments = fee_payments.clone();

                        // Update intent hash status
                        if let Some(next_epoch) = Self::read_epoch(&mut track) {
                            Self::update_transaction_tracker(
                                &mut track,
                                next_epoch,
                                executable.intent_hash(),
                                is_success,
                            );
                        }

                        // Finalize everything
                        let (application_events, application_logs) =
                            runtime_module.finalize(is_success);
                        let execution_trace =
                            execution_trace_module.finalize(&fee_payments, is_success);
                        let (tracked_nodes, deleted_partitions) = track.finalize();
                        let state_update_summary =
                            StateUpdateSummary::new(self.substate_db, &tracked_nodes);
                        let state_updates = to_state_updates::<SpreadPrefixKeyMapper>(
                            tracked_nodes,
                            deleted_partitions,
                        );

                        TransactionResult::Commit(CommitResult {
                            state_updates,
                            state_update_summary,
                            outcome: match outcome {
                                Ok(o) => TransactionOutcome::Success(o),
                                Err(e) => TransactionOutcome::Failure(e),
                            },
                            fee_summary,
                            application_events,
                            application_logs,
                            execution_trace,
                        })
                    }
                    TransactionResultType::Reject(error) => {
                        TransactionResult::Reject(RejectResult { error })
                    }
                    TransactionResultType::Abort(error) => {
                        TransactionResult::Abort(AbortResult { reason: error })
                    }
                }
            }
            Err(error) => TransactionResult::Reject(RejectResult { error }),
        };

        // Stop hardware resource usage tracker
        let resources_usage = match () {
            #[cfg(not(all(target_os = "linux", feature = "std", feature = "cpu_ram_metrics")))]
            () => ResourcesUsage::default(),
            #[cfg(all(target_os = "linux", feature = "std", feature = "cpu_ram_metrics"))]
            () => resources_tracker.end_measurement(),
        };

        // Produce final receipt
        let receipt = TransactionReceipt {
            transaction_result: result,
            resources_usage,
        };

        // Dump summary
        #[cfg(not(feature = "alloc"))]
        if execution_config
            .enabled_modules
            .contains(EnabledModules::KERNEL_TRACE)
        {
            Self::print_execution_summary(&receipt);
        }

        receipt
    }

    fn read_epoch(track: &mut Track<S, SpreadPrefixKeyMapper>) -> Option<Epoch> {
        // TODO - Instead of doing a check of the exact epoch, we could do a check in range [X, Y]
        //        Which could allow for better caching of transaction validity over epoch boundaries
        let handle = match track.acquire_lock(
            CONSENSUS_MANAGER.as_node_id(),
            MAIN_BASE_PARTITION,
            &ConsensusManagerField::ConsensusManager.into(),
            LockFlags::read_only(),
        ) {
            Ok(x) => x.0,
            Err(_) => {
                return None;
            }
        };
        let substate: ConsensusManagerSubstate = track.read_substate(handle).0.as_typed().unwrap();
        track.close_substate(handle);
        Some(substate.epoch)
    }

    fn validate_epoch_range(
        current_epoch: Epoch,
        start_epoch_inclusive: Epoch,
        end_epoch_exclusive: Epoch,
    ) -> Result<(), RejectionError> {
        if current_epoch < start_epoch_inclusive {
            return Err(RejectionError::TransactionEpochNotYetValid {
                valid_from: start_epoch_inclusive,
                current_epoch,
            });
        }
        if current_epoch >= end_epoch_exclusive {
            return Err(RejectionError::TransactionEpochNoLongerValid {
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
    ) -> Result<(), RejectionError> {
        let handle = track
            .acquire_lock(
                TRANSACTION_TRACKER.as_node_id(),
                MAIN_BASE_PARTITION,
                &TransactionTrackerField::TransactionTracker.into(),
                LockFlags::read_only(),
            )
            .unwrap()
            .0;
        let substate: TransactionTrackerSubstate =
            track.read_substate(handle).0.as_typed().unwrap();
        track.close_substate(handle);

        let partition_number = substate
            .partition_for_expiry_epoch(expiry_epoch)
            .expect("Transaction tracker should cover all valid epoch ranges");

        let handle = track
            .acquire_lock_virtualize(
                TRANSACTION_TRACKER.as_node_id(),
                PartitionNumber(partition_number),
                &SubstateKey::Map(intent_hash.to_vec()),
                LockFlags::read_only(),
                || {
                    Some(IndexedScryptoValue::from_typed(&KeyValueEntrySubstate {
                        value: Option::<TransactionStatus>::None,
                        mutability: SubstateMutability::Mutable,
                    }))
                },
            )
            .unwrap()
            .0;
        let substate: KeyValueEntrySubstate<TransactionStatus> =
            track.read_substate(handle).0.as_typed().unwrap();
        track.close_substate(handle);

        match substate.value {
            Some(status) => match status {
                TransactionStatus::CommittedSuccess | TransactionStatus::CommittedFailure => {
                    return Err(RejectionError::IntentHashPreviouslyCommitted);
                }
                TransactionStatus::Cancelled => {
                    return Err(RejectionError::IntentHashPreviouslyCancelled);
                }
            },
            None => {}
        }

        Ok(())
    }

    fn interpret_manifest(
        &self,
        track: &mut Track<S, SpreadPrefixKeyMapper>,
        executable: &Executable,
        execution_config: &ExecutionConfig,
        fee_reserve: SystemLoanFeeReserve,
        fee_table: FeeTable,
    ) -> (
        Result<Vec<InstructionOutput>, RuntimeError>,
        (
            CostingModule,
            TransactionRuntimeModule,
            ExecutionTraceModule,
        ),
    ) {
        let mut id_allocator = IdAllocator::new(executable.intent_hash().to_hash());
        let mut system = SystemConfig {
            blueprint_cache: NonIterMap::new(),
            auth_cache: NonIterMap::new(),
            schema_cache: NonIterMap::new(),
            callback_obj: Vm {
                scrypto_vm: self.scrypto_vm,
            },
            modules: SystemModuleMixer::new(
                execution_config.enabled_modules,
                executable.intent_hash().to_hash(),
                executable.auth_zone_params().clone(),
                fee_reserve,
                fee_table,
                executable.payload_size(),
                executable.auth_zone_params().initial_proofs.len(),
                execution_config,
            ),
        };

        let kernel_boot = KernelBoot {
            id_allocator: &mut id_allocator,
            callback: &mut system,
            store: track,
        };

        let interpretation_result = kernel_boot
            .call_transaction_processor(
                executable.encoded_instructions(),
                executable.pre_allocated_addresses(),
                executable.references(),
                executable.blobs(),
            )
            .and_then(|x| {
                let info = track.get_commit_info();
                for commit in &info {
                    if let Err(e) = system.modules.apply_execution_cost(CostingEntry::Commit {
                        store_commit: commit,
                    }) {
                        return Err(e);
                    }
                }
                for commit in &info {
                    if let Err(e) = system.modules.apply_state_expansion_cost(commit) {
                        return Err(e);
                    }
                }

                Ok(x)
            })
            .map(|rtn| {
                let output: Vec<InstructionOutput> = scrypto_decode(&rtn).unwrap();
                output
            });

        (interpretation_result, system.modules.unpack())
    }

    fn determine_result_type(
        mut interpretation_result: Result<Vec<InstructionOutput>, RuntimeError>,
        fee_reserve: &mut SystemLoanFeeReserve,
    ) -> TransactionResultType {
        // A `SuccessButFeeLoanNotRepaid` error is issued if a transaction finishes before
        // the SYSTEM_LOAN_AMOUNT is reached (which trigger a repay event) and even though
        // enough fee has been locked.
        //
        // Do another `repay` try during finalization to remedy it.
        if let Err(err) = fee_reserve.repay_all() {
            if interpretation_result.is_ok() {
                interpretation_result = Err(RuntimeError::SystemModuleError(
                    SystemModuleError::CostingError(CostingError::FeeReserveError(err)),
                ));
            }
        }

        // First - check for required rejections from explicit invoke result errors
        match &interpretation_result {
            Err(RuntimeError::ApplicationError(ApplicationError::TransactionProcessorError(
                err,
            ))) => match err {
                TransactionProcessorError::TransactionEpochNotYetValid {
                    valid_from,
                    current_epoch,
                } => {
                    return TransactionResultType::Reject(
                        RejectionError::TransactionEpochNotYetValid {
                            valid_from: *valid_from,
                            current_epoch: *current_epoch,
                        },
                    )
                }
                TransactionProcessorError::TransactionEpochNoLongerValid {
                    valid_until,
                    current_epoch,
                } => {
                    return TransactionResultType::Reject(
                        RejectionError::TransactionEpochNoLongerValid {
                            valid_until: *valid_until,
                            current_epoch: *current_epoch,
                        },
                    )
                }
                _ => {}
            },
            Err(err) => {
                if let Some(abort_reason) = err.abortion() {
                    return TransactionResultType::Abort(abort_reason.clone());
                }
            }
            _ => {}
        }

        // Check for errors before loan is repaid - in which case, we also reject
        if !fee_reserve.fully_repaid() {
            return match interpretation_result {
                Ok(..) => TransactionResultType::Reject(RejectionError::SuccessButFeeLoanNotRepaid),
                Err(error) => {
                    TransactionResultType::Reject(RejectionError::ErrorBeforeFeeLoanRepaid(error))
                }
            };
        }

        TransactionResultType::Commit(interpretation_result)
    }

    fn finalize_fees(
        track: &mut Track<S, SpreadPrefixKeyMapper>,
        fee_reserve: SystemLoanFeeReserve,
        is_success: bool,
    ) -> (FeeSummary, IndexMap<NodeId, Decimal>) {
        // Distribute royalty
        for (_, (recipient_vault_id, amount)) in fee_reserve.royalty_cost() {
            let node_id = recipient_vault_id;
            let substate_key = FungibleVaultField::LiquidFungible.into();
            let (handle, _store_access) = track
                .acquire_lock(
                    &node_id,
                    MAIN_BASE_PARTITION,
                    &substate_key,
                    LockFlags::MUTABLE,
                )
                .unwrap();
            let (substate_value, _store_access) = track.read_substate(handle);
            let mut substate: LiquidFungibleResource = substate_value.as_typed().unwrap();
            substate.put(LiquidFungibleResource::new(amount));
            track.update_substate(handle, IndexedScryptoValue::from_typed(&substate));
            track.close_substate(handle);
        }

        // Take fee payments
        let fee_summary = fee_reserve.finalize();
        let mut fee_payments: IndexMap<NodeId, Decimal> = index_map_new();
        let mut required = fee_summary.total_execution_cost_xrd
            + fee_summary.total_tipping_cost_xrd
            + fee_summary.total_state_expansion_cost_xrd
            + fee_summary.total_royalty_cost_xrd;
        let mut collected_fees = LiquidFungibleResource::new(Decimal::ZERO);
        for (vault_id, mut locked, contingent) in fee_summary.locked_fees.iter().cloned().rev() {
            let amount = if contingent {
                if is_success {
                    Decimal::min(locked.amount(), required)
                } else {
                    Decimal::zero()
                }
            } else {
                Decimal::min(locked.amount(), required)
            };

            // Take fees
            collected_fees.put(locked.take_by_amount(amount).unwrap());
            required -= amount;

            // Refund overpayment
            let (handle, _store_access) = track
                .acquire_lock(
                    &vault_id,
                    MAIN_BASE_PARTITION,
                    &FungibleVaultField::LiquidFungible.into(),
                    LockFlags::MUTABLE,
                )
                .unwrap();
            let (substate_value, _store_access) = track.read_substate(handle);
            let mut substate: LiquidFungibleResource = substate_value.as_typed().unwrap();
            substate.put(locked);
            track.update_substate(handle, IndexedScryptoValue::from_typed(&substate));
            track.close_substate(handle);

            // Record final payments
            *fee_payments.entry(vault_id).or_default() += amount;
        }

        let tips_to_distribute = fee_summary.tips_to_distribute();
        let fees_to_distribute = fee_summary.fees_to_distribute();

        // Sanity check
        assert_eq!(required, Decimal::ZERO);
        assert_eq!(fee_summary.total_bad_debt_xrd, Decimal::ZERO);
        assert_eq!(
            tips_to_distribute + fees_to_distribute,
            collected_fees.amount() - fee_summary.total_royalty_cost_xrd /* royalty already distributed */
        );

        if !tips_to_distribute.is_zero() || !fees_to_distribute.is_zero() {
            // Fetch current leader
            // TODO: maybe we should move current leader into validator rewards?
            let handle = track
                .acquire_lock(
                    CONSENSUS_MANAGER.as_node_id(),
                    MAIN_BASE_PARTITION,
                    &ConsensusManagerField::ConsensusManager.into(),
                    LockFlags::read_only(),
                )
                .unwrap()
                .0;
            let substate: ConsensusManagerSubstate =
                track.read_substate(handle).0.as_typed().unwrap();
            let current_leader = substate.current_leader;
            track.close_substate(handle);

            // Update validator rewards
            let handle = track
                .acquire_lock(
                    CONSENSUS_MANAGER.as_node_id(),
                    MAIN_BASE_PARTITION,
                    &ConsensusManagerField::ValidatorRewards.into(),
                    LockFlags::MUTABLE,
                )
                .unwrap()
                .0;
            let mut substate: ValidatorRewardsSubstate =
                track.read_substate(handle).0.as_typed().unwrap();
            let proposer_rewards = if let Some(current_leader) = current_leader {
                let rewards = tips_to_distribute * TIPS_PROPOSER_SHARE_PERCENTAGE / dec!(100)
                    + fees_to_distribute * FEES_PROPOSER_SHARE_PERCENTAGE / dec!(100);
                substate
                    .proposer_rewards
                    .entry(current_leader)
                    .or_default()
                    .add_assign(rewards);
                rewards
            } else {
                Decimal::ZERO
            };
            let validator_set_rewards = {
                tips_to_distribute * TIPS_VALIDATOR_SET_SHARE_PERCENTAGE / dec!(100)
                    + fees_to_distribute * FEES_VALIDATOR_SET_SHARE_PERCENTAGE / dec!(100)
            };
            let vault_node_id = substate.rewards_vault.0 .0;
            track.update_substate(handle, IndexedScryptoValue::from_typed(&substate));
            track.close_substate(handle);

            // Put validator rewards into the vault
            let handle = track
                .acquire_lock(
                    &vault_node_id,
                    MAIN_BASE_PARTITION,
                    &FungibleVaultField::LiquidFungible.into(),
                    LockFlags::MUTABLE,
                )
                .unwrap()
                .0;
            let mut substate: LiquidFungibleResource =
                track.read_substate(handle).0.as_typed().unwrap();
            substate.put(
                collected_fees
                    .take_by_amount(proposer_rewards + validator_set_rewards)
                    .unwrap(),
            );
            track.update_substate(handle, IndexedScryptoValue::from_typed(&substate));
            track.close_substate(handle);
        }

        (fee_summary, fee_payments)
    }

    fn update_transaction_tracker(
        track: &mut Track<S, SpreadPrefixKeyMapper>,
        next_epoch: Epoch,
        intent_hash: &TransactionIntentHash,
        is_success: bool,
    ) {
        // Read the intent hash store
        let handle = track
            .acquire_lock(
                TRANSACTION_TRACKER.as_node_id(),
                MAIN_BASE_PARTITION,
                &TransactionTrackerField::TransactionTracker.into(),
                LockFlags::MUTABLE,
            )
            .unwrap()
            .0;
        let mut transaction_tracker: TransactionTrackerSubstate =
            track.read_substate(handle).0.as_typed().unwrap();

        // Update the status of the intent hash
        if let TransactionIntentHash::ToCheck {
            expiry_epoch,
            intent_hash,
        } = intent_hash
        {
            if let Some(partition_number) =
                transaction_tracker.partition_for_expiry_epoch(*expiry_epoch)
            {
                let handle = track
                    .acquire_lock_virtualize(
                        TRANSACTION_TRACKER.as_node_id(),
                        PartitionNumber(partition_number),
                        &SubstateKey::Map(intent_hash.to_vec()),
                        LockFlags::MUTABLE,
                        || {
                            Some(IndexedScryptoValue::from_typed(&KeyValueEntrySubstate {
                                value: Option::<TransactionStatus>::None,
                                mutability: SubstateMutability::Mutable,
                            }))
                        },
                    )
                    .unwrap()
                    .0;
                track.update_substate(
                    handle,
                    IndexedScryptoValue::from_typed(&KeyValueEntrySubstate {
                        value: Some(if is_success {
                            TransactionStatus::CommittedSuccess
                        } else {
                            TransactionStatus::CommittedFailure
                        }),
                        // TODO: maybe make it immutable, but how does this affect partition deletion?
                        mutability: SubstateMutability::Mutable,
                    }),
                );
                track.close_substate(handle);
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
        track.update_substate(
            handle,
            IndexedScryptoValue::from_typed(&transaction_tracker),
        );
        track.close_substate(handle);
    }

    #[cfg(not(feature = "alloc"))]
    fn print_executable(executable: &Executable) {
        println!("{:-^100}", "Executable");
        println!("Intent hash: {}", executable.intent_hash().as_hash());
        println!("Payload size: {}", executable.payload_size());
        println!("Fee payment: {:?}", executable.fee_payment());
        println!(
            "Pre-allocated addresses: {:?}",
            executable.pre_allocated_addresses()
        );
        println!("Blobs: {:?}", executable.blobs().keys());
        println!("References: {:?}", executable.references());
    }

    #[cfg(not(feature = "alloc"))]
    fn print_execution_summary(receipt: &TransactionReceipt) {
        match &receipt.transaction_result {
            TransactionResult::Commit(commit) => {
                // NB - we use "to_string" to ensure they align correctly

                println!("{:-^100}", "Cost Breakdown");
                for (k, v) in &commit.fee_summary.execution_cost_breakdown {
                    println!("{:<75}: {:>15}", k, v.to_string());
                }

                println!("{:-^100}", "Cost Totals");
                println!(
                    "{:<30}: {:>15}",
                    "Cost Unit Limit",
                    commit.fee_summary.cost_unit_limit.to_string()
                );
                println!(
                    "{:<30}: {:>15}",
                    "Cost Units Consumed",
                    commit.fee_summary.execution_cost_sum.to_string()
                );
                println!(
                    "{:<30}: {:>15}",
                    "Execution Costs in XRD",
                    commit.fee_summary.total_execution_cost_xrd.to_string()
                );
                println!(
                    "{:<30}: {:>15}",
                    "Tipping Costs in XRD",
                    commit.fee_summary.total_tipping_cost_xrd.to_string()
                );
                println!(
                    "{:<30}: {:>15}",
                    "State Expansion Costs in XRD",
                    commit
                        .fee_summary
                        .total_state_expansion_cost_xrd
                        .to_string()
                );
                println!(
                    "{:<30}: {:>15}",
                    "Royalty Costs in XRD",
                    commit.fee_summary.total_royalty_cost_xrd.to_string()
                );

                println!("{:-^100}", "Application Logs");
                for (level, message) in &commit.application_logs {
                    println!("[{}] {}", level, message);
                }

                println!("{:-^100}", "Outcome");
                println!(
                    "{}",
                    match &commit.outcome {
                        TransactionOutcome::Success(_) => "Success".to_string(),
                        TransactionOutcome::Failure(error) => format!("Failure: {:?}", error),
                    }
                );
            }
            TransactionResult::Reject(e) => {
                println!("{:-^100}", "Transaction Rejected");
                println!("{:?}", e.error);
            }
            TransactionResult::Abort(e) => {
                println!("{:-^100}", "Transaction Aborted");
                println!("{:?}", e);
            }
        }
        println!("{:-^100}", "Finish");
    }
}

pub fn execute_and_commit_transaction<
    S: SubstateDatabase + CommittableSubstateDatabase,
    W: WasmEngine,
>(
    substate_db: &mut S,
    scrypto_interpreter: &ScryptoVm<W>,
    fee_reserve_config: &FeeReserveConfig,
    execution_config: &ExecutionConfig,
    transaction: &Executable,
) -> TransactionReceipt {
    let receipt = execute_transaction(
        substate_db,
        scrypto_interpreter,
        fee_reserve_config,
        execution_config,
        transaction,
    );
    if let TransactionResult::Commit(commit) = &receipt.transaction_result {
        substate_db.commit(&commit.state_updates.database_updates);
    }
    receipt
}

pub fn execute_transaction<S: SubstateDatabase, W: WasmEngine>(
    substate_db: &S,
    scrypto_interpreter: &ScryptoVm<W>,
    fee_reserve_config: &FeeReserveConfig,
    execution_config: &ExecutionConfig,
    transaction: &Executable,
) -> TransactionReceipt {
    TransactionExecutor::new(substate_db, scrypto_interpreter).execute(
        transaction,
        fee_reserve_config,
        execution_config,
    )
}

enum TransactionResultType {
    Commit(Result<Vec<InstructionOutput>, RuntimeError>),
    Reject(RejectionError),
    Abort(AbortReason),
}
