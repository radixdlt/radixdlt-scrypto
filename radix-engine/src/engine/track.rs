use indexmap::{IndexMap, IndexSet};
use sbor::rust::collections::*;
use sbor::rust::ops::RangeFull;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::*;
use scrypto::buffer::scrypto_decode;
use scrypto::buffer::scrypto_encode;
use scrypto::engine::types::*;

use crate::engine::{
    ComponentObjects, IdAllocator, IdSpace, SubstateOperation, SubstateOperationsReceipt,
};
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

    vaults: IndexMap<(ComponentAddress, VaultId), SubstateUpdate<Vault>>,
    borrowed_vaults: HashMap<(ComponentAddress, VaultId), Option<PhysicalSubstateId>>,
}

pub enum TrackError {
    Reentrancy,
    NotFound,
}

pub struct BorrowedSNodes {
    borrowed_substates: HashSet<Address>,
    borrowed_vaults: HashMap<(ComponentAddress, VaultId), Option<PhysicalSubstateId>>,
}

impl BorrowedSNodes {
    pub fn is_empty(&self) -> bool {
        self.borrowed_substates.is_empty() && self.borrowed_vaults.is_empty()
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
    LazyMap(ComponentAddress, LazyMapId),
}

#[derive(Debug, Clone)]
pub enum SubstateValue {
    Resource(ResourceManager),
    Component(Component),
    Package(Package),
    NonFungible(Option<NonFungible>),
    LazyMapEntry(Option<Vec<u8>>),
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
            Address::NonFungibleSet(resource_address) => {
                resource_to_non_fungible_space!(resource_address.clone())
            }
            Address::LazyMap(component_address, lazy_map_id) => {
                let mut entry_address = scrypto_encode(component_address);
                entry_address.extend(scrypto_encode(lazy_map_id));
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
            SubstateValue::NonFungible(non_fungible) => scrypto_encode(non_fungible),
            SubstateValue::LazyMapEntry(value) => scrypto_encode(value),
        }
    }
}

impl Into<SubstateValue> for Package {
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

impl Into<SubstateValue> for Option<NonFungible> {
    fn into(self) -> SubstateValue {
        SubstateValue::NonFungible(self)
    }
}

impl Into<SubstateValue> for Option<Vec<u8>> {
    fn into(self) -> SubstateValue {
        SubstateValue::LazyMapEntry(self)
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

            vaults: IndexMap::new(),
            borrowed_vaults: HashMap::new(),
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
                    let package: Package = scrypto_decode(&substate.value).unwrap();
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
            return cur.clone();
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
            Address::LazyMap(..) => self
                .substate_store
                .get_substate(&address)
                .map(|r| {
                    let lazy_map_entry = scrypto_decode(&r.value).unwrap();
                    SubstateValue::LazyMapEntry(lazy_map_entry)
                })
                .unwrap_or(SubstateValue::LazyMapEntry(None)),
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

    pub fn borrow_vault_mut(
        &mut self,
        component_address: &ComponentAddress,
        vid: &VaultId,
    ) -> Vault {
        let canonical_id = (component_address.clone(), vid.clone());
        if self.borrowed_vaults.contains_key(&canonical_id) {
            panic!("Invalid vault reentrancy");
        }

        if let Some(SubstateUpdate { value, prev_id }) = self.vaults.remove(&canonical_id) {
            self.borrowed_vaults.insert(canonical_id, prev_id);
            return value;
        }

        if let Some((vault, substate_id)) = self
            .substate_store
            .get_decoded_child_substate(component_address, vid)
        {
            self.borrowed_vaults.insert(canonical_id, Some(substate_id));
            return vault;
        }

        panic!("Should not get here");
    }

    pub fn return_borrowed_vault(
        &mut self,
        component_address: &ComponentAddress,
        vid: &VaultId,
        vault: Vault,
    ) {
        let canonical_id = (component_address.clone(), vid.clone());
        if let Some(prev_id) = self.borrowed_vaults.remove(&canonical_id) {
            self.vaults.insert(
                canonical_id,
                SubstateUpdate {
                    prev_id,
                    value: vault,
                },
            );
        } else {
            panic!("Vault was never borrowed");
        }
    }

    /// Inserts a new vault.
    pub fn insert_new_vault(
        &mut self,
        component_address: ComponentAddress,
        vault_id: VaultId,
        vault: Vault,
    ) {
        let canonical_id = (component_address, vault_id);
        self.vaults.insert(
            canonical_id,
            SubstateUpdate {
                prev_id: None,
                value: vault,
            },
        );
    }

    pub fn insert_new_lazy_map(
        &mut self,
        component_address: ComponentAddress,
        lazy_map_id: LazyMapId,
    ) {
        let mut space_address = scrypto_encode(&component_address);
        space_address.extend(scrypto_encode(&lazy_map_id));
        self.up_virtual_substate_space.insert(space_address);
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
    pub fn new_lazy_map_id(&mut self) -> LazyMapId {
        self.id_allocator
            .new_lazy_map_id(self.transaction_hash())
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

        for ((component_address, vault_id), vault) in self.vaults.drain(RangeFull) {
            if let Some(substate_id) = vault.prev_id {
                store_instructions.push(SubstateOperation::Down(substate_id));
            }
            let mut vault_address = scrypto_encode(&component_address);
            vault_address.extend(scrypto_encode(&vault_id));
            store_instructions.push(SubstateOperation::Up(
                vault_address,
                scrypto_encode(&vault.value),
            ));
        }

        let substates = SubstateOperationsReceipt {
            substate_operations: store_instructions,
        };
        let borrowed = BorrowedSNodes {
            borrowed_substates: self.borrowed_substates,
            borrowed_vaults: self.borrowed_vaults,
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
        new_objects: ComponentObjects,
        component_address: ComponentAddress,
    ) {
        for (vault_id, vault) in new_objects.vaults {
            self.insert_new_vault(component_address, vault_id, vault);
        }
        for (lazy_map_id, unclaimed) in new_objects.lazy_maps {
            self.insert_new_lazy_map(component_address, lazy_map_id);
            for (k, v) in unclaimed.lazy_map {
                let parent_address = Address::LazyMap(component_address, lazy_map_id);
                self.set_key_value(parent_address, k, Some(v));
            }

            for (child_lazy_map_id, child_lazy_map) in unclaimed.descendent_lazy_maps {
                self.insert_new_lazy_map(component_address, child_lazy_map_id);
                for (k, v) in child_lazy_map {
                    let parent_address = Address::LazyMap(component_address, child_lazy_map_id);
                    self.set_key_value(parent_address, k, Some(v));
                }
            }
            for (vault_id, vault) in unclaimed.descendent_vaults {
                self.insert_new_vault(component_address, vault_id, vault);
            }
        }
    }
}
