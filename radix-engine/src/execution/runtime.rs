use lru::LruCache;
use scrypto::rust::collections::*;
use scrypto::rust::string::String;
use scrypto::rust::vec::Vec;
use scrypto::types::*;
use wasmi::*;

use crate::execution::*;
use crate::ledger::*;
use crate::model::*;

/// Runtime is an abstraction to the execution state of a transaction.
///
/// It acts as the facade of ledger state and keeps track of all temporary state updates,
/// util the `flush()` method is called.
///
/// Typically, a runtime is shared by a series of processes, created during the life time
/// of a transaction.
///
pub struct Runtime<'le, T: Ledger> {
    tx_hash: H256,
    ledger: &'le mut T,
    alloc: AddressAllocator,
    logs: Vec<(Level, String)>,
    packages: HashMap<Address, Package>,
    components: HashMap<Address, Component>,
    storages: HashMap<SID, Storage>,
    resources: HashMap<Address, Resource>,
    buckets: HashMap<BID, PersistedBucket>,
    updated_packages: HashSet<Address>,
    updated_components: HashSet<Address>,
    updated_storages: HashSet<SID>,
    updated_resources: HashSet<Address>,
    updated_buckets: HashSet<BID>,
    new_addresses: Vec<Address>,
    cache: LruCache<Address, Module>, // TODO: move to ledger level
}

impl<'le, T: Ledger> Runtime<'le, T> {
    pub fn new(tx_hash: H256, ledger: &'le mut T) -> Self {
        Self {
            tx_hash,
            ledger,
            alloc: AddressAllocator::new(),
            logs: Vec::new(),
            packages: HashMap::new(),
            components: HashMap::new(),
            storages: HashMap::new(),
            resources: HashMap::new(),
            buckets: HashMap::new(),
            updated_packages: HashSet::new(),
            updated_components: HashSet::new(),
            updated_storages: HashSet::new(),
            updated_resources: HashSet::new(),
            updated_buckets: HashSet::new(),
            new_addresses: Vec::new(),
            cache: LruCache::new(1024),
        }
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
    pub fn get_storage(&mut self, sid: SID) -> Option<&Storage> {
        if self.storages.contains_key(&sid) {
            return self.storages.get(&sid);
        }

        if let Some(storage) = self.ledger.get_storage(sid) {
            self.storages.insert(sid, storage);
            self.storages.get(&sid)
        } else {
            None
        }
    }
    /// Returns a mutable reference to a map, if exists.
    pub fn get_storage_mut(&mut self, sid: SID) -> Option<&mut Storage> {
        self.updated_storages.insert(sid);

        if self.storages.contains_key(&sid) {
            return self.storages.get_mut(&sid);
        }

        if let Some(storage) = self.ledger.get_storage(sid) {
            self.storages.insert(sid, storage);
            self.storages.get_mut(&sid)
        } else {
            None
        }
    }

    /// Inserts a new map.
    pub fn put_storage(&mut self, sid: SID, storage: Storage) {
        self.updated_storages.insert(sid);

        self.storages.insert(sid, storage);
    }

    /// Returns an immutable reference to a resource, if exists.
    pub fn get_resource(&mut self, address: Address) -> Option<&Resource> {
        if self.resources.contains_key(&address) {
            return self.resources.get(&address);
        }

        if let Some(resource) = self.ledger.get_resource(address) {
            self.resources.insert(address, resource);
            self.resources.get(&address)
        } else {
            None
        }
    }

    /// Returns a mutable reference to a resource, if exists.
    #[allow(dead_code)]
    pub fn get_resource_mut(&mut self, address: Address) -> Option<&mut Resource> {
        self.updated_resources.insert(address);

        if self.resources.contains_key(&address) {
            return self.resources.get_mut(&address);
        }

        if let Some(resource) = self.ledger.get_resource(address) {
            self.resources.insert(address, resource);
            self.resources.get_mut(&address)
        } else {
            None
        }
    }

    /// Inserts a new resource.
    pub fn put_resource(&mut self, address: Address, resource: Resource) {
        self.updated_resources.insert(address);

        self.resources.insert(address, resource);
    }

    /// Returns an immutable reference to a bucket, if exists.
    #[allow(dead_code)]
    pub fn get_bucket(&mut self, bid: BID) -> Option<&PersistedBucket> {
        if self.buckets.contains_key(&bid) {
            return self.buckets.get(&bid);
        }

        if let Some(bucket) = self.ledger.get_bucket(bid) {
            self.buckets.insert(bid, bucket);
            self.buckets.get(&bid)
        } else {
            None
        }
    }

    /// Returns a mutable reference to a bucket, if exists.
    pub fn get_bucket_mut(&mut self, bid: BID) -> Option<&mut PersistedBucket> {
        self.updated_buckets.insert(bid);

        if self.buckets.contains_key(&bid) {
            return self.buckets.get_mut(&bid);
        }

        if let Some(bucket) = self.ledger.get_bucket(bid) {
            self.buckets.insert(bid, bucket);
            self.buckets.get_mut(&bid)
        } else {
            None
        }
    }

    /// Inserts a new bucket.
    pub fn put_bucket(&mut self, bid: BID, bucket: PersistedBucket) {
        self.updated_buckets.insert(bid);

        self.buckets.insert(bid, bucket);
    }

    /// Creates a new package bid.
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

    /// Creates a new transient bucket id.
    pub fn new_transient_bid(&mut self) -> BID {
        self.alloc.new_transient_bid()
    }

    /// Creates a new persisted bucket id.
    pub fn new_persisted_bid(&mut self) -> BID {
        self.alloc.new_persisted_bid(self.tx_hash())
    }

    /// Creates a new reference id.
    pub fn new_rid(&mut self) -> RID {
        self.alloc.new_rid()
    }

    /// Creates a new map id.
    pub fn new_sid(&mut self) -> SID {
        self.alloc.new_sid(self.tx_hash())
    }

    /// Flush changes to ledger.
    pub fn flush(&mut self) {
        for address in self.updated_packages.clone() {
            self.ledger
                .put_package(address, self.packages.get(&address).unwrap().clone());
        }

        for address in self.updated_components.clone() {
            self.ledger
                .put_component(address, self.components.get(&address).unwrap().clone());
        }

        for sid in self.updated_storages.clone() {
            self.ledger
                .put_storage(sid, self.storages.get(&sid).unwrap().clone());
        }

        for address in self.updated_resources.clone() {
            self.ledger
                .put_resource(address, self.resources.get(&address).unwrap().clone());
        }

        for bucket in self.updated_buckets.clone() {
            self.ledger
                .put_bucket(bucket, self.buckets.get(&bucket).unwrap().clone());
        }
    }
}
