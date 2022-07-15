use sbor::rust::collections::*;
use sbor::rust::vec::Vec;
use scrypto::buffer::{scrypto_decode, scrypto_encode};

use crate::ledger::*;

struct StagedExecutionStoreNode {
    parent_id: u64,
    locked: bool,
    spaces: BTreeMap<Vec<u8>, OutputId>,
    outputs: BTreeMap<Vec<u8>, Output>,
}

impl StagedExecutionStoreNode {
    fn new(parent_id: u64) -> Self {
        StagedExecutionStoreNode {
            parent_id,
            locked: false,
            spaces: BTreeMap::new(),
            outputs: BTreeMap::new(),
        }
    }
}

pub struct StagedExecutionStores<'s, S: ReadableSubstateStore + WriteableSubstateStore> {
    parent: &'s mut S,
    nodes: HashMap<u64, StagedExecutionStoreNode>,
    cur_id: u64,
}

impl<'s, S: ReadableSubstateStore + WriteableSubstateStore> StagedExecutionStores<'s, S> {
    pub fn new(parent: &'s mut S) -> Self {
        StagedExecutionStores {
            parent,
            nodes: HashMap::new(),
            cur_id: 0,
        }
    }

    pub fn new_branch(&mut self, parent_id: u64) -> u64 {
        if parent_id != 0 {
            let parent = self.nodes.get_mut(&parent_id).unwrap();
            parent.locked = true;
        }

        self.cur_id = self.cur_id + 1;
        self.nodes
            .insert(self.cur_id, StagedExecutionStoreNode::new(parent_id));
        self.cur_id
    }

    pub fn get_output_store<'t>(&'t mut self, id: u64) -> ExecutionStore<'t, 's, S> {
        if self.nodes.get(&id).unwrap().locked {
            panic!("Should not write to locked node");
        }

        ExecutionStore { dag: self, id }
    }

    fn remove_node(&mut self, id: u64, remove_children: bool) -> StagedExecutionStoreNode {
        let result = self.nodes.remove(&id).unwrap();

        if remove_children {
            let mut to_delete = Vec::new();
            for (to_delete_id, node) in &self.nodes {
                if node.parent_id == id {
                    to_delete.push(*to_delete_id);
                }
            }
            for to_delete_id in to_delete {
                self.remove_node(to_delete_id, true);
            }
        } else {
            for node in self.nodes.values_mut().filter(|node| id == node.parent_id) {
                node.parent_id = 0;
            }
        }

        result
    }

    pub fn merge_to_parent(&mut self, id: u64) {
        self.merge_to_parent_recurse(id, false)
    }

    fn merge_to_parent_recurse(&mut self, id: u64, remove_children: bool) {
        if id == 0 {
            return;
        }

        let node = self.remove_node(id, remove_children);
        self.merge_to_parent_recurse(node.parent_id, true);

        for (address, output_id) in node.spaces {
            self.parent.put_space(&address, output_id);
        }

        for (address, output) in node.outputs {
            self.parent.put_substate(&address, output);
        }
    }
}

pub struct ExecutionStore<'t, 's, S: ReadableSubstateStore + WriteableSubstateStore> {
    dag: &'t mut StagedExecutionStores<'s, S>,
    id: u64,
}

impl<'t, 's, S: ReadableSubstateStore + WriteableSubstateStore> ExecutionStore<'t, 's, S> {
    fn get_substate_recurse(&self, address: &[u8], id: u64) -> Option<Output> {
        if id == 0 {
            return self.dag.parent.get_substate(address);
        }

        let node = self.dag.nodes.get(&id).unwrap();
        if let Some(output) = node.outputs.get(address) {
            // TODO: Remove encoding/decoding
            let encoded_output = scrypto_encode(output);
            return Some(scrypto_decode(&encoded_output).unwrap());
        }

        self.get_substate_recurse(address, node.parent_id)
    }

    fn get_space_recurse(&self, address: &[u8], id: u64) -> OutputId {
        if id == 0 {
            return self.dag.parent.get_space(address);
        }

        let node = self.dag.nodes.get(&id).unwrap();
        if let Some(output_id) = node.spaces.get(address) {
            return output_id.clone();
        }

        self.get_space_recurse(address, node.parent_id)
    }
}

impl<'t, 's, S: ReadableSubstateStore + WriteableSubstateStore> ReadableSubstateStore
    for ExecutionStore<'t, 's, S>
{
    fn get_substate(&self, address: &[u8]) -> Option<Output> {
        self.get_substate_recurse(address, self.id)
    }

    fn get_space(&self, address: &[u8]) -> OutputId {
        self.get_space_recurse(address, self.id)
    }
}

impl<'t, 's, S: ReadableSubstateStore + WriteableSubstateStore> WriteableSubstateStore
    for ExecutionStore<'t, 's, S>
{
    fn put_space(&mut self, address: &[u8], output_id: OutputId) {
        if self.id == 0 {
            self.dag.parent.put_space(address, output_id);
        } else {
            let node = self.dag.nodes.get_mut(&self.id).unwrap();
            node.spaces.insert(address.to_vec(), output_id);
        }
    }

    fn put_substate(&mut self, address: &[u8], output: Output) {
        if self.id == 0 {
            self.dag.parent.put_substate(address, output);
        } else {
            let node = self.dag.nodes.get_mut(&self.id).unwrap();
            node.outputs.insert(address.to_vec(), output);
        }
    }
}
