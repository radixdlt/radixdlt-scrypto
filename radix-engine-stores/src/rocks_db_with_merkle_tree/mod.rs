//use super::compute_state_tree_update;
use crate::hash_tree::tree_store::{encode_key, NodeKey, Payload, ReadableTreeStore, TreeNode};
use itertools::Itertools;
use radix_engine_common::data::scrypto::{scrypto_decode, scrypto_encode};
use radix_engine_derive::ScryptoSbor;
use radix_engine_store_interface::interface::*;
pub use rocksdb::{BlockBasedOptions, LogLevel, Options};
use rocksdb::{
    ColumnFamily, ColumnFamilyDescriptor, DBWithThreadMode, Direction, IteratorMode,
    SingleThreaded, WriteBatch, DB,
};
use sbor::rust::prelude::*;
use std::path::PathBuf;
use utils::copy_u8_array;
mod state_tree;
use state_tree::*;

const META_CF: &str = "meta";
const SUBSTATES_CF: &str = "substates";
const MERKLE_NODES_CF: &str = "merkle_nodes";
const STALE_MERKLE_NODE_KEYS_CF: &str = "stale_merkle_node_keys";

pub struct RocksDBWithMerkleTreeSubstateStore {
    db: DBWithThreadMode<SingleThreaded>,
}

impl RocksDBWithMerkleTreeSubstateStore {
    pub fn standard(root: PathBuf) -> Self {
        let mut options = Options::default();
        options.create_if_missing(true);
        options.create_missing_column_families(true);
        Self::with_options(&options, root)
    }

    pub fn with_options(options: &Options, root: PathBuf) -> Self {
        let db = DB::open_cf_descriptors(
            options,
            root.as_path(),
            [
                META_CF,
                SUBSTATES_CF,
                MERKLE_NODES_CF,
                STALE_MERKLE_NODE_KEYS_CF,
            ]
            .into_iter()
            .map(|name| ColumnFamilyDescriptor::new(name, Options::default()))
            .collect::<Vec<_>>(),
        )
        .unwrap();
        Self { db }
    }

    fn cf(&self, cf: &str) -> &ColumnFamily {
        self.db.cf_handle(cf).unwrap()
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
            });
        let parent_state_version = metadata.current_state_version;
        let next_state_version = parent_state_version + 1;

        // prepare a batch write (we use the same approach in the actual Node)
        let mut batch = WriteBatch::default();

        // put regular substate changes
        for (patrition_key, partition_updates) in database_updates {
            for (sort_key, database_update) in partition_updates {
                let key_bytes = encode_to_rocksdb_bytes(patrition_key, sort_key);
                match database_update {
                    DatabaseUpdate::Set(value_bytes) => {
                        batch.put_cf(self.cf(SUBSTATES_CF), key_bytes, value_bytes)
                    }
                    DatabaseUpdate::Delete => batch.delete_cf(self.cf(SUBSTATES_CF), key_bytes),
                };
            }
        }

        // derive and put new JMT nodes (also record keys of stale nodes, for later amortized background GC [not implemented here!])
        let state_hash_tree_update =
            compute_state_tree_update(self, parent_state_version, database_updates);
        for (key, node) in state_hash_tree_update.new_re_node_layer_nodes {
            batch.put_cf(
                self.cf(MERKLE_NODES_CF),
                encode_key(&key),
                scrypto_encode(&node).unwrap(),
            );
        }
        for (key, node) in state_hash_tree_update.new_substate_layer_nodes {
            batch.put_cf(
                self.cf(MERKLE_NODES_CF),
                encode_key(&key),
                scrypto_encode(&node).unwrap(),
            );
        }
        let encoded_node_keys = state_hash_tree_update
            .stale_hash_tree_node_keys
            .iter()
            .map(encode_key)
            .collect::<Vec<_>>();
        batch.put_cf(
            self.cf(STALE_MERKLE_NODE_KEYS_CF),
            next_state_version.to_be_bytes(),
            scrypto_encode(&encoded_node_keys).unwrap(),
        );

        // update the metadata
        batch.put_cf(
            self.cf(META_CF),
            [],
            scrypto_encode(&Metadata {
                current_state_version: next_state_version,
            })
            .unwrap(),
        );

        // flush the batch
        self.db.write(batch).unwrap();
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

impl<P: Payload> ReadableTreeStore<P> for RocksDBWithMerkleTreeSubstateStore {
    fn get_node(&self, key: &NodeKey) -> Option<TreeNode<P>> {
        self.db
            .get_cf(self.cf(MERKLE_NODES_CF), &encode_key(key))
            .unwrap()
            .map(|bytes| scrypto_decode(&bytes).unwrap())
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

#[derive(Debug, Clone, Hash, PartialEq, Eq, ScryptoSbor)]
struct Metadata {
    current_state_version: u64,
}
