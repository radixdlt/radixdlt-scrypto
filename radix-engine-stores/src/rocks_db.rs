use itertools::Itertools;
use radix_engine_store_interface::interface::*;
pub use rocksdb::{BlockBasedOptions, LogLevel, Options};
use rocksdb::{DBWithThreadMode, Direction, IteratorMode, SingleThreaded, DB};
use sbor::rust::prelude::*;
use std::path::PathBuf;
use utils::copy_u8_array;

pub struct RocksdbSubstateStore {
    db: DBWithThreadMode<SingleThreaded>,
}

impl RocksdbSubstateStore {
    pub fn standard(root: PathBuf) -> Self {
        let db = DB::open_default(root.as_path()).expect("IO Error");

        Self { db }
    }
    pub fn with_options(options: &Options, root: PathBuf) -> Self {
        let db = DB::open(options, root.as_path()).expect("IO Error");

        Self { db }
    }
}

impl SubstateDatabase for RocksdbSubstateStore {
    fn get_substate(
        &self,
        partition_key: &DbPartitionKey,
        sort_key: &DbSortKey,
    ) -> Option<DbSubstateValue> {
        let key_bytes = encode_to_rocksdb_bytes(partition_key, sort_key);
        self.db.get(&key_bytes).expect("IO Error")
    }

    fn list_entries(
        &self,
        partition_key: &DbPartitionKey,
    ) -> Box<dyn Iterator<Item = PartitionEntry> + '_> {
        let partition_key = partition_key.clone();
        let start_key_bytes = encode_to_rocksdb_bytes(&partition_key, &DbSortKey(vec![]));
        let iter = self
            .db
            .iterator(IteratorMode::From(&start_key_bytes, Direction::Forward))
            .map(|kv| {
                let (iter_key_bytes, iter_value) = kv.as_ref().unwrap();
                let iter_key = decode_from_rocksdb_bytes(iter_key_bytes);
                (iter_key, iter_value.to_vec())
            })
            .take_while(move |((iter_partition_key, _), _)| *iter_partition_key == partition_key)
            .map(|((_, iter_sort_key), iter_value)| (iter_sort_key, iter_value.to_vec()));

        Box::new(iter)
    }
}

impl CommittableSubstateDatabase for RocksdbSubstateStore {
    fn commit(&mut self, database_updates: &DatabaseUpdates) {
        for (patrition_key, partition_updates) in database_updates {
            for (sort_key, database_update) in partition_updates {
                let key_bytes = encode_to_rocksdb_bytes(patrition_key, sort_key);
                let result = match database_update {
                    DatabaseUpdate::Set(value_bytes) => self.db.put(key_bytes, value_bytes),
                    DatabaseUpdate::Delete => self.db.delete(key_bytes),
                };
                result.expect("IO error");
            }
        }
    }
}

impl ListableSubstateDatabase for RocksdbSubstateStore {
    fn list_partition_keys(&self) -> Box<dyn Iterator<Item = DbPartitionKey> + '_> {
        Box::new(
            self.db
                .iterator(IteratorMode::Start)
                .map(|kv| {
                    let (iter_key_bytes, _) = kv.as_ref().unwrap();
                    let (iter_key, _) = decode_from_rocksdb_bytes(iter_key_bytes);
                    iter_key
                })
                // Rocksdb iterator returns sorted entries, so ok to to eliminate
                // duplicates with dedup()
                .dedup(),
        )
    }
}

fn encode_to_rocksdb_bytes(partition_key: &DbPartitionKey, sort_key: &DbSortKey) -> Vec<u8> {
    let mut buffer = Vec::new();
    buffer.extend(u32::try_from(partition_key.0.len()).unwrap().to_be_bytes());
    buffer.extend(partition_key.0.clone());
    buffer.extend(sort_key.0.clone());
    buffer
}

fn decode_from_rocksdb_bytes(buffer: &[u8]) -> DbSubstateKey {
    let partition_key_len =
        usize::try_from(u32::from_be_bytes(copy_u8_array(&buffer[..4]))).unwrap();
    let sort_key_offset = 4 + partition_key_len;
    let partition_key = DbPartitionKey(buffer[4..sort_key_offset].to_vec());
    let sort_key = DbSortKey(buffer[sort_key_offset..].to_vec());
    (partition_key, sort_key)
}
