use core::iter;

use super::jellyfish::JellyfishMerkleTree;
use super::jellyfish::TreeUpdateBatch;
use super::tree_store::*;
use super::types::*;
use radix_common::crypto::Hash;
use radix_rust::prelude::*;
use radix_rust::rust::ops::Deref;
use radix_substate_store_interface::interface::DbNodeKey;

// Rename for this file to avoid confusion with TreeNodes!
pub(crate) type DbEntityKey = DbNodeKey;

pub const TIER_SEPARATOR: u8 = b'_';

pub trait StoredNode {
    type Payload;

    fn into_jmt_node(&self, key: &TreeNodeKey) -> Node<Self::Payload>;
    fn from_jmt_node(node: &Node<Self::Payload>, key: &TreeNodeKey) -> Self;
}

pub trait StateTreeTier {
    type TypedLeafKey;
    type StoredNode: StoredNode<Payload = Self::Payload>;
    type Payload: Clone;

    fn to_leaf_key(typed_key: &Self::TypedLeafKey) -> LeafKey;
    fn to_typed_key(leaf_key: LeafKey) -> Self::TypedLeafKey;
    fn root_version(&self) -> Option<Version>;
}

pub trait ReadableTier: StateTreeTier {
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

pub struct TierLeaf<T: StateTreeTier> {
    pub key: T::TypedLeafKey,
    pub value_hash: Hash,
    pub payload: T::Payload,

    /// A local tree node key of the leaf (i.e. expressed within tier [`T`]).
    ///
    /// Note: this is a somewhat leaky low-level detail, not needed by most use-cases. However, it
    /// is cheap to capture, and allows to resolve the actual (i.e. global) [`StoredTreeNodeKey`]
    /// for cases which need to directly reference the state tree storage - see e.g. the
    /// [`SubstateSummary::state_tree_leaf_key`].
    pub local_key: TreeNodeKey,
}

impl<T: StateTreeTier<Payload = Version>> TierLeaf<T> {
    pub fn new(local_key: TreeNodeKey, leaf: TreeLeafNode) -> Self {
        let TreeLeafNode {
            key_suffix,
            value_hash,
            last_hash_change_version,
        } = leaf;
        let full_key = NibblePath::from_iter(
            local_key
                .nibble_path()
                .nibbles()
                .chain(key_suffix.nibbles()),
        );
        Self {
            key: T::to_typed_key(LeafKey::new(full_key.bytes())),
            value_hash,
            payload: last_hash_change_version,
            local_key,
        }
    }
}

/// Returns a lexicographically-sorted iterator of all the leaves existing at the given `tier`'s
/// current version and greater or equal to the given `from_key`.
pub fn iter_leaves_from<'t, T>(
    tier: impl Deref<Target = T> + Clone + 't,
    from_key: Option<&T::TypedLeafKey>,
) -> Box<dyn Iterator<Item = TierLeaf<T>> + 't>
where
    T: ReadableTier<StoredNode = TreeNode, Payload = Version> + 't,
    T::TypedLeafKey: 't,
{
    tier.root_version()
        .map(|version| {
            recurse_until_leaves(
                tier,
                TreeNodeKey::new_empty_path(version),
                from_key
                    .map(|from| T::to_leaf_key(from).into_path().nibbles().collect())
                    .unwrap_or_else(|| VecDeque::new()),
            )
        })
        .unwrap_or_else(|| Box::new(iter::empty()))
}

/// Returns a lexicographically-sorted iterator of all the leaves located below the `at_key` node
/// and having [`NibblePath`]s greater or equal to the given `from_nibbles`.
///
/// The algorithm:
/// - starts at the given `at_key`,
/// - then goes down the tree, guided by the given `from_nibbles` chain, for as long as it is
///   possible,
///   - Note: this means it will either locate exactly this nibble path, or - if it does not
///     exist - settle at its direct successor.
/// - and then continues as if it was a classic DFS all the way,
/// - but only leaf nodes are returned.
///
/// The implementation is a lazy recursive composite of child iterators.
pub fn recurse_until_leaves<'t, T>(
    tier: impl Deref<Target = T> + Clone + 't,
    at_key: TreeNodeKey,
    from_nibbles: VecDeque<Nibble>,
) -> Box<dyn Iterator<Item = TierLeaf<T>> + 't>
where
    T: ReadableTier<StoredNode = TreeNode, Payload = Version> + 't,
    T::TypedLeafKey: 't,
{
    let Some(node) = tier.get_local_node(&at_key) else {
        panic!("{:?} referenced but not found in the storage", at_key);
    };
    match node {
        TreeNode::Internal(internal) => {
            let mut child_from_nibbles = from_nibbles;
            let from_nibble = child_from_nibbles.pop_front();
            Box::new(
                internal
                    .children
                    .into_iter()
                    .filter(move |child| Some(child.nibble) >= from_nibble)
                    .flat_map(move |child| {
                        let child_key = at_key.gen_child_node_key(child.version, child.nibble);
                        let child_from_nibbles = if Some(child.nibble) == from_nibble {
                            mem::take(&mut child_from_nibbles)
                        } else {
                            VecDeque::new()
                        };
                        recurse_until_leaves(tier.clone(), child_key, child_from_nibbles)
                    }),
            )
        }
        TreeNode::Leaf(leaf) => Box::new(
            Some(leaf)
                .filter(move |leaf| leaf.key_suffix.nibbles().ge(from_nibbles))
                .map(|leaf| TierLeaf::new(at_key, leaf))
                .into_iter(),
        ),
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

pub trait WriteableTier: StateTreeTier {
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

pub trait RwTier: ReadableTier + WriteableTier {
    fn generate_tier_update_batch<'a>(
        &self,
        new_version: Version,
        leaf_updates: impl Iterator<Item = (&'a Self::TypedLeafKey, Option<(Hash, Self::Payload)>)>,
    ) -> TierUpdateBatch<Self::Payload>
    where
        <Self as StateTreeTier>::TypedLeafKey: 'a,
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

impl<T: ReadableTier + WriteableTier> RwTier for T {}
