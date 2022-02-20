use scrypto::buffer::*;
use scrypto::engine::types::*;
use scrypto::rust::collections::HashMap;
use scrypto::rust::vec::Vec;

use crate::ledger::*;
use crate::model::*;

/// An in-memory ledger stores all substates in host memory.
#[derive(Debug, Clone)]
pub struct InMemorySubstateStore {
    packages: HashMap<PackageId, Package>,
    components: HashMap<ComponentId, Component>,
    lazy_map_entries: HashMap<(ComponentId, LazyMapId, Vec<u8>), Vec<u8>>,
    resource_defs: HashMap<ResourceDefId, ResourceDef>,
    vaults: HashMap<(ComponentId, VaultId), Vec<u8>>,
    non_fungibles: HashMap<(ResourceDefId, NonFungibleKey), NonFungible>,
    current_epoch: u64,
    nonce: u64,
}

impl InMemorySubstateStore {
    pub fn new() -> Self {
        Self {
            packages: HashMap::new(),
            components: HashMap::new(),
            lazy_map_entries: HashMap::new(),
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
    fn get_resource_def(&self, resource_def_id: ResourceDefId) -> Option<ResourceDef> {
        self.resource_defs.get(&resource_def_id).map(Clone::clone)
    }

    fn put_resource_def(&mut self, resource_def_id: ResourceDefId, resource_def: ResourceDef) {
        self.resource_defs.insert(resource_def_id, resource_def);
    }

    fn get_package(&self, package_id: PackageId) -> Option<Package> {
        self.packages.get(&package_id).map(Clone::clone)
    }

    fn put_package(&mut self, package_id: PackageId, package: Package) {
        self.packages.insert(package_id, package);
    }

    fn get_component(&self, component_id: ComponentId) -> Option<Component> {
        self.components.get(&component_id).map(Clone::clone)
    }

    fn put_component(&mut self, component_id: ComponentId, component: Component) {
        self.components.insert(component_id, component);
    }

    fn get_lazy_map_entry(
        &self,
        component_id: ComponentId,
        lazy_map_id: &LazyMapId,
        key: &[u8],
    ) -> Option<Vec<u8>> {
        self.lazy_map_entries
            .get(&(component_id.clone(), lazy_map_id.clone(), key.to_vec()))
            .cloned()
    }

    fn put_lazy_map_entry(
        &mut self,
        component_id: ComponentId,
        lazy_map_id: LazyMapId,
        key: Vec<u8>,
        value: Vec<u8>,
    ) {
        self.lazy_map_entries
            .insert((component_id, lazy_map_id, key), value);
    }

    fn get_vault(&self, component_id: ComponentId, vault_id: &VaultId) -> Vault {
        self.vaults
            .get(&(component_id.clone(), vault_id.clone()))
            .map(|data| scrypto_decode(data).unwrap())
            .unwrap()
    }

    fn put_vault(&mut self, component_id: ComponentId, vault_id: VaultId, vault: Vault) {
        let data = scrypto_encode(&vault);
        self.vaults.insert((component_id, vault_id), data);
    }

    fn get_non_fungible(
        &self,
        resource_def_id: ResourceDefId,
        key: &NonFungibleKey,
    ) -> Option<NonFungible> {
        self.non_fungibles
            .get(&(resource_def_id, key.clone()))
            .cloned()
    }

    fn put_non_fungible(
        &mut self,
        resource_def_id: ResourceDefId,
        key: &NonFungibleKey,
        non_fungible: NonFungible,
    ) {
        self.non_fungibles
            .insert((resource_def_id, key.clone()), non_fungible);
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
