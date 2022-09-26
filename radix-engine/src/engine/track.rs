use transaction::model::Validated;

use crate::engine::AppStateTrack;
use crate::engine::BaseStateTrack;
use crate::engine::*;
use crate::fee::FeeReserve;
use crate::fee::FeeReserveError;
use crate::fee::FeeSummary;
use crate::fee::FeeTable;
use crate::ledger::*;
use crate::model::KeyValueStoreEntrySubstate;
use crate::model::LockableResource;
use crate::model::NonFungibleSubstate;
use crate::model::Resource;
use crate::model::Vault;
use crate::model::VaultSubstate;
use crate::transaction::CommitResult;
use crate::transaction::EntityChanges;
use crate::transaction::RejectResult;
use crate::transaction::TransactionOutcome;
use crate::transaction::TransactionResult;
use crate::types::*;

#[derive(Debug)]
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
    Raw(Substate),
    Converted(Vault),
}

// TODO: explore the following options
// 1. Make it an invariant that every node must be persistable at the end of a transaction, so no need of this error.
// 2. Make `Track` more dynamic and allow nodes to define whether it's ready for persistence.
// 3. Make transient property part of substate rather than node.
#[derive(Debug, Encode, Decode, TypeId)]
pub enum NodeToSubstateFailure {
    VaultPartiallyLocked,
}

impl SubstateCache {
    pub fn raw(&self) -> &Substate {
        match self {
            Self::Raw(substate) => substate,
            Self::Converted(_) => {
                panic!("Attempted to read a raw substate which has been converted into a node")
            }
        }
    }

    pub fn raw_mut(&mut self) -> &mut Substate {
        match self {
            Self::Raw(substate) => substate,
            Self::Converted(_) => {
                panic!("Attempted to read a raw substate which has been converted into a node")
            }
        }
    }

    // Turns substate into a node for dynamic behaviors
    pub fn convert_to_node(&mut self) {
        match self {
            SubstateCache::Raw(substate) => {
                let substate: VaultSubstate = substate.clone().into();
                *self = Self::Converted(Vault::new(substate.0));
            }
            SubstateCache::Converted(_) => {}
        }
    }

    pub fn convert_to_substate(self) -> Result<Substate, NodeToSubstateFailure> {
        match self {
            SubstateCache::Raw(substate) => Ok(substate),
            SubstateCache::Converted(vault) => {
                let resource = vault
                    .resource()
                    .map_err(|_| NodeToSubstateFailure::VaultPartiallyLocked)?;
                Ok(Substate::Vault(VaultSubstate(resource)))
            }
        }
    }

    pub fn vault(&mut self) -> &Vault {
        match self {
            Self::Raw(_) => panic!("Attempted to read a raw substate as a node"),
            Self::Converted(vault) => vault,
        }
    }

    pub fn vault_mut(&mut self) -> &mut Vault {
        match self {
            Self::Raw(_) => panic!("Attempted to read a raw substate as a node"),
            Self::Converted(vault) => vault,
        }
    }
}

#[derive(Debug)]
pub struct BorrowedSubstate {
    pub substate: SubstateCache,
    pub lock_state: LockState,
}

/// Transaction-wide states and side effects
pub struct Track<'s, R: FeeReserve> {
    application_logs: Vec<(Level, String)>,
    new_substates: Vec<SubstateId>,
    state_track: AppStateTrack<'s>,
    borrowed_substates: HashMap<SubstateId, BorrowedSubstate>,
    pub fee_reserve: R,
    pub fee_table: FeeTable,
}

#[derive(Debug, Encode, Decode, TypeId)]
pub enum TrackError {
    NotFound(SubstateId),
    NotAvailable(SubstateId),
    AlreadyLoaded(SubstateId),
    NodeToSubstateFailure(NodeToSubstateFailure),
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
            new_substates: Vec::new(),
            state_track,
            borrowed_substates: HashMap::new(),
            fee_reserve,
            fee_table,
        }
    }

    /// Adds a log message.
    pub fn add_log(&mut self, level: Level, message: String) {
        self.application_logs.push((level, message));
    }

    /// Creates a row with the given key/value
    pub fn create_uuid_substate<V: Into<Substate>>(
        &mut self,
        substate_id: SubstateId,
        value: V,
        is_root: bool,
    ) {
        self.new_substates.push(substate_id.clone());
        self.state_track
            .put_substate(substate_id.clone(), value.into());
        if is_root {
            self.state_track.set_substate_root(substate_id);
        }
    }

    // TODO: Clean this up
    pub fn is_root(&mut self, substate_id: &SubstateId) -> bool {
        self.state_track.is_root(substate_id)
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
        if write_through && self.borrowed_substates.contains_key(&substate_id) {
            return Err(TrackError::AlreadyLoaded(substate_id));
        }

        // Load the substate from state track
        if !self.borrowed_substates.contains_key(&substate_id) {
            let maybe_substate = if write_through {
                self.state_track.get_substate_from_base(&substate_id)
            } else {
                self.state_track.get_substate(&substate_id)
            };

            if let Some(substate) = maybe_substate {
                self.borrowed_substates.insert(
                    substate_id.clone(),
                    BorrowedSubstate {
                        substate: SubstateCache::Raw(substate),
                        lock_state: LockState::no_lock(),
                    },
                );
            } else {
                return Err(TrackError::NotFound(substate_id));
            }
        }

        let borrowed = self
            .borrowed_substates
            .get_mut(&substate_id)
            .expect("Existence checked upfront");
        match borrowed.lock_state {
            LockState::Read(n) => {
                if mutable {
                    if n != 0 {
                        return Err(TrackError::NotAvailable(substate_id));
                    }
                    borrowed.lock_state = LockState::Write;
                } else {
                    borrowed.lock_state = LockState::Read(n + 1);
                }
            }
            LockState::Write => {
                return Err(TrackError::NotAvailable(substate_id));
            }
        }

        Ok(())
    }

    pub fn release_lock(
        &mut self,
        substate_id: SubstateId,
        write_through: bool,
    ) -> Result<(), TrackError> {
        let mut borrowed = self
            .borrowed_substates
            .remove(&substate_id)
            .expect("Attempted to release lock on never borrowed substate");

        match &borrowed.lock_state {
            LockState::Read(n) => borrowed.lock_state = LockState::Read(n - 1),
            LockState::Write => borrowed.lock_state = LockState::no_lock(),
        }

        if write_through && borrowed.lock_state.is_free() {
            self.state_track.put_substate_to_base(
                substate_id,
                borrowed
                    .substate
                    .convert_to_substate()
                    .map_err(TrackError::NodeToSubstateFailure)?,
            );
        } else {
            self.borrowed_substates.insert(substate_id, borrowed);
        }
        Ok(())
    }

    pub fn borrow_substate(&self, substate_id: SubstateId) -> &SubstateCache {
        &self
            .borrowed_substates
            .get(&substate_id)
            .expect(&format!("Substate {:?} was never locked", substate_id))
            .substate
    }

    pub fn borrow_substate_mut(&mut self, substate_id: SubstateId) -> &mut SubstateCache {
        &mut self
            .borrowed_substates
            .get_mut(&substate_id)
            .expect(&format!("Substate {:?} was never locked", substate_id))
            .substate
    }

    pub fn take_substate(&mut self, substate_id: SubstateId) -> BorrowedSubstate {
        self.borrowed_substates
            .remove(&substate_id)
            .expect(&format!("Substate {:?} was never locked", substate_id))
    }

    pub fn return_substate(&mut self, substate_id: SubstateId, substate: BorrowedSubstate) {
        self.borrowed_substates.insert(substate_id, substate);
    }

    /// Returns the value of a key value pair
    pub fn read_key_value(&mut self, parent_address: SubstateId, key: Vec<u8>) -> Substate {
        // TODO: consider using a single address as function input
        let substate_id = match parent_address {
            SubstateId::NonFungibleSpace(resource_address) => {
                SubstateId::NonFungible(resource_address, NonFungibleId(key))
            }
            SubstateId::KeyValueStoreSpace(kv_store_id) => {
                SubstateId::KeyValueStoreEntry(kv_store_id, key)
            }
            _ => panic!("Unsupported key value"),
        };

        match parent_address {
            SubstateId::NonFungibleSpace(_) => self
                .state_track
                .get_substate(&substate_id)
                .unwrap_or(Substate::NonFungible(NonFungibleSubstate(None))),
            SubstateId::KeyValueStoreSpace(..) => self
                .state_track
                .get_substate(&substate_id)
                .unwrap_or(Substate::KeyValueStoreEntry(KeyValueStoreEntrySubstate(
                    None,
                ))),
            _ => panic!("Invalid keyed value address {:?}", parent_address),
        }
    }

    /// Sets a key value
    pub fn set_key_value<V: Into<Substate>>(
        &mut self,
        parent_substate_id: SubstateId,
        key: Vec<u8>,
        value: V,
    ) {
        // TODO: consider using a single address as function input
        let substate_id = match parent_substate_id {
            SubstateId::NonFungibleSpace(resource_address) => {
                SubstateId::NonFungible(resource_address, NonFungibleId(key.clone()))
            }
            SubstateId::KeyValueStoreSpace(kv_store_id) => {
                SubstateId::KeyValueStoreEntry(kv_store_id, key.clone())
            }
            _ => panic!("Unsupported key value"),
        };

        self.state_track.put_substate(substate_id, value.into());
    }

    pub fn apply_pre_execution_costs(
        mut self,
        transaction: &Validated,
    ) -> Result<Self, PreExecutionError> {
        let result = self
            .fee_reserve
            .consume(self.fee_table.tx_base_fee(), "base_fee", false)
            .and_then(|()| {
                self.fee_reserve.consume(
                    self.fee_table.tx_manifest_decoding_per_byte()
                        * transaction.manifest_instructions_size() as u32,
                    "decode_manifest",
                    false,
                )
            })
            .and_then(|()| {
                self.fee_reserve.consume(
                    self.fee_table.tx_manifest_verification_per_byte()
                        * transaction.manifest_instructions_size() as u32,
                    "verify_manifest",
                    false,
                )
            })
            .and_then(|()| {
                self.fee_reserve.consume(
                    self.fee_table.tx_signature_verification_per_sig()
                        * transaction.initial_proofs().len() as u32,
                    "verify_signatures",
                    false,
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

    pub fn finalize(
        mut self,
        invoke_result: Result<Vec<Vec<u8>>, RuntimeError>,
        resource_changes: Vec<ResourceChange>, // TODO: wrong abstraction, resource change should be derived from track instead of kernel
    ) -> TrackReceipt {
        let is_success = invoke_result.is_ok();

        // Flush all borrowed substates to state track
        if is_success {
            for (substate_id, borrowed) in self.borrowed_substates.drain() {
                let substate = borrowed.substate.convert_to_substate().expect(
                    "Invariant: at the end of transaction, all borrowed substate should be ready for persisting"
                );
                self.state_track.put_substate(substate_id, substate);
            }
        }

        // Commit/rollback application state changes
        if is_success {
            self.state_track.commit();
        } else {
            self.state_track.rollback();
            self.borrowed_substates.clear();
            self.new_substates.clear();
        }

        // Close fee reserve
        let fee_summary = self.fee_reserve.finalize();
        let is_rejection = !fee_summary.loan_fully_repaid;

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
                let substate_id = SubstateId::Vault(vault_id);
                let mut substate = self
                    .state_track
                    .get_substate_from_base(&substate_id)
                    .expect("Failed to fetch a fee-locking vault");
                substate
                    .vault_mut()
                    .0
                    .put(locked)
                    .expect("Failed to put a fee-locking vault");
                self.state_track.put_substate_to_base(substate_id, substate);
            }

            // TODO: update XRD supply or disable it
            // TODO: pay tips to the lead validator

            let mut new_component_addresses = Vec::new();
            let mut new_resource_addresses = Vec::new();
            let mut new_package_addresses = Vec::new();
            for substate_id in self.new_substates {
                match substate_id {
                    SubstateId::ComponentInfo(component_address) => {
                        new_component_addresses.push(component_address)
                    }
                    SubstateId::ResourceManager(resource_address) => {
                        new_resource_addresses.push(resource_address)
                    }
                    SubstateId::Package(package_address) => {
                        new_package_addresses.push(package_address)
                    }
                    _ => {}
                }
            }

            TransactionResult::Commit(CommitResult {
                outcome: match invoke_result {
                    Ok(output) => TransactionOutcome::Success(output),
                    Err(error) => TransactionOutcome::Failure(error),
                },
                state_updates: self.state_track.into_base().generate_diff(),
                entity_changes: EntityChanges {
                    new_package_addresses,
                    new_component_addresses,
                    new_resource_addresses,
                },
                resource_changes,
            })
        };

        TrackReceipt {
            fee_summary,
            application_logs: self.application_logs,
            result,
        }
    }
}
