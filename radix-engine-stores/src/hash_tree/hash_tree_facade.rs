use std::collections::HashMap;
use std::io::Error;

use super::jellyfish::JellyfishMerkleTree;
use super::types::NibblePath;
use radix_engine_interface::api::types::SubstateId;
use radix_engine_interface::crypto::{hash, Hash};
use radix_engine_interface::data::scrypto_encode;

use super::tree_store::{
    ReadableTreeStore, TreeChildEntry, TreeInternalNode, TreeLeafNode, TreeNode, TreeStore,
};
use super::types::{Child, InternalNode, LeafNode, Nibble, Node, NodeKey, NodeType, TreeReader};

/// A top-level API for a hash-computing tree.
pub struct HashTree<'s, S: TreeStore> {
    /// Storage SPI.
    store: &'s mut S,
    /// Latest state version expected to be found in the storage.
    /// This is equal to a number of past [`HashTree::put_at_next_version()`]
    /// invocations (i.e. in practice, equal to a number of executed
    /// transactions that lead to a particular state of a substate store).
    /// This value can potentially be 0 (for an absolutely empty state).
    current_version: u64,
}

impl<'s, S: TreeStore> HashTree<'s, S> {
    /// A direct constructor.
    /// The root node of the given [`current_version`] (when non-0) is assumed
    /// to exist in the underlying storage.
    pub fn new(store: &'s mut S, current_version: u64) -> HashTree<'s, S> {
        HashTree {
            store,
            current_version,
        }
    }

    /// Inserts a new set of nodes at version [`HashTree::current_version`] + 1.
    /// This inserts a new leaf node for each given "change", together with an
    /// entire new "parent chain" leading from that leaf to a new root node
    /// (common for all of them).
    /// Each change may either create/update a substate's value (denoted by
    /// `Some(hash(scrypto_encode(value)))`), or delete a substate (denoted by
    /// `None`).
    /// All nodes that became stale precisely due to this (i.e. not previous)
    /// operation will be repoerted before the function returns (see
    /// [`tree_store::WriteableTreeStore::record_stale_node`]).
    /// Returns the hash of the newly-created root (i.e. representing state at
    /// version [`HashTree::current_version`] + 1).
    ///
    /// # Panics
    /// Panics if a root node for [`HashTree::current_version`] does not exist.
    pub fn put_at_next_version(&mut self, changes: &[(SubstateId, Option<Hash>)]) -> Hash {
        let value_set: Vec<(Hash, Option<(Hash, SubstateId)>)> = changes
            .iter()
            .map(|(id, value_hash)| {
                (
                    hash(scrypto_encode(id).unwrap()),
                    value_hash.map(|value_hash| (value_hash, id.clone())),
                )
            })
            .collect();
        let next_version = self.current_version + 1;
        let (root_hash, update_result) = JellyfishMerkleTree::new(self.store)
            .batch_put_value_set(
                value_set
                    .iter()
                    .map(|(x, y)| (x.clone(), y.as_ref()))
                    .collect(),
                None,
                Some(self.current_version).filter(|version| *version > 0u64),
                next_version,
            )
            .expect("error while reading tree during put");
        for (key, node) in update_result.node_batch.iter().flatten() {
            self.store.insert_node(key, TreeNode::from(key, node));
        }
        for stale_node in update_result.stale_node_index_batch.iter().flatten() {
            self.store.record_stale_node(&stale_node.node_key);
        }
        self.current_version = next_version;
        root_hash
    }

    /// Returns the hash of a state at version [`HashTree::current_version`].
    ///
    /// # Panics
    /// Panics if a root node for [`HashTree::current_version`] does not exist.
    pub fn get_current_root_hash(&self) -> Hash {
        JellyfishMerkleTree::new(self.store)
            .get_root_hash(self.current_version)
            .expect("error while reading root hash")
    }
}

impl TreeInternalNode {
    fn from(internal_node: &InternalNode) -> Self {
        let children = internal_node
            .children_sorted()
            .map(|(nibble, child)| TreeChildEntry {
                nibble: nibble.clone(),
                version: child.version,
                hash: child.hash,
                is_leaf: child.is_leaf(),
            })
            .collect::<Vec<TreeChildEntry>>();
        TreeInternalNode { children }
    }
}

impl TreeLeafNode {
    fn from(key: &NodeKey, leaf_node: &LeafNode<SubstateId>) -> Self {
        TreeLeafNode {
            key_suffix: NibblePath::from_iter(
                NibblePath::new_even(leaf_node.account_key().to_vec())
                    .nibbles()
                    .skip(key.nibble_path().num_nibbles()),
            ),
            substate_id: leaf_node.value_index().0.clone(),
            value_hash: leaf_node.value_hash(),
        }
    }
}

impl TreeNode {
    fn from(key: &NodeKey, node: &Node<SubstateId>) -> Self {
        match node {
            Node::Internal(internal_node) => {
                TreeNode::Internal(TreeInternalNode::from(internal_node))
            }
            Node::Leaf(leaf_node) => TreeNode::Leaf(TreeLeafNode::from(key, leaf_node)),
            Node::Null => TreeNode::Null,
        }
    }
}

impl InternalNode {
    fn from(internal_node: &TreeInternalNode) -> Self {
        let child_map: HashMap<Nibble, Child> = internal_node
            .children
            .iter()
            .map(|child_entry| {
                let child: Child = Child::new(
                    child_entry.hash,
                    child_entry.version,
                    if child_entry.is_leaf {
                        NodeType::Leaf
                    } else {
                        // Note: the `0` passed here may be replaced with an actual value (which we
                        // would have to persist) once we have use-cases for quick look-ups of leaf
                        // counts.
                        NodeType::Internal { leaf_count: 0 }
                    },
                );
                (child_entry.nibble, child)
            })
            .collect();
        InternalNode::new(child_map)
    }
}

impl LeafNode<SubstateId> {
    fn from(key: &NodeKey, leaf_node: &TreeLeafNode) -> Self {
        let full_key = NibblePath::from_iter(
            key.nibble_path()
                .nibbles()
                .chain(leaf_node.key_suffix.nibbles()),
        );
        LeafNode::new(
            Hash::try_from(full_key.bytes()).unwrap(),
            leaf_node.value_hash,
            (leaf_node.substate_id.clone(), key.version()),
        )
    }
}

impl Node<SubstateId> {
    fn from(key: &NodeKey, tree_node: &TreeNode) -> Self {
        match tree_node {
            TreeNode::Internal(internal_node) => Node::Internal(InternalNode::from(internal_node)),
            TreeNode::Leaf(leaf_node) => Node::Leaf(LeafNode::from(key, leaf_node)),
            TreeNode::Null => Node::Null,
        }
    }
}

impl<R: ReadableTreeStore> TreeReader<SubstateId> for R {
    fn get_node_option(&self, node_key: &NodeKey) -> Result<Option<Node<SubstateId>>, Error> {
        Ok(self
            .get_node(node_key)
            .map(|tree_node| Node::from(node_key, &tree_node)))
    }
}
