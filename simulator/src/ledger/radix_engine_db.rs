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
        self.list_items(Address::Package([0; 26]), Address::Package([255; 26]))
    }

    pub fn list_components(&self) -> Vec<Address> {
        self.list_items(Address::Component([0; 26]), Address::Component([255; 26]))
    }

    pub fn list_resource_defs(&self) -> Vec<Address> {
        self.list_items(
            Address::ResourceDef([0; 26]),
            Address::ResourceDef([255; 26]),
        )
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

    fn read<K: Encode, V: Decode>(&self, key: K) -> Option<V> {
        self.db
            .get(scrypto_encode(&key))
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
    fn get_resource_def(&self, address: Address) -> Option<ResourceDef> {
        self.read(address)
    }

    fn put_resource_def(&mut self, address: Address, resource_def: ResourceDef) {
        self.write(address, resource_def)
    }

    fn get_package(&self, address: Address) -> Option<Package> {
        self.read(address)
    }

    fn put_package(&mut self, address: Address, package: Package) {
        self.write(address, package)
    }

    fn get_component(&self, address: Address) -> Option<Component> {
        self.read(address)
    }

    fn put_component(&mut self, address: Address, component: Component) {
        self.write(address, component)
    }

    fn get_lazy_map(&self, mid: Mid) -> Option<LazyMap> {
        self.read(mid)
    }

    fn put_lazy_map(&mut self, mid: Mid, lazy_map: LazyMap) {
        self.write(mid, lazy_map)
    }

    fn get_vault(&self, vid: Vid) -> Option<Vault> {
        self.read(vid)
    }

    fn put_vault(&mut self, vid: Vid, vault: Vault) {
        self.write(vid, vault)
    }

    fn get_non_fungible(&self, resource_address: Address, key: &NonFungibleKey) -> Option<NonFungible> {
        self.read((resource_address, key.clone()))
    }

    fn put_non_fungible(&mut self, resource_address: Address, key: &NonFungibleKey, non_fungible: NonFungible) {
        self.write((resource_address, key.clone()), non_fungible)
    }

    fn get_epoch(&self) -> u64 {
        self.read("epoch").unwrap_or(0)
    }

    fn set_epoch(&mut self, epoch: u64) {
        self.write("epoch", epoch)
    }

    fn get_nonce(&self) -> u64 {
        self.read("nonce").unwrap_or(0)
    }

    fn increase_nonce(&mut self) {
        self.write("nonce", self.get_nonce() + 1)
    }
}
