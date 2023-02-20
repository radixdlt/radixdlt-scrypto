use sbor::rust::collections::HashMap;
use sbor::rust::vec::Vec;

use radix_engine_interface::crypto::Hash;

use super::tree_store::{
    Payload, ReadableTreeStore, TreeChildEntry, TreeInternalNode, TreeLeafNode, TreeNode,
};
use super::types::{
    Child, InternalNode, LeafNode, Nibble, NibblePath, Node, NodeKey, NodeType, StorageError,
    TreeReader,
};

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

impl<P: Clone> TreeLeafNode<P> {
    fn from(key: &NodeKey, leaf_node: &LeafNode<P>) -> Self {
        TreeLeafNode {
            key_suffix: NibblePath::from_iter(
                NibblePath::new_even(leaf_node.account_key().to_vec())
                    .nibbles()
                    .skip(key.nibble_path().num_nibbles()),
            ),
            payload: leaf_node.value_index().0.clone(),
            value_hash: leaf_node.value_hash(),
        }
    }
}

impl<P: Clone> TreeNode<P> {
    pub fn from(key: &NodeKey, node: &Node<P>) -> Self {
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

impl<P: Clone> Node<P> {
    fn from(key: &NodeKey, tree_node: &TreeNode<P>) -> Self {
        match tree_node {
            TreeNode::Internal(internal_node) => Node::Internal(InternalNode::from(internal_node)),
            TreeNode::Leaf(leaf_node) => Node::Leaf(LeafNode::from(key, leaf_node)),
            TreeNode::Null => Node::Null,
        }
    }
}

impl<P: Clone> LeafNode<P> {
    fn from(key: &NodeKey, leaf_node: &TreeLeafNode<P>) -> Self {
        let full_key = NibblePath::from_iter(
            key.nibble_path()
                .nibbles()
                .chain(leaf_node.key_suffix.nibbles()),
        );
        LeafNode::new(
            Hash::try_from(full_key.bytes()).unwrap(),
            leaf_node.value_hash,
            (leaf_node.payload.clone(), key.version()),
        )
    }
}

impl<R: ReadableTreeStore<P>, P: Payload> TreeReader<P> for R {
    fn get_node_option(&self, node_key: &NodeKey) -> Result<Option<Node<P>>, StorageError> {
        Ok(self
            .get_node(node_key)
            .map(|tree_node| Node::from(node_key, &tree_node)))
    }
}
