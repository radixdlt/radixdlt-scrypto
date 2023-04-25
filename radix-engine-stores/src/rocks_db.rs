use crate::interface::*;
use radix_engine_interface::types::*;
use rocksdb::{DBWithThreadMode, Direction, IteratorMode, SingleThreaded, DB};
use sbor::rust::prelude::*;
use std::path::PathBuf;

pub struct RocksdbSubstateStore {
    db: DBWithThreadMode<SingleThreaded>,
}

impl RocksdbSubstateStore {
    pub fn standard(root: PathBuf) -> Self {
        let db = DB::open_default(root.as_path()).expect("IO Error");

        Self { db }
    }

    // TODO: Is this still important?
    /*
    pub fn list_nodes(&self) -> Vec<NodeId> {
        let mut items = Vec::new();
        let mut iter = self
            .db
            .iterator(IteratorMode::From(&[], Direction::Forward));
        while let Some(kv) = iter.next() {
            let (key, _value) = kv.unwrap();
            if key.len() < NodeId::LENGTH {
                continue;
            }
            let (index_id, _) = decode_substate_id(key.as_ref()).unwrap();
            let node_id = NodeId(index_id[0..NodeId::LENGTH].to_vec().try_into().unwrap());
            if items.last() != Some(&node_id) {
                items.push(node_id);
            }
        }
        items
    }

    pub fn list_packages(&self) -> Vec<PackageAddress> {
        self.list_nodes()
            .into_iter()
            .filter_map(|x| PackageAddress::try_from(x.as_ref()).ok())
            .collect()
    }

    pub fn list_components(&self) -> Vec<ComponentAddress> {
        self.list_nodes()
            .into_iter()
            .filter_map(|x| ComponentAddress::try_from(x.as_ref()).ok())
            .collect()
    }

    pub fn list_resource_managers(&self) -> Vec<ResourceAddress> {
        self.list_nodes()
            .into_iter()
            .filter_map(|x| ResourceAddress::try_from(x.as_ref()).ok())
            .collect()
    }
     */
}

impl SubstateDatabase for RocksdbSubstateStore {
    fn get_substate(
        &self,
        index_id: &Vec<u8>,
        db_key: &Vec<u8>,
    ) -> Option<Vec<u8>> {
        let key = encode_substate_id(index_id, db_key);
        self.db.get(&key).expect("IO Error")
    }

    fn list_substates(
        &self,
        index_id: &Vec<u8>,
    ) -> Box<dyn Iterator<Item = (Vec<u8>, Vec<u8>)> + '_> {
        let index_id = index_id.clone();

        let start = encode_substate_id(&index_id, &vec![0]);

        let iter = self
            .db
            .iterator(IteratorMode::From(&start, Direction::Forward))
            .take_while(move |kv| {
                let (key, _value) = kv.as_ref().unwrap();
                key[0..26].eq(&index_id)
            })
            .map(|kv| {
                let (key, value) = kv.unwrap();
                let (_, substate_key) =
                    decode_substate_id(key.as_ref()).expect("Failed to decode substate ID");
                let value = value.as_ref().to_vec();
                (substate_key, value)
            });

        Box::new(iter)
    }
}

impl CommittableSubstateDatabase for RocksdbSubstateStore {
    fn commit(&mut self, state_changes: &DatabaseUpdates) {
        for (index_id, index_updates) in &state_changes.database_updates
        {
            for (db_key, update) in index_updates {
                let substate_id = encode_substate_id(index_id, db_key);
                match update {
                    DatabaseUpdate::Set(substate_value) => {
                        self.db.put(substate_id, substate_value).expect("IO error");
                    }
                    DatabaseUpdate::Delete => {
                        self.db.delete(substate_id).expect("IO error");
                    }
                }
            }
        }
    }
}
