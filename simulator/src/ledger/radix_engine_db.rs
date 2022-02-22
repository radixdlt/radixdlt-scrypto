use std::collections::HashMap;
use std::path::PathBuf;

use radix_engine::ledger::*;
use rocksdb::{DBWithThreadMode, Direction, IteratorMode, SingleThreaded, DB};
use sbor::{Decode, Encode};
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

    fn read(&self, key: &[u8]) -> Option<Vec<u8>> {
        // TODO: Use get_pinned
        self.db.get(key).unwrap()
    }

    fn write(&self, key: &[u8], value: &[u8]) {
        self.db.put(key, value).unwrap();
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
        let key_size = id.len();

        let mut iter = self
            .db
            .iterator(IteratorMode::From(&id, Direction::Forward));
        let mut items = HashMap::new();
        while let Some((key, value)) = iter.next() {
            if !key.starts_with(&id) {
                break;
            }

            let local_key = key.split_at(key_size).1.to_vec();
            items.insert(local_key, value.to_vec());
        }
        items
    }
}

impl SubstateStore for RadixEngineDB {
    fn get_substate<T: Encode>(&self, address: &T) -> Option<Substate> {
        self.read(&scrypto_encode(address))
            .map(|b| {
                let (phys_id, value) = b.split_at(8);
                Substate {
                    value: value.to_vec(),
                    phys_id: u64::from_le_bytes(phys_id.try_into().unwrap())
                }
            })
    }

    fn put_substate<T: Encode>(&mut self, address: &T, substate: Substate) {
        let mut value = substate.phys_id.to_le_bytes().to_vec();
        value.extend(substate.value);

        self.write(&scrypto_encode(address), &value);
    }

    fn get_child_substate<T: Encode>(&self, address: &T, key: &[u8]) -> Option<Vec<u8>> {
        let mut id = scrypto_encode(address);
        id.extend(key.to_vec());
        self.read(&id)
    }

    fn put_child_substate<T: Encode>(&mut self, address: &T, key: &[u8], substate: &[u8]) {
        let mut id = scrypto_encode(address);
        id.extend(key.to_vec());
        self.write(&id, substate);
    }

    fn get_epoch(&self) -> u64 {
        let id = scrypto_encode(&"epoch");
        self.read(&id)
            .map(|v| scrypto_decode(&v).unwrap())
            .unwrap_or(0)
    }

    fn set_epoch(&mut self, epoch: u64) {
        let id = scrypto_encode(&"epoch");
        let value = scrypto_encode(&epoch);
        self.write(&id, &value)
    }

    fn get_nonce(&self) -> u64 {
        let id = scrypto_encode(&"nonce");
        self.read(&id)
            .map(|v| scrypto_decode(&v).unwrap())
            .unwrap_or(0)
    }

    fn increase_nonce(&mut self) {
        let id = scrypto_encode(&"nonce");
        let value = scrypto_encode(&(self.get_nonce() + 1));
        self.write(&id, &value)
    }
}
