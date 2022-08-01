use sbor::rust::collections::*;
use sbor::rust::vec::Vec;
use scrypto::buffer::{scrypto_decode, scrypto_encode};

use crate::{engine::SubstateId, ledger::*};
use indexmap::IndexMap;

/// Nodes form an acyclic graph towards the parent
struct StagedSubstateStoreNode {
    parent_id: u64,
    locked: bool,
    outputs: IndexMap<SubstateId, OutputValue>,
}

impl StagedSubstateStoreNode {
    fn new(parent_id: u64) -> Self {
        StagedSubstateStoreNode {
            parent_id,
            locked: false,
            outputs: IndexMap::new(),
        }
    }
}

/// Structure which manages the acyclic graph
pub struct StagedSubstateStoreManager<'s, S: ReadableSubstateStore + WriteableSubstateStore> {
    parent: &'s mut S,
    nodes: HashMap<u64, StagedSubstateStoreNode>,
    cur_id: u64,
}

impl<'s, S: ReadableSubstateStore + WriteableSubstateStore> StagedSubstateStoreManager<'s, S> {
    pub fn new(parent: &'s mut S) -> Self {
        StagedSubstateStoreManager {
            parent,
            nodes: HashMap::new(),
            cur_id: 0,
        }
    }

    pub fn new_child_node(&mut self, parent_id: u64) -> u64 {
        if parent_id != 0 {
            let parent = self.nodes.get_mut(&parent_id).unwrap();
            parent.locked = true;
        }

        self.cur_id = self.cur_id + 1;
        self.nodes
            .insert(self.cur_id, StagedSubstateStoreNode::new(parent_id));
        self.cur_id
    }

    pub fn get_output_store<'t>(&'t mut self, id: u64) -> StagedSubstateStore<'t, 's, S> {
        if id != 0 && self.nodes.get(&id).unwrap().locked {
            panic!("Should not write to locked node");
        }

        StagedSubstateStore { stores: self, id }
    }

    fn remove_children(&mut self, id: u64) {
        let mut to_delete = Vec::new();
        for (to_delete_id, node) in &self.nodes {
            if node.parent_id == id {
                to_delete.push(*to_delete_id);
            }
        }
        for to_delete_id in to_delete {
            self.remove_children(to_delete_id);
            self.nodes.remove(&to_delete_id);
        }
    }

    fn set_root_parent(&mut self, id: u64) {
        for node in self.nodes.values_mut().filter(|node| id == node.parent_id) {
            node.parent_id = 0;
        }
    }

    pub fn merge_to_parent(&mut self, id: u64) {
        self.merge_to_parent_recurse(id, false)
    }

    fn merge_to_parent_recurse(&mut self, id: u64, remove_children: bool) {
        if id == 0 {
            if remove_children {
                self.remove_children(0);
            }
            return;
        }

        let node = self.nodes.remove(&id).unwrap();
        if remove_children {
            self.remove_children(id);
        }

        self.merge_to_parent_recurse(node.parent_id, true);

        for (substate_id, output) in node.outputs {
            self.parent.put_substate(substate_id, output);
        }

        if !remove_children {
            self.set_root_parent(id);
        }
    }
}

pub struct StagedSubstateStore<'t, 's, S: ReadableSubstateStore + WriteableSubstateStore> {
    stores: &'t mut StagedSubstateStoreManager<'s, S>,
    id: u64,
}

impl<'t, 's, S: ReadableSubstateStore + WriteableSubstateStore> StagedSubstateStore<'t, 's, S> {
    fn get_substate_recurse(&self, substate_id: &SubstateId, id: u64) -> Option<OutputValue> {
        if id == 0 {
            return self.stores.parent.get_substate(substate_id);
        }

        let node = self.stores.nodes.get(&id).unwrap();
        if let Some(output) = node.outputs.get(substate_id) {
            // TODO: Remove encoding/decoding
            let encoded_output = scrypto_encode(output);
            return Some(scrypto_decode(&encoded_output).unwrap());
        }

        self.get_substate_recurse(substate_id, node.parent_id)
    }
}

impl<'t, 's, S: ReadableSubstateStore + WriteableSubstateStore> ReadableSubstateStore
    for StagedSubstateStore<'t, 's, S>
{
    fn get_substate(&self, substate_id: &SubstateId) -> Option<OutputValue> {
        self.get_substate_recurse(substate_id, self.id)
    }
}

impl<'t, 's, S: ReadableSubstateStore + WriteableSubstateStore> WriteableSubstateStore
    for StagedSubstateStore<'t, 's, S>
{
    fn put_substate(&mut self, substate_id: SubstateId, output: OutputValue) {
        if self.id == 0 {
            self.stores.parent.put_substate(substate_id, output);
        } else {
            let node = self.stores.nodes.get_mut(&self.id).unwrap();
            node.outputs.insert(substate_id, output);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ledger::InMemorySubstateStore;
    use crate::state_manager::StagedSubstateStoreManager;

    #[test]
    fn test_complicated_merge() {
        // Arrange
        let mut store = InMemorySubstateStore::with_bootstrap();
        let mut stores = StagedSubstateStoreManager::new(&mut store);
        let child_node1 = stores.new_child_node(0);
        let child_node2 = stores.new_child_node(child_node1);
        let child_node3 = stores.new_child_node(child_node2);
        let _child_node4 = stores.new_child_node(child_node3);
        let child_node5 = stores.new_child_node(child_node3);
        let child_node6 = stores.new_child_node(child_node5);
        let child_node7 = stores.new_child_node(0);
        let _child_node8 = stores.new_child_node(child_node7);
        let child_node9 = stores.new_child_node(child_node6);
        let child_node10 = stores.new_child_node(child_node9);

        // Act
        stores.merge_to_parent(child_node5);

        // Assert
        assert_eq!(stores.nodes.len(), 3);
        let node = stores.nodes.get(&child_node6).expect("Should exist");
        assert_eq!(node.parent_id, 0);
        let node = stores.nodes.get(&child_node9).expect("Should exist");
        assert_eq!(node.parent_id, child_node6);
        let node = stores.nodes.get(&child_node10).expect("Should exist");
        assert_eq!(node.parent_id, child_node9);
    }
}
