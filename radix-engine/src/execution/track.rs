use lru::LruCache;
use scrypto::rust::collections::*;
use scrypto::rust::string::String;
use scrypto::rust::vec::Vec;
use scrypto::types::*;
use wasmi::*;

use crate::execution::*;
use crate::ledger::*;
use crate::model::*;

/// An abstraction of transaction execution state.
///
/// It acts as the facade of ledger state and keeps track of all temporary state updates,
/// until the `commit()` method is called.
///
/// Typically, a track involves a series of processes.
///
pub struct Track<'le, L: Ledger> {
    tx_hash: H256,
    ledger: &'le mut L,
    alloc: AddressAllocator,
    logs: Vec<(Level, String)>,
    packages: HashMap<Address, Package>,
    components: HashMap<Address, Component>,
    lazy_maps: HashMap<MID, LazyMap>,
    resources: HashMap<Address, ResourceDef>,
    vaults: HashMap<VID, Vault>,
    updated_packages: HashSet<Address>,
    updated_components: HashSet<Address>,
    updated_lazy_maps: HashSet<MID>,
    updated_resources: HashSet<Address>,
    updated_vaults: HashSet<VID>,
    new_addresses: Vec<Address>,
    cache: LruCache<Address, Module>, // TODO: move to ledger level
}

impl<'le, L: Ledger> Track<'le, L> {
    pub fn new(tx_hash: H256, ledger: &'le mut L) -> Self {
        Self {
            tx_hash,
            ledger,
            alloc: AddressAllocator::new(),
            logs: Vec::new(),
            packages: HashMap::new(),
            components: HashMap::new(),
            lazy_maps: HashMap::new(),
            resources: HashMap::new(),
            vaults: HashMap::new(),
            updated_packages: HashSet::new(),
            updated_components: HashSet::new(),
            updated_lazy_maps: HashSet::new(),
            updated_resources: HashSet::new(),
            updated_vaults: HashSet::new(),
            new_addresses: Vec::new(),
            cache: LruCache::new(1024),
        }
    }

    /// Start a process.
    pub fn start_process<'rt>(&'rt mut self, verbose: bool) -> Process<'rt, 'le, L> {
        Process::new(0, verbose, self)
    }

    /// Returns the transaction hash.
    pub fn tx_hash(&self) -> H256 {
        self.tx_hash
    }

    /// Returns the logs collected so far.
    pub fn logs(&self) -> &Vec<(Level, String)> {
        &self.logs
    }

    /// Returns new addresses created so far.
    pub fn new_addresses(&self) -> &[Address] {
        &self.new_addresses
    }

    /// Adds a log message.
    pub fn add_log(&mut self, level: Level, message: String) {
        self.logs.push((level, message));
    }

    /// Loads a module.
    pub fn load_module(&mut self, address: Address) -> Option<(ModuleRef, MemoryRef)> {
        match self.get_package(address).map(Clone::clone) {
            Some(p) => {
                if let Some(m) = self.cache.get(&address) {
                    Some(instantiate_module(m).unwrap())
                } else {
                    let module = parse_module(p.code()).unwrap();
                    let inst = instantiate_module(&module).unwrap();
                    self.cache.put(address, module);
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

        if let Some(package) = self.ledger.get_package(address) {
            self.packages.insert(address, package);
            self.packages.get(&address)
        } else {
            None
        }
    }

    /// Returns a mutable reference to a package, if exists.
    #[allow(dead_code)]
    pub fn get_package_mut(&mut self, address: Address) -> Option<&mut Package> {
        self.updated_packages.insert(address);

        if self.packages.contains_key(&address) {
            return self.packages.get_mut(&address);
        }

        if let Some(package) = self.ledger.get_package(address) {
            self.packages.insert(address, package);
            self.packages.get_mut(&address)
        } else {
            None
        }
    }

    /// Inserts a new package.
    pub fn put_package(&mut self, address: Address, package: Package) {
        self.updated_packages.insert(address);

        self.packages.insert(address, package);
    }

    /// Returns an immutable reference to a component, if exists.
    pub fn get_component(&mut self, address: Address) -> Option<&Component> {
        if self.components.contains_key(&address) {
            return self.components.get(&address);
        }

        if let Some(component) = self.ledger.get_component(address) {
            self.components.insert(address, component);
            self.components.get(&address)
        } else {
            None
        }
    }
    /// Returns a mutable reference to a component, if exists.
    pub fn get_component_mut(&mut self, address: Address) -> Option<&mut Component> {
        self.updated_components.insert(address);

        if self.components.contains_key(&address) {
            return self.components.get_mut(&address);
        }

        if let Some(component) = self.ledger.get_component(address) {
            self.components.insert(address, component);
            self.components.get_mut(&address)
        } else {
            None
        }
    }

    /// Inserts a new component.
    pub fn put_component(&mut self, address: Address, component: Component) {
        self.updated_components.insert(address);

        self.components.insert(address, component);
    }

    /// Returns an immutable reference to a map, if exists.
    pub fn get_lazy_map(&mut self, mid: MID) -> Option<&LazyMap> {
        if self.lazy_maps.contains_key(&mid) {
            return self.lazy_maps.get(&mid);
        }

        if let Some(lazy_map) = self.ledger.get_lazy_map(mid) {
            self.lazy_maps.insert(mid, lazy_map);
            self.lazy_maps.get(&mid)
        } else {
            None
        }
    }
    /// Returns a mutable reference to a map, if exists.
    pub fn get_lazy_map_mut(&mut self, mid: MID) -> Option<&mut LazyMap> {
        self.updated_lazy_maps.insert(mid);

        if self.lazy_maps.contains_key(&mid) {
            return self.lazy_maps.get_mut(&mid);
        }

        if let Some(lazy_map) = self.ledger.get_lazy_map(mid) {
            self.lazy_maps.insert(mid, lazy_map);
            self.lazy_maps.get_mut(&mid)
        } else {
            None
        }
    }

    /// Inserts a new map.
    pub fn put_lazy_map(&mut self, mid: MID, lazy_map: LazyMap) {
        self.updated_lazy_maps.insert(mid);

        self.lazy_maps.insert(mid, lazy_map);
    }

    /// Returns an immutable reference to a resource, if exists.
    pub fn get_resource_def(&mut self, address: Address) -> Option<&ResourceDef> {
        if self.resources.contains_key(&address) {
            return self.resources.get(&address);
        }

        if let Some(resource) = self.ledger.get_resource_def(address) {
            self.resources.insert(address, resource);
            self.resources.get(&address)
        } else {
            None
        }
    }

    /// Returns a mutable reference to a resource, if exists.
    #[allow(dead_code)]
    pub fn get_resource_def_mut(&mut self, address: Address) -> Option<&mut ResourceDef> {
        self.updated_resources.insert(address);

        if self.resources.contains_key(&address) {
            return self.resources.get_mut(&address);
        }

        if let Some(resource) = self.ledger.get_resource_def(address) {
            self.resources.insert(address, resource);
            self.resources.get_mut(&address)
        } else {
            None
        }
    }

    /// Inserts a new resource.
    pub fn put_resource_def(&mut self, address: Address, resource: ResourceDef) {
        self.updated_resources.insert(address);

        self.resources.insert(address, resource);
    }

    /// Returns an immutable reference to a vault, if exists.
    #[allow(dead_code)]
    pub fn get_vault(&mut self, vid: VID) -> Option<&Vault> {
        if self.vaults.contains_key(&vid) {
            return self.vaults.get(&vid);
        }

        if let Some(vault) = self.ledger.get_vault(vid) {
            self.vaults.insert(vid, vault);
            self.vaults.get(&vid)
        } else {
            None
        }
    }

    /// Returns a mutable reference to a vault, if exists.
    pub fn get_vault_mut(&mut self, vid: VID) -> Option<&mut Vault> {
        self.updated_vaults.insert(vid);

        if self.vaults.contains_key(&vid) {
            return self.vaults.get_mut(&vid);
        }

        if let Some(vault) = self.ledger.get_vault(vid) {
            self.vaults.insert(vid, vault);
            self.vaults.get_mut(&vid)
        } else {
            None
        }
    }

    /// Inserts a new vault.
    pub fn put_vault(&mut self, vid: VID, vault: Vault) {
        self.updated_vaults.insert(vid);

        self.vaults.insert(vid, vault);
    }

    /// Creates a new package address.
    pub fn new_package_address(&mut self) -> Address {
        let address = self.alloc.new_package_address(self.tx_hash());
        self.new_addresses.push(address);
        address
    }

    /// Creates a new component address.
    pub fn new_component_address(&mut self) -> Address {
        let address = self.alloc.new_component_address(self.tx_hash());
        self.new_addresses.push(address);
        address
    }

    /// Creates a new resource address.
    pub fn new_resource_address(&mut self) -> Address {
        let address = self.alloc.new_resource_address(self.tx_hash());
        self.new_addresses.push(address);
        address
    }

    /// Creates a new bucket ID.
    pub fn new_bucket_id(&mut self) -> BID {
        self.alloc.new_bucket_id()
    }

    /// Creates a new vault ID.
    pub fn new_vault_id(&mut self) -> VID {
        self.alloc.new_vault_id(self.tx_hash())
    }

    /// Creates a new reference id.
    pub fn new_rid(&mut self) -> RID {
        self.alloc.new_rid()
    }

    /// Creates a new map id.
    pub fn new_mid(&mut self) -> MID {
        self.alloc.new_mid(self.tx_hash())
    }

    /// Commits changes to ledger.
    pub fn commit(&mut self) {
        for address in self.updated_packages.clone() {
            self.ledger
                .put_package(address, self.packages.get(&address).unwrap().clone());
        }

        for address in self.updated_components.clone() {
            self.ledger
                .put_component(address, self.components.get(&address).unwrap().clone());
        }

        for mid in self.updated_lazy_maps.clone() {
            self.ledger
                .put_lazy_map(mid, self.lazy_maps.get(&mid).unwrap().clone());
        }

        for address in self.updated_resources.clone() {
            self.ledger
                .put_resource_def(address, self.resources.get(&address).unwrap().clone());
        }

        for vault in self.updated_vaults.clone() {
            self.ledger
                .put_vault(vault, self.vaults.get(&vault).unwrap().clone());
        }
    }
}
