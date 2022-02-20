use lru::LruCache;
use sbor::Decode;
use scrypto::buffer::scrypto_decode;
use scrypto::engine::*;
use scrypto::prelude::scrypto_encode;
use scrypto::rust::collections::*;
use scrypto::rust::string::String;
use scrypto::rust::vec::Vec;
use scrypto::types::*;
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
    transaction_hash: H256,
    transaction_signers: Vec<EcdsaPublicKey>,
    id_allocator: IdAllocator,
    logs: Vec<(LogLevel, String)>,

    packages: HashMap<Address, Package>,
    components: HashMap<Address, Component>,
    resource_defs: HashMap<Address, ResourceDef>,
    lazy_map_entries: HashMap<(Address, Mid, Vec<u8>), Vec<u8>>,
    vaults: HashMap<(Address, Vid), Vault>,
    non_fungibles: HashMap<(Address, NonFungibleKey), NonFungible>,

    new_entities: Vec<Address>,
    code_cache: LruCache<Address, Module>, // TODO: move to ledger level
}

impl<'s, S: SubstateStore> Track<'s, S> {
    pub fn new(
        ledger: &'s mut S,
        transaction_hash: H256,
        transaction_signers: Vec<EcdsaPublicKey>,
    ) -> Self {
        Self {
            substate_store: ledger,
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
            new_entities: Vec::new(),
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
            .map(|key| NonFungibleKey::new(key.to_vec()))
            .collect();
        let mut process = Process::new(0, verbose, self);

        // Always create a virtual bucket of signatures even if there is none.
        // This is to make reasoning at transaction manifest & validator easier.
        let ecdsa_bucket = Bucket::new(
            ECDSA_TOKEN,
            ResourceType::NonFungible,
            Supply::NonFungible { keys: signers },
        );
        process.create_virtual_bucket_ref(ECDSA_TOKEN_BID, ECDSA_TOKEN_RID, ecdsa_bucket);

        process
    }

    /// Returns the transaction hash.
    pub fn transaction_hash(&self) -> H256 {
        self.transaction_hash
    }

    /// Returns the current epoch.
    pub fn current_epoch(&self) -> u64 {
        self.substate_store.get_epoch()
    }

    /// Returns the logs collected so far.
    pub fn logs(&self) -> &Vec<(LogLevel, String)> {
        &self.logs
    }

    /// Returns new entities created so far.
    pub fn new_entities(&self) -> &[Address] {
        &self.new_entities
    }

    /// Adds a log message.
    pub fn add_log(&mut self, level: LogLevel, message: String) {
        self.logs.push((level, message));
    }

    /// Loads a module.
    pub fn load_module(&mut self, address: Address) -> Option<(ModuleRef, MemoryRef)> {
        match self.get_package(address).map(Clone::clone) {
            Some(p) => {
                if let Some(m) = self.code_cache.get(&address) {
                    Some(instantiate_module(m).unwrap())
                } else {
                    let module = parse_module(p.code()).unwrap();
                    let inst = instantiate_module(&module).unwrap();
                    self.code_cache.put(address, module);
                    Some(inst)
                }
            }
            None => None,
        }
    }

    /// Returns an immutable reference to a package, if exists.
    pub fn get_package(&mut self, address: Address) -> Option<&Package> {
        if self.packages.contains_key(&address) {
            return self.packages.get(&address);
        }

        if let Some(package) = self.get_substate(&address) {
            self.packages.insert(address, package);
            self.packages.get(&address)
        } else {
            None
        }
    }

    /// Returns a mutable reference to a package, if exists.
    #[allow(dead_code)]
    pub fn get_package_mut(&mut self, address: Address) -> Option<&mut Package> {
        if self.packages.contains_key(&address) {
            return self.packages.get_mut(&address);
        }

        if let Some(package) = self.get_substate(&address) {
            self.packages.insert(address, package);
            self.packages.get_mut(&address)
        } else {
            None
        }
    }

    /// Inserts a new package.
    pub fn put_package(&mut self, address: Address, package: Package) {
        self.packages.insert(address, package);
    }

    /// Returns an immutable reference to a component, if exists.
    pub fn get_component(&mut self, address: Address) -> Option<&Component> {
        if self.components.contains_key(&address) {
            return self.components.get(&address);
        }

        if let Some(component) = self.get_substate(&address) {
            self.components.insert(address, component);
            self.components.get(&address)
        } else {
            None
        }
    }

    /// Returns a mutable reference to a component, if exists.
    pub fn get_component_mut(&mut self, address: Address) -> Option<&mut Component> {
        if self.components.contains_key(&address) {
            return self.components.get_mut(&address);
        }

        if let Some(component) = self.get_substate(&address) {
            self.components.insert(address, component);
            self.components.get_mut(&address)
        } else {
            None
        }
    }

    /// Inserts a new component.
    pub fn put_component(&mut self, address: Address, component: Component) {
        self.components.insert(address, component);
    }

    /// Returns an immutable reference to a non-fungible, if exists.
    pub fn get_non_fungible(
        &mut self,
        resource_address: Address,
        key: &NonFungibleKey,
    ) -> Option<&NonFungible> {
        if self
            .non_fungibles
            .contains_key(&(resource_address, key.clone()))
        {
            return self.non_fungibles.get(&(resource_address, key.clone()));
        }

        if let Some(non_fungible) = self
            .substate_store
            .get_non_fungible(&resource_address, &key)
        {
            self.non_fungibles
                .insert((resource_address, key.clone()), non_fungible);
            self.non_fungibles.get(&(resource_address, key.clone()))
        } else {
            None
        }
    }

    /// Returns a mutable reference to a non-fungible, if exists.
    pub fn get_non_fungible_mut(
        &mut self,
        resource_address: Address,
        key: &NonFungibleKey,
    ) -> Option<&mut NonFungible> {
        if self
            .non_fungibles
            .contains_key(&(resource_address, key.clone()))
        {
            return self.non_fungibles.get_mut(&(resource_address, key.clone()));
        }

        if let Some(non_fungible) = self
            .substate_store
            .get_non_fungible(&resource_address, &key)
        {
            self.non_fungibles
                .insert((resource_address, key.clone()), non_fungible);
            self.non_fungibles.get_mut(&(resource_address, key.clone()))
        } else {
            None
        }
    }

    /// Inserts a new non-fungible.
    pub fn put_non_fungible(
        &mut self,
        resource_address: Address,
        key: &NonFungibleKey,
        non_fungible: NonFungible,
    ) {
        self.non_fungibles
            .insert((resource_address, key.clone()), non_fungible);
    }

    /// Returns a mutable reference to a lazy map
    pub fn get_lazy_map_entry(
        &mut self,
        component_address: &Address,
        mid: &Mid,
        key: &[u8],
    ) -> Option<Vec<u8>> {
        let entry_id = (component_address.clone(), mid.clone(), key.to_vec());

        if self.lazy_map_entries.contains_key(&entry_id) {
            return Some(self.lazy_map_entries.get(&entry_id).unwrap().clone());
        }

        let mut child_key = scrypto_encode(mid);
        child_key.extend(key.to_vec());

        let value = self.substate_store.get_child_substate(component_address, &child_key);
        if let Some(ref entry_bytes) = value {
            self.lazy_map_entries.insert(entry_id, entry_bytes.clone());
        }
        value
    }

    /// Inserts a new lazy map.
    pub fn put_lazy_map_entry(
        &mut self,
        component_address: Address,
        mid: Mid,
        key: Vec<u8>,
        value: Vec<u8>,
    ) {
        let lazy_map_id = (component_address, mid, key);
        self.lazy_map_entries.insert(lazy_map_id, value);
    }

    fn get_substate<T: Decode>(&self, address: &Address) -> Option<T> {
        self.substate_store
            .get_substate(address)
            .and_then(|v| scrypto_decode(&v).map(|r| Some(r)).unwrap_or(None))
    }

    fn get_child_substate<T: Decode>(&self, address: &Address, key: &[u8]) -> Option<T> {
        self.substate_store
            .get_child_substate(address, key)
            .and_then(|v| scrypto_decode(&v).map(|r| Some(r)).unwrap_or(None))
    }

    /// Returns an immutable reference to a resource definition, if exists.
    pub fn get_resource_def(&mut self, address: Address) -> Option<&ResourceDef> {
        if self.resource_defs.contains_key(&address) {
            return self.resource_defs.get(&address);
        }

        if let Some(resource_def) = self.get_substate(&address) {
            self.resource_defs.insert(address, resource_def);
            self.resource_defs.get(&address)
        } else {
            None
        }
    }

    /// Returns a mutable reference to a resource definition, if exists.
    #[allow(dead_code)]
    pub fn get_resource_def_mut(&mut self, address: Address) -> Option<&mut ResourceDef> {
        if self.resource_defs.contains_key(&address) {
            return self.resource_defs.get_mut(&address);
        }

        if let Some(resource_def) = self.get_substate(&address) {
            self.resource_defs.insert(address, resource_def);
            self.resource_defs.get_mut(&address)
        } else {
            None
        }
    }

    /// Inserts a new resource definition.
    pub fn put_resource_def(&mut self, address: Address, resource_def: ResourceDef) {
        self.resource_defs.insert(address, resource_def);
    }

    /// Returns a mutable reference to a vault, if exists.
    pub fn get_vault_mut(&mut self, component_address: &Address, vid: &Vid) -> &mut Vault {
        let vault_id = (component_address.clone(), vid.clone());

        if self.vaults.contains_key(&vault_id) {
            return self.vaults.get_mut(&vault_id).unwrap();
        }

        let vault_key = scrypto_encode(vid);
        let vault: Vault = self.get_child_substate(component_address, &vault_key).unwrap();
        self.vaults.insert(vault_id, vault);
        self.vaults.get_mut(&vault_id).unwrap()
    }

    /// Inserts a new vault.
    pub fn put_vault(&mut self, component_address: Address, vid: Vid, vault: Vault) {
        let vault_id = (component_address, vid);
        self.vaults.insert(vault_id, vault);
    }

    /// Creates a new package address.
    pub fn new_package_address(&mut self) -> Address {
        // Security Alert: ensure ID allocating will practically never fail
        let address = self
            .id_allocator
            .new_package_address(self.transaction_hash())
            .unwrap();
        self.new_entities.push(address);
        address
    }

    /// Creates a new component address.
    pub fn new_component_address(&mut self) -> Address {
        let address = self
            .id_allocator
            .new_component_address(self.transaction_hash())
            .unwrap();
        self.new_entities.push(address);
        address
    }

    /// Creates a new resource definition address.
    pub fn new_resource_address(&mut self) -> Address {
        let address = self
            .id_allocator
            .new_resource_address(self.transaction_hash())
            .unwrap();
        self.new_entities.push(address);
        address
    }

    /// Creates a new UUID.
    pub fn new_uuid(&mut self) -> u128 {
        self.id_allocator.new_uuid(self.transaction_hash()).unwrap()
    }

    /// Creates a new bucket ID.
    pub fn new_bid(&mut self) -> Bid {
        self.id_allocator.new_bid().unwrap()
    }

    /// Creates a new vault ID.
    pub fn new_vid(&mut self) -> Vid {
        self.id_allocator.new_vid(self.transaction_hash()).unwrap()
    }

    /// Creates a new reference id.
    pub fn new_rid(&mut self) -> Rid {
        self.id_allocator.new_rid().unwrap()
    }

    /// Creates a new map id.
    pub fn new_mid(&mut self) -> Mid {
        self.id_allocator.new_mid(self.transaction_hash()).unwrap()
    }

    /// Commits changes to the underlying ledger.
    /// Currently none of these objects are deleted so all commits are puts
    pub fn commit(&mut self) {
        let package_addresses: Vec<Address> = self
            .packages
            .iter()
            .map(|(address, _)| address.clone())
            .collect();
        for package_address in package_addresses {
            let package = self.packages.remove(&package_address).unwrap();
            let value = &scrypto_encode(&package);
            self.substate_store.put_substate(&package_address, value);
        }

        let component_addresses: Vec<Address> = self
            .components
            .iter()
            .map(|(address, _)| address.clone())
            .collect();
        for component_address in component_addresses {
            let component = self.components.remove(&component_address).unwrap();
            let value = &scrypto_encode(&component);
            self.substate_store.put_substate(&component_address, value);
        }

        let resource_def_addresses: Vec<Address> = self
            .resource_defs
            .iter()
            .map(|(address, _)| address.clone())
            .collect();
        for resource_def_address in resource_def_addresses {
            let resource_def = self.resource_defs.remove(&resource_def_address).unwrap();
            let value = &scrypto_encode(&resource_def);
            self.substate_store
                .put_substate(&resource_def_address, value);
        }

        let entry_ids: Vec<(Address, Mid, Vec<u8>)> = self
            .lazy_map_entries
            .iter()
            .map(|(id, _)| id.clone())
            .collect();
        for entry_id in entry_ids {
            let entry = self.lazy_map_entries.remove(&entry_id).unwrap();
            let (component_address, mid, key) = entry_id;
            let mut child_key = scrypto_encode(&mid);
            child_key.extend(key);
            self.substate_store.put_child_substate(&component_address, &child_key, &entry);
        }

        let vault_ids: Vec<(Address, Vid)> = self.vaults.iter().map(|(id, _)| id.clone()).collect();
        for vault_id in vault_ids {
            let vault = self.vaults.remove(&vault_id).unwrap();
            let (component_address, vid) = vault_id;
            let child_key = scrypto_encode(&vid);
            self.substate_store.put_child_substate(&component_address, &child_key, &scrypto_encode(&vault));
        }

        let non_fungible_ids: Vec<(Address, NonFungibleKey)> = self
            .non_fungibles
            .iter()
            .map(|(id, _)| id.clone())
            .collect();
        for non_fungible_id in non_fungible_ids {
            let non_fungible = self.non_fungibles.remove(&non_fungible_id).unwrap();
            let (resource_address, non_fungible_key) = non_fungible_id;
            self.substate_store.put_non_fungible(
                &resource_address,
                &non_fungible_key,
                non_fungible,
            );
        }
    }
}
