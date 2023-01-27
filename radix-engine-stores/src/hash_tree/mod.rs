use crate::hash_tree::jellyfish::JellyfishMerkleTree;
use crate::hash_tree::tree_store::{TreeNode, TreeStore};
use crate::hash_tree::types::Version;
use radix_engine_interface::api::types::SubstateId;
use radix_engine_interface::crypto::{hash, Hash};
use radix_engine_interface::data::scrypto_encode;
use sbor::rust::vec::Vec;

pub mod hash_tree_facade;
pub mod tree_store;

// The sources copied from Aptos (the `jellyfish` and `types` modules) contain support for
// generating proofs, which we plan to use in near future. Hence, we do not delete that code, but
// suppress warnings.

#[allow(dead_code)]
mod jellyfish;
#[cfg(test)]
mod test;
#[allow(dead_code)]
mod types;

/// Inserts a new set of nodes at version `current_version` + 1 into the tree
/// persisted within the given `store`.
/// This inserts a new leaf node for each given "change", together with an
/// entire new "parent chain" leading from that leaf to a new root node (common
/// for all of them).
/// Each change may either create/update a substate's value (denoted by
/// `Some(hash(scrypto_encode(value)))`), or delete a substate (denoted by
/// `None`).
/// All nodes that became stale precisely due to this (i.e. not any previous)
/// operation will be reported before the function returns (see
/// `WriteableTreeStore::record_stale_node`).
/// Returns the hash of the newly-created root (i.e. representing state at
/// version `current_version` + 1).
///
/// # Panics
/// Panics if a root node for `current_version` does not exist. The caller
/// should use `None` to denote an empty, initial state of the tree (i.e.
/// inserting at version 1).
pub fn put_at_next_version<S: TreeStore>(
    store: &mut S,
    current_version: Option<Version>,
    changes: &[(SubstateId, Option<Hash>)],
) -> Hash {
    let value_set: Vec<(Hash, Option<(Hash, SubstateId)>)> = changes
        .iter()
        .map(|(id, value_hash)| {
            (
                hash(scrypto_encode(id).unwrap()),
                value_hash.map(|value_hash| (value_hash, id.clone())),
            )
        })
        .collect();
    let (root_hash, update_result) = JellyfishMerkleTree::new(store)
        .batch_put_value_set(
            value_set
                .iter()
                .map(|(x, y)| (x.clone(), y.as_ref()))
                .collect(),
            None,
            current_version,
            current_version.unwrap_or(0) + 1,
        )
        .expect("error while reading tree during put");
    for (key, node) in update_result.node_batch.iter().flatten() {
        store.insert_node(key, TreeNode::from(key, node));
    }
    for stale_node in update_result.stale_node_index_batch.iter().flatten() {
        store.record_stale_node(&stale_node.node_key);
    }
    root_hash
}
