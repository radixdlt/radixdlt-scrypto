use super::jellyfish::TreeUpdateBatch;
use super::tier_framework::*;
use super::tree_store::*;
use super::types::*;
use radix_engine_common::crypto::{hash, Hash};
use substate_store_interface::interface::*;
use utils::prelude::*;

/// The bottom tier of the 3-tier JMT, corresponding to the DbSortKey part of a substate key.
///
/// Its leaf keys are DbSortKeys (an ordered key for substates under a partition).
///
/// Its leaves have:
///   * Value Hash: The blake2b_256_hash of the substate value
///   * Payload: The state version when the value was set
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

impl<'s, S> TierView for SubstateTier<'s, S> {
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

    fn set_root_version(&mut self, new_version: Option<Version>) {
        self.root_version = new_version;
    }
}

impl<'s, S: ReadableTreeStore + WriteableTreeStore> SubstateTier<'s, S> {
    pub fn apply_partition_updates(
        &mut self,
        next_version: Version,
        updates: &PartitionDatabaseUpdates,
    ) -> Option<Hash> {
        let (leaf_updates, substate_value_map): (Box<dyn Iterator<Item = _>>, _) = match updates {
            PartitionDatabaseUpdates::Delta { substate_updates } => {
                let leaf_updates = substate_updates.iter().map(|(sort_key, update)| {
                    let new_leaf = match update {
                        DatabaseUpdate::Set(value) => Some(Self::new_leaf(value, next_version)),
                        DatabaseUpdate::Delete => None,
                    };
                    (sort_key, new_leaf)
                });

                let substate_value_map = substate_updates
                    .iter()
                    .filter_map(|(sort_key, update)| match update {
                        DatabaseUpdate::Set(value) => Some((Self::to_leaf_key(sort_key), value)),
                        DatabaseUpdate::Delete => None,
                    })
                    .collect();

                (Box::new(leaf_updates), substate_value_map)
            }
            PartitionDatabaseUpdates::Reset {
                new_substate_values,
            } => {
                // First we handle the reset by:
                // * Recording the stale subtree for cleanup
                // * Setting this tier's root version to None, so that when we generate an update batch, it's
                //   on an empty tree
                if let Some(substate_root_version) = self.root_version {
                    self.base_store
                        .record_stale_tree_part(StaleTreePart::Subtree(
                            self.stored_node_key(&TreeNodeKey::new_empty_path(
                                substate_root_version,
                            )),
                        ));
                }
                self.set_root_version(None);

                // Then we handle the substate sets similarly to above:
                let leaf_updates =
                    new_substate_values
                        .iter()
                        .map(|(sort_key, new_substate_value)| {
                            let new_leaf = Some(Self::new_leaf(new_substate_value, next_version));
                            (sort_key, new_leaf)
                        });

                let substate_value_map = new_substate_values
                    .iter()
                    .map(|(sort_key, new_substate_value)| {
                        (Self::to_leaf_key(sort_key), new_substate_value)
                    })
                    .collect();

                (Box::new(leaf_updates), substate_value_map)
            }
        };

        let tier_update_batch = self.generate_tier_update_batch(next_version, leaf_updates);

        self.apply_tier_update_batch(&tier_update_batch);
        self.associate_substate_values(substate_value_map, &tier_update_batch.tree_update_batch);

        tier_update_batch.new_root_hash
    }

    fn new_leaf(
        new_substate_value: &DbSubstateValue,
        new_version: Version,
    ) -> (Hash, <Self as TierView>::Payload) {
        let value_hash = hash(new_substate_value);
        // We set a payload of the version for consistency with the leaves of other tiers.
        let new_leaf_payload = new_version;
        (value_hash, new_leaf_payload)
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
