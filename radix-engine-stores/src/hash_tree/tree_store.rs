use sbor::rust::collections::HashMap;
use sbor::rust::vec::Vec;

pub use super::types::{Nibble, NibblePath, NodeKey, Version};
use radix_engine_interface::api::types::SubstateId;
use radix_engine_interface::crypto::Hash;

/// A physical tree node, to be used in the storage.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum TreeNode {
    /// Internal node - always metadata-only, as per JMT design.
    Internal(TreeInternalNode),
    /// Leaf node.
    Leaf(TreeLeafNode),
    /// An "empty tree" indicator, which may only be used as a root.
    Null,
}

/// Internal node.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct TreeInternalNode {
    /// Metadata of each existing child.
    pub children: Vec<TreeChildEntry>,
}

/// Child node metadata.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct TreeChildEntry {
    /// First of the remaining nibbles in the key.
    pub nibble: Nibble,
    /// State version at which this child's node was created.
    pub version: u64,
    /// Cached child hash (i.e. needed only for performance).
    pub hash: Hash,
    /// Cached child type indicator (i.e. needed only for performance).
    pub is_leaf: bool,
}

/// Physical leaf node (which may represent a ReNode or a Substate).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct TreeLeafNode {
    /// All the remaining nibbles in the _hashed_ `substate_id`.
    pub key_suffix: NibblePath,
    /// ID of the substate's value in an external storage.
    pub substate_id: SubstateId,
    /// An externally-provided hash of the Substate's value.
    pub value_hash: Hash,
}

/// The "read" part of a physical tree node storage SPI.
pub trait ReadableTreeStore {
    /// Gets node by key, if it exists.
    fn get_node(&self, key: &NodeKey) -> Option<TreeNode>;
}

/// The "write" part of a physical tree node storage SPI.
pub trait WriteableTreeStore {
    /// Inserts the node under a new, unique key (i.e. never an update).
    fn insert_node(&mut self, key: &NodeKey, node: TreeNode);

    /// Marks the given node for a (potential) future removal by an arbitrary
    /// external pruning process.
    fn record_stale_node(&mut self, key: &NodeKey);
}

/// A complete tree node storage SPI.
pub trait TreeStore: ReadableTreeStore + WriteableTreeStore {}
impl<S: ReadableTreeStore + WriteableTreeStore> TreeStore for S {}

/// A `TreeStore` based on memory object copies (i.e. no serialization).
pub struct MemoryTreeStore {
    pub memory: HashMap<NodeKey, TreeNode>,
    pub stale_key_buffer: Vec<NodeKey>,
}

impl MemoryTreeStore {
    /// A constructor of a newly-initialized, empty store.
    pub fn new() -> MemoryTreeStore {
        MemoryTreeStore {
            memory: HashMap::new(),
            stale_key_buffer: Vec::new(),
        }
    }
}

impl ReadableTreeStore for MemoryTreeStore {
    fn get_node(&self, key: &NodeKey) -> Option<TreeNode> {
        self.memory.get(key).cloned()
    }
}

impl WriteableTreeStore for MemoryTreeStore {
    fn insert_node(&mut self, key: &NodeKey, node: TreeNode) {
        self.memory.insert(key.clone(), node);
    }

    fn record_stale_node(&mut self, key: &NodeKey) {
        self.stale_key_buffer.push(key.clone());
    }
}

/// A [`TreeStore`] which overlays a "base" read-only store with a read-write
/// [`MemoryTreeStore`] layer.
pub struct StagedTreeStore<'s, R: ReadableTreeStore> {
    base: &'s R,
    overlay: MemoryTreeStore,
}

impl<'s, R: ReadableTreeStore> StagedTreeStore<'s, R> {
    /// A direct constructor.
    pub fn new(base: &'s R) -> StagedTreeStore<'s, R> {
        StagedTreeStore {
            base,
            overlay: MemoryTreeStore::new(),
        }
    }
}

impl<'s, R: ReadableTreeStore> ReadableTreeStore for StagedTreeStore<'s, R> {
    fn get_node(&self, key: &NodeKey) -> Option<TreeNode> {
        self.overlay
            .get_node(key)
            .or_else(|| self.base.get_node(key))
    }
}

impl<'s, R: ReadableTreeStore> WriteableTreeStore for StagedTreeStore<'s, R> {
    fn insert_node(&mut self, key: &NodeKey, node: TreeNode) {
        self.overlay.insert_node(key, node);
    }

    fn record_stale_node(&mut self, key: &NodeKey) {
        self.overlay.record_stale_node(key)
    }
}
