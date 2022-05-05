use sbor::*;
use indexmap::{IndexMap, IndexSet};
use scrypto::buffer::scrypto_decode;
use scrypto::constants::*;
use scrypto::engine::types::*;
use scrypto::prelude::scrypto_encode;
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
    borrowed_components: HashMap<ComponentAddress, Option<PhysicalSubstateId>>,
    borrowed_resource_managers: HashMap<ResourceAddress, Option<PhysicalSubstateId>>,
    borrowed_vaults: HashMap<(ComponentAddress, VaultId), Option<PhysicalSubstateId>>,
}

impl BorrowedSNodes {
    pub fn is_empty(&self) -> bool {
        self.borrowed_components.is_empty() &&
        self.borrowed_resource_managers.is_empty() &&
        self.borrowed_vaults.is_empty()
    }
}

pub struct TrackReceipt {
    pub borrowed: BorrowedSNodes,
    pub new_packages: Vec<PackageAddress>,
    pub new_components: Vec<ComponentAddress>,
    pub new_resources: Vec<ResourceAddress>,
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

    packages: IndexMap<PackageAddress, SubstateUpdate<Package>>,

    components: IndexMap<ComponentAddress, SubstateUpdate<Component>>,
    borrowed_components: HashMap<ComponentAddress, Option<PhysicalSubstateId>>,

    resource_managers: IndexMap<ResourceAddress, SubstateUpdate<ResourceManager>>,
    borrowed_resource_managers: HashMap<ResourceAddress, Option<PhysicalSubstateId>>,

    vaults: IndexMap<(ComponentAddress, VaultId), SubstateUpdate<Vault>>,
    borrowed_vaults: HashMap<(ComponentAddress, VaultId), Option<PhysicalSubstateId>>,

    new_spaces: IndexSet<Vec<u8>>,
    non_fungibles: IndexMap<NonFungibleAddress, KeyedSubstateUpdate<Option<NonFungible>>>,
    lazy_map_entries: IndexMap<(ComponentAddress, LazyMapId, Vec<u8>), KeyedSubstateUpdate<Vec<u8>>>,
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
            packages: IndexMap::new(),
            components: IndexMap::new(),
            borrowed_components: HashMap::new(),
            resource_managers: IndexMap::new(),
            borrowed_resource_managers: HashMap::new(),
            new_spaces: IndexSet::new(),
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

    /// Returns an immutable reference to a package, if exists.
    pub fn get_package(&mut self, package_address: &PackageAddress) -> Option<&Package> {
        if self.packages.contains_key(package_address) {
            return self.packages.get(package_address).map(|p| &p.value);
        }

        if let Some((package, substate_id)) = self.substate_store.get_decoded_substate(package_address)
        {
            self.packages.insert(
                package_address.clone(),
                SubstateUpdate {
                    prev_id: Some(substate_id),
                    value: package,
                },
            );
            self.packages.get(package_address).map(|p| &p.value)
        } else {
            None
        }
    }

    /// Inserts a new package.
    pub fn create_package(&mut self, package: Package) -> PackageAddress {
        let package_address = self.new_package_address();
        self.packages.insert(
            package_address,
            SubstateUpdate {
                prev_id: None,
                value: package,
            },
        );
        package_address
    }

    pub fn borrow_global_mut_component(
        &mut self,
        component_address: ComponentAddress,
    ) -> Result<Component, RuntimeError> {
        let maybe_component = self.components.remove(&component_address);
        if let Some(SubstateUpdate { value, prev_id }) = maybe_component {
            self.borrowed_components.insert(component_address, prev_id);
            Ok(value)
        } else if self.borrowed_components.contains_key(&component_address) {
            Err(RuntimeError::ComponentReentrancy(component_address))
        } else if let Some((component, substate_id)) =
            self.substate_store.get_decoded_substate(&component_address)
        {
            self.borrowed_components
                .insert(component_address, Some(substate_id));
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
        if let Some(prev_id) = self.borrowed_components.remove(&component_address) {
            self.components.insert(
                component_address,
                SubstateUpdate {
                    prev_id,
                    value: component,
                },
            );
        } else {
            panic!("Component was never borrowed");
        }
    }

    /// Returns an immutable reference to a component, if exists.
    pub fn get_component(&mut self, component_address: ComponentAddress) -> Option<&Component> {
        if self.components.contains_key(&component_address) {
            return self.components.get(&component_address).map(|c| &c.value);
        }

        if let Some((component, substate_id)) =
            self.substate_store.get_decoded_substate(&component_address)
        {
            self.components.insert(
                component_address,
                SubstateUpdate {
                    prev_id: Some(substate_id),
                    value: component,
                },
            );
            self.components.get(&component_address).map(|c| &c.value)
        } else {
            None
        }
    }

    /// Inserts a new component.
    pub fn create_component(&mut self, component: Component) -> ComponentAddress {
        let component_address = self.new_component_address();
        self.components.insert(
            component_address,
            SubstateUpdate {
                prev_id: None,
                value: component,
            },
        );
        component_address
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

    /// Returns an immutable reference to a resource manager, if exists.
    pub fn get_resource_manager(
        &mut self,
        resource_address: &ResourceAddress,
    ) -> Option<&ResourceManager> {
        if self.resource_managers.contains_key(resource_address) {
            return self
                .resource_managers
                .get(resource_address)
                .map(|r| &r.value);
        }

        if let Some((resource_manager, substate_id)) =
            self.substate_store.get_decoded_substate(resource_address)
        {
            self.resource_managers.insert(
                resource_address.clone(),
                SubstateUpdate {
                    prev_id: Some(substate_id),
                    value: resource_manager,
                },
            );
            self.resource_managers
                .get(resource_address)
                .map(|r| &r.value)
        } else {
            None
        }
    }

    pub fn borrow_global_mut_resource_manager(
        &mut self,
        resource_address: ResourceAddress,
    ) -> Result<ResourceManager, RuntimeError> {
        let maybe_resource = self.resource_managers.remove(&resource_address);
        if self
            .borrowed_resource_managers
            .contains_key(&resource_address)
        {
            panic!("Invalid resource manager reentrancy");
        } else if let Some(SubstateUpdate { value, prev_id }) = maybe_resource {
            self.borrowed_resource_managers
                .insert(resource_address, prev_id);
            Ok(value)
        } else if let Some((resource_manager, substate_id)) =
            self.substate_store.get_decoded_substate(&resource_address)
        {
            self.borrowed_resource_managers
                .insert(resource_address, Some(substate_id));
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
        if let Some(prev_id) = self.borrowed_resource_managers.remove(&resource_address) {
            self.resource_managers.insert(
                resource_address,
                SubstateUpdate {
                    prev_id,
                    value: resource_manager,
                },
            );
        } else {
            panic!("Resource manager was never borrowed");
        }
    }

    /// Inserts a new resource manager.
    pub fn create_resource_manager(
        &mut self,
        resource_manager: ResourceManager,
    ) -> ResourceAddress {
        let resource_address = self.new_resource_address();

        // TODO: Move this into application layer
        if let ResourceType::NonFungible = resource_manager.resource_type() {
            let space_address = resource_to_non_fungible_space!(resource_address);
            self.new_spaces.insert(space_address);
        }

        self.resource_managers.insert(
            resource_address,
            SubstateUpdate {
                prev_id: None,
                value: resource_manager,
            },
        );
        resource_address
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
        self.new_spaces.insert(space_address);
    }

    fn get_substate_parent_id(
        &mut self,
        space_address: &[u8],
    ) -> SubstateParentId {
        if let Some(index) = self.new_spaces.get_index_of(space_address) {
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
        let mut new_packages = Vec::new();
        let mut new_components = Vec::new();
        let mut new_resources = Vec::new();

        let mut store_instructions = Vec::new();
        for (package_address, package) in self.packages.drain(RangeFull) {
            if let Some(substate_id) = package.prev_id {
                store_instructions.push(SubstateOperation::Down(substate_id));
            } else {
                new_packages.push(package_address);
            }
            store_instructions.push(SubstateOperation::Up(scrypto_encode(&package_address), scrypto_encode(&package.value)));
        }
        for (component_address, component) in self.components.drain(RangeFull) {
            if let Some(substate_id) = component.prev_id {
                store_instructions.push(SubstateOperation::Down(substate_id));
            } else {
                new_components.push(component_address);
            }
            store_instructions.push(SubstateOperation::Up(scrypto_encode(&component_address), scrypto_encode(&component.value)));
        }
        for (resource_address, resource_manager) in self.resource_managers.drain(RangeFull) {
            if let Some(substate_id) = resource_manager.prev_id {
                store_instructions.push(SubstateOperation::Down(substate_id));
            } else {
                new_resources.push(resource_address);
            }
            store_instructions.push(SubstateOperation::Up(scrypto_encode(&resource_address), scrypto_encode(&resource_manager.value)));
        }
        for ((component_address, vault_id), vault) in self.vaults.drain(RangeFull) {
            if let Some(substate_id) = vault.prev_id {
                store_instructions.push(SubstateOperation::Down(substate_id));
            }
            let mut vault_address = scrypto_encode(&component_address);
            vault_address.extend(scrypto_encode(&vault_id));
            store_instructions.push(SubstateOperation::Up(vault_address, scrypto_encode(&vault.value)));
        }

        for space_address in self.new_spaces.drain(RangeFull) {
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
            borrowed_components: self.borrowed_components,
            borrowed_vaults: self.borrowed_vaults,
            borrowed_resource_managers: self.borrowed_resource_managers,
        };
        TrackReceipt {
            new_packages,
            new_components,
            new_resources,
            borrowed,
            substates,
            logs: self.logs,
        }
    }
}
