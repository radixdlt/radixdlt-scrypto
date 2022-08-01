use std::collections::HashMap;
use std::path::PathBuf;

use radix_engine::engine::Substate;
use radix_engine::engine::SubstateId;
use radix_engine::ledger::*;
use rocksdb::{DBWithThreadMode, Direction, IteratorMode, SingleThreaded, DB};
use sbor::Decode;
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
        let substate_store = Self::new(root);
        bootstrap(substate_store)
    }

    pub fn list_packages(&self) -> Vec<PackageAddress> {
        let start = &scrypto_encode(&PackageAddress([0; 27]));
        let end = &scrypto_encode(&PackageAddress([255; 27]));
        self.list_items(start, end)
    }

    pub fn list_components(&self) -> Vec<ComponentAddress> {
        let start = &scrypto_encode(&ComponentAddress([0; 27]));
        let end = &scrypto_encode(&ComponentAddress([255; 27]));
        self.list_items(start, end)
    }

    pub fn list_resource_managers(&self) -> Vec<ResourceAddress> {
        let start = &scrypto_encode(&ResourceAddress([0; 27]));
        let end = &scrypto_encode(&ResourceAddress([255; 27]));
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
            if kv.0.len() == start.len() {
                items.push(scrypto_decode(kv.0.as_ref()).unwrap());
            }
        }
        items
    }

    fn read(&self, substate_id: &SubstateId) -> Option<Vec<u8>> {
        // TODO: Use get_pinned
        self.db.get(scrypto_encode(substate_id)).unwrap()
    }

    fn write(&self, substate_id: SubstateId, value: Vec<u8>) {
        self.db.put(scrypto_encode(&substate_id), value).unwrap();
    }
}

impl QueryableSubstateStore for RadixEngineDB {
    fn get_kv_store_entries(
        &self,
        component_address: ComponentAddress,
        kv_store_id: &KeyValueStoreId,
    ) -> HashMap<Vec<u8>, Substate> {
        let mut id = scrypto_encode(&component_address);
        id.extend(scrypto_encode(kv_store_id));
        let key_size = id.len();

        let mut iter = self
            .db
            .iterator(IteratorMode::From(&id, Direction::Forward));
        iter.next(); // Key Value Store
        let mut items = HashMap::new();
        while let Some((key, value)) = iter.next() {
            if !key.starts_with(&id) {
                break;
            }

            let local_key = key.split_at(key_size).1.to_vec();
            let substate: OutputValue = scrypto_decode(&value.to_vec()).unwrap();
            items.insert(local_key, substate.substate);
        }
        items
    }
}

impl ReadableSubstateStore for RadixEngineDB {
    fn get_substate(&self, substate_id: &SubstateId) -> Option<OutputValue> {
        self.read(substate_id).map(|b| scrypto_decode(&b).unwrap())
    }
}

impl WriteableSubstateStore for RadixEngineDB {
    fn put_substate(&mut self, substate_id: SubstateId, substate: OutputValue) {
        self.write(substate_id, scrypto_encode(&substate));
    }
}
