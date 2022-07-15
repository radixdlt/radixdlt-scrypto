use sbor::rust::collections::*;
use sbor::rust::format;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::*;
use scrypto::buffer::scrypto_encode;
use scrypto::crypto::hash;
use scrypto::engine::types::*;

use crate::engine::*;
use crate::ledger::*;

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

#[derive(Debug)]
pub enum TrackError {
    Reentrancy,
    NotFound,
}

pub struct TrackReceipt {
    pub new_addresses: Vec<Address>,
    pub borrowed_substates: HashMap<Address, BorrowedSubstate>,
    pub logs: Vec<(Level, String)>,
    pub diff: TrackDiff,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubstateUpdate<T> {
    pub prev_id: Option<OutputId>,
    pub value: T,
}

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub enum SubstateParentId {
    Exists(OutputId),
    New(Vec<u8>),
}

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct VirtualSubstateId(pub SubstateParentId, pub Vec<u8>);

/// Facilitates transactional state updates.
pub struct Track<'s, S: ReadableSubstateStore> {
    logs: Vec<(Level, String)>,

    new_addresses: Vec<Address>,
    borrowed_substates: HashMap<Address, BorrowedSubstate>,

    substate_store: &'s mut S,
    diff: TrackDiff,
}

impl<'s, S: ReadableSubstateStore> Track<'s, S> {
    pub fn new(substate_store: &'s mut S) -> Self {
        Self {
            substate_store,
            logs: Vec::new(),

            new_addresses: Vec::new(),
            borrowed_substates: HashMap::new(),

            diff: TrackDiff {
                downed_substates: Vec::new(),
                down_virtual_substates: Vec::new(),
                up_substates: BTreeMap::new(),
                up_virtual_substate_space: BTreeSet::new(),
            },
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
        self.diff
            .up_substates
            .insert(address.encode(), value.into());
    }

    pub fn create_key_space(&mut self, address: Address) {
        self.diff.up_virtual_substate_space.insert(address.encode());
    }

    pub fn take_lock<A: Into<Address>>(
        &mut self,
        addr: A,
        mutable: bool,
    ) -> Result<(), TrackError> {
        let address = addr.into();
        let maybe_value = self.diff.up_substates.remove(&address.encode());
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
            self.diff.downed_substates.push(substate.output_id);
            self.borrowed_substates.insert(
                address.clone(),
                BorrowedSubstate::loaded(substate.substate, mutable),
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
            .insert(address.clone(), BorrowedSubstate::Taken)
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
                self.diff.up_substates.insert(address.encode(), value);
            }
            BorrowedSubstate::Loaded(value, mut count) => {
                count = count - 1;
                if count == 0 {
                    self.diff.up_substates.insert(address.encode(), value);
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
        if let Some(cur) = self.diff.up_substates.get(&address) {
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
                .map(|s| s.substate)
                .unwrap_or(Substate::NonFungible(None)),
            Address::KeyValueStore(..) => self
                .substate_store
                .get_substate(&address)
                .map(|s| s.substate)
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

        if self.diff.up_substates.remove(&address).is_none() {
            let cur: Option<Output> = self.substate_store.get_substate(&address);
            if let Some(Output { substate: _, output_id: phys_id }) = cur {
                self.diff.downed_substates.push(phys_id);
            } else {
                let parent_id = self.get_substate_parent_id(&parent_address.encode());
                let virtual_substate_id = VirtualSubstateId(parent_id, key);
                self.diff.down_virtual_substates.push(virtual_substate_id);
            }
        };

        self.diff.up_substates.insert(address, value.into());
    }

    fn get_substate_parent_id(&mut self, space_address: &[u8]) -> SubstateParentId {
        if self.diff.up_virtual_substate_space.contains(space_address) {
            SubstateParentId::New(space_address.to_vec())
        } else {
            let substate_id = self.substate_store.get_space(space_address);
            SubstateParentId::Exists(substate_id)
        }
    }

    /// Commits changes to the underlying ledger.
    /// Currently none of these objects are deleted so all commits are puts
    pub fn to_receipt(self) -> TrackReceipt {
        TrackReceipt {
            new_addresses: self.new_addresses,
            borrowed_substates: self.borrowed_substates,
            logs: self.logs,
            diff: self.diff,
        }
    }
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct TrackDiff {
    up_virtual_substate_space: BTreeSet<Vec<u8>>,
    up_substates: BTreeMap<Vec<u8>, Substate>,
    down_virtual_substates: Vec<VirtualSubstateId>,
    downed_substates: Vec<OutputId>,
}

impl TrackDiff {
    /// Commits changes to the underlying ledger.
    /// Currently none of these objects are deleted so all commits are puts
    pub fn commit<S: WriteableSubstateStore>(self, store: &mut S) -> CommitReceipt {
        let hash = hash(scrypto_encode(&self));
        let mut receipt = CommitReceipt::new();
        let mut id_gen = OutputIdGenerator::new(hash);
        let mut virtual_outputs = HashMap::new();

        for space_address in self.up_virtual_substate_space {
            let phys_id = id_gen.next();
            receipt.virtual_space_up(phys_id.clone());
            store.put_space(&space_address, phys_id.clone());
            virtual_outputs.insert(space_address, phys_id);
        }
        for output_id in self.downed_substates {
            receipt.down(output_id);
        }
        for VirtualSubstateId(parent_id, key) in self.down_virtual_substates {
            let parent_hard_id = match parent_id {
                SubstateParentId::Exists(real_id) => real_id,
                SubstateParentId::New(key) => virtual_outputs.get(&key).cloned().unwrap(),
            };
            let virtual_substate_id = HardVirtualSubstateId(parent_hard_id, key);
            receipt.virtual_down(virtual_substate_id);
        }
        for (address, value) in self.up_substates {
            let phys_id = id_gen.next();
            receipt.up(phys_id.clone());
            let substate = Output { substate: value, output_id: phys_id };
            store.put_substate(&address, substate);
        }

        receipt
    }
}
