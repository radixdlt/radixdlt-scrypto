use core::iter;

use super::jellyfish::JellyfishMerkleTree;
use super::jellyfish::TreeUpdateBatch;
use super::tree_store::*;
use super::types::*;
use radix_engine_common::crypto::Hash;
use substate_store_interface::interface::DbNodeKey;
use utils::prelude::*;

// Rename for this file to avoid confusion with TreeNodes!
pub(crate) type DbEntityKey = DbNodeKey;

pub const TIER_SEPARATOR: u8 = b'_';

pub trait StoredNode {
    type Payload;

    fn into_jmt_node(&self, key: &TreeNodeKey) -> Node<Self::Payload>;
    fn from_jmt_node(node: &Node<Self::Payload>, key: &TreeNodeKey) -> Self;
}

pub trait TierView {
    type TypedLeafKey;
    type StoredNode: StoredNode<Payload = Self::Payload>;
    type Payload: Clone;

    fn to_leaf_key(typed_key: &Self::TypedLeafKey) -> LeafKey;
    fn to_typed_key(leaf_key: LeafKey) -> Self::TypedLeafKey;
    fn root_version(&self) -> Option<Version>;
}

pub trait ReadableTier: TierView {
    /// Gets node by key, if it exists.
    fn get_local_node(&self, local_key: &TreeNodeKey) -> Option<Self::StoredNode>;

    fn jmt(&self) -> JellyfishMerkleTree<Self, Self::Payload> {
        JellyfishMerkleTree::new(self)
    }

    fn get_persisted_leaf_payload(&self, key: &Self::TypedLeafKey) -> Option<Self::Payload> {
        let Some(root_version) = self.root_version() else {
            return None;
        };

        let leaf_key = Self::to_leaf_key(&key);

        let (leaf_node_data, _proof) = self.jmt().get_with_proof(&leaf_key, root_version).unwrap();
        leaf_node_data.map(|(_hash, payload, _version)| payload)
    }
}

pub trait IterableLeaves: TierView {
    fn iter_leaves(
        &self,
    ) -> Box<dyn Iterator<Item = (Hash, Self::TypedLeafKey, Self::Payload)> + '_>;
}

impl<T: ReadableTier<StoredNode = TreeNode, Payload = Version>> IterableLeaves for T {
    fn iter_leaves(
        &self,
    ) -> Box<dyn Iterator<Item = (Hash, Self::TypedLeafKey, Self::Payload)> + '_> {
        match self.root_version() {
            Some(version) => iter_leaves_internal(self, TreeNodeKey::new_empty_path(version)),
            None => Box::new(iter::empty()),
        }
    }
}

fn iter_leaves_internal<T>(
    tier: &T,
    from_key: TreeNodeKey,
) -> Box<dyn Iterator<Item = (Hash, T::TypedLeafKey, T::Payload)> + '_>
where
    T: ReadableTier<StoredNode = TreeNode, Payload = Version>,
{
    let Some(node) = tier.get_local_node(&from_key) else {
        panic!("{:?} referenced but not found in the storage", from_key);
    };
    match node {
        TreeNode::Internal(internal) => {
            Box::new(internal.children.into_iter().flat_map(move |child| {
                iter_leaves_internal(
                    tier,
                    from_key.gen_child_node_key(child.version, child.nibble),
                )
            }))
        }
        TreeNode::Leaf(leaf) => {
            let leaf_node = LeafNode::from(&from_key, &leaf);
            let (leaf_key, value_hash, payload, _version) = leaf_node.into();
            Box::new(iter::once((value_hash, T::to_typed_key(leaf_key), payload)))
        }
        TreeNode::Null => Box::new(iter::empty()),
    }
}

impl<R: ReadableTier + ?Sized> TreeReader<<R::StoredNode as StoredNode>::Payload> for R {
    fn get_node_option(
        &self,
        node_key: &TreeNodeKey,
    ) -> Result<Option<Node<<R::StoredNode as StoredNode>::Payload>>, StorageError> {
        Ok(self
            .get_local_node(node_key)
            .map(|tree_node| tree_node.into_jmt_node(&node_key)))
    }
}

pub trait WritableTier: TierView {
    /// Inserts the node under a new, unique key (i.e. never an update).
    fn insert_local_node(&self, local_key: &TreeNodeKey, node: Self::StoredNode);

    /// Marks the given tree part for a future removal by an arbitrary external pruning
    /// process.
    fn record_stale_local_node(&self, local_key: &TreeNodeKey);

    /// Sets the root version of the TierView
    fn set_root_version(&mut self, new_version: Option<Version>);
}

pub struct TierUpdateBatch<P> {
    pub new_version: Version,
    pub new_root_hash: Option<Hash>,
    pub tree_update_batch: TreeUpdateBatch<P>,
}

pub trait ReadWritableTier: ReadableTier + WritableTier {
    fn generate_tier_update_batch<'a>(
        &self,
        new_version: Version,
        leaf_updates: impl Iterator<Item = (&'a Self::TypedLeafKey, Option<(Hash, Self::Payload)>)>,
    ) -> TierUpdateBatch<Self::Payload>
    where
        <Self as TierView>::TypedLeafKey: 'a,
    {
        let value_set = leaf_updates
            .map(|(key, option)| (Self::to_leaf_key(&key), option))
            .collect();
        let (root_hash, update_batch) = self
            .jmt()
            .batch_put_value_set(value_set, None, self.root_version(), new_version)
            .expect("error while reading tree during put");

        let root_hash = if root_hash == SPARSE_MERKLE_PLACEHOLDER_HASH {
            None
        } else {
            Some(root_hash)
        };

        TierUpdateBatch {
            new_version,
            tree_update_batch: update_batch,
            new_root_hash: root_hash,
        }
    }

    fn apply_tier_update_batch(&mut self, tier_update_batch: &TierUpdateBatch<Self::Payload>) {
        let TierUpdateBatch {
            new_version,
            tree_update_batch: update_batch,
            new_root_hash: _,
        } = tier_update_batch;
        for (key, node) in update_batch.node_batch.iter().flatten() {
            self.insert_local_node(key, Self::StoredNode::from_jmt_node(node, &key));
        }
        for stale_node in update_batch.stale_node_index_batch.iter().flatten() {
            self.record_stale_local_node(&stale_node.node_key);
        }

        self.set_root_version(Some(*new_version));
    }
}

impl<T: ReadableTier + WritableTier> ReadWritableTier for T {}
