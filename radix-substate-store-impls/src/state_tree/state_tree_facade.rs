use super::tier_framework::*;
use super::tree_store::*;
use super::types::*;
use radix_rust::prelude::*;

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
    fn from(key: &TreeNodeKey, leaf_node: &LeafNode<Version>) -> Self {
        TreeLeafNode {
            key_suffix: NibblePath::from_iter(
                NibblePath::new_even(leaf_node.leaf_key().bytes.clone())
                    .nibbles()
                    .skip(key.nibble_path().num_nibbles()),
            ),
            value_hash: leaf_node.value_hash(),
            last_hash_change_version: leaf_node.payload().clone(),
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

impl StoredNode for TreeNode {
    type Payload = Version;

    fn into_jmt_node(&self, key: &TreeNodeKey) -> Node<Version> {
        match self {
            TreeNode::Internal(internal_node) => Node::Internal(InternalNode::from(&internal_node)),
            TreeNode::Leaf(leaf_node) => Node::Leaf(LeafNode::from(key, &leaf_node)),
            TreeNode::Null => Node::Null,
        }
    }

    fn from_jmt_node(node: &Node<Self::Payload>, key: &TreeNodeKey) -> Self {
        match node {
            Node::Internal(internal_node) => {
                TreeNode::Internal(TreeInternalNode::from(&internal_node))
            }
            Node::Leaf(leaf_node) => TreeNode::Leaf(TreeLeafNode::from(key, &leaf_node)),
            Node::Null => TreeNode::Null,
        }
    }
}

impl LeafNode<Version> {
    pub fn from(key: &TreeNodeKey, leaf_node: &TreeLeafNode) -> Self {
        let full_key = NibblePath::from_iter(
            key.nibble_path()
                .nibbles()
                .chain(leaf_node.key_suffix.nibbles()),
        );
        LeafNode::new(
            LeafKey::new(full_key.bytes()),
            leaf_node.value_hash,
            leaf_node.last_hash_change_version,
            key.version(),
        )
    }
}
