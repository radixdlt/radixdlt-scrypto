use super::partition_tier::*;
use super::tier_framework::*;
use super::tree_store::*;
use super::types::*;
use radix_common::crypto::Hash;
use radix_rust::prelude::*;
use radix_substate_store_interface::interface::DatabaseUpdates;

/// The top tier of the 3-tier JMT, corresponding to the `DbNodeKey` (aka `DbEntityKey`) part of a substate key.
/// We use the synonym "Entity" rather than "Node" to avoid confusion with TreeNodes.
///
/// Its leaf keys are `DbEntityKey` (a hash of the ReNodeId, to promote spread leaves for a performant JMT).
///
/// Its leaves have:
/// * Value Hash: The entity root hash of the corresponding nested entity tree in the `PartitionTier`
/// * Payload: The state version of the root of the corresponding nested entity tree in the `PartitionTier`
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
    pub fn iter_entity_partition_tiers_from(
        &self,
        from: Option<&DbEntityKey>,
    ) -> impl Iterator<Item = PartitionTier<'s, S>> + '_ {
        iter_leaves_from(self, from).map(self.create_partition_tier_mapper())
    }

    pub fn into_iter_entity_partition_tiers_from(
        self,
        from: Option<&DbEntityKey>,
    ) -> impl Iterator<Item = PartitionTier<'s, S>> + 's {
        let partition_tier_mapper = self.create_partition_tier_mapper(); // we soon lose `self`
        iter_leaves_from(Rc::new(self), from).map(partition_tier_mapper)
    }

    pub fn get_entity_partition_tier(&self, entity_key: DbEntityKey) -> PartitionTier<'s, S> {
        let entity_root_version = self.get_persisted_leaf_payload(&entity_key);
        PartitionTier::new(self.base_store, entity_root_version, entity_key)
    }

    fn create_partition_tier_mapper(&self) -> impl FnMut(TierLeaf<Self>) -> PartitionTier<'s, S> {
        let base_store = self.base_store; // Note: This avoids capturing the `_ lifetime below.
        move |leaf| PartitionTier::new(base_store, Some(leaf.payload), leaf.key)
    }
}

impl<'s, S> StateTreeTier for EntityTier<'s, S> {
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

impl<'s, S: WriteableTreeStore> WriteableTier for EntityTier<'s, S> {
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
    pub fn put_next_version_entity_updates(&mut self, updates: &DatabaseUpdates) -> Option<Hash> {
        self.put_entity_updates(self.root_version.unwrap_or(0) + 1, updates)
    }

    pub fn put_entity_updates(
        &mut self,
        next_version: Version,
        updates: &DatabaseUpdates,
    ) -> Option<Hash> {
        let leaf_updates =
            updates
                .node_updates
                .iter()
                .map(|(entity_key, entity_database_updates)| {
                    let new_entity_root_hash = self
                        .get_entity_partition_tier(entity_key.clone())
                        .apply_entity_updates(next_version, entity_database_updates);
                    let new_leaf = new_entity_root_hash.map(|new_entity_root_hash| {
                        let new_leaf_hash = new_entity_root_hash;
                        let new_leaf_payload = next_version;
                        (new_leaf_hash, new_leaf_payload)
                    });
                    (entity_key, new_leaf)
                });
        let update_batch = self.generate_tier_update_batch(next_version, leaf_updates);
        self.apply_tier_update_batch(&update_batch);
        update_batch.new_root_hash
    }
}
