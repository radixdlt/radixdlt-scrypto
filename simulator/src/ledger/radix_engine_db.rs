use std::path::PathBuf;

use radix_engine::ledger::*;
use radix_engine::model::*;
use rocksdb::{DBWithThreadMode, Direction, IteratorMode, SingleThreaded, DB};
use sbor::*;
use scrypto::buffer::*;
use scrypto::engine::types::*;

pub struct RadixEngineDB {
    db: DBWithThreadMode<SingleThreaded>,
}

impl RadixEngineDB {
    pub fn new(root: PathBuf) -> Self {
        let db = DB::open_default(root.as_path()).unwrap();
        Self { db }
    }

    pub fn with_bootstrap(root: PathBuf) -> Self {
        let mut ledger = Self::new(root);
        ledger.bootstrap();
        ledger
    }

    pub fn list_packages(&self) -> Vec<PackageId> {
        self.list_items(PackageId([0; 26]), PackageId([255; 26]))
    }

    pub fn list_components(&self) -> Vec<ComponentId> {
        self.list_items(ComponentId([0; 26]), ComponentId([255; 26]))
    }

    pub fn list_resource_defs(&self) -> Vec<ResourceDefId> {
        self.list_items(ResourceDefId([0; 26]), ResourceDefId([255; 26]))
    }

    fn list_items<K: Encode + Decode>(&self, start: K, end: K) -> Vec<K> {
        let mut iter = self.db.iterator(IteratorMode::From(
            &scrypto_encode(&start),
            Direction::Forward,
        ));
        let mut items = Vec::new();
        while let Some(kv) = iter.next() {
            if kv.0.as_ref() > &scrypto_encode(&end) {
                break;
            }
            items.push(scrypto_decode(kv.0.as_ref()).unwrap());
        }
        items
    }

    fn read<K: Encode, V: Decode>(&self, key: &K) -> Option<V> {
        self.db
            .get(scrypto_encode(key))
            .unwrap()
            .map(|bytes| scrypto_decode(&bytes).unwrap())
    }

    fn write<K: Encode, V: Encode>(&self, key: K, value: V) {
        self.db
            .put(scrypto_encode(&key), scrypto_encode(&value))
            .unwrap();
    }
}

impl SubstateStore for RadixEngineDB {
    fn get_resource_def(&self, resource_def_id: ResourceDefId) -> Option<ResourceDef> {
        self.read(&resource_def_id)
    }

    fn put_resource_def(&mut self, resource_def_id: ResourceDefId, resource_def: ResourceDef) {
        self.write(resource_def_id, resource_def)
    }

    fn get_package(&self, package_id: PackageId) -> Option<Package> {
        self.read(&package_id)
    }

    fn put_package(&mut self, package_id: PackageId, package: Package) {
        self.write(package_id, package)
    }

    fn get_component(&self, component_id: ComponentId) -> Option<Component> {
        self.read(&component_id)
    }

    fn put_component(&mut self, component_id: ComponentId, component: Component) {
        self.write(component_id, component)
    }

    fn get_lazy_map(&self, component_id: ComponentId, lazy_map_id: LazyMapId) -> Option<LazyMap> {
        self.read(&(component_id.clone(), lazy_map_id))
    }

    fn put_lazy_map(
        &mut self,
        component_id: ComponentId,
        lazy_map_id: LazyMapId,
        lazy_map: LazyMap,
    ) {
        self.write((component_id, lazy_map_id), lazy_map)
    }

    fn get_vault(&self, component_id: ComponentId, vault_id: VaultId) -> Option<Vault> {
        self.read(&(component_id.clone(), vault_id.clone()))
    }

    fn put_vault(&mut self, component_id: ComponentId, vault_id: VaultId, vault: Vault) {
        self.write((component_id, vault_id), vault)
    }

    fn get_non_fungible(
        &self,
        resource_def_id: ResourceDefId,
        key: &NonFungibleKey,
    ) -> Option<NonFungible> {
        self.read(&(resource_def_id, key.clone()))
    }

    fn put_non_fungible(
        &mut self,
        resource_def_id: ResourceDefId,
        key: &NonFungibleKey,
        non_fungible: NonFungible,
    ) {
        self.write((resource_def_id, key.clone()), non_fungible)
    }

    fn get_epoch(&self) -> u64 {
        self.read(&"epoch").unwrap_or(0)
    }

    fn set_epoch(&mut self, epoch: u64) {
        self.write("epoch", epoch)
    }

    fn get_nonce(&self) -> u64 {
        self.read(&"nonce").unwrap_or(0)
    }

    fn increase_nonce(&mut self) {
        self.write("nonce", self.get_nonce() + 1)
    }
}
