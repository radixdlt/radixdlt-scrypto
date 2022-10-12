use indexmap::IndexMap;
use transaction::model::Executable;

use crate::engine::AppStateTrack;
use crate::engine::BaseStateTrack;
use crate::engine::*;
use crate::fee::FeeReserve;
use crate::fee::FeeReserveError;
use crate::fee::FeeSummary;
use crate::fee::FeeTable;
use crate::ledger::*;
use crate::model::node_to_substates;
use crate::model::nodes_to_substates;
use crate::model::LockableResource;
use crate::model::NonFungibleSubstate;
use crate::model::Resource;
use crate::model::RuntimeSubstate;
use crate::model::Vault;
use crate::model::VaultSubstate;
use crate::model::{KeyValueStoreEntrySubstate, PersistedSubstate};
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

    pub fn is_free(&self) -> bool {
        matches!(self, LockState::Read(0))
    }
}

#[derive(Debug)]
pub enum SubstateCache {
    Free(RuntimeSubstate),
    Taken,
}

impl SubstateCache {
    pub fn borrow(&self) -> &RuntimeSubstate {
        match self {
            Self::Free(substate) => substate,
            Self::Taken => {
                panic!("Attempted to borrow already taken substate")
            }
        }
    }

    pub fn borrow_mut(&mut self) -> &mut RuntimeSubstate {
        match self {
            Self::Free(substate) => substate,
            Self::Taken => {
                panic!("Attempted to borrow already taken substate")
            }
        }
    }

    pub fn take(&mut self) -> RuntimeSubstate {
        match core::mem::replace(self, SubstateCache::Taken) {
            Self::Free(substate) => substate,
            Self::Taken => {
                panic!("Attempted to take already taken substate")
            }
        }
    }

    pub fn put(&mut self, substate: RuntimeSubstate) {
        *self = SubstateCache::Free(substate);
    }

    pub fn is_taken(&self) -> bool {
        matches!(self, SubstateCache::Taken)
    }
}

#[derive(Debug)]
pub struct LoadedSubstate {
    pub substate: SubstateCache,
    pub lock_state: LockState,
}

/// Transaction-wide states and side effects
pub struct Track<'s, R: FeeReserve> {
    application_logs: Vec<(Level, String)>,
    state_track: AppStateTrack<'s>,
    loaded_substates: IndexMap<SubstateId, LoadedSubstate>,
    loaded_nodes: IndexMap<RENodeId, HeapRENode>,
    pub new_global_addresses: Vec<GlobalAddress>,
    pub fee_reserve: R,
    pub fee_table: FeeTable,
    pub vault_ops: Vec<(REActor, VaultId, VaultOp)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, TypeId)]
pub enum TrackError {
    NotFound(SubstateId),
    SubstateLocked(SubstateId, LockState),
    AlreadyLoaded(SubstateId),
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
        let base_state_track = BaseStateTrack::new(substate_store);
        let state_track = AppStateTrack::new(base_state_track);

        Self {
            application_logs: Vec::new(),
            state_track,
            loaded_substates: IndexMap::new(),
            loaded_nodes: IndexMap::new(),
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
        mutable: bool,
        write_through: bool,
    ) -> Result<(), TrackError> {
        if write_through && self.loaded_substates.contains_key(&substate_id) {
            return Err(TrackError::AlreadyLoaded(substate_id));
        }

        // Load the substate from state track
        if !self.loaded_substates.contains_key(&substate_id) {
            let maybe_substate = if write_through {
                self.state_track.get_substate_from_base(&substate_id)
            } else {
                self.state_track.get_substate(&substate_id)
            };

            if let Some(substate) = maybe_substate {
                self.loaded_substates.insert(
                    substate_id.clone(),
                    LoadedSubstate {
                        substate: SubstateCache::Free(substate.to_runtime()),
                        lock_state: LockState::no_lock(),
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
        match loaded_substate.lock_state {
            LockState::Read(n) => {
                if mutable {
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
        write_through: bool,
    ) -> Result<(), TrackError> {
        let mut loaded_substate = self
            .loaded_substates
            .remove(&substate_id)
            .expect("Attempted to release lock on never borrowed substate");

        match &loaded_substate.lock_state {
            LockState::Read(n) => loaded_substate.lock_state = LockState::Read(n - 1),
            LockState::Write => loaded_substate.lock_state = LockState::no_lock(),
        }

        if write_through {
            let node_id = match substate_id {
                SubstateId(
                    RENodeId::Vault(vault_id),
                    SubstateOffset::Vault(VaultOffset::Vault),
                ) => RENodeId::Vault(vault_id),
                _ => panic!("Not supported yet"),
            };
            let node = self.loaded_nodes.remove(&node_id).unwrap();
            for (offset, substate) in node_to_substates(node) {
                self.state_track
                    .put_substate_to_base(SubstateId(node_id, offset), substate.to_persisted());
            }
        } else {
            self.loaded_substates.insert(substate_id, loaded_substate);
        }
        Ok(())
    }

    fn create_node_if_missing(&mut self, node_id: &RENodeId) {
        if !self.loaded_nodes.contains_key(node_id) {
            match node_id {
                RENodeId::AuthZone(_)
                | RENodeId::Bucket(_)
                | RENodeId::Proof(_)
                | RENodeId::Global(..)
                | RENodeId::KeyValueStore(_)
                | RENodeId::NonFungibleStore(_)
                | RENodeId::Component(..)
                | RENodeId::ResourceManager(..)
                | RENodeId::Package(..)
                | RENodeId::System(..)
                | RENodeId::Worktop => panic!("Unexpected"),
                RENodeId::Vault(..) => {
                    let offset = SubstateOffset::Vault(VaultOffset::Vault);
                    let substate: VaultSubstate =
                        self.take_substate(SubstateId(*node_id, offset)).into();
                    let node = HeapRENode::Vault(Vault::new(substate.0));
                    self.loaded_nodes.insert(node_id.clone(), node);
                }
            }
        }
    }

    // TODO: Clean this up!
    // Despite being named as borrow_*, borrow rules are not enforced here but within `acquire_lock`.

    pub fn borrow_node(&mut self, node_id: &RENodeId) -> &HeapRENode {
        self.create_node_if_missing(node_id);
        self.loaded_nodes.get(node_id).expect("Node not available")
    }

    pub fn borrow_node_mut(&mut self, node_id: &RENodeId) -> &mut HeapRENode {
        self.create_node_if_missing(node_id);
        self.loaded_nodes
            .get_mut(node_id)
            .expect("Node not available")
    }

    pub fn put_node(&mut self, node_id: RENodeId, node: HeapRENode) {
        self.loaded_nodes.insert(node_id, node);
    }

    pub fn borrow_substate(&self, node_id: RENodeId, offset: SubstateOffset) -> &RuntimeSubstate {
        let substate_id = SubstateId(node_id, offset);
        self.loaded_substates
            .get(&substate_id)
            .expect(&format!("Substate {:?} was never locked", substate_id))
            .substate
            .borrow()
    }

    pub fn borrow_substate_mut(
        &mut self,
        node_id: RENodeId,
        offset: SubstateOffset,
    ) -> &mut RuntimeSubstate {
        let substate_id = SubstateId(node_id, offset);
        self.loaded_substates
            .get_mut(&substate_id)
            .expect(&format!("Substate {:?} was never locked", substate_id))
            .substate
            .borrow_mut()
    }

    pub fn take_substate(&mut self, substate_id: SubstateId) -> RuntimeSubstate {
        self.loaded_substates
            .get_mut(&substate_id)
            .expect(&format!("Substate {:?} was never locked", substate_id))
            .substate
            .take()
    }

    // TODO remove
    // Currently used by node globalization
    pub fn put_substate(&mut self, substate_id: SubstateId, substate: RuntimeSubstate) {
        if !self.loaded_substates.contains_key(&substate_id) {
            self.loaded_substates.insert(
                substate_id.clone(),
                LoadedSubstate {
                    substate: SubstateCache::Free(substate),
                    lock_state: LockState::no_lock(),
                },
            );
        } else {
            self.loaded_substates
                .get_mut(&substate_id)
                .unwrap()
                .substate = SubstateCache::Free(substate);
        }
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
                            substate: SubstateCache::Free(substate),
                            lock_state: LockState::no_lock(),
                        },
                    );
                }

                self.loaded_substates
                    .get(&substate_id)
                    .unwrap()
                    .substate
                    .borrow()
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
                            substate: SubstateCache::Free(substate),
                            lock_state: LockState::no_lock(),
                        },
                    );
                }

                self.loaded_substates
                    .get(&substate_id)
                    .unwrap()
                    .substate
                    .borrow()
            }
            _ => panic!("Invalid keyed value address {:?}", parent_address),
        }
    }

    pub fn read_key_value_mut(
        &mut self,
        parent_address: SubstateId,
        key: Vec<u8>,
    ) -> &mut RuntimeSubstate {
        // TODO: consider using a single address as function input
        let substate_id = match parent_address {
            SubstateId(
                RENodeId::NonFungibleStore(non_fungible_store_id),
                SubstateOffset::NonFungibleStore(NonFungibleStoreOffset::Space),
            ) => SubstateId(
                RENodeId::NonFungibleStore(non_fungible_store_id),
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
                            substate: SubstateCache::Free(substate),
                            lock_state: LockState::no_lock(),
                        },
                    );
                }

                self.loaded_substates
                    .get_mut(&substate_id)
                    .unwrap()
                    .substate
                    .borrow_mut()
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
                            substate: SubstateCache::Free(substate),
                            lock_state: LockState::no_lock(),
                        },
                    );
                }

                self.loaded_substates
                    .get_mut(&substate_id)
                    .unwrap()
                    .substate
                    .borrow_mut()
            }
            _ => panic!("Invalid keyed value address {:?}", parent_address),
        }
    }

    /// Sets a key value
    pub fn set_key_value<V: Into<RuntimeSubstate>>(
        &mut self,
        parent_substate_id: SubstateId,
        key: Vec<u8>,
        value: V,
    ) {
        // TODO: consider using a single address as function input
        let substate_id = match parent_substate_id {
            SubstateId(
                RENodeId::NonFungibleStore(resource_address),
                SubstateOffset::NonFungibleStore(NonFungibleStoreOffset::Space),
            ) => SubstateId(
                RENodeId::NonFungibleStore(resource_address),
                SubstateOffset::NonFungibleStore(NonFungibleStoreOffset::Entry(NonFungibleId(
                    key.clone(),
                ))),
            ),
            SubstateId(
                RENodeId::KeyValueStore(kv_store_id),
                SubstateOffset::KeyValueStore(KeyValueStoreOffset::Space),
            ) => SubstateId(
                RENodeId::KeyValueStore(kv_store_id),
                SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(key.clone())),
            ),
            _ => panic!("Unsupported key value"),
        };

        if let Some(loaded) = self.loaded_substates.get_mut(&substate_id) {
            loaded.substate = SubstateCache::Free(value.into());
        } else {
            let substate = value.into();
            self.state_track
                .put_substate(substate_id, substate.to_persisted());
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

        // Commit/rollback application state changes
        if is_success {
            for (id, loaded) in self.loaded_substates {
                if let SubstateCache::Free(substate) = loaded.substate {
                    self.state_track.put_substate(id, substate.to_persisted());
                }
            }

            for (id, substate) in nodes_to_substates(self.loaded_nodes.into_iter().collect()) {
                self.state_track.put_substate(id, substate.to_persisted());
            }

            self.state_track.commit();
        } else {
            self.state_track.rollback();
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
                    .get_substate_from_base(&substate_id)
                    .expect("Failed to fetch a fee-locking vault")
                    .to_runtime();
                substate
                    .vault_mut()
                    .0
                    .put(locked)
                    .expect("Failed to put a fee-locking vault");
                self.state_track
                    .put_substate_to_base(substate_id, substate.to_persisted());

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
                state_updates: self.state_track.into_base().generate_diff(),
                entity_changes: EntityChanges::new(self.new_global_addresses),
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
