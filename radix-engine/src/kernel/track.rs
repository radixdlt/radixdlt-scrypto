use crate::blueprints::transaction_processor::TransactionProcessorError;
use crate::errors::*;
use crate::system::kernel_modules::costing::FinalizingFeeReserve;
use crate::system::kernel_modules::costing::{CostingError, FeeReserveError};
use crate::system::kernel_modules::costing::{FeeSummary, SystemLoanFeeReserve};
use crate::system::node_modules::type_info::TypeInfoSubstate;
use crate::transaction::BalanceChange;
use crate::transaction::RejectResult;
use crate::transaction::StateUpdateSummary;
use crate::transaction::TransactionOutcome;
use crate::transaction::TransactionResult;
use crate::transaction::{AbortReason, AbortResult, CommitResult};
use crate::types::*;
use radix_engine_interface::api::substate_api::LockFlags;
use radix_engine_interface::blueprints::resource::LiquidFungibleResource;
use radix_engine_interface::blueprints::resource::VAULT_BLUEPRINT;
use radix_engine_interface::blueprints::transaction_processor::InstructionOutput;
use radix_engine_interface::crypto::hash;
use radix_engine_interface::types::Level;
use radix_engine_interface::types::*;
use radix_engine_stores::interface::{
    StateDependencies, StateUpdates, StoreLockError, SubstateDatabase, SubstateStore,
};
use sbor::rust::prelude::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Sbor)]
pub enum SubstateLockState {
    Read(usize),
    Write,
}

impl SubstateLockState {
    pub fn no_lock() -> Self {
        Self::Read(0)
    }
}

#[derive(Debug)]
pub enum ExistingMetaState {
    Loaded,
    Updated(Option<IndexedScryptoValue>),
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
    substate: IndexedScryptoValue,
    lock_state: SubstateLockState,
    meta_state: SubstateMetaState,
}

/// Transaction-wide states and side effects
pub struct Track<'s> {
    substate_db: &'s dyn SubstateDatabase,
    loaded_substates: IndexMap<NodeId, IndexMap<ModuleId, IndexMap<SubstateKey, LoadedSubstate>>>,
    locks: IndexMap<u32, (NodeId, ModuleId, SubstateKey, LockFlags)>,
    next_lock_id: u32,
}

impl<'s> Track<'s> {
    pub fn new(substate_db: &'s dyn SubstateDatabase) -> Self {
        Self {
            substate_db,
            loaded_substates: index_map_new(),
            locks: index_map_new(),
            next_lock_id: 0,
        }
    }

    fn new_lock_handle(
        &mut self,
        node_id: &NodeId,
        module_id: ModuleId,
        substate_key: &SubstateKey,
        flags: LockFlags,
    ) -> u32 {
        let new_lock = self.next_lock_id;
        self.locks
            .insert(new_lock, (*node_id, module_id, substate_key.clone(), flags));
        self.next_lock_id += 1;
        new_lock
    }

    fn loaded_substate<'a>(
        loaded_substates: &'a IndexMap<
            NodeId,
            IndexMap<ModuleId, IndexMap<SubstateKey, LoadedSubstate>>,
        >,
        node_id: &NodeId,
        module_id: ModuleId,
        substate_key: &SubstateKey,
    ) -> Option<&'a LoadedSubstate> {
        loaded_substates
            .get(node_id)
            .and_then(|m| m.get(&module_id))
            .and_then(|m| m.get(substate_key))
    }

    fn loaded_substate_mut<'a>(
        loaded_substates: &'a mut IndexMap<
            NodeId,
            IndexMap<ModuleId, IndexMap<SubstateKey, LoadedSubstate>>,
        >,
        node_id: &NodeId,
        module_id: ModuleId,
        substate_key: &SubstateKey,
    ) -> Option<&'a mut LoadedSubstate> {
        loaded_substates
            .get(node_id)
            .and_then(|m| m.get(&module_id))
            .and_then(|m| m.get(substate_key))
    }

    fn load_substate(
        &mut self,
        node_id: &NodeId,
        module_id: ModuleId,
        substate_key: &SubstateKey,
    ) -> Option<(IndexedScryptoValue, u32)> {
        self.substate_db
            .get_substate(node_id, module_id, substate_key)
            .expect("Database error")
            .map(|e| {
                (
                    IndexedScryptoValue::from_vec(e.0).expect("Failed to decode substate"),
                    0,
                )
            })
    }

    fn add_loaded_substate(
        &mut self,
        node_id: &NodeId,
        module_id: ModuleId,
        substate_key: &SubstateKey,
        substate_value_and_version: (IndexedScryptoValue, u32),
    ) {
        self.loaded_substates
            .entry(node_id)
            .or_default()
            .entry(&module_id)
            .or_default()
            .insert(
                substate_key.clone(),
                LoadedSubstate {
                    substate: substate_value_and_version.0,
                    lock_state: SubstateLockState::no_lock(),
                    meta_state: SubstateMetaState::Existing {
                        old_version: substate_value_and_version.1,
                        state: ExistingMetaState::Loaded,
                    },
                },
            );
    }
}

impl<'s> SubstateStore for Track<'s> {
    fn acquire_lock(
        &mut self,
        node_id: &NodeId,
        module_id: ModuleId,
        substate_key: &SubstateKey,
        flags: LockFlags,
    ) -> Result<u32, StoreLockError> {
        // Load the substate from state track
        if Self::loaded_substate(&self.loaded_substates, node_id, module_id, substate_key).is_none()
        {
            let maybe_substate = self.load_substate(node_id, module_id, substate_key);
            if let Some(output) = maybe_substate {
                self.add_loaded_substate(node_id, module_id, substate_key, output);
            } else {
                return Err(StoreLockError::NotFound(
                    *node_id,
                    module_id,
                    substate_key.clone(),
                ));
            }
        }

        // Check substate state
        let loaded_substate =
            Self::loaded_substate(&self.loaded_substates, node_id, module_id, substate_key)
                .unwrap();
        if flags.contains(LockFlags::UNMODIFIED_BASE) {
            match loaded_substate.meta_state {
                SubstateMetaState::New => {
                    return Err(StoreLockError::LockUnmodifiedBaseOnNewSubstate(
                        *node_id,
                        module_id,
                        substate_key.clone(),
                    ))
                }
                SubstateMetaState::Existing {
                    state: ExistingMetaState::Updated(..),
                    ..
                } => {
                    return Err(StoreLockError::LockUnmodifiedBaseOnOnUpdatedSubstate(
                        *node_id,
                        module_id,
                        substate_key.clone(),
                    ))
                }
                SubstateMetaState::Existing {
                    state: ExistingMetaState::Loaded,
                    ..
                } => {}
            }
        }

        // Check read/write permission
        match loaded_substate.lock_state {
            SubstateLockState::Read(n) => {
                if flags.contains(LockFlags::MUTABLE) {
                    if n != 0 {
                        return Err(StoreLockError::SubstateLocked(
                            *node_id,
                            module_id,
                            substate_key.clone(),
                        ));
                    }
                    loaded_substate.lock_state = SubstateLockState::Write;
                } else {
                    loaded_substate.lock_state = SubstateLockState::Read(n + 1);
                }
            }
            SubstateLockState::Write => {
                return Err(StoreLockError::SubstateLocked(
                    *node_id,
                    module_id,
                    substate_key.clone(),
                ));
            }
        }

        Ok(self.new_lock_handle(node_id, module_id, substate_key, flags))
    }

    fn release_lock(&mut self, handle: u32) {
        let (node_id, module_id, substate_key, flags) =
            self.locks.remove(&handle).expect("Invalid lock handle");

        let loaded_substate = Self::loaded_substate_mut(
            &mut self.loaded_substates,
            &node_id,
            module_id,
            &substate_key,
        )
        .expect("Substate missing for valid lock handle");

        match loaded_substate.lock_state {
            SubstateLockState::Read(n) => {
                loaded_substate.lock_state = SubstateLockState::Read(n - 1)
            }
            SubstateLockState::Write => {
                loaded_substate.lock_state = SubstateLockState::no_lock();

                if flags.contains(LockFlags::FORCE_WRITE) {
                    match &mut loaded_substate.meta_state {
                        SubstateMetaState::Existing { state, .. } => {
                            *state = ExistingMetaState::Updated(Some(loaded_substate.substate));
                        }
                        SubstateMetaState::New => {
                            panic!("Unexpected");
                        }
                    }
                } else {
                    match &mut loaded_substate.meta_state {
                        SubstateMetaState::New => {}
                        SubstateMetaState::Existing { state, .. } => match state {
                            ExistingMetaState::Loaded => *state = ExistingMetaState::Updated(None),
                            ExistingMetaState::Updated(..) => {}
                        },
                    }
                }
            }
        }
    }

    fn get_substate(&self, handle: u32) -> &IndexedScryptoValue {
        let (node_id, module_id, substate_key, flags) =
            self.locks.get(&handle).expect("Invalid lock handle");

        &Self::loaded_substate(&self.loaded_substates, node_id, module_id, substate_key)
            .expect("Substate missing for valid lock handle")
            .substate
    }

    fn put_substate(&mut self, handle: u32, substate_value: IndexedScryptoValue) {
        let (node_id, module_id, substate_key, flags) =
            self.locks.get(&handle).expect("Invalid lock handle");

        if !flags.contains(LockFlags::MUTABLE) {
            panic!("No write permission for {}", handle);
        }

        Self::loaded_substate_mut(&mut self.loaded_substates, node_id, module_id, substate_key)
            .expect("Substate missing for valid lock handle")
            .substate = substate_value;
    }

    fn insert_substate(
        &mut self,
        node_id: NodeId,
        module_id: ModuleId,
        substate_key: SubstateKey,
        substate_value: IndexedScryptoValue,
    ) {
        self.loaded_substates
            .entry(node_id)
            .or_default()
            .entry(&module_id)
            .or_default()
            .insert(
                substate_key.clone(),
                LoadedSubstate {
                    substate: substate_value,
                    lock_state: SubstateLockState::no_lock(),
                    meta_state: SubstateMetaState::New,
                },
            );
    }

    fn list_substates(
        &mut self,
        node_id: &NodeId,
        module_id: ModuleId,
    ) -> Box<dyn Iterator<Item = (SubstateKey, IndexedScryptoValue)>> {
        todo!()
    }

    fn revert_non_force_write_changes(&mut self) {
        todo!()
    }

    fn finalize(self) -> (StateUpdates, StateDependencies) {
        todo!()
    }
}

// TODO: consider moving `pre_finalize` to `TransactionExecutor`.

impl<'s> Track<'s> {
    fn pre_finalize(
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
                    for (_, (recipient_vault_id, amount)) in fee_reserve.royalty_cost() {
                        let node_id = recipient_vault_id;
                        let module_id = TypedModuleId::ObjectState;
                        let substate_key = VaultOffset::LiquidFungible.into();
                        let handle = self
                            .acquire_lock(&node_id, module_id, &substate_key, LockFlags::MUTABLE)
                            .unwrap();
                        let substate: LiquidFungibleResource =
                            self.get_substate(handle).as_typed().unwrap();
                        substate.put(LiquidFungibleResource::new(amount)).unwrap();
                        self.put_substate(handle, IndexedScryptoValue::from_typed(&substate));
                        self.release_lock(handle);
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
                    substate_db: self.substate_db,
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
    substate_db: &'s dyn SubstateDatabase,
    loaded_substates: IndexMap<SubstateId, LoadedSubstate>,
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
        let mut state_updates = index_map_new();
        if is_success {
            for (id, loaded) in self.loaded_substates {
                let old_version = match &loaded.meta_state {
                    SubstateMetaState::New => None,
                    SubstateMetaState::Existing { old_version, .. } => Some(*old_version),
                };
                state_updates.insert(id, (loaded.substate.to_persisted(), old_version));
            }
        } else {
            for (id, loaded) in self.loaded_substates {
                match loaded.meta_state {
                    SubstateMetaState::Existing {
                        old_version,
                        state: ExistingMetaState::Updated(Some(force_persist)),
                    } => {
                        state_updates.insert(id, (force_persist, Some(old_version)));
                    }
                    _ => {}
                }
            }
        };

        // Finalize fee payments
        let fee_summary = fee_reserve.finalize();
        let mut fee_payments: IndexMap<NodeId, Decimal> = index_map_new();
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
                NodeId::Object(vault_id),
                TypedModuleId::ObjectState,
                SubstateKey::Vault(VaultOffset::LiquidFungible),
            );

            // Update substate
            let (substate, _) = state_updates.get_mut(&substate_id).unwrap();
            substate.vault_liquid_fungible_mut().put(locked).unwrap();

            // Record final payments
            *fee_payments.entry(vault_id).or_default() += amount;
        }

        // TODO: update XRD total supply or disable it
        // TODO: pay tips to the lead validator

        let state_update_summary = Self::summarize_update(self.substate_db, &state_updates);
        let state_updates = Self::generate_diff(self.substate_db, state_updates);

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

    pub fn summarize_update(
        substate_db: &dyn SubstateDatabase,
        state_updates: &IndexMap<SubstateId, (PersistedSubstate, Option<u32>)>,
    ) -> StateUpdateSummary {
        let mut new_packages = index_set_new();
        let mut new_components = index_set_new();
        let mut new_resources = index_set_new();
        for (k, v) in state_updates {
            if v.1.is_none() {
                match k.0 {
                    NodeId::GlobalObject(Address::Package(address)) => {
                        new_packages.insert(address);
                    }
                    NodeId::GlobalObject(Address::Component(address)) => {
                        new_components.insert(address);
                    }
                    NodeId::GlobalObject(Address::Resource(address)) => {
                        new_resources.insert(address);
                    }
                    _ => {}
                }
            }
        }

        let (balance_changes, direct_vault_updates) =
            BalanceChangeAccounting::new(substate_db, &state_updates).run();

        StateUpdateSummary {
            new_packages: new_packages.into_iter().collect(),
            new_components: new_components.into_iter().collect(),
            new_resources: new_resources.into_iter().collect(),
            balance_changes,
            direct_vault_updates,
        }
    }

    pub fn generate_diff(
        substate_db: &dyn SubstateDatabase,
        state_updates: IndexMap<SubstateId, (PersistedSubstate, Option<u32>)>,
    ) -> StateUpdates {
        let mut diff = StateUpdates::new();

        for (substate_id, (substate, ..)) in state_updates {
            let next_version = if let Some(existing_output_id) =
                Self::loaded_substate_output_id(substate_db, &substate_id)
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

        diff
    }

    fn get_substate_output_id(
        substate_db: &dyn SubstateDatabase,
        substate_id: &SubstateId,
    ) -> Option<OutputId> {
        substate_db.get_substate(&substate_id).map(|s| OutputId {
            substate_id: substate_id.clone(),
            substate_hash: hash(
                scrypto_encode(&s.substate).expect("Saved substate couldn't be re-encoded"),
            ),
            version: s.version,
        })
    }
}
