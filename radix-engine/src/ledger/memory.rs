use scrypto::rust::collections::HashMap;
use scrypto::types::*;

use crate::ledger::*;
use crate::model::*;

/// An in-memory ledger stores all substates in host memory.
pub struct InMemoryLedger {
    packages: HashMap<Address, Package>,
    components: HashMap<Address, Component>,
    lazy_maps: HashMap<MID, LazyMap>,
    resource_defs: HashMap<Address, ResourceDef>,
    vaults: HashMap<VID, Vault>,
}

impl InMemoryLedger {
    pub fn new() -> Self {
        Self {
            packages: HashMap::new(),
            components: HashMap::new(),
            lazy_maps: HashMap::new(),
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

    fn put_resource_def(&mut self, address: Address, resource_def: ResourceDef) {
        self.resource_defs.insert(address, resource_def);
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

    fn get_lazy_map(&self, mid: MID) -> Option<LazyMap> {
        self.lazy_maps.get(&mid).map(Clone::clone)
    }

    fn put_lazy_map(&mut self, mid: MID, lazy_map: LazyMap) {
        self.lazy_maps.insert(mid, lazy_map);
    }

    fn get_vault(&self, vid: VID) -> Option<Vault> {
        self.vaults.get(&vid).map(Clone::clone)
    }

    fn put_vault(&mut self, vid: VID, vault: Vault) {
        self.vaults.insert(vid, vault);
    }
}
