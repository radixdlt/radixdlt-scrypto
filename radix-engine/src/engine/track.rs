use indexmap::IndexMap;
use transaction::model::Executable;

use crate::engine::StateTrack;
use crate::engine::*;
use crate::fee::FeeReserve;
use crate::fee::FeeReserveError;
use crate::fee::FeeSummary;
use crate::fee::FeeTable;
use crate::ledger::*;
use crate::model::NonFungibleSubstate;
use crate::model::Resource;
use crate::model::RuntimeSubstate;
use crate::model::{KeyValueStoreEntrySubstate, PersistedSubstate};
use crate::model::{LockableResource, SubstateRef};
use crate::transaction::CommitResult;
use crate::transaction::EntityChanges;
use crate::transaction::RejectResult;
use crate::transaction::TransactionOutcome;
use crate::transaction::TransactionResult;
use crate::types::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Encode, Decode, TypeId)]
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
pub enum SubstateMetaState {
    New,
    Updated,
    Loaded,
}

#[derive(Debug)]
pub struct LoadedSubstate {
    pub substate: RuntimeSubstate,
    pub lock_state: LockState,
    pub metastate: SubstateMetaState,
}

/// Transaction-wide states and side effects
pub struct Track<'s, R: FeeReserve> {
    application_logs: Vec<(Level, String)>,
    state_track: StateTrack<'s>,
    loaded_substates: IndexMap<SubstateId, LoadedSubstate>,
    pub fee_reserve: R,
    pub fee_table: FeeTable,
    pub vault_ops: Vec<(REActor, VaultId, VaultOp)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, TypeId)]
pub enum TrackError {
    NotFound(SubstateId),
    SubstateLocked(SubstateId, LockState),
    LockUnmodifiedBaseOnNewSubstate(SubstateId),
    LockUnmodifiedBaseOnOnUpdatedSubstate(SubstateId),
}

pub struct TrackReceipt {
    pub fee_summary: FeeSummary,
    pub application_logs: Vec<(Level, String)>,
    pub result: TransactionResult,
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
        let state_track = StateTrack::new(substate_store);

        Self {
            application_logs: Vec::new(),
            state_track,
            loaded_substates: IndexMap::new(),
            fee_reserve,
            fee_table,
            vault_ops: Vec::new(),
        }
    }

    /// Adds a log message.
    pub fn add_log(&mut self, level: Level, message: String) {
        self.application_logs.push((level, message));
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
            let maybe_substate = self.state_track.get_substate(&substate_id);
            if let Some(substate) = maybe_substate {
                self.loaded_substates.insert(
                    substate_id.clone(),
                    LoadedSubstate {
                        substate: substate.to_runtime(),
                        lock_state: LockState::no_lock(),
                        metastate: SubstateMetaState::Loaded,
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
                SubstateMetaState::Updated => {
                    return Err(TrackError::LockUnmodifiedBaseOnOnUpdatedSubstate(
                        substate_id,
                    ))
                }
                SubstateMetaState::Loaded => {}
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
                loaded_substate.metastate = SubstateMetaState::Updated;

                if force_write {
                    let persisted_substate = loaded_substate.substate.clone_to_persisted();
                    self.state_track
                        .put_substate(substate_id, persisted_substate);
                }
            }
        }

        Ok(())
    }

    /// Returns the value of a key value pair
    pub fn read_key_value(&mut self, parent_address: SubstateId, key: Vec<u8>) -> &RuntimeSubstate {
        // TODO: consider using a single address as function input
        let substate_id = match parent_address {
            SubstateId(
                RENodeId::NonFungibleStore(store_id),
                SubstateOffset::NonFungibleStore(NonFungibleStoreOffset::Space),
            ) => SubstateId(
                RENodeId::NonFungibleStore(store_id),
                SubstateOffset::NonFungibleStore(NonFungibleStoreOffset::Entry(NonFungibleId(key))),
            ),
            SubstateId(
                RENodeId::KeyValueStore(kv_store_id),
                SubstateOffset::KeyValueStore(KeyValueStoreOffset::Space),
            ) => SubstateId(
                RENodeId::KeyValueStore(kv_store_id),
                SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(key)),
            ),
            _ => panic!("Unsupported key value"),
        };

        match parent_address {
            SubstateId(RENodeId::NonFungibleStore(..), ..) => {
                if !self.loaded_substates.contains_key(&substate_id) {
                    let substate = self
                        .state_track
                        .get_substate(&substate_id)
                        .map(PersistedSubstate::to_runtime)
                        .unwrap_or(RuntimeSubstate::NonFungible(NonFungibleSubstate(None)));

                    self.loaded_substates.insert(
                        substate_id.clone(),
                        LoadedSubstate {
                            substate,
                            lock_state: LockState::no_lock(),
                            metastate: SubstateMetaState::Loaded,
                        },
                    );
                }

                &self.loaded_substates.get(&substate_id).unwrap().substate
            }
            SubstateId(RENodeId::KeyValueStore(..), ..) => {
                if !self.loaded_substates.contains_key(&substate_id) {
                    let substate = self
                        .state_track
                        .get_substate(&substate_id)
                        .map(PersistedSubstate::to_runtime)
                        .unwrap_or(RuntimeSubstate::KeyValueStoreEntry(
                            KeyValueStoreEntrySubstate(None),
                        ));

                    self.loaded_substates.insert(
                        substate_id.clone(),
                        LoadedSubstate {
                            substate,
                            lock_state: LockState::no_lock(),
                            metastate: SubstateMetaState::Loaded,
                        },
                    );
                }

                &self.loaded_substates.get(&substate_id).unwrap().substate
            }
            _ => panic!("Invalid keyed value address {:?}", parent_address),
        }
    }

    pub fn get_substate(&mut self, node_id: RENodeId, offset: &SubstateOffset) -> SubstateRef {
        let runtime_substate = match (node_id, offset) {
            (
                RENodeId::KeyValueStore(..),
                SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(key)),
            ) => {
                let parent_substate_id = SubstateId(
                    node_id,
                    SubstateOffset::KeyValueStore(KeyValueStoreOffset::Space),
                );
                self.read_key_value(parent_substate_id, key.to_vec())
            }
            (
                RENodeId::NonFungibleStore(..),
                SubstateOffset::NonFungibleStore(NonFungibleStoreOffset::Entry(non_fungible_id)),
            ) => {
                let parent_substate_id = SubstateId(
                    node_id,
                    SubstateOffset::NonFungibleStore(NonFungibleStoreOffset::Space),
                );
                self.read_key_value(parent_substate_id, non_fungible_id.to_vec())
            }
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

    pub fn borrow_substate_mut(
        &mut self,
        node_id: RENodeId,
        offset: SubstateOffset,
    ) -> &mut RuntimeSubstate {
        let substate_id = SubstateId(node_id, offset);
        &mut self
            .loaded_substates
            .get_mut(&substate_id)
            .expect(&format!("Substate {:?} was never locked", substate_id))
            .substate
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

    pub fn read_key_value_mut(
        &mut self,
        parent_address: SubstateId,
        key: Vec<u8>,
    ) -> &mut RuntimeSubstate {
        // TODO: consider using a single address as function input
        let substate_id = match parent_address {
            SubstateId(
                RENodeId::NonFungibleStore(nf_store_id),
                SubstateOffset::NonFungibleStore(NonFungibleStoreOffset::Space),
            ) => SubstateId(
                RENodeId::NonFungibleStore(nf_store_id),
                SubstateOffset::NonFungibleStore(NonFungibleStoreOffset::Entry(NonFungibleId(key))),
            ),
            SubstateId(
                RENodeId::KeyValueStore(kv_store_id),
                SubstateOffset::KeyValueStore(KeyValueStoreOffset::Space),
            ) => SubstateId(
                RENodeId::KeyValueStore(kv_store_id),
                SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(key)),
            ),
            _ => panic!("Unsupported key value"),
        };

        match parent_address {
            SubstateId(RENodeId::NonFungibleStore(..), ..) => {
                if !self.loaded_substates.contains_key(&substate_id) {
                    let substate = self
                        .state_track
                        .get_substate(&substate_id)
                        .map(PersistedSubstate::to_runtime)
                        .unwrap_or(RuntimeSubstate::NonFungible(NonFungibleSubstate(None)));

                    self.loaded_substates.insert(
                        substate_id.clone(),
                        LoadedSubstate {
                            substate,
                            lock_state: LockState::no_lock(),
                            metastate: SubstateMetaState::New,
                        },
                    );
                }

                &mut self
                    .loaded_substates
                    .get_mut(&substate_id)
                    .unwrap()
                    .substate
            }
            SubstateId(RENodeId::KeyValueStore(..), ..) => {
                if !self.loaded_substates.contains_key(&substate_id) {
                    let substate = self
                        .state_track
                        .get_substate(&substate_id)
                        .map(PersistedSubstate::to_runtime)
                        .unwrap_or(RuntimeSubstate::KeyValueStoreEntry(
                            KeyValueStoreEntrySubstate(None),
                        ));

                    self.loaded_substates.insert(
                        substate_id.clone(),
                        LoadedSubstate {
                            substate,
                            lock_state: LockState::no_lock(),
                            metastate: SubstateMetaState::New,
                        },
                    );
                }

                &mut self
                    .loaded_substates
                    .get_mut(&substate_id)
                    .unwrap()
                    .substate
            }
            _ => panic!("Invalid keyed value address {:?}", parent_address),
        }
    }

    pub fn apply_pre_execution_costs(
        mut self,
        transaction: &Executable,
    ) -> Result<Self, PreExecutionError> {
        let result = self
            .fee_reserve
            .consume(self.fee_table.tx_base_fee(), "base_fee", true)
            .and_then(|()| {
                self.fee_reserve.consume(
                    self.fee_table.tx_manifest_decoding_per_byte()
                        * transaction.manifest_instructions_size() as u32,
                    "decode_manifest",
                    true,
                )
            })
            .and_then(|()| {
                self.fee_reserve.consume(
                    self.fee_table.tx_manifest_verification_per_byte()
                        * transaction.manifest_instructions_size() as u32,
                    "verify_manifest",
                    true,
                )
            })
            .and_then(|()| {
                self.fee_reserve.consume(
                    self.fee_table.tx_signature_verification_per_sig()
                        * transaction.auth_zone_params().initial_proofs.len() as u32,
                    "verify_signatures",
                    true,
                )
            })
            .and_then(|()| {
                self.fee_reserve.consume(
                    transaction.blobs().iter().map(|b| b.len()).sum::<usize>() as u32
                        * self.fee_table.tx_blob_price_per_byte(),
                    "blobs",
                    true,
                )
            });

        match result {
            Ok(()) => Ok(self),
            Err(error) => Err(PreExecutionError {
                fee_summary: self.fee_reserve.finalize(),
                error,
            }),
        }
    }

    pub fn finalize(mut self, invoke_result: Result<Vec<Vec<u8>>, RuntimeError>) -> TrackReceipt {
        let is_success = invoke_result.is_ok();

        let mut new_global_addresses = Vec::new();

        // Commit/rollback application state changes
        if is_success {
            for (id, loaded) in self.loaded_substates {
                match (&id, &loaded) {
                    (
                        SubstateId(
                            RENodeId::Global(global_address),
                            SubstateOffset::Global(GlobalOffset::Global),
                        ),
                        LoadedSubstate {
                            metastate: SubstateMetaState::New,
                            ..
                        },
                    ) => {
                        new_global_addresses.push(*global_address);
                    }
                    _ => {}
                }

                self.state_track
                    .put_substate(id, loaded.substate.to_persisted());
            }
        } else {
            self.loaded_substates.clear();
        }

        // Close fee reserve
        let fee_summary = self.fee_reserve.finalize();
        let is_rejection = !fee_summary.loan_fully_repaid;

        let mut actual_fee_payments: HashMap<VaultId, Decimal> = HashMap::new();

        // Commit fee state changes
        let result = if is_rejection {
            TransactionResult::Reject(RejectResult {
                error: match invoke_result {
                    Ok(..) => RejectionError::SuccessButFeeLoanNotRepaid,
                    Err(error) => RejectionError::ErrorBeforeFeeLoanRepaid(error),
                },
            })
        } else {
            let mut required = fee_summary.burned + fee_summary.tipped;
            let mut collector: LockableResource =
                Resource::new_empty(RADIX_TOKEN, ResourceType::Fungible { divisibility: 18 })
                    .into();
            for (vault_id, mut locked, contingent) in fee_summary.payments.iter().cloned().rev() {
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
                collector
                    .put(
                        locked
                            .take_by_amount(amount)
                            .expect("Failed to extract locked fee"),
                    )
                    .expect("Failed to add fee to fee collector");

                // Refund overpayment
                let substate_id = SubstateId(
                    RENodeId::Vault(vault_id),
                    SubstateOffset::Vault(VaultOffset::Vault),
                );

                let mut substate = self
                    .state_track
                    .get_substate(&substate_id)
                    .expect("Failed to fetch a fee-locking vault")
                    .to_runtime();
                substate
                    .vault_mut()
                    .borrow_resource_mut()
                    .put(locked)
                    .expect("Failed to put a fee-locking vault");
                self.state_track
                    .put_substate(substate_id, substate.to_persisted());

                *actual_fee_payments.entry(vault_id).or_default() += amount;
            }
            let execution_trace_receipt = ExecutionTraceReceipt::new(
                self.vault_ops,
                actual_fee_payments,
                &mut self.state_track,
                invoke_result.is_ok(),
            );

            // TODO: update XRD supply or disable it
            // TODO: pay tips to the lead validator

            TransactionResult::Commit(CommitResult {
                outcome: match invoke_result {
                    Ok(output) => TransactionOutcome::Success(output),
                    Err(error) => TransactionOutcome::Failure(error),
                },
                state_updates: self.state_track.generate_diff(),
                entity_changes: EntityChanges::new(new_global_addresses),
                resource_changes: execution_trace_receipt.resource_changes,
            })
        };

        TrackReceipt {
            fee_summary,
            application_logs: self.application_logs,
            result,
        }
    }
}
