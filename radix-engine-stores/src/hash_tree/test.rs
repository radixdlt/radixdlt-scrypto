use super::types::{Nibble, NibblePath, SPARSE_MERKLE_PLACEHOLDER_HASH};
use crate::hash_tree::put_at_next_version;
use crate::hash_tree::tree_store::{
    SerializedInMemoryTreeStore, TreeChildEntry, TreeInternalNode, TreeLeafNode, TreeNode,
    TypedInMemoryTreeStore, Version,
};
use itertools::Itertools;
use radix_engine::system::node_substates::PersistedSubstate;
use radix_engine::types::PackageAddress;
use radix_engine_interface::api::component::KeyValueStoreEntrySubstate;
use radix_engine_interface::api::types::{
    Address, KeyValueStoreOffset, NodeModuleId, RENodeId, SubstateId, SubstateOffset,
};
use radix_engine_interface::crypto::{hash, Hash};
use radix_engine_interface::data::{scrypto_decode, scrypto_encode};

#[test]
fn hash_of_next_version_differs_when_value_changed() {
    let mut store = TypedInMemoryTreeStore::new();
    let hash_v1 = put_at_next_version(&mut store, None, &[(substate_id(1, 2), value_hash(30))]);
    let hash_v2 = put_at_next_version(&mut store, Some(1), &[(substate_id(1, 2), value_hash(70))]);
    assert_ne!(hash_v1, hash_v2);
}

#[test]
fn hash_of_next_version_same_when_write_repeated() {
    let mut store = TypedInMemoryTreeStore::new();
    let hash_v1 = put_at_next_version(
        &mut store,
        None,
        &[
            (substate_id(4, 6), value_hash(30)),
            (substate_id(3, 9), value_hash(40)),
        ],
    );
    let hash_v2 = put_at_next_version(&mut store, Some(1), &[(substate_id(4, 6), value_hash(30))]);
    assert_eq!(hash_v1, hash_v2);
}

#[test]
fn hash_of_next_version_same_when_write_empty() {
    let mut store = TypedInMemoryTreeStore::new();
    let hash_v1 = put_at_next_version(
        &mut store,
        None,
        &[
            (substate_id(1, 2), value_hash(30)),
            (substate_id(3, 1), value_hash(40)),
        ],
    );
    let hash_v2 = put_at_next_version(&mut store, Some(1), &[]);
    assert_eq!(hash_v1, hash_v2);
}

#[test]
fn hash_of_next_version_differs_when_substate_added() {
    let mut store = TypedInMemoryTreeStore::new();
    let hash_v1 = put_at_next_version(&mut store, None, &[(substate_id(1, 2), value_hash(30))]);
    let hash_v2 = put_at_next_version(&mut store, Some(1), &[(substate_id(1, 8), value_hash(30))]);
    assert_ne!(hash_v1, hash_v2);
}

#[test]
fn hash_of_next_version_differs_when_substate_removed() {
    let mut store = TypedInMemoryTreeStore::new();
    let hash_v1 = put_at_next_version(
        &mut store,
        None,
        &[
            (substate_id(1, 2), value_hash(30)),
            (substate_id(4, 3), value_hash(20)),
        ],
    );
    let hash_v2 = put_at_next_version(&mut store, Some(1), &[(substate_id(1, 2), None)]);
    assert_ne!(hash_v1, hash_v2);
}

#[test]
fn hash_returns_to_same_when_previous_state_restored() {
    let mut store = TypedInMemoryTreeStore::new();
    let hash_v1 = put_at_next_version(
        &mut store,
        None,
        &[
            (substate_id(1, 2), value_hash(30)),
            (substate_id(3, 1), value_hash(40)),
        ],
    );
    put_at_next_version(
        &mut store,
        Some(1),
        &[
            (substate_id(1, 2), value_hash(90)),
            (substate_id(3, 1), None),
            (substate_id(1, 5), value_hash(10)),
        ],
    );
    let hash_v3 = put_at_next_version(
        &mut store,
        Some(1),
        &[
            (substate_id(1, 2), value_hash(30)),
            (substate_id(3, 1), value_hash(40)),
            (substate_id(1, 5), None),
        ],
    );
    assert_eq!(hash_v1, hash_v3);
}

#[test]
fn hash_differs_when_states_only_differ_by_keys() {
    let mut store_1 = TypedInMemoryTreeStore::new();
    let hash_1 = put_at_next_version(&mut store_1, None, &[(substate_id(1, 2), value_hash(30))]);
    let mut store_2 = TypedInMemoryTreeStore::new();
    let hash_2 = put_at_next_version(&mut store_2, None, &[(substate_id(1, 3), value_hash(30))]);
    assert_ne!(hash_1, hash_2);
}

#[test]
fn supports_empty_state() {
    let mut store = TypedInMemoryTreeStore::new();
    let hash_v1 = put_at_next_version(&mut store, None, &[]);
    assert_eq!(hash_v1, SPARSE_MERKLE_PLACEHOLDER_HASH);
    let hash_v2 = put_at_next_version(&mut store, Some(1), &[(substate_id(1, 2), value_hash(30))]);
    assert_ne!(hash_v2, SPARSE_MERKLE_PLACEHOLDER_HASH);
    let hash_v3 = put_at_next_version(&mut store, Some(2), &[(substate_id(1, 2), None)]);
    assert_eq!(hash_v3, SPARSE_MERKLE_PLACEHOLDER_HASH);
}

#[test]
fn records_stale_tree_node_keys() {
    let mut store = TypedInMemoryTreeStore::new();
    put_at_next_version(&mut store, None, &[(substate_id(4, 6), value_hash(30))]);
    put_at_next_version(&mut store, Some(1), &[(substate_id(3, 9), value_hash(70))]);
    put_at_next_version(&mut store, Some(2), &[(substate_id(3, 9), value_hash(80))]);
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
        substate_id: substate_id(13, 37),
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
            substate_id: substate_id(13, 37),
            value_hash: Hash([7; 32]),
        }),
        TreeNode::Null,
    ];
    let encoded = scrypto_encode(&nodes).unwrap();
    let decoded = scrypto_decode::<Vec<TreeNode>>(&encoded).unwrap();
    assert_eq!(nodes, decoded);
}

#[test]
fn serialized_keys_are_strictly_increasing() {
    let mut store = SerializedInMemoryTreeStore::new();
    put_at_next_version(&mut store, None, &[(substate_id(3, 4), value_hash(90))]);
    let previous_key = store.memory.keys().collect_vec()[0].clone();
    put_at_next_version(&mut store, Some(1), &[(substate_id(1, 2), value_hash(80))]);
    let next_key = store
        .memory
        .keys()
        .filter(|key| **key != previous_key)
        .collect_vec()[0]
        .clone();
    assert!(next_key > previous_key);
}

fn substate_id(re_node_id_seed: u8, substate_offset_seed: u8) -> SubstateId {
    let fake_pkg_address = PackageAddress::Normal([re_node_id_seed; 26]);
    let fake_kvs_entry_id = vec![substate_offset_seed; substate_offset_seed as usize];
    SubstateId(
        RENodeId::Global(Address::Package(fake_pkg_address)),
        NodeModuleId::SELF,
        SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(fake_kvs_entry_id)),
    )
}

fn value_hash(value_seed: u8) -> Option<Hash> {
    let fake_kvs_key = scrypto_encode(&vec![value_seed; value_seed as usize]).unwrap();
    let fake_kvs_value = scrypto_encode(&vec![value_seed; value_seed as usize]).unwrap();
    let fake_kvs_entry = PersistedSubstate::KeyValueStoreEntry(KeyValueStoreEntrySubstate::Some(
        scrypto_decode(&fake_kvs_key).unwrap(),
        scrypto_decode(&fake_kvs_value).unwrap(),
    ));
    Some(hash(scrypto_encode(&fake_kvs_entry).unwrap()))
}

fn nibbles(hex_string: &str) -> NibblePath {
    NibblePath::from_iter(
        hex_string
            .chars()
            .map(|nibble| Nibble::from(char::to_digit(nibble, 16).unwrap() as u8)),
    )
}
