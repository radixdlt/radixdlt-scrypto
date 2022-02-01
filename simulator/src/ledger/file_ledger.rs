use std::path::{PathBuf};

use rocksdb::{DB, DBWithThreadMode, Direction, IteratorMode, SingleThreaded};
use radix_engine::ledger::*;
use radix_engine::model::*;
use scrypto::types::*;

/// A file-based ledger that stores substates in a folder.
pub struct FileBasedLedger {
    db: DBWithThreadMode<SingleThreaded>,
}

impl FileBasedLedger {
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
        self.list_items(Address::ResourceDef([0; 26]), Address::ResourceDef([255; 26]))
    }

    fn list_items(&self, start: Address, end: Address) -> Vec<Address> {
        let mut iter = self.db.iterator(IteratorMode::From(&start.to_vec(), Direction::Forward));
        let mut items = Vec::new();
        while let Some(kv) = iter.next() {
            if kv.0.as_ref() > &end.to_vec() {
                break;
            }
            items.push(Address::try_from(kv.0.as_ref()).unwrap());
        }
        items
    }

    pub fn encode<T: sbor::Encode>(v: &T) -> Vec<u8> {
        sbor::encode_with_type(Vec::with_capacity(512), v)
    }

    pub fn decode<T: sbor::Decode>(bytes: Vec<u8>) -> T {
        sbor::decode_with_type(&bytes).unwrap()
    }
}

impl SubstateStore for FileBasedLedger {
    fn get_resource_def(&self, address: Address) -> Option<ResourceDef> {
        self.db.get(address.to_vec()).unwrap().map(Self::decode)
    }

    fn put_resource_def(&mut self, address: Address, resource_def: ResourceDef) {
        self.db.put(address.to_vec(), Self::encode(&resource_def)).unwrap()
    }

    fn get_package(&self, address: Address) -> Option<Package> {
        self.db.get(address.to_vec()).unwrap().map(Self::decode)
    }

    fn put_package(&mut self, address: Address, package: Package) {
        self.db.put(address.to_vec(), Self::encode(&package)).unwrap()
    }

    fn get_component(&self, address: Address) -> Option<Component> {
        self.db.get(address.to_vec()).unwrap().map(Self::decode)
    }

    fn put_component(&mut self, address: Address, component: Component) {
        self.db.put(address.to_vec(), Self::encode(&component)).unwrap()
    }

    fn get_lazy_map(&self, mid: Mid) -> Option<LazyMap> {
        self.db.get(mid.to_vec()).unwrap().map(Self::decode)
    }

    fn put_lazy_map(&mut self, mid: Mid, lazy_map: LazyMap) {
        self.db.put(mid.to_vec(), Self::encode(&lazy_map)).unwrap()
    }

    fn get_vault(&self, vid: Vid) -> Option<Vault> {
        self.db.get(vid.to_vec()).unwrap().map(Self::decode)
    }

    fn put_vault(&mut self, vid: Vid, vault: Vault) {
        self.db.put(vid.to_vec(), Self::encode(&vault)).unwrap()
    }

    fn get_nft(&self, resource_address: Address, key: &NftKey) -> Option<Nft> {
        let mut nft_id = resource_address.to_vec();
        nft_id.append(&mut key.to_vec());
        self.db.get(nft_id.to_vec()).unwrap().map(Self::decode)
    }

    fn put_nft(&mut self, resource_address: Address, key: &NftKey, nft: Nft) {
        let mut nft_id = resource_address.to_vec();
        nft_id.append(&mut key.to_vec());
        self.db.put(nft_id.to_vec(), Self::encode(&nft)).unwrap()
    }
}
