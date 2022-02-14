use std::path::PathBuf;

use radix_engine::ledger::*;
use radix_engine::model::*;
use rocksdb::{DBWithThreadMode, Direction, IteratorMode, SingleThreaded, DB};
use sbor::*;
use scrypto::buffer::*;
use scrypto::types::*;

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

    pub fn list_packages(&self) -> Vec<Address> {
        let start = &scrypto_encode(&Address::Package([0; 26]));
        let end = &scrypto_encode(&Address::Package([255; 26]));
        self.list_items(start, end)
    }

    pub fn list_components(&self) -> Vec<Address> {
        let start = &scrypto_encode(&Address::Component([0; 26]));
        let end = &scrypto_encode(&Address::Component([255; 26]));
        self.list_items(start, end)
    }

    pub fn list_resource_defs(&self) -> Vec<Address> {
        let start = &scrypto_encode(&Address::ResourceDef([0; 26]));
        let end = &scrypto_encode(&Address::ResourceDef([255; 26]));
        self.list_items(start, end)
    }

    fn list_items(&self, start: &[u8], inclusive_end: &[u8]) -> Vec<Address> {
        let mut iter = self.db.iterator(IteratorMode::From(
            start,
            Direction::Forward,
        ));
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
        self.db
            .put(key, scrypto_encode(&value))
            .unwrap();
    }
}

impl SubstateStore for RadixEngineDB {
    fn get_resource_def(&self, address: Address) -> Option<ResourceDef> {
        self.read(&scrypto_encode(&address))
    }

    fn put_resource_def(&mut self, address: Address, resource_def: ResourceDef) {
        let key = &scrypto_encode(&address);
        self.write(key, resource_def)
    }

    fn get_package(&self, address: Address) -> Option<Package> {
        self.read(&scrypto_encode(&address))
    }

    fn put_package(&mut self, address: Address, package: Package) {
        let key = &scrypto_encode(&address);
        self.write(key, package)
    }

    fn get_component(&self, address: Address) -> Option<Component> {
        self.read(&scrypto_encode(&address))
    }

    fn put_component(&mut self, address: Address, component: Component) {
        let key = &scrypto_encode(&address);
        self.write(key, component)
    }

    fn get_lazy_map_entry(
        &self,
        component_address: &Address,
        mid: &Mid,
        key: &[u8],
    ) -> Option<Vec<u8>> {
        let mut id = scrypto_encode(component_address);
        id.extend(scrypto_encode(mid));
        id.extend(key.to_vec());
        self.read(&id)
    }

    fn put_lazy_map_entry(
        &mut self,
        component_address: Address,
        mid: Mid,
        key: Vec<u8>,
        value: Vec<u8>,
    ) {
        let mut id = scrypto_encode(&component_address);
        id.extend(scrypto_encode(&mid));
        id.extend(key);
        self.write(&id, value)
    }

    fn get_vault(&self, component_address: &Address, vid: &Vid) -> Vault {
        let mut id = scrypto_encode(component_address);
        id.extend(scrypto_encode(vid));
        self.read(&id).unwrap()
    }

    fn put_vault(&mut self, component_address: Address, vid: Vid, vault: Vault) {
        let mut id = scrypto_encode(&component_address);
        id.extend(scrypto_encode(&vid));
        self.write(&id, vault)
    }

    fn get_non_fungible(
        &self,
        resource_address: Address,
        key: &NonFungibleKey,
    ) -> Option<NonFungible> {
        let id = scrypto_encode(&(resource_address, key.clone()));
        self.read(&id)
    }

    fn put_non_fungible(
        &mut self,
        resource_address: Address,
        key: &NonFungibleKey,
        non_fungible: NonFungible,
    ) {
        let id = scrypto_encode(&(resource_address, key.clone()));
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
