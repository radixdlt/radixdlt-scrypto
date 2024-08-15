use super::jellyfish::TreeUpdateBatch;
use super::tier_framework::*;
use super::tree_store::*;
use super::types::*;
use radix_common::prelude::*;
use radix_substate_store_interface::interface::*;

/// The bottom tier of the 3-tier JMT, corresponding to the `DbSortKey` part of a substate key.
///
/// Its leaf keys are `DbSortKeys` (an ordered key for substates under a partition).
///
/// Its leaves have:
/// * Value Hash: The `blake2b_256_hash` of the substate value
/// * Payload: The state version when the substate value was set
pub struct SubstateTier<'s, S> {
    base_store: &'s S,
    root_version: Option<Version>,
    partition_key: DbPartitionKey,
    tree_node_prefix: Vec<u8>,
}

impl<'s, S> SubstateTier<'s, S> {
    pub fn new(
        base_store: &'s S,
        root_version: Option<Version>,
        entity_key: DbEntityKey,
        partition: DbPartitionNum,
    ) -> Self {
        let mut tree_node_prefix = Vec::with_capacity(entity_key.len() + 3);
        tree_node_prefix.extend_from_slice(&entity_key);
        tree_node_prefix.push(TIER_SEPARATOR);
        tree_node_prefix.push(partition);
        tree_node_prefix.push(TIER_SEPARATOR);

        Self {
            base_store,
            root_version,
            partition_key: DbPartitionKey {
                node_key: entity_key,
                partition_num: partition,
            },
            tree_node_prefix,
        }
    }

    pub fn partition_key(&self) -> &DbPartitionKey {
        &self.partition_key
    }

    fn stored_node_key(&self, local_key: &TreeNodeKey) -> StoredTreeNodeKey {
        StoredTreeNodeKey::prefixed(&self.tree_node_prefix, local_key)
    }
}

impl<'s, S> StateTreeTier for SubstateTier<'s, S> {
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

impl<'s, S: WriteableTreeStore> WriteableTier for SubstateTier<'s, S> {
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

impl<'s, S: ReadableTreeStore> SubstateTier<'s, S> {
    pub fn get_substate_summary(&self, sort_key: &DbSortKey) -> Option<SubstateSummary> {
        // Performance note:
        // When reading from a tree-based store, getting a leaf has the same cost as starting an
        // iterator and taking its first element. The only possible savings would be available in
        // the "not found" case, which is rare in our use-cases.
        // Hence, for simplicity, we prefer to re-use the single (non-trivial) leaf-locating code.
        self.iter_substate_summaries_from(Some(sort_key))
            .next()
            .filter(|least_ge_summary| &least_ge_summary.sort_key == sort_key)
    }

    pub fn iter_substate_summaries_from(
        &self,
        from: Option<&DbSortKey>,
    ) -> impl Iterator<Item = SubstateSummary> + '_ {
        iter_leaves_from(self, from).map(self.create_summary_mapper())
    }

    pub fn into_iter_substate_summaries_from(
        self,
        from: Option<&DbSortKey>,
    ) -> impl Iterator<Item = SubstateSummary> + 's {
        let summary_mapper = self.create_summary_mapper(); // we soon lose `self`
        iter_leaves_from(Rc::new(self), from).map(summary_mapper)
    }

    fn create_summary_mapper(&self) -> impl FnMut(TierLeaf<Self>) -> SubstateSummary {
        let tree_node_prefix = self.tree_node_prefix.clone();
        move |leaf| SubstateSummary {
            sort_key: leaf.key,
            upsert_version: leaf.payload,
            value_hash: leaf.value_hash,
            state_tree_leaf_key: StoredTreeNodeKey::prefixed(&tree_node_prefix, &leaf.local_key),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SubstateSummary {
    pub sort_key: DbSortKey,
    pub upsert_version: Version,
    pub value_hash: Hash,

    /// A global tree node key of this Substate's leaf.
    ///
    /// Note: this is a low-level detail, needed e.g. to properly correlate the Substate value
    /// stored by [`WriteableTreeStore::associate_substate()`].
    pub state_tree_leaf_key: StoredTreeNodeKey,
}

impl<'s, S: ReadableTreeStore + WriteableTreeStore> SubstateTier<'s, S> {
    pub fn apply_partition_updates(
        &mut self,
        next_version: Version,
        updates: &PartitionDatabaseUpdates,
    ) -> Option<Hash> {
        let leaf_updates: Box<dyn Iterator<Item = _>> = match updates {
            PartitionDatabaseUpdates::Delta { substate_updates } => {
                Box::new(substate_updates.iter().map(|(sort_key, update)| {
                    let new_leaf = match update {
                        DatabaseUpdate::Set(value) => Some(Self::new_leaf(value, next_version)),
                        DatabaseUpdate::Delete => None,
                    };
                    (sort_key, new_leaf)
                }))
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

                Box::new(
                    new_substate_values
                        .iter()
                        .map(|(sort_key, new_substate_value)| {
                            let new_leaf = Some(Self::new_leaf(new_substate_value, next_version));
                            (sort_key, new_leaf)
                        }),
                )
            }
        };

        let tier_update_batch = self.generate_tier_update_batch(next_version, leaf_updates);
        self.apply_tier_update_batch(&tier_update_batch);
        self.associate_substates(updates, &tier_update_batch.tree_update_batch);

        tier_update_batch.new_root_hash
    }

    fn new_leaf(
        new_substate_value: &DbSubstateValue,
        new_version: Version,
    ) -> (Hash, <Self as StateTreeTier>::Payload) {
        let value_hash = hash(new_substate_value);
        // We set a payload of the version for consistency with the leaves of other tiers.
        let new_leaf_payload = new_version;
        (value_hash, new_leaf_payload)
    }

    fn associate_substates(
        &self,
        substate_updates: &PartitionDatabaseUpdates,
        tree_update_batch: &TreeUpdateBatch<Version>,
    ) {
        for (key, node) in tree_update_batch.node_batch.iter().flatten() {
            // We promised to associate Substate values; but not all newly-created nodes are leaves:
            let Node::Leaf(leaf_node) = &node else {
                continue;
            };
            let sort_key = Self::to_typed_key(leaf_node.leaf_key().clone());
            let substate_value = substate_updates
                .get_substate_change(&sort_key)
                .map(|change| match change {
                    DatabaseUpdateRef::Set(value) => AssociatedSubstateValue::Upserted(value),
                    DatabaseUpdateRef::Delete => {
                        panic!("deletes are not represented by new tree leafs")
                    }
                })
                .unwrap_or_else(|| AssociatedSubstateValue::Unchanged);
            self.base_store.associate_substate(
                &self.stored_node_key(&key),
                &self.partition_key,
                &sort_key,
                substate_value,
            );
        }
    }
}
