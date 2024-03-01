use super::substate_tier::*;
use super::tier_framework::*;
use super::tree_store::*;
use super::types::*;
use radix_engine_common::crypto::Hash;
use substate_store_interface::interface::NodeDatabaseUpdates;
use substate_store_interface::interface::*;
use utils::prelude::*;

/// The middle tier of the 3-tier JMT, corresponding to the partition part of a substate key.
///
/// Its leaf keys are partition numbers (a single byte, two nibbles).
///
/// Its leaves have:
///   * Value Hash: The partition root hashes of the nested tree for a partition in the SubstateTier
///   * Payload: The state version of the root of the partition in the Substate Tier
pub struct PartitionTier<'s, S> {
    base_store: &'s S,
    root_version: Option<Version>,
    entity_key: DbEntityKey,
    tree_node_prefix: Vec<u8>,
}

impl<'s, S> TierView for PartitionTier<'s, S> {
    type TypedLeafKey = DbPartitionNum;
    type StoredNode = TreeNode;
    type Payload = Version;

    fn to_leaf_key(partition: &DbPartitionNum) -> LeafKey {
        LeafKey::new(&[*partition])
    }

    fn to_typed_key(leaf_key: LeafKey) -> DbPartitionNum {
        leaf_key.bytes[0]
    }

    fn root_version(&self) -> Option<Version> {
        self.root_version
    }
}

impl<'s, S> PartitionTier<'s, S> {
    pub fn new(base_store: &'s S, root_version: Option<Version>, entity_key: DbEntityKey) -> Self {
        let mut tree_node_prefix = Vec::with_capacity(entity_key.len() + 1);
        tree_node_prefix.extend_from_slice(&entity_key);
        tree_node_prefix.push(TIER_SEPARATOR);

        Self {
            base_store,
            root_version,
            entity_key,
            tree_node_prefix,
        }
    }

    pub fn entity_key(&self) -> &DbEntityKey {
        &self.entity_key
    }

    fn stored_node_key(&self, local_key: &TreeNodeKey) -> StoredTreeNodeKey {
        StoredTreeNodeKey::prefixed(&self.tree_node_prefix, local_key)
    }
}

impl<'s, S: ReadableTreeStore> PartitionTier<'s, S> {
    fn resolve_substate_tier(&'_ self, partition: DbPartitionNum) -> SubstateTier<'s, S> {
        let partition_root_version = self.get_persisted_leaf_payload(&partition);
        SubstateTier::new(
            &self.base_store,
            partition_root_version,
            self.entity_key.clone(),
            partition,
        )
    }
}

impl<'s, S: ReadableTreeStore> ReadableTier for PartitionTier<'s, S> {
    fn get_local_node(&self, local_key: &TreeNodeKey) -> Option<TreeNode> {
        self.base_store.get_node(&self.stored_node_key(local_key))
    }
}

impl<'s, S: WriteableTreeStore> WritableTier for PartitionTier<'s, S> {
    fn insert_local_node(&self, local_key: &TreeNodeKey, node: Self::StoredNode) {
        self.base_store
            .insert_node(self.stored_node_key(local_key), node);
    }

    fn record_stale_local_node(&self, local_key: &TreeNodeKey) {
        self.base_store
            .record_stale_tree_part(StaleTreePart::Node(self.stored_node_key(local_key)))
    }

    fn set_root_version(&mut self, new_version: Option<Version>) {
        self.root_version = new_version;
    }
}

impl<'s, S: ReadableTreeStore + WriteableTreeStore> PartitionTier<'s, S> {
    pub(crate) fn apply_entity_updates(
        &mut self,
        next_version: Version,
        updates: &NodeDatabaseUpdates,
    ) -> Option<Hash> {
        let leaf_updates =
            updates
                .partition_updates
                .iter()
                .map(|(partition, partition_database_updates)| {
                    let new_partition_root_hash = self
                        .resolve_substate_tier(*partition)
                        .apply_partition_updates(next_version, partition_database_updates);
                    let new_leaf = new_partition_root_hash.map(|new_partition_root_hash| {
                        let new_leaf_hash = new_partition_root_hash;
                        let new_leaf_payload = next_version;
                        (new_leaf_hash, new_leaf_payload)
                    });
                    (partition, new_leaf)
                });
        let update_batch = self.generate_tier_update_batch(next_version, leaf_updates);
        self.apply_tier_update_batch(&update_batch);
        update_batch.new_root_hash
    }
}
