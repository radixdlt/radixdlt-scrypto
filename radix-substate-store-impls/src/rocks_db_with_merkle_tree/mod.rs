use crate::state_tree::tree_store::*;
use itertools::Itertools;
use radix_common::constants::MAX_SUBSTATE_KEY_SIZE;
use radix_common::prelude::*;
use radix_substate_store_interface::interface::*;
pub use rocksdb::{BlockBasedOptions, LogLevel, Options};
use rocksdb::{
    ColumnFamily, ColumnFamilyDescriptor, DBWithThreadMode, Direction, IteratorMode,
    SingleThreaded, WriteBatch, DB,
};
use std::path::PathBuf;

mod state_tree;
use crate::rocks_db::{decode_from_rocksdb_bytes, encode_to_rocksdb_bytes};
pub use state_tree::*;

const META_CF: &str = "meta";
const SUBSTATES_CF: &str = "substates";
const MERKLE_NODES_CF: &str = "merkle_nodes";
const STALE_MERKLE_TREE_PARTS_CF: &str = "stale_merkle_tree_parts";

pub struct RocksDBWithMerkleTreeSubstateStore {
    db: DBWithThreadMode<SingleThreaded>,
    pruning_enabled: bool,
}

impl RocksDBWithMerkleTreeSubstateStore {
    pub fn clear(root: PathBuf) -> Self {
        if root.exists() {
            std::fs::remove_dir_all(&root).unwrap();
        } else {
            std::fs::create_dir_all(&root).unwrap();
        }
        Self::standard(root)
    }

    pub fn standard(root: PathBuf) -> Self {
        let mut options = Options::default();
        options.create_if_missing(true);
        options.create_missing_column_families(true);
        Self::with_options(&options, root, true)
    }

    pub fn with_options(options: &Options, root: PathBuf, pruning_enabled: bool) -> Self {
        let db = DB::open_cf_descriptors(
            options,
            root.as_path(),
            [
                META_CF,
                SUBSTATES_CF,
                MERKLE_NODES_CF,
                STALE_MERKLE_TREE_PARTS_CF,
            ]
            .into_iter()
            .map(|name| ColumnFamilyDescriptor::new(name, Options::default()))
            .collect::<Vec<_>>(),
        )
        .unwrap();
        Self {
            db,
            pruning_enabled,
        }
    }

    fn cf(&self, cf: &str) -> &ColumnFamily {
        self.db.cf_handle(cf).unwrap()
    }

    pub fn get_current_version(&self) -> u64 {
        self.db
            .get_cf(self.cf(META_CF), &[])
            .unwrap()
            .map(|bytes| {
                scrypto_decode::<Metadata>(&bytes)
                    .unwrap()
                    .current_state_version
            })
            .unwrap_or(0)
    }

    pub fn get_current_root_hash(&self) -> Hash {
        self.db
            .get_cf(self.cf(META_CF), &[])
            .unwrap()
            .map(|bytes| {
                scrypto_decode::<Metadata>(&bytes)
                    .unwrap()
                    .current_state_root_hash
            })
            .unwrap_or(Hash([0u8; Hash::LENGTH]))
    }

    pub fn overwrite_metadata(&mut self, meta: &Metadata) {
        self.db
            .put_cf(self.cf(META_CF), &[], scrypto_encode(meta).unwrap())
            .unwrap();
    }
}

impl SubstateDatabase for RocksDBWithMerkleTreeSubstateStore {
    fn get_raw_substate_by_db_key(
        &self,
        partition_key: &DbPartitionKey,
        sort_key: &DbSortKey,
    ) -> Option<DbSubstateValue> {
        let key_bytes = encode_to_rocksdb_bytes(partition_key, sort_key);
        self.db
            .get_cf(self.cf(SUBSTATES_CF), &key_bytes)
            .expect("IO Error")
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
                self.cf(SUBSTATES_CF),
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

impl CommittableSubstateDatabase for RocksDBWithMerkleTreeSubstateStore {
    fn commit(&mut self, database_updates: &DatabaseUpdates) {
        // read required info about current database state (here I fake it a bit)
        let metadata = self
            .db
            .get_cf(self.cf(META_CF), [])
            .unwrap()
            .map(|bytes| scrypto_decode::<Metadata>(&bytes).unwrap())
            .unwrap_or_else(|| Metadata {
                current_state_version: 0,
                current_state_root_hash: Hash([0u8; Hash::LENGTH]),
            });
        let parent_state_version = metadata.current_state_version;
        let next_state_version = parent_state_version + 1;

        // prepare a batch write (we use the same approach in the actual Node)
        let mut batch = WriteBatch::default();

        // put regular substate changes
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
                                    self.db
                                        .put_cf(self.cf(SUBSTATES_CF), key_bytes, value_bytes)
                                }
                                DatabaseUpdate::Delete => {
                                    self.db.delete_cf(self.cf(SUBSTATES_CF), key_bytes)
                                }
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
                                self.cf(SUBSTATES_CF),
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
                                .put_cf(self.cf(SUBSTATES_CF), key_bytes, value_bytes)
                                .expect("IO error");
                        }
                    }
                }
            }
        }

        // derive and put new JMT nodes (also record references to stale parts, for later amortized background GC [not implemented here!])
        let (state_tree_diff, new_root_hash) =
            compute_state_tree_update(self, parent_state_version, database_updates);
        for (key, node) in state_tree_diff.new_nodes.take() {
            batch.put_cf(
                self.cf(MERKLE_NODES_CF),
                encode_key(&key),
                scrypto_encode(&VersionedTreeNode::from_latest_version(node)).unwrap(),
            );
        }
        if !self.pruning_enabled {
            // If pruning is not enabled, we store the stale nodes in DB.
            batch.put_cf(
                self.cf(STALE_MERKLE_TREE_PARTS_CF),
                next_state_version.to_be_bytes(),
                scrypto_encode(&state_tree_diff.stale_tree_parts).unwrap(),
            );
        }

        // update the metadata
        batch.put_cf(
            self.cf(META_CF),
            [],
            scrypto_encode(&Metadata {
                current_state_version: next_state_version,
                current_state_root_hash: new_root_hash,
            })
            .unwrap(),
        );

        // flush the batch
        self.db.write(batch).unwrap();

        if self.pruning_enabled {
            for part in state_tree_diff.stale_tree_parts.take() {
                match part {
                    StaleTreePart::Node(node_key) => {
                        self.db
                            .delete_cf(self.cf(MERKLE_NODES_CF), encode_key(&node_key))
                            .unwrap();
                    }
                    StaleTreePart::Subtree(node_key) => {
                        let mut queue = VecDeque::new();
                        queue.push_back(node_key);

                        while let Some(node_key) = queue.pop_front() {
                            if let Some(bytes) = self
                                .db
                                .get_cf(self.cf(MERKLE_NODES_CF), encode_key(&node_key))
                                .unwrap()
                            {
                                self.db
                                    .delete_cf(self.cf(MERKLE_NODES_CF), encode_key(&node_key))
                                    .unwrap();
                                let value: VersionedTreeNode = scrypto_decode(&bytes).unwrap();
                                match value.fully_update_and_into_latest_version() {
                                    TreeNodeV1::Internal(x) => {
                                        for child in x.children {
                                            queue.push_back(
                                                node_key.gen_child_node_key(
                                                    child.version,
                                                    child.nibble,
                                                ),
                                            )
                                        }
                                    }
                                    TreeNodeV1::Leaf(_) => {}
                                    TreeNodeV1::Null => {}
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

impl ListableSubstateDatabase for RocksDBWithMerkleTreeSubstateStore {
    fn list_partition_keys(&self) -> Box<dyn Iterator<Item = DbPartitionKey> + '_> {
        Box::new(
            self.db
                .iterator_cf(self.cf(SUBSTATES_CF), IteratorMode::Start)
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

impl ReadableTreeStore for RocksDBWithMerkleTreeSubstateStore {
    fn get_node(&self, key: &StoredTreeNodeKey) -> Option<TreeNode> {
        self.db
            .get_cf(self.cf(MERKLE_NODES_CF), &encode_key(key))
            .unwrap()
            .map(|bytes| scrypto_decode::<VersionedTreeNode>(&bytes).unwrap())
            .map(|versioned| versioned.fully_update_and_into_latest_version())
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, ScryptoSbor)]
pub struct Metadata {
    pub current_state_version: u64,
    pub current_state_root_hash: Hash,
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
        let mut db = RocksDBWithMerkleTreeSubstateStore::standard(temp_dir.into_path());

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
