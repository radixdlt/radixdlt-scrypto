use super::tier_framework::StoredNode;
// Re-exports
pub use super::types::{Nibble, NibblePath, TreeNodeKey, Version};
use super::{Node, StorageError, TreeReader};

use radix_common::prelude::*;
use radix_substate_store_interface::interface::*;
use sbor::rust::cell::Ref;
use sbor::rust::cell::RefCell;

define_single_versioned! {
    #[derive(Clone, PartialEq, Eq, Hash, Debug, Sbor)]
    pub VersionedTreeNode(TreeNodeVersions) => TreeNode = TreeNodeV1,
    outer_attributes: [
        // This is used in a Rocks CF in the node, so needs to have a backwards compatibility
        // assertion trait.
        #[derive(ScryptoSborAssertion)]
        #[sbor_assert(backwards_compatible(
            cuttlefish = "FILE:versioned_tree_node_schema.bin"
        ))]
    ]
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
    Node(StoredTreeNodeKey),
    /// An entire subtree of descendants of a specific node (including itself).
    Subtree(StoredTreeNodeKey),
}

/// A global tree node key, made collision-free from other layers
#[derive(Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd, Sbor)]
pub struct StoredTreeNodeKey {
    // The version at which the node is created.
    version: Version,
    // The nibble path this node represents in the tree.
    nibble_path: NibblePath,
}

impl StoredTreeNodeKey {
    pub fn new(version: Version, nibble_path: NibblePath) -> Self {
        Self {
            version,
            nibble_path,
        }
    }

    pub fn unprefixed(local_node_key: TreeNodeKey) -> Self {
        let (version, nibble_path) = local_node_key.into();
        Self {
            version,
            nibble_path,
        }
    }

    pub fn prefixed(prefix_bytes: &[u8], local_node_key: &TreeNodeKey) -> Self {
        Self {
            version: local_node_key.version(),
            nibble_path: local_node_key.nibble_path().prefix_with(prefix_bytes),
        }
    }

    /// Gets the version.
    pub fn version(&self) -> Version {
        self.version
    }

    /// Gets the nibble path.
    pub fn nibble_path(&self) -> &NibblePath {
        &self.nibble_path
    }

    /// Generates a child node key based on this node key.
    pub fn gen_child_node_key(&self, version: Version, n: Nibble) -> Self {
        let mut node_nibble_path = self.nibble_path().clone();
        node_nibble_path.push(n);
        Self::new(version, node_nibble_path)
    }

    /// Generates parent node key at the same version based on this node key.
    pub fn gen_parent_node_key(&self) -> Self {
        let mut node_nibble_path = self.nibble_path().clone();
        assert!(
            node_nibble_path.pop().is_some(),
            "Current node key is root.",
        );
        Self::new(self.version, node_nibble_path)
    }
}

impl From<StoredTreeNodeKey> for (Version, NibblePath) {
    fn from(value: StoredTreeNodeKey) -> Self {
        (value.version, value.nibble_path)
    }
}

/// The "read" part of a physical tree node storage SPI.
pub trait ReadableTreeStore {
    /// Gets node by key, if it exists.
    fn get_node(&self, global_key: &StoredTreeNodeKey) -> Option<TreeNode>;
}

/// The "write" part of a physical tree node storage SPI.
pub trait WriteableTreeStore {
    /// Inserts the node under a new, unique key (i.e. never an update).
    fn insert_node(&self, global_key: StoredTreeNodeKey, node: TreeNode);

    /// Associates an inserted Substate-Tier tree leaf with the Substate it represents.
    ///
    /// This method will be called after the [`Self::insert_node()`] of Substate-Tier leaf nodes,
    /// and allows the storage to keep correlated historical values, if required. The correlation
    /// may be implemented either directly (i.e. to the given `substate_value`) or via the given
    /// `partition_key` + `sort_key` (if it makes sense e.g. for performance reasons).
    fn associate_substate(
        &self,
        state_tree_leaf_key: &StoredTreeNodeKey,
        partition_key: &DbPartitionKey,
        sort_key: &DbSortKey,
        substate_value: AssociatedSubstateValue,
    );

    /// Marks the given tree part for a (potential) future removal by an arbitrary external pruning
    /// process.
    fn record_stale_tree_part(&self, global_tree_part: StaleTreePart);
}

/// A Substate value associated with a tree leaf (see [`WriteableTreeStore#associate_substate()`]).
///
/// Implementation note: _("why can't we simply pass `&DbSubstateValue` there?")_
/// In JMT, a new leaf node may be inserted *not* only due to Substate upsert, but also as a result
/// of a "tree restructuring" (an internal JMT behavior, needed e.g. when a previously-only-child
/// gains a sibling). In such cases, the associated Substate itself is actually untouched, and we
/// would have to load it from the [`SubstateDatabase`] (only to pass its value). We choose to
/// avoid this potential performance implication by having an explicit way of associating an
/// "unchanged" Substate with its new tree leaf.
pub enum AssociatedSubstateValue<'v> {
    Upserted(&'v [u8]),
    Unchanged,
}

/// A complete tree node storage SPI.
pub trait TreeStore: ReadableTreeStore + WriteableTreeStore {}
impl<S: ReadableTreeStore + WriteableTreeStore> TreeStore for S {}

/// A `TreeStore` based on memory object copies (i.e. no serialization).
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TypedInMemoryTreeStore {
    pub tree_nodes: RefCell<HashMap<StoredTreeNodeKey, TreeNode>>,
    pub stale_part_buffer: RefCell<Vec<StaleTreePart>>,
    pub associated_substates:
        RefCell<HashMap<StoredTreeNodeKey, (DbSubstateKey, Option<DbSubstateValue>)>>,
    pub pruning_enabled: bool,
    pub store_associated_substates: bool,
}

impl TypedInMemoryTreeStore {
    /// A constructor of a newly-initialized, empty store.
    pub fn new() -> Self {
        Self {
            tree_nodes: RefCell::new(hash_map_new()),
            stale_part_buffer: RefCell::new(Vec::new()),
            associated_substates: RefCell::new(hash_map_new()),
            pruning_enabled: false,
            store_associated_substates: false,
        }
    }

    pub fn with_pruning_enabled(self) -> Self {
        Self {
            pruning_enabled: true,
            ..self
        }
    }

    pub fn storing_associated_substates(self) -> Self {
        Self {
            store_associated_substates: true,
            ..self
        }
    }
}

// This implementation allows interpreting the TypedInMemoryTreeStore as a single store
impl TreeReader<Version> for TypedInMemoryTreeStore {
    fn get_node_option(
        &self,
        node_key: &TreeNodeKey,
    ) -> Result<Option<Node<Version>>, StorageError> {
        Ok(
            ReadableTreeStore::get_node(self, &StoredTreeNodeKey::unprefixed(node_key.clone()))
                .map(|tree_node| tree_node.into_jmt_node(&node_key)),
        )
    }
}

impl ReadableTreeStore for TypedInMemoryTreeStore {
    fn get_node(&self, key: &StoredTreeNodeKey) -> Option<TreeNode> {
        self.tree_nodes.borrow().get(key).cloned()
    }
}

impl WriteableTreeStore for TypedInMemoryTreeStore {
    fn insert_node(&self, key: StoredTreeNodeKey, node: TreeNode) {
        self.tree_nodes.borrow_mut().insert(key, node);
    }

    fn associate_substate(
        &self,
        state_tree_leaf_key: &StoredTreeNodeKey,
        partition_key: &DbPartitionKey,
        sort_key: &DbSortKey,
        substate_value: AssociatedSubstateValue,
    ) {
        if self.store_associated_substates {
            let substate_value = match substate_value {
                AssociatedSubstateValue::Upserted(value) => Some(value.to_owned()),
                AssociatedSubstateValue::Unchanged => None,
            };
            self.associated_substates.borrow_mut().insert(
                state_tree_leaf_key.clone(),
                ((partition_key.clone(), sort_key.clone()), substate_value),
            );
        }
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
    fn get_node(&self, key: &StoredTreeNodeKey) -> Option<TreeNode> {
        self.memory
            .borrow()
            .get(&encode_key(key))
            .map(|bytes| scrypto_decode(bytes).unwrap())
    }
}

impl WriteableTreeStore for SerializedInMemoryTreeStore {
    fn insert_node(&self, key: StoredTreeNodeKey, node: TreeNode) {
        self.memory
            .borrow_mut()
            .insert(encode_key(&key), scrypto_encode(&node).unwrap());
    }

    fn associate_substate(
        &self,
        _state_tree_leaf_key: &StoredTreeNodeKey,
        _partition_key: &DbPartitionKey,
        _sort_key: &DbSortKey,
        _substate_value: AssociatedSubstateValue,
    ) {
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
pub fn encode_key(key: &StoredTreeNodeKey) -> Vec<u8> {
    let version_bytes = &key.version().to_be_bytes();
    let nibble_path_bytes = key.nibble_path().bytes();
    let parity_byte = &[(key.nibble_path().num_nibbles() % 2) as u8; 1];
    [version_bytes, nibble_path_bytes, parity_byte].concat()
}

// Note: We need completely custom serialization scheme only for the node keys. The remaining
// structures can simply use SBOR, with only the most efficiency-sensitive parts having custom
// codecs, implemented below:

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
