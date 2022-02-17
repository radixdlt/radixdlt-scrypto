use scrypto::buffer::*;
use scrypto::engine::types::*;
use scrypto::rust::collections::HashMap;
use scrypto::rust::vec::Vec;

use crate::ledger::*;
use crate::model::*;

/// An in-memory ledger stores all substates in host memory.
#[derive(Debug, Clone)]
pub struct InMemorySubstateStore {
    packages: HashMap<PackageRef, Package>,
    components: HashMap<ComponentRef, Component>,
    lazy_maps: HashMap<(ComponentRef, LazyMapId), LazyMap>,
    resource_defs: HashMap<ResourceDefRef, ResourceDef>,
    vaults: HashMap<(ComponentRef, VaultId), Vec<u8>>,
    non_fungibles: HashMap<(ResourceDefRef, NonFungibleKey), NonFungible>,
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
    fn get_resource_def(&self, resource_def_ref: ResourceDefRef) -> Option<ResourceDef> {
        self.resource_defs.get(&resource_def_ref).map(Clone::clone)
    }

    fn put_resource_def(&mut self, resource_def_ref: ResourceDefRef, resource_def: ResourceDef) {
        self.resource_defs.insert(resource_def_ref, resource_def);
    }

    fn get_package(&self, package_ref: PackageRef) -> Option<Package> {
        self.packages.get(&package_ref).map(Clone::clone)
    }

    fn put_package(&mut self, package_ref: PackageRef, package: Package) {
        self.packages.insert(package_ref, package);
    }

    fn get_component(&self, component_ref: ComponentRef) -> Option<Component> {
        self.components.get(&component_ref).map(Clone::clone)
    }

    fn put_component(&mut self, component_ref: ComponentRef, component: Component) {
        self.components.insert(component_ref, component);
    }

    fn get_lazy_map(&self, component_ref: ComponentRef, lazy_map_id: LazyMapId) -> Option<LazyMap> {
        self.lazy_maps
            .get(&(component_ref, lazy_map_id))
            .map(Clone::clone)
    }

    fn put_lazy_map(
        &mut self,
        component_ref: ComponentRef,
        lazy_map_id: LazyMapId,
        lazy_map: LazyMap,
    ) {
        self.lazy_maps
            .insert((component_ref, lazy_map_id), lazy_map);
    }

    fn get_vault(&self, component_ref: ComponentRef, vault_id: VaultId) -> Option<Vault> {
        self.vaults
            .get(&(component_ref.clone(), vault_id.clone()))
            .map(|data| scrypto_decode(data).unwrap())
    }

    fn put_vault(&mut self, component_ref: ComponentRef, vault_id: VaultId, vault: Vault) {
        let data = scrypto_encode(&vault);
        self.vaults.insert((component_ref, vault_id), data);
    }

    fn get_non_fungible(
        &self,
        resource_def_ref: ResourceDefRef,
        key: &NonFungibleKey,
    ) -> Option<NonFungible> {
        self.non_fungibles
            .get(&(resource_def_ref, key.clone()))
            .cloned()
    }

    fn put_non_fungible(
        &mut self,
        resource_def_ref: ResourceDefRef,
        key: &NonFungibleKey,
        non_fungible: NonFungible,
    ) {
        self.non_fungibles
            .insert((resource_def_ref, key.clone()), non_fungible);
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
