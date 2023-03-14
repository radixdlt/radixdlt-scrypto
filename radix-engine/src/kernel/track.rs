use crate::blueprints::resource::NonFungibleSubstate;
use crate::blueprints::transaction_processor::TransactionProcessorError;
use crate::errors::*;
use crate::ledger::*;
use crate::state_manager::StateDiff;
use crate::system::kernel_modules::costing::u128_to_decimal;
use crate::system::kernel_modules::costing::FinalizingFeeReserve;
use crate::system::kernel_modules::costing::{CostingError, FeeReserveError};
use crate::system::kernel_modules::costing::{FeeSummary, SystemLoanFeeReserve};
use crate::system::node_substates::{
    PersistedSubstate, RuntimeSubstate, SubstateRef, SubstateRefMut,
};
use crate::transaction::RejectResult;
use crate::transaction::StateUpdateSummary;
use crate::transaction::TransactionOutcome;
use crate::transaction::TransactionResult;
use crate::transaction::{AbortReason, AbortResult, CommitResult};
use crate::types::*;
use radix_engine_interface::api::component::KeyValueStoreEntrySubstate;
use radix_engine_interface::api::substate_api::LockFlags;
use radix_engine_interface::api::types::Level;
use radix_engine_interface::api::types::*;
use radix_engine_interface::blueprints::resource::LiquidFungibleResource;
use radix_engine_interface::blueprints::transaction_processor::InstructionOutput;
use radix_engine_interface::crypto::hash;
use sbor::rust::collections::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Sbor)]
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
pub struct Track<'s> {
    substate_store: &'s dyn ReadableSubstateStore,
    loaded_substates: HashMap<SubstateId, LoadedSubstate>,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum TrackError {
    NotFound(SubstateId),
    SubstateLocked(SubstateId, LockState),
    LockUnmodifiedBaseOnNewSubstate(SubstateId),
    LockUnmodifiedBaseOnOnUpdatedSubstate(SubstateId),
}

pub struct PreExecutionError {
    pub fee_summary: FeeSummary,
    pub error: FeeReserveError,
}

impl<'s> Track<'s> {
    pub fn new(substate_store: &'s dyn ReadableSubstateStore) -> Self {
        Self {
            substate_store,
            loaded_substates: HashMap::new(),
        }
    }

    /// Returns a copy of the substate associated with the given address, if exists
    fn load_substate(&mut self, substate_id: &SubstateId) -> Option<OutputValue> {
        self.substate_store.get_substate(substate_id)
    }

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

    pub fn get_substate(
        &mut self,
        node_id: RENodeId,
        module_id: NodeModuleId,
        offset: &SubstateOffset,
    ) -> SubstateRef {
        let runtime_substate = match (node_id, offset) {
            (_, SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(..)))
            | (
                RENodeId::NonFungibleStore(..),
                SubstateOffset::NonFungibleStore(NonFungibleStoreOffset::Entry(..)),
            ) => self.read_key_value(node_id, module_id, offset),
            _ => {
                let substate_id = SubstateId(node_id, module_id, offset.clone());
                &self
                    .loaded_substates
                    .get(&substate_id)
                    .unwrap_or_else(|| panic!("Substate {:?} was never locked", substate_id))
                    .substate
            }
        };
        runtime_substate.to_ref()
    }

    pub fn get_substate_mut(
        &mut self,
        node_id: RENodeId,
        module_id: NodeModuleId,
        offset: &SubstateOffset,
    ) -> SubstateRefMut {
        let runtime_substate = match (node_id, module_id, offset) {
            (_, _, SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(..)))
            | (
                RENodeId::NonFungibleStore(..),
                NodeModuleId::SELF,
                SubstateOffset::NonFungibleStore(NonFungibleStoreOffset::Entry(..)),
            ) => self.read_key_value_mut(node_id, module_id, offset),
            _ => {
                let substate_id = SubstateId(node_id, module_id, offset.clone());
                &mut self
                    .loaded_substates
                    .get_mut(&substate_id)
                    .unwrap_or_else(|| panic!("Substate {:?} was never locked", substate_id))
                    .substate
            }
        };
        runtime_substate.to_ref_mut()
    }

    pub fn insert_substate(&mut self, substate_id: SubstateId, substate: RuntimeSubstate) {
        assert!(!self.loaded_substates.contains_key(&substate_id));

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
    fn read_key_value(
        &mut self,
        node_id: RENodeId,
        module_id: NodeModuleId,
        offset: &SubstateOffset,
    ) -> &RuntimeSubstate {
        match (node_id, offset) {
            (
                RENodeId::NonFungibleStore(..),
                SubstateOffset::NonFungibleStore(NonFungibleStoreOffset::Entry(..)),
            ) => {
                let substate_id = SubstateId(node_id, module_id, offset.clone());
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
            (_, SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(..))) => {
                let substate_id = SubstateId(node_id, module_id, offset.clone());
                if !self.loaded_substates.contains_key(&substate_id) {
                    let output = self.load_substate(&substate_id);
                    let (substate, version) = output
                        .map(|o| (o.substate.to_runtime(), o.version))
                        .unwrap_or((
                            RuntimeSubstate::KeyValueStoreEntry(KeyValueStoreEntrySubstate::None),
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
        module_id: NodeModuleId,
        offset: &SubstateOffset,
    ) -> &mut RuntimeSubstate {
        match (node_id, offset) {
            (
                RENodeId::NonFungibleStore(..),
                SubstateOffset::NonFungibleStore(NonFungibleStoreOffset::Entry(..)),
            ) => {
                let substate_id = SubstateId(node_id, module_id, offset.clone());
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
            (_, SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(..))) => {
                let substate_id = SubstateId(node_id, module_id, offset.clone());
                if !self.loaded_substates.contains_key(&substate_id) {
                    let output = self.load_substate(&substate_id);
                    let (substate, version) = output
                        .map(|o| (o.substate.to_runtime(), o.version))
                        .unwrap_or((
                            RuntimeSubstate::KeyValueStoreEntry(KeyValueStoreEntrySubstate::None),
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

    pub fn finalize(
        mut self,
        mut invoke_result: Result<Vec<InstructionOutput>, RuntimeError>,
        mut fee_reserve: SystemLoanFeeReserve,
        application_events: Vec<(EventTypeIdentifier, Vec<u8>)>,
        application_logs: Vec<(Level, String)>,
    ) -> TransactionResult {
        // A `SuccessButFeeLoanNotRepaid` error is issued if a transaction finishes before
        // the SYSTEM_LOAN_AMOUNT is reached (which trigger a repay event) and even though
        // enough fee has been locked.
        //
        // Do another `repay` try during finalization to remedy it.
        if let Err(err) = fee_reserve.repay_all() {
            if invoke_result.is_ok() {
                invoke_result = Err(RuntimeError::ModuleError(ModuleError::CostingError(
                    CostingError::FeeReserveError(err),
                )));
            }
        }

        match determine_result_type(invoke_result, fee_reserve.fully_repaid()) {
            TransactionResultType::Commit(invoke_result) => {
                let is_success = invoke_result.is_ok();

                // Commit/rollback royalty
                if is_success {
                    for (recipient_vault_id, amount) in fee_reserve.royalty_cost() {
                        let node_id = RENodeId::Object(*recipient_vault_id);
                        let module_id = NodeModuleId::SELF;
                        let offset = SubstateOffset::Vault(VaultOffset::LiquidFungible);
                        self.acquire_lock(
                            SubstateId(node_id, module_id, offset.clone()),
                            LockFlags::MUTABLE,
                        )
                        .unwrap();
                        let substate: &mut LiquidFungibleResource =
                            self.get_substate_mut(node_id, module_id, &offset).into();
                        substate
                            .put(LiquidFungibleResource::new(u128_to_decimal(*amount)))
                            .unwrap();
                        self.release_lock(SubstateId(node_id, module_id, offset.clone()), false)
                            .unwrap();
                    }
                } else {
                    fee_reserve.revert_royalty();
                }

                // Keep/rollback events
                let application_events = if is_success {
                    application_events
                } else {
                    Vec::new()
                };

                // Keep logs always, for better debuggability
                let application_logs = application_logs;

                let finalizing_track = FinalizingTrack {
                    substate_store: self.substate_store,
                    loaded_substates: self.loaded_substates.into_iter().collect(),
                };
                TransactionResult::Commit(finalizing_track.calculate_commit_result(
                    invoke_result,
                    application_events,
                    application_logs,
                    fee_reserve,
                ))
            }
            TransactionResultType::Reject(rejection_error) => {
                TransactionResult::Reject(RejectResult {
                    error: rejection_error,
                })
            }
            TransactionResultType::Abort(abort_reason) => TransactionResult::Abort(AbortResult {
                reason: abort_reason,
            }),
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
    is_loan_fully_repaid: bool,
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
    if !is_loan_fully_repaid {
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
    loaded_substates: BTreeMap<SubstateId, LoadedSubstate>,
}

impl<'s> FinalizingTrack<'s> {
    fn calculate_commit_result(
        self,
        invoke_result: Result<Vec<InstructionOutput>, RuntimeError>,
        application_events: Vec<(EventTypeIdentifier, Vec<u8>)>,
        application_logs: Vec<(Level, String)>,
        fee_reserve: SystemLoanFeeReserve,
    ) -> CommitResult {
        let is_success = invoke_result.is_ok();

        // Calculate the substates for persistence
        let mut to_persist = HashMap::new();
        if is_success {
            for (id, loaded) in self.loaded_substates {
                let old_version = match &loaded.metastate {
                    SubstateMetaState::New => None,
                    SubstateMetaState::Existing { old_version, .. } => Some(*old_version),
                };
                to_persist.insert(id, (loaded.substate.to_persisted(), old_version));
            }
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
        };

        // Finalize fee payments
        let fee_summary = fee_reserve.finalize();
        let mut fee_payments: BTreeMap<ObjectId, Decimal> = BTreeMap::new();
        let mut required = fee_summary.total_execution_cost_xrd
            + fee_summary.total_royalty_cost_xrd
            - fee_summary.total_bad_debt_xrd;
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
            locked.take_by_amount(amount).unwrap();
            required -= amount;

            // Refund overpayment
            let substate_id = SubstateId(
                RENodeId::Object(vault_id),
                NodeModuleId::SELF,
                SubstateOffset::Vault(VaultOffset::LiquidFungible),
            );

            // Update substate
            let (substate, _) = to_persist.get_mut(&substate_id).unwrap();
            substate.vault_liquid_fungible_mut().put(locked).unwrap();

            // Record final payments
            *fee_payments.entry(vault_id).or_default() += amount;
        }

        // TODO: update XRD total supply or disable it
        // TODO: pay tips to the lead validator

        let (state_updates, state_update_summary) =
            Self::generate_diff(self.substate_store, to_persist);

        CommitResult {
            state_updates,
            state_update_summary,
            outcome: match invoke_result {
                Ok(output) => TransactionOutcome::Success(output),
                Err(error) => TransactionOutcome::Failure(error),
            },
            fee_summary,
            fee_payments,

            application_events,
            application_logs,
        }
    }

    pub fn generate_diff(
        substate_store: &dyn ReadableSubstateStore,
        to_persist: HashMap<SubstateId, (PersistedSubstate, Option<u32>)>,
    ) -> (StateDiff, StateUpdateSummary) {
        let mut diff = StateDiff::new();

        for (substate_id, (substate, ..)) in to_persist {
            let next_version = if let Some(existing_output_id) =
                Self::get_substate_output_id(substate_store, &substate_id)
            {
                let next_version = existing_output_id.version + 1;
                diff.down_substates.insert(existing_output_id);
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

        (diff, StateUpdateSummary::default())
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
