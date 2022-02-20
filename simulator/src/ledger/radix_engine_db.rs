use std::collections::HashMap;
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
        let start = &scrypto_encode(&PackageId([0; 26]));
        let end = &scrypto_encode(&PackageId([255; 26]));
        self.list_items(start, end)
    }

    pub fn list_components(&self) -> Vec<ComponentId> {
        let start = &scrypto_encode(&ComponentId([0; 26]));
        let end = &scrypto_encode(&ComponentId([255; 26]));
        self.list_items(start, end)
    }

    pub fn list_resource_defs(&self) -> Vec<ResourceDefId> {
        let start = &scrypto_encode(&ResourceDefId([0; 26]));
        let end = &scrypto_encode(&ResourceDefId([255; 26]));
        self.list_items(start, end)
    }

    fn list_items<T: Decode>(&self, start: &[u8], inclusive_end: &[u8]) -> Vec<T> {
        let mut iter = self
            .db
            .iterator(IteratorMode::From(start, Direction::Forward));
        let mut items = Vec::new();
        while let Some(kv) = iter.next() {
            if kv.0.as_ref() > inclusive_end {
                break;
            }
            if kv.0.len() == 27 {
                items.push(scrypto_decode(kv.0.as_ref()).unwrap());
            }
        }
        items
    }

    fn read<V: Decode>(&self, key: &[u8]) -> Option<V> {
        self.db
            .get(key)
            .unwrap()
            .map(|bytes| scrypto_decode(&bytes).unwrap())
    }

    fn write<V: Encode>(&self, key: &[u8], value: V) {
        self.db.put(key, scrypto_encode(&value)).unwrap();
    }
}

impl QueryableSubstateStore for RadixEngineDB {
    fn get_lazy_map_entries(
        &self,
        component_id: ComponentId,
        lazy_map_id: &LazyMapId,
    ) -> HashMap<Vec<u8>, Vec<u8>> {
        let mut id = scrypto_encode(&component_id);
        id.extend(scrypto_encode(lazy_map_id));

        let mut iter = self
            .db
            .iterator(IteratorMode::From(&id, Direction::Forward));
        let mut items = HashMap::new();
        while let Some((key, value)) = iter.next() {
            if !key.starts_with(&id) {
                break;
            }
            items.insert(key.to_vec(), value.to_vec());
        }
        items
    }
}

impl SubstateStore for RadixEngineDB {
    fn get_resource_def(&self, resource_def_id: ResourceDefId) -> Option<ResourceDef> {
        self.read(&scrypto_encode(&resource_def_id))
    }

    fn put_resource_def(&mut self, resource_def_id: ResourceDefId, resource_def: ResourceDef) {
        let key = &scrypto_encode(&resource_def_id);
        self.write(key, resource_def)
    }

    fn get_package(&self, package_id: PackageId) -> Option<Package> {
        self.read(&scrypto_encode(&package_id))
    }

    fn put_package(&mut self, package_id: PackageId, package: Package) {
        let key = &scrypto_encode(&package_id);
        self.write(key, package)
    }

    fn get_component(&self, component_id: ComponentId) -> Option<Component> {
        self.read(&scrypto_encode(&component_id))
    }

    fn put_component(&mut self, component_id: ComponentId, component: Component) {
        let key = &scrypto_encode(&component_id);
        self.write(key, component)
    }

    fn get_lazy_map_entry(
        &self,
        component_id: ComponentId,
        lazy_map_id: &LazyMapId,
        key: &[u8],
    ) -> Option<Vec<u8>> {
        let mut id = scrypto_encode(&component_id);
        id.extend(scrypto_encode(lazy_map_id));
        id.extend(key.to_vec());
        self.read(&id)
    }

    fn put_lazy_map_entry(
        &mut self,
        component_id: ComponentId,
        lazy_map_id: LazyMapId,
        key: Vec<u8>,
        value: Vec<u8>,
    ) {
        let mut id = scrypto_encode(&component_id);
        id.extend(scrypto_encode(&lazy_map_id));
        id.extend(key);
        self.write(&id, value)
    }

    fn get_vault(&self, component_id: ComponentId, vault_id: &VaultId) -> Vault {
        let mut id = scrypto_encode(&component_id);
        id.extend(scrypto_encode(vault_id));
        self.read(&id).unwrap()
    }

    fn put_vault(&mut self, component_id: ComponentId, vault_id: VaultId, vault: Vault) {
        let mut id = scrypto_encode(&component_id);
        id.extend(scrypto_encode(&vault_id));
        self.write(&id, vault)
    }

    fn get_non_fungible(
        &self,
        resource_def_id: ResourceDefId,
        key: &NonFungibleKey,
    ) -> Option<NonFungible> {
        let id = scrypto_encode(&(resource_def_id, key.clone()));
        self.read(&id)
    }

    fn put_non_fungible(
        &mut self,
        resource_def_id: ResourceDefId,
        key: &NonFungibleKey,
        non_fungible: NonFungible,
    ) {
        let id = scrypto_encode(&(resource_def_id, key.clone()));
        self.write(&id, non_fungible)
    }

    fn get_epoch(&self) -> u64 {
        let id = scrypto_encode(&"epoch");
        self.read(&id).unwrap_or(0)
    }

    fn set_epoch(&mut self, epoch: u64) {
        let id = scrypto_encode(&"epoch");
        self.write(&id, epoch)
    }

    fn get_nonce(&self) -> u64 {
        let id = scrypto_encode(&"nonce");
        self.read(&id).unwrap_or(0)
    }

    fn increase_nonce(&mut self) {
        let id = scrypto_encode(&"nonce");
        self.write(&id, self.get_nonce() + 1)
    }
}
