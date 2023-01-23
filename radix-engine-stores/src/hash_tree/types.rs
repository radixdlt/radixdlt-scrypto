// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

//! This file contains copy-pasted excerpts from a couple of Aptos source files
//! which were used by the Aptos JMT implementation (see jellyfish.rs), with a
//! couple of manual adjustments (aimed at reducing further dependency on Aptos).
//! Each copied fragment starts with a "COPY-PASTED: from ..." comment.
//! Each non-trivial adjustment is annotated with an "ADJUSTMENT: ..." comment.

use itertools::Itertools;
use radix_engine_interface::crypto::{hash, Hash};
use std::collections::hash_map::HashMap;
use std::io::{Error, ErrorKind};
use std::{fmt, iter::FromIterator};

// COPY-PASTED: from https://github.com/aptos-labs/aptos-core/blob/f279d68ba1f69136166fc2e6789b98a4fc9e8949/types/src/proof/definition.rs#L182
/// A more detailed version of `SparseMerkleProof` with the only difference that all the leaf
/// siblings are explicitly set as `SparseMerkleLeafNode` instead of its hash value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SparseMerkleProofExt {
    leaf: Option<SparseMerkleLeafNode>,
    /// All siblings in this proof, including the default ones. Siblings are ordered from the bottom
    /// level to the root level.
    siblings: Vec<NodeInProof>,
}

impl SparseMerkleProofExt {
    /// Constructs a new `SparseMerkleProofExt` using leaf and a list of sibling nodes.
    pub fn new(leaf: Option<SparseMerkleLeafNode>, siblings: Vec<NodeInProof>) -> Self {
        Self { leaf, siblings }
    }

    /// Returns the leaf node in this proof.
    pub fn leaf(&self) -> Option<SparseMerkleLeafNode> {
        self.leaf
    }

    /// Returns the list of siblings in this proof.
    pub fn siblings(&self) -> &[NodeInProof] {
        &self.siblings
    }
}

impl From<SparseMerkleProofExt> for SparseMerkleProof {
    fn from(proof_ext: SparseMerkleProofExt) -> Self {
        Self::new(
            proof_ext.leaf,
            proof_ext
                .siblings
                .into_iter()
                .map(|node| node.hash())
                .collect(),
        )
    }
}

// COPY-PASTED: from https://github.com/aptos-labs/aptos-core/blob/f279d68ba1f69136166fc2e6789b98a4fc9e8949/types/src/proof/definition.rs#L135
impl SparseMerkleProof {
    /// Constructs a new `SparseMerkleProof` using leaf and a list of siblings.
    pub fn new(leaf: Option<SparseMerkleLeafNode>, siblings: Vec<Hash>) -> Self {
        SparseMerkleProof { leaf, siblings }
    }

    /// Returns the leaf node in this proof.
    pub fn leaf(&self) -> Option<SparseMerkleLeafNode> {
        self.leaf
    }

    /// Returns the list of siblings in this proof.
    pub fn siblings(&self) -> &[Hash] {
        &self.siblings
    }
}

/// A proof that can be used to authenticate an element in a Sparse Merkle Tree given trusted root
/// hash. For example, `TransactionInfoToAccountProof` can be constructed on top of this structure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SparseMerkleProof {
    /// This proof can be used to authenticate whether a given leaf exists in the tree or not.
    ///     - If this is `Some(leaf_node)`
    ///         - If `leaf_node.key` equals requested key, this is an inclusion proof and
    ///           `leaf_node.value_hash` equals the hash of the corresponding account blob.
    ///         - Otherwise this is a non-inclusion proof. `leaf_node.key` is the only key
    ///           that exists in the subtree and `leaf_node.value_hash` equals the hash of the
    ///           corresponding account blob.
    ///     - If this is `None`, this is also a non-inclusion proof which indicates the subtree is
    ///       empty.
    leaf: Option<SparseMerkleLeafNode>,

    /// All siblings in this proof, including the default ones. Siblings are ordered from the bottom
    /// level to the root level.
    siblings: Vec<Hash>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NodeInProof {
    Leaf(SparseMerkleLeafNode),
    Other(Hash),
}

impl From<Hash> for NodeInProof {
    fn from(hash: Hash) -> Self {
        Self::Other(hash)
    }
}

impl From<SparseMerkleLeafNode> for NodeInProof {
    fn from(leaf: SparseMerkleLeafNode) -> Self {
        Self::Leaf(leaf)
    }
}

impl NodeInProof {
    pub fn hash(&self) -> Hash {
        match self {
            Self::Leaf(leaf) => leaf.hash(),
            Self::Other(hash) => *hash,
        }
    }
}

// COPY-PASTED: from https://github.com/aptos-labs/aptos-core/blob/f279d68ba1f69136166fc2e6789b98a4fc9e8949/types/src/proof/definition.rs#L681
/// Note: this is not a range proof in the sense that a range of nodes is verified!
/// Instead, it verifies the entire left part of the tree up to a known rightmost node.
/// See the description below.
///
/// A proof that can be used to authenticate a range of consecutive leaves, from the leftmost leaf to
/// the rightmost known one, in a sparse Merkle tree. For example, given the following sparse Merkle tree:
///
/// ```text
///                   root
///                  /     \
///                 /       \
///                /         \
///               o           o
///              / \         / \
///             a   o       o   h
///                / \     / \
///               o   d   e   X
///              / \         / \
///             b   c       f   g
/// ```
///
/// if the proof wants show that `[a, b, c, d, e]` exists in the tree, it would need the siblings
/// `X` and `h` on the right.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SparseMerkleRangeProof {
    /// The vector of siblings on the right of the path from root to last leaf. The ones near the
    /// bottom are at the beginning of the vector. In the above example, it's `[X, h]`.
    right_siblings: Vec<Hash>,
}

impl SparseMerkleRangeProof {
    /// Constructs a new `SparseMerkleRangeProof`.
    pub fn new(right_siblings: Vec<Hash>) -> Self {
        Self { right_siblings }
    }

    /// Returns the right siblings.
    pub fn right_siblings(&self) -> &[Hash] {
        &self.right_siblings
    }
}

// COPY-PASTED: from https://github.com/aptos-labs/aptos-core/blob/f279d68ba1f69136166fc2e6789b98a4fc9e8949/types/src/proof/mod.rs#L97
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SparseMerkleLeafNode {
    key: Hash,
    value_hash: Hash,
}

impl SparseMerkleLeafNode {
    pub fn new(key: Hash, value_hash: Hash) -> Self {
        SparseMerkleLeafNode { key, value_hash }
    }

    pub fn key(&self) -> Hash {
        self.key
    }

    pub fn value_hash(&self) -> Hash {
        self.value_hash
    }

    pub fn hash(&self) -> Hash {
        hash([self.key.0, self.value_hash.0].concat())
    }
}

pub struct SparseMerkleInternalNode {
    left_child: Hash,
    right_child: Hash,
}

impl SparseMerkleInternalNode {
    pub fn new(left_child: Hash, right_child: Hash) -> Self {
        Self {
            left_child,
            right_child,
        }
    }

    fn hash(&self) -> Hash {
        hash([self.left_child.0, self.right_child.0].concat())
    }
}

// ADJUSTMENT: we propagate usage of our own `Hash` (instead of Aptos' `HashValue`) to avoid
// copy-pasting entire https://github.com/aptos-labs/aptos-core/blob/f279d68ba1f69136166fc2e6789b98a4fc9e8949/crates/aptos-crypto/src/hash.rs#L125
pub const SPARSE_MERKLE_PLACEHOLDER_HASH: Hash = Hash([0u8; Hash::LENGTH]);

// COPY-PASTE from: https://github.com/aptos-labs/aptos-core/blob/f279d68ba1f69136166fc2e6789b98a4fc9e8949/crates/aptos-crypto/src/hash.rs#L422
/// An iterator over `Hash` that generates one bit for each iteration.
pub struct HashBitIterator<'a> {
    /// The reference to the bytes that represent the `Hash`.
    hash_bytes: &'a [u8],
    pos: std::ops::Range<usize>,
    // invariant hash_bytes.len() == Hash::LENGTH;
    // invariant pos.end == hash_bytes.len() * 8;
}

impl<'a> DoubleEndedIterator for HashBitIterator<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.pos.next_back().map(|x| self.get_bit(x))
    }
}

impl<'a> ExactSizeIterator for HashBitIterator<'a> {}

impl<'a> HashBitIterator<'a> {
    /// Constructs a new `HashBitIterator` using given `Hash`.
    fn new(hash: &'a Hash) -> Self {
        HashBitIterator {
            hash_bytes: hash.as_ref(),
            pos: (0..Hash::LENGTH * 8),
        }
    }

    /// Returns the `index`-th bit in the bytes.
    fn get_bit(&self, index: usize) -> bool {
        let pos = index / 8;
        let bit = 7 - index % 8;
        (self.hash_bytes[pos] >> bit) & 1 != 0
    }
}

impl<'a> Iterator for HashBitIterator<'a> {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        self.pos.next().map(|x| self.get_bit(x))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.pos.size_hint()
    }
}

// ADJUSTMENT: since we use our Hash here, we need it to implement these for it
pub trait IteratedHash {
    fn iter_bits(&self) -> HashBitIterator<'_>;

    fn get_nibble(&self, index: usize) -> Nibble;
}

impl IteratedHash for Hash {
    fn iter_bits(&self) -> HashBitIterator<'_> {
        HashBitIterator::new(self)
    }

    fn get_nibble(&self, index: usize) -> Nibble {
        Nibble::from(if index % 2 == 0 {
            self.0[index / 2] >> 4
        } else {
            self.0[index / 2] & 0x0F
        })
    }
}

// COPY-PASTED: from https://github.com/aptos-labs/aptos-core/blob/f279d68ba1f69136166fc2e6789b98a4fc9e8949/types/src/transaction/mod.rs#L57
pub type Version = u64;

// COPY-PASTED: from https://github.com/aptos-labs/aptos-core/blob/f279d68ba1f69136166fc2e6789b98a4fc9e8949/types/src/nibble/mod.rs#L20
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Nibble(u8);

impl From<u8> for Nibble {
    fn from(nibble: u8) -> Self {
        assert!(nibble < 16, "Nibble out of range: {}", nibble);
        Self(nibble)
    }
}

impl From<Nibble> for u8 {
    fn from(nibble: Nibble) -> Self {
        nibble.0
    }
}

impl fmt::LowerHex for Nibble {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:x}", self.0)
    }
}

// COPY-PASTED: from https://github.com/aptos-labs/aptos-core/blob/f279d68ba1f69136166fc2e6789b98a4fc9e8949/types/src/nibble/nibble_path/mod.rs#L22
/// NibblePath defines a path in Merkle tree in the unit of nibble (4 bits).
#[derive(Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct NibblePath {
    /// Indicates the total number of nibbles in bytes. Either `bytes.len() * 2 - 1` or
    /// `bytes.len() * 2`.
    // Guarantees intended ordering based on the top-to-bottom declaration order of the struct's
    // members.
    num_nibbles: usize,
    /// The underlying bytes that stores the path, 2 nibbles per byte. If the number of nibbles is
    /// odd, the second half of the last byte must be 0.
    bytes: Vec<u8>,
}

/// Supports debug format by concatenating nibbles literally. For example, [0x12, 0xa0] with 3
/// nibbles will be printed as "12a".
impl fmt::Debug for NibblePath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.nibbles().try_for_each(|x| write!(f, "{:x}", x))
    }
}

// ADJUSTMENT: just to show it in errors
impl fmt::Display for NibblePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let hex_string = self
            .bytes
            .iter()
            .flat_map(|b| vec![b >> 4, b & 15])
            .map(|b| char::from_digit(b as u32, 16).unwrap())
            .take(self.num_nibbles)
            .collect::<String>();
        write!(f, "{}", hex_string)
    }
}

/// Convert a vector of bytes into `NibblePath` using the lower 4 bits of each byte as nibble.
impl FromIterator<Nibble> for NibblePath {
    fn from_iter<I: IntoIterator<Item = Nibble>>(iter: I) -> Self {
        let mut nibble_path = NibblePath::new_even(vec![]);
        for nibble in iter {
            nibble_path.push(nibble);
        }
        nibble_path
    }
}

impl NibblePath {
    /// Creates a new `NibblePath` from a vector of bytes assuming each byte has 2 nibbles.
    pub fn new_even(bytes: Vec<u8>) -> Self {
        let num_nibbles = bytes.len() * 2;
        NibblePath { num_nibbles, bytes }
    }

    /// Similar to `new()` but asserts that the bytes have one less nibble.
    pub fn new_odd(bytes: Vec<u8>) -> Self {
        assert_eq!(
            bytes.last().expect("Should have odd number of nibbles.") & 0x0F,
            0,
            "Last nibble must be 0."
        );
        let num_nibbles = bytes.len() * 2 - 1;
        NibblePath { num_nibbles, bytes }
    }

    /// Adds a nibble to the end of the nibble path.
    pub fn push(&mut self, nibble: Nibble) {
        if self.num_nibbles % 2 == 0 {
            self.bytes.push(u8::from(nibble) << 4);
        } else {
            self.bytes[self.num_nibbles / 2] |= u8::from(nibble);
        }
        self.num_nibbles += 1;
    }

    /// Pops a nibble from the end of the nibble path.
    pub fn pop(&mut self) -> Option<Nibble> {
        let poped_nibble = if self.num_nibbles % 2 == 0 {
            self.bytes.last_mut().map(|last_byte| {
                let nibble = *last_byte & 0x0F;
                *last_byte &= 0xF0;
                Nibble::from(nibble)
            })
        } else {
            self.bytes.pop().map(|byte| Nibble::from(byte >> 4))
        };
        if poped_nibble.is_some() {
            self.num_nibbles -= 1;
        }
        poped_nibble
    }

    /// Returns the last nibble.
    pub fn last(&self) -> Option<Nibble> {
        let last_byte_option = self.bytes.last();
        if self.num_nibbles % 2 == 0 {
            last_byte_option.map(|last_byte| Nibble::from(*last_byte & 0x0F))
        } else {
            let last_byte = last_byte_option.expect("Last byte must exist if num_nibbles is odd.");
            Some(Nibble::from(*last_byte >> 4))
        }
    }

    /// Get the i-th bit.
    fn get_bit(&self, i: usize) -> bool {
        assert!(i < self.num_nibbles * 4);
        let pos = i / 8;
        let bit = 7 - i % 8;
        ((self.bytes[pos] >> bit) & 1) != 0
    }

    /// Get the i-th nibble.
    pub fn get_nibble(&self, i: usize) -> Nibble {
        assert!(i < self.num_nibbles);
        Nibble::from((self.bytes[i / 2] >> (if i % 2 == 1 { 0 } else { 4 })) & 0xF)
    }

    /// Get a bit iterator iterates over the whole nibble path.
    pub fn bits(&self) -> BitIterator {
        BitIterator {
            nibble_path: self,
            pos: (0..self.num_nibbles * 4),
        }
    }

    /// Get a nibble iterator iterates over the whole nibble path.
    pub fn nibbles(&self) -> NibbleIterator {
        NibbleIterator::new(self, 0, self.num_nibbles)
    }

    /// Get the total number of nibbles stored.
    pub fn num_nibbles(&self) -> usize {
        self.num_nibbles
    }

    ///  Returns `true` if the nibbles contains no elements.
    pub fn is_empty(&self) -> bool {
        self.num_nibbles() == 0
    }

    /// Get the underlying bytes storing nibbles.
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn truncate(&mut self, len: usize) {
        assert!(len <= self.num_nibbles);
        self.num_nibbles = len;
        self.bytes.truncate((len + 1) / 2);
        if len % 2 != 0 {
            *self.bytes.last_mut().expect("must exist.") &= 0xF0;
        }
    }
}

pub trait Peekable: Iterator {
    /// Returns the `next()` value without advancing the iterator.
    fn peek(&self) -> Option<Self::Item>;
}

/// BitIterator iterates a nibble path by bit.
pub struct BitIterator<'a> {
    nibble_path: &'a NibblePath,
    pos: std::ops::Range<usize>,
}

impl<'a> Peekable for BitIterator<'a> {
    /// Returns the `next()` value without advancing the iterator.
    fn peek(&self) -> Option<Self::Item> {
        if self.pos.start < self.pos.end {
            Some(self.nibble_path.get_bit(self.pos.start))
        } else {
            None
        }
    }
}

/// BitIterator spits out a boolean each time. True/false denotes 1/0.
impl<'a> Iterator for BitIterator<'a> {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        self.pos.next().map(|i| self.nibble_path.get_bit(i))
    }
}

/// Support iterating bits in reversed order.
impl<'a> DoubleEndedIterator for BitIterator<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.pos.next_back().map(|i| self.nibble_path.get_bit(i))
    }
}

/// NibbleIterator iterates a nibble path by nibble.
#[derive(Debug)]
pub struct NibbleIterator<'a> {
    /// The underlying nibble path that stores the nibbles
    nibble_path: &'a NibblePath,

    /// The current index, `pos.start`, will bump by 1 after calling `next()` until `pos.start ==
    /// pos.end`.
    pos: std::ops::Range<usize>,

    /// The start index of the iterator. At the beginning, `pos.start == start`. [start, pos.end)
    /// defines the range of `nibble_path` this iterator iterates over. `nibble_path` refers to
    /// the entire underlying buffer but the range may only be partial.
    start: usize,
    // invariant self.start <= self.pos.start;
    // invariant self.pos.start <= self.pos.end;
}

/// NibbleIterator spits out a byte each time. Each byte must be in range [0, 16).
impl<'a> Iterator for NibbleIterator<'a> {
    type Item = Nibble;

    fn next(&mut self) -> Option<Self::Item> {
        self.pos.next().map(|i| self.nibble_path.get_nibble(i))
    }
}

impl<'a> Peekable for NibbleIterator<'a> {
    /// Returns the `next()` value without advancing the iterator.
    fn peek(&self) -> Option<Self::Item> {
        if self.pos.start < self.pos.end {
            Some(self.nibble_path.get_nibble(self.pos.start))
        } else {
            None
        }
    }
}

impl<'a> NibbleIterator<'a> {
    fn new(nibble_path: &'a NibblePath, start: usize, end: usize) -> Self {
        assert!(start <= end);
        Self {
            nibble_path,
            pos: (start..end),
            start,
        }
    }

    /// Returns a nibble iterator that iterates all visited nibbles.
    pub fn visited_nibbles(&self) -> NibbleIterator<'a> {
        Self::new(self.nibble_path, self.start, self.pos.start)
    }

    /// Returns a nibble iterator that iterates all remaining nibbles.
    pub fn remaining_nibbles(&self) -> NibbleIterator<'a> {
        Self::new(self.nibble_path, self.pos.start, self.pos.end)
    }

    /// Turn it into a `BitIterator`.
    pub fn bits(&self) -> BitIterator<'a> {
        BitIterator {
            nibble_path: self.nibble_path,
            pos: (self.pos.start * 4..self.pos.end * 4),
        }
    }

    /// Cut and return the range of the underlying `nibble_path` that this iterator is iterating
    /// over as a new `NibblePath`
    pub fn get_nibble_path(&self) -> NibblePath {
        self.visited_nibbles()
            .chain(self.remaining_nibbles())
            .collect()
    }

    /// Get the number of nibbles that this iterator covers.
    pub fn num_nibbles(&self) -> usize {
        assert!(self.start <= self.pos.end); // invariant
        self.pos.end - self.start
    }

    /// Return `true` if the iteration is over.
    pub fn is_finished(&self) -> bool {
        self.peek().is_none()
    }
}

// COPY-PASTED: from https://github.com/aptos-labs/aptos-core/blob/f279d68ba1f69136166fc2e6789b98a4fc9e8949/storage/jellyfish-merkle/src/node_type/mod.rs#L48
/// The unique key of each node.
#[derive(Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct NodeKey {
    // The version at which the node is created.
    version: Version,
    // The nibble path this node represents in the tree.
    nibble_path: NibblePath,
}

impl NodeKey {
    /// Creates a new `NodeKey`.
    pub fn new(version: Version, nibble_path: NibblePath) -> Self {
        Self {
            version,
            nibble_path,
        }
    }

    /// A shortcut to generate a node key consisting of a version and an empty nibble path.
    pub fn new_empty_path(version: Version) -> Self {
        Self::new(version, NibblePath::new_even(vec![]))
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

// ADJUSTMENT: just to show it in errors
impl fmt::Display for NodeKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "v{}:{}", self.version, self.nibble_path)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NodeType {
    Leaf,
    Null,
    /// A internal node that haven't been finished the leaf count migration, i.e. None or not all
    /// of the children leaf counts are known.
    Internal {
        leaf_count: usize,
    },
}

/// Each child of [`InternalNode`] encapsulates a nibble forking at this node.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Child {
    /// The hash value of this child node.
    pub hash: Hash,
    /// `version`, the `nibble_path` of the [`NodeKey`] of this [`InternalNode`] the child belongs
    /// to and the child's index constitute the [`NodeKey`] to uniquely identify this child node
    /// from the storage. Used by `[`NodeKey::gen_child_node_key`].
    pub version: Version,
    /// Indicates if the child is a leaf, or if it's an internal node, the total number of leaves
    /// under it (though it can be unknown during migration).
    pub node_type: NodeType,
}

impl Child {
    pub fn new(hash: Hash, version: Version, node_type: NodeType) -> Self {
        Self {
            hash,
            version,
            node_type,
        }
    }

    pub fn is_leaf(&self) -> bool {
        matches!(self.node_type, NodeType::Leaf)
    }

    pub fn leaf_count(&self) -> usize {
        match self.node_type {
            NodeType::Leaf => 1,
            NodeType::Internal { leaf_count } => leaf_count,
            NodeType::Null => unreachable!("Child cannot be Null"),
        }
    }
}

/// [`Children`] is just a collection of children belonging to a [`InternalNode`], indexed from 0 to
/// 15, inclusive.
pub(crate) type Children = HashMap<Nibble, Child>;

/// Represents a 4-level subtree with 16 children at the bottom level. Theoretically, this reduces
/// IOPS to query a tree by 4x since we compress 4 levels in a standard Merkle tree into 1 node.
/// Though we choose the same internal node structure as that of Patricia Merkle tree, the root hash
/// computation logic is similar to a 4-level sparse Merkle tree except for some customizations.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InternalNode {
    /// Up to 16 children.
    children: Children,
    /// Total number of leaves under this internal node
    leaf_count: usize,
}

impl InternalNode {
    /// Creates a new Internal node.
    pub fn new(children: Children) -> Self {
        let leaf_count = children.values().map(Child::leaf_count).sum();
        Self {
            children,
            leaf_count,
        }
    }

    pub fn leaf_count(&self) -> usize {
        self.leaf_count
    }

    pub fn node_type(&self) -> NodeType {
        NodeType::Internal {
            leaf_count: self.leaf_count,
        }
    }

    pub fn hash(&self) -> Hash {
        self.merkle_hash(
            0,  /* start index */
            16, /* the number of leaves in the subtree of which we want the hash of root */
            self.generate_bitmaps(),
        )
    }

    pub fn children_sorted(&self) -> impl Iterator<Item = (&Nibble, &Child)> {
        self.children.iter().sorted_by_key(|(nibble, _)| **nibble)
    }

    /// Gets the `n`-th child.
    pub fn child(&self, n: Nibble) -> Option<&Child> {
        self.children.get(&n)
    }

    /// Generates `existence_bitmap` and `leaf_bitmap` as a pair of `u16`s: child at index `i`
    /// exists if `existence_bitmap[i]` is set; child at index `i` is leaf node if
    /// `leaf_bitmap[i]` is set.
    pub fn generate_bitmaps(&self) -> (u16, u16) {
        let mut existence_bitmap = 0;
        let mut leaf_bitmap = 0;
        for (nibble, child) in self.children.iter() {
            let i = u8::from(*nibble);
            existence_bitmap |= 1u16 << i;
            if child.is_leaf() {
                leaf_bitmap |= 1u16 << i;
            }
        }
        // `leaf_bitmap` must be a subset of `existence_bitmap`.
        assert_eq!(existence_bitmap | leaf_bitmap, existence_bitmap);
        (existence_bitmap, leaf_bitmap)
    }

    /// Given a range [start, start + width), returns the sub-bitmap of that range.
    fn range_bitmaps(start: u8, width: u8, bitmaps: (u16, u16)) -> (u16, u16) {
        assert!(start < 16 && width.count_ones() == 1 && start % width == 0);
        assert!(width <= 16 && (start + width) <= 16);
        // A range with `start == 8` and `width == 4` will generate a mask 0b0000111100000000.
        // use as converting to smaller integer types when 'width == 16'
        let mask = (((1u32 << width) - 1) << start) as u16;
        (bitmaps.0 & mask, bitmaps.1 & mask)
    }

    fn merkle_hash(
        &self,
        start: u8,
        width: u8,
        (existence_bitmap, leaf_bitmap): (u16, u16),
    ) -> Hash {
        // Given a bit [start, 1 << nibble_height], return the value of that range.
        let (range_existence_bitmap, range_leaf_bitmap) =
            Self::range_bitmaps(start, width, (existence_bitmap, leaf_bitmap));
        if range_existence_bitmap == 0 {
            // No child under this subtree
            SPARSE_MERKLE_PLACEHOLDER_HASH
        } else if width == 1 || (range_existence_bitmap.count_ones() == 1 && range_leaf_bitmap != 0)
        {
            // Only 1 leaf child under this subtree or reach the lowest level
            let only_child_index = Nibble::from(range_existence_bitmap.trailing_zeros() as u8);
            self.child(only_child_index)
                .expect("Corrupted internal node: existence_bitmap inconsistent")
                .hash
        } else {
            let left_child = self.merkle_hash(
                start,
                width / 2,
                (range_existence_bitmap, range_leaf_bitmap),
            );
            let right_child = self.merkle_hash(
                start + width / 2,
                width / 2,
                (range_existence_bitmap, range_leaf_bitmap),
            );
            SparseMerkleInternalNode::new(left_child, right_child).hash()
        }
    }

    fn gen_node_in_proof<K: Clone, R: TreeReader<K>>(
        &self,
        start: u8,
        width: u8,
        (existence_bitmap, leaf_bitmap): (u16, u16),
        (tree_reader, node_key): (&R, &NodeKey),
    ) -> Result<NodeInProof, Error> {
        // Given a bit [start, 1 << nibble_height], return the value of that range.
        let (range_existence_bitmap, range_leaf_bitmap) =
            Self::range_bitmaps(start, width, (existence_bitmap, leaf_bitmap));
        Ok(if range_existence_bitmap == 0 {
            // No child under this subtree
            NodeInProof::Other(SPARSE_MERKLE_PLACEHOLDER_HASH)
        } else if width == 1 || (range_existence_bitmap.count_ones() == 1 && range_leaf_bitmap != 0)
        {
            // Only 1 leaf child under this subtree or reach the lowest level
            let only_child_index = Nibble::from(range_existence_bitmap.trailing_zeros() as u8);
            let only_child = self
                .child(only_child_index)
                .expect("Corrupted internal node: existence_bitmap inconsistent");
            if matches!(only_child.node_type, NodeType::Leaf) {
                let only_child_node_key =
                    node_key.gen_child_node_key(only_child.version, only_child_index);
                match tree_reader.get_node(&only_child_node_key)? {
                    Node::Internal(_) => unreachable!(
                        "Corrupted internal node: in-memory leaf child is internal node on disk"
                    ),
                    Node::Leaf(leaf_node) => {
                        NodeInProof::Leaf(SparseMerkleLeafNode::from(leaf_node))
                    }
                    Node::Null => unreachable!("Child cannot be Null"),
                }
            } else {
                NodeInProof::Other(only_child.hash)
            }
        } else {
            let left_child = self.merkle_hash(
                start,
                width / 2,
                (range_existence_bitmap, range_leaf_bitmap),
            );
            let right_child = self.merkle_hash(
                start + width / 2,
                width / 2,
                (range_existence_bitmap, range_leaf_bitmap),
            );
            NodeInProof::Other(SparseMerkleInternalNode::new(left_child, right_child).hash())
        })
    }

    /// Gets the child and its corresponding siblings that are necessary to generate the proof for
    /// the `n`-th child. If it is an existence proof, the returned child must be the `n`-th
    /// child; otherwise, the returned child may be another child. See inline explanation for
    /// details. When calling this function with n = 11 (node `b` in the following graph), the
    /// range at each level is illustrated as a pair of square brackets:
    ///
    /// ```text
    ///     4      [f   e   d   c   b   a   9   8   7   6   5   4   3   2   1   0] -> root level
    ///            ---------------------------------------------------------------
    ///     3      [f   e   d   c   b   a   9   8] [7   6   5   4   3   2   1   0] width = 8
    ///                                  chs <--┘                        shs <--┘
    ///     2      [f   e   d   c] [b   a   9   8] [7   6   5   4] [3   2   1   0] width = 4
    ///                  shs <--┘               └--> chs
    ///     1      [f   e] [d   c] [b   a] [9   8] [7   6] [5   4] [3   2] [1   0] width = 2
    ///                          chs <--┘       └--> shs
    ///     0      [f] [e] [d] [c] [b] [a] [9] [8] [7] [6] [5] [4] [3] [2] [1] [0] width = 1
    ///     ^                chs <--┘   └--> shs
    ///     |   MSB|<---------------------- uint 16 ---------------------------->|LSB
    ///  height    chs: `child_half_start`         shs: `sibling_half_start`
    /// ```
    pub fn get_child_with_siblings<K: Clone, R: TreeReader<K>>(
        &self,
        node_key: &NodeKey,
        n: Nibble,
        reader: Option<&R>,
    ) -> Result<(Option<NodeKey>, Vec<NodeInProof>), Error> {
        let mut siblings = vec![];
        let (existence_bitmap, leaf_bitmap) = self.generate_bitmaps();

        // Nibble height from 3 to 0.
        for h in (0..4).rev() {
            // Get the number of children of the internal node that each subtree at this height
            // covers.
            let width = 1 << h;
            let (child_half_start, sibling_half_start) = get_child_and_sibling_half_start(n, h);
            // Compute the root hash of the subtree rooted at the sibling of `r`.
            if let Some(reader) = reader {
                siblings.push(self.gen_node_in_proof(
                    sibling_half_start,
                    width,
                    (existence_bitmap, leaf_bitmap),
                    (reader, node_key),
                )?);
            } else {
                siblings.push(
                    self.merkle_hash(sibling_half_start, width, (existence_bitmap, leaf_bitmap))
                        .into(),
                );
            }

            let (range_existence_bitmap, range_leaf_bitmap) =
                Self::range_bitmaps(child_half_start, width, (existence_bitmap, leaf_bitmap));

            if range_existence_bitmap == 0 {
                // No child in this range.
                return Ok((None, siblings));
            } else if width == 1
                || (range_existence_bitmap.count_ones() == 1 && range_leaf_bitmap != 0)
            {
                // Return the only 1 leaf child under this subtree or reach the lowest level
                // Even this leaf child is not the n-th child, it should be returned instead of
                // `None` because it's existence indirectly proves the n-th child doesn't exist.
                // Please read proof format for details.
                let only_child_index = Nibble::from(range_existence_bitmap.trailing_zeros() as u8);
                return Ok((
                    {
                        let only_child_version = self
                            .child(only_child_index)
                            // Should be guaranteed by the self invariants, but these are not easy to express at the moment
                            .expect("Corrupted internal node: child_bitmap inconsistent")
                            .version;
                        Some(node_key.gen_child_node_key(only_child_version, only_child_index))
                    },
                    siblings,
                ));
            }
        }
        unreachable!("Impossible to get here without returning even at the lowest level.")
    }
}

/// Given a nibble, computes the start position of its `child_half_start` and `sibling_half_start`
/// at `height` level.
pub(crate) fn get_child_and_sibling_half_start(n: Nibble, height: u8) -> (u8, u8) {
    // Get the index of the first child belonging to the same subtree whose root, let's say `r` is
    // at `height` that the n-th child belongs to.
    // Note: `child_half_start` will be always equal to `n` at height 0.
    let child_half_start = (0xFF << height) & u8::from(n);

    // Get the index of the first child belonging to the subtree whose root is the sibling of `r`
    // at `height`.
    let sibling_half_start = child_half_start ^ (1 << height);

    (child_half_start, sibling_half_start)
}

/// Represents an account.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LeafNode<K> {
    // The hashed key associated with this leaf node.
    account_key: Hash,
    // The hash of the value.
    value_hash: Hash,
    // The key and version that points to the value
    value_index: (K, Version),
}

impl<K: Clone> LeafNode<K> {
    /// Creates a new leaf node.
    pub fn new(account_key: Hash, value_hash: Hash, value_index: (K, Version)) -> Self {
        Self {
            account_key,
            value_hash,
            value_index,
        }
    }

    /// Gets the account key, the hashed account address.
    pub fn account_key(&self) -> Hash {
        self.account_key
    }

    /// Gets the associated value hash.
    pub fn value_hash(&self) -> Hash {
        self.value_hash
    }

    /// Get the index key to locate the value.
    pub fn value_index(&self) -> &(K, Version) {
        &self.value_index
    }

    pub fn hash(&self) -> Hash {
        SparseMerkleLeafNode::new(self.account_key, self.value_hash).hash()
    }
}

impl<K> From<LeafNode<K>> for SparseMerkleLeafNode {
    fn from(leaf_node: LeafNode<K>) -> Self {
        Self::new(leaf_node.account_key, leaf_node.value_hash)
    }
}

/// The concrete node type of [`JellyfishMerkleTree`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Node<K> {
    /// A wrapper of [`InternalNode`].
    Internal(InternalNode),
    /// A wrapper of [`LeafNode`].
    Leaf(LeafNode<K>),
    /// Represents empty tree only
    Null,
}

impl<K> From<InternalNode> for Node<K> {
    fn from(node: InternalNode) -> Self {
        Node::Internal(node)
    }
}

impl From<InternalNode> for Children {
    fn from(node: InternalNode) -> Self {
        node.children
    }
}

impl<K: Clone> From<LeafNode<K>> for Node<K> {
    fn from(node: LeafNode<K>) -> Self {
        Node::Leaf(node)
    }
}

impl<K: Clone> Node<K> {
    /// Creates the [`Internal`](Node::Internal) variant.
    #[cfg(any(test, feature = "fuzzing"))]
    pub fn new_internal(children: Children) -> Self {
        Node::Internal(InternalNode::new(children))
    }

    /// Creates the [`Leaf`](Node::Leaf) variant.
    pub fn new_leaf(account_key: Hash, value_hash: Hash, value_index: (K, Version)) -> Self {
        Node::Leaf(LeafNode::new(account_key, value_hash, value_index))
    }

    /// Returns `true` if the node is a leaf node.
    pub fn is_leaf(&self) -> bool {
        matches!(self, Node::Leaf(_))
    }

    /// Returns `NodeType`
    pub fn node_type(&self) -> NodeType {
        match self {
            // The returning value will be used to construct a `Child` of a internal node, while an
            // internal node will never have a child of Node::Null.
            Self::Leaf(_) => NodeType::Leaf,
            Self::Internal(n) => n.node_type(),
            Self::Null => NodeType::Null,
        }
    }

    /// Returns leaf count if known
    pub fn leaf_count(&self) -> usize {
        match self {
            Node::Leaf(_) => 1,
            Node::Internal(internal_node) => internal_node.leaf_count,
            Node::Null => 0,
        }
    }

    /// Computes the hash of nodes.
    pub fn hash(&self) -> Hash {
        match self {
            Node::Internal(internal_node) => internal_node.hash(),
            Node::Leaf(leaf_node) => leaf_node.hash(),
            Node::Null => SPARSE_MERKLE_PLACEHOLDER_HASH,
        }
    }
}

// COPY-PASTED: from https://github.com/aptos-labs/aptos-core/blob/a665c55ac939369e937b211a6616283d9ba90ba8/storage/jellyfish-merkle/src/lib.rs#L129
pub trait TreeReader<K> {
    /// Gets node given a node key. Returns error if the node does not exist.
    fn get_node(&self, node_key: &NodeKey) -> Result<Node<K>, Error> {
        self.get_node_option(node_key)?
            .ok_or_else(|| Error::new(ErrorKind::NotFound, format!("inexistent node {}", node_key)))
    }

    /// Gets node given a node key. Returns `None` if the node does not exist.
    fn get_node_option(&self, node_key: &NodeKey) -> Result<Option<Node<K>>, Error>;
}
