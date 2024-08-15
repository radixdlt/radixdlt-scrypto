use super::jellyfish::JellyfishMerkleTree;
use super::tier_framework::{StateTreeTier, TIER_SEPARATOR};
use super::tree_store::*;
use super::types::*;
use crate::state_tree::entity_tier::EntityTier;
use crate::state_tree::substate_tier::SubstateSummary;
use itertools::Itertools;
use radix_common::crypto::{hash, Hash};
use radix_common::data::scrypto::{scrypto_decode, scrypto_encode};
use radix_common::prelude::*;
use radix_substate_store_interface::interface::*;
use sbor::prelude::indexmap::indexmap;
use std::ops::Deref;

// Note: in some tests, we assert on the low-level DB key encoding, so we need this detail, and
// we alias it for brevity.
const TSEP: u8 = TIER_SEPARATOR;

#[test]
fn hash_of_next_version_differs_when_value_changed() {
    let mut tester = StateTreeTester::new_empty();
    let hash_v1 = tester.put_substate_changes(vec![change(1, 6, 2, Some(30))]);
    let hash_v2 = tester.put_substate_changes(vec![change(1, 6, 2, Some(70))]);
    assert_ne!(hash_v1, hash_v2);
}

#[test]
fn hash_of_next_version_same_when_write_repeated() {
    let mut tester = StateTreeTester::new_empty();
    let hash_v1 =
        tester.put_substate_changes(vec![change(4, 1, 6, Some(30)), change(3, 2, 9, Some(40))]);
    let hash_v2 = tester.put_substate_changes(vec![change(4, 1, 6, Some(30))]);
    assert_eq!(hash_v1, hash_v2);
}

#[test]
fn hash_of_next_version_same_when_write_empty() {
    let mut tester = StateTreeTester::new_empty();
    let hash_v1 =
        tester.put_substate_changes(vec![change(1, 6, 2, Some(30)), change(3, 7, 1, Some(40))]);
    let hash_v2 = tester.put_substate_changes(vec![]);
    assert_eq!(hash_v1, hash_v2);
}

#[test]
fn hash_of_next_version_differs_when_entry_added() {
    let mut tester = StateTreeTester::new_empty();
    let hash_v1 = tester.put_substate_changes(vec![change(1, 6, 2, Some(30))]);
    let hash_v2 = tester.put_substate_changes(vec![change(1, 6, 8, Some(30))]);
    assert_ne!(hash_v1, hash_v2);
}

#[test]
fn hash_of_next_version_differs_when_entry_removed() {
    let mut tester = StateTreeTester::new_empty();
    let hash_v1 =
        tester.put_substate_changes(vec![change(1, 6, 2, Some(30)), change(4, 7, 3, Some(20))]);
    let hash_v2 = tester.put_substate_changes(vec![change(1, 6, 2, None)]);
    assert_ne!(hash_v1, hash_v2);
}

#[test]
fn hash_returns_to_same_when_previous_state_restored() {
    let mut tester = StateTreeTester::new_empty();
    let hash_v1 =
        tester.put_substate_changes(vec![change(1, 6, 2, Some(30)), change(3, 7, 1, Some(40))]);
    tester.put_substate_changes(vec![
        change(1, 6, 2, Some(90)),
        change(3, 7, 1, None),
        change(1, 6, 5, Some(10)),
    ]);
    let hash_v3 = tester.put_substate_changes(vec![
        change(1, 6, 2, Some(30)),
        change(3, 7, 1, Some(40)),
        change(1, 6, 5, None),
    ]);
    assert_eq!(hash_v1, hash_v3);
}

#[test]
fn hash_computed_consistently_after_higher_tier_leafs_deleted() {
    // Compute a "reference" hash of state containing simply [2:3:4, 2:3:5].
    let mut reference_tester = StateTreeTester::new_empty();
    let reference_root = reference_tester
        .put_substate_changes(vec![change(2, 3, 4, Some(234)), change(2, 3, 5, Some(235))]);

    // Compute a hash of the same state, at which we arrive after deleting some unrelated NodeId.
    let mut tester = StateTreeTester::new_empty();
    tester.put_substate_changes(vec![
        change(1, 6, 2, Some(162)),
        change(1, 6, 3, Some(163)),
        change(2, 3, 4, Some(234)),
    ]);
    tester.put_substate_changes(vec![change(1, 6, 2, None), change(1, 6, 3, None)]);
    let root_after_deletes = tester.put_substate_changes(vec![change(2, 3, 5, Some(235))]);

    // We did [1:6:2, 1:6:3, 2:3:4] - [1:6:2, 1:6:3] + [2:3:5] = [2:3:4, 2:3:5] (i.e. same state).
    assert_eq!(root_after_deletes, reference_root);
}

#[test]
fn hash_computed_consistently_after_adding_higher_tier_sibling() {
    // Compute a "reference" hash of state containing simply [1:9:6, 2:3:4, 2:3:5].
    let mut reference_tester = StateTreeTester::new_empty();
    let reference_root = reference_tester.put_substate_changes(vec![
        change(1, 9, 6, Some(196)),
        change(2, 3, 4, Some(234)),
        change(2, 3, 5, Some(235)),
    ]);

    // Compute a hash of the same state, at which we arrive after adding some sibling NodeId.
    let mut tester = StateTreeTester::new_empty();
    tester.put_substate_changes(vec![change(2, 3, 4, Some(234))]);
    tester.put_substate_changes(vec![change(1, 9, 6, Some(196))]);
    let root_after_adding_sibling = tester.put_substate_changes(vec![change(2, 3, 5, Some(235))]);

    // We did [2:3:4] + [1:9:6] + [2:3:5] = [1:9:6, 2:3:4, 2:3:5] (i.e. same state).
    assert_eq!(root_after_adding_sibling, reference_root);
}

#[test]
fn hash_differs_when_states_only_differ_by_node_key() {
    let mut tester_1 = StateTreeTester::new_empty();
    let hash_1 = tester_1.put_substate_changes(vec![change(1, 6, 3, Some(30))]);
    let mut tester_2 = StateTreeTester::new_empty();
    let hash_2 = tester_2.put_substate_changes(vec![change(2, 6, 3, Some(30))]);
    assert_ne!(hash_1, hash_2);
}

#[test]
fn hash_differs_when_states_only_differ_by_partition_num() {
    let mut tester_1 = StateTreeTester::new_empty();
    let hash_1 = tester_1.put_substate_changes(vec![change(1, 6, 3, Some(30))]);
    let mut tester_2 = StateTreeTester::new_empty();
    let hash_2 = tester_2.put_substate_changes(vec![change(1, 7, 3, Some(30))]);
    assert_ne!(hash_1, hash_2);
}

#[test]
fn hash_differs_when_states_only_differ_by_sort_key() {
    let mut tester_1 = StateTreeTester::new_empty();
    let hash_1 = tester_1.put_substate_changes(vec![change(1, 6, 2, Some(30))]);
    let mut tester_2 = StateTreeTester::new_empty();
    let hash_2 = tester_2.put_substate_changes(vec![change(1, 6, 3, Some(30))]);
    assert_ne!(hash_1, hash_2);
}

#[test]
fn hash_of_different_re_nodes_is_same_when_contained_entries_are_same() {
    let mut tester = StateTreeTester::new_empty();
    tester.put_substate_changes(vec![
        change(1, 6, 2, Some(30)),
        change(1, 7, 9, Some(40)),
        change(7, 6, 2, Some(30)),
        change(7, 7, 9, Some(40)),
    ]);
    let re_node_leaf_hashes = tester
        .get_leafs_of_tier(Tier::ReNode)
        .into_values()
        .collect::<Vec<_>>();
    assert_eq!(re_node_leaf_hashes.len(), 2);
    assert_eq!(re_node_leaf_hashes[0], re_node_leaf_hashes[1])
}

#[test]
fn physical_nodes_of_tiered_jmt_have_expected_keys_and_contents() {
    let mut tester = StateTreeTester::new_empty();
    tester.put_substate_changes(vec![
        change_exact(vec![1, 3, 3, 7], 99, vec![253], Some(vec![1])),
        change_exact(vec![1, 3, 3, 7], 99, vec![66], Some(vec![2])),
        change_exact(vec![123, 12, 1, 0], 88, vec![6, 6, 6], Some(vec![3])),
        change_exact(vec![123, 12, 1, 0], 88, vec![6, 6, 7], Some(vec![4])),
        change_exact(vec![123, 12, 1, 0], 66, vec![1, 2, 3], Some(vec![5])),
    ]);

    // Assert on the keys of Node-tier leafs:
    let node_tier_leaf_keys = tester
        .get_leafs_of_tier(Tier::ReNode)
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
    let partition_tier_leaf_keys = tester
        .get_leafs_of_tier(Tier::Partition)
        .into_keys()
        .collect::<HashSet<_>>();
    assert_eq!(
        partition_tier_leaf_keys,
        hashset!(
            LeafKey {
                bytes: vec![1, 3, 3, 7, TSEP, 99]
            },
            LeafKey {
                bytes: vec![123, 12, 1, 0, TSEP, 66]
            },
            LeafKey {
                bytes: vec![123, 12, 1, 0, TSEP, 88]
            },
        )
    );

    // Assert on the keys and hashes of the Substate-tier leaves:
    let substate_tier_leaves = tester.get_leafs_of_tier(Tier::Substate);
    assert_eq!(
        substate_tier_leaves,
        hashmap!(
            LeafKey { bytes: vec![1, 3, 3, 7, TSEP, 99, TSEP, 253] } => hash([1]),
            LeafKey { bytes: vec![1, 3, 3, 7, TSEP, 99, TSEP, 66] } => hash([2]),
            LeafKey { bytes: vec![123, 12, 1, 0, TSEP, 88, TSEP, 6, 6, 6] } => hash([3]),
            LeafKey { bytes: vec![123, 12, 1, 0, TSEP, 88, TSEP, 6, 6, 7] } => hash([4]),
            LeafKey { bytes: vec![123, 12, 1, 0, TSEP, 66, TSEP, 1, 2, 3] } => hash([5]),
        )
    );
}

#[test]
fn substate_values_get_associated_with_substate_tier_leaves() {
    let mut tester =
        StateTreeTester::new(TypedInMemoryTreeStore::new().storing_associated_substates());
    tester.put_substate_changes(vec![
        change_exact(vec![123, 12, 1], 8, vec![6, 6, 1], Some(vec![4])),
        change_exact(vec![123, 12, 1], 8, vec![6, 6, 2], Some(vec![])),
        change_exact(vec![123, 12, 1], 8, vec![6, 7, 5, 9], Some(vec![1, 2])),
        change_exact(vec![220, 3], 99, vec![253], Some(vec![7; 66])),
    ]);

    let associated_substates = tester.tree_store.associated_substates.borrow();
    assert_eq!(
        associated_substates.deref(),
        &hashmap!(
            // 2 incidentally-complete node keys: (they differ only at their last byte)
            StoredTreeNodeKey::new(1, NibblePath::new_even(vec![123, 12, 1, TSEP, 8, TSEP, 6, 6, 1])) => (
                (partition_key(vec![123, 12, 1], 8), DbSortKey(vec![6, 6, 1])), Some(vec![4])
            ),
            StoredTreeNodeKey::new(1, NibblePath::new_even(vec![123, 12, 1, TSEP, 8, TSEP, 6, 6, 2])) => (
                (partition_key(vec![123, 12, 1], 8), DbSortKey(vec![6, 6, 2])), Some(vec![])
            ),
            // A slightly-incomplete node key: (cut short at the first byte it differs)
            StoredTreeNodeKey::new(1, NibblePath::new_even(vec![123, 12, 1, TSEP, 8, TSEP, 6, 7])) => (
                (partition_key(vec![123, 12, 1], 8), DbSortKey(vec![6, 7, 5, 9])), Some(vec![1, 2])
            ),
            // A very incomplete node key, representing Substate-Tier root: (since it is the only Substate within its partition)
            StoredTreeNodeKey::new(1, NibblePath::new_even(vec![220, 3, TSEP, 99, TSEP])) => (
                (partition_key(vec![220, 3], 99), DbSortKey(vec![253])), Some(vec![7; 66])
            ),
        )
    );
}

#[test]
fn substate_values_get_re_associated_after_tree_restructuring() {
    let mut tester =
        StateTreeTester::new(TypedInMemoryTreeStore::new().storing_associated_substates());
    // Let's start with the same set-up as in the base `substate_values_get_associated_with_substate_tier_leaves` test:
    tester.put_substate_changes(vec![
        change_exact(vec![123, 12, 1], 8, vec![6, 6, 1], Some(vec![4])),
        change_exact(vec![123, 12, 1], 8, vec![6, 6, 2], Some(vec![])),
        change_exact(vec![123, 12, 1], 8, vec![6, 7, 5, 9], Some(vec![1, 2])),
        change_exact(vec![220, 3], 99, vec![253], Some(vec![7; 66])),
    ]);

    // For clearer assert, let's disregard the substates associated in this step:
    tester.tree_store.associated_substates.borrow_mut().clear();

    // Now inserting this "sibling" substate forces rewrite of the leaf associated with sort key `vec![6, 7, 5, 9]`:
    tester.put_substate_changes(vec![change_exact(
        vec![123, 12, 1],
        8,
        vec![6, 7, 5, 4],
        Some(vec![3]),
    )]);

    let associated_substates = tester.tree_store.associated_substates.borrow();
    assert_eq!(
        associated_substates.deref(),
        &hashmap!(
            // The newly-inserted substate:
            StoredTreeNodeKey::new(2, NibblePath::new_even(vec![123, 12, 1, TSEP, 8, TSEP, 6, 7, 5, 4])) => (
                (partition_key(vec![123, 12, 1], 8), DbSortKey(vec![6, 7, 5, 4])), Some(vec![3])
            ),
            // Its previously-existing sibling, whose tree node had to be re-inserted due to restructuring:
            StoredTreeNodeKey::new(2, NibblePath::new_even(vec![123, 12, 1, TSEP, 8, TSEP, 6, 7, 5, 9])) => (
                (partition_key(vec![123, 12, 1], 8), DbSortKey(vec![6, 7, 5, 9])), None // denotes "unchanged value"
            ),
        )
    );
}

#[test]
fn substate_values_get_re_associated_on_partition_reset() {
    let mut tester =
        StateTreeTester::new(TypedInMemoryTreeStore::new().storing_associated_substates());
    // Let's start with the same set-up as in the base `substate_values_get_associated_with_substate_tier_leaves` test:
    tester.put_substate_changes(vec![
        change_exact(vec![123, 12, 1], 8, vec![6, 6, 1], Some(vec![4])),
        change_exact(vec![123, 12, 1], 8, vec![6, 6, 2], Some(vec![])),
        change_exact(vec![123, 12, 1], 8, vec![6, 7, 5, 9], Some(vec![1, 2])),
        change_exact(vec![220, 3], 99, vec![253], Some(vec![7; 66])),
    ]);

    // For clearer assert, let's disregard the substates associated in this step:
    tester.tree_store.associated_substates.borrow_mut().clear();

    // Now we achieve the "add sibling substate", but using a partition reset:
    tester.reset_partition(
        vec![123, 12, 1],
        8,
        vec![
            (DbSortKey(vec![6, 7, 5, 9]), vec![1, 2]), // we re-create this one exactly the same
            (DbSortKey(vec![6, 7, 5, 4]), vec![3]),    // and we give it a sibling
        ],
    );

    let associated_substates = tester.tree_store.associated_substates.borrow();
    assert_eq!(
        associated_substates.deref(),
        &hashmap!(
            // The newly-inserted substate:
            StoredTreeNodeKey::new(2, NibblePath::new_even(vec![123, 12, 1, TSEP, 8, TSEP, 6, 7, 5, 4])) => (
                (partition_key(vec![123, 12, 1], 8), DbSortKey(vec![6, 7, 5, 4])), Some(vec![3])
            ),
            // Its previously-existing sibling (which was re-created on reset), treated as if it was completely new:
            StoredTreeNodeKey::new(2, NibblePath::new_even(vec![123, 12, 1, TSEP, 8, TSEP, 6, 7, 5, 9])) => (
                (partition_key(vec![123, 12, 1], 8), DbSortKey(vec![6, 7, 5, 9])), Some(vec![1, 2])
            ),
        )
    );
}

#[test]
fn deletes_node_tier_leaf_when_all_its_entries_deleted() {
    let mut tester = StateTreeTester::new_empty();
    tester.put_substate_changes(vec![
        change(1, 6, 2, Some(30)),
        change(1, 6, 9, Some(40)),
        change(2, 7, 3, Some(30)),
    ]);
    assert_eq!(tester.get_leafs_of_tier(Tier::ReNode).len(), 2);
    tester.put_substate_changes(vec![change(1, 6, 2, None), change(1, 6, 9, None)]);
    assert_eq!(tester.get_leafs_of_tier(Tier::ReNode).len(), 1);
    tester.put_substate_changes(vec![change(2, 7, 3, None)]);
    assert_eq!(tester.get_leafs_of_tier(Tier::ReNode).len(), 0);
}

#[test]
fn supports_empty_state() {
    let mut tester = StateTreeTester::new_empty();
    let hash_v1 = tester.put_substate_changes(vec![]);
    assert_eq!(hash_v1, None);
    let hash_v2 = tester.put_substate_changes(vec![change(1, 6, 2, Some(30))]);
    assert_ne!(hash_v2, None);
    let hash_v3 = tester.put_substate_changes(vec![change(1, 6, 2, None)]);
    assert_eq!(hash_v3, None);
}

#[test]
fn records_stale_tree_node_keys() {
    let mut tester = StateTreeTester::new_empty();
    tester.put_substate_changes(vec![change(4, 1, 6, Some(30))]);
    tester.put_substate_changes(vec![change(3, 2, 9, Some(70))]);
    tester.put_substate_changes(vec![change(3, 2, 9, Some(80))]);
    let stale_versions = tester
        .tree_store
        .stale_part_buffer
        .borrow()
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
fn hash_computed_consistently_after_different_deletes() {
    // Reference: simply put [2:1:3, 2:3:4]
    let mut reference_tester = StateTreeTester::new_empty();
    let reference_root = reference_tester
        .put_substate_changes(vec![change(2, 1, 3, Some(213)), change(2, 3, 4, Some(234))]);

    // Delete 2 individual substates to arrive at the same state:
    let mut single_deletes_tester = StateTreeTester::new_empty();
    single_deletes_tester.put_substate_changes(vec![
        change(2, 1, 3, Some(213)),
        change(2, 3, 4, Some(234)),
        change(2, 3, 5, Some(235)),
        change(2, 3, 6, Some(236)),
    ]);
    let single_deletes_root = single_deletes_tester
        .put_substate_changes(vec![change(2, 3, 5, None), change(2, 3, 6, None)]);
    assert_eq!(single_deletes_root, reference_root);

    // Delete entire partition 2:3, and then add back 2:3:4 in next version:
    let mut delete_and_put_tester = StateTreeTester::new_empty();
    delete_and_put_tester.put_substate_changes(vec![
        change(2, 1, 3, Some(213)),
        change(2, 3, 4, Some(234)),
        change(2, 3, 5, Some(235)),
        change(2, 3, 6, Some(236)),
    ]);
    delete_and_put_tester.reset_partition(from_seed(2), 3, vec![]);
    let delete_and_put_root =
        delete_and_put_tester.put_substate_changes(vec![change(2, 3, 4, Some(234))]);
    assert_eq!(delete_and_put_root, reference_root);

    // Reset entire partition 2:3 to only contain 2:3:4:
    let mut reset_tester = StateTreeTester::new_empty();
    reset_tester.put_substate_changes(vec![
        change(2, 1, 3, Some(213)),
        change(2, 3, 4, Some(234)),
        change(2, 3, 5, Some(235)),
        change(2, 3, 6, Some(236)),
    ]);
    let reset_root = reset_tester.reset_partition(
        from_seed(2),
        3,
        vec![(DbSortKey(from_seed(4)), from_seed(234))],
    );
    assert_eq!(reset_root, reference_root);
}

#[test]
fn records_stale_subtree_root_key_when_partition_removed() {
    let mut tester = StateTreeTester::new_empty();
    tester.put_substate_changes(vec![
        change(4, 7, 6, Some(36)),
        change(4, 7, 7, Some(37)),
        change(4, 7, 8, Some(38)),
    ]);
    tester.reset_partition(from_seed(4), 7, vec![]);
    assert_eq!(
        tester.tree_store.stale_part_buffer.borrow().to_vec(),
        vec![
            // The entire subtree starting at the root of substate-tier JMT of partition `4:7`:
            StaleTreePart::Subtree(StoredTreeNodeKey::new(
                1,
                NibblePath::new_even([from_seed(4), vec![TSEP, 7, TSEP]].concat())
            )),
            // Regular single-node stale nodes up to the root, caused by hash update:
            StaleTreePart::Node(StoredTreeNodeKey::new(
                1,
                NibblePath::new_even([from_seed(4), vec![TSEP]].concat())
            )),
            StaleTreePart::Node(StoredTreeNodeKey::new(1, NibblePath::new_even(vec![]))),
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
    let mut tester = StateTreeTester::new(SerializedInMemoryTreeStore::new());
    tester.put_substate_changes(vec![change(3, 6, 4, Some(90))]);
    let previous_keys = tester
        .tree_store
        .memory()
        .keys()
        .cloned()
        .collect::<HashSet<_>>();
    tester.put_substate_changes(vec![change(1, 7, 2, Some(80))]);
    let tree_store_mem = tester.tree_store.memory();
    let min_next_key = tree_store_mem
        .keys()
        .filter(|key| !previous_keys.contains(*key))
        .max()
        .unwrap();
    let max_previous_key = previous_keys.iter().max().unwrap();
    assert!(min_next_key > max_previous_key);
}

#[test]
fn returns_partition_tier_for_entity_id() {
    let mut tester = StateTreeTester::new_empty();
    tester.put_substate_changes(vec![
        change(1, 9, 6, Some(196)),
        change(2, 3, 4, Some(234)),
        change(2, 3, 5, Some(235)),
        change(2, 4, 7, Some(237)),
    ]);

    let existent_partition_tier = tester
        .create_subject()
        .get_entity_partition_tier(from_seed(1));
    assert!(existent_partition_tier.root_version().is_some());

    let non_existent_partition_tier = tester
        .create_subject()
        .get_entity_partition_tier(from_seed(3));
    assert!(non_existent_partition_tier.root_version().is_none());
}

#[test]
fn lists_partition_tiers() {
    let mut tester = StateTreeTester::new_empty();
    tester.put_substate_changes(vec![
        change(2, 9, 6, Some(196)),
        change(4, 3, 4, Some(234)),
        change(4, 3, 5, Some(235)),
        change(9, 4, 7, Some(237)),
    ]);

    let all_partition_tiers = tester
        .create_subject()
        .iter_entity_partition_tiers_from(None)
        .map(|tier| tier.entity_key().clone())
        .collect::<Vec<_>>();
    assert_eq!(
        all_partition_tiers,
        vec![from_seed(2), from_seed(4), from_seed(9)]
    );

    let from_existent = tester
        .create_subject()
        .iter_entity_partition_tiers_from(Some(&from_seed(4)))
        .map(|tier| tier.entity_key().clone())
        .collect::<Vec<_>>();
    assert_eq!(from_existent, vec![from_seed(4), from_seed(9)]);

    let from_non_existent = tester
        .create_subject()
        .iter_entity_partition_tiers_from(Some(&from_seed(3)))
        .map(|tier| tier.entity_key().clone())
        .collect::<Vec<_>>();
    assert_eq!(from_non_existent, vec![from_seed(4), from_seed(9)]);

    let from_lt_min = tester
        .create_subject()
        .iter_entity_partition_tiers_from(Some(&from_seed(1)))
        .map(|tier| tier.entity_key().clone())
        .collect::<Vec<_>>();
    assert_eq!(from_lt_min, vec![from_seed(2), from_seed(4), from_seed(9)]);

    let from_gt_max = tester
        .create_subject()
        .iter_entity_partition_tiers_from(Some(&from_seed(14)))
        .map(|tier| tier.entity_key().clone())
        .collect::<Vec<_>>();
    assert!(from_gt_max.is_empty());
}

#[test]
fn returns_substate_tier_for_partition_num() {
    let mut tester = StateTreeTester::new_empty();
    tester.put_substate_changes(vec![
        change(1, 9, 6, Some(196)),
        change(2, 3, 4, Some(234)),
        change(2, 3, 5, Some(235)),
        change(2, 4, 7, Some(237)),
    ]);

    let existent_substate_tier = tester
        .create_subject()
        .get_entity_partition_tier(from_seed(2))
        .get_partition_substate_tier(3);
    assert!(existent_substate_tier.root_version().is_some());

    let non_existent_from_entity = tester
        .create_subject()
        .get_entity_partition_tier(from_seed(3))
        .get_partition_substate_tier(3);
    assert!(non_existent_from_entity.root_version().is_none());

    let non_existent_from_partition = tester
        .create_subject()
        .get_entity_partition_tier(from_seed(2))
        .get_partition_substate_tier(5);
    assert!(non_existent_from_partition.root_version().is_none());
}

#[test]
fn lists_substate_tiers() {
    let mut tester = StateTreeTester::new_empty();
    tester.put_substate_changes(vec![
        change(1, 9, 6, Some(196)),
        change(3, 2, 4, Some(234)),
        change(3, 6, 5, Some(235)),
        change(3, 7, 4, Some(234)),
        change(9, 4, 7, Some(237)),
    ]);

    let all_substate_tiers = tester
        .create_subject()
        .get_entity_partition_tier(from_seed(3))
        .iter_partition_substate_tiers_from(None)
        .map(|tier| tier.partition_key().partition_num)
        .collect::<Vec<_>>();
    assert_eq!(all_substate_tiers, vec![2, 6, 7]);

    let from_existent = tester
        .create_subject()
        .get_entity_partition_tier(from_seed(3))
        .iter_partition_substate_tiers_from(Some(6))
        .map(|tier| tier.partition_key().partition_num)
        .collect::<Vec<_>>();
    assert_eq!(from_existent, vec![6, 7]);

    let from_non_existent = tester
        .create_subject()
        .get_entity_partition_tier(from_seed(3))
        .iter_partition_substate_tiers_from(Some(4))
        .map(|tier| tier.partition_key().partition_num)
        .collect::<Vec<_>>();
    assert_eq!(from_non_existent, vec![6, 7]);

    let from_lt_min = tester
        .create_subject()
        .get_entity_partition_tier(from_seed(3))
        .iter_partition_substate_tiers_from(Some(1))
        .map(|tier| tier.partition_key().partition_num)
        .collect::<Vec<_>>();
    assert_eq!(from_lt_min, vec![2, 6, 7]);

    let from_gt_max = tester
        .create_subject()
        .get_entity_partition_tier(from_seed(3))
        .iter_partition_substate_tiers_from(Some(9))
        .map(|tier| tier.partition_key().partition_num)
        .collect::<Vec<_>>();
    assert_eq!(from_gt_max, vec![]);
}

#[test]
fn returns_substate_summary_by_sort_key() {
    let mut tester = StateTreeTester::new_empty();
    tester.put_substate_changes(vec![
        change(1, 9, 6, Some(196)),
        change(2, 3, 4, Some(234)),
        change(2, 3, 5, Some(235)),
        change(2, 4, 7, Some(237)),
    ]);

    let existent_substate = tester
        .create_subject()
        .get_entity_partition_tier(from_seed(2))
        .get_partition_substate_tier(3)
        .get_substate_summary(&DbSortKey(from_seed(5)));
    assert_eq!(
        existent_substate,
        Some(SubstateSummary {
            sort_key: DbSortKey(from_seed(5)),
            upsert_version: 1,
            value_hash: hash(from_seed(235)),
            state_tree_leaf_key: stored_node_key(1, vec![2, 2, TSEP, 3, TSEP, 5]),
        })
    );

    let non_existent_from_entity = tester
        .create_subject()
        .get_entity_partition_tier(from_seed(3))
        .get_partition_substate_tier(3)
        .get_substate_summary(&DbSortKey(from_seed(5)));
    assert_eq!(non_existent_from_entity, None);

    let non_existent_from_partition = tester
        .create_subject()
        .get_entity_partition_tier(from_seed(2))
        .get_partition_substate_tier(5)
        .get_substate_summary(&DbSortKey(from_seed(5)));
    assert_eq!(non_existent_from_partition, None);

    let non_existent_from_sort_key = tester
        .create_subject()
        .get_entity_partition_tier(from_seed(2))
        .get_partition_substate_tier(3)
        .get_substate_summary(&DbSortKey(from_seed(6)));
    assert_eq!(non_existent_from_sort_key, None);
}

#[test]
fn lists_substate_summaries() {
    let mut tester = StateTreeTester::new_empty();
    // This use-case is the most important one; we want to test multiple versions/upserts/deletes:
    tester.put_substate_changes(vec![
        change(1, 9, 1, Some(196)), // irrelevant node
        change(3, 2, 6, Some(16)),  // Note: this one will be overwritten!
    ]);
    tester.put_substate_changes(vec![
        change(3, 2, 4, Some(24)),
        change(3, 2, 7, Some(27)), // Note: this one will be deleted!
    ]);
    tester.put_substate_changes(vec![
        change(3, 2, 6, Some(36)), // (the overwrite)
        change(3, 2, 7, None),     // (the delete)
        change(3, 2, 8, Some(38)),
        change(3, 4, 7, Some(237)), // irrelevant partition
    ]);

    // Handcrafted expected substates of Entity=3, Partition=2, at current version=3:
    let known_substate_summaries = vec![
        SubstateSummary {
            sort_key: DbSortKey(from_seed(4)),
            upsert_version: 2,
            value_hash: hash(from_seed(24)),
            state_tree_leaf_key: stored_node_key(2, vec![3, 3, 3, TSEP, 2, TSEP, 4]),
        },
        SubstateSummary {
            sort_key: DbSortKey(from_seed(6)),
            upsert_version: 3,
            value_hash: hash(from_seed(36)),
            state_tree_leaf_key: stored_node_key(3, vec![3, 3, 3, TSEP, 2, TSEP, 6]),
        },
        SubstateSummary {
            sort_key: DbSortKey(from_seed(8)),
            upsert_version: 3,
            value_hash: hash(from_seed(38)),
            state_tree_leaf_key: stored_node_key(3, vec![3, 3, 3, TSEP, 2, TSEP, 8]),
        },
    ];

    let all_substate_summaries = tester
        .create_subject()
        .get_entity_partition_tier(from_seed(3))
        .get_partition_substate_tier(2)
        .iter_substate_summaries_from(None)
        .collect::<Vec<_>>();
    assert_eq!(all_substate_summaries, known_substate_summaries);

    let from_existent = tester
        .create_subject()
        .get_entity_partition_tier(from_seed(3))
        .get_partition_substate_tier(2)
        .iter_substate_summaries_from(Some(&DbSortKey(from_seed(6))))
        .collect::<Vec<_>>();
    assert_eq!(from_existent, known_substate_summaries[1..]);

    let from_non_existent = tester
        .create_subject()
        .get_entity_partition_tier(from_seed(3))
        .get_partition_substate_tier(2)
        .iter_substate_summaries_from(Some(&DbSortKey(from_seed(5))))
        .collect::<Vec<_>>();
    assert_eq!(from_non_existent, known_substate_summaries[1..]);

    let from_lt_min = tester
        .create_subject()
        .get_entity_partition_tier(from_seed(3))
        .get_partition_substate_tier(2)
        .iter_substate_summaries_from(Some(&DbSortKey(from_seed(1))))
        .collect::<Vec<_>>();
    assert_eq!(from_lt_min, known_substate_summaries);

    let from_gt_max = tester
        .create_subject()
        .get_entity_partition_tier(from_seed(3))
        .get_partition_substate_tier(2)
        .iter_substate_summaries_from(Some(&DbSortKey(from_seed(9))))
        .collect::<Vec<_>>();
    assert_eq!(from_gt_max, vec![]);

    // We do not use this in practice, but the tree is capable of starting listing from prefix too:
    let from_prefix = tester
        .create_subject()
        .get_entity_partition_tier(from_seed(3))
        .get_partition_substate_tier(2)
        .iter_substate_summaries_from(Some(&DbSortKey(from_seed(8)[..3].to_vec())))
        .collect::<Vec<_>>();
    assert_eq!(from_prefix, known_substate_summaries[2..]); // we get the last element, `from_seed(8)`
}

#[test]
fn lists_historical_substate_summaries() {
    let mut tester = StateTreeTester::new_empty();
    // We use the same set-up as in `lists_substate_summaries()`:
    tester.put_substate_changes(vec![
        change(1, 9, 1, Some(196)), // irrelevant node
        change(3, 2, 6, Some(16)),  // Note: this one will be overwritten!
    ]);
    tester.put_substate_changes(vec![
        change(3, 2, 4, Some(24)),
        change(3, 2, 7, Some(27)), // Note: this one will be deleted!
    ]);
    tester.put_substate_changes(vec![
        change(3, 2, 6, Some(36)), // (the overwrite)
        change(3, 2, 7, None),     // (the delete)
        change(3, 2, 8, Some(38)),
        change(3, 4, 7, Some(237)), // irrelevant partition
    ]);

    let historical_substates = EntityTier::new(&tester.tree_store, Some(2))
        .get_entity_partition_tier(from_seed(3))
        .get_partition_substate_tier(2)
        .iter_substate_summaries_from(Some(&DbSortKey(from_seed(5))))
        .collect::<Vec<_>>();
    assert_eq!(
        historical_substates,
        vec![
            SubstateSummary {
                sort_key: DbSortKey(from_seed(6)),
                upsert_version: 1,
                value_hash: hash(from_seed(16)), // the value before update
                state_tree_leaf_key: stored_node_key(2, vec![3, 3, 3, TSEP, 2, TSEP, 6]),
            },
            SubstateSummary {
                sort_key: DbSortKey(from_seed(7)), // this one gets later deleted
                upsert_version: 2,
                value_hash: hash(from_seed(27)),
                state_tree_leaf_key: stored_node_key(2, vec![3, 3, 3, TSEP, 2, TSEP, 7]),
            },
            // substate `from_seed(8)` gets created later
        ]
    );
}

type SingleSubstateChange = (DbSubstateKey, DatabaseUpdate);

fn change(
    node_key_seed: u8,
    partition_num: u8,
    sort_key_seed: u8,
    value_seed: Option<u8>,
) -> SingleSubstateChange {
    change_exact(
        from_seed(node_key_seed),
        partition_num,
        from_seed(sort_key_seed),
        value_seed.map(|value_seed| from_seed(value_seed)),
    )
}

pub fn change_exact(
    node_key: Vec<u8>,
    partition_num: u8,
    sort_key: Vec<u8>,
    value: Option<Vec<u8>>,
) -> SingleSubstateChange {
    (
        (partition_key(node_key, partition_num), DbSortKey(sort_key)),
        value
            .map(|value| DatabaseUpdate::Set(value))
            .unwrap_or(DatabaseUpdate::Delete),
    )
}

fn partition_key(node_key: Vec<u8>, partition_num: u8) -> DbPartitionKey {
    DbPartitionKey {
        node_key,
        partition_num,
    }
}

fn from_seed(seed: u8) -> Vec<u8> {
    vec![seed; seed as usize]
}

fn stored_node_key(version: Version, bytes: Vec<u8>) -> StoredTreeNodeKey {
    StoredTreeNodeKey::new(version, NibblePath::new_even(bytes))
}

fn nibbles(hex_string: &str) -> NibblePath {
    NibblePath::from_iter(
        hex_string
            .chars()
            .map(|nibble| Nibble::from(char::to_digit(nibble, 16).unwrap() as u8)),
    )
}

pub enum Tier {
    ReNode,
    Partition,
    Substate,
}

pub struct StateTreeTester<S> {
    pub tree_store: S,
    pub current_version: Option<Version>,
}

impl<S: TreeStore> StateTreeTester<S> {
    pub fn new(tree_store: S) -> Self {
        Self {
            tree_store,
            current_version: None,
        }
    }

    pub fn put_substate_changes(
        &mut self,
        changes: impl IntoIterator<Item = SingleSubstateChange>,
    ) -> Option<Hash> {
        self.apply_database_updates(&DatabaseUpdates::from_delta_maps(
            Self::index_to_delta_maps(changes),
        ))
    }

    pub fn reset_partition(
        &mut self,
        node_key: DbNodeKey,
        partition_num: DbPartitionNum,
        values: impl IntoIterator<Item = (DbSortKey, DbSubstateValue)>,
    ) -> Option<Hash> {
        self.apply_database_updates(&DatabaseUpdates {
            node_updates: indexmap!(
                node_key => NodeDatabaseUpdates {
                    partition_updates: indexmap!(
                        partition_num => PartitionDatabaseUpdates::Reset {
                            new_substate_values: values.into_iter().collect()
                        }
                    )
                }
            ),
        })
    }

    pub fn create_subject(&self) -> EntityTier<S> {
        EntityTier::new(&self.tree_store, self.current_version)
    }

    fn apply_database_updates(&mut self, database_updates: &DatabaseUpdates) -> Option<Hash> {
        let mut entity_tier = self.create_subject();
        let new_root_hash = entity_tier.put_next_version_entity_updates(database_updates);
        self.current_version = entity_tier.root_version();
        new_root_hash
    }

    fn index_to_delta_maps(
        changes: impl IntoIterator<Item = SingleSubstateChange>,
    ) -> IndexMap<DbPartitionKey, IndexMap<DbSortKey, DatabaseUpdate>> {
        let mut delta_maps = index_map_new::<DbPartitionKey, IndexMap<DbSortKey, DatabaseUpdate>>();
        for change in changes {
            let ((partition_key, sort_key), update) = change;
            delta_maps
                .entry(partition_key)
                .or_default()
                .insert(sort_key, update);
        }
        delta_maps
    }
}

impl StateTreeTester<TypedInMemoryTreeStore> {
    pub fn new_empty() -> Self {
        Self::new(TypedInMemoryTreeStore::new())
    }

    pub fn get_leafs_of_tier(&mut self, tier: Tier) -> HashMap<LeafKey, Hash> {
        let binding = self.tree_store.stale_part_buffer.borrow().clone();
        let stale_node_keys = binding
            .into_iter()
            .flat_map(|stale_part| match stale_part {
                StaleTreePart::Node(key) => vec![key],
                StaleTreePart::Subtree(key) => JellyfishMerkleTree::new(&mut self.tree_store)
                    .get_all_nodes_referenced(TreeNodeKey::new(
                        key.version(),
                        key.nibble_path().clone(),
                    ))
                    .unwrap()
                    .into_iter()
                    .map(|key| StoredTreeNodeKey::unprefixed(key))
                    .collect(),
            })
            .collect::<HashSet<_>>();
        let expected_separator_count = tier as usize;
        self.tree_store
            .tree_nodes
            .borrow()
            .iter()
            .filter(|(key, _)| {
                let separator_count = key
                    .nibble_path()
                    .bytes()
                    .iter()
                    .filter(|byte| **byte == TSEP)
                    .count();
                separator_count == expected_separator_count
            })
            .filter(|(key, _)| !stale_node_keys.contains(key))
            .filter_map(|(key, node)| match node {
                TreeNode::Leaf(leaf) => Some((
                    Self::leaf_key(key, &leaf.key_suffix),
                    leaf.value_hash.clone(),
                )),
                _ => None,
            })
            .collect()
    }

    fn leaf_key(node_key: &StoredTreeNodeKey, suffix_from_leaf: &NibblePath) -> LeafKey {
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
}
