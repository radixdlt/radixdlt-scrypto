use super::types::{Nibble, NibblePath, Version, SPARSE_MERKLE_PLACEHOLDER_HASH};
use crate::hash_tree::jellyfish::JellyfishMerkleTree;
use crate::hash_tree::tree_store::{
    SerializedInMemoryTreeStore, StaleTreePart, TreeChildEntry, TreeInternalNode, TreeLeafNode,
    TreeNode, TypedInMemoryTreeStore,
};
use crate::hash_tree::types::{LeafKey, NodeKey};
use crate::hash_tree::{put_at_next_version, BatchChange, HashChange, SubstateHashChange};
use itertools::Itertools;
use radix_engine_common::crypto::{hash, Hash};
use radix_engine_common::data::scrypto::{scrypto_decode, scrypto_encode};
use radix_engine_store_interface::interface::{
    DbNodeKey, DbPartitionKey, DbPartitionNum, DbSortKey,
};
use utils::rust::collections::{hashmap, hashset, HashMap, HashSet};

#[test]
fn hash_of_next_version_differs_when_value_changed() {
    let mut store = TypedInMemoryTreeStore::new();
    let hash_v1 = put_at_next_version(&mut store, None, vec![change(1, 6, 2, Some(30))]);
    let hash_v2 = put_at_next_version(&mut store, Some(1), vec![change(1, 6, 2, Some(70))]);
    assert_ne!(hash_v1, hash_v2);
}

#[test]
fn hash_of_next_version_same_when_write_repeated() {
    let mut store = TypedInMemoryTreeStore::new();
    let hash_v1 = put_at_next_version(
        &mut store,
        None,
        vec![change(4, 1, 6, Some(30)), change(3, 2, 9, Some(40))],
    );
    let hash_v2 = put_at_next_version(&mut store, Some(1), vec![change(4, 1, 6, Some(30))]);
    assert_eq!(hash_v1, hash_v2);
}

#[test]
fn hash_of_next_version_same_when_write_empty() {
    let mut store = TypedInMemoryTreeStore::new();
    let hash_v1 = put_at_next_version(
        &mut store,
        None,
        vec![change(1, 6, 2, Some(30)), change(3, 7, 1, Some(40))],
    );
    let hash_v2 = put_at_next_version(&mut store, Some(1), vec![]);
    assert_eq!(hash_v1, hash_v2);
}

#[test]
fn hash_of_next_version_differs_when_entry_added() {
    let mut store = TypedInMemoryTreeStore::new();
    let hash_v1 = put_at_next_version(&mut store, None, vec![change(1, 6, 2, Some(30))]);
    let hash_v2 = put_at_next_version(&mut store, Some(1), vec![change(1, 6, 8, Some(30))]);
    assert_ne!(hash_v1, hash_v2);
}

#[test]
fn hash_of_next_version_differs_when_entry_removed() {
    let mut store = TypedInMemoryTreeStore::new();
    let hash_v1 = put_at_next_version(
        &mut store,
        None,
        vec![change(1, 6, 2, Some(30)), change(4, 7, 3, Some(20))],
    );
    let hash_v2 = put_at_next_version(&mut store, Some(1), vec![change(1, 6, 2, None)]);
    assert_ne!(hash_v1, hash_v2);
}

#[test]
fn hash_returns_to_same_when_previous_state_restored() {
    let mut store = TypedInMemoryTreeStore::new();
    let hash_v1 = put_at_next_version(
        &mut store,
        None,
        vec![change(1, 6, 2, Some(30)), change(3, 7, 1, Some(40))],
    );
    put_at_next_version(
        &mut store,
        Some(1),
        vec![
            change(1, 6, 2, Some(90)),
            change(3, 7, 1, None),
            change(1, 6, 5, Some(10)),
        ],
    );
    let hash_v3 = put_at_next_version(
        &mut store,
        Some(1),
        vec![
            change(1, 6, 2, Some(30)),
            change(3, 7, 1, Some(40)),
            change(1, 6, 5, None),
        ],
    );
    assert_eq!(hash_v1, hash_v3);
}

#[test]
fn hash_computed_consistently_after_higher_tier_leafs_deleted() {
    // Compute a "reference" hash of state containing simply [2:3:4, 2:3:5].
    let mut reference_store = TypedInMemoryTreeStore::new();
    let reference_root = put_at_next_version(
        &mut reference_store,
        None,
        vec![change(2, 3, 4, Some(234)), change(2, 3, 5, Some(235))],
    );

    // Compute a hash of the same state, at which we arrive after deleting some unrelated NodeId.
    let mut store = TypedInMemoryTreeStore::new();
    put_at_next_version(
        &mut store,
        None,
        vec![
            change(1, 6, 2, Some(162)),
            change(1, 6, 3, Some(163)),
            change(2, 3, 4, Some(234)),
        ],
    );
    put_at_next_version(
        &mut store,
        Some(1),
        vec![change(1, 6, 2, None), change(1, 6, 3, None)],
    );
    let root_after_deletes =
        put_at_next_version(&mut store, Some(2), vec![change(2, 3, 5, Some(235))]);

    // We did [1:6:2, 1:6:3, 2:3:4] - [1:6:2, 1:6:3] + [2:3:5] = [2:3:4, 2:3:5] (i.e. same state).
    assert_eq!(root_after_deletes, reference_root);
}

#[test]
fn hash_computed_consistently_after_adding_higher_tier_sibling() {
    // Compute a "reference" hash of state containing simply [1:9:6, 2:3:4, 2:3:5].
    let mut reference_store = TypedInMemoryTreeStore::new();
    let reference_root = put_at_next_version(
        &mut reference_store,
        None,
        vec![
            change(1, 9, 6, Some(196)),
            change(2, 3, 4, Some(234)),
            change(2, 3, 5, Some(235)),
        ],
    );

    // Compute a hash of the same state, at which we arrive after adding some sibling NodeId.
    let mut store = TypedInMemoryTreeStore::new();
    put_at_next_version(&mut store, None, vec![change(2, 3, 4, Some(234))]);
    put_at_next_version(&mut store, Some(1), vec![change(1, 9, 6, Some(196))]);
    let root_after_deletes =
        put_at_next_version(&mut store, Some(2), vec![change(2, 3, 5, Some(235))]);

    // We did [2:3:4] + [1:9:6] + [2:3:5] = [1:9:6, 2:3:4, 2:3:5] (i.e. same state).
    assert_eq!(root_after_deletes, reference_root);
}

#[test]
fn hash_differs_when_states_only_differ_by_node_key() {
    let mut store_1 = TypedInMemoryTreeStore::new();
    let hash_1 = put_at_next_version(&mut store_1, None, vec![change(1, 6, 3, Some(30))]);
    let mut store_2 = TypedInMemoryTreeStore::new();
    let hash_2 = put_at_next_version(&mut store_2, None, vec![change(2, 6, 3, Some(30))]);
    assert_ne!(hash_1, hash_2);
}

#[test]
fn hash_differs_when_states_only_differ_by_partition_num() {
    let mut store_1 = TypedInMemoryTreeStore::new();
    let hash_1 = put_at_next_version(&mut store_1, None, vec![change(1, 6, 3, Some(30))]);
    let mut store_2 = TypedInMemoryTreeStore::new();
    let hash_2 = put_at_next_version(&mut store_2, None, vec![change(1, 7, 3, Some(30))]);
    assert_ne!(hash_1, hash_2);
}

#[test]
fn hash_differs_when_states_only_differ_by_sort_key() {
    let mut store_1 = TypedInMemoryTreeStore::new();
    let hash_1 = put_at_next_version(&mut store_1, None, vec![change(1, 6, 2, Some(30))]);
    let mut store_2 = TypedInMemoryTreeStore::new();
    let hash_2 = put_at_next_version(&mut store_2, None, vec![change(1, 6, 3, Some(30))]);
    assert_ne!(hash_1, hash_2);
}

#[test]
fn hash_of_different_re_nodes_is_same_when_contained_entries_are_same() {
    let mut store = TypedInMemoryTreeStore::new();
    put_at_next_version(
        &mut store,
        None,
        vec![
            change(1, 6, 2, Some(30)),
            change(1, 7, 9, Some(40)),
            change(7, 6, 2, Some(30)),
            change(7, 7, 9, Some(40)),
        ],
    );

    let re_node_leaf_hashes = get_leafs_of_tier(&store, Tier::ReNode)
        .into_values()
        .collect::<Vec<_>>();
    assert_eq!(re_node_leaf_hashes.len(), 2);
    assert_eq!(re_node_leaf_hashes[0], re_node_leaf_hashes[1])
}

#[test]
fn physical_nodes_of_tiered_jmt_have_expected_keys_and_contents() {
    let mut store = TypedInMemoryTreeStore::new();

    put_at_next_version(
        &mut store,
        None,
        vec![
            HashChange::Single(SubstateHashChange::new(
                (db_partition_key(vec![1, 3, 3, 7], 99), DbSortKey(vec![253])),
                Some(Hash([1; Hash::LENGTH])),
            )),
            HashChange::Single(SubstateHashChange::new(
                (db_partition_key(vec![1, 3, 3, 7], 99), DbSortKey(vec![66])),
                Some(Hash([2; Hash::LENGTH])),
            )),
            HashChange::Single(SubstateHashChange::new(
                (
                    db_partition_key(vec![123, 12, 1, 0], 88),
                    DbSortKey(vec![6, 6, 6]),
                ),
                Some(Hash([3; Hash::LENGTH])),
            )),
            HashChange::Single(SubstateHashChange::new(
                (
                    db_partition_key(vec![123, 12, 1, 0], 88),
                    DbSortKey(vec![6, 6, 7]),
                ),
                Some(Hash([4; Hash::LENGTH])),
            )),
            HashChange::Single(SubstateHashChange::new(
                (
                    db_partition_key(vec![123, 12, 1, 0], 66),
                    DbSortKey(vec![1, 2, 3, 4]),
                ),
                Some(Hash([5; Hash::LENGTH])),
            )),
        ],
    );

    // Assert on the keys of Node-tier leafs:
    let node_tier_leaf_keys = get_leafs_of_tier(&store, Tier::ReNode)
        .into_keys()
        .collect::<HashSet<_>>();
    assert_eq!(
        node_tier_leaf_keys,
        hashset!(
            LeafKey {
                bytes: vec![1, 3, 3, 7]
            },
            LeafKey {
                bytes: vec![123, 12, 1, 0]
            },
        )
    );

    // Assert on the keys of Partition-tier leafs:
    let partition_tier_leaf_keys = get_leafs_of_tier(&store, Tier::Partition)
        .into_keys()
        .collect::<HashSet<_>>();
    assert_eq!(
        partition_tier_leaf_keys,
        hashset!(
            LeafKey {
                bytes: vec![1, 3, 3, 7, TIER_SEPARATOR, 99]
            },
            LeafKey {
                bytes: vec![123, 12, 1, 0, TIER_SEPARATOR, 66]
            },
            LeafKey {
                bytes: vec![123, 12, 1, 0, TIER_SEPARATOR, 88]
            },
        )
    );

    // Assert on the keys and hashes of the Substate-tier leaves:
    let substate_tier_leaves = get_leafs_of_tier(&store, Tier::Substate);
    assert_eq!(
        substate_tier_leaves,
        hashmap!(
            LeafKey { bytes: vec![1, 3, 3, 7, TIER_SEPARATOR, 99, TIER_SEPARATOR, 253] } => Hash([1; Hash::LENGTH]),
            LeafKey { bytes: vec![1, 3, 3, 7, TIER_SEPARATOR, 99, TIER_SEPARATOR, 66] } => Hash([2; Hash::LENGTH]),
            LeafKey { bytes: vec![123, 12, 1, 0, TIER_SEPARATOR, 88, TIER_SEPARATOR, 6, 6, 6] } => Hash([3; Hash::LENGTH]),
            LeafKey { bytes: vec![123, 12, 1, 0, TIER_SEPARATOR, 88, TIER_SEPARATOR, 6, 6, 7] } => Hash([4; Hash::LENGTH]),
            LeafKey { bytes: vec![123, 12, 1, 0, TIER_SEPARATOR, 66, TIER_SEPARATOR, 1, 2, 3, 4] } => Hash([5; Hash::LENGTH]),
        )
    );
}

#[test]
fn deletes_node_tier_leaf_when_all_its_entries_deleted() {
    let mut store = TypedInMemoryTreeStore::new();
    put_at_next_version(
        &mut store,
        None,
        vec![
            change(1, 6, 2, Some(30)),
            change(1, 6, 9, Some(40)),
            change(2, 7, 3, Some(30)),
        ],
    );
    assert_eq!(get_leafs_of_tier(&store, Tier::ReNode).len(), 2);
    put_at_next_version(
        &mut store,
        Some(1),
        vec![change(1, 6, 2, None), change(1, 6, 9, None)],
    );
    assert_eq!(get_leafs_of_tier(&store, Tier::ReNode).len(), 1);
    put_at_next_version(&mut store, Some(2), vec![change(2, 7, 3, None)]);
    assert_eq!(get_leafs_of_tier(&store, Tier::ReNode).len(), 0);
}

#[test]
fn supports_empty_state() {
    let mut store = TypedInMemoryTreeStore::new();
    let hash_v1 = put_at_next_version(&mut store, None, vec![]);
    assert_eq!(hash_v1, SPARSE_MERKLE_PLACEHOLDER_HASH);
    let hash_v2 = put_at_next_version(&mut store, Some(1), vec![change(1, 6, 2, Some(30))]);
    assert_ne!(hash_v2, SPARSE_MERKLE_PLACEHOLDER_HASH);
    let hash_v3 = put_at_next_version(&mut store, Some(2), vec![change(1, 6, 2, None)]);
    assert_eq!(hash_v3, SPARSE_MERKLE_PLACEHOLDER_HASH);
}

#[test]
fn records_stale_tree_node_keys() {
    let mut store = TypedInMemoryTreeStore::new();
    put_at_next_version(&mut store, None, vec![change(4, 1, 6, Some(30))]);
    put_at_next_version(&mut store, Some(1), vec![change(3, 2, 9, Some(70))]);
    put_at_next_version(&mut store, Some(2), vec![change(3, 2, 9, Some(80))]);
    let stale_versions = store
        .stale_part_buffer
        .iter()
        .map(|stale_part| {
            let StaleTreePart::Node(key) = stale_part else {
                panic!("expected only single node removals");
            };
            key.version()
        })
        .unique()
        .sorted()
        .collect::<Vec<Version>>();
    assert_eq!(stale_versions, vec![1, 2]);
}

#[test]
fn records_stale_subtree_root_key_when_partition_removed() {
    let mut store = TypedInMemoryTreeStore::new();
    put_at_next_version(
        &mut store,
        None,
        vec![
            change(4, 7, 6, Some(36)),
            change(4, 7, 7, Some(37)),
            change(4, 7, 8, Some(38)),
        ],
    );
    put_at_next_version(&mut store, Some(1), vec![delete_partition(4, 7)]);
    assert_eq!(
        store.stale_part_buffer,
        vec![
            // The entire subtree starting at the root of substate-tier JMT of partition `4:7`:
            StaleTreePart::Subtree(NodeKey::new(
                1,
                NibblePath::new_even(
                    [
                        vec![4; Hash::LENGTH],
                        vec![TIER_SEPARATOR, 7, TIER_SEPARATOR]
                    ]
                    .concat()
                )
            )),
            // Regular single-node stale nodes up to the root, caused by hash update:
            StaleTreePart::Node(NodeKey::new(
                1,
                NibblePath::new_even([vec![4; Hash::LENGTH], vec![TIER_SEPARATOR]].concat())
            )),
            StaleTreePart::Node(NodeKey::new(1, NibblePath::new_even(vec![]))),
            // Importantly: individual 3x deletes of substate-tier nodes are not recorded
        ]
    );
}

#[test]
fn sbor_uses_custom_direct_codecs_for_nibbles() {
    let nibbles = nibbles("a1a2a3");
    let direct_bytes = nibbles.bytes().to_vec();
    let node = TreeNode::Leaf(TreeLeafNode {
        key_suffix: nibbles,
        value_hash: Hash([7; 32]),
        last_hash_change_version: 1337,
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
            value_hash: Hash([7; 32]),
            last_hash_change_version: 1337,
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
    put_at_next_version(&mut store, None, vec![change(3, 6, 4, Some(90))]);
    let previous_keys = store.memory.keys().cloned().collect::<HashSet<_>>();
    put_at_next_version(&mut store, Some(1), vec![change(1, 7, 2, Some(80))]);
    let min_next_key = store
        .memory
        .keys()
        .filter(|key| !previous_keys.contains(*key))
        .max()
        .unwrap();
    let max_previous_key = previous_keys.iter().max().unwrap();
    assert!(min_next_key > max_previous_key);
}

fn change(
    node_key_seed: u8,
    partition_num: u8,
    sort_key_seed: u8,
    value_hash_seed: Option<u8>,
) -> HashChange {
    HashChange::Single(SubstateHashChange::new(
        (
            db_partition_key(vec![node_key_seed; Hash::LENGTH], partition_num),
            DbSortKey(vec![sort_key_seed; sort_key_seed as usize]),
        ),
        value_hash_seed.map(|value_seed| value_hash(value_seed)),
    ))
}

fn delete_partition(node_key_seed: u8, partition_num: u8) -> HashChange {
    HashChange::Batch(BatchChange::DeletePartition(db_partition_key(
        vec![node_key_seed; Hash::LENGTH],
        partition_num,
    )))
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

fn leaf_key(node_key: &NodeKey, suffix_from_leaf: &NibblePath) -> LeafKey {
    LeafKey::new(
        NibblePath::from_iter(
            node_key
                .nibble_path()
                .nibbles()
                .chain(suffix_from_leaf.nibbles()),
        )
        .bytes(),
    )
}

fn db_partition_key(node_key: DbNodeKey, partition_num: DbPartitionNum) -> DbPartitionKey {
    DbPartitionKey {
        node_key,
        partition_num,
    }
}

enum Tier {
    ReNode,
    Partition,
    Substate,
}

const TIER_SEPARATOR: u8 = b'_';

fn get_leafs_of_tier(store: &TypedInMemoryTreeStore, tier: Tier) -> HashMap<LeafKey, Hash> {
    let stale_node_keys = store
        .stale_part_buffer
        .iter()
        .cloned()
        .flat_map(|stale_part| match stale_part {
            StaleTreePart::Node(key) => vec![key],
            StaleTreePart::Subtree(key) => JellyfishMerkleTree::new(store)
                .get_all_nodes_referenced(key)
                .unwrap(),
        })
        .collect::<HashSet<_>>();
    let expected_separator_count = tier as usize;
    store
        .tree_nodes
        .iter()
        .filter(|(key, _)| {
            let separator_count = key
                .nibble_path()
                .bytes()
                .iter()
                .filter(|byte| **byte == TIER_SEPARATOR)
                .count();
            separator_count == expected_separator_count
        })
        .filter(|(key, _)| !stale_node_keys.contains(key))
        .filter_map(|(key, node)| match node {
            TreeNode::Leaf(leaf) => {
                Some((leaf_key(key, &leaf.key_suffix), leaf.value_hash.clone()))
            }
            _ => None,
        })
        .collect()
}
