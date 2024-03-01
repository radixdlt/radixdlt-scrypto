use super::jellyfish::TreeUpdateBatch;
use super::tier_framework::*;
use super::tree_store::*;
use super::types::*;
use radix_engine_common::crypto::{hash, Hash};
use substate_store_interface::interface::*;
use utils::prelude::*;

/// The bottom tier of the 3-tier JMT.
pub struct SubstateTier<'s, S> {
    base_store: &'s S,
    root_version: Option<Version>,
    entity_key: DbEntityKey,
    partition: DbPartitionNum,
    tree_node_prefix: Vec<u8>,
}

impl<'s, S> SubstateTier<'s, S> {
    pub fn new(
        base_store: &'s S,
        root_version: Option<Version>,
        entity_key: DbEntityKey,
        partition: DbPartitionNum,
    ) -> Self {
        let mut tree_node_prefix = Vec::with_capacity(entity_key.len() + 1);
        tree_node_prefix.extend_from_slice(&entity_key);
        tree_node_prefix.push(TIER_SEPARATOR);
        tree_node_prefix.push(partition);
        tree_node_prefix.push(TIER_SEPARATOR);

        Self {
            base_store,
            root_version,
            entity_key,
            partition,
            tree_node_prefix,
        }
    }

    pub fn entity_key(&self) -> &DbEntityKey {
        &self.entity_key
    }

    pub fn partition(&self) -> DbPartitionNum {
        self.partition
    }

    fn stored_node_key(&self, local_key: &TreeNodeKey) -> StoredTreeNodeKey {
        StoredTreeNodeKey::prefixed(&self.tree_node_prefix, local_key)
    }
}

impl<'s, S> Tier for SubstateTier<'s, S> {
    type TypedLeafKey = DbSortKey;
    type StoredNode = TreeNode;
    type Payload = Version;

    fn to_leaf_key(sort_key: &DbSortKey) -> LeafKey {
        LeafKey::new(&sort_key.0)
    }

    fn to_typed_key(leaf_key: LeafKey) -> DbSortKey {
        DbSortKey(leaf_key.bytes)
    }

    fn root_version(&self) -> Option<Version> {
        self.root_version
    }
}

impl<'s, S: ReadableTreeStore> ReadableTier for SubstateTier<'s, S> {
    fn get_local_node(&self, local_key: &TreeNodeKey) -> Option<TreeNode> {
        self.base_store.get_node(&self.stored_node_key(local_key))
    }
}

impl<'s, S: WriteableTreeStore> WritableTier for SubstateTier<'s, S> {
    fn insert_local_node(&self, local_key: &TreeNodeKey, node: Self::StoredNode) {
        self.base_store
            .insert_node(self.stored_node_key(local_key), node);
    }

    fn record_stale_local_node(&self, local_key: &TreeNodeKey) {
        self.base_store
            .record_stale_tree_part(StaleTreePart::Node(self.stored_node_key(local_key)))
    }
}

impl<'s, S: ReadableTreeStore + WriteableTreeStore> SubstateTier<'s, S> {
    pub fn put_partition_substate_updates(
        &self,
        next_version: Version,
        updates: &PartitionDatabaseUpdates,
    ) -> Option<Hash> {
        match updates {
            PartitionDatabaseUpdates::Delta { substate_updates } => {
                let leaf_updates = substate_updates.iter().map(|(sort_key, update)| {
                    let value = match update {
                        DatabaseUpdate::Set(value) => Some(value),
                        DatabaseUpdate::Delete => None,
                    };
                    let new_leaf = value.map(|value| {
                        let value_hash = hash(value);
                        // We set a payload of the version for consistency with the leaves of other tiers.
                        let new_leaf_payload = next_version;
                        (value_hash, new_leaf_payload)
                    });
                    (sort_key, new_leaf)
                });

                let substate_value_map = substate_updates
                    .iter()
                    .filter_map(|(sort_key, update)| match update {
                        DatabaseUpdate::Set(value) => Some((Self::to_leaf_key(sort_key), value)),
                        DatabaseUpdate::Delete => None,
                    })
                    .collect();

                let (new_root_hash, update_batch) =
                    self.apply_leaf_updates(BaseTree::Existing, next_version, leaf_updates);

                self.associate_substate_values(substate_value_map, &update_batch);

                new_root_hash
            }
            PartitionDatabaseUpdates::Reset {
                new_substate_values,
            } => {
                // First we record the stale subtree for cleanup
                if let Some(substate_root_version) = self.root_version {
                    self.base_store
                        .record_stale_tree_part(StaleTreePart::Subtree(
                            self.stored_node_key(&TreeNodeKey::new_empty_path(
                                substate_root_version,
                            )),
                        ));
                }

                let leaf_updates =
                    new_substate_values
                        .iter()
                        .map(|(sort_key, new_substate_value)| {
                            let value_hash = hash(new_substate_value);
                            let new_leaf_payload = next_version;
                            let new_leaf = Some((value_hash, new_leaf_payload));
                            (sort_key, new_leaf)
                        });

                let substate_value_map = new_substate_values
                    .iter()
                    .map(|(sort_key, new_substate_value)| {
                        (Self::to_leaf_key(sort_key), new_substate_value)
                    })
                    .collect();

                // Then we apply updates on top of an empty base tree
                let (new_root_hash, update_batch) =
                    self.apply_leaf_updates(BaseTree::Empty, next_version, leaf_updates);

                self.associate_substate_values(substate_value_map, &update_batch);

                new_root_hash
            }
        }
    }

    fn associate_substate_values(
        &self,
        substate_value_map: HashMap<LeafKey, &DbSubstateValue>,
        update_batch: &TreeUpdateBatch<Version>,
    ) {
        for (key, node) in update_batch.node_batch.iter().flatten() {
            // We promised to associate Substate values; but not all newly-created nodes are leaves:
            if let Node::Leaf(leaf_node) = &node {
                // And not every newly-created leaf comes from a value change: (sometimes it is just a tree re-structuring!)
                if let Some(substate_value) = substate_value_map.get(leaf_node.leaf_key()) {
                    self.base_store
                        .associate_substate_value(&self.stored_node_key(&key), *substate_value);
                }
            }
        }
    }
}
