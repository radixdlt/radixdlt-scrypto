use super::partition_tier::*;
use super::tier_framework::*;
use super::tree_store::*;
use super::types::*;
use radix_engine_common::crypto::Hash;
use substate_store_interface::interface::DatabaseUpdates;
use utils::prelude::*;

/// The top tier of the 3-tier JMT.
/// Also known as the "Node Tier", but to avoid confusion with TreeNodes, we use the term Entity instead.
pub struct EntityTier<'s, S> {
    base_store: &'s S,
    root_version: Option<Version>,
}

impl<'s, S> EntityTier<'s, S> {
    pub fn new(base_store: &'s S, root_version: Option<Version>) -> Self {
        Self {
            base_store,
            root_version,
        }
    }

    fn stored_node_key(&self, local_key: &TreeNodeKey) -> StoredTreeNodeKey {
        StoredTreeNodeKey::unprefixed(local_key.clone())
    }
}

impl<'s, S: ReadableTreeStore> EntityTier<'s, S> {
    fn resolve_partition_tier(&self, entity_key: DbEntityKey) -> PartitionTier<'s, S> {
        let entity_root_version = self.get_persisted_leaf_payload(&entity_key);
        PartitionTier::new(&self.base_store, entity_root_version, entity_key)
    }
}

impl<'s, S> TierView for EntityTier<'s, S> {
    type TypedLeafKey = DbEntityKey;
    type StoredNode = TreeNode;
    type Payload = Version;

    fn to_leaf_key(entity_key: &DbEntityKey) -> LeafKey {
        LeafKey::new(entity_key)
    }

    fn to_typed_key(leaf_key: LeafKey) -> Self::TypedLeafKey {
        leaf_key.bytes
    }

    fn root_version(&self) -> Option<Version> {
        self.root_version
    }
}

impl<'s, S: ReadableTreeStore> ReadableTier for EntityTier<'s, S> {
    fn get_local_node(&self, local_key: &TreeNodeKey) -> Option<TreeNode> {
        // No prefixing needed in top layer
        self.base_store.get_node(&self.stored_node_key(local_key))
    }
}

impl<'s, S: WriteableTreeStore> WritableTier for EntityTier<'s, S> {
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

impl<'s, S: ReadableTreeStore + WriteableTreeStore> EntityTier<'s, S> {
    pub fn put_all_entity_updates(
        &mut self,
        next_version: Version,
        updates: &DatabaseUpdates,
    ) -> Option<Hash> {
        let leaf_updates =
            updates
                .node_updates
                .iter()
                .map(|(entity_key, entity_database_updates)| {
                    let new_partition_root_hash = self
                        .resolve_partition_tier(entity_key.clone())
                        .put_entity_partition_updates(next_version, entity_database_updates);
                    let new_leaf = new_partition_root_hash.map(|hash| {
                        // In order to be able to resolve the new root of the child tree,
                        //  we set the new leaf payload to be the version at which it was updated.
                        let new_leaf_payload = next_version;
                        (hash, new_leaf_payload)
                    });
                    (entity_key, new_leaf)
                });
        let update_batch = self.generate_tier_update_batch(next_version, leaf_updates);
        self.apply_tier_update_batch(&update_batch);
        update_batch.new_root_hash
    }
}
