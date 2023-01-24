use crate::transaction::TransactionReceipt;
use crate::types::*;
use crate::{ledger::*, transaction::TransactionResult};

use im::hashmap::HashMap as ImmutableHashMap;
use sbor::rust::vec::Vec;
use slotmap::{new_key_type, SlotMap};

/// An immutable/persistent store (i.e a store built from a [`parent`] store
/// shares data with it). Note: while from the abstract representation point
/// of view you can delete keys, the underlying data is freed only and only
/// if there are no references to it (there are no paths from any root reference
/// to the node representing that (key, value)). For this reason, extra steps
/// are taken to free up (key, value) pairs accumulating over time.
/// This is intended as an wrapper/abstraction layer, so that changes to the
/// ReadableSubstateStore/WriteableSubstateStore traits are easier to maintain.
#[derive(Clone)]
struct ImmutableStore {
    outputs: ImmutableHashMap<SubstateId, OutputValue>,
}

impl ImmutableStore {
    fn new() -> Self {
        ImmutableStore {
            outputs: ImmutableHashMap::new(),
        }
    }

    fn from_parent(parent: &ImmutableStore) -> Self {
        ImmutableStore {
            // Note: this clone is O(1), only the root node is actually cloned
            // Check im::collections::HashMap for details
            outputs: parent.outputs.clone(),
        }
    }
}

impl WriteableSubstateStore for ImmutableStore {
    fn put_substate(&mut self, substate_id: SubstateId, output: OutputValue) {
        self.outputs.insert(substate_id, output);
    }
}

impl ReadableSubstateStore for ImmutableStore {
    fn get_substate(&self, substate_id: &SubstateId) -> Option<OutputValue> {
        self.outputs.get(substate_id).cloned()
    }
}

new_key_type! {
    pub struct StagedSubstateStoreNodeKey;
}

/// Because the root store (which eventually is saved on disk) is not an
/// StagedSubstateStoreNode/ImmutableStore we need to be able to
/// distinguish it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StagedSubstateStoreKey {
    RootStoreKey,
    InternalNodeStoreKey(StagedSubstateStoreNodeKey),
}

/// Nodes form a tree towards the [`parent_key`]. If it is of type
/// StagedSubstateStoreKey::RootStoreKey than the parent is the root store
/// living in the StagedSubstateStoreManager.
/// [`parent_key`] and [`children_keys`] are needed in order to traverse
/// the tree.
/// [`receipt`] applied to the parent's store, results in this node store.
/// We need to keep the [`receipt`] to both recompute the stores when doing
/// the data reconstruction/"garbage collection" but also to retrieve it
/// when caching.
pub struct StagedSubstateStoreNode {
    parent_key: StagedSubstateStoreKey,
    children_keys: Vec<StagedSubstateStoreNodeKey>,
    receipt: TransactionReceipt,
    store: ImmutableStore,
}

impl StagedSubstateStoreNode {
    fn new(
        parent_key: StagedSubstateStoreKey,
        receipt: TransactionReceipt,
        mut store: ImmutableStore,
    ) -> Self {
        if let TransactionResult::Commit(commit) = &receipt.result {
            commit.state_updates.commit(&mut store);
        }
        StagedSubstateStoreNode {
            parent_key,
            children_keys: Vec::new(),
            receipt,
            store,
        }
    }

    /// Weight is defined as the number of changes to the ImmutableStore
    /// done exclusively by this node.
    fn weight(&self) -> usize {
        match &self.receipt.result {
            // NOTE for future Substate delete support: add down_substates.len()
            // to the weight as well.
            TransactionResult::Commit(commit) => commit.state_updates.up_substates.len(),
            TransactionResult::Reject(_) => 0,
            TransactionResult::Abort(_) => 0,
        }
    }
}

/// Structure which manages the staged store tree
pub struct StagedSubstateStoreManager<S: ReadableSubstateStore> {
    pub root: S,
    nodes: SlotMap<StagedSubstateStoreNodeKey, StagedSubstateStoreNode>,
    children_keys: Vec<StagedSubstateStoreNodeKey>,
    dead_weight: usize,
    total_weight: usize,
}

impl<S: ReadableSubstateStore> StagedSubstateStoreManager<S> {
    pub fn new(root: S) -> Self {
        StagedSubstateStoreManager {
            root,
            nodes: SlotMap::with_capacity_and_key(1000),
            children_keys: Vec::new(),
            dead_weight: 0,
            total_weight: 0,
        }
    }

    pub fn new_child_node(
        &mut self,
        parent_key: StagedSubstateStoreKey,
        receipt: TransactionReceipt,
    ) -> StagedSubstateStoreKey {
        let store = match parent_key {
            StagedSubstateStoreKey::RootStoreKey => ImmutableStore::new(),
            StagedSubstateStoreKey::InternalNodeStoreKey(parent_key) => {
                ImmutableStore::from_parent(&self.nodes.get(parent_key).unwrap().store)
            }
        };

        // Build new node by applying the receipt to the parent store
        let new_node = StagedSubstateStoreNode::new(parent_key, receipt, store);

        // Update the `total_weight` of the tree
        self.total_weight += new_node.weight();
        let new_node_key = self.nodes.insert(new_node);
        match parent_key {
            StagedSubstateStoreKey::RootStoreKey => {
                self.children_keys.push(new_node_key);
            }
            StagedSubstateStoreKey::InternalNodeStoreKey(parent_key) => {
                let parent_node = self.nodes.get_mut(parent_key).unwrap();
                parent_node.children_keys.push(new_node_key);
            }
        }
        StagedSubstateStoreKey::InternalNodeStoreKey(new_node_key)
    }

    pub fn get_store<'t>(&'t self, key: StagedSubstateStoreKey) -> StagedSubstateStore<'t, S> {
        StagedSubstateStore { manager: self, key }
    }

    pub fn get_receipt(&self, key: &StagedSubstateStoreKey) -> Option<&TransactionReceipt> {
        match self.get_node(key) {
            None => None,
            Some(node) => Some(&node.receipt),
        }
    }

    pub fn get_node(&self, key: &StagedSubstateStoreKey) -> Option<&StagedSubstateStoreNode> {
        match key {
            StagedSubstateStoreKey::RootStoreKey => None,
            StagedSubstateStoreKey::InternalNodeStoreKey(key) => self.nodes.get(*key),
        }
    }

    fn recompute_data_recursive(
        nodes: &mut SlotMap<StagedSubstateStoreNodeKey, StagedSubstateStoreNode>,
        node_key: StagedSubstateStoreNodeKey,
    ) {
        let parent_store = ImmutableStore::from_parent(&nodes.get(node_key).unwrap().store);

        let children_keys = nodes.get(node_key).unwrap().children_keys.clone();
        for child_key in children_keys.iter() {
            let child_node = nodes.get_mut(*child_key).unwrap();
            child_node.store = parent_store.clone();
            if let TransactionResult::Commit(commit) = &child_node.receipt.result {
                commit.state_updates.commit(&mut child_node.store);
            }
            Self::recompute_data_recursive(nodes, *child_key);
        }
    }

    /// Rebuilds ImmutableStores by starting from the root with new, empty ones
    /// and recursively reapplies the [`receipt`]s.
    fn recompute_data(&mut self) {
        // Reset the [`dead_weight`]
        self.dead_weight = 0;

        for node_key in self.children_keys.iter() {
            let node = self.nodes.get_mut(*node_key).unwrap();
            node.store = ImmutableStore::new();
            if let TransactionResult::Commit(commit) = &node.receipt.result {
                commit.state_updates.commit(&mut node.store);
            }
            Self::recompute_data_recursive(&mut self.nodes, *node_key);
        }
    }

    fn remove_node<CB>(
        nodes: &mut SlotMap<StagedSubstateStoreNodeKey, StagedSubstateStoreNode>,
        total_weight: &mut usize,
        callback: &mut CB,
        node_key: &StagedSubstateStoreNodeKey,
    ) where
        CB: FnMut(&StagedSubstateStoreNodeKey),
    {
        *total_weight -= nodes.get(*node_key).unwrap().weight();
        nodes.remove(*node_key);
        callback(node_key);
    }

    /// Recursively deletes all nodes that are not in new_root_key subtree and returns the
    /// sum of weights from current root to new_root_key. Updates to ImmutableStore on this
    /// path will persist even after deleting the nodes.
    fn delete_recursive<CB>(
        nodes: &mut SlotMap<StagedSubstateStoreNodeKey, StagedSubstateStoreNode>,
        total_weight: &mut usize,
        new_root_key: &StagedSubstateStoreNodeKey,
        callback: &mut CB,
        node_key: &StagedSubstateStoreNodeKey,
        root_path_weight_sum: usize,
    ) -> usize
    where
        CB: FnMut(&StagedSubstateStoreNodeKey),
    {
        let root_path_weight_sum = root_path_weight_sum + nodes.get(*node_key).unwrap().weight();
        if *node_key == *new_root_key {
            return root_path_weight_sum;
        }

        let mut dead_weight = 0;
        let children_keys = nodes.get(*node_key).unwrap().children_keys.clone();
        for child_key in children_keys {
            // Instead of doing max([0, 0, 0, max, 0,.. 0]) we can do sum()
            dead_weight += Self::delete_recursive(
                nodes,
                total_weight,
                new_root_key,
                callback,
                &child_key,
                root_path_weight_sum,
            );
        }

        Self::remove_node(nodes, total_weight, callback, node_key);

        dead_weight
    }

    /// Each node created via [`new_child_node`] represents one store state. At some point (e.g
    /// in `commit` step) after creating multiple versions (e.g in `prepare` step), we want to
    /// move the chain of state changes from the staging store into the real store.
    /// While the changes to the real store are out of scope for this structure and done
    /// separately, we still need to inform the staging store about what current version the
    /// root store is pointing to in order for it to be able to drop no longer relevant branches.
    /// Note that because retroactive deletion for a history of persistent/immutable data
    /// structure is not possible, it is not guaranteed that the chain of state changes
    /// ([`ImmutableStore`]s. [`StagedSubstateStoreNode`]s however, are always deleted) committed
    /// to the real store are discarded (every time `reparent` is called).
    /// This does not really matter from a correctness perspective (the staging store
    /// will act as a cache for the real store) but as an memory overhead. The memory
    /// is freed when [`recompute_data`] is called (which is called so that the overall
    /// cost is amortized).
    /// To better understand please check:
    /// Diagram here: https://whimsical.com/persistent-staged-store-amortized-reparenting-Lyc6gRgVXVzLdqWvwVT3v4
    /// And `test_complicated_reparent` unit test
    pub fn reparent<CB>(&mut self, new_root_key: StagedSubstateStoreKey, callback: &mut CB)
    where
        CB: FnMut(&StagedSubstateStoreNodeKey),
    {
        match new_root_key {
            StagedSubstateStoreKey::RootStoreKey => {}
            StagedSubstateStoreKey::InternalNodeStoreKey(new_root_key) => {
                // Delete all nodes that are not in new_root_key subtree
                for node_key in self.children_keys.iter() {
                    self.dead_weight += Self::delete_recursive(
                        &mut self.nodes,
                        &mut self.total_weight,
                        &new_root_key,
                        callback,
                        node_key,
                        0,
                    );
                }

                let new_root = self.nodes.get(new_root_key).unwrap();
                // Reparent to new_root node and delete it
                self.children_keys = new_root.children_keys.clone();
                for key in self.children_keys.iter() {
                    let node = self.nodes.get_mut(*key).unwrap();
                    node.parent_key = StagedSubstateStoreKey::RootStoreKey;
                }

                Self::remove_node(
                    &mut self.nodes,
                    &mut self.total_weight,
                    callback,
                    &new_root_key,
                );

                // If the number of state changes that overlap with the self.root (dead_weight) store is greater
                // than the number of state changes applied on top of it (total_weight), we recalculate the
                // ImmutableStores in order to free up memory.
                if self.dead_weight > self.total_weight {
                    self.recompute_data();
                }
            }
        }
    }
}

pub struct StagedSubstateStore<'t, S: ReadableSubstateStore> {
    manager: &'t StagedSubstateStoreManager<S>,
    key: StagedSubstateStoreKey,
}

impl<'t, S: ReadableSubstateStore> ReadableSubstateStore for StagedSubstateStore<'t, S> {
    fn get_substate(&self, substate_id: &SubstateId) -> Option<OutputValue> {
        match self.key {
            StagedSubstateStoreKey::RootStoreKey => self.manager.root.get_substate(substate_id),
            StagedSubstateStoreKey::InternalNodeStoreKey(key) => {
                // NOTE for future Substate delete support: in order to properly reflect
                // deleted keys, a Sentinel/Tombstone value should be stored instead of
                // actually removing the key. When querying here, convert the Tombstone back
                // into a None (Option can be used as the Tombstone).
                match self
                    .manager
                    .nodes
                    .get(key)
                    .unwrap()
                    .store
                    .get_substate(substate_id)
                {
                    Some(output_value) => Some(output_value),
                    None => self.manager.root.get_substate(substate_id),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::engine::ScryptoInterpreter;
    use crate::fee::FeeSummary;
    use crate::ledger::{OutputValue, ReadableSubstateStore, TypedInMemorySubstateStore};
    use crate::model::{PersistedSubstate, Resource, VaultSubstate};
    use crate::state_manager::{StagedSubstateStoreKey, StagedSubstateStoreManager, StateDiff};
    use crate::transaction::{
        CommitResult, EntityChanges, TransactionExecution, TransactionOutcome, TransactionReceipt,
        TransactionResult,
    };
    use crate::types::rust::iter::zip;
    use crate::wasm::DefaultWasmEngine;
    use radix_engine_interface::api::types::{RENodeId, SubstateId, SubstateOffset, VaultOffset};
    use radix_engine_interface::math::Decimal;
    use radix_engine_interface::model::ResourceAddress;
    use sbor::rust::collections::BTreeMap;
    use sbor::rust::collections::HashMap;
    use sbor::rust::vec::Vec;

    fn build_transaction_receipt_from_state_diff(state_diff: StateDiff) -> TransactionReceipt {
        TransactionReceipt {
            execution: TransactionExecution {
                fee_summary: FeeSummary {
                    cost_unit_price: Decimal::default(),
                    tip_percentage: 0,
                    cost_unit_limit: 10,
                    cost_unit_consumed: 1,
                    total_execution_cost_xrd: Decimal::default(),
                    total_royalty_cost_xrd: Decimal::default(),
                    bad_debt_xrd: Decimal::default(),
                    vault_locks: Vec::new(),
                    vault_payments_xrd: None,
                    execution_cost_unit_breakdown: HashMap::new(),
                    royalty_cost_unit_breakdown: HashMap::new(),
                },
                events: Vec::new(),
            },
            result: TransactionResult::Commit(CommitResult {
                application_logs: Vec::new(),
                next_epoch: None,
                outcome: TransactionOutcome::Success(Vec::new()),
                state_updates: state_diff,
                entity_changes: EntityChanges {
                    new_component_addresses: Vec::new(),
                    new_package_addresses: Vec::new(),
                    new_resource_addresses: Vec::new(),
                },
                resource_changes: Vec::new(),
            }),
        }
    }

    fn build_dummy_substate_id(id: [u8; 36]) -> SubstateId {
        SubstateId {
            0: RENodeId::Vault(id),
            1: SubstateOffset::Vault(VaultOffset::Vault),
        }
    }

    fn build_dummy_output_value(version: u32) -> OutputValue {
        OutputValue {
            substate: PersistedSubstate::Vault(VaultSubstate(Resource::Fungible {
                resource_address: ResourceAddress::Normal([2u8; 26]),
                divisibility: 56,
                amount: Decimal::one(),
            })),
            version,
        }
    }

    #[derive(Clone)]
    struct TestNodeData {
        parent_id: usize,
        updates: Vec<(usize, usize)>,
    }

    #[test]
    fn test_complicated_reparent() {
        // Arrange
        let scrypto_interpreter = ScryptoInterpreter::<DefaultWasmEngine>::default();
        let store = TypedInMemorySubstateStore::with_bootstrap(&scrypto_interpreter);
        let mut manager = StagedSubstateStoreManager::new(store);

        let substate_ids: Vec<SubstateId> = (0u8..5u8)
            .into_iter()
            .map(|id| build_dummy_substate_id([id; 36]))
            .collect();
        let output_values: Vec<OutputValue> = (0u32..5u32)
            .into_iter()
            .map(|version| build_dummy_output_value(version))
            .collect();

        let node_test_data = [
            TestNodeData {
                // child_node[1]
                parent_id: 0, // root
                updates: [
                    (0, 1), // manager.get_store(child_node[1]).get_substate(substate_ids[0]) == output_values[1]
                ]
                .to_vec(),
            },
            TestNodeData {
                // child_node[2]
                parent_id: 1,
                updates: [(0, 2), (2, 0)].to_vec(),
            },
            TestNodeData {
                // child_node[3]
                parent_id: 2,
                updates: [(3, 1), (4, 3), (0, 3)].to_vec(),
            },
            TestNodeData {
                // child_node[4]
                parent_id: 3,
                updates: [(0, 4), (1, 3), (2, 2), (3, 1), (4, 0)].to_vec(),
            },
            TestNodeData {
                // child_node[5]
                parent_id: 4,
                updates: [(2, 1), (0, 3)].to_vec(),
            },
            TestNodeData {
                // child_node[6]
                parent_id: 5,
                updates: [(2, 2), (3, 4)].to_vec(),
            },
            TestNodeData {
                // child_node[7]
                parent_id: 0, // root
                updates: [(2, 2)].to_vec(),
            },
            TestNodeData {
                // child_node[8]
                parent_id: 7,
                updates: [(2, 1)].to_vec(),
            },
            TestNodeData {
                // child_node[9]
                parent_id: 6,
                updates: [(2, 3), (4, 4)].to_vec(),
            },
            TestNodeData {
                // child_node[10]
                parent_id: 9,
                updates: [(2, 0)].to_vec(),
            },
        ]
        .to_vec();

        let mut expected_total_weight = 0;
        let mut child_node = [StagedSubstateStoreKey::RootStoreKey].to_vec();
        let mut expected_node_states = [BTreeMap::new()].to_vec();
        let mut expected_weights = [0].to_vec();
        for node_data in node_test_data.iter() {
            let up_substates: BTreeMap<SubstateId, OutputValue> = node_data
                .updates
                .iter()
                .map(|(substate_id, output_id)| {
                    (
                        substate_ids[*substate_id].clone(),
                        output_values[*output_id].clone(),
                    )
                })
                .collect();
            let state_diff = StateDiff {
                up_substates: up_substates.clone(),
                down_substates: Vec::new(),
            };
            let new_child_node = manager.new_child_node(
                child_node[node_data.parent_id],
                build_transaction_receipt_from_state_diff(state_diff.clone()),
            );
            child_node.push(new_child_node);

            let mut expected_node_state = expected_node_states[node_data.parent_id].clone();
            expected_node_state.extend(up_substates);

            expected_node_states.push(expected_node_state);
            expected_weights.push(node_data.updates.len());
            expected_total_weight += node_data.updates.len();

            // check that all stores have the expected state
            for (child_node, expected_node_state) in
                zip(child_node.iter(), expected_node_states.iter())
            {
                let store = manager.get_store(*child_node);
                for (substate_id, output_value) in expected_node_state.iter() {
                    assert_eq!(
                        store.get_substate(substate_id),
                        Some((*output_value).clone())
                    );
                }
            }

            assert_eq!(manager.total_weight, expected_total_weight);
            assert_eq!(manager.dead_weight, 0);
        }

        // State tree layout:
        // root -> 1 -> 2 -> 3 -> 4
        //      │            └──> 5 -> 6 -> 9 -> 10
        //      └> 7 -> 8
        // After reparenting to 3: 7 and 8 are discarded completely. 1, 2 and 3 discarded but leave dead weight behind
        manager.reparent(child_node[3], &mut |_| {});
        let expected_dead_weight = [1, 2, 3]
            .iter()
            .fold(0, |acc, node_id| acc + expected_weights[*node_id]);
        expected_total_weight -= [1, 2, 3, 7, 8]
            .iter()
            .fold(0, |acc, node_id| acc + expected_weights[*node_id]);
        assert_eq!(manager.total_weight, expected_total_weight);
        assert_eq!(manager.dead_weight, expected_dead_weight);
        assert_eq!(manager.nodes.len(), 5);

        // After reparenting to 5: node 4 gets discarded completely. Node 5 is discarded and added to the dead weight.
        // This should trigger the recomputation/garbage collection.
        manager.reparent(child_node[5], &mut |_| {});
        expected_total_weight -= [4, 5]
            .iter()
            .fold(0, |acc, node_id| acc + expected_weights[*node_id]);
        assert_eq!(manager.total_weight, expected_total_weight);
        assert_eq!(manager.dead_weight, 0);
        assert_eq!(manager.nodes.len(), 3);

        let node = manager.get_node(&child_node[6]).expect("Should exist");
        assert_eq!(node.parent_key, StagedSubstateStoreKey::RootStoreKey);
        let node = manager.get_node(&child_node[9]).expect("Should exist");
        assert_eq!(node.parent_key, child_node[6]);
        let node = manager.get_node(&child_node[10]).expect("Should exist");
        assert_eq!(node.parent_key, child_node[9]);
    }
}
