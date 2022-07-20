use sbor::rust::collections::*;
use sbor::rust::format;
use sbor::rust::rc::Rc;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::*;
use scrypto::core::Network;
use scrypto::engine::types::*;
use scrypto::values::ScryptoValue;
use transaction::validation::*;

use crate::engine::track::BorrowedSubstate::Taken;
use crate::engine::{RENode, SubstateOperationsReceipt};
use crate::ledger::*;
use crate::model::*;

use super::AppStateTrack;
use super::BaseStateTrack;
use super::StateTrack;

enum BorrowedSubstate {
    Loaded(SubstateValue, u32),
    LoadedMut(SubstateValue),
    Taken,
}

impl BorrowedSubstate {
    fn loaded(value: SubstateValue, mutable: bool) -> Self {
        if mutable {
            BorrowedSubstate::LoadedMut(value)
        } else {
            BorrowedSubstate::Loaded(value, 1)
        }
    }
}

/// Manages global objects created or loaded for transaction.
/// TODO: rename to `TransactionGlobals` or similar
pub struct Track {
    transaction_hash: Hash,
    transaction_network: Network,
    id_allocator: IdAllocator,
    logs: Vec<(Level, String)>,
    new_addresses: Vec<Address>,
    state_track: AppStateTrack,
    borrowed_substates: HashMap<Address, BorrowedSubstate>,
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
    pub new_addresses: Vec<Address>,
    pub logs: Vec<(Level, String)>,
    pub state_changes: SubstateOperationsReceipt,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubstateUpdate<T> {
    pub prev_id: Option<PhysicalSubstateId>,
    pub value: T,
}

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub enum SubstateParentId {
    Exists(PhysicalSubstateId),
    New(usize),
}

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct VirtualSubstateId(pub SubstateParentId, pub Vec<u8>);

/// Represents a Radix Engine address. Each maps a unique substate key.
#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq, Hash)]
pub enum Address {
    Resource(ResourceAddress),
    GlobalComponent(ComponentAddress),
    Package(PackageAddress),
    NonFungibleSet(ResourceAddress),
    NonFungible(ResourceAddress, Vec<u8>),
    KeyValueStore(KeyValueStoreId),
    KeyValueStoreEntry(KeyValueStoreId, Vec<u8>),
    Vault(VaultId),
    LocalComponent(ComponentAddress),
    System,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub enum SubstateValue {
    System(System),
    Resource(ResourceManager),
    Component(Component),
    Package(ValidatedPackage),
    Vault(Vault, Option<ResourceContainer>),
    NonFungible(Option<NonFungible>),
    KeyValueStoreEntry(Option<Vec<u8>>),
}

impl Into<Address> for PackageAddress {
    fn into(self) -> Address {
        Address::Package(self)
    }
}

impl Into<Address> for ResourceAddress {
    fn into(self) -> Address {
        Address::Resource(self)
    }
}

impl Into<Address> for VaultId {
    fn into(self) -> Address {
        Address::Vault(self)
    }
}

impl Into<PackageAddress> for Address {
    fn into(self) -> PackageAddress {
        if let Address::Package(package_address) = self {
            return package_address;
        } else {
            panic!("Address is not a package address");
        }
    }
}

impl Into<ComponentAddress> for Address {
    fn into(self) -> ComponentAddress {
        if let Address::GlobalComponent(component_address) = self {
            return component_address;
        } else {
            panic!("Address is not a component address");
        }
    }
}

impl Into<ResourceAddress> for Address {
    fn into(self) -> ResourceAddress {
        if let Address::Resource(resource_address) = self {
            return resource_address;
        } else {
            panic!("Address is not a resource address");
        }
    }
}

impl Into<VaultId> for Address {
    fn into(self) -> VaultId {
        if let Address::Vault(id) = self {
            return id;
        } else {
            panic!("Address is not a vault address");
        }
    }
}

impl SubstateValue {
    pub fn vault_mut(&mut self) -> (&mut Vault, &mut Option<ResourceContainer>) {
        if let SubstateValue::Vault(liquid, locked) = self {
            (liquid, locked)
        } else {
            panic!("Not a vault");
        }
    }

    pub fn vault(&self) -> (&Vault, &Option<ResourceContainer>) {
        if let SubstateValue::Vault(liquid, locked) = self {
            (liquid, locked)
        } else {
            panic!("Not a vault");
        }
    }

    pub fn resource_manager_mut(&mut self) -> &mut ResourceManager {
        if let SubstateValue::Resource(resource_manager) = self {
            resource_manager
        } else {
            panic!("Not a resource manager");
        }
    }

    pub fn system(&self) -> &System {
        if let SubstateValue::System(system) = self {
            system
        } else {
            panic!("Not a system value");
        }
    }

    pub fn system_mut(&mut self) -> &mut System {
        if let SubstateValue::System(system) = self {
            system
        } else {
            panic!("Not a system value");
        }
    }

    pub fn resource_manager(&self) -> &ResourceManager {
        if let SubstateValue::Resource(resource_manager) = self {
            resource_manager
        } else {
            panic!("Not a resource manager");
        }
    }

    pub fn component(&self) -> &Component {
        if let SubstateValue::Component(component) = self {
            component
        } else {
            panic!("Not a component");
        }
    }

    pub fn component_mut(&mut self) -> &mut Component {
        if let SubstateValue::Component(component) = self {
            component
        } else {
            panic!("Not a component");
        }
    }

    pub fn package(&self) -> &ValidatedPackage {
        if let SubstateValue::Package(package) = self {
            package
        } else {
            panic!("Not a package");
        }
    }

    pub fn non_fungible(&self) -> &Option<NonFungible> {
        if let SubstateValue::NonFungible(non_fungible) = self {
            non_fungible
        } else {
            panic!("Not a NonFungible");
        }
    }

    pub fn kv_entry(&self) -> &Option<Vec<u8>> {
        if let SubstateValue::KeyValueStoreEntry(kv_entry) = self {
            kv_entry
        } else {
            panic!("Not a KVEntry");
        }
    }
}

impl Into<SubstateValue> for System {
    fn into(self) -> SubstateValue {
        SubstateValue::System(self)
    }
}

impl Into<SubstateValue> for ValidatedPackage {
    fn into(self) -> SubstateValue {
        SubstateValue::Package(self)
    }
}

impl Into<SubstateValue> for Component {
    fn into(self) -> SubstateValue {
        SubstateValue::Component(self)
    }
}

impl Into<SubstateValue> for ResourceManager {
    fn into(self) -> SubstateValue {
        SubstateValue::Resource(self)
    }
}

impl Into<SubstateValue> for Vault {
    fn into(self) -> SubstateValue {
        SubstateValue::Vault(self, None)
    }
}

impl Into<SubstateValue> for Option<NonFungible> {
    fn into(self) -> SubstateValue {
        SubstateValue::NonFungible(self)
    }
}

impl Into<SubstateValue> for Option<ScryptoValue> {
    fn into(self) -> SubstateValue {
        SubstateValue::KeyValueStoreEntry(self.map(|v| v.raw))
    }
}

impl Into<Component> for SubstateValue {
    fn into(self) -> Component {
        if let SubstateValue::Component(component) = self {
            component
        } else {
            panic!("Not a component");
        }
    }
}

impl Into<ResourceManager> for SubstateValue {
    fn into(self) -> ResourceManager {
        if let SubstateValue::Resource(resource_manager) = self {
            resource_manager
        } else {
            panic!("Not a resource manager");
        }
    }
}

impl Into<ValidatedPackage> for SubstateValue {
    fn into(self) -> ValidatedPackage {
        if let SubstateValue::Package(package) = self {
            package
        } else {
            panic!("Not a resource manager");
        }
    }
}

impl Into<Option<NonFungible>> for SubstateValue {
    fn into(self) -> Option<NonFungible> {
        if let SubstateValue::NonFungible(non_fungible) = self {
            non_fungible
        } else {
            panic!("Not a resource manager");
        }
    }
}

impl Into<Vault> for SubstateValue {
    fn into(self) -> Vault {
        if let SubstateValue::Vault(liquid, locked) = self {
            assert!(
                locked.is_none(),
                "Attempted to convert a partially-locked vault into substate value"
            );
            liquid
        } else {
            panic!("Not a vault");
        }
    }
}

impl Track {
    pub fn new(
        substate_store: Rc<dyn ReadableSubstateStore>,
        transaction_hash: Hash,
        transaction_network: Network,
    ) -> Self {
        let base_state_track = BaseStateTrack::new(substate_store);
        let state_track = AppStateTrack::new(base_state_track);

        Self {
            transaction_hash,
            transaction_network,
            id_allocator: IdAllocator::new(IdSpace::Application),
            logs: Vec::new(),
            new_addresses: Vec::new(),
            state_track,
            borrowed_substates: HashMap::new(),
        }
    }

    /// Returns the transaction hash.
    pub fn transaction_hash(&self) -> Hash {
        self.transaction_hash
    }
    pub fn transaction_network(&self) -> Network {
        self.transaction_network.clone()
    }

    /// Adds a log message.
    pub fn add_log(&mut self, level: Level, message: String) {
        self.logs.push((level, message));
    }

    /// Creates a row with the given key/value
    pub fn create_uuid_value<A: Into<Address>, V: Into<SubstateValue>>(
        &mut self,
        addr: A,
        value: V,
    ) {
        let address = addr.into();
        self.new_addresses.push(address.clone());
        self.state_track.put_substate(address, value.into());
    }

    pub fn create_key_space(&mut self, address: Address) {
        self.state_track.put_space(address);
    }

    pub fn take_lock<A: Into<Address>>(
        &mut self,
        addr: A,
        mutable: bool,
        write_through: bool,
    ) -> Result<(), TrackError> {
        let address = addr.into();

        // TODO: to read/write a value owned by track requires three coordinated steps:
        // 1. Attempt to acquire the lock
        // 2. Apply the operation
        // 3. Release lock
        //
        // A better idea is properly move the lock-unlock into the operation OR to have a proper
        // representation of locked resource and apply operation on top of it.

        if write_through {
            // TODO:
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

        if let Some(substate) = self.state_track.get_substate(&address) {
            let value = match address {
                Address::GlobalComponent(_)
                | Address::LocalComponent(..)
                | Address::Resource(_)
                | Address::Vault(..)
                | Address::Package(..)
                | Address::System => substate,
                _ => panic!("Attempting to borrow unsupported value {:?}", address),
            };

            self.borrowed_substates
                .insert(address.clone(), BorrowedSubstate::loaded(value, mutable));
            Ok(())
        } else {
            Err(TrackError::NotFound)
        }
    }

    pub fn read_value<A: Into<Address>>(&self, addr: A) -> &SubstateValue {
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

    pub fn take_value<A: Into<Address>>(&mut self, addr: A) -> SubstateValue {
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

    pub fn write_value<A: Into<Address>, V: Into<SubstateValue>>(&mut self, addr: A, value: V) {
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

    pub fn release_lock<A: Into<Address>>(&mut self, addr: A, write_through: bool) {
        let address = addr.into();
        let borrowed = self
            .borrowed_substates
            .remove(&address)
            .expect("Value was never borrowed");

        if write_through {
            // TODO
        }

        match borrowed {
            BorrowedSubstate::Taken => panic!("Value was never returned"),
            BorrowedSubstate::LoadedMut(value) => {
                self.state_track.put_substate(address, value);
            }
            BorrowedSubstate::Loaded(value, mut count) => {
                count = count - 1;
                if count == 0 {
                    self.state_track.put_substate(address, value);
                } else {
                    self.borrowed_substates
                        .insert(address, BorrowedSubstate::Loaded(value, count));
                }
            }
        }
    }

    /// Returns the value of a key value pair
    pub fn read_key_value(&mut self, parent_address: Address, key: Vec<u8>) -> SubstateValue {
        // TODO: consider using a single address as function input
        let address = match parent_address {
            Address::NonFungibleSet(resource_address) => {
                Address::NonFungible(resource_address, key)
            }
            Address::KeyValueStore(kv_store_id) => Address::KeyValueStoreEntry(kv_store_id, key),
            _ => panic!("Unsupported key value"),
        };

        match parent_address {
            Address::NonFungibleSet(_) => self
                .state_track
                .get_substate(&address)
                .unwrap_or(SubstateValue::NonFungible(None)),
            Address::KeyValueStore(..) => self
                .state_track
                .get_substate(&address)
                .unwrap_or(SubstateValue::KeyValueStoreEntry(None)),
            _ => panic!("Invalid keyed value address {:?}", parent_address),
        }
    }

    /// Sets a key value
    pub fn set_key_value<V: Into<SubstateValue>>(
        &mut self,
        parent_address: Address,
        key: Vec<u8>,
        value: V,
    ) {
        // TODO: consider using a single address as function input
        let address = match parent_address {
            Address::NonFungibleSet(resource_address) => {
                Address::NonFungible(resource_address, key.clone())
            }
            Address::KeyValueStore(kv_store_id) => {
                Address::KeyValueStoreEntry(kv_store_id, key.clone())
            }
            _ => panic!("Unsupported key value"),
        };

        self.state_track.put_substate(address, value.into());
    }

    /// Creates a new package ID.
    pub fn new_package_address(&mut self) -> PackageAddress {
        // Security Alert: ensure ID allocating will practically never fail
        let package_address = self
            .id_allocator
            .new_package_address(self.transaction_hash())
            .unwrap();
        package_address
    }

    /// Creates a new component address.
    pub fn new_component_address(&mut self, component: &Component) -> ComponentAddress {
        let component_address = self
            .id_allocator
            .new_component_address(
                self.transaction_hash(),
                &component.package_address(),
                component.blueprint_name(),
            )
            .unwrap();
        component_address
    }

    /// Creates a new resource address.
    pub fn new_resource_address(&mut self) -> ResourceAddress {
        let resource_address = self
            .id_allocator
            .new_resource_address(self.transaction_hash())
            .unwrap();
        resource_address
    }

    /// Creates a new UUID.
    pub fn new_uuid(&mut self) -> u128 {
        self.id_allocator.new_uuid(self.transaction_hash()).unwrap()
    }

    /// Creates a new bucket ID.
    pub fn new_bucket_id(&mut self) -> BucketId {
        self.id_allocator.new_bucket_id().unwrap()
    }

    /// Creates a new vault ID.
    pub fn new_vault_id(&mut self) -> VaultId {
        self.id_allocator
            .new_vault_id(self.transaction_hash())
            .unwrap()
    }

    /// Creates a new reference id.
    pub fn new_proof_id(&mut self) -> ProofId {
        self.id_allocator.new_proof_id().unwrap()
    }

    /// Creates a new map id.
    pub fn new_kv_store_id(&mut self) -> KeyValueStoreId {
        self.id_allocator
            .new_kv_store_id(self.transaction_hash())
            .unwrap()
    }

    pub fn insert_non_root_nodes(&mut self, values: HashMap<ValueId, RENode>) {
        for (id, node) in values {
            match node {
                RENode::Vault(vault) => {
                    let addr = Address::Vault(id.into());
                    self.create_uuid_value(addr, vault);
                }
                RENode::Component(component) => {
                    let addr = Address::LocalComponent(id.into());
                    self.create_uuid_value(addr, component);
                }
                RENode::KeyValueStore(store) => {
                    let id = id.into();
                    let address = Address::KeyValueStore(id);
                    self.create_key_space(address.clone());
                    for (k, v) in store.store {
                        self.set_key_value(address.clone(), k, Some(v));
                    }
                }
                _ => panic!("Invalid node being persisted: {:?}", node),
            }
        }
    }

    pub fn to_receipt(mut self, commit_app_state_changes: bool) -> TrackReceipt {
        if commit_app_state_changes {
            self.state_track.flush();
        }

        TrackReceipt {
            new_addresses: self.new_addresses,
            logs: self.logs,
            state_changes: self.state_track.unwrap().to_receipt(),
        }
    }
}
