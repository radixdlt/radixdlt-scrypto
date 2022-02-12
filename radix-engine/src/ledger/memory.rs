use scrypto::rust::collections::HashMap;
use scrypto::types::*;

use crate::ledger::*;
use crate::model::*;

/// An in-memory ledger stores all substates in host memory.
#[derive(Debug, Clone)]
pub struct InMemorySubstateStore {
    packages: HashMap<Address, Package>,
    components: HashMap<Address, Component>,
    lazy_maps: HashMap<(Address, Mid), LazyMap>,
    resource_defs: HashMap<Address, ResourceDef>,
    vaults: HashMap<(Address, Vid), Vault>,
    non_fungibles: HashMap<(Address, NonFungibleKey), NonFungible>,
    current_epoch: u64,
    nonce: u64,
}

impl InMemorySubstateStore {
    pub fn new() -> Self {
        Self {
            packages: HashMap::new(),
            components: HashMap::new(),
            lazy_maps: HashMap::new(),
            resource_defs: HashMap::new(),
            vaults: HashMap::new(),
            non_fungibles: HashMap::new(),
            current_epoch: 0,
            nonce: 0,
        }
    }

    pub fn with_bootstrap() -> Self {
        let mut ledger = Self::new();
        ledger.bootstrap();
        ledger
    }
}

impl Default for InMemorySubstateStore {
    fn default() -> Self {
        Self::new()
    }
}

impl SubstateStore for InMemorySubstateStore {
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

    fn get_lazy_map(&self, component_address: &Address, mid: &Mid) -> LazyMap {
        self.lazy_maps
            .get(&(component_address.clone(), mid.clone()))
            .map(Clone::clone)
            .unwrap()
    }

    fn put_lazy_map(&mut self, component_address: Address, mid: Mid, lazy_map: LazyMap) {
        self.lazy_maps.insert((component_address, mid), lazy_map);
    }

    fn get_vault(&self, component_address: &Address, vid: &Vid) -> Vault {
        self.vaults
            .get(&(component_address.clone(), vid.clone()))
            .map(Clone::clone)
            .unwrap()
    }

    fn put_vault(&mut self, component_address: Address, vid: Vid, vault: Vault) {
        self.vaults.insert((component_address, vid), vault);
    }

    fn get_non_fungible(
        &self,
        resource_address: Address,
        key: &NonFungibleKey,
    ) -> Option<NonFungible> {
        self.non_fungibles
            .get(&(resource_address, key.clone()))
            .cloned()
    }

    fn put_non_fungible(
        &mut self,
        resource_address: Address,
        key: &NonFungibleKey,
        non_fungible: NonFungible,
    ) {
        self.non_fungibles
            .insert((resource_address, key.clone()), non_fungible);
    }

    fn get_epoch(&self) -> u64 {
        self.current_epoch
    }

    fn set_epoch(&mut self, epoch: u64) {
        self.current_epoch = epoch;
    }

    fn get_nonce(&self) -> u64 {
        self.nonce
    }

    fn increase_nonce(&mut self) {
        self.nonce += 1;
    }
}
