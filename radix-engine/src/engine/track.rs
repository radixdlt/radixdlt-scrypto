use scrypto::constants::*;
use scrypto::engine::types::*;
use scrypto::rust::collections::*;
use scrypto::rust::string::String;
use scrypto::rust::vec::Vec;

use crate::engine::*;
use crate::errors::RuntimeError;
use crate::ledger::*;
use crate::model::*;

pub struct CommitReceipt {
    pub down_substates: HashSet<(Hash, u32)>,
    pub up_substates: Vec<(Hash, u32)>,
}

impl CommitReceipt {
    fn new() -> Self {
        CommitReceipt {
            down_substates: HashSet::new(),
            up_substates: Vec::new(),
        }
    }

    fn down(&mut self, id: (Hash, u32)) {
        self.down_substates.insert(id);
    }

    fn up(&mut self, id: (Hash, u32)) {
        self.up_substates.push(id);
    }
}

struct SubstateUpdate<T> {
    prev_id: Option<(Hash, u32)>,
    value: T,
}

/// An abstraction of transaction execution state.
///
/// It acts as the facade of ledger state and keeps track of all temporary state updates,
/// until the `commit()` method is called.
///
/// Typically, a track is shared by all the processes created within a transaction.
///
pub struct Track<'s, S: SubstateStore> {
    substate_store: &'s mut S,
    transaction_hash: Hash,
    transaction_signers: Vec<EcdsaPublicKey>,
    id_allocator: IdAllocator,
    logs: Vec<(Level, String)>,

    packages: HashMap<PackageAddress, SubstateUpdate<Package>>,

    components: HashMap<ComponentAddress, SubstateUpdate<Component>>,
    borrowed_components: HashMap<ComponentAddress, Option<(Hash, u32)>>,

    resource_managers: HashMap<ResourceAddress, SubstateUpdate<ResourceManager>>,
    borrowed_resource_managers: HashMap<ResourceAddress, Option<(Hash, u32)>>,

    vaults: HashMap<(ComponentAddress, VaultId), SubstateUpdate<Vault>>,
    non_fungibles: HashMap<NonFungibleAddress, SubstateUpdate<NonFungible>>,

    lazy_map_entries: HashMap<(ComponentAddress, LazyMapId, Vec<u8>), SubstateUpdate<Vec<u8>>>,
}

impl<'s, S: SubstateStore> Track<'s, S> {
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
            packages: HashMap::new(),
            components: HashMap::new(),
            borrowed_components: HashMap::new(),
            resource_managers: HashMap::new(),
            borrowed_resource_managers: HashMap::new(),
            lazy_map_entries: HashMap::new(),
            vaults: HashMap::new(),
            non_fungibles: HashMap::new(),
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
        let mut process = Process::new(0, verbose, self);

        // With the latest change, proof amount can't be zero, thus a virtual proof is created
        // only if there are signers.
        //
        // Transactions that refer to the signature virtual proof will pass static check
        // but will fail at runtime, if there are no signers.
        //
        // TODO: possible to update static check to reject them early?
        if !signers.is_empty() {
            // Proofs can't be zero amount
            let ecdsa_bucket =
                Bucket::new(ResourceContainer::new_non_fungible(ECDSA_TOKEN, signers));
            process
                .create_virtual_proof(ECDSA_TOKEN_BUCKET_ID, ECDSA_TOKEN_PROOF_ID, ecdsa_bucket)
                .unwrap();
            process.push_to_auth_zone(ECDSA_TOKEN_PROOF_ID).unwrap();
        }

        process
    }

    /// Returns the transaction hash.
    pub fn transaction_hash(&self) -> Hash {
        self.transaction_hash
    }

    /// Returns the current epoch.
    pub fn current_epoch(&self) -> u64 {
        self.substate_store.get_epoch()
    }

    /// Returns the logs collected so far.
    pub fn logs(&self) -> &Vec<(Level, String)> {
        &self.logs
    }

    /// Returns new packages created so far.
    pub fn new_package_addresses(&self) -> Vec<PackageAddress> {
        let mut package_addresses = Vec::new();
        for (package_address, update) in self.packages.iter() {
            if let None = update.prev_id {
                package_addresses.push(package_address.clone());
            }
        }
        package_addresses
    }

    /// Returns new components created so far.
    pub fn new_component_addresses(&self) -> Vec<ComponentAddress> {
        let mut component_addresses = Vec::new();
        for (component_address, update) in self.components.iter() {
            if let None = update.prev_id {
                component_addresses.push(component_address.clone());
            }
        }
        component_addresses
    }

    /// Returns new resource addresses created so far.
    pub fn new_resource_addresses(&self) -> Vec<ResourceAddress> {
        let mut resource_addresses = Vec::new();
        for (resource_address, update) in self.resource_managers.iter() {
            if let None = update.prev_id {
                resource_addresses.push(resource_address.clone());
            }
        }
        resource_addresses
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

        if let Some((package, phys_id)) = self.substate_store.get_decoded_substate(package_address)
        {
            self.packages.insert(
                package_address.clone(),
                SubstateUpdate {
                    prev_id: Some(phys_id),
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
        } else if let Some((component, phys_id)) =
            self.substate_store.get_decoded_substate(&component_address)
        {
            self.borrowed_components
                .insert(component_address, Some(phys_id));
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

        if let Some((component, phys_id)) =
            self.substate_store.get_decoded_substate(&component_address)
        {
            self.components.insert(
                component_address,
                SubstateUpdate {
                    prev_id: Some(phys_id),
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
    ) -> Option<&NonFungible> {
        if self.non_fungibles.contains_key(non_fungible_address) {
            return self
                .non_fungibles
                .get(non_fungible_address)
                .map(|s| &s.value);
        }

        if let Some((non_fungible, phys_id)) = self.substate_store.get_decoded_child_substate(
            &non_fungible_address.resource_address(),
            &non_fungible_address.non_fungible_id(),
        ) {
            self.non_fungibles.insert(
                non_fungible_address.clone(),
                SubstateUpdate {
                    prev_id: Some(phys_id),
                    value: non_fungible,
                },
            );
            self.non_fungibles
                .get(non_fungible_address)
                .map(|s| &s.value)
        } else {
            None
        }
    }

    /// Returns a mutable reference to a non-fungible, if exists.
    pub fn get_non_fungible_mut(
        &mut self,
        non_fungible_address: &NonFungibleAddress,
    ) -> Option<&mut NonFungible> {
        if self.non_fungibles.contains_key(non_fungible_address) {
            return self
                .non_fungibles
                .get_mut(non_fungible_address)
                .map(|s| &mut s.value);
        }

        if let Some((non_fungible, phys_id)) = self.substate_store.get_decoded_child_substate(
            &non_fungible_address.resource_address(),
            &non_fungible_address.non_fungible_id(),
        ) {
            self.non_fungibles.insert(
                non_fungible_address.clone(),
                SubstateUpdate {
                    prev_id: Some(phys_id),
                    value: non_fungible,
                },
            );
            self.non_fungibles
                .get_mut(non_fungible_address)
                .map(|s| &mut s.value)
        } else {
            None
        }
    }

    /// Inserts a new non-fungible.
    pub fn put_non_fungible(
        &mut self,
        non_fungible_address: NonFungibleAddress,
        non_fungible: NonFungible,
    ) {
        self.non_fungibles.insert(
            non_fungible_address,
            SubstateUpdate {
                prev_id: None,
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
        let value = self.substate_store.get_decoded_grand_child_substate(
            &component_address,
            lazy_map_id,
            &grand_child_key,
        );
        if let Some((ref entry_bytes, phys_id)) = value {
            self.lazy_map_entries.insert(
                canonical_id,
                SubstateUpdate {
                    prev_id: Some(phys_id),
                    value: entry_bytes.clone(),
                },
            );
        }
        value.map(|r| r.0)
    }

    pub fn put_lazy_map_entry(
        &mut self,
        component_address: ComponentAddress,
        lazy_map_id: LazyMapId,
        key: Vec<u8>,
        value: Vec<u8>,
    ) {
        let canonical_id = (component_address.clone(), lazy_map_id.clone(), key.clone());

        if !self.lazy_map_entries.contains_key(&canonical_id) {
            let entry = self.substate_store.get_decoded_grand_child_substate(
                &component_address,
                &lazy_map_id,
                &key,
            );
            if let Some((_, phys_id)) = entry {
                self.lazy_map_entries.insert(
                    canonical_id,
                    SubstateUpdate {
                        prev_id: Some(phys_id),
                        value,
                    },
                );
                return;
            }
        }

        if let Some(entry) = self.lazy_map_entries.get_mut(&canonical_id) {
            entry.value = value;
        } else {
            // TODO: Virtual Down
            self.lazy_map_entries.insert(
                canonical_id,
                SubstateUpdate {
                    prev_id: None,
                    value,
                },
            );
        }
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

        if let Some((resource_manager, phys_id)) =
            self.substate_store.get_decoded_substate(resource_address)
        {
            self.resource_managers.insert(
                resource_address.clone(),
                SubstateUpdate {
                    prev_id: Some(phys_id),
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
        } else if let Some((resource_manager, phys_id)) =
            self.substate_store.get_decoded_substate(&resource_address)
        {
            self.borrowed_resource_managers
                .insert(resource_address, Some(phys_id));
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

    /// Returns a mutable reference to a resource manager, if exists.
    // TODO: Remove
    #[allow(dead_code)]
    pub fn get_resource_manager_mut(
        &mut self,
        resource_address: &ResourceAddress,
    ) -> Option<&mut ResourceManager> {
        if self.resource_managers.contains_key(resource_address) {
            return self
                .resource_managers
                .get_mut(resource_address)
                .map(|r| &mut r.value);
        }

        if let Some((resource_manager, phys_id)) =
            self.substate_store.get_decoded_substate(resource_address)
        {
            self.resource_managers.insert(
                resource_address.clone(),
                SubstateUpdate {
                    prev_id: Some(phys_id),
                    value: resource_manager,
                },
            );
            self.resource_managers
                .get_mut(resource_address)
                .map(|r| &mut r.value)
        } else {
            None
        }
    }

    /// Inserts a new resource manager.
    pub fn create_resource_manager(
        &mut self,
        resource_manager: ResourceManager,
    ) -> ResourceAddress {
        let resource_address = self.new_resource_address();
        self.resource_managers.insert(
            resource_address,
            SubstateUpdate {
                prev_id: None,
                value: resource_manager,
            },
        );
        resource_address
    }

    /// Returns a mutable reference to a vault, if exists.
    pub fn get_vault_mut(
        &mut self,
        component_address: &ComponentAddress,
        vid: &VaultId,
    ) -> &mut Vault {
        let canonical_id = (component_address.clone(), vid.clone());

        if self.vaults.contains_key(&canonical_id) {
            return self
                .vaults
                .get_mut(&canonical_id)
                .map(|r| &mut r.value)
                .unwrap();
        }

        let (vault, phys_id) = self
            .substate_store
            .get_decoded_child_substate(component_address, vid)
            .unwrap();
        self.vaults.insert(
            canonical_id,
            SubstateUpdate {
                prev_id: Some(phys_id),
                value: vault,
            },
        );
        self.vaults
            .get_mut(&canonical_id)
            .map(|r| &mut r.value)
            .unwrap()
    }

    /// Inserts a new vault.
    pub fn put_vault(
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
    pub fn commit(&mut self) -> CommitReceipt {
        // Sanity check
        if !self.borrowed_components.is_empty() {
            panic!("Borrowed components should be empty by end of transaction.");
        }
        if !self.borrowed_resource_managers.is_empty() {
            panic!("Borrowed resource managers should be empty by end of transaction.");
        }

        let mut receipt = CommitReceipt::new();
        let mut id_gen = SubstateIdGenerator::new(self.transaction_hash());

        let package_addresses: Vec<PackageAddress> = self.packages.keys().cloned().collect();
        for package_address in package_addresses {
            let package = self.packages.remove(&package_address).unwrap();

            if let Some(prev_id) = package.prev_id {
                receipt.down(prev_id);
            }
            let phys_id = id_gen.next();
            receipt.up(phys_id);

            self.substate_store
                .put_encoded_substate(&package_address, &package.value, phys_id);
        }

        let component_addresses: Vec<ComponentAddress> = self.components.keys().cloned().collect();
        for component_address in component_addresses {
            let component = self.components.remove(&component_address).unwrap();

            if let Some(prev_id) = component.prev_id {
                receipt.down(prev_id);
            }
            let phys_id = id_gen.next();
            receipt.up(phys_id);

            self.substate_store
                .put_encoded_substate(&component_address, &component.value, phys_id);
        }

        let resource_addresses: Vec<ResourceAddress> =
            self.resource_managers.keys().cloned().collect();
        for resource_address in resource_addresses {
            let resource_manager = self.resource_managers.remove(&resource_address).unwrap();

            if let Some(prev_id) = resource_manager.prev_id {
                receipt.down(prev_id);
            }
            let phys_id = id_gen.next();
            receipt.up(phys_id);

            self.substate_store.put_encoded_substate(
                &resource_address,
                &resource_manager.value,
                phys_id,
            );
        }

        let entry_ids: Vec<(ComponentAddress, LazyMapId, Vec<u8>)> =
            self.lazy_map_entries.keys().cloned().collect();
        for entry_id in entry_ids {
            let entry = self.lazy_map_entries.remove(&entry_id).unwrap();
            if let Some(prev_id) = entry.prev_id {
                receipt.down(prev_id);
            }
            let phys_id = id_gen.next();
            receipt.up(phys_id);

            let (component_address, lazy_map_id, key) = entry_id;
            self.substate_store.put_encoded_grand_child_substate(
                &component_address,
                &lazy_map_id,
                &key,
                &entry.value,
                phys_id,
            );
        }

        let vault_ids: Vec<(ComponentAddress, VaultId)> = self.vaults.keys().cloned().collect();
        for vault_id in vault_ids {
            let vault = self.vaults.remove(&vault_id).unwrap();
            if let Some(prev_id) = vault.prev_id {
                receipt.down(prev_id);
            }
            let phys_id = id_gen.next();
            receipt.up(phys_id);

            let (component_address, vault_id) = vault_id;
            self.substate_store.put_encoded_child_substate(
                &component_address,
                &vault_id,
                &vault.value,
                phys_id,
            );
        }

        let non_fungible_addresses: Vec<NonFungibleAddress> =
            self.non_fungibles.keys().cloned().collect();
        for non_fungible_address in non_fungible_addresses {
            let non_fungible = self.non_fungibles.remove(&non_fungible_address).unwrap();
            if let Some(prev_id) = non_fungible.prev_id {
                receipt.down(prev_id);
            }
            let phys_id = id_gen.next();
            receipt.up(phys_id);

            self.substate_store.put_encoded_child_substate(
                &non_fungible_address.resource_address(),
                &non_fungible_address.non_fungible_id(),
                &non_fungible.value,
                phys_id,
            );
        }

        receipt
    }
}
