use lru::LruCache;
use scrypto::constants::*;
use scrypto::engine::types::*;
use scrypto::prelude::NonFungibleAddress;
use scrypto::rust::collections::*;
use scrypto::rust::string::String;
use scrypto::rust::vec::Vec;
use wasmi::*;

use crate::engine::*;
use crate::ledger::*;
use crate::model::*;

pub struct CommitReceipt {
    pub down_substates: HashSet<u64>,
    pub up_substates: Vec<u64>,
}

impl CommitReceipt {
    fn new() -> Self {
        CommitReceipt {
            down_substates: HashSet::new(),
            up_substates: Vec::new(),
        }
    }

    fn down(&mut self, id: u64) {
        self.down_substates.insert(id);
    }

    fn up(&mut self, id: u64) {
        self.up_substates.push(id);
    }
}

struct SubstateUpdate<T> {
    prev_id: Option<u64>,
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

    packages: HashMap<PackageId, SubstateUpdate<Package>>,
    components: HashMap<ComponentId, SubstateUpdate<Component>>,
    resource_defs: HashMap<ResourceDefId, SubstateUpdate<ResourceDef>>,

    vaults: HashMap<(ComponentId, VaultId), SubstateUpdate<Vault>>,
    non_fungibles: HashMap<NonFungibleAddress, SubstateUpdate<NonFungible>>,

    lazy_map_entries: HashMap<(ComponentId, LazyMapId, Vec<u8>), SubstateUpdate<Vec<u8>>>,

    code_cache: LruCache<PackageId, Module>, // TODO: move to ledger level
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
            resource_defs: HashMap::new(),
            lazy_map_entries: HashMap::new(),
            vaults: HashMap::new(),
            non_fungibles: HashMap::new(),
            code_cache: LruCache::new(1024),
        }
    }

    /// Start a process.
    pub fn start_process<'r>(&'r mut self, verbose: bool) -> Process<'r, 's, S> {
        // FIXME: This is a temp solution
        let signers: BTreeSet<NonFungibleKey> = self
            .transaction_signers
            .clone()
            .into_iter()
            .map(|public_key| NonFungibleKey::new(public_key.to_vec()))
            .collect();
        let mut process = Process::new(0, verbose, self);

        // Always create a virtual bucket of signatures even if there is none.
        // This is to make reasoning at transaction manifest & validator easier.
        let ecdsa_bucket = Bucket::new(
            ECDSA_TOKEN,
            ResourceType::NonFungible,
            Resource::NonFungible { keys: signers },
        );
        process.create_virtual_proof(ECDSA_TOKEN_BUCKET_ID, ECDSA_TOKEN_PROOF_ID, ecdsa_bucket);

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
    pub fn new_package_ids(&self) -> Vec<PackageId> {
        let mut package_ids = Vec::new();
        for (package_id, update) in self.packages.iter() {
            if let None = update.prev_id {
                package_ids.push(package_id.clone());
            }
        }
        package_ids
    }

    /// Returns new components created so far.
    pub fn new_component_ids(&self) -> Vec<ComponentId> {
        let mut component_ids = Vec::new();
        for (component_id, update) in self.components.iter() {
            if let None = update.prev_id {
                component_ids.push(component_id.clone());
            }
        }
        component_ids
    }

    /// Returns new resource defs created so far.
    pub fn new_resource_def_ids(&self) -> Vec<ResourceDefId> {
        let mut resource_def_ids = Vec::new();
        for (resource_def_id, update) in self.resource_defs.iter() {
            if let None = update.prev_id {
                resource_def_ids.push(resource_def_id.clone());
            }
        }
        resource_def_ids
    }

    /// Adds a log message.
    pub fn add_log(&mut self, level: Level, message: String) {
        self.logs.push((level, message));
    }

    /// Loads a module.
    pub fn load_module(&mut self, package_id: PackageId) -> Option<(ModuleRef, MemoryRef)> {
        match self.get_package(package_id).map(Clone::clone) {
            Some(p) => {
                if let Some(m) = self.code_cache.get(&package_id) {
                    Some(instantiate_module(m).unwrap())
                } else {
                    let module = parse_module(p.code()).unwrap();
                    let inst = instantiate_module(&module).unwrap();
                    self.code_cache.put(package_id, module);
                    Some(inst)
                }
            }
            None => None,
        }
    }

    /// Returns an immutable reference to a package, if exists.
    pub fn get_package(&mut self, package_id: PackageId) -> Option<&Package> {
        if self.packages.contains_key(&package_id) {
            return self.packages.get(&package_id).map(|p| &p.value);
        }

        if let Some((package, phys_id)) = self.substate_store.get_decoded_substate(&package_id) {
            self.packages.insert(
                package_id,
                SubstateUpdate {
                    prev_id: Some(phys_id),
                    value: package,
                },
            );
            self.packages.get(&package_id).map(|p| &p.value)
        } else {
            None
        }
    }

    /// Inserts a new package.
    pub fn create_package(&mut self, package: Package) -> PackageId {
        let package_id = self.new_package_id();
        self.packages.insert(
            package_id,
            SubstateUpdate {
                prev_id: None,
                value: package,
            },
        );
        package_id
    }

    /// Returns an immutable reference to a component, if exists.
    pub fn get_component(&mut self, component_id: ComponentId) -> Option<&Component> {
        if self.components.contains_key(&component_id) {
            return self.components.get(&component_id).map(|c| &c.value);
        }

        if let Some((component, phys_id)) = self.substate_store.get_decoded_substate(&component_id)
        {
            self.components.insert(
                component_id,
                SubstateUpdate {
                    prev_id: Some(phys_id),
                    value: component,
                },
            );
            self.components.get(&component_id).map(|c| &c.value)
        } else {
            None
        }
    }

    /// Returns a mutable reference to a component, if exists.
    pub fn get_component_mut(&mut self, component_id: ComponentId) -> Option<&mut Component> {
        if self.components.contains_key(&component_id) {
            return self.components.get_mut(&component_id).map(|c| &mut c.value);
        }

        if let Some((component, phys_id)) = self.substate_store.get_decoded_substate(&component_id)
        {
            self.components.insert(
                component_id,
                SubstateUpdate {
                    prev_id: Some(phys_id),
                    value: component,
                },
            );
            self.components.get_mut(&component_id).map(|c| &mut c.value)
        } else {
            None
        }
    }

    /// Inserts a new component.
    pub fn create_component(&mut self, component: Component) -> ComponentId {
        let component_id = self.new_component_id();
        self.components.insert(
            component_id,
            SubstateUpdate {
                prev_id: None,
                value: component,
            },
        );
        component_id
    }

    /// Returns an immutable reference to a non-fungible, if exists.
    pub fn get_non_fungible(&mut self, non_fungible_id: &NonFungibleAddress) -> Option<&NonFungible> {
        if self.non_fungibles.contains_key(non_fungible_id) {
            return self.non_fungibles.get(non_fungible_id).map(|s| &s.value);
        }

        if let Some((non_fungible, phys_id)) = self
            .substate_store
            .get_decoded_child_substate(&non_fungible_id.resource_def_id(), &non_fungible_id.key())
        {
            self.non_fungibles.insert(
                non_fungible_id.clone(),
                SubstateUpdate {
                    prev_id: Some(phys_id),
                    value: non_fungible,
                },
            );
            self.non_fungibles.get(non_fungible_id).map(|s| &s.value)
        } else {
            None
        }
    }

    /// Returns a mutable reference to a non-fungible, if exists.
    pub fn get_non_fungible_mut(
        &mut self,
        non_fungible_id: &NonFungibleAddress,
    ) -> Option<&mut NonFungible> {
        if self.non_fungibles.contains_key(non_fungible_id) {
            return self
                .non_fungibles
                .get_mut(non_fungible_id)
                .map(|s| &mut s.value);
        }

        if let Some((non_fungible, phys_id)) = self
            .substate_store
            .get_decoded_child_substate(&non_fungible_id.resource_def_id(), &non_fungible_id.key())
        {
            self.non_fungibles.insert(
                non_fungible_id.clone(),
                SubstateUpdate {
                    prev_id: Some(phys_id),
                    value: non_fungible,
                },
            );
            self.non_fungibles
                .get_mut(non_fungible_id)
                .map(|s| &mut s.value)
        } else {
            None
        }
    }

    /// Inserts a new non-fungible.
    pub fn put_non_fungible(&mut self, non_fungible_id: NonFungibleAddress, non_fungible: NonFungible) {
        self.non_fungibles.insert(
            non_fungible_id,
            SubstateUpdate {
                prev_id: None,
                value: non_fungible,
            },
        );
    }

    pub fn get_lazy_map_entry(
        &mut self,
        component_id: ComponentId,
        lazy_map_id: &LazyMapId,
        key: &[u8],
    ) -> Option<Vec<u8>> {
        let canonical_id = (component_id.clone(), lazy_map_id.clone(), key.to_vec());

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
            &component_id,
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
        component_id: ComponentId,
        lazy_map_id: LazyMapId,
        key: Vec<u8>,
        value: Vec<u8>,
    ) {
        let canonical_id = (component_id.clone(), lazy_map_id.clone(), key.clone());

        if !self.lazy_map_entries.contains_key(&canonical_id) {
            let entry = self.substate_store.get_decoded_grand_child_substate(
                &component_id,
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

    /// Returns an immutable reference to a resource definition, if exists.
    pub fn get_resource_def(&mut self, resource_def_id: &ResourceDefId) -> Option<&ResourceDef> {
        if self.resource_defs.contains_key(resource_def_id) {
            return self.resource_defs.get(resource_def_id).map(|r| &r.value);
        }

        if let Some((resource_def, phys_id)) =
            self.substate_store.get_decoded_substate(resource_def_id)
        {
            self.resource_defs.insert(
                resource_def_id.clone(),
                SubstateUpdate {
                    prev_id: Some(phys_id),
                    value: resource_def,
                },
            );
            self.resource_defs.get(resource_def_id).map(|r| &r.value)
        } else {
            None
        }
    }

    /// Returns a mutable reference to a resource definition, if exists.
    #[allow(dead_code)]
    pub fn get_resource_def_mut(
        &mut self,
        resource_def_id: &ResourceDefId,
    ) -> Option<&mut ResourceDef> {
        if self.resource_defs.contains_key(resource_def_id) {
            return self
                .resource_defs
                .get_mut(resource_def_id)
                .map(|r| &mut r.value);
        }

        if let Some((resource_def, phys_id)) =
            self.substate_store.get_decoded_substate(resource_def_id)
        {
            self.resource_defs.insert(
                resource_def_id.clone(),
                SubstateUpdate {
                    prev_id: Some(phys_id),
                    value: resource_def,
                },
            );
            self.resource_defs
                .get_mut(resource_def_id)
                .map(|r| &mut r.value)
        } else {
            None
        }
    }

    /// Inserts a new resource definition.
    pub fn create_resource_def(&mut self, resource_def: ResourceDef) -> ResourceDefId {
        let resource_def_id = self.new_resource_def_id();
        self.resource_defs.insert(
            resource_def_id,
            SubstateUpdate {
                prev_id: None,
                value: resource_def,
            },
        );
        resource_def_id
    }

    /// Returns a mutable reference to a vault, if exists.
    pub fn get_vault_mut(&mut self, component_id: &ComponentId, vid: &VaultId) -> &mut Vault {
        let canonical_id = (component_id.clone(), vid.clone());

        if self.vaults.contains_key(&canonical_id) {
            return self
                .vaults
                .get_mut(&canonical_id)
                .map(|r| &mut r.value)
                .unwrap();
        }

        let (vault, phys_id) = self
            .substate_store
            .get_decoded_child_substate(component_id, vid)
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
    pub fn put_vault(&mut self, component_id: ComponentId, vault_id: VaultId, vault: Vault) {
        let canonical_id = (component_id, vault_id);
        self.vaults.insert(
            canonical_id,
            SubstateUpdate {
                prev_id: None,
                value: vault,
            },
        );
    }

    /// Creates a new package ID.
    fn new_package_id(&mut self) -> PackageId {
        // Security Alert: ensure ID allocating will practically never fail
        let package_id = self
            .id_allocator
            .new_package_id(self.transaction_hash())
            .unwrap();
        package_id
    }

    /// Creates a new component ID.
    fn new_component_id(&mut self) -> ComponentId {
        let component_id = self
            .id_allocator
            .new_component_id(self.transaction_hash())
            .unwrap();
        component_id
    }

    /// Creates a new resource definition ID.
    fn new_resource_def_id(&mut self) -> ResourceDefId {
        let resource_def_id = self
            .id_allocator
            .new_resource_def_id(self.transaction_hash())
            .unwrap();
        resource_def_id
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
        let mut receipt = CommitReceipt::new();

        let package_ids: Vec<PackageId> = self
            .packages
            .iter()
            .map(|(address, _)| address.clone())
            .collect();
        for package_id in package_ids {
            let package = self.packages.remove(&package_id).unwrap();

            if let Some(prev_id) = package.prev_id {
                receipt.down(prev_id);
            }
            let phys_id = self.substate_store.get_nonce();
            receipt.up(phys_id);

            self.substate_store
                .put_encoded_substate(&package_id, &package.value, phys_id);
        }

        let component_ids: Vec<ComponentId> = self
            .components
            .iter()
            .map(|(address, _)| address.clone())
            .collect();
        for component_id in component_ids {
            let component = self.components.remove(&component_id).unwrap();

            if let Some(prev_id) = component.prev_id {
                receipt.down(prev_id);
            }
            let phys_id = self.substate_store.get_nonce();
            receipt.up(phys_id);

            self.substate_store
                .put_encoded_substate(&component_id, &component.value, phys_id);
        }

        let resource_def_ids: Vec<ResourceDefId> = self
            .resource_defs
            .iter()
            .map(|(address, _)| address.clone())
            .collect();
        for resource_def_id in resource_def_ids {
            let resource_def = self.resource_defs.remove(&resource_def_id).unwrap();

            if let Some(prev_id) = resource_def.prev_id {
                receipt.down(prev_id);
            }
            let phys_id = self.substate_store.get_nonce();
            receipt.up(phys_id);

            self.substate_store.put_encoded_substate(
                &resource_def_id,
                &resource_def.value,
                phys_id,
            );
        }

        let entry_ids: Vec<(ComponentId, LazyMapId, Vec<u8>)> = self
            .lazy_map_entries
            .iter()
            .map(|(id, _)| id.clone())
            .collect();
        for entry_id in entry_ids {
            let entry = self.lazy_map_entries.remove(&entry_id).unwrap();
            if let Some(prev_id) = entry.prev_id {
                receipt.down(prev_id);
            }
            let phys_id = self.substate_store.get_nonce();
            receipt.up(phys_id);

            let (component_id, lazy_map_id, key) = entry_id;
            self.substate_store.put_encoded_grand_child_substate(
                &component_id,
                &lazy_map_id,
                &key,
                &entry.value,
                phys_id,
            );
        }

        let vault_ids: Vec<(ComponentId, VaultId)> =
            self.vaults.iter().map(|(id, _)| id.clone()).collect();
        for vault_id in vault_ids {
            let vault = self.vaults.remove(&vault_id).unwrap();
            if let Some(prev_id) = vault.prev_id {
                receipt.down(prev_id);
            }
            let phys_id = self.substate_store.get_nonce();
            receipt.up(phys_id);

            let (component_id, vault_id) = vault_id;
            self.substate_store.put_encoded_child_substate(
                &component_id,
                &vault_id,
                &vault.value,
                phys_id,
            );
        }

        let non_fungible_ids: Vec<NonFungibleAddress> = self
            .non_fungibles
            .iter()
            .map(|(id, _)| id.clone())
            .collect();
        for non_fungible_id in non_fungible_ids {
            let non_fungible = self.non_fungibles.remove(&non_fungible_id).unwrap();
            if let Some(prev_id) = non_fungible.prev_id {
                receipt.down(prev_id);
            }
            let phys_id = self.substate_store.get_nonce();
            receipt.up(phys_id);

            self.substate_store.put_encoded_child_substate(
                &non_fungible_id.resource_def_id(),
                &non_fungible_id.key(),
                &non_fungible.value,
                phys_id,
            );
        }

        receipt
    }
}
