use super::substate_tier::*;
use super::tier_framework::*;
use super::tree_store::*;
use super::types::*;
use radix_engine_common::crypto::Hash;
use substate_store_interface::interface::NodeDatabaseUpdates;
use substate_store_interface::interface::*;
use utils::prelude::*;

/// The middle tier of the 3-tier JMT.
pub struct PartitionTier<'s, S> {
    base_store: &'s S,
    root_version: Option<Version>,
    entity_key: DbEntityKey,
    tree_node_prefix: Vec<u8>,
}

impl<'s, S> Tier for PartitionTier<'s, S> {
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
    fn resolve_substate_tier(&self, partition: DbPartitionNum) -> SubstateTier<'s, S> {
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
}

impl<'s, S: ReadableTreeStore + WriteableTreeStore> PartitionTier<'s, S> {
    pub(crate) fn put_entity_partition_updates(
        &self,
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
                        .put_partition_substate_updates(next_version, partition_database_updates);
                    let new_leaf = new_partition_root_hash.map(|hash| {
                        // In order to be able to resolve the new root of the child tree,
                        //  we set the new leaf payload to be the version at which it was updated.
                        let new_leaf_payload = next_version;
                        (hash, new_leaf_payload)
                    });
                    (partition, new_leaf)
                });
        let (new_root_hash, _) =
            self.apply_leaf_updates(BaseTree::Existing, next_version, leaf_updates);
        new_root_hash
    }
}
