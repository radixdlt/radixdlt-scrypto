use lru::LruCache;
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
    ledger: &'s mut S,
    transaction_hash: Hash,
    transaction_signers: Vec<EcdsaPublicKey>,
    id_allocator: IdAllocator,
    logs: Vec<(Level, String)>,
    packages: HashMap<PackageRef, Package>,
    components: HashMap<ComponentRef, Component>,
    resource_defs: HashMap<ResourceDefRef, ResourceDef>,
    lazy_maps: HashMap<(ComponentRef, LazyMapId), LazyMap>,
    vaults: HashMap<(ComponentRef, VaultId), Vault>,
    non_fungibles: HashMap<(ResourceDefRef, NonFungibleKey), NonFungible>,
    updated_packages: HashSet<PackageRef>,
    updated_components: HashSet<ComponentRef>,
    updated_lazy_maps: HashSet<(ComponentRef, LazyMapId)>,
    updated_resource_defs: HashSet<ResourceDefRef>,
    updated_vaults: HashSet<(ComponentRef, VaultId)>,
    updated_non_fungibles: HashSet<(ResourceDefRef, NonFungibleKey)>,
    new_package_refs: Vec<PackageRef>,
    new_component_refs: Vec<ComponentRef>,
    new_resource_def_refs: Vec<ResourceDefRef>,
    code_cache: LruCache<PackageRef, Module>, // TODO: move to ledger level
}

impl<'s, S: SubstateStore> Track<'s, S> {
    pub fn new(
        ledger: &'s mut S,
        transaction_hash: Hash,
        transaction_signers: Vec<EcdsaPublicKey>,
    ) -> Self {
        Self {
            ledger,
            transaction_hash,
            transaction_signers,
            id_allocator: IdAllocator::new(IdSpace::Application),
            logs: Vec::new(),
            packages: HashMap::new(),
            components: HashMap::new(),
            resource_defs: HashMap::new(),
            lazy_maps: HashMap::new(),
            vaults: HashMap::new(),
            non_fungibles: HashMap::new(),
            updated_packages: HashSet::new(),
            updated_components: HashSet::new(),
            updated_lazy_maps: HashSet::new(),
            updated_resource_defs: HashSet::new(),
            updated_vaults: HashSet::new(),
            updated_non_fungibles: HashSet::new(),
            new_package_refs: Vec::new(),
            new_component_refs: Vec::new(),
            new_resource_def_refs: Vec::new(),
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
        self.ledger.get_epoch()
    }

    /// Returns the logs collected so far.
    pub fn logs(&self) -> &Vec<(Level, String)> {
        &self.logs
    }

    /// Returns new packages created so far.
    pub fn new_package_refs(&self) -> &[PackageRef] {
        &self.new_package_refs
    }

    /// Returns new components created so far.
    pub fn new_component_refs(&self) -> &[ComponentRef] {
        &self.new_component_refs
    }

    /// Returns new resource defs created so far.
    pub fn new_resource_def_refs(&self) -> &[ResourceDefRef] {
        &self.new_resource_def_refs
    }

    /// Adds a log message.
    pub fn add_log(&mut self, level: Level, message: String) {
        self.logs.push((level, message));
    }

    /// Loads a module.
    pub fn load_module(&mut self, package_ref: PackageRef) -> Option<(ModuleRef, MemoryRef)> {
        match self.get_package(package_ref).map(Clone::clone) {
            Some(p) => {
                if let Some(m) = self.code_cache.get(&package_ref) {
                    Some(instantiate_module(m).unwrap())
                } else {
                    let module = parse_module(p.code()).unwrap();
                    let inst = instantiate_module(&module).unwrap();
                    self.code_cache.put(package_ref, module);
                    Some(inst)
                }
            }
            None => None,
        }
    }

    /// Returns an immutable reference to a package, if exists.
    pub fn get_package(&mut self, package_ref: PackageRef) -> Option<&Package> {
        if self.packages.contains_key(&package_ref) {
            return self.packages.get(&package_ref);
        }

        if let Some(package) = self.ledger.get_package(package_ref) {
            self.packages.insert(package_ref, package);
            self.packages.get(&package_ref)
        } else {
            None
        }
    }

    /// Returns a mutable reference to a package, if exists.
    #[allow(dead_code)]
    pub fn get_package_mut(&mut self, package_ref: PackageRef) -> Option<&mut Package> {
        self.updated_packages.insert(package_ref);

        if self.packages.contains_key(&package_ref) {
            return self.packages.get_mut(&package_ref);
        }

        if let Some(package) = self.ledger.get_package(package_ref) {
            self.packages.insert(package_ref, package);
            self.packages.get_mut(&package_ref)
        } else {
            None
        }
    }

    /// Inserts a new package.
    pub fn put_package(&mut self, package_ref: PackageRef, package: Package) {
        self.updated_packages.insert(package_ref);

        self.packages.insert(package_ref, package);
    }

    /// Returns an immutable reference to a component, if exists.
    pub fn get_component(&mut self, component_ref: ComponentRef) -> Option<&Component> {
        if self.components.contains_key(&component_ref) {
            return self.components.get(&component_ref);
        }

        if let Some(component) = self.ledger.get_component(component_ref) {
            self.components.insert(component_ref, component);
            self.components.get(&component_ref)
        } else {
            None
        }
    }
    /// Returns a mutable reference to a component, if exists.
    pub fn get_component_mut(&mut self, component_ref: ComponentRef) -> Option<&mut Component> {
        self.updated_components.insert(component_ref);

        if self.components.contains_key(&component_ref) {
            return self.components.get_mut(&component_ref);
        }

        if let Some(component) = self.ledger.get_component(component_ref) {
            self.components.insert(component_ref, component);
            self.components.get_mut(&component_ref)
        } else {
            None
        }
    }

    /// Inserts a new component.
    pub fn put_component(&mut self, component_ref: ComponentRef, component: Component) {
        self.updated_components.insert(component_ref);

        self.components.insert(component_ref, component);
    }

    /// Returns an immutable reference to a non-fungible, if exists.
    pub fn get_non_fungible(
        &mut self,
        resource_def_ref: ResourceDefRef,
        key: &NonFungibleKey,
    ) -> Option<&NonFungible> {
        if self
            .non_fungibles
            .contains_key(&(resource_def_ref, key.clone()))
        {
            return self.non_fungibles.get(&(resource_def_ref, key.clone()));
        }

        if let Some(non_fungible) = self.ledger.get_non_fungible(resource_def_ref, key) {
            self.non_fungibles
                .insert((resource_def_ref, key.clone()), non_fungible);
            self.non_fungibles.get(&(resource_def_ref, key.clone()))
        } else {
            None
        }
    }

    /// Returns a mutable reference to a non-fungible, if exists.
    pub fn get_non_fungible_mut(
        &mut self,
        resource_def_ref: ResourceDefRef,
        key: &NonFungibleKey,
    ) -> Option<&mut NonFungible> {
        self.updated_non_fungibles
            .insert((resource_def_ref, key.clone()));

        if self
            .non_fungibles
            .contains_key(&(resource_def_ref, key.clone()))
        {
            return self.non_fungibles.get_mut(&(resource_def_ref, key.clone()));
        }

        if let Some(non_fungible) = self.ledger.get_non_fungible(resource_def_ref, key) {
            self.non_fungibles
                .insert((resource_def_ref, key.clone()), non_fungible);
            self.non_fungibles.get_mut(&(resource_def_ref, key.clone()))
        } else {
            None
        }
    }

    /// Inserts a new non-fungible.
    pub fn put_non_fungible(
        &mut self,
        resource_def_ref: ResourceDefRef,
        key: &NonFungibleKey,
        non_fungible: NonFungible,
    ) {
        self.updated_non_fungibles
            .insert((resource_def_ref, key.clone()));

        self.non_fungibles
            .insert((resource_def_ref, key.clone()), non_fungible);
    }

    /// Returns an immutable reference to a lazy map, if exists.
    pub fn get_lazy_map(
        &mut self,
        component_ref: ComponentRef,
        lazy_map_id: LazyMapId,
    ) -> Option<&LazyMap> {
        let canonical_id = (component_ref, lazy_map_id);

        if self.lazy_maps.contains_key(&canonical_id) {
            return self.lazy_maps.get(&canonical_id);
        }

        if let Some(lazy_map) = self.ledger.get_lazy_map(component_ref, lazy_map_id) {
            self.lazy_maps.insert(canonical_id, lazy_map);
            self.lazy_maps.get(&canonical_id)
        } else {
            None
        }
    }

    /// Returns a mutable reference to a lazy map, if exists.
    pub fn get_lazy_map_mut(
        &mut self,
        component_ref: ComponentRef,
        lazy_map_id: LazyMapId,
    ) -> Option<&mut LazyMap> {
        let canonical_id = (component_ref, lazy_map_id);
        self.updated_lazy_maps.insert(canonical_id.clone());

        if self.lazy_maps.contains_key(&canonical_id) {
            return self.lazy_maps.get_mut(&canonical_id);
        }

        if let Some(lazy_map) = self.ledger.get_lazy_map(component_ref, lazy_map_id) {
            self.lazy_maps.insert(canonical_id, lazy_map);
            self.lazy_maps.get_mut(&canonical_id)
        } else {
            None
        }
    }

    /// Inserts a new lazy map.
    pub fn put_lazy_map(
        &mut self,
        component_ref: ComponentRef,
        lazy_map_id: LazyMapId,
        lazy_map: LazyMap,
    ) {
        let canonical_id = (component_ref, lazy_map_id);
        self.updated_lazy_maps.insert(canonical_id.clone());
        self.lazy_maps.insert(canonical_id, lazy_map);
    }

    /// Returns an immutable reference to a resource definition, if exists.
    pub fn get_resource_def(&mut self, resource_def_ref: ResourceDefRef) -> Option<&ResourceDef> {
        if self.resource_defs.contains_key(&resource_def_ref) {
            return self.resource_defs.get(&resource_def_ref);
        }

        if let Some(resource_def) = self.ledger.get_resource_def(resource_def_ref) {
            self.resource_defs.insert(resource_def_ref, resource_def);
            self.resource_defs.get(&resource_def_ref)
        } else {
            None
        }
    }

    /// Returns a mutable reference to a resource definition, if exists.
    #[allow(dead_code)]
    pub fn get_resource_def_mut(
        &mut self,
        resource_def_ref: ResourceDefRef,
    ) -> Option<&mut ResourceDef> {
        self.updated_resource_defs.insert(resource_def_ref);

        if self.resource_defs.contains_key(&resource_def_ref) {
            return self.resource_defs.get_mut(&resource_def_ref);
        }

        if let Some(resource_def) = self.ledger.get_resource_def(resource_def_ref) {
            self.resource_defs.insert(resource_def_ref, resource_def);
            self.resource_defs.get_mut(&resource_def_ref)
        } else {
            None
        }
    }

    /// Inserts a new resource definition.
    pub fn put_resource_def(
        &mut self,
        resource_def_ref: ResourceDefRef,
        resource_def: ResourceDef,
    ) {
        self.updated_resource_defs.insert(resource_def_ref);

        self.resource_defs.insert(resource_def_ref, resource_def);
    }

    /// Returns a mutable reference to a vault, if exists.
    pub fn get_vault_mut(
        &mut self,
        component_ref: ComponentRef,
        vault_id: VaultId,
    ) -> Option<&mut Vault> {
        let canonical_id = (component_ref.clone(), vault_id.clone());
        self.updated_vaults.insert(canonical_id.clone());

        if self.vaults.contains_key(&canonical_id) {
            return self.vaults.get_mut(&canonical_id);
        }

        if let Some(vault) = self.ledger.get_vault(component_ref, vault_id) {
            self.vaults.insert(canonical_id, vault);
            self.vaults.get_mut(&canonical_id)
        } else {
            None
        }
    }

    /// Inserts a new vault.
    pub fn put_vault(&mut self, component_ref: ComponentRef, vault_id: VaultId, vault: Vault) {
        let canonical_id = (component_ref, vault_id);
        self.updated_vaults.insert(canonical_id);
        self.vaults.insert(canonical_id, vault);
    }

    /// Creates a new package ref.
    pub fn new_package_ref(&mut self) -> PackageRef {
        // Security Alert: ensure ID allocating will practically never fail
        let package_ref = self
            .id_allocator
            .new_package_ref(self.transaction_hash())
            .unwrap();
        self.new_package_refs.push(package_ref);
        package_ref
    }

    /// Creates a new component ref.
    pub fn new_component_ref(&mut self) -> ComponentRef {
        let component_ref = self
            .id_allocator
            .new_component_ref(self.transaction_hash())
            .unwrap();
        self.new_component_refs.push(component_ref);
        component_ref
    }

    /// Creates a new resource definition ref.
    pub fn new_resource_def_ref(&mut self) -> ResourceDefRef {
        let resource_def_ref = self
            .id_allocator
            .new_resource_def_ref(self.transaction_hash())
            .unwrap();
        self.new_resource_def_refs.push(resource_def_ref);
        resource_def_ref
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
    pub fn commit(&mut self) {
        for package_ref in self.updated_packages.clone() {
            self.ledger.put_package(
                package_ref,
                self.packages.get(&package_ref).unwrap().clone(),
            );
        }

        for component_ref in self.updated_components.clone() {
            self.ledger.put_component(
                component_ref,
                self.components.get(&component_ref).unwrap().clone(),
            );
        }

        for resource_def_ref in self.updated_resource_defs.clone() {
            self.ledger.put_resource_def(
                resource_def_ref,
                self.resource_defs.get(&resource_def_ref).unwrap().clone(),
            );
        }

        for (component_ref, lazy_map_id) in self.updated_lazy_maps.clone() {
            let lazy_map = self
                .lazy_maps
                .get(&(component_ref, lazy_map_id))
                .unwrap()
                .clone();
            self.ledger
                .put_lazy_map(component_ref, lazy_map_id, lazy_map);
        }

        for (component_ref, vault_id) in self.updated_vaults.clone() {
            let vault = self.vaults.get(&(component_ref, vault_id)).unwrap().clone();
            self.ledger.put_vault(component_ref, vault_id, vault);
        }

        for (resource_def, key) in self.updated_non_fungibles.clone() {
            self.ledger.put_non_fungible(
                resource_def,
                &key,
                self.non_fungibles
                    .get(&(resource_def, key.clone()))
                    .unwrap()
                    .clone(),
            );
        }
    }
}
