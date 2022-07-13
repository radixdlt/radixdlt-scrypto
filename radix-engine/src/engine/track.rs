use indexmap::{IndexMap, IndexSet};
use sbor::rust::collections::*;
use sbor::rust::format;
use sbor::rust::ops::RangeFull;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::*;
use scrypto::engine::types::*;

use crate::engine::track::BorrowedSubstate::Taken;
use crate::engine::{Address, Substate, SubstateOperation, SubstateOperationsReceipt};
use crate::ledger::*;

enum BorrowedSubstate {
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

/// Facilitates transactional state updates.
pub struct Track<'s, S: ReadableSubstateStore> {
    substate_store: &'s mut S,
    logs: Vec<(Level, String)>,

    new_addresses: Vec<Address>,
    borrowed_substates: HashMap<Address, BorrowedSubstate>,

    downed_substates: Vec<OutputId>,
    down_virtual_substates: Vec<VirtualSubstateId>,
    up_substates: IndexMap<Vec<u8>, Substate>,
    up_virtual_substate_space: IndexSet<Vec<u8>>,
}

#[derive(Debug)]
pub enum TrackError {
    Reentrancy,
    NotFound,
}

pub struct BorrowedSNodes {
    borrowed_substates: HashSet<Address>,
}

impl BorrowedSNodes {
    pub fn is_empty(&self) -> bool {
        self.borrowed_substates.is_empty()
    }
}

pub struct TrackReceipt {
    pub borrowed: BorrowedSNodes,
    pub new_addresses: Vec<Address>,
    pub logs: Vec<(Level, String)>,
    pub substates: SubstateOperationsReceipt,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubstateUpdate<T> {
    pub prev_id: Option<OutputId>,
    pub value: T,
}

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub enum SubstateParentId {
    Exists(OutputId),
    New(usize),
}

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct VirtualSubstateId(pub SubstateParentId, pub Vec<u8>);

impl<'s, S: ReadableSubstateStore> Track<'s, S> {
    pub fn new(substate_store: &'s mut S) -> Self {
        Self {
            substate_store,
            logs: Vec::new(),

            new_addresses: Vec::new(),
            borrowed_substates: HashMap::new(),

            downed_substates: Vec::new(),
            down_virtual_substates: Vec::new(),
            up_substates: IndexMap::new(),
            up_virtual_substate_space: IndexSet::new(),
        }
    }

    /// Adds a log message.
    pub fn add_log(&mut self, level: Level, message: String) {
        self.logs.push((level, message));
    }

    /// Creates a row with the given key/value
    pub fn create_uuid_value<A: Into<Address>, V: Into<Substate>>(&mut self, addr: A, value: V) {
        let address = addr.into();
        self.new_addresses.push(address.clone());
        self.up_substates.insert(address.encode(), value.into());
    }

    pub fn create_key_space(&mut self, address: Address) {
        self.up_virtual_substate_space.insert(address.encode());
    }

    pub fn take_lock<A: Into<Address>>(
        &mut self,
        addr: A,
        mutable: bool,
    ) -> Result<(), TrackError> {
        let address = addr.into();
        let maybe_value = self.up_substates.remove(&address.encode());
        if let Some(value) = maybe_value {
            self.borrowed_substates
                .insert(address, BorrowedSubstate::loaded(value, mutable));
            return Ok(());
        }

        if let Some(current) = self.borrowed_substates.get_mut(&address) {
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

        if let Some(substate) = self.substate_store.get_substate(&address.encode()) {
            self.downed_substates.push(substate.phys_id);
            self.borrowed_substates.insert(
                address.clone(),
                BorrowedSubstate::loaded(substate.value, mutable),
            );
            Ok(())
        } else {
            Err(TrackError::NotFound)
        }
    }

    pub fn read_value<A: Into<Address>>(&self, addr: A) -> &Substate {
        let address: Address = addr.into();
        match self
            .borrowed_substates
            .get(&address)
            .expect(&format!("{:?} was never locked", address))
        {
            BorrowedSubstate::LoadedMut(value) => value,
            BorrowedSubstate::Loaded(value, ..) => value,
            BorrowedSubstate::Taken => panic!("Value was already taken"),
        }
    }

    pub fn take_value<A: Into<Address>>(&mut self, addr: A) -> Substate {
        let address: Address = addr.into();
        match self
            .borrowed_substates
            .insert(address.clone(), Taken)
            .expect(&format!("{:?} was never locked", address))
        {
            BorrowedSubstate::LoadedMut(value) => value,
            BorrowedSubstate::Loaded(..) => panic!("Cannot take value on immutable: {:?}", address),
            BorrowedSubstate::Taken => panic!("Value was already taken"),
        }
    }

    pub fn write_value<A: Into<Address>, V: Into<Substate>>(&mut self, addr: A, value: V) {
        let address: Address = addr.into();

        let cur_value = self
            .borrowed_substates
            .get(&address)
            .expect("value was never locked");
        match cur_value {
            BorrowedSubstate::Loaded(..) => panic!("Cannot write to immutable"),
            BorrowedSubstate::LoadedMut(..) | BorrowedSubstate::Taken => {}
        }

        self.borrowed_substates
            .insert(address, BorrowedSubstate::LoadedMut(value.into()));
    }

    // TODO: Replace with more generic write_value once Component is split into more substates
    pub fn write_component_value(&mut self, address: Address, value: Vec<u8>) {
        match address {
            Address::GlobalComponent(..) | Address::LocalComponent(..) => {}
            _ => panic!("Unexpected address"),
        }

        let borrowed = self
            .borrowed_substates
            .get_mut(&address)
            .expect("Value was never locked");
        match borrowed {
            BorrowedSubstate::Taken => panic!("Value was taken"),
            BorrowedSubstate::Loaded(..) => panic!("Cannot write to immutable"),
            BorrowedSubstate::LoadedMut(component_val) => {
                component_val.component_mut().set_state(value);
            }
        }
    }

    pub fn release_lock<A: Into<Address>>(&mut self, addr: A) {
        let address = addr.into();
        let borrowed = self
            .borrowed_substates
            .remove(&address)
            .expect("Value was never borrowed");
        match borrowed {
            BorrowedSubstate::Taken => panic!("Value was never returned"),
            BorrowedSubstate::LoadedMut(value) => {
                self.up_substates.insert(address.encode(), value);
            }
            BorrowedSubstate::Loaded(value, mut count) => {
                count = count - 1;
                if count == 0 {
                    self.up_substates.insert(address.encode(), value);
                } else {
                    self.borrowed_substates
                        .insert(address, BorrowedSubstate::Loaded(value, count));
                }
            }
        }
    }

    /// Returns the value of a key value pair
    pub fn read_key_value(&mut self, parent_address: Address, key: Vec<u8>) -> Substate {
        let mut address = parent_address.encode();
        address.extend(key);
        if let Some(cur) = self.up_substates.get(&address) {
            match cur {
                Substate::KeyValueStoreEntry(e) => return Substate::KeyValueStoreEntry(e.clone()),
                Substate::NonFungible(n) => return Substate::NonFungible(n.clone()),
                _ => panic!("Unsupported key value"),
            }
        }
        match parent_address {
            Address::NonFungibleSet(_) => self
                .substate_store
                .get_substate(&address)
                .map(|s| s.value)
                .unwrap_or(Substate::NonFungible(None)),
            Address::KeyValueStore(..) => self
                .substate_store
                .get_substate(&address)
                .map(|s| s.value)
                .unwrap_or(Substate::KeyValueStoreEntry(None)),
            _ => panic!("Invalid keyed value address {:?}", parent_address),
        }
    }

    /// Sets a key value
    pub fn set_key_value<V: Into<Substate>>(
        &mut self,
        parent_address: Address,
        key: Vec<u8>,
        value: V,
    ) {
        let mut address = parent_address.encode();
        address.extend(key.clone());

        if self.up_substates.remove(&address).is_none() {
            let cur: Option<Output> = self.substate_store.get_substate(&address);
            if let Some(Output { value: _, phys_id }) = cur {
                self.downed_substates.push(phys_id);
            } else {
                let parent_id = self.get_substate_parent_id(&parent_address.encode());
                let virtual_substate_id = VirtualSubstateId(parent_id, key);
                self.down_virtual_substates.push(virtual_substate_id);
            }
        };

        self.up_substates.insert(address, value.into());
    }

    fn get_substate_parent_id(&mut self, space_address: &[u8]) -> SubstateParentId {
        if let Some(index) = self.up_virtual_substate_space.get_index_of(space_address) {
            SubstateParentId::New(index)
        } else {
            let substate_id = self.substate_store.get_space(space_address).unwrap();
            SubstateParentId::Exists(substate_id)
        }
    }

    /// Commits changes to the underlying ledger.
    /// Currently none of these objects are deleted so all commits are puts
    pub fn to_receipt(mut self) -> TrackReceipt {
        let mut store_instructions = Vec::new();
        for substate_id in self.downed_substates {
            store_instructions.push(SubstateOperation::Down(substate_id));
        }
        for virtual_substate_id in self.down_virtual_substates {
            store_instructions.push(SubstateOperation::VirtualDown(virtual_substate_id));
        }
        for (address, value) in self.up_substates.drain(RangeFull) {
            store_instructions.push(SubstateOperation::Up(address, value));
        }
        for space_address in self.up_virtual_substate_space.drain(RangeFull) {
            store_instructions.push(SubstateOperation::VirtualUp(space_address));
        }

        let substates = SubstateOperationsReceipt {
            substate_operations: store_instructions,
        };
        let borrowed = BorrowedSNodes {
            borrowed_substates: self.borrowed_substates.into_keys().collect(),
        };
        TrackReceipt {
            new_addresses: self.new_addresses,
            borrowed,
            substates,
            logs: self.logs,
        }
    }
}
