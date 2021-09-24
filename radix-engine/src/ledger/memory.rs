use scrypto::rust::collections::HashMap;
use scrypto::types::*;

use crate::ledger::*;
use crate::model::*;

/// An in-memory ledger stores all substates in host memory.
pub struct InMemoryLedger {
    packages: HashMap<Address, Package>,
    components: HashMap<Address, Component>,
    storages: HashMap<SID, Storage>,
    resource_defs: HashMap<Address, ResourceDef>,
    vaults: HashMap<VID, Vault>,
}

impl InMemoryLedger {
    pub fn new() -> Self {
        Self {
            packages: HashMap::new(),
            components: HashMap::new(),
            storages: HashMap::new(),
            resource_defs: HashMap::new(),
            vaults: HashMap::new(),
        }
    }
}

impl Default for InMemoryLedger {
    fn default() -> Self {
        Self::new()
    }
}

impl Ledger for InMemoryLedger {
    fn get_resource_def(&self, address: Address) -> Option<ResourceDef> {
        self.resource_defs.get(&address).map(Clone::clone)
    }

    fn put_resource_def(&mut self, address: Address, resource: ResourceDef) {
        self.resource_defs.insert(address, resource);
    }

    fn get_package(&self, address: Address) -> Option<Package> {
        self.packages.get(&address).map(Clone::clone)
    }

    fn put_package(&mut self, address: Address, package: Package) {
        self.packages.insert(address, package);
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

    fn get_vault(&self, vid: VID) -> Option<Vault> {
        self.vaults.get(&vid).map(Clone::clone)
    }

    fn put_vault(&mut self, vid: VID, vault: Vault) {
        self.vaults.insert(vid, vault);
    }
}
