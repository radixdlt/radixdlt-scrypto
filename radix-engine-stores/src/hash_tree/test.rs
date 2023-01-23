use super::hash_tree_facade::HashTree;
use super::tree_store::MemoryTreeStore;
use super::tree_store::{Nib, Nibs, TreeNodeKey};
use super::types::SPARSE_MERKLE_PLACEHOLDER_HASH;
use radix_engine::model::{KeyValueStoreEntrySubstate, PersistedSubstate};
use radix_engine_interface::api::types::{
    GlobalAddress, KeyValueStoreOffset, RENodeId, SubstateId, SubstateOffset,
};
use radix_engine_interface::crypto::{hash, Hash};
use radix_engine_interface::data::scrypto_encode;
use radix_engine_interface::model::PackageAddress;
use std::collections::HashSet;

#[test]
fn hash_of_next_version_differs_when_value_changed() {
    let mut store = MemoryTreeStore::new();
    let mut tree = HashTree::new(&mut store, 0);

    tree.put_at_next_version(&[(substate_id(1, 2), value_hash(30))]);
    let hash_v1 = tree.get_current_root_hash();

    tree.put_at_next_version(&[(substate_id(1, 2), value_hash(70))]);
    let hash_v2 = tree.get_current_root_hash();

    assert_ne!(hash_v1, hash_v2);
}

#[test]
fn hash_of_next_version_same_when_write_repeated() {
    let mut store = MemoryTreeStore::new();
    let mut tree = HashTree::new(&mut store, 0);

    tree.put_at_next_version(&[
        (substate_id(4, 6), value_hash(30)),
        (substate_id(3, 9), value_hash(40)),
    ]);
    let hash_v1 = tree.get_current_root_hash();

    tree.put_at_next_version(&[(substate_id(4, 6), value_hash(30))]);
    let hash_v2 = tree.get_current_root_hash();

    assert_eq!(hash_v1, hash_v2);
}

#[test]
fn hash_of_next_version_same_when_write_empty() {
    let mut store = MemoryTreeStore::new();
    let mut tree = HashTree::new(&mut store, 0);

    tree.put_at_next_version(&[
        (substate_id(1, 2), value_hash(30)),
        (substate_id(3, 1), value_hash(40)),
    ]);
    let hash_v1 = tree.get_current_root_hash();

    tree.put_at_next_version(&[]);
    let hash_v2 = tree.get_current_root_hash();

    assert_eq!(hash_v1, hash_v2);
}

#[test]
fn hash_of_next_version_differs_when_substate_added() {
    let mut store = MemoryTreeStore::new();
    let mut tree = HashTree::new(&mut store, 0);

    tree.put_at_next_version(&[(substate_id(1, 2), value_hash(30))]);
    let hash_v1 = tree.get_current_root_hash();

    tree.put_at_next_version(&[(substate_id(1, 8), value_hash(30))]);
    let hash_v2 = tree.get_current_root_hash();

    assert_ne!(hash_v1, hash_v2);
}

#[test]
fn hash_of_next_version_differs_when_substate_removed() {
    let mut store = MemoryTreeStore::new();
    let mut tree = HashTree::new(&mut store, 0);

    tree.put_at_next_version(&[
        (substate_id(1, 2), value_hash(30)),
        (substate_id(4, 3), value_hash(20)),
    ]);
    let hash_v1 = tree.get_current_root_hash();

    tree.put_at_next_version(&[(substate_id(1, 2), None)]);
    let hash_v2 = tree.get_current_root_hash();

    assert_ne!(hash_v1, hash_v2);
}

#[test]
fn hash_returns_to_same_when_previous_state_restored() {
    let mut store = MemoryTreeStore::new();
    let mut tree = HashTree::new(&mut store, 0);

    tree.put_at_next_version(&[
        (substate_id(1, 2), value_hash(30)),
        (substate_id(3, 1), value_hash(40)),
    ]);
    let hash_v1 = tree.get_current_root_hash();

    tree.put_at_next_version(&[
        (substate_id(1, 2), value_hash(90)),
        (substate_id(3, 1), None),
        (substate_id(1, 5), value_hash(10)),
    ]);

    tree.put_at_next_version(&[
        (substate_id(1, 2), value_hash(30)),
        (substate_id(3, 1), value_hash(40)),
        (substate_id(1, 5), None),
    ]);
    let hash_v3 = tree.get_current_root_hash();

    assert_eq!(hash_v1, hash_v3);
}

#[test]
fn hash_differs_when_states_only_differ_by_keys() {
    let mut store_1 = MemoryTreeStore::new();
    let mut tree_1 = HashTree::new(&mut store_1, 0);
    tree_1.put_at_next_version(&[(substate_id(1, 2), value_hash(30))]);
    let hash_1 = tree_1.get_current_root_hash();

    let mut store_2 = MemoryTreeStore::new();
    let mut tree_2 = HashTree::new(&mut store_2, 0);
    tree_2.put_at_next_version(&[(substate_id(1, 3), value_hash(30))]);
    let hash_2 = tree_2.get_current_root_hash();

    assert_ne!(hash_1, hash_2);
}

#[test]
#[should_panic]
fn does_not_support_hash_at_version_0() {
    let mut store = MemoryTreeStore::new();
    let tree = HashTree::new(&mut store, 0);
    tree.get_current_root_hash();
}

#[test]
fn supports_empty_state() {
    let mut store = MemoryTreeStore::new();
    let mut tree = HashTree::new(&mut store, 0);

    tree.put_at_next_version(&[]);
    let hash_v1 = tree.get_current_root_hash();
    assert_eq!(hash_v1, SPARSE_MERKLE_PLACEHOLDER_HASH);

    tree.put_at_next_version(&[(substate_id(1, 2), value_hash(30))]);
    let hash_v2 = tree.get_current_root_hash();
    assert_ne!(hash_v2, SPARSE_MERKLE_PLACEHOLDER_HASH);

    tree.put_at_next_version(&[(substate_id(1, 2), None)]);
    let hash_v3 = tree.get_current_root_hash();
    assert_eq!(hash_v3, SPARSE_MERKLE_PLACEHOLDER_HASH);
}

// Note: this test relies on the impl details of underlying tree structure.
// In particular, it is now assuming a single tree, keyed by hash(substate_id).
// The asserts will need to be adjusted after introducing separate ReNode tree
// vs Substate trees.
#[test]
fn records_stale_tree_node_keys() {
    let mut store = MemoryTreeStore::new();
    let mut tree = HashTree::new(&mut store, 0);

    // the substate_id(4, 6) and substate_id(3, 9) below are deliberately
    // chosen so that their hashes have a common prefix (a nibble 8).
    tree.put_at_next_version(&[(substate_id(4, 6), value_hash(30))]);
    tree.put_at_next_version(&[(substate_id(3, 9), value_hash(70))]);
    tree.put_at_next_version(&[(substate_id(3, 9), value_hash(80))]);
    assert_eq!(
        store.stale_key_buffer.iter().collect::<HashSet<_>>(),
        vec![
            // tree nodes obsoleted by v=2:
            TreeNodeKey {
                // the only node == root == leaf for substate_id(4, 6)
                version: 1,
                nib_prefix: Nibs(vec![])
            },
            // tree nodes obsoleted by v=3:
            TreeNodeKey {
                // the leaf for substate_id(3, 9)
                version: 2,
                nib_prefix: Nibs(vec![Nib(8), Nib(4)])
            },
            TreeNodeKey {
                // the common parent of 2 leaves at v=2
                version: 2,
                nib_prefix: Nibs(vec![Nib(8)])
            },
            TreeNodeKey {
                // the root at v=2
                version: 2,
                nib_prefix: Nibs(vec![])
            },
        ]
        .iter()
        .collect::<HashSet<_>>()
    );
}

#[test]
fn hash_returned_by_put_same_as_queried_directly() {
    let mut store = MemoryTreeStore::new();
    let mut tree = HashTree::new(&mut store, 0);

    let returned = tree.put_at_next_version(&[(substate_id(1, 2), value_hash(30))]);
    let queried = tree.get_current_root_hash();

    assert_eq!(returned, queried);
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
