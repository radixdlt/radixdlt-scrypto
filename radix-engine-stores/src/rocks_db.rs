use itertools::Itertools;
use radix_engine_store_interface::interface::*;
pub use rocksdb::{BlockBasedOptions, LogLevel, Options};
use rocksdb::{
    ColumnFamily, ColumnFamilyDescriptor, DBWithThreadMode, Direction, IteratorMode,
    SingleThreaded, DB,
};
use sbor::rust::prelude::*;
use std::path::PathBuf;
use utils::copy_u8_array;

pub struct RocksdbSubstateStore {
    db: DBWithThreadMode<SingleThreaded>,
}

impl RocksdbSubstateStore {
    // Techincally we don't need CFs here at all; however, delete range API is only available for CF
    const THE_ONLY_CF: &str = "the_only";

    pub fn standard(root: PathBuf) -> Self {
        Self::with_options(&Options::default(), root)
    }
    pub fn with_options(options: &Options, root: PathBuf) -> Self {
        let mut options = options.clone();
        options.create_missing_column_families(true);
        let db = DB::open_cf_descriptors(
            &options,
            root.as_path(),
            vec![ColumnFamilyDescriptor::new(
                Self::THE_ONLY_CF,
                Options::default(),
            )],
        )
        .unwrap();
        Self { db }
    }

    fn cf(&self) -> &ColumnFamily {
        self.db.cf_handle(Self::THE_ONLY_CF).unwrap()
    }
}

impl SubstateDatabase for RocksdbSubstateStore {
    fn get_substate(
        &self,
        partition_key: &DbPartitionKey,
        sort_key: &DbSortKey,
    ) -> Option<DbSubstateValue> {
        let key_bytes = encode_to_rocksdb_bytes(partition_key, sort_key);
        self.db.get_cf(self.cf(), &key_bytes).expect("IO Error")
    }

    fn list_entries(
        &self,
        partition_key: &DbPartitionKey,
    ) -> Box<dyn Iterator<Item = PartitionEntry> + '_> {
        let partition_key = partition_key.clone();
        let start_key_bytes = encode_to_rocksdb_bytes(&partition_key, &DbSortKey(vec![]));
        let iter = self
            .db
            .iterator_cf(
                self.cf(),
                IteratorMode::From(&start_key_bytes, Direction::Forward),
            )
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
        for (node_key, node_updates) in &database_updates.node_updates {
            for (partition_num, partition_updates) in &node_updates.partition_updates {
                let partition_key = DbPartitionKey {
                    node_key: node_key.clone(),
                    partition_num: *partition_num,
                };
                match partition_updates {
                    PartitionDatabaseUpdates::Delta { substate_updates } => {
                        for (sort_key, update) in substate_updates {
                            let key_bytes = encode_to_rocksdb_bytes(&partition_key, sort_key);
                            match update {
                                DatabaseUpdate::Set(value_bytes) => {
                                    self.db.put_cf(self.cf(), key_bytes, value_bytes)
                                }
                                DatabaseUpdate::Delete => self.db.delete_cf(self.cf(), key_bytes),
                            }
                            .expect("IO error");
                        }
                    }
                    PartitionDatabaseUpdates::Batch(batch) => {
                        match batch {
                            BatchPartitionDatabaseUpdate::Reset {
                                new_substate_values,
                            } => {
                                // Note: a plain `delete_range()` is missing from rocksdb's API, and
                                // (at the moment of writing) this is the only reason of having CF.
                                self.db
                                    .delete_range_cf(
                                        self.cf(),
                                        encode_to_rocksdb_bytes(&partition_key, &DbSortKey(vec![])),
                                        encode_to_rocksdb_bytes(
                                            &partition_key.next(),
                                            &DbSortKey(vec![]),
                                        ),
                                    )
                                    .expect("IO error");
                                for (sort_key, value_bytes) in new_substate_values {
                                    let key_bytes =
                                        encode_to_rocksdb_bytes(&partition_key, sort_key);
                                    self.db
                                        .put_cf(self.cf(), key_bytes, value_bytes)
                                        .expect("IO error");
                                }
                            }
                        }
                    }
                }
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

pub fn encode_to_rocksdb_bytes(partition_key: &DbPartitionKey, sort_key: &DbSortKey) -> Vec<u8> {
    let mut buffer = Vec::new();
    buffer.extend(
        u32::try_from(partition_key.node_key.len())
            .unwrap()
            .to_be_bytes(),
    );
    buffer.extend(partition_key.node_key.clone());
    buffer.push(partition_key.partition_num);
    buffer.extend(sort_key.0.clone());
    buffer
}

pub fn decode_from_rocksdb_bytes(buffer: &[u8]) -> DbSubstateKey {
    let partition_key_len =
        usize::try_from(u32::from_be_bytes(copy_u8_array(&buffer[..4]))).unwrap();
    let partition_byte_offset = 4 + partition_key_len;
    let partition_key = DbPartitionKey {
        node_key: buffer[4..partition_byte_offset].to_vec(),
        partition_num: buffer[partition_byte_offset],
    };
    let sort_key = DbSortKey(buffer[partition_byte_offset + 1..].to_vec());
    (partition_key, sort_key)
}
