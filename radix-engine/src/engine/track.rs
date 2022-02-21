use lru::LruCache;
use sbor::Decode;
use sbor::Encode;
use scrypto::buffer::scrypto_decode;
use scrypto::buffer::scrypto_encode;
use scrypto::constants::*;
use scrypto::engine::types::*;
use scrypto::rust::collections::*;
use scrypto::rust::string::String;
use scrypto::rust::vec::Vec;
use wasmi::*;

use crate::engine::*;
use crate::ledger::*;
use crate::model::*;

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

    packages: HashMap<PackageId, Package>,
    components: HashMap<ComponentId, Component>,
    resource_defs: HashMap<ResourceDefId, ResourceDef>,
    lazy_map_entries: HashMap<(ComponentId, LazyMapId, Vec<u8>), Vec<u8>>,
    vaults: HashMap<(ComponentId, VaultId), Vault>,
    non_fungibles: HashMap<(ResourceDefId, NonFungibleKey), NonFungible>,

    new_package_ids: Vec<PackageId>,
    new_component_ids: Vec<ComponentId>,
    new_resource_def_ids: Vec<ResourceDefId>,

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
            new_package_ids: Vec::new(),
            new_component_ids: Vec::new(),
            new_resource_def_ids: Vec::new(),
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
    pub fn new_package_ids(&self) -> &[PackageId] {
        &self.new_package_ids
    }

    /// Returns new components created so far.
    pub fn new_component_ids(&self) -> &[ComponentId] {
        &self.new_component_ids
    }

    /// Returns new resource defs created so far.
    pub fn new_resource_def_ids(&self) -> &[ResourceDefId] {
        &self.new_resource_def_ids
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

    fn put_substate<A: Encode, V: Encode>(&mut self, address: &A, value: &V) {
        self.substate_store
            .put_substate(address, &scrypto_encode(value));
    }

    fn get_substate<A: Encode, T: Decode>(&self, address: &A) -> Option<T> {
        self.substate_store
            .get_substate(address)
            .and_then(|v| scrypto_decode(&v).map(|r| Some(r)).unwrap_or(None))
    }

    fn put_child_substate<A: Encode, K: Encode, V: Encode>(
        &mut self,
        address: &A,
        key: &K,
        value: &V,
    ) {
        let child_key = &scrypto_encode(key);
        self.substate_store
            .put_child_substate(address, child_key, &scrypto_encode(value));
    }

    fn get_child_substate<A: Encode, K: Encode, T: Decode>(
        &self,
        address: &A,
        key: &K,
    ) -> Option<T> {
        let child_key = &scrypto_encode(key);
        self.substate_store
            .get_child_substate(address, child_key)
            .and_then(|v| scrypto_decode(&v).map(|r| Some(r)).unwrap_or(None))
    }

    fn put_grand_child_substate<A: Encode, C: Encode>(
        &mut self,
        address: &A,
        child_key: &C,
        grand_child_key: &[u8],
        value: &[u8],
    ) {
        let mut key = scrypto_encode(child_key);
        key.extend(grand_child_key.to_vec());
        self.substate_store.put_child_substate(address, &key, value);
    }

    fn get_grand_child_substate<A: Encode, C: Encode>(
        &self,
        address: &A,
        child_key: &C,
        grand_child_key: &[u8],
    ) -> Option<Vec<u8>> {
        let mut key = scrypto_encode(child_key);
        key.extend(grand_child_key.to_vec());
        self.substate_store.get_child_substate(address, &key)
    }

    /// Returns an immutable reference to a package, if exists.
    pub fn get_package(&mut self, package_id: PackageId) -> Option<&Package> {
        if self.packages.contains_key(&package_id) {
            return self.packages.get(&package_id);
        }

        if let Some(package) = self.get_substate(&package_id) {
            self.packages.insert(package_id, package);
            self.packages.get(&package_id)
        } else {
            None
        }
    }

    /// Returns a mutable reference to a package, if exists.
    #[allow(dead_code)]
    pub fn get_package_mut(&mut self, package_id: PackageId) -> Option<&mut Package> {
        if self.packages.contains_key(&package_id) {
            return self.packages.get_mut(&package_id);
        }

        if let Some(package) = self.get_substate(&package_id) {
            self.packages.insert(package_id, package);
            self.packages.get_mut(&package_id)
        } else {
            None
        }
    }

    /// Inserts a new package.
    pub fn put_package(&mut self, package_id: PackageId, package: Package) {
        self.packages.insert(package_id, package);
    }

    /// Returns an immutable reference to a component, if exists.
    pub fn get_component(&mut self, component_id: ComponentId) -> Option<&Component> {
        if self.components.contains_key(&component_id) {
            return self.components.get(&component_id);
        }

        if let Some(component) = self.get_substate(&component_id) {
            self.components.insert(component_id, component);
            self.components.get(&component_id)
        } else {
            None
        }
    }

    /// Returns a mutable reference to a component, if exists.
    pub fn get_component_mut(&mut self, component_id: ComponentId) -> Option<&mut Component> {
        if self.components.contains_key(&component_id) {
            return self.components.get_mut(&component_id);
        }

        if let Some(component) = self.get_substate(&component_id) {
            self.components.insert(component_id, component);
            self.components.get_mut(&component_id)
        } else {
            None
        }
    }

    /// Inserts a new component.
    pub fn put_component(&mut self, component_id: ComponentId, component: Component) {
        self.components.insert(component_id, component);
    }

    /// Returns an immutable reference to a non-fungible, if exists.
    pub fn get_non_fungible(
        &mut self,
        resource_def_id: ResourceDefId,
        key: &NonFungibleKey,
    ) -> Option<&NonFungible> {
        if self
            .non_fungibles
            .contains_key(&(resource_def_id, key.clone()))
        {
            return self.non_fungibles.get(&(resource_def_id, key.clone()));
        }

        if let Some(non_fungible) = self.get_child_substate(&resource_def_id, key) {
            self.non_fungibles
                .insert((resource_def_id, key.clone()), non_fungible);
            self.non_fungibles.get(&(resource_def_id, key.clone()))
        } else {
            None
        }
    }

    /// Returns a mutable reference to a non-fungible, if exists.
    pub fn get_non_fungible_mut(
        &mut self,
        resource_def_id: ResourceDefId,
        key: &NonFungibleKey,
    ) -> Option<&mut NonFungible> {
        if self
            .non_fungibles
            .contains_key(&(resource_def_id, key.clone()))
        {
            return self.non_fungibles.get_mut(&(resource_def_id, key.clone()));
        }

        if let Some(non_fungible) = self.get_child_substate(&resource_def_id, key) {
            self.non_fungibles
                .insert((resource_def_id, key.clone()), non_fungible);
            self.non_fungibles.get_mut(&(resource_def_id, key.clone()))
        } else {
            None
        }
    }

    /// Inserts a new non-fungible.
    pub fn put_non_fungible(
        &mut self,
        resource_def_id: ResourceDefId,
        key: &NonFungibleKey,
        non_fungible: NonFungible,
    ) {
        self.non_fungibles
            .insert((resource_def_id, key.clone()), non_fungible);
    }

    /// Returns a mutable reference to a lazy map
    pub fn get_lazy_map_entry(
        &mut self,
        component_id: ComponentId,
        lazy_map_id: &LazyMapId,
        key: &[u8],
    ) -> Option<Vec<u8>> {
        let entry_id = (component_id.clone(), lazy_map_id.clone(), key.to_vec());

        if self.lazy_map_entries.contains_key(&entry_id) {
            return Some(self.lazy_map_entries.get(&entry_id).unwrap().clone());
        }

        let grand_child_key = key.to_vec();
        let value = self.get_grand_child_substate(&component_id, lazy_map_id, &grand_child_key);
        if let Some(ref entry_bytes) = value {
            self.lazy_map_entries.insert(entry_id, entry_bytes.clone());
        }
        value
    }

    /// Inserts a new lazy map.
    pub fn put_lazy_map_entry(
        &mut self,
        component_id: ComponentId,
        lazy_map_id: LazyMapId,
        key: Vec<u8>,
        value: Vec<u8>,
    ) {
        let canonical_id = (component_id, lazy_map_id, key);
        self.lazy_map_entries.insert(canonical_id, value);
    }

    /// Returns an immutable reference to a resource definition, if exists.
    pub fn get_resource_def(&mut self, resource_def_id: ResourceDefId) -> Option<&ResourceDef> {
        if self.resource_defs.contains_key(&resource_def_id) {
            return self.resource_defs.get(&resource_def_id);
        }

        if let Some(resource_def) = self.get_substate(&resource_def_id) {
            self.resource_defs.insert(resource_def_id, resource_def);
            self.resource_defs.get(&resource_def_id)
        } else {
            None
        }
    }

    /// Returns a mutable reference to a resource definition, if exists.
    #[allow(dead_code)]
    pub fn get_resource_def_mut(
        &mut self,
        resource_def_id: ResourceDefId,
    ) -> Option<&mut ResourceDef> {
        if self.resource_defs.contains_key(&resource_def_id) {
            return self.resource_defs.get_mut(&resource_def_id);
        }

        if let Some(resource_def) = self.get_substate(&resource_def_id) {
            self.resource_defs.insert(resource_def_id, resource_def);
            self.resource_defs.get_mut(&resource_def_id)
        } else {
            None
        }
    }

    /// Inserts a new resource definition.
    pub fn put_resource_def(&mut self, resource_def_id: ResourceDefId, resource_def: ResourceDef) {
        self.resource_defs.insert(resource_def_id, resource_def);
    }

    /// Returns a mutable reference to a vault, if exists.
    pub fn get_vault_mut(&mut self, component_id: &ComponentId, vid: &VaultId) -> &mut Vault {
        let canonical_id = (component_id.clone(), vid.clone());

        if self.vaults.contains_key(&canonical_id) {
            return self.vaults.get_mut(&canonical_id).unwrap();
        }

        let vault: Vault = self.get_child_substate(component_id, vid).unwrap();
        self.vaults.insert(canonical_id, vault);
        self.vaults.get_mut(&canonical_id).unwrap()
    }

    /// Inserts a new vault.
    pub fn put_vault(&mut self, component_id: ComponentId, vault_id: VaultId, vault: Vault) {
        let canonical_id = (component_id, vault_id);
        self.vaults.insert(canonical_id, vault);
    }

    /// Creates a new package ID.
    pub fn new_package_id(&mut self) -> PackageId {
        // Security Alert: ensure ID allocating will practically never fail
        let package_id = self
            .id_allocator
            .new_package_id(self.transaction_hash())
            .unwrap();
        self.new_package_ids.push(package_id);
        package_id
    }

    /// Creates a new component ID.
    pub fn new_component_id(&mut self) -> ComponentId {
        let component_id = self
            .id_allocator
            .new_component_id(self.transaction_hash())
            .unwrap();
        self.new_component_ids.push(component_id);
        component_id
    }

    /// Creates a new resource definition ID.
    pub fn new_resource_def_id(&mut self) -> ResourceDefId {
        let resource_def_id = self
            .id_allocator
            .new_resource_def_id(self.transaction_hash())
            .unwrap();
        self.new_resource_def_ids.push(resource_def_id);
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
    pub fn commit(&mut self) {
        let package_ids: Vec<PackageId> = self
            .packages
            .iter()
            .map(|(address, _)| address.clone())
            .collect();
        for package_id in package_ids {
            let package = self.packages.remove(&package_id).unwrap();
            self.put_substate(&package_id, &package);
        }

        let component_ids: Vec<ComponentId> = self
            .components
            .iter()
            .map(|(address, _)| address.clone())
            .collect();
        for component_id in component_ids {
            let component = self.components.remove(&component_id).unwrap();
            self.put_substate(&component_id, &component);
        }

        let resource_def_ids: Vec<ResourceDefId> = self
            .resource_defs
            .iter()
            .map(|(address, _)| address.clone())
            .collect();
        for resource_def_id in resource_def_ids {
            let resource_def = self.resource_defs.remove(&resource_def_id).unwrap();
            self.put_substate(&resource_def_id, &resource_def);
        }

        let entry_ids: Vec<(ComponentId, LazyMapId, Vec<u8>)> = self
            .lazy_map_entries
            .iter()
            .map(|(id, _)| id.clone())
            .collect();
        for entry_id in entry_ids {
            let entry = self.lazy_map_entries.remove(&entry_id).unwrap();
            let (component_id, lazy_map_id, key) = entry_id;
            self.put_grand_child_substate(&component_id, &lazy_map_id, &key, &entry);
        }

        let vault_ids: Vec<(ComponentId, VaultId)> =
            self.vaults.iter().map(|(id, _)| id.clone()).collect();
        for vault_id in vault_ids {
            let vault = self.vaults.remove(&vault_id).unwrap();
            let (component_id, vault_id) = vault_id;
            self.put_child_substate(&component_id, &vault_id, &vault);
        }

        let non_fungible_ids: Vec<(ResourceDefId, NonFungibleKey)> = self
            .non_fungibles
            .iter()
            .map(|(id, _)| id.clone())
            .collect();
        for non_fungible_id in non_fungible_ids {
            let non_fungible = self.non_fungibles.remove(&non_fungible_id).unwrap();
            let (resource_def_id, non_fungible_key) = non_fungible_id;
            self.put_child_substate(&resource_def_id, &non_fungible_key, &non_fungible);
        }
    }
}
