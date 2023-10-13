use crate::hash_tree::tree_store::{
    encode_key, NodeKey, ReadableTreeStore, StaleTreePart, TreeNode, TreeNodeV1, VersionedTreeNode,
};
use itertools::Itertools;
use radix_engine_common::data::scrypto::{scrypto_decode, scrypto_encode};
use radix_engine_common::prelude::Hash;
use radix_engine_derive::ScryptoSbor;
use radix_engine_store_interface::interface::*;
pub use rocksdb::{BlockBasedOptions, LogLevel, Options};
use rocksdb::{
    ColumnFamily, ColumnFamilyDescriptor, DBWithThreadMode, Direction, IteratorMode,
    SingleThreaded, WriteBatch, DB,
};
use sbor::prelude::*;
use std::path::PathBuf;

mod state_tree;
use crate::rocks_db::{decode_from_rocksdb_bytes, encode_to_rocksdb_bytes};
use state_tree::*;

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
}

impl SubstateDatabase for RocksDBWithMerkleTreeSubstateStore {
    fn get_substate(
        &self,
        partition_key: &DbPartitionKey,
        sort_key: &DbSortKey,
    ) -> Option<DbSubstateValue> {
        let key_bytes = encode_to_rocksdb_bytes(partition_key, sort_key);
        self.db
            .get_cf(self.cf(SUBSTATES_CF), &key_bytes)
            .expect("IO Error")
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
                                encode_to_rocksdb_bytes(&partition_key.next(), &DbSortKey(vec![])),
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
        let (state_hash_tree_update, new_root_hash) =
            compute_state_tree_update(self, parent_state_version, database_updates);
        for (key, node) in state_hash_tree_update.new_nodes {
            batch.put_cf(
                self.cf(MERKLE_NODES_CF),
                encode_key(&key),
                scrypto_encode(&VersionedTreeNode::new_latest(node)).unwrap(),
            );
        }
        if !self.pruning_enabled {
            // If pruning is not enabled, we store the stale nodes in DB.
            batch.put_cf(
                self.cf(STALE_MERKLE_TREE_PARTS_CF),
                next_state_version.to_be_bytes(),
                scrypto_encode(&state_hash_tree_update.stale_tree_parts).unwrap(),
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
            for part in state_hash_tree_update.stale_tree_parts {
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
                                match value.into_latest() {
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

impl ReadableTreeStore for RocksDBWithMerkleTreeSubstateStore {
    fn get_node(&self, key: &NodeKey) -> Option<TreeNode> {
        self.db
            .get_cf(self.cf(MERKLE_NODES_CF), &encode_key(key))
            .unwrap()
            .map(|bytes| scrypto_decode::<VersionedTreeNode>(&bytes).unwrap())
            .map(|versioned| versioned.into_latest())
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, ScryptoSbor)]
struct Metadata {
    current_state_version: u64,
    current_state_root_hash: Hash,
}
