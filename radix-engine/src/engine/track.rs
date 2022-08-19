use crate::engine::AppStateTrack;
use crate::engine::BaseStateTrack;
use crate::engine::StateTrackError;
use crate::engine::*;
use crate::ledger::*;
use crate::model::KeyValueStoreEntryWrapper;
use crate::model::NonFungibleWrapper;
use crate::state_manager::StateDiff;
use crate::types::*;

#[derive(Debug)]
pub enum BorrowedSubstate {
    Loaded(Substate, u32),
    LoadedMut(Substate),
    Taken,
}

impl BorrowedSubstate {
    fn loaded(value: Substate, mutable: bool) -> Self {
        if mutable {
            BorrowedSubstate::LoadedMut(value)
        } else {
            BorrowedSubstate::Loaded(value, 1)
        }
    }
}

/// Enforces borrow semantics of global objects and collects transaction-wise side effects,
/// such as logs and events.
pub struct Track<'s> {
    application_logs: Vec<(Level, String)>,
    new_substates: Vec<SubstateId>,
    state_track: AppStateTrack<'s>,
    borrowed_substates: HashMap<SubstateId, BorrowedSubstate>,
}

#[derive(Debug)]
pub enum TrackError {
    Reentrancy,
    NotFound,
    StateTrackError(StateTrackError),
}

pub struct TrackReceipt {
    pub new_addresses: Vec<SubstateId>,
    pub application_logs: Vec<(Level, String)>,
    pub state_updates: StateDiff,
}

impl<'s> Track<'s> {
    pub fn new(substate_store: &'s dyn ReadableSubstateStore) -> Self {
        let base_state_track = BaseStateTrack::new(substate_store);
        let state_track = AppStateTrack::new(base_state_track);

        Self {
            application_logs: Vec::new(),
            new_substates: Vec::new(),
            state_track,
            borrowed_substates: HashMap::new(),
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
        if let Some(current) = self.borrowed_substates.get_mut(&substate_id) {
            if mutable {
                return Err(TrackError::Reentrancy);
            } else {
                match current {
                    BorrowedSubstate::Taken | BorrowedSubstate::LoadedMut(..) => {
                        panic!("Should never get here")
                    }
                    BorrowedSubstate::Loaded(_, ref mut count) => *count = *count + 1,
                }
                return Ok(());
            }
        }

        if write_through {
            let value = self
                .state_track
                .get_substate_from_base(&substate_id)
                .map_err(TrackError::StateTrackError)?
                .ok_or(TrackError::NotFound)?;
            self.borrowed_substates.insert(
                substate_id.clone(),
                BorrowedSubstate::loaded(value, mutable),
            );
            Ok(())
        } else {
            if let Some(substate) = self.state_track.get_substate(&substate_id) {
                let substate = match substate_id {
                    SubstateId::ComponentInfo(..)
                    | SubstateId::ResourceManager(..)
                    | SubstateId::Vault(..)
                    | SubstateId::Package(..)
                    | SubstateId::ComponentState(..)
                    | SubstateId::System => substate,
                    _ => panic!(
                        "Attempting to borrow unsupported substate {:?}",
                        substate_id
                    ),
                };

                self.borrowed_substates.insert(
                    substate_id.clone(),
                    BorrowedSubstate::loaded(substate, mutable),
                );
                Ok(())
            } else {
                Err(TrackError::NotFound)
            }
        }
    }

    pub fn release_lock(&mut self, substate_id: SubstateId, write_through: bool) {
        let borrowed = self
            .borrowed_substates
            .remove(&substate_id)
            .expect("Attempted to release lock on never borrowed substate");

        if write_through {
            match borrowed {
                BorrowedSubstate::Taken => panic!("Value was never returned"),
                BorrowedSubstate::LoadedMut(value) => {
                    self.state_track.put_substate_to_base(substate_id, value);
                }
                BorrowedSubstate::Loaded(value, mut count) => {
                    count = count - 1;
                    if count == 0 {
                        self.state_track.put_substate_to_base(substate_id, value);
                    } else {
                        self.borrowed_substates
                            .insert(substate_id, BorrowedSubstate::Loaded(value, count));
                    }
                }
            }
        } else {
            match borrowed {
                BorrowedSubstate::Taken => panic!("Value was never returned"),
                BorrowedSubstate::LoadedMut(value) => {
                    self.state_track.put_substate(substate_id, value);
                }
                BorrowedSubstate::Loaded(value, mut count) => {
                    count = count - 1;
                    if count == 0 {
                        self.state_track.put_substate(substate_id, value);
                    } else {
                        self.borrowed_substates
                            .insert(substate_id, BorrowedSubstate::Loaded(value, count));
                    }
                }
            }
        }
    }

    pub fn read_substate(&self, substate_id: SubstateId) -> &Substate {
        match self
            .borrowed_substates
            .get(&substate_id)
            .expect(&format!("Substate {:?} was never locked", substate_id))
        {
            BorrowedSubstate::LoadedMut(substate) => substate,
            BorrowedSubstate::Loaded(substate, ..) => substate,
            BorrowedSubstate::Taken => panic!("Substate was already taken"),
        }
    }

    pub fn take_substate(&mut self, substate_id: SubstateId) -> Substate {
        match self
            .borrowed_substates
            .insert(substate_id.clone(), BorrowedSubstate::Taken)
            .expect(&format!("Substate {:?} was never locked", substate_id))
        {
            BorrowedSubstate::LoadedMut(value) => value,
            BorrowedSubstate::Loaded(..) => {
                panic!("Cannot take value on immutable: {:?}", substate_id)
            }
            BorrowedSubstate::Taken => panic!("Substate was already taken"),
        }
    }

    pub fn write_substate<V: Into<Substate>>(&mut self, substate_id: SubstateId, value: V) {
        let cur_value = self
            .borrowed_substates
            .get(&substate_id)
            .expect(&format!("Substate {:?} was never locked", substate_id));
        match cur_value {
            BorrowedSubstate::Loaded(..) => panic!("Cannot write to immutable"),
            BorrowedSubstate::LoadedMut(..) | BorrowedSubstate::Taken => {}
        }

        self.borrowed_substates
            .insert(substate_id, BorrowedSubstate::LoadedMut(value.into()));
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
                .unwrap_or(Substate::NonFungible(NonFungibleWrapper(None))),
            SubstateId::KeyValueStoreSpace(..) => self
                .state_track
                .get_substate(&substate_id)
                .unwrap_or(Substate::KeyValueStoreEntry(KeyValueStoreEntryWrapper(
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

    pub fn commit(&mut self) {
        self.state_track.commit();
    }

    pub fn rollback(&mut self) {
        self.state_track.rollback();

        // self.application_logs.clear();
        self.new_substates.clear();
        self.borrowed_substates.clear();
    }

    pub fn to_receipt(self) -> TrackReceipt {
        TrackReceipt {
            new_addresses: self.new_substates,
            application_logs: self.application_logs,
            state_updates: self.state_track.into_base().generate_diff(),
        }
    }
}
