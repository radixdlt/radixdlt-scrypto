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

pub trait Tier {
    type TypedLeafKey;
    type StoredNode: StoredNode<Payload = Self::Payload>;
    type Payload: Clone;

    fn to_leaf_key(typed_key: &Self::TypedLeafKey) -> LeafKey;
    fn to_typed_key(leaf_key: LeafKey) -> Self::TypedLeafKey;
    fn root_version(&self) -> Option<Version>;
}

pub trait ReadableTier: Tier {
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

pub trait IterableLeaves: Tier {
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

pub trait WritableTier: Tier {
    /// Inserts the node under a new, unique key (i.e. never an update).
    fn insert_local_node(&self, local_key: &TreeNodeKey, node: Self::StoredNode);

    /// Marks the given tree part for a future removal by an arbitrary external pruning
    /// process.
    fn record_stale_local_node(&self, local_key: &TreeNodeKey);
}

pub enum BaseTree {
    Existing,
    Empty,
}

pub trait ReadWritableTier: ReadableTier + WritableTier {
    fn apply_leaf_updates<'a>(
        &self,
        base_tree: BaseTree,
        next_version: Version,
        leaf_updates: impl Iterator<Item = (&'a Self::TypedLeafKey, Option<(Hash, Self::Payload)>)>,
    ) -> (Option<Hash>, TreeUpdateBatch<Self::Payload>)
    where
        <Self as Tier>::TypedLeafKey: 'a,
    {
        let value_set = leaf_updates
            .map(|(key, option)| (Self::to_leaf_key(&key), option))
            .collect();
        let (root_hash, update_result) = self
            .jmt()
            .batch_put_value_set(
                value_set,
                None,
                match base_tree {
                    BaseTree::Existing => self.root_version(),
                    BaseTree::Empty => None,
                },
                next_version,
            )
            .expect("error while reading tree during put");

        for (key, node) in update_result.node_batch.iter().flatten() {
            self.insert_local_node(key, Self::StoredNode::from_jmt_node(node, &key));
        }
        for stale_node in update_result.stale_node_index_batch.iter().flatten() {
            self.record_stale_local_node(&stale_node.node_key);
        }

        let root_hash = if root_hash == SPARSE_MERKLE_PLACEHOLDER_HASH {
            None
        } else {
            Some(root_hash)
        };

        (root_hash, update_result)
    }
}

impl<T: ReadableTier + WritableTier> ReadWritableTier for T {}
