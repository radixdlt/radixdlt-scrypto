use sbor::*;
use indexmap::{IndexMap, IndexSet};
use scrypto::buffer::scrypto_decode;
use scrypto::constants::*;
use scrypto::engine::types::*;
use scrypto::prelude::{scrypto_encode};
use scrypto::rust::ops::RangeFull;
use scrypto::rust::collections::*;
use scrypto::rust::string::String;
use scrypto::rust::vec::Vec;

use crate::engine::*;
use crate::errors::RuntimeError;
use crate::ledger::*;
use crate::model::*;


// TODO: Replace NonFungible with real re address
// TODO: Move this logic into application layer
macro_rules! resource_to_non_fungible_space {
    ($resource_address:expr) => {{
        let mut addr = scrypto_encode(&$resource_address);
        addr.push(0u8);
        addr
    }};
}

// TODO: Replace NonFungible with real re address
// TODO: Move this logic into application layer
macro_rules! non_fungible_to_re_address {
    ($non_fungible:expr) => {{
        let mut addr = resource_to_non_fungible_space!($non_fungible.resource_address());
        addr.extend($non_fungible.non_fungible_id().to_vec());
        addr
    }};
}

pub struct BorrowedSNodes {
    borrowed_substates: HashSet<Address>,
    borrowed_vaults: HashMap<(ComponentAddress, VaultId), Option<PhysicalSubstateId>>,
}

impl BorrowedSNodes {
    pub fn is_empty(&self) -> bool {
        self.borrowed_substates.is_empty() &&
        self.borrowed_vaults.is_empty()
    }
}

pub struct TrackReceipt {
    pub borrowed: BorrowedSNodes,
    pub new_addresses: Vec<Address>,
    pub logs: Vec<(Level, String)>,
    pub substates: SubstateOperationsReceipt,
}


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


pub enum KeyedSubstateId {
    Physical(PhysicalSubstateId),
    Virtual(VirtualSubstateId),
}

pub struct KeyedSubstateUpdate<T> {
    pub prev_id: KeyedSubstateId,
    pub value: T,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Address {
    Resource(ResourceAddress),
    Component(ComponentAddress),
    Package(PackageAddress),
}

impl Address {
    fn encode(&self) -> Vec<u8> {
        match self {
            Address::Resource(resource_address) => scrypto_encode(resource_address),
            Address::Component(component_address) => scrypto_encode(component_address),
            Address::Package(package_address) => scrypto_encode(package_address),
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

pub enum SubstateValue {
    Resource(ResourceManager),
    Component(Component),
    Package(Package),
}

impl SubstateValue {
    fn encode(&self) -> Vec<u8> {
        match self {
            SubstateValue::Resource(resource_manager) => scrypto_encode(resource_manager),
            SubstateValue::Package(package) => scrypto_encode(package),
            SubstateValue::Component(component) => scrypto_encode(component),
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


/// An abstraction of transaction execution state.
///
/// It acts as the facade of ledger state and keeps track of all temporary state updates,
/// until the `commit()` method is called.
///
/// Typically, a track is shared by all the processes created within a transaction.
///
pub struct Track<'s, S: ReadableSubstateStore> {
    substate_store: &'s mut S,
    transaction_hash: Hash,
    transaction_signers: Vec<EcdsaPublicKey>,
    id_allocator: IdAllocator,
    logs: Vec<(Level, String)>,

    new_addresses: Vec<Address>,
    borrowed_substates: HashSet<Address>,
    read_substates: IndexMap<Address, SubstateValue>,

    downed_substates: Vec<PhysicalSubstateId>,
    up_substates: IndexMap<Address, SubstateValue>,
    up_virtual_substate_space: IndexSet<Vec<u8>>,

    non_fungibles: IndexMap<NonFungibleAddress, KeyedSubstateUpdate<Option<NonFungible>>>,
    lazy_map_entries: IndexMap<(ComponentAddress, LazyMapId, Vec<u8>), KeyedSubstateUpdate<Vec<u8>>>,

    // TODO: Change this interface to take/put
    vaults: IndexMap<(ComponentAddress, VaultId), SubstateUpdate<Vault>>,
    borrowed_vaults: HashMap<(ComponentAddress, VaultId), Option<PhysicalSubstateId>>,
}

impl<'s, S: ReadableSubstateStore> Track<'s, S> {
    pub fn new(
        substate_store: &'s mut S,
        transaction_hash: Hash,
        transaction_signers: Vec<EcdsaPublicKey>,
    ) -> Self {
        Self {
            substate_store,
            transaction_hash,
            transaction_signers,
            id_allocator: IdAllocator::new(IdSpace::Application),
            logs: Vec::new(),

            new_addresses: Vec::new(),
            borrowed_substates: HashSet::new(),
            read_substates: IndexMap::new(),

            downed_substates: Vec::new(),
            up_substates: IndexMap::new(),
            up_virtual_substate_space: IndexSet::new(),

            lazy_map_entries: IndexMap::new(),
            vaults: IndexMap::new(),
            borrowed_vaults: HashMap::new(),
            non_fungibles: IndexMap::new(),
        }
    }

    /// Start a process.
    pub fn start_process<'r>(&'r mut self, verbose: bool) -> Process<'r, 's, S> {
        let signers: BTreeSet<NonFungibleId> = self
            .transaction_signers
            .clone()
            .into_iter()
            .map(|public_key| NonFungibleId::from_bytes(public_key.to_vec()))
            .collect();

        // With the latest change, proof amount can't be zero, thus a virtual proof is created
        // only if there are signers.
        //
        // Transactions that refer to the signature virtual proof will pass static check
        // but will fail at runtime, if there are no signers.
        //
        // TODO: possible to update static check to reject them early?
        let mut initial_auth_zone_proofs = Vec::new();
        if !signers.is_empty() {
            // Proofs can't be zero amount
            let mut ecdsa_bucket =
                Bucket::new(ResourceContainer::new_non_fungible(ECDSA_TOKEN, signers));
            let ecdsa_proof = ecdsa_bucket.create_proof(ECDSA_TOKEN_BUCKET_ID).unwrap();
            initial_auth_zone_proofs.push(ecdsa_proof);
        }

        Process::new(
            0,
            verbose,
            self,
            Some(AuthZone::new_with_proofs(initial_auth_zone_proofs)),
            Some(Worktop::new()),
            HashMap::new(),
            HashMap::new(),
        )
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

    /// Inserts a new package.
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
        };

        self.new_addresses.push(address.clone());
        self.up_substates.insert(address.clone(), substate_value);
        address
    }

    /// Returns an immutable reference to a value, if exists.
    pub fn read_value<A: Into<Address>>(&mut self, addr: A) -> Option<&SubstateValue> {
        let address: Address = addr.into();

        if let Some(v) = self.up_substates.get(&address) {
            return Some(v);
        }

        match address {
            Address::Package(package_address) => {
                if let Some(package) = self.substate_store.get_decoded_substate(&package_address)
                    .map(|(package, _)| package) {
                    self.read_substates.insert(address.clone(), SubstateValue::Package(package));
                    self.read_substates.get(&address)
                } else {
                    None
                }
            }
            Address::Component(component_address) => {
                if let Some(component) = self.substate_store.get_decoded_substate(&component_address)
                    .map(|(component, _)| component) {
                    self.read_substates.insert(address.clone(), SubstateValue::Component(component));
                    self.read_substates.get(&address)
                } else {
                    None
                }
            }
            Address::Resource(resource_address) => {
                if let Some(resource_manager) = self.substate_store.get_decoded_substate(&resource_address)
                    .map(|(resource_manager, _)| resource_manager) {
                    self.read_substates.insert(address.clone(), SubstateValue::Resource(resource_manager));
                    self.read_substates.get(&address)
                } else {
                    None
                }
            }
        }
    }

    pub fn borrow_global_mut_component(
        &mut self,
        component_address: ComponentAddress,
    ) -> Result<Component, RuntimeError> {
        let address = Address::Component(component_address);
        let maybe_value = self.up_substates.remove(&address);
        if let Some(value) = maybe_value {
            self.borrowed_substates.insert(address);
            Ok(value.into())
        } else if self.borrowed_substates.contains(&address) {
            Err(RuntimeError::ComponentReentrancy(component_address))
        } else if let Some((component, substate_id)) =
            self.substate_store.get_decoded_substate(&component_address)
        {
            self.downed_substates.push(substate_id);
            self.borrowed_substates.insert(address);
            Ok(component)
        } else {
            Err(RuntimeError::ComponentNotFound(component_address))
        }
    }

    pub fn return_borrowed_global_component(
        &mut self,
        component_address: ComponentAddress,
        component: Component,
    ) {
        let address = Address::Component(component_address);

        if !self.borrowed_substates.remove(&address) {
            panic!("Component was never borrowed");
        }

        self.up_substates.insert(address, SubstateValue::Component(component));
    }

    pub fn borrow_global_mut_resource_manager(
        &mut self,
        resource_address: ResourceAddress,
    ) -> Result<ResourceManager, RuntimeError> {
        let address = Address::Resource(resource_address);
        let maybe_value = self.up_substates.remove(&address);
        if let Some(value) = maybe_value {
            self.borrowed_substates.insert(address);
            Ok(value.into())
        } else if self.borrowed_substates.contains(&address) {
            panic!("Invalid resource manager reentrancy");
        } else if let Some((resource_manager, substate_id)) = self.substate_store.get_decoded_substate(&resource_address) {
            self.downed_substates.push(substate_id);
            self.borrowed_substates.insert(address);
            Ok(resource_manager)
        } else {
            Err(RuntimeError::ResourceManagerNotFound(resource_address))
        }
    }

    pub fn return_borrowed_global_resource_manager(
        &mut self,
        resource_address: ResourceAddress,
        resource_manager: ResourceManager,
    ) {
        let address = Address::Resource(resource_address);
        if !self.borrowed_substates.remove(&address) {
            panic!("Resource Manager was never borrowed");
        }
        self.up_substates.insert(address, SubstateValue::Resource(resource_manager));
    }

    /// Returns an immutable reference to a non-fungible, if exists.
    pub fn get_non_fungible(
        &mut self,
        non_fungible_address: &NonFungibleAddress,
    ) -> Option<NonFungible> {
        if let Some(cur) = self.non_fungibles.get(non_fungible_address) {
            return cur.value.as_ref().map(|n| n.clone())
        }

        let nf_address = non_fungible_to_re_address!(non_fungible_address);
        self.substate_store.get_substate(&nf_address).map(|r| scrypto_decode(&r.value).unwrap())
    }

    /// Sets a non-fungible.
    pub fn set_non_fungible(
        &mut self,
        non_fungible_address: NonFungibleAddress,
        non_fungible: Option<NonFungible>,
    ) {
        let nf_address = non_fungible_to_re_address!(non_fungible_address);
        let cur: Option<Substate> = self.substate_store.get_substate(&nf_address);
        let prev_id = if let Some(Substate { value: _, phys_id }) = cur {
            KeyedSubstateId::Physical(PhysicalSubstateId(phys_id.0, phys_id.1))
        } else {
            let space_address = resource_to_non_fungible_space!(non_fungible_address.resource_address());
            let parent_id = self.get_substate_parent_id(&space_address);

            KeyedSubstateId::Virtual(VirtualSubstateId(parent_id, non_fungible_address.non_fungible_id().to_vec()))
        };

        self.non_fungibles.insert(
            non_fungible_address,
            KeyedSubstateUpdate {
                prev_id,
                value: non_fungible,
            },
        );
    }

    pub fn get_lazy_map_entry(
        &mut self,
        component_address: ComponentAddress,
        lazy_map_id: &LazyMapId,
        key: &[u8],
    ) -> Option<Vec<u8>> {
        let canonical_id = (component_address.clone(), lazy_map_id.clone(), key.to_vec());

        if self.lazy_map_entries.contains_key(&canonical_id) {
            return Some(
                self.lazy_map_entries
                    .get(&canonical_id)
                    .map(|r| r.value.clone())
                    .unwrap(),
            );
        }

        let grand_child_key = key.to_vec();
        self.substate_store.get_decoded_grand_child_substate(
            &component_address,
            lazy_map_id,
            &grand_child_key,
        ).map(|r| r.0)
    }

    pub fn put_lazy_map_entry(
        &mut self,
        component_address: ComponentAddress,
        lazy_map_id: LazyMapId,
        key: Vec<u8>,
        value: Vec<u8>,
    ) {
        let canonical_id = (component_address.clone(), lazy_map_id.clone(), key.clone());
        let entry = self.substate_store.get_decoded_grand_child_substate(
            &component_address,
            &lazy_map_id,
            &key,
        );
        let prev_id = if let Some((_, substate_id)) = entry {
            KeyedSubstateId::Physical(substate_id)
        } else {
            let mut space_address = scrypto_encode(&component_address);
            space_address.extend(scrypto_encode(&lazy_map_id));
            let parent_id = self.get_substate_parent_id(&space_address);
            KeyedSubstateId::Virtual(VirtualSubstateId(parent_id, key.to_vec()))
        };

        self.lazy_map_entries.insert(
            canonical_id,
            KeyedSubstateUpdate {
                prev_id,
                value,
            },
        );
    }


    pub fn borrow_vault_mut(&mut self, component_address: &ComponentAddress, vid: &VaultId) -> Vault {
        let canonical_id = (component_address.clone(), vid.clone());
        if self.borrowed_vaults.contains_key(&canonical_id) {
            panic!("Invalid vault reentrancy");
        }

        if let Some(SubstateUpdate { value, prev_id }) = self.vaults.remove(&canonical_id) {
            self.borrowed_vaults.insert(canonical_id, prev_id);
            return value;
        }

        if let Some((vault, substate_id)) = self.substate_store.get_decoded_child_substate(component_address, vid) {
            self.borrowed_vaults
                .insert(canonical_id, Some(substate_id));
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

    fn get_substate_parent_id(
        &mut self,
        space_address: &[u8],
    ) -> SubstateParentId {
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
        for (address, value) in self.up_substates.drain(RangeFull) {
            store_instructions.push(SubstateOperation::Up(address.encode(), value.encode()));
        }

        for ((component_address, vault_id), vault) in self.vaults.drain(RangeFull) {
            if let Some(substate_id) = vault.prev_id {
                store_instructions.push(SubstateOperation::Down(substate_id));
            }
            let mut vault_address = scrypto_encode(&component_address);
            vault_address.extend(scrypto_encode(&vault_id));
            store_instructions.push(SubstateOperation::Up(vault_address, scrypto_encode(&vault.value)));
        }

        for space_address in self.up_virtual_substate_space.drain(RangeFull) {
            store_instructions.push(SubstateOperation::VirtualUp(space_address));
        }

        for (addr, update) in self.non_fungibles.drain(RangeFull) {
            match update.prev_id {
                KeyedSubstateId::Physical(physical_substate_id) => {
                    store_instructions.push(SubstateOperation::Down(physical_substate_id));
                },
                KeyedSubstateId::Virtual(virtual_substate_id) => {
                    store_instructions.push(SubstateOperation::VirtualDown(virtual_substate_id));
                }
            }

            let non_fungible_address = non_fungible_to_re_address!(addr);
            store_instructions.push(SubstateOperation::Up(non_fungible_address, scrypto_encode(&update.value)));
        }
        for ((component_address, lazy_map_id, key), entry) in self.lazy_map_entries.drain(RangeFull) {
            match entry.prev_id {
                KeyedSubstateId::Physical(physical_substate_id) => {
                    store_instructions.push(SubstateOperation::Down(physical_substate_id));
                },
                KeyedSubstateId::Virtual(virtual_substate_id) => {
                    store_instructions.push(SubstateOperation::VirtualDown(virtual_substate_id));
                }
            }

            let mut entry_address = scrypto_encode(&component_address);
            entry_address.extend(scrypto_encode(&lazy_map_id));
            entry_address.extend(key);
            store_instructions.push(SubstateOperation::Up(entry_address, entry.value));
        }

        let substates = SubstateOperationsReceipt { substate_operations: store_instructions };
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
}
