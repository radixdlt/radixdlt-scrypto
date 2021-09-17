use scrypto::rust::collections::HashMap;
use scrypto::types::*;

use crate::ledger::*;
use crate::model::*;

/// An in-memory ledger stores all substates in host memory.
pub struct InMemoryLedger {
    packages: HashMap<Address, Package>,
    components: HashMap<Address, Component>,
    storages: HashMap<SID, Storage>,
    resources: HashMap<Address, Resource>,
    buckets: HashMap<BID, PersistentBucket>,
}

impl InMemoryLedger {
    pub fn new() -> Self {
        Self {
            packages: HashMap::new(),
            components: HashMap::new(),
            storages: HashMap::new(),
            resources: HashMap::new(),
            buckets: HashMap::new(),
        }
    }
}

impl Default for InMemoryLedger {
    fn default() -> Self {
        Self::new()
    }
}

impl Ledger for InMemoryLedger {
    fn get_package(&self, address: Address) -> Option<Package> {
        self.packages.get(&address).map(Clone::clone)
    }

    fn put_package(&mut self, address: Address, package: Package) {
        self.packages.insert(address, package);
    }

    fn get_resource(&self, address: Address) -> Option<Resource> {
        self.resources.get(&address).map(Clone::clone)
    }

    fn put_resource(&mut self, address: Address, info: Resource) {
        self.resources.insert(address, info);
    }

    fn get_component(&self, address: Address) -> Option<Component> {
        self.components.get(&address).map(Clone::clone)
    }

    fn put_component(&mut self, address: Address, component: Component) {
        self.components.insert(address, component);
    }

    fn get_storage(&self, sid: SID) -> Option<Storage> {
        self.storages.get(&sid).map(Clone::clone)
    }

    fn put_storage(&mut self, sid: SID, storage: Storage) {
        self.storages.insert(sid, storage);
    }

    fn get_bucket(&self, bid: BID) -> Option<PersistentBucket> {
        self.buckets.get(&bid).map(Clone::clone)
    }

    fn put_bucket(&mut self, bid: BID, bucket: PersistentBucket) {
        self.buckets.insert(bid, bucket);
    }
}
