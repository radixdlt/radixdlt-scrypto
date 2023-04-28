use crate::utils::{decode_substate_id, encode_substate_id};
use radix_engine_store_interface::interface::{
    CommittableSubstateDatabase, DatabaseUpdate, DatabaseUpdates, SubstateDatabase,
};
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
}

impl SubstateDatabase for RocksdbSubstateStore {
    fn get_substate(&self, index_id: &Vec<u8>, db_key: &Vec<u8>) -> Option<Vec<u8>> {
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
    fn commit(&mut self, database_updates: &DatabaseUpdates) {
        for (index_id, index_updates) in database_updates {
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
