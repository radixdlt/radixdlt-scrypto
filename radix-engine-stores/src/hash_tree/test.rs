use super::types::{Nibble, NibblePath, Version, SPARSE_MERKLE_PLACEHOLDER_HASH};
use crate::hash_tree::jellyfish::JellyfishMerkleTree;
use crate::hash_tree::put_at_next_version;
use crate::hash_tree::tree_store::{
    SerializedInMemoryTreeStore, StaleTreePart, TreeChildEntry, TreeInternalNode, TreeLeafNode,
    TreeNode, TreeStore, TypedInMemoryTreeStore,
};
use crate::hash_tree::types::{LeafKey, NodeKey};
use itertools::Itertools;
use radix_engine_common::crypto::{hash, Hash};
use radix_engine_common::data::scrypto::{scrypto_decode, scrypto_encode};
use radix_engine_store_interface::interface::{
    BatchPartitionDatabaseUpdate, DatabaseUpdate, DatabaseUpdates, DbNodeKey, DbPartitionKey,
    DbPartitionNum, DbSortKey, DbSubstateKey, DbSubstateValue, NodeDatabaseUpdates,
    PartitionDatabaseUpdates,
};
use sbor::prelude::indexmap::indexmap;
use utils::prelude::{index_map_new, IndexMap};
use utils::rust::collections::{hashmap, hashset, HashMap, HashSet};

#[test]
fn hash_of_next_version_differs_when_value_changed() {
    let mut tester = HashTreeTester::new_empty();
    let hash_v1 = tester.put_substate_changes(vec![change(1, 6, 2, Some(30))]);
    let hash_v2 = tester.put_substate_changes(vec![change(1, 6, 2, Some(70))]);
    assert_ne!(hash_v1, hash_v2);
}

#[test]
fn hash_of_next_version_same_when_write_repeated() {
    let mut tester = HashTreeTester::new_empty();
    let hash_v1 =
        tester.put_substate_changes(vec![change(4, 1, 6, Some(30)), change(3, 2, 9, Some(40))]);
    let hash_v2 = tester.put_substate_changes(vec![change(4, 1, 6, Some(30))]);
    assert_eq!(hash_v1, hash_v2);
}

#[test]
fn hash_of_next_version_same_when_write_empty() {
    let mut tester = HashTreeTester::new_empty();
    let hash_v1 =
        tester.put_substate_changes(vec![change(1, 6, 2, Some(30)), change(3, 7, 1, Some(40))]);
    let hash_v2 = tester.put_substate_changes(vec![]);
    assert_eq!(hash_v1, hash_v2);
}

#[test]
fn hash_of_next_version_differs_when_entry_added() {
    let mut tester = HashTreeTester::new_empty();
    let hash_v1 = tester.put_substate_changes(vec![change(1, 6, 2, Some(30))]);
    let hash_v2 = tester.put_substate_changes(vec![change(1, 6, 8, Some(30))]);
    assert_ne!(hash_v1, hash_v2);
}

#[test]
fn hash_of_next_version_differs_when_entry_removed() {
    let mut tester = HashTreeTester::new_empty();
    let hash_v1 =
        tester.put_substate_changes(vec![change(1, 6, 2, Some(30)), change(4, 7, 3, Some(20))]);
    let hash_v2 = tester.put_substate_changes(vec![change(1, 6, 2, None)]);
    assert_ne!(hash_v1, hash_v2);
}

#[test]
fn hash_returns_to_same_when_previous_state_restored() {
    let mut tester = HashTreeTester::new_empty();
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
    let mut reference_tester = HashTreeTester::new_empty();
    let reference_root = reference_tester
        .put_substate_changes(vec![change(2, 3, 4, Some(234)), change(2, 3, 5, Some(235))]);

    // Compute a hash of the same state, at which we arrive after deleting some unrelated NodeId.
    let mut tester = HashTreeTester::new_empty();
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
    let mut reference_tester = HashTreeTester::new_empty();
    let reference_root = reference_tester.put_substate_changes(vec![
        change(1, 9, 6, Some(196)),
        change(2, 3, 4, Some(234)),
        change(2, 3, 5, Some(235)),
    ]);

    // Compute a hash of the same state, at which we arrive after adding some sibling NodeId.
    let mut tester = HashTreeTester::new_empty();
    tester.put_substate_changes(vec![change(2, 3, 4, Some(234))]);
    tester.put_substate_changes(vec![change(1, 9, 6, Some(196))]);
    let root_after_adding_sibling = tester.put_substate_changes(vec![change(2, 3, 5, Some(235))]);

    // We did [2:3:4] + [1:9:6] + [2:3:5] = [1:9:6, 2:3:4, 2:3:5] (i.e. same state).
    assert_eq!(root_after_adding_sibling, reference_root);
}

#[test]
fn hash_differs_when_states_only_differ_by_node_key() {
    let mut tester_1 = HashTreeTester::new_empty();
    let hash_1 = tester_1.put_substate_changes(vec![change(1, 6, 3, Some(30))]);
    let mut tester_2 = HashTreeTester::new_empty();
    let hash_2 = tester_2.put_substate_changes(vec![change(2, 6, 3, Some(30))]);
    assert_ne!(hash_1, hash_2);
}

#[test]
fn hash_differs_when_states_only_differ_by_partition_num() {
    let mut tester_1 = HashTreeTester::new_empty();
    let hash_1 = tester_1.put_substate_changes(vec![change(1, 6, 3, Some(30))]);
    let mut tester_2 = HashTreeTester::new_empty();
    let hash_2 = tester_2.put_substate_changes(vec![change(1, 7, 3, Some(30))]);
    assert_ne!(hash_1, hash_2);
}

#[test]
fn hash_differs_when_states_only_differ_by_sort_key() {
    let mut tester_1 = HashTreeTester::new_empty();
    let hash_1 = tester_1.put_substate_changes(vec![change(1, 6, 2, Some(30))]);
    let mut tester_2 = HashTreeTester::new_empty();
    let hash_2 = tester_2.put_substate_changes(vec![change(1, 6, 3, Some(30))]);
    assert_ne!(hash_1, hash_2);
}

#[test]
fn hash_of_different_re_nodes_is_same_when_contained_entries_are_same() {
    let mut tester = HashTreeTester::new_empty();
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
    let mut tester = HashTreeTester::new_empty();
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
                bytes: vec![1, 3, 3, 7, TIER_SEP, 99]
            },
            LeafKey {
                bytes: vec![123, 12, 1, 0, TIER_SEP, 66]
            },
            LeafKey {
                bytes: vec![123, 12, 1, 0, TIER_SEP, 88]
            },
        )
    );

    // Assert on the keys and hashes of the Substate-tier leaves:
    let substate_tier_leaves = tester.get_leafs_of_tier(Tier::Substate);
    assert_eq!(
        substate_tier_leaves,
        hashmap!(
            LeafKey { bytes: vec![1, 3, 3, 7, TIER_SEP, 99, TIER_SEP, 253] } => hash([1]),
            LeafKey { bytes: vec![1, 3, 3, 7, TIER_SEP, 99, TIER_SEP, 66] } => hash([2]),
            LeafKey { bytes: vec![123, 12, 1, 0, TIER_SEP, 88, TIER_SEP, 6, 6, 6] } => hash([3]),
            LeafKey { bytes: vec![123, 12, 1, 0, TIER_SEP, 88, TIER_SEP, 6, 6, 7] } => hash([4]),
            LeafKey { bytes: vec![123, 12, 1, 0, TIER_SEP, 66, TIER_SEP, 1, 2, 3] } => hash([5]),
        )
    );
}

#[test]
fn deletes_node_tier_leaf_when_all_its_entries_deleted() {
    let mut tester = HashTreeTester::new_empty();
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
    let mut tester = HashTreeTester::new_empty();
    let hash_v1 = tester.put_substate_changes(vec![]);
    assert_eq!(hash_v1, SPARSE_MERKLE_PLACEHOLDER_HASH);
    let hash_v2 = tester.put_substate_changes(vec![change(1, 6, 2, Some(30))]);
    assert_ne!(hash_v2, SPARSE_MERKLE_PLACEHOLDER_HASH);
    let hash_v3 = tester.put_substate_changes(vec![change(1, 6, 2, None)]);
    assert_eq!(hash_v3, SPARSE_MERKLE_PLACEHOLDER_HASH);
}

#[test]
fn records_stale_tree_node_keys() {
    let mut tester = HashTreeTester::new_empty();
    tester.put_substate_changes(vec![change(4, 1, 6, Some(30))]);
    tester.put_substate_changes(vec![change(3, 2, 9, Some(70))]);
    tester.put_substate_changes(vec![change(3, 2, 9, Some(80))]);
    let stale_versions = tester
        .tree_store
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
fn hash_computed_consistently_after_different_deletes() {
    // Reference: simply put [2:1:3, 2:3:4]
    let mut reference_tester = HashTreeTester::new_empty();
    let reference_root = reference_tester
        .put_substate_changes(vec![change(2, 1, 3, Some(213)), change(2, 3, 4, Some(234))]);

    // Delete 2 individual substates to arrive at the same state:
    let mut single_deletes_tester = HashTreeTester::new_empty();
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
    let mut delete_and_put_tester = HashTreeTester::new_empty();
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
    let mut reset_tester = HashTreeTester::new_empty();
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
    let mut tester = HashTreeTester::new_empty();
    tester.put_substate_changes(vec![
        change(4, 7, 6, Some(36)),
        change(4, 7, 7, Some(37)),
        change(4, 7, 8, Some(38)),
    ]);
    tester.reset_partition(from_seed(4), 7, vec![]);
    assert_eq!(
        tester.tree_store.stale_part_buffer,
        vec![
            // The entire subtree starting at the root of substate-tier JMT of partition `4:7`:
            StaleTreePart::Subtree(NodeKey::new(
                1,
                NibblePath::new_even([from_seed(4), vec![TIER_SEP, 7, TIER_SEP]].concat())
            )),
            // Regular single-node stale nodes up to the root, caused by hash update:
            StaleTreePart::Node(NodeKey::new(
                1,
                NibblePath::new_even([from_seed(4), vec![TIER_SEP]].concat())
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
    let mut tester = HashTreeTester::new(SerializedInMemoryTreeStore::new(), None);
    tester.put_substate_changes(vec![change(3, 6, 4, Some(90))]);
    let previous_keys = tester
        .tree_store
        .memory
        .keys()
        .cloned()
        .collect::<HashSet<_>>();
    tester.put_substate_changes(vec![change(1, 7, 2, Some(80))]);
    let min_next_key = tester
        .tree_store
        .memory
        .keys()
        .filter(|key| !previous_keys.contains(*key))
        .max()
        .unwrap();
    let max_previous_key = previous_keys.iter().max().unwrap();
    assert!(min_next_key > max_previous_key);
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
        (
            DbPartitionKey {
                node_key,
                partition_num,
            },
            DbSortKey(sort_key),
        ),
        value
            .map(|value| DatabaseUpdate::Set(value))
            .unwrap_or(DatabaseUpdate::Delete),
    )
}

fn from_seed(node_key_seed: u8) -> Vec<u8> {
    vec![node_key_seed; node_key_seed as usize]
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

const TIER_SEP: u8 = b'_';

pub struct HashTreeTester<S> {
    pub tree_store: S,
    pub current_version: Option<Version>,
}

impl<S: TreeStore> HashTreeTester<S> {
    pub fn new(tree_store: S, current_version: Option<Version>) -> Self {
        Self {
            tree_store,
            current_version,
        }
    }

    pub fn put_substate_changes(
        &mut self,
        changes: impl IntoIterator<Item = SingleSubstateChange>,
    ) -> Hash {
        self.apply_database_updates(&DatabaseUpdates::from_delta_maps(
            Self::index_to_delta_maps(changes),
        ))
    }

    pub fn reset_partition(
        &mut self,
        node_key: DbNodeKey,
        partition_num: DbPartitionNum,
        values: impl IntoIterator<Item = (DbSortKey, DbSubstateValue)>,
    ) -> Hash {
        self.apply_database_updates(&DatabaseUpdates {
            node_updates: indexmap!(
                node_key => NodeDatabaseUpdates {
                    partition_updates: indexmap!(
                        partition_num => PartitionDatabaseUpdates::Batch(
                            BatchPartitionDatabaseUpdate::Reset {
                                new_substate_values: values.into_iter().collect()
                            }
                        )
                    )
                }
            ),
        })
    }

    fn apply_database_updates(&mut self, database_updates: &DatabaseUpdates) -> Hash {
        let next_version = self.current_version.unwrap_or(0) + 1;
        let current_version = self.current_version.replace(next_version);
        put_at_next_version(&mut self.tree_store, current_version, database_updates)
    }

    fn index_to_delta_maps(
        changes: impl IntoIterator<Item = SingleSubstateChange>,
    ) -> IndexMap<DbNodeKey, IndexMap<DbPartitionNum, IndexMap<DbSortKey, DatabaseUpdate>>> {
        let mut delta_maps = index_map_new::<
            DbNodeKey,
            IndexMap<DbPartitionNum, IndexMap<DbSortKey, DatabaseUpdate>>,
        >();
        for change in changes {
            let (
                (
                    DbPartitionKey {
                        node_key,
                        partition_num,
                    },
                    sort_key,
                ),
                update,
            ) = change;
            delta_maps
                .entry(node_key)
                .or_default()
                .entry(partition_num)
                .or_default()
                .insert(sort_key, update);
        }
        delta_maps
    }
}

impl HashTreeTester<TypedInMemoryTreeStore> {
    pub fn new_empty() -> Self {
        Self::new(TypedInMemoryTreeStore::new(), None)
    }

    pub fn get_leafs_of_tier(&mut self, tier: Tier) -> HashMap<LeafKey, Hash> {
        let stale_node_keys = self
            .tree_store
            .stale_part_buffer
            .clone()
            .into_iter()
            .flat_map(|stale_part| match stale_part {
                StaleTreePart::Node(key) => vec![key],
                StaleTreePart::Subtree(key) => JellyfishMerkleTree::new(&mut self.tree_store)
                    .get_all_nodes_referenced(key)
                    .unwrap(),
            })
            .collect::<HashSet<_>>();
        let expected_separator_count = tier as usize;
        self.tree_store
            .tree_nodes
            .iter()
            .filter(|(key, _)| {
                let separator_count = key
                    .nibble_path()
                    .bytes()
                    .iter()
                    .filter(|byte| **byte == TIER_SEP)
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
}
