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
            TransactionResult::Commit(commit) => commit.state_updates.up_substates.len(),
            TransactionResult::Reject(_) => 0,
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

pub trait StagedSubstateStoreVisitor {
    fn remove_node(&mut self, key: &StagedSubstateStoreNodeKey);
}

pub struct StagedSubstateStoreIgnoreVisitor {}

impl StagedSubstateStoreVisitor for StagedSubstateStoreIgnoreVisitor {
    fn remove_node(&mut self, _key: &StagedSubstateStoreNodeKey) {}
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
        let new_node = StagedSubstateStoreNode::new(parent_key, receipt, store);
        // Update the total total_weight of the tree
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
    /// and recursively reapplies the [`state_diffs`]s.
    pub fn recompute_data(&mut self) {
        // Reset the [`dead_weight`]
        self.dead_weight = 0;

        for node_key in self.children_keys.iter() {
            let node = self.nodes.get_mut(*node_key).unwrap();
            node.store = ImmutableStore::new();
            Self::recompute_data_recursive(&mut self.nodes, *node_key);
        }
    }

    fn remove_node<V: StagedSubstateStoreVisitor>(
        nodes: &mut SlotMap<StagedSubstateStoreNodeKey, StagedSubstateStoreNode>,
        total_weight: &mut usize,
        visitor: &mut V,
        node_key: &StagedSubstateStoreNodeKey,
    ) {
        *total_weight -= nodes.get(*node_key).unwrap().weight();
        nodes.remove(*node_key);
        visitor.remove_node(node_key);
    }

    /// Recursively deletes all nodes that are not in new_root_key subtree and returns the
    /// sum of weights from current root to new_root_key. Updates to ImmutableStore on this
    /// path will persist even after deleting the nodes.
    fn delete_recursive<V: StagedSubstateStoreVisitor>(
        nodes: &mut SlotMap<StagedSubstateStoreNodeKey, StagedSubstateStoreNode>,
        total_weight: &mut usize,
        new_root_key: &StagedSubstateStoreNodeKey,
        visitor: &mut V,
        node_key: &StagedSubstateStoreNodeKey,
        root_path_weight_sum: usize,
    ) -> usize {
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
                visitor,
                &child_key,
                root_path_weight_sum,
            );
        }

        Self::remove_node(nodes, total_weight, visitor, node_key);

        dead_weight
    }

    pub fn reparent<V: StagedSubstateStoreVisitor>(
        &mut self,
        new_root_key: StagedSubstateStoreKey,
        visitor: &mut V,
    ) {
        match new_root_key {
            StagedSubstateStoreKey::RootStoreKey => {}
            StagedSubstateStoreKey::InternalNodeStoreKey(new_root_key) => {
                // Delete all nodes that are not in new_root_key subtree
                for node_key in self.children_keys.iter() {
                    self.dead_weight += Self::delete_recursive(
                        &mut self.nodes,
                        &mut self.total_weight,
                        &new_root_key,
                        visitor,
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
                    visitor,
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
    use crate::ledger::{OutputValue, TypedInMemorySubstateStore};
    use crate::model::{PersistedSubstate, Resource, VaultSubstate};
    use crate::state_manager::{
        StagedSubstateStoreIgnoreVisitor, StagedSubstateStoreKey, StagedSubstateStoreManager,
        StateDiff,
    };
    use crate::transaction::{
        CommitResult, EntityChanges, TransactionExecution, TransactionOutcome, TransactionReceipt,
        TransactionResult,
    };
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
                next_validator_set: None,
                outcome: TransactionOutcome::Success(Vec::new()),
                state_updates: state_diff,
                entity_changes: EntityChanges {
                    new_component_addresses: Vec::new(),
                    new_package_addresses: Vec::new(),
                    new_resource_addresses: Vec::new(),
                    new_system_addresses: Vec::new(),
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

        let node1_state_diff = StateDiff {
            up_substates: BTreeMap::from([(substate_ids[0].clone(), output_values[0].clone())]),
            down_substates: Vec::new(),
        };
        let mut expected_total_weight = 0;
        let child_node1 = manager.new_child_node(
            StagedSubstateStoreKey::RootStoreKey,
            build_transaction_receipt_from_state_diff(node1_state_diff.clone()),
        );
        expected_total_weight += node1_state_diff.up_substates.len();
        assert_eq!(manager.total_weight, expected_total_weight);

        let node2_state_diff = StateDiff {
            up_substates: BTreeMap::from([
                (substate_ids[1].clone(), output_values[0].clone()),
                (substate_ids[2].clone(), output_values[2].clone()),
            ]),
            down_substates: Vec::new(),
        };
        let child_node2 = manager.new_child_node(
            child_node1,
            build_transaction_receipt_from_state_diff(node2_state_diff.clone()),
        );
        expected_total_weight += node2_state_diff.up_substates.len();
        assert_eq!(manager.total_weight, expected_total_weight);

        let node3_state_diff = StateDiff {
            up_substates: BTreeMap::from([
                (substate_ids[3].clone(), output_values[1].clone()),
                (substate_ids[4].clone(), output_values[3].clone()),
                (substate_ids[0].clone(), output_values[3].clone()),
            ]),
            down_substates: Vec::new(),
        };
        let child_node3 = manager.new_child_node(
            child_node2,
            build_transaction_receipt_from_state_diff(node3_state_diff.clone()),
        );
        expected_total_weight += node3_state_diff.up_substates.len();
        assert_eq!(manager.total_weight, expected_total_weight);

        let node4_state_diff = StateDiff {
            up_substates: BTreeMap::from([
                (substate_ids[0].clone(), output_values[4].clone()),
                (substate_ids[1].clone(), output_values[3].clone()),
                (substate_ids[2].clone(), output_values[2].clone()),
                (substate_ids[3].clone(), output_values[1].clone()),
                (substate_ids[4].clone(), output_values[0].clone()),
            ]),
            down_substates: Vec::new(),
        };
        let _child_node4 = manager.new_child_node(
            child_node3,
            build_transaction_receipt_from_state_diff(node4_state_diff.clone()),
        );
        expected_total_weight += node4_state_diff.up_substates.len();
        assert_eq!(manager.total_weight, expected_total_weight);

        let node5_state_diff = StateDiff {
            up_substates: BTreeMap::from([(substate_ids[2].clone(), output_values[0].clone())]),
            down_substates: Vec::new(),
        };
        let child_node5 = manager.new_child_node(
            child_node3,
            build_transaction_receipt_from_state_diff(node5_state_diff.clone()),
        );
        expected_total_weight += node5_state_diff.up_substates.len();
        assert_eq!(manager.total_weight, expected_total_weight);

        let node6_state_diff = StateDiff {
            up_substates: BTreeMap::from([(substate_ids[2].clone(), output_values[0].clone())]),
            down_substates: Vec::new(),
        };
        let child_node6 = manager.new_child_node(
            child_node5,
            build_transaction_receipt_from_state_diff(node6_state_diff.clone()),
        );
        expected_total_weight += node6_state_diff.up_substates.len();
        assert_eq!(manager.total_weight, expected_total_weight);

        let node7_state_diff = StateDiff {
            up_substates: BTreeMap::from([(substate_ids[2].clone(), output_values[0].clone())]),
            down_substates: Vec::new(),
        };
        let child_node7 = manager.new_child_node(
            StagedSubstateStoreKey::RootStoreKey,
            build_transaction_receipt_from_state_diff(node7_state_diff.clone()),
        );
        expected_total_weight += node7_state_diff.up_substates.len();
        assert_eq!(manager.total_weight, expected_total_weight);

        let node8_state_diff = StateDiff {
            up_substates: BTreeMap::from([(substate_ids[2].clone(), output_values[0].clone())]),
            down_substates: Vec::new(),
        };
        let _child_node8 = manager.new_child_node(
            child_node7,
            build_transaction_receipt_from_state_diff(node8_state_diff.clone()),
        );
        expected_total_weight += node8_state_diff.up_substates.len();
        assert_eq!(manager.total_weight, expected_total_weight);

        let node9_state_diff = StateDiff {
            up_substates: BTreeMap::from([(substate_ids[2].clone(), output_values[0].clone())]),
            down_substates: Vec::new(),
        };
        let child_node9 = manager.new_child_node(
            child_node6,
            build_transaction_receipt_from_state_diff(node9_state_diff.clone()),
        );
        expected_total_weight += node9_state_diff.up_substates.len();
        assert_eq!(manager.total_weight, expected_total_weight);

        let node10_state_diff = StateDiff {
            up_substates: BTreeMap::from([(substate_ids[2].clone(), output_values[0].clone())]),
            down_substates: Vec::new(),
        };
        let child_node10 = manager.new_child_node(
            child_node9,
            build_transaction_receipt_from_state_diff(node10_state_diff.clone()),
        );
        expected_total_weight += node10_state_diff.up_substates.len();
        assert_eq!(manager.total_weight, expected_total_weight);
        assert_eq!(manager.dead_weight, 0);

        let mut dummy_visitor = StagedSubstateStoreIgnoreVisitor {};
        // State tree layout:
        // root -> 1 -> 2 -> 3 -> 4
        //      │            └──> 5 -> 6 -> 9 -> 10
        //      └> 7 -> 8
        // After reparenting to 3: 7 and 8 are discarded completely. 1, 2 and 3 discarded but leave dead weight behind
        manager.reparent(child_node3, &mut dummy_visitor);
        let expected_dead_weight = [&node1_state_diff, &node2_state_diff, &node3_state_diff]
            .iter()
            .fold(0, |acc, state_diff| acc + state_diff.up_substates.len());
        expected_total_weight -= [
            &node1_state_diff,
            &node2_state_diff,
            &node3_state_diff,
            &node7_state_diff,
            &node8_state_diff,
        ]
        .iter()
        .fold(0, |acc, state_diff| acc + state_diff.up_substates.len());
        assert_eq!(manager.total_weight, expected_total_weight);
        assert_eq!(manager.dead_weight, expected_dead_weight);
        assert_eq!(manager.nodes.len(), 5);

        // After reparenting to 5: node 4 gets discarded completely. Node 5 is discarded and added to the dead weight.
        // This should trigger the recomputation/garbage collection.
        manager.reparent(child_node5, &mut dummy_visitor);
        expected_total_weight -= [&node4_state_diff, &node5_state_diff]
            .iter()
            .fold(0, |acc, state_diff| acc + state_diff.up_substates.len());
        assert_eq!(manager.total_weight, expected_total_weight);
        assert_eq!(manager.dead_weight, 0);
        assert_eq!(manager.nodes.len(), 3);

        let node = manager.get_node(&child_node6).expect("Should exist");
        assert_eq!(node.parent_key, StagedSubstateStoreKey::RootStoreKey);
        let node = manager.get_node(&child_node9).expect("Should exist");
        assert_eq!(node.parent_key, child_node6);
        let node = manager.get_node(&child_node10).expect("Should exist");
        assert_eq!(node.parent_key, child_node9);
    }
}
