use super::types::{Nibble, NibblePath, Version, SPARSE_MERKLE_PLACEHOLDER_HASH};
use crate::hash_tree::tree_store::{
    SerializedInMemoryTreeStore, TreeChildEntry, TreeInternalNode, TreeLeafNode, TreeNode,
    TypedInMemoryTreeStore,
};
use crate::hash_tree::{put_at_next_version, SubstateHashChange};
use itertools::Itertools;
use radix_engine_interface::crypto::{hash, Hash};
use radix_engine_interface::data::scrypto::{scrypto_decode, scrypto_encode};
use radix_engine_interface::types::{ModuleId, NodeId, PackageAddress, SubstateKey, TypedModuleId};

#[test]
fn hash_of_next_version_differs_when_value_changed() {
    let mut store = TypedInMemoryTreeStore::new();
    let hash_v1 = put_at_next_version(
        &mut store,
        None,
        vec![change(1, TypedModuleId::ObjectState, 2, Some(30))],
    );
    let hash_v2 = put_at_next_version(
        &mut store,
        Some(1),
        vec![change(1, TypedModuleId::ObjectState, 2, Some(70))],
    );
    assert_ne!(hash_v1, hash_v2);
}

#[test]
fn hash_of_next_version_same_when_write_repeated() {
    let mut store = TypedInMemoryTreeStore::new();
    let hash_v1 = put_at_next_version(
        &mut store,
        None,
        vec![
            change(4, TypedModuleId::ObjectState, 6, Some(30)),
            change(3, TypedModuleId::ObjectState, 9, Some(40)),
        ],
    );
    let hash_v2 = put_at_next_version(
        &mut store,
        Some(1),
        vec![change(4, TypedModuleId::ObjectState, 6, Some(30))],
    );
    assert_eq!(hash_v1, hash_v2);
}

#[test]
fn hash_of_next_version_same_when_write_empty() {
    let mut store = TypedInMemoryTreeStore::new();
    let hash_v1 = put_at_next_version(
        &mut store,
        None,
        vec![
            change(1, TypedModuleId::ObjectState, 2, Some(30)),
            change(3, TypedModuleId::ObjectState, 1, Some(40)),
        ],
    );
    let hash_v2 = put_at_next_version(&mut store, Some(1), vec![]);
    assert_eq!(hash_v1, hash_v2);
}

#[test]
fn hash_of_next_version_differs_when_substate_added() {
    let mut store = TypedInMemoryTreeStore::new();
    let hash_v1 = put_at_next_version(
        &mut store,
        None,
        vec![change(1, TypedModuleId::ObjectState, 2, Some(30))],
    );
    let hash_v2 = put_at_next_version(
        &mut store,
        Some(1),
        vec![change(1, TypedModuleId::ObjectState, 8, Some(30))],
    );
    assert_ne!(hash_v1, hash_v2);
}

#[test]
fn hash_of_next_version_differs_when_substate_removed() {
    let mut store = TypedInMemoryTreeStore::new();
    let hash_v1 = put_at_next_version(
        &mut store,
        None,
        vec![
            change(1, TypedModuleId::ObjectState, 2, Some(30)),
            change(4, TypedModuleId::ObjectState, 3, Some(20)),
        ],
    );
    let hash_v2 = put_at_next_version(
        &mut store,
        Some(1),
        vec![change(1, TypedModuleId::ObjectState, 2, None)],
    );
    assert_ne!(hash_v1, hash_v2);
}

#[test]
fn hash_returns_to_same_when_previous_state_restored() {
    let mut store = TypedInMemoryTreeStore::new();
    let hash_v1 = put_at_next_version(
        &mut store,
        None,
        vec![
            change(1, TypedModuleId::ObjectState, 2, Some(30)),
            change(3, TypedModuleId::ObjectState, 1, Some(40)),
        ],
    );
    put_at_next_version(
        &mut store,
        Some(1),
        vec![
            change(1, TypedModuleId::ObjectState, 2, Some(90)),
            change(3, TypedModuleId::ObjectState, 1, None),
            change(1, TypedModuleId::ObjectState, 5, Some(10)),
        ],
    );
    let hash_v3 = put_at_next_version(
        &mut store,
        Some(1),
        vec![
            change(1, TypedModuleId::ObjectState, 2, Some(30)),
            change(3, TypedModuleId::ObjectState, 1, Some(40)),
            change(1, TypedModuleId::ObjectState, 5, None),
        ],
    );
    assert_eq!(hash_v1, hash_v3);
}

#[test]
fn hash_differs_when_states_only_differ_by_node_id() {
    let mut store_1 = TypedInMemoryTreeStore::new();
    let hash_1 = put_at_next_version(
        &mut store_1,
        None,
        vec![change(1, TypedModuleId::ObjectState, 3, Some(30))],
    );
    let mut store_2 = TypedInMemoryTreeStore::new();
    let hash_2 = put_at_next_version(
        &mut store_2,
        None,
        vec![change(2, TypedModuleId::ObjectState, 3, Some(30))],
    );
    assert_ne!(hash_1, hash_2);
}

#[test]
fn hash_differs_when_states_only_differ_by_module_id() {
    let mut store_1 = TypedInMemoryTreeStore::new();
    let hash_1 = put_at_next_version(
        &mut store_1,
        None,
        vec![change(1, TypedModuleId::Metadata, 3, Some(30))],
    );
    let mut store_2 = TypedInMemoryTreeStore::new();
    let hash_2 = put_at_next_version(
        &mut store_2,
        None,
        vec![change(1, TypedModuleId::AccessRules, 3, Some(30))],
    );
    assert_ne!(hash_1, hash_2);
}

#[test]
fn hash_differs_when_states_only_differ_by_offset() {
    let mut store_1 = TypedInMemoryTreeStore::new();
    let hash_1 = put_at_next_version(
        &mut store_1,
        None,
        vec![change(1, TypedModuleId::ObjectState, 2, Some(30))],
    );
    let mut store_2 = TypedInMemoryTreeStore::new();
    let hash_2 = put_at_next_version(
        &mut store_2,
        None,
        vec![change(1, TypedModuleId::ObjectState, 3, Some(30))],
    );
    assert_ne!(hash_1, hash_2);
}

#[test]
fn hash_of_different_re_node_nested_trees_is_same_when_contained_substates_are_same() {
    let mut store = TypedInMemoryTreeStore::new();
    put_at_next_version(
        &mut store,
        None,
        vec![
            change(1, TypedModuleId::Metadata, 2, Some(30)),
            change(1, TypedModuleId::Metadata, 9, Some(40)),
            change(7, TypedModuleId::AccessRules, 2, Some(30)),
            change(7, TypedModuleId::AccessRules, 9, Some(40)),
        ],
    );

    let nested_tree_hashes = store
        .root_tree_nodes
        .values()
        .filter_map(|node| match node {
            TreeNode::Leaf(TreeLeafNode { value_hash, .. }) => Some(value_hash.clone()),
            _ => None,
        })
        .collect::<Vec<Hash>>();
    assert_eq!(nested_tree_hashes.len(), 2);
    assert_eq!(nested_tree_hashes[0], nested_tree_hashes[1])
}

#[test]
fn deletes_re_node_layer_leaf_when_all_its_substates_deleted() {
    fn count_current_re_node_leafs(store: &TypedInMemoryTreeStore) -> usize {
        store
            .root_tree_nodes
            .iter()
            .filter(|(key, _)| !store.stale_key_buffer.contains(key))
            .filter(|(_, node)| matches!(node, TreeNode::Leaf(TreeLeafNode { .. })))
            .count()
    }

    let mut store = TypedInMemoryTreeStore::new();
    put_at_next_version(
        &mut store,
        None,
        vec![
            change(1, TypedModuleId::Metadata, 2, Some(30)),
            change(1, TypedModuleId::Metadata, 9, Some(40)),
            change(1, TypedModuleId::AccessRules, 3, Some(30)),
        ],
    );
    assert_eq!(count_current_re_node_leafs(&store), 2);
    put_at_next_version(
        &mut store,
        Some(1),
        vec![
            change(1, TypedModuleId::Metadata, 2, None),
            change(1, TypedModuleId::Metadata, 9, None),
        ],
    );
    assert_eq!(count_current_re_node_leafs(&store), 1);
    put_at_next_version(
        &mut store,
        Some(2),
        vec![change(1, TypedModuleId::AccessRules, 3, None)],
    );
    assert_eq!(count_current_re_node_leafs(&store), 0);
}

#[test]
fn supports_empty_state() {
    let mut store = TypedInMemoryTreeStore::new();
    let hash_v1 = put_at_next_version(&mut store, None, vec![]);
    assert_eq!(hash_v1, SPARSE_MERKLE_PLACEHOLDER_HASH);
    let hash_v2 = put_at_next_version(
        &mut store,
        Some(1),
        vec![change(1, TypedModuleId::ObjectState, 2, Some(30))],
    );
    assert_ne!(hash_v2, SPARSE_MERKLE_PLACEHOLDER_HASH);
    let hash_v3 = put_at_next_version(
        &mut store,
        Some(2),
        vec![change(1, TypedModuleId::ObjectState, 2, None)],
    );
    assert_eq!(hash_v3, SPARSE_MERKLE_PLACEHOLDER_HASH);
}

#[test]
fn records_stale_tree_node_keys() {
    let mut store = TypedInMemoryTreeStore::new();
    put_at_next_version(
        &mut store,
        None,
        vec![change(4, TypedModuleId::ObjectState, 6, Some(30))],
    );
    put_at_next_version(
        &mut store,
        Some(1),
        vec![change(3, TypedModuleId::ObjectState, 9, Some(70))],
    );
    put_at_next_version(
        &mut store,
        Some(2),
        vec![change(3, TypedModuleId::ObjectState, 9, Some(80))],
    );
    let stale_versions = store
        .stale_key_buffer
        .iter()
        .map(|key| key.version())
        .unique()
        .sorted()
        .collect::<Vec<Version>>();
    assert_eq!(stale_versions, vec![1, 2]);
}

#[test]
fn sbor_uses_custom_direct_codecs_for_nibbles() {
    let nibbles = nibbles("a1a2a3");
    let direct_bytes = nibbles.bytes().to_vec();
    let node = TreeNode::Leaf(TreeLeafNode {
        key_suffix: nibbles,
        payload: substate_id(13, TypedModuleId::Royalty, 37),
        value_hash: Hash([7; 32]),
    });
    let encoded = scrypto_encode(&node).unwrap();
    assert!(encoded
        .windows(direct_bytes.len())
        .position(|bytes| bytes == direct_bytes)
        .is_some());
}

#[test]
fn sbor_decodes_what_was_encoded() {
    let nodes = vec![
        TreeNode::Internal(TreeInternalNode {
            children: vec![
                TreeChildEntry {
                    nibble: Nibble::from(15),
                    version: 999,
                    hash: Hash([3; 32]),
                    is_leaf: false,
                },
                TreeChildEntry {
                    nibble: Nibble::from(8),
                    version: 2,
                    hash: Hash([254; 32]),
                    is_leaf: true,
                },
            ],
        }),
        TreeNode::Leaf(TreeLeafNode {
            key_suffix: nibbles("abc"),
            payload: substate_id(13, TypedModuleId::ObjectState, 37),
            value_hash: Hash([7; 32]),
        }),
        TreeNode::Null,
    ];
    let encoded = scrypto_encode(&nodes).unwrap();
    let decoded =
        scrypto_decode::<Vec<TreeNode<(NodeId, ModuleId, SubstateKey)>>>(&encoded).unwrap();
    assert_eq!(nodes, decoded);
}

#[test]
fn serialized_keys_are_strictly_increasing() {
    let mut store = SerializedInMemoryTreeStore::new();
    put_at_next_version(
        &mut store,
        None,
        vec![change(3, TypedModuleId::ObjectState, 4, Some(90))],
    );
    let previous_key = store.memory.keys().collect_vec()[0].clone();
    put_at_next_version(
        &mut store,
        Some(1),
        vec![change(1, TypedModuleId::ObjectState, 2, Some(80))],
    );
    let next_key = store
        .memory
        .keys()
        .filter(|key| **key != previous_key)
        .collect_vec()[0]
        .clone();
    assert!(next_key > previous_key);
}

fn change(
    node_id_seed: u8,
    module_id: TypedModuleId,
    substate_offset_seed: u8,
    value_hash_seed: Option<u8>,
) -> SubstateHashChange {
    SubstateHashChange::new(
        substate_id(node_id_seed, module_id, substate_offset_seed),
        value_hash_seed.map(|value_seed| value_hash(value_seed)),
    )
}

fn substate_id(
    node_id_seed: u8,
    module_id: TypedModuleId,
    substate_offset_seed: u8,
) -> (NodeId, ModuleId, SubstateKey) {
    let fake_pkg_address = PackageAddress::new_unchecked([node_id_seed; 27]);
    let fake_kvs_entry_id = vec![substate_offset_seed; substate_offset_seed as usize];
    (
        NodeId(fake_pkg_address.into()),
        ModuleId(module_id as u8),
        SubstateKey::from_vec(fake_kvs_entry_id).unwrap(),
    )
}

fn value_hash(value_seed: u8) -> Hash {
    let fake_kvs_value = scrypto_encode(&vec![value_seed; value_seed as usize]).unwrap();
    hash(fake_kvs_value)
}

fn nibbles(hex_string: &str) -> NibblePath {
    NibblePath::from_iter(
        hex_string
            .chars()
            .map(|nibble| Nibble::from(char::to_digit(nibble, 16).unwrap() as u8)),
    )
}
