// Re-exports
pub use super::types::{Nibble, NibblePath, NodeKey, Version};

use radix_engine_common::crypto::Hash;
use radix_engine_common::data::scrypto::{scrypto_decode, scrypto_encode};
use sbor::rust::cell::Ref;
use sbor::rust::cell::RefCell;
use sbor::*;
use substate_store_interface::interface::DbSubstateValue;
use utils::rust::collections::VecDeque;
use utils::rust::collections::{hash_map_new, HashMap};
use utils::rust::vec::Vec;

define_single_versioned! {
    #[derive(Clone, PartialEq, Eq, Hash, Debug, Sbor)]
    pub enum VersionedTreeNode => TreeNode = TreeNodeV1
}

/// A physical tree node, to be used in the storage.
#[derive(Clone, PartialEq, Eq, Hash, Debug, Sbor)]
pub enum TreeNodeV1 {
    /// Internal node - always metadata-only, as per JMT design.
    Internal(TreeInternalNode),
    /// Leaf node.
    Leaf(TreeLeafNode),
    /// An "empty tree" indicator, which may only be used as a root.
    Null,
}

/// Internal node.
#[derive(Clone, PartialEq, Eq, Hash, Debug, Sbor)]
pub struct TreeInternalNode {
    /// Metadata of each existing child.
    pub children: Vec<TreeChildEntry>,
}

/// Child node metadata.
#[derive(Clone, PartialEq, Eq, Hash, Debug, Sbor)]
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

/// Leaf node.
#[derive(Clone, PartialEq, Eq, Hash, Debug, Sbor)]
pub struct TreeLeafNode {
    /// All the remaining nibbles in the _hashed_ payload's key.
    pub key_suffix: NibblePath,
    /// An externally-provided hash of the payload.
    pub value_hash: Hash,
    /// A version at which the [`value_hash`] has most recently changed.
    pub last_hash_change_version: Version,
}

/// A part of a tree that may become stale (i.e. need eventual pruning).
#[derive(Clone, PartialEq, Eq, Hash, Debug, Sbor)]
pub enum StaleTreePart {
    /// A single node to be removed.
    Node(NodeKey),
    /// An entire subtree of descendants of a specific node (including itself).
    Subtree(NodeKey),
}

/// The "read" part of a physical tree node storage SPI.
pub trait ReadableTreeStore {
    /// Gets node by key, if it exists.
    fn get_node(&self, key: &NodeKey) -> Option<TreeNode>;
}

/// The "write" part of a physical tree node storage SPI.
pub trait WriteableTreeStore {
    /// Inserts the node under a new, unique key (i.e. never an update).
    fn insert_node(&self, key: NodeKey, node: TreeNode);

    /// Associates the actually upserted Substate's value with the given key.
    ///
    /// This method will be called before the [`Self::insert_node()`] of Substate-Tier leaf nodes,
    /// and allows the storage to keep correlated historical values, if required.
    fn associate_substate_value(&self, key: &NodeKey, substate_value: &DbSubstateValue);

    /// Marks the given tree part for a (potential) future removal by an arbitrary external pruning
    /// process.
    fn record_stale_tree_part(&self, part: StaleTreePart);
}

/// A complete tree node storage SPI.
pub trait TreeStore: ReadableTreeStore + WriteableTreeStore {}
impl<S: ReadableTreeStore + WriteableTreeStore> TreeStore for S {}

/// A `TreeStore` based on memory object copies (i.e. no serialization).
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TypedInMemoryTreeStore {
    pub tree_nodes: RefCell<HashMap<NodeKey, TreeNode>>,
    pub stale_part_buffer: RefCell<Vec<StaleTreePart>>,
    pub pruning_enabled: bool,
}

impl TypedInMemoryTreeStore {
    /// A constructor of a newly-initialized, empty store.
    pub fn new() -> TypedInMemoryTreeStore {
        TypedInMemoryTreeStore {
            tree_nodes: RefCell::new(hash_map_new()),
            stale_part_buffer: RefCell::new(Vec::new()),
            pruning_enabled: false,
        }
    }

    pub fn with_pruning() -> TypedInMemoryTreeStore {
        TypedInMemoryTreeStore {
            tree_nodes: RefCell::new(hash_map_new()),
            stale_part_buffer: RefCell::new(Vec::new()),
            pruning_enabled: true,
        }
    }
}

impl ReadableTreeStore for TypedInMemoryTreeStore {
    fn get_node(&self, key: &NodeKey) -> Option<TreeNode> {
        self.tree_nodes.borrow().get(key).cloned()
    }
}

impl WriteableTreeStore for TypedInMemoryTreeStore {
    fn insert_node(&self, key: NodeKey, node: TreeNode) {
        self.tree_nodes.borrow_mut().insert(key, node);
    }

    fn associate_substate_value(&self, _key: &NodeKey, _substate_value: &DbSubstateValue) {
        // intentionally empty
    }

    fn record_stale_tree_part(&self, part: StaleTreePart) {
        if self.pruning_enabled {
            match part {
                StaleTreePart::Node(node_key) => {
                    self.tree_nodes.borrow_mut().remove(&node_key);
                }
                StaleTreePart::Subtree(node_key) => {
                    let mut queue = VecDeque::new();
                    queue.push_back(node_key);

                    while let Some(node_key) = queue.pop_front() {
                        if let Some(value) = self.tree_nodes.borrow_mut().remove(&node_key) {
                            match value {
                                TreeNodeV1::Internal(x) => {
                                    for child in x.children {
                                        queue.push_back(
                                            node_key
                                                .gen_child_node_key(child.version, child.nibble),
                                        )
                                    }
                                }
                                TreeNodeV1::Leaf(_) => {}
                                TreeNodeV1::Null => {}
                            }
                        }
                    }
                }
            }
        } else {
            self.stale_part_buffer.borrow_mut().push(part);
        }
    }
}

/// A `TreeStore` based on serialized payloads stored in memory.
#[derive(Debug, PartialEq, Eq)]
pub struct SerializedInMemoryTreeStore {
    memory: RefCell<HashMap<Vec<u8>, Vec<u8>>>,
    stale_part_buffer: RefCell<Vec<Vec<u8>>>,
}

impl SerializedInMemoryTreeStore {
    /// A constructor of a newly-initialized, empty store.
    pub fn new() -> Self {
        Self {
            memory: RefCell::new(hash_map_new()),
            stale_part_buffer: RefCell::new(Vec::new()),
        }
    }

    pub fn memory(&self) -> Ref<HashMap<Vec<u8>, Vec<u8>>> {
        self.memory.borrow()
    }
}

impl ReadableTreeStore for SerializedInMemoryTreeStore {
    fn get_node(&self, key: &NodeKey) -> Option<TreeNode> {
        self.memory
            .borrow()
            .get(&encode_key(key))
            .map(|bytes| scrypto_decode(bytes).unwrap())
    }
}

impl WriteableTreeStore for SerializedInMemoryTreeStore {
    fn insert_node(&self, key: NodeKey, node: TreeNode) {
        self.memory
            .borrow_mut()
            .insert(encode_key(&key), scrypto_encode(&node).unwrap());
    }

    fn associate_substate_value(&self, _key: &NodeKey, _substate_value: &DbSubstateValue) {
        // intentionally empty
    }

    fn record_stale_tree_part(&self, part: StaleTreePart) {
        self.stale_part_buffer
            .borrow_mut()
            .push(scrypto_encode(&part).unwrap());
    }
}

/// Encodes the given node key in a format friendly to Level-like databases (i.e. strictly ordered
/// by numeric version).
pub fn encode_key(key: &NodeKey) -> Vec<u8> {
    let version_bytes = &key.version().to_be_bytes();
    let nibble_path_bytes = key.nibble_path().bytes();
    let parity_byte = &[(key.nibble_path().num_nibbles() % 2) as u8; 1];
    [version_bytes, nibble_path_bytes, parity_byte].concat()
}

// Note: We need completely custom serialization scheme only for the node keys. The remaining
// structures can simply use SBOR, with only the most efficiency-sensitive parts having custom
// codecs, implemented below:

impl<X: CustomValueKind> Categorize<X> for Nibble {
    #[inline]
    fn value_kind() -> ValueKind<X> {
        ValueKind::U8
    }
}

impl<X: CustomValueKind, E: Encoder<X>> Encode<X, E> for Nibble {
    #[inline]
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(Self::value_kind())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        u8::from(*self).encode_body(encoder)
    }
}

impl<X: CustomValueKind, D: Decoder<X>> Decode<X, D> for Nibble {
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<X>,
    ) -> Result<Self, DecodeError> {
        Ok(Nibble::from(u8::decode_body_with_value_kind(
            decoder, value_kind,
        )?))
    }
}

impl<T: CustomTypeKind<RustTypeId>> Describe<T> for Nibble {
    const TYPE_ID: RustTypeId = RustTypeId::WellKnown(basic_well_known_types::U8_TYPE);

    fn type_data() -> TypeData<T, RustTypeId> {
        basic_well_known_types::u8_type_data()
    }
}

impl<X: CustomValueKind> Categorize<X> for NibblePath {
    #[inline]
    fn value_kind() -> ValueKind<X> {
        ValueKind::Tuple
    }
}

impl<X: CustomValueKind, E: Encoder<X>> Encode<X, E> for NibblePath {
    #[inline]
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(Self::value_kind())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        let even = self.num_nibbles() % 2 == 0;
        (even, self.bytes()).encode_body(encoder)
    }
}

impl<X: CustomValueKind, D: Decoder<X>> Decode<X, D> for NibblePath {
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<X>,
    ) -> Result<Self, DecodeError> {
        let (even, bytes): (bool, Vec<u8>) =
            Decode::<X, D>::decode_body_with_value_kind(decoder, value_kind)?;
        let path = if even {
            NibblePath::new_even(bytes)
        } else {
            NibblePath::new_odd(bytes)
        };
        Ok(path)
    }
}

impl<T: CustomTypeKind<RustTypeId>> Describe<T> for NibblePath {
    const TYPE_ID: RustTypeId = <(bool, Vec<u8>) as Describe<T>>::TYPE_ID;

    fn type_data() -> TypeData<T, RustTypeId> {
        <(bool, Vec<u8>) as Describe<T>>::type_data()
    }
}
