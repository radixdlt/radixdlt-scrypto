use itertools::Itertools;
use radix_common::constants::MAX_SUBSTATE_KEY_SIZE;
use radix_common::prelude::*;
use radix_rust::copy_u8_array;
use radix_substate_store_interface::interface::*;
pub use rocksdb::{BlockBasedOptions, LogLevel, Options};
use rocksdb::{
    ColumnFamily, ColumnFamilyDescriptor, DBWithThreadMode, Direction, IteratorMode,
    SingleThreaded, DB,
};
use std::path::PathBuf;

pub struct RocksdbSubstateStore {
    db: DBWithThreadMode<SingleThreaded>,
}

impl RocksdbSubstateStore {
    // Technically we don't need CFs here at all; however, delete range API is only available for CF
    const THE_ONLY_CF: &'static str = "the_only";

    pub fn standard(root: PathBuf) -> Self {
        Self::with_options(&Options::default(), root)
    }
    pub fn with_options(options: &Options, root: PathBuf) -> Self {
        let mut options = options.clone();
        options.create_if_missing(true);
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
    fn get_raw_substate_by_db_key(
        &self,
        partition_key: &DbPartitionKey,
        sort_key: &DbSortKey,
    ) -> Option<DbSubstateValue> {
        let key_bytes = encode_to_rocksdb_bytes(partition_key, sort_key);
        self.db.get_cf(self.cf(), &key_bytes).expect("IO Error")
    }

    fn list_raw_values_from_db_key(
        &self,
        partition_key: &DbPartitionKey,
        from_sort_key: Option<&DbSortKey>,
    ) -> Box<dyn Iterator<Item = PartitionEntry> + '_> {
        let partition_key = partition_key.clone();
        let empty_sort_key = DbSortKey(vec![]);
        let from_sort_key = from_sort_key.unwrap_or(&empty_sort_key);
        let start_key_bytes = encode_to_rocksdb_bytes(&partition_key, from_sort_key);
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
                    PartitionDatabaseUpdates::Reset {
                        new_substate_values,
                    } => {
                        // Note: a plain `delete_range()` is missing from rocksdb's API, and
                        // (at the moment of writing) this is the only reason of having CF.
                        self.db
                            .delete_range_cf(
                                self.cf(),
                                encode_to_rocksdb_bytes(&partition_key, &DbSortKey(vec![])),
                                encode_to_rocksdb_bytes(
                                    &partition_key,
                                    &DbSortKey(vec![u8::MAX; 2 * MAX_SUBSTATE_KEY_SIZE]),
                                ),
                            )
                            .expect("IO error");
                        for (sort_key, value_bytes) in new_substate_values {
                            let key_bytes = encode_to_rocksdb_bytes(&partition_key, sort_key);
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

impl ListableSubstateDatabase for RocksdbSubstateStore {
    fn list_partition_keys(&self) -> Box<dyn Iterator<Item = DbPartitionKey> + '_> {
        Box::new(
            self.db
                .iterator_cf(self.cf(), IteratorMode::Start)
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

#[cfg(test)]
mod tests {
    use super::*;
    use radix_substate_store_interface::interface::{
        CommittableSubstateDatabase, DatabaseUpdates, DbSortKey, NodeDatabaseUpdates,
        PartitionDatabaseUpdates,
    };

    #[cfg(not(feature = "alloc"))]
    #[test]
    fn test_partition_deletion() {
        let temp_dir = tempfile::tempdir().unwrap();
        let mut db = RocksdbSubstateStore::standard(temp_dir.into_path());

        let node_updates = NodeDatabaseUpdates {
            partition_updates: indexmap! {
                0 => PartitionDatabaseUpdates::Reset {
                    new_substate_values: indexmap! {
                        DbSortKey(vec![5]) => vec![6]
                    }
                },
                1 => PartitionDatabaseUpdates::Reset {
                    new_substate_values: indexmap! {
                        DbSortKey(vec![7]) => vec![8]
                    }
                },
                255 => PartitionDatabaseUpdates::Reset {
                    new_substate_values: indexmap! {
                        DbSortKey(vec![9]) => vec![10]
                    }
                }
            },
        };
        let updates = DatabaseUpdates {
            node_updates: indexmap! {
                vec![0] => node_updates.clone(),
                vec![1] => node_updates.clone(),
                vec![255] => node_updates.clone(),
            },
        };
        db.commit(&updates);

        assert_eq!(db.list_partition_keys().count(), 9);
        db.commit(&DatabaseUpdates {
            node_updates: indexmap! {
                vec![0] => NodeDatabaseUpdates {
                    partition_updates: indexmap!{
                        255 => PartitionDatabaseUpdates::Reset { new_substate_values: indexmap!{} }
                    }
                }
            },
        });
        assert_eq!(db.list_partition_keys().count(), 8);
    }
}
