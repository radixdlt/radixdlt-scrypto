use indexmap::{IndexMap, IndexSet};
use sbor::rust::collections::*;
use sbor::rust::ops::RangeFull;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::*;
use scrypto::buffer::scrypto_decode;
use scrypto::buffer::scrypto_encode;
use scrypto::engine::types::*;
use scrypto::values::ScryptoValue;
use transaction::validation::*;

use crate::engine::{StoredValue, SubstateOperation, SubstateOperationsReceipt};
use crate::ledger::*;
use crate::model::*;

/// Facilitates transactional state updates.
pub struct Track<'s, S: ReadableSubstateStore> {
    substate_store: &'s mut S,
    transaction_hash: Hash,
    id_allocator: IdAllocator,
    logs: Vec<(Level, String)>,

    new_addresses: Vec<Address>,
    borrowed_substates: HashSet<Address>,
    read_substates: IndexMap<Address, SubstateValue>,

    downed_substates: Vec<PhysicalSubstateId>,
    down_virtual_substates: Vec<VirtualSubstateId>,
    up_substates: IndexMap<Vec<u8>, SubstateValue>,
    up_virtual_substate_space: IndexSet<Vec<u8>>,
}

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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Address {
    Resource(ResourceAddress),
    Component(ComponentAddress),
    Package(PackageAddress),
    NonFungibleSet(ResourceAddress),
    KeyValueStore(ComponentAddress, KeyValueStoreId),
    Vault(ComponentAddress, VaultId),
}

#[derive(Debug)]
pub enum SubstateValue {
    Resource(ResourceManager),
    Component(Component),
    Package(ValidatedPackage),
    Vault(Vault),
    NonFungible(Option<NonFungible>),
    KeyValueStoreEntry(Option<Vec<u8>>),
}

// TODO: Replace NonFungible with real re address
// TODO: Move this logic into application layer
macro_rules! resource_to_non_fungible_space {
    ($resource_address:expr) => {{
        let mut addr = scrypto_encode(&$resource_address);
        addr.push(0u8);
        addr
    }};
}

impl Address {
    fn encode(&self) -> Vec<u8> {
        match self {
            Address::Resource(resource_address) => scrypto_encode(resource_address),
            Address::Component(component_address) => scrypto_encode(component_address),
            Address::Package(package_address) => scrypto_encode(package_address),
            Address::Vault(component_address, vault_id) => {
                let mut vault_address = scrypto_encode(component_address);
                vault_address.extend(scrypto_encode(vault_id));
                vault_address
            }
            Address::NonFungibleSet(resource_address) => {
                resource_to_non_fungible_space!(resource_address.clone())
            }
            Address::KeyValueStore(component_address, kv_store_id) => {
                let mut entry_address = scrypto_encode(component_address);
                entry_address.extend(scrypto_encode(kv_store_id));
                entry_address
            }
        }
    }
}

impl Into<Address> for PackageAddress {
    fn into(self) -> Address {
        Address::Package(self)
    }
}

impl Into<Address> for ComponentAddress {
    fn into(self) -> Address {
        Address::Component(self)
    }
}

impl Into<Address> for ResourceAddress {
    fn into(self) -> Address {
        Address::Resource(self)
    }
}

impl Into<Address> for (ComponentAddress, VaultId) {
    fn into(self) -> Address {
        Address::Vault(self.0, self.1)
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
        if let Address::Component(component_address) = self {
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

impl SubstateValue {
    fn encode(&self) -> Vec<u8> {
        match self {
            SubstateValue::Resource(resource_manager) => scrypto_encode(resource_manager),
            SubstateValue::Package(package) => scrypto_encode(package),
            SubstateValue::Component(component) => scrypto_encode(component),
            SubstateValue::Vault(vault) => scrypto_encode(vault),
            SubstateValue::NonFungible(non_fungible) => scrypto_encode(non_fungible),
            SubstateValue::KeyValueStoreEntry(value) => scrypto_encode(value),
        }
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
        SubstateValue::Vault(self)
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

impl Into<Vault> for SubstateValue {
    fn into(self) -> Vault {
        if let SubstateValue::Vault(vault) = self {
            vault
        } else {
            panic!("Not a vault");
        }
    }
}

impl<'s, S: ReadableSubstateStore> Track<'s, S> {
    pub fn new(substate_store: &'s mut S, transaction_hash: Hash) -> Self {
        Self {
            substate_store,
            transaction_hash,
            id_allocator: IdAllocator::new(IdSpace::Application),
            logs: Vec::new(),

            new_addresses: Vec::new(),
            borrowed_substates: HashSet::new(),
            read_substates: IndexMap::new(),

            downed_substates: Vec::new(),
            down_virtual_substates: Vec::new(),
            up_substates: IndexMap::new(),
            up_virtual_substate_space: IndexSet::new(),
        }
    }

    /// Returns the transaction hash.
    pub fn transaction_hash(&self) -> Hash {
        self.transaction_hash
    }

    /// Returns the current epoch.
    pub fn current_epoch(&self) -> u64 {
        self.substate_store.get_epoch()
    }

    /// Adds a log message.
    pub fn add_log(&mut self, level: Level, message: String) {
        self.logs.push((level, message));
    }

    /// Creates a new uuid key with a given value
    pub fn create_uuid_value<T: Into<SubstateValue>>(&mut self, value: T) -> Address {
        let substate_value = value.into();
        let address = match substate_value {
            SubstateValue::Package(_) => {
                let package_address = self.new_package_address();
                Address::Package(package_address)
            }
            SubstateValue::Component(_) => {
                let component_address = self.new_component_address();
                Address::Component(component_address)
            }
            SubstateValue::Resource(ref resource_manager) => {
                let resource_address = self.new_resource_address();
                // TODO: Move this into application layer
                if let ResourceType::NonFungible = resource_manager.resource_type() {
                    let space_address = resource_to_non_fungible_space!(resource_address);
                    self.up_virtual_substate_space.insert(space_address);
                }
                Address::Resource(resource_address)
            }
            _ => panic!("Trying to create uuid value with invalid value"),
        };

        self.new_addresses.push(address.clone());
        self.up_substates.insert(address.encode(), substate_value);
        address
    }

    pub fn create_uuid_value_2<A: Into<Address>, V: Into<SubstateValue>>(
        &mut self,
        addr: A,
        value: V,
    ) {
        self.up_substates.insert(addr.into().encode(), value.into());
    }

    pub fn create_key_space(
        &mut self,
        component_address: ComponentAddress,
        kv_store_id: KeyValueStoreId,
    ) {
        let mut space_address = scrypto_encode(&component_address);
        space_address.extend(scrypto_encode(&kv_store_id));
        self.up_virtual_substate_space.insert(space_address);
    }

    /// Returns an immutable reference to a value, if exists.
    pub fn read_value<A: Into<Address>>(&mut self, addr: A) -> Option<&SubstateValue> {
        let address: Address = addr.into();

        if let Some(v) = self.up_substates.get(&address.encode()) {
            return Some(v);
        }

        let maybe_substate = self.substate_store.get_substate(&address.encode());
        if let Some(substate) = maybe_substate {
            match address {
                Address::Package(_) => {
                    let package: ValidatedPackage = scrypto_decode(&substate.value).unwrap();
                    self.read_substates
                        .insert(address.clone(), SubstateValue::Package(package));
                    self.read_substates.get(&address)
                }
                Address::Component(_) => {
                    let component: Component = scrypto_decode(&substate.value).unwrap();
                    self.read_substates
                        .insert(address.clone(), SubstateValue::Component(component));
                    self.read_substates.get(&address)
                }
                Address::Resource(_) => {
                    let resource_manager: ResourceManager =
                        scrypto_decode(&substate.value).unwrap();
                    self.read_substates
                        .insert(address.clone(), SubstateValue::Resource(resource_manager));
                    self.read_substates.get(&address)
                }
                _ => panic!("Reading value of invalid address"),
            }
        } else {
            None
        }
    }

    // TODO: Add checks to see verify that immutable values aren't being borrowed
    pub fn borrow_global_mut_value<A: Into<Address>>(
        &mut self,
        addr: A,
    ) -> Result<SubstateValue, TrackError> {
        let address = addr.into();
        let maybe_value = self.up_substates.remove(&address.encode());
        if let Some(value) = maybe_value {
            self.borrowed_substates.insert(address);
            Ok(value)
        } else if self.borrowed_substates.contains(&address) {
            Err(TrackError::Reentrancy)
        } else if let Some(substate) = self.substate_store.get_substate(&address.encode()) {
            self.downed_substates.push(substate.phys_id);
            self.borrowed_substates.insert(address.clone());
            match address {
                Address::Component(_) => {
                    let component = scrypto_decode(&substate.value).unwrap();
                    Ok(SubstateValue::Component(component))
                }
                Address::Resource(_) => {
                    let resource_manager = scrypto_decode(&substate.value).unwrap();
                    Ok(SubstateValue::Resource(resource_manager))
                }
                Address::Vault(..) => {
                    let vault = scrypto_decode(&substate.value).unwrap();
                    Ok(SubstateValue::Vault(vault))
                }
                _ => panic!("Attempting to borrow unsupported value"),
            }
        } else {
            Err(TrackError::NotFound)
        }
    }

    pub fn return_borrowed_global_mut_value<A: Into<Address>, V: Into<SubstateValue>>(
        &mut self,
        addr: A,
        value: V,
    ) {
        let address = addr.into();
        if !self.borrowed_substates.remove(&address) {
            panic!("Value was never borrowed");
        }
        self.up_substates.insert(address.encode(), value.into());
    }

    /// Returns the value of a key value pair
    pub fn read_key_value(&mut self, parent_address: Address, key: Vec<u8>) -> SubstateValue {
        let mut address = parent_address.encode();
        address.extend(key);
        if let Some(cur) = self.up_substates.get(&address) {
            match cur {
                SubstateValue::KeyValueStoreEntry(e) => {
                    return SubstateValue::KeyValueStoreEntry(e.clone())
                }
                SubstateValue::NonFungible(n) => return SubstateValue::NonFungible(n.clone()),
                _ => panic!("Unsupported key value"),
            }
        }
        match parent_address {
            Address::NonFungibleSet(_) => self
                .substate_store
                .get_substate(&address)
                .map(|r| {
                    let non_fungible = scrypto_decode(&r.value).unwrap();
                    SubstateValue::NonFungible(non_fungible)
                })
                .unwrap_or(SubstateValue::NonFungible(None)),
            Address::KeyValueStore(..) => self
                .substate_store
                .get_substate(&address)
                .map(|r| {
                    let kv_store_entry = scrypto_decode(&r.value).unwrap();
                    SubstateValue::KeyValueStoreEntry(kv_store_entry)
                })
                .unwrap_or(SubstateValue::KeyValueStoreEntry(None)),
            _ => panic!("Invalid keyed value address"),
        }
    }

    /// Sets a key value
    pub fn set_key_value<V: Into<SubstateValue>>(
        &mut self,
        parent_address: Address,
        key: Vec<u8>,
        value: V,
    ) {
        let mut address = parent_address.encode();
        address.extend(key.clone());

        if self.up_substates.remove(&address).is_none() {
            let cur: Option<Substate> = self.substate_store.get_substate(&address);
            if let Some(Substate { value: _, phys_id }) = cur {
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

    /// Creates a new package ID.
    fn new_package_address(&mut self) -> PackageAddress {
        // Security Alert: ensure ID allocating will practically never fail
        let package_address = self
            .id_allocator
            .new_package_address(self.transaction_hash())
            .unwrap();
        package_address
    }

    /// Creates a new component address.
    fn new_component_address(&mut self) -> ComponentAddress {
        let component_address = self
            .id_allocator
            .new_component_address(self.transaction_hash())
            .unwrap();
        component_address
    }

    /// Creates a new resource address.
    fn new_resource_address(&mut self) -> ResourceAddress {
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
            store_instructions.push(SubstateOperation::Up(address, value.encode()));
        }
        for space_address in self.up_virtual_substate_space.drain(RangeFull) {
            store_instructions.push(SubstateOperation::VirtualUp(space_address));
        }

        let substates = SubstateOperationsReceipt {
            substate_operations: store_instructions,
        };
        let borrowed = BorrowedSNodes {
            borrowed_substates: self.borrowed_substates,
        };
        TrackReceipt {
            new_addresses: self.new_addresses,
            borrowed,
            substates,
            logs: self.logs,
        }
    }

    pub fn insert_objects_into_component(
        &mut self,
        values: Vec<StoredValue>,
        component_address: ComponentAddress,
    ) {
        for value in values {
            match value {
                StoredValue::Vault(vault_id, vault) => {
                    self.create_uuid_value_2((component_address, vault_id), vault);
                }
                StoredValue::KeyValueStore(kv_store_id, kv_store) => {
                    self.create_key_space(component_address, kv_store_id);
                    let parent_address = Address::KeyValueStore(component_address, kv_store_id);
                    for (k, v) in kv_store.store {
                        self.set_key_value(parent_address.clone(), k, Some(v));
                    }
                    let child_values = kv_store
                        .child_values
                        .into_values()
                        .map(|v| v.into_inner())
                        .collect();
                    self.insert_objects_into_component(child_values, component_address);
                }
            }
        }
    }
}
