use super::tree_store::MemoryTreeStore;
use super::types::{Nibble, NibblePath, NodeKey, SPARSE_MERKLE_PLACEHOLDER_HASH};
use crate::hash_tree::put_at_next_version;
use radix_engine::model::{KeyValueStoreEntrySubstate, PersistedSubstate};
use radix_engine_interface::api::types::{
    GlobalAddress, KeyValueStoreOffset, RENodeId, SubstateId, SubstateOffset,
};
use radix_engine_interface::crypto::{hash, Hash};
use radix_engine_interface::data::scrypto_encode;
use radix_engine_interface::model::PackageAddress;
use sbor::rust::collections::HashSet;

#[test]
fn hash_of_next_version_differs_when_value_changed() {
    let mut store = MemoryTreeStore::new();
    let hash_v1 = put_at_next_version(&mut store, None, &[(substate_id(1, 2), value_hash(30))]);
    let hash_v2 = put_at_next_version(&mut store, Some(1), &[(substate_id(1, 2), value_hash(70))]);
    assert_ne!(hash_v1, hash_v2);
}

#[test]
fn hash_of_next_version_same_when_write_repeated() {
    let mut store = MemoryTreeStore::new();
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
    let mut store = MemoryTreeStore::new();
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
    let mut store = MemoryTreeStore::new();
    let hash_v1 = put_at_next_version(&mut store, None, &[(substate_id(1, 2), value_hash(30))]);
    let hash_v2 = put_at_next_version(&mut store, Some(1), &[(substate_id(1, 8), value_hash(30))]);
    assert_ne!(hash_v1, hash_v2);
}

#[test]
fn hash_of_next_version_differs_when_substate_removed() {
    let mut store = MemoryTreeStore::new();
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
    let mut store = MemoryTreeStore::new();
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
    let mut store_1 = MemoryTreeStore::new();
    let hash_1 = put_at_next_version(&mut store_1, None, &[(substate_id(1, 2), value_hash(30))]);
    let mut store_2 = MemoryTreeStore::new();
    let hash_2 = put_at_next_version(&mut store_2, None, &[(substate_id(1, 3), value_hash(30))]);
    assert_ne!(hash_1, hash_2);
}

#[test]
fn supports_empty_state() {
    let mut store = MemoryTreeStore::new();
    let hash_v1 = put_at_next_version(&mut store, None, &[]);
    assert_eq!(hash_v1, SPARSE_MERKLE_PLACEHOLDER_HASH);
    let hash_v2 = put_at_next_version(&mut store, Some(1), &[(substate_id(1, 2), value_hash(30))]);
    assert_ne!(hash_v2, SPARSE_MERKLE_PLACEHOLDER_HASH);
    let hash_v3 = put_at_next_version(&mut store, Some(2), &[(substate_id(1, 2), None)]);
    assert_eq!(hash_v3, SPARSE_MERKLE_PLACEHOLDER_HASH);
}

// Note: this test relies on the impl details of underlying tree structure.
// In particular, it is now assuming a single tree, keyed by hash(substate_id).
// The asserts will need to be adjusted after introducing separate ReNode tree
// vs Substate trees.
#[test]
fn records_stale_tree_node_keys() {
    let mut store = MemoryTreeStore::new();
    // the substate_id(4, 6) and substate_id(3, 9) below are deliberately
    // chosen so that their hashes have a common prefix (a nibble 8).
    put_at_next_version(&mut store, None, &[(substate_id(4, 6), value_hash(30))]);
    put_at_next_version(&mut store, Some(1), &[(substate_id(3, 9), value_hash(70))]);
    put_at_next_version(&mut store, Some(2), &[(substate_id(3, 9), value_hash(80))]);
    assert_eq!(
        store.stale_key_buffer.iter().collect::<HashSet<_>>(),
        vec![
            // tree nodes obsoleted by v=2:
            // the only node == root == leaf for substate_id(4, 6)
            NodeKey::new(1, nibbles("")),
            // tree nodes obsoleted by v=3:
            // the leaf for substate_id(3, 9)
            NodeKey::new(2, nibbles("84")),
            // the common parent of 2 leaves at v=2
            NodeKey::new(2, nibbles("8")),
            // the root at v=2
            NodeKey::new(2, nibbles("")),
        ]
        .iter()
        .collect::<HashSet<_>>()
    );
}

fn substate_id(re_node_id_seed: u8, substate_offset_seed: u8) -> SubstateId {
    let fake_pkg_address = PackageAddress::Normal([re_node_id_seed; 26]);
    let fake_kvs_key = vec![substate_offset_seed; substate_offset_seed as usize];
    SubstateId(
        RENodeId::Global(GlobalAddress::Package(fake_pkg_address)),
        SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(fake_kvs_key)),
    )
}

fn value_hash(value_seed: u8) -> Option<Hash> {
    let fake_kvs_value = Some(vec![value_seed; value_seed as usize]);
    let fake_kvs_entry =
        PersistedSubstate::KeyValueStoreEntry(KeyValueStoreEntrySubstate(fake_kvs_value));
    Some(hash(scrypto_encode(&fake_kvs_entry).unwrap()))
}

fn nibbles(hex_string: &str) -> NibblePath {
    NibblePath::from_iter(
        hex_string
            .chars()
            .map(|nibble| Nibble::from(char::to_digit(nibble, 16).unwrap() as u8)),
    )
}
