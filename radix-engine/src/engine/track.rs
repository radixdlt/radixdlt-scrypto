use radix_engine_interface::api::types::{
    GlobalAddress, GlobalOffset, KeyValueStoreOffset, Level, NonFungibleStoreOffset, RENodeId,
    SubstateId, SubstateOffset, VaultId, VaultOffset,
};
use radix_engine_interface::crypto::hash;
use radix_engine_interface::model::*;
use sbor::rust::collections::*;
use transaction::model::Executable;

use crate::engine::*;
use crate::fee::FeeSummary;
use crate::fee::FeeTable;
use crate::fee::{ExecutionFeeReserve, FeeReserveError};
use crate::fee::{FeeReserve, RoyaltyReceiver};
use crate::ledger::*;
use crate::model::RuntimeSubstate;
use crate::model::SubstateRef;
use crate::model::TransactionProcessorError;
use crate::model::*;
use crate::model::{KeyValueStoreEntrySubstate, PersistedSubstate};
use crate::model::{NonFungibleSubstate, SubstateRefMut};
use crate::state_manager::StateDiff;
use crate::transaction::EntityChanges;
use crate::transaction::RejectResult;
use crate::transaction::TransactionOutcome;
use crate::transaction::TransactionResult;
use crate::transaction::{AbortReason, AbortResult, CommitResult};
use crate::types::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Encode, Decode, Categorize)]
pub enum LockState {
    Read(usize),
    Write,
}

impl LockState {
    pub fn no_lock() -> Self {
        Self::Read(0)
    }
}

#[derive(Debug)]
pub enum ExistingMetaState {
    Loaded,
    Updated(Option<PersistedSubstate>),
}

#[derive(Debug)]
pub enum SubstateMetaState {
    New,
    Existing {
        old_version: u32,
        state: ExistingMetaState,
    },
}

#[derive(Debug)]
pub struct LoadedSubstate {
    substate: RuntimeSubstate,
    lock_state: LockState,
    metastate: SubstateMetaState,
}

/// Transaction-wide states and side effects
pub struct Track<'s, R: FeeReserve> {
    application_logs: Vec<(Level, String)>,
    substate_store: &'s dyn ReadableSubstateStore,
    loaded_substates: BTreeMap<SubstateId, LoadedSubstate>,
    new_global_addresses: Vec<GlobalAddress>,
    fee_reserve: R,
    pub fee_table: FeeTable,
    pub vault_ops: Vec<(ResolvedActor, VaultId, VaultOp)>,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum TrackError {
    NotFound(SubstateId),
    SubstateLocked(SubstateId, LockState),
    LockUnmodifiedBaseOnNewSubstate(SubstateId),
    LockUnmodifiedBaseOnOnUpdatedSubstate(SubstateId),
}

pub struct TrackReceipt {
    pub fee_summary: FeeSummary,
    //pub application_logs: Vec<(Level, String)>,
    pub result: TransactionResult,
    pub events: Vec<TrackedEvent>,
}

pub struct PreExecutionError {
    pub fee_summary: FeeSummary,
    pub error: FeeReserveError,
}

impl<'s, R: FeeReserve> Track<'s, R> {
    pub fn new(
        substate_store: &'s dyn ReadableSubstateStore,
        fee_reserve: R,
        fee_table: FeeTable,
    ) -> Self {
        Self {
            application_logs: Vec::new(),
            substate_store,
            loaded_substates: BTreeMap::new(),
            new_global_addresses: Vec::new(),
            fee_reserve,
            fee_table,
            vault_ops: Vec::new(),
        }
    }

    /// Adds a log message.
    pub fn add_log(&mut self, level: Level, message: String) {
        self.application_logs.push((level, message));
    }

    /// Returns a copy of the substate associated with the given address, if exists
    fn load_substate(&mut self, substate_id: &SubstateId) -> Option<OutputValue> {
        self.substate_store.get_substate(substate_id)
    }

    #[inline]
    /// During execution, we only allow access to the Execution subset of the FeeReserve
    pub fn fee_reserve(&mut self) -> &mut impl ExecutionFeeReserve {
        &mut self.fee_reserve
    }

    // TODO: to read/write a value owned by track requires three coordinated steps:
    // 1. Attempt to acquire the lock
    // 2. Apply the operation
    // 3. Release lock
    //
    // A better idea is properly to move the lock-unlock logic into the operation themselves OR to have a
    // representation of locked resource and apply operation on top of it.
    //
    // Also enables us to store state associated with the lock, like the `write_through` flag.

    pub fn acquire_lock(
        &mut self,
        substate_id: SubstateId,
        flags: LockFlags,
    ) -> Result<(), TrackError> {
        // Load the substate from state track
        if !self.loaded_substates.contains_key(&substate_id) {
            let maybe_substate = self.load_substate(&substate_id);
            if let Some(output) = maybe_substate {
                self.loaded_substates.insert(
                    substate_id.clone(),
                    LoadedSubstate {
                        substate: output.substate.to_runtime(),
                        lock_state: LockState::no_lock(),
                        metastate: SubstateMetaState::Existing {
                            old_version: output.version,
                            state: ExistingMetaState::Loaded,
                        },
                    },
                );
            } else {
                return Err(TrackError::NotFound(substate_id));
            }
        }

        let loaded_substate = self
            .loaded_substates
            .get_mut(&substate_id)
            .expect("Existence checked upfront");

        if flags.contains(LockFlags::UNMODIFIED_BASE) {
            match loaded_substate.metastate {
                SubstateMetaState::New => {
                    return Err(TrackError::LockUnmodifiedBaseOnNewSubstate(substate_id))
                }
                SubstateMetaState::Existing {
                    state: ExistingMetaState::Updated(..),
                    ..
                } => {
                    return Err(TrackError::LockUnmodifiedBaseOnOnUpdatedSubstate(
                        substate_id,
                    ))
                }
                SubstateMetaState::Existing {
                    state: ExistingMetaState::Loaded,
                    ..
                } => {}
            }
        }

        match loaded_substate.lock_state {
            LockState::Read(n) => {
                if flags.contains(LockFlags::MUTABLE) {
                    if n != 0 {
                        return Err(TrackError::SubstateLocked(
                            substate_id,
                            loaded_substate.lock_state,
                        ));
                    }
                    loaded_substate.lock_state = LockState::Write;
                } else {
                    loaded_substate.lock_state = LockState::Read(n + 1);
                }
            }
            LockState::Write => {
                return Err(TrackError::SubstateLocked(
                    substate_id,
                    loaded_substate.lock_state,
                ));
            }
        }

        Ok(())
    }

    pub fn release_lock(
        &mut self,
        substate_id: SubstateId,
        force_write: bool,
    ) -> Result<(), TrackError> {
        let loaded_substate = self
            .loaded_substates
            .get_mut(&substate_id)
            .expect("Attempted to release lock on never borrowed substate");

        match loaded_substate.lock_state {
            LockState::Read(n) => loaded_substate.lock_state = LockState::Read(n - 1),
            LockState::Write => {
                loaded_substate.lock_state = LockState::no_lock();

                if force_write {
                    let persisted_substate = loaded_substate.substate.clone_to_persisted();
                    match &mut loaded_substate.metastate {
                        SubstateMetaState::Existing { state, .. } => {
                            *state = ExistingMetaState::Updated(Some(persisted_substate));
                        }
                        SubstateMetaState::New => {
                            panic!("Unexpected");
                        }
                    }
                } else {
                    match &mut loaded_substate.metastate {
                        SubstateMetaState::New => {}
                        SubstateMetaState::Existing { state, .. } => match state {
                            ExistingMetaState::Loaded => *state = ExistingMetaState::Updated(None),
                            ExistingMetaState::Updated(..) => {}
                        },
                    }
                }
            }
        }

        Ok(())
    }

    pub fn get_substate(&mut self, node_id: RENodeId, offset: &SubstateOffset) -> SubstateRef {
        let runtime_substate = match (node_id, offset) {
            (
                RENodeId::KeyValueStore(..),
                SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(..)),
            )
            | (
                RENodeId::NonFungibleStore(..),
                SubstateOffset::NonFungibleStore(NonFungibleStoreOffset::Entry(..)),
            ) => self.read_key_value(node_id, offset),
            _ => {
                let substate_id = SubstateId(node_id, offset.clone());
                &self
                    .loaded_substates
                    .get(&substate_id)
                    .expect(&format!("Substate {:?} was never locked", substate_id))
                    .substate
            }
        };
        runtime_substate.to_ref()
    }

    pub fn get_substate_mut(
        &mut self,
        node_id: RENodeId,
        offset: &SubstateOffset,
    ) -> SubstateRefMut {
        let runtime_substate = match (node_id, offset) {
            (
                RENodeId::KeyValueStore(..),
                SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(..)),
            )
            | (
                RENodeId::NonFungibleStore(..),
                SubstateOffset::NonFungibleStore(NonFungibleStoreOffset::Entry(..)),
            ) => self.read_key_value_mut(node_id, offset),
            _ => {
                let substate_id = SubstateId(node_id, offset.clone());
                &mut self
                    .loaded_substates
                    .get_mut(&substate_id)
                    .expect(&format!("Substate {:?} was never locked", substate_id))
                    .substate
            }
        };
        runtime_substate.to_ref_mut()
    }

    pub fn insert_substate(&mut self, substate_id: SubstateId, substate: RuntimeSubstate) {
        assert!(!self.loaded_substates.contains_key(&substate_id));

        match &substate_id {
            SubstateId(
                RENodeId::Global(global_address),
                SubstateOffset::Global(GlobalOffset::Global),
            ) => {
                self.new_global_addresses.push(*global_address);
            }
            _ => {}
        }

        self.loaded_substates.insert(
            substate_id,
            LoadedSubstate {
                substate,
                lock_state: LockState::no_lock(),
                metastate: SubstateMetaState::New,
            },
        );
    }

    /// Returns the value of a key value pair
    fn read_key_value(&mut self, node_id: RENodeId, offset: &SubstateOffset) -> &RuntimeSubstate {
        match (node_id, offset) {
            (
                RENodeId::NonFungibleStore(..),
                SubstateOffset::NonFungibleStore(NonFungibleStoreOffset::Entry(..)),
            ) => {
                let substate_id = SubstateId(node_id, offset.clone());
                if !self.loaded_substates.contains_key(&substate_id) {
                    let output = self.load_substate(&substate_id);
                    let (substate, version) = output
                        .map(|o| (o.substate.to_runtime(), o.version))
                        .unwrap_or((RuntimeSubstate::NonFungible(NonFungibleSubstate(None)), 0));

                    self.loaded_substates.insert(
                        substate_id.clone(),
                        LoadedSubstate {
                            substate,
                            lock_state: LockState::no_lock(),
                            metastate: SubstateMetaState::Existing {
                                old_version: version,
                                state: ExistingMetaState::Loaded,
                            },
                        },
                    );
                }

                &self.loaded_substates.get(&substate_id).unwrap().substate
            }
            (
                RENodeId::KeyValueStore(..),
                SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(..)),
            ) => {
                let substate_id = SubstateId(node_id, offset.clone());
                if !self.loaded_substates.contains_key(&substate_id) {
                    let output = self.load_substate(&substate_id);
                    let (substate, version) = output
                        .map(|o| (o.substate.to_runtime(), o.version))
                        .unwrap_or((
                            RuntimeSubstate::KeyValueStoreEntry(KeyValueStoreEntrySubstate(None)),
                            0,
                        ));

                    self.loaded_substates.insert(
                        substate_id.clone(),
                        LoadedSubstate {
                            substate,
                            lock_state: LockState::no_lock(),
                            metastate: SubstateMetaState::Existing {
                                old_version: version,
                                state: ExistingMetaState::Loaded,
                            },
                        },
                    );
                }

                &self.loaded_substates.get(&substate_id).unwrap().substate
            }
            _ => panic!("Invalid keyed value"),
        }
    }

    fn read_key_value_mut(
        &mut self,
        node_id: RENodeId,
        offset: &SubstateOffset,
    ) -> &mut RuntimeSubstate {
        match (node_id, offset) {
            (
                RENodeId::NonFungibleStore(..),
                SubstateOffset::NonFungibleStore(NonFungibleStoreOffset::Entry(..)),
            ) => {
                let substate_id = SubstateId(node_id, offset.clone());
                if !self.loaded_substates.contains_key(&substate_id) {
                    let output = self.load_substate(&substate_id);
                    let (substate, version) = output
                        .map(|o| (o.substate.to_runtime(), o.version))
                        .unwrap_or((RuntimeSubstate::NonFungible(NonFungibleSubstate(None)), 0));

                    self.loaded_substates.insert(
                        substate_id.clone(),
                        LoadedSubstate {
                            substate,
                            lock_state: LockState::no_lock(),
                            metastate: SubstateMetaState::Existing {
                                old_version: version,
                                state: ExistingMetaState::Loaded,
                            },
                        },
                    );
                }

                &mut self
                    .loaded_substates
                    .get_mut(&substate_id)
                    .unwrap()
                    .substate
            }
            (
                RENodeId::KeyValueStore(..),
                SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(..)),
            ) => {
                let substate_id = SubstateId(node_id, offset.clone());
                if !self.loaded_substates.contains_key(&substate_id) {
                    let output = self.load_substate(&substate_id);
                    let (substate, version) = output
                        .map(|o| (o.substate.to_runtime(), o.version))
                        .unwrap_or((
                            RuntimeSubstate::KeyValueStoreEntry(KeyValueStoreEntrySubstate(None)),
                            0,
                        ));

                    self.loaded_substates.insert(
                        substate_id.clone(),
                        LoadedSubstate {
                            substate,
                            lock_state: LockState::no_lock(),
                            metastate: SubstateMetaState::Existing {
                                old_version: version,
                                state: ExistingMetaState::Loaded,
                            },
                        },
                    );
                }

                &mut self
                    .loaded_substates
                    .get_mut(&substate_id)
                    .unwrap()
                    .substate
            }
            _ => panic!("Invalid keyed value"),
        }
    }

    pub fn apply_pre_execution_costs(
        mut self,
        transaction: &Executable,
    ) -> Result<Self, PreExecutionError> {
        let result = self.attempt_apply_pre_execution_costs(transaction);

        match result {
            Ok(()) => Ok(self),
            Err(error) => Err(PreExecutionError {
                fee_summary: self.fee_reserve.finalize(),
                error,
            }),
        }
    }

    fn attempt_apply_pre_execution_costs(
        &mut self,
        executable: &Executable,
    ) -> Result<(), FeeReserveError> {
        self.fee_reserve
            .consume_deferred(self.fee_table.tx_base_fee(), 1, "tx_base_fee")
            .and_then(|()| {
                self.fee_reserve.consume_deferred(
                    self.fee_table.tx_payload_cost_per_byte(),
                    executable.payload_size(),
                    "tx_payload_cost",
                )
            })
            .and_then(|()| {
                self.fee_reserve.consume_deferred(
                    self.fee_table.tx_signature_verification_per_sig(),
                    executable.auth_zone_params().initial_proofs.len(),
                    "tx_signature_verification",
                )
            })
    }

    pub fn finalize(
        self,
        invoke_result: Result<Vec<InstructionOutput>, RuntimeError>,
        events: Vec<TrackedEvent>,
    ) -> TrackReceipt {
        // Close fee reserve
        let mut fee_summary = self.fee_reserve.finalize();

        let result = match determine_result_type(invoke_result, &fee_summary) {
            TransactionResultType::Commit(invoke_result) => {
                let finalizing_track = FinalizingTrack {
                    substate_store: self.substate_store,
                    new_global_addresses: self.new_global_addresses,
                    loaded_substates: self.loaded_substates,
                    vault_ops: self.vault_ops,
                };
                finalizing_track.calculate_commit_result(invoke_result, &mut fee_summary)
            }
            TransactionResultType::Reject(rejection_error) => {
                TransactionResult::Reject(RejectResult {
                    error: rejection_error,
                })
            }
            TransactionResultType::Abort(abort_reason) => TransactionResult::Abort(AbortResult {
                reason: abort_reason,
            }),
        };

        TrackReceipt {
            fee_summary,
            result,
            events,
        }
    }
}

pub enum TransactionResultType {
    Commit(Result<Vec<InstructionOutput>, RuntimeError>),
    Reject(RejectionError),
    Abort(AbortReason),
}

fn determine_result_type(
    invoke_result: Result<Vec<InstructionOutput>, RuntimeError>,
    fee_summary: &FeeSummary,
) -> TransactionResultType {
    // First - check for required rejections from explicit invoke result errors
    match &invoke_result {
        Err(RuntimeError::ApplicationError(ApplicationError::TransactionProcessorError(err))) => {
            match err {
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
            }
        }
        Err(err) => {
            if let Some(abort_reason) = err.abortion() {
                return TransactionResultType::Abort(abort_reason.clone());
            }
        }
        _ => {}
    }

    // Check for errors before loan is repaid - in which case, we also reject
    if !fee_summary.loan_fully_repaid() {
        return match invoke_result {
            Ok(..) => TransactionResultType::Reject(RejectionError::SuccessButFeeLoanNotRepaid),
            Err(error) => {
                TransactionResultType::Reject(RejectionError::ErrorBeforeFeeLoanRepaid(error))
            }
        };
    }

    return TransactionResultType::Commit(invoke_result);
}

/// This is just used when finalizing track into a commit
struct FinalizingTrack<'s> {
    substate_store: &'s dyn ReadableSubstateStore,
    new_global_addresses: Vec<GlobalAddress>,
    loaded_substates: BTreeMap<SubstateId, LoadedSubstate>,
    vault_ops: Vec<(ResolvedActor, VaultId, VaultOp)>,
}

impl<'s> FinalizingTrack<'s> {
    fn calculate_commit_result(
        self,
        invoke_result: Result<Vec<InstructionOutput>, RuntimeError>,
        fee_summary: &mut FeeSummary,
    ) -> TransactionResult {
        let is_success = invoke_result.is_ok();

        // Commit/rollback application state changes
        let mut to_persist = HashMap::new();
        let mut application_logs = Vec::new();
        let mut next_epoch = None;
        let new_global_addresses = if is_success {
            for (id, loaded) in self.loaded_substates {
                let old_version = match &loaded.metastate {
                    SubstateMetaState::New => None,
                    SubstateMetaState::Existing { old_version, .. } => Some(*old_version),
                };

                match id.1 {
                    SubstateOffset::Logger(LoggerOffset::Logger) => {
                        let logger: LoggerSubstate = loaded.substate.into();
                        application_logs.extend(logger.logs);
                    }
                    SubstateOffset::EpochManager(EpochManagerOffset::CurrentValidatorSet) => {
                        // TODO: Use application layer events rather than state updates to get this info
                        match &loaded.metastate {
                            SubstateMetaState::New
                            | SubstateMetaState::Existing {
                                state: ExistingMetaState::Updated(..),
                                ..
                            } => {
                                let validator_set = loaded.substate.validator_set();
                                let epoch = validator_set.epoch;
                                let validator_set = validator_set.validator_set.clone();
                                next_epoch = Some((validator_set, epoch));
                            }
                            _ => {}
                        }

                        to_persist.insert(id, (loaded.substate.to_persisted(), old_version));
                    }
                    _ => {
                        to_persist.insert(id, (loaded.substate.to_persisted(), old_version));
                    }
                }
            }

            self.new_global_addresses
        } else {
            for (id, loaded) in self.loaded_substates {
                match loaded.metastate {
                    SubstateMetaState::Existing {
                        old_version,
                        state: ExistingMetaState::Updated(Some(force_persist)),
                    } => {
                        to_persist.insert(id, (force_persist, Option::Some(old_version)));
                    }
                    _ => {}
                }
            }
            Vec::new()
        };

        // Revert royalty in case of failure
        if !is_success {
            fee_summary.total_royalty_cost_xrd = Decimal::ZERO;
            fee_summary.royalty_cost_unit_breakdown = HashMap::new();
        }

        // Finalize payments
        let mut actual_fee_payments: BTreeMap<VaultId, Decimal> = BTreeMap::new();
        let mut required = fee_summary.total_execution_cost_xrd
            + fee_summary.total_royalty_cost_xrd
            - fee_summary.bad_debt_xrd;
        let mut fees: Resource =
            Resource::new_empty(RADIX_TOKEN, ResourceType::Fungible { divisibility: 18 });
        for (vault_id, mut locked, contingent) in fee_summary.vault_locks.iter().cloned().rev() {
            let amount = if contingent {
                if is_success {
                    Decimal::min(locked.amount(), required)
                } else {
                    Decimal::zero()
                }
            } else {
                Decimal::min(locked.amount(), required)
            };

            // Deduct fee required
            required = required - amount;

            // Collect fees into collector
            fees.put(locked.take_by_amount(amount).unwrap()).unwrap();

            // Refund overpayment
            let substate_id = SubstateId(
                RENodeId::Vault(vault_id),
                SubstateOffset::Vault(VaultOffset::Vault),
            );

            // Update substate
            let (substate, old_version) = to_persist.remove(&substate_id).unwrap();
            let mut runtime_substate = substate.to_runtime();
            runtime_substate
                .vault_mut()
                .borrow_resource_mut()
                .put(locked)
                .unwrap();
            to_persist.insert(substate_id, (runtime_substate.to_persisted(), old_version));

            // Record final payments
            *actual_fee_payments.entry(vault_id).or_default() += amount;
        }
        fee_summary.vault_payments_xrd = Some(actual_fee_payments);

        // TODO: update XRD supply or disable it
        // TODO: pay tips to the lead validator

        for (receiver, amount) in &fee_summary.royalty_cost_unit_breakdown {
            match receiver {
                RoyaltyReceiver::Package(_, node_id) => {
                    let substate_id = SubstateId(
                        node_id.clone(),
                        SubstateOffset::Package(PackageOffset::RoyaltyAccumulator),
                    );
                    let accumulator_substate = to_persist.get(&substate_id).unwrap();
                    let royalty_vault_id = accumulator_substate
                        .0
                        .package_royalty_accumulator()
                        .royalty
                        .vault_id();
                    let royalty_vault_substate = to_persist
                        .get_mut(&SubstateId(
                            RENodeId::Vault(royalty_vault_id),
                            SubstateOffset::Vault(VaultOffset::Vault),
                        ))
                        .unwrap();
                    royalty_vault_substate
                        .0
                        .vault_mut()
                        .0
                        .put(
                            fees.take_by_amount(fee_summary.cost_unit_price * amount.clone())
                                .unwrap(),
                        )
                        .unwrap();
                }
                RoyaltyReceiver::Component(_, node_id) => {
                    let substate_id = SubstateId(
                        node_id.clone(),
                        SubstateOffset::Component(ComponentOffset::RoyaltyAccumulator),
                    );
                    let accumulator_substate = to_persist.get(&substate_id).unwrap();
                    let royalty_vault_id = accumulator_substate
                        .0
                        .component_royalty_accumulator()
                        .royalty
                        .vault_id();
                    let royalty_vault_substate = to_persist
                        .get_mut(&SubstateId(
                            RENodeId::Vault(royalty_vault_id),
                            SubstateOffset::Vault(VaultOffset::Vault),
                        ))
                        .unwrap();
                    royalty_vault_substate
                        .0
                        .vault_mut()
                        .0
                        .put(
                            fees.take_by_amount(fee_summary.cost_unit_price * amount.clone())
                                .unwrap(),
                        )
                        .unwrap();
                }
            }
        }

        // Generate commit result
        let execution_trace_receipt = ExecutionTraceReceipt::new(
            self.vault_ops,
            fee_summary.vault_payments_xrd.as_ref().unwrap(),
            &mut to_persist,
            invoke_result.is_ok(),
        );
        TransactionResult::Commit(CommitResult {
            outcome: match invoke_result {
                Ok(output) => TransactionOutcome::Success(output),
                Err(error) => TransactionOutcome::Failure(error),
            },
            state_updates: Self::generate_diff(self.substate_store, to_persist),
            entity_changes: EntityChanges::new(new_global_addresses),
            resource_changes: execution_trace_receipt.resource_changes,
            application_logs,
            next_epoch,
        })
    }

    pub fn generate_diff(
        substate_store: &dyn ReadableSubstateStore,
        to_persist: HashMap<SubstateId, (PersistedSubstate, Option<u32>)>,
    ) -> StateDiff {
        let mut diff = StateDiff::new();

        for (substate_id, (substate, ..)) in to_persist {
            let next_version = if let Some(existing_output_id) =
                Self::get_substate_output_id(substate_store, &substate_id)
            {
                let next_version = existing_output_id.version + 1;
                diff.down_substates.push(existing_output_id);
                next_version
            } else {
                0
            };
            let output_value = OutputValue {
                substate,
                version: next_version,
            };
            diff.up_substates.insert(substate_id.clone(), output_value);
        }

        diff
    }

    fn get_substate_output_id(
        substate_store: &dyn ReadableSubstateStore,
        substate_id: &SubstateId,
    ) -> Option<OutputId> {
        substate_store.get_substate(&substate_id).map(|s| OutputId {
            substate_id: substate_id.clone(),
            substate_hash: hash(
                scrypto_encode(&s.substate).expect("Saved substate couldn't be re-encoded"),
            ),
            version: s.version,
        })
    }
}
