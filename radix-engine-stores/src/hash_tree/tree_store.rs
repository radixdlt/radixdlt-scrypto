use radix_engine::types::{ScryptoCategorize, ScryptoDecode, ScryptoEncode};
use sbor::rust::collections::HashMap;
use sbor::rust::vec::Vec;

pub use super::types::{Nibble, NibblePath, NodeKey, Version};
use radix_engine_interface::api::types::{NodeModuleId, RENodeId, SubstateOffset};
use radix_engine_interface::crypto::Hash;
use radix_engine_interface::data::{scrypto_decode, scrypto_encode, ScryptoCustomValueKind};
use sbor::{Categorize, Decode, DecodeError, Decoder, Encode, EncodeError, Encoder, ValueKind};

/// A physical tree node, to be used in the storage.
#[derive(Clone, PartialEq, Eq, Hash, Debug, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum TreeNode<P> {
    /// Internal node - always metadata-only, as per JMT design.
    Internal(TreeInternalNode),
    /// Leaf node.
    Leaf(TreeLeafNode<P>),
    /// An "empty tree" indicator, which may only be used as a root.
    Null,
}

/// Internal node.
#[derive(Clone, PartialEq, Eq, Hash, Debug, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct TreeInternalNode {
    /// Metadata of each existing child.
    pub children: Vec<TreeChildEntry>,
}

/// Child node metadata.
#[derive(Clone, PartialEq, Eq, Hash, Debug, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
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
#[derive(Clone, PartialEq, Eq, Hash, Debug, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct TreeLeafNode<P> {
    /// All the remaining nibbles in the _hashed_ payload's key.
    pub key_suffix: NibblePath,
    /// Payload; contents depend on the layer.
    pub payload: P,
    /// An externally-provided hash of the payload.
    pub value_hash: Hash,
}

/// Payload of the leafs within the upper (ReNode+Module) layer.
/// Please note that a ReNode leaf is conceptually identical to a root of the Substates' subtree
/// (i.e. one exists if and only if the other exists). For this reason, this payload does _not_
/// just reference the subtree root, but actually contains it inside.
/// This design decision also brings minor space and runtime benefits, and avoids special-casing
/// the physical `NodeKey`s (no clashes can occur between ReNode leaf and Substates' root).
#[derive(Clone, PartialEq, Eq, Hash, Debug, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct ReNodeModulePayload {
    /// ReNode ID.
    pub re_node_id: RENodeId,
    /// Module ID.
    pub node_mode_id: NodeModuleId,
    /// An embedded root of the descendant Substate layer tree.
    pub substates_root: TreeNode<SubstateOffset>,
}

/// A payload carried by a physical leaf.
/// The top ReNodeModule tree carries an `ReNodeModulePayload` payload.
/// The sub-trees carry  a `SubstateOffset` payload.
pub trait Payload:
    Clone
    + PartialEq
    + Eq
    + sbor::rust::hash::Hash
    + sbor::rust::fmt::Debug
    + ScryptoCategorize
    + ScryptoEncode
    + ScryptoDecode
{
}

impl Payload for ReNodeModulePayload {}

impl Payload for SubstateOffset {}

/// The "read" part of a physical tree node storage SPI.
pub trait ReadableTreeStore<P: Payload> {
    /// Gets node by key, if it exists.
    fn get_node(&self, key: &NodeKey) -> Option<TreeNode<P>>;
}

/// The "write" part of a physical tree node storage SPI.
pub trait WriteableTreeStore<P: Payload> {
    /// Inserts the node under a new, unique key (i.e. never an update).
    fn insert_node(&mut self, key: NodeKey, node: TreeNode<P>);

    /// Marks the given node for a (potential) future removal by an arbitrary
    /// external pruning process.
    fn record_stale_node(&mut self, key: NodeKey);
}

/// A complete tree node storage SPI.
pub trait TreeStore<P: Payload>: ReadableTreeStore<P> + WriteableTreeStore<P> {}
impl<S: ReadableTreeStore<P> + WriteableTreeStore<P>, P: Payload> TreeStore<P> for S {}

/// A `TreeStore` based on memory object copies (i.e. no serialization).
#[derive(Debug, PartialEq, Eq)]
pub struct TypedInMemoryTreeStore {
    pub root_tree_nodes: HashMap<NodeKey, TreeNode<ReNodeModulePayload>>,
    pub sub_tree_nodes: HashMap<NodeKey, TreeNode<SubstateOffset>>,
    pub stale_key_buffer: Vec<NodeKey>,
}

impl TypedInMemoryTreeStore {
    /// A constructor of a newly-initialized, empty store.
    pub fn new() -> TypedInMemoryTreeStore {
        TypedInMemoryTreeStore {
            root_tree_nodes: HashMap::new(),
            sub_tree_nodes: HashMap::new(),
            stale_key_buffer: Vec::new(),
        }
    }
}

impl ReadableTreeStore<SubstateOffset> for TypedInMemoryTreeStore {
    fn get_node(&self, key: &NodeKey) -> Option<TreeNode<SubstateOffset>> {
        self.sub_tree_nodes.get(key).cloned()
    }
}

impl WriteableTreeStore<SubstateOffset> for TypedInMemoryTreeStore {
    fn insert_node(&mut self, key: NodeKey, node: TreeNode<SubstateOffset>) {
        self.sub_tree_nodes.insert(key, node);
    }

    fn record_stale_node(&mut self, key: NodeKey) {
        self.stale_key_buffer.push(key);
    }
}

impl ReadableTreeStore<ReNodeModulePayload> for TypedInMemoryTreeStore {
    fn get_node(&self, key: &NodeKey) -> Option<TreeNode<ReNodeModulePayload>> {
        self.root_tree_nodes.get(key).cloned()
    }
}

impl WriteableTreeStore<ReNodeModulePayload> for TypedInMemoryTreeStore {
    fn insert_node(&mut self, key: NodeKey, node: TreeNode<ReNodeModulePayload>) {
        self.root_tree_nodes.insert(key, node);
    }

    fn record_stale_node(&mut self, key: NodeKey) {
        self.stale_key_buffer.push(key);
    }
}

/// A `TreeStore` based on serialized payloads stored in memory.
#[derive(Debug, PartialEq, Eq)]
pub struct SerializedInMemoryTreeStore {
    pub memory: HashMap<Vec<u8>, Vec<u8>>,
    pub stale_key_buffer: Vec<Vec<u8>>,
}

impl SerializedInMemoryTreeStore {
    /// A constructor of a newly-initialized, empty store.
    pub fn new() -> Self {
        Self {
            memory: HashMap::new(),
            stale_key_buffer: Vec::new(),
        }
    }
}

impl<P: Payload> ReadableTreeStore<P> for SerializedInMemoryTreeStore {
    fn get_node(&self, key: &NodeKey) -> Option<TreeNode<P>> {
        self.memory
            .get(&encode_key(key))
            .map(|bytes| scrypto_decode(bytes).unwrap())
    }
}

impl<P: Payload> WriteableTreeStore<P> for SerializedInMemoryTreeStore {
    fn insert_node(&mut self, key: NodeKey, node: TreeNode<P>) {
        self.memory
            .insert(encode_key(&key), scrypto_encode(&node).unwrap());
    }

    fn record_stale_node(&mut self, key: NodeKey) {
        self.stale_key_buffer.push(encode_key(&key));
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

impl Categorize<ScryptoCustomValueKind> for Nibble {
    #[inline]
    fn value_kind() -> ValueKind<ScryptoCustomValueKind> {
        ValueKind::U8
    }
}

impl<E: Encoder<ScryptoCustomValueKind>> Encode<ScryptoCustomValueKind, E> for Nibble {
    #[inline]
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(Self::value_kind())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        u8::from(*self).encode_body(encoder)
    }
}

impl<D: Decoder<ScryptoCustomValueKind>> Decode<ScryptoCustomValueKind, D> for Nibble {
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<ScryptoCustomValueKind>,
    ) -> Result<Self, DecodeError> {
        Ok(Nibble::from(u8::decode_body_with_value_kind(
            decoder, value_kind,
        )?))
    }
}

impl Categorize<ScryptoCustomValueKind> for NibblePath {
    #[inline]
    fn value_kind() -> ValueKind<ScryptoCustomValueKind> {
        ValueKind::Tuple
    }
}

impl<E: Encoder<ScryptoCustomValueKind>> Encode<ScryptoCustomValueKind, E> for NibblePath {
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

impl<D: Decoder<ScryptoCustomValueKind>> Decode<ScryptoCustomValueKind, D> for NibblePath {
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<ScryptoCustomValueKind>,
    ) -> Result<Self, DecodeError> {
        let (even, bytes): (bool, Vec<u8>) =
            Decode::<ScryptoCustomValueKind, D>::decode_body_with_value_kind(decoder, value_kind)?;
        let path = if even {
            NibblePath::new_even(bytes)
        } else {
            NibblePath::new_odd(bytes)
        };
        Ok(path)
    }
}
