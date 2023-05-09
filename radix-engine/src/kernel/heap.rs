use crate::blueprints::resource::*;
use crate::track::interface::NodeSubstates;
use crate::types::*;
use radix_engine_interface::blueprints::resource::{
    LiquidFungibleResource, LiquidNonFungibleResource, LockedFungibleResource,
    LockedNonFungibleResource,
};
use sbor::rust::collections::btree_map::Entry;

#[derive(Debug, Default)]
pub struct HeapNode {
    substates: NodeSubstates,
    borrow_count: usize,
}

pub struct Heap {
    nodes: NonIterMap<NodeId, HeapNode>,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum HeapRemoveModuleError {
    NodeNotFound(NodeId),
    ModuleNotFound(ModuleNumber),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum HeapRemoveNodeError {
    NodeNotFound(NodeId),
    NodeBorrowed(NodeId, usize),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum HeapLockSubstateError {
    LockUnmodifiedBaseOnHeapNode,
    SubstateNotFound(NodeId, ModuleNumber, SubstateKey),
}

impl Heap {
    pub fn new() -> Self {
        Self {
            nodes: NonIterMap::new(),
        }
    }

    /// Checks if the given node is in this heap.
    pub fn contains_node(&self, node_id: &NodeId) -> bool {
        self.nodes.contains_key(node_id)
    }

    pub fn list_modules(&self, node_id: &NodeId) -> Option<BTreeSet<ModuleNumber>> {
        self.nodes
            .get(node_id)
            .map(|node| node.substates.keys().cloned().collect())
    }

    pub fn remove_module(
        &mut self,
        node_id: &NodeId,
        module_number: ModuleNumber,
    ) -> Result<BTreeMap<SubstateKey, IndexedScryptoValue>, HeapRemoveModuleError> {
        if let Some(modules) = self.nodes.get_mut(node_id).map(|node| &mut node.substates) {
            let module = modules
                .remove(&module_number)
                .ok_or(HeapRemoveModuleError::ModuleNotFound(module_number))?;
            Ok(module)
        } else {
            Err(HeapRemoveModuleError::NodeNotFound(node_id.clone()))
        }
    }

    pub fn get_substate_virtualize<F: FnOnce() -> IndexedScryptoValue>(
        &mut self,
        node_id: &NodeId,
        module_num: ModuleNumber,
        substate_key: &SubstateKey,
        virtualize: F,
    ) -> &IndexedScryptoValue {
        let entry = self
            .nodes
            .entry(*node_id)
            .or_insert(HeapNode::default())
            .substates
            .entry(module_num)
            .or_insert(BTreeMap::new())
            .entry(substate_key.clone());
        if let Entry::Vacant(e) = entry {
            let value = virtualize();
            e.insert(value);
        }

        self.nodes
            .get(node_id)
            .and_then(|node| node.substates.get(&module_num))
            .and_then(|module_substates| module_substates.get(substate_key))
            .unwrap()
    }

    /// Reads a substate
    pub fn get_substate(
        &self,
        node_id: &NodeId,
        module_num: ModuleNumber,
        substate_key: &SubstateKey,
    ) -> Option<&IndexedScryptoValue> {
        self.nodes
            .get(node_id)
            .and_then(|node| node.substates.get(&module_num))
            .and_then(|module_substates| module_substates.get(substate_key))
    }

    /// Inserts or overwrites a substate
    pub fn set_substate(
        &mut self,
        node_id: NodeId,
        module_num: ModuleNumber,
        substate_key: SubstateKey,
        substate_value: IndexedScryptoValue,
    ) {
        self.nodes
            .entry(node_id)
            .or_insert_with(|| HeapNode::default())
            .substates
            .entry(module_num)
            .or_default()
            .insert(substate_key, substate_value);
    }

    pub fn delete_substate(
        &mut self,
        node_id: &NodeId,
        module_num: ModuleNumber,
        substate_key: &SubstateKey,
    ) -> Option<IndexedScryptoValue> {
        self.nodes
            .get_mut(node_id)
            .and_then(|n| n.substates.get_mut(&module_num))
            .and_then(|s| s.remove(substate_key))
    }

    pub fn scan_substates(
        &mut self,
        node_id: &NodeId,
        module_num: ModuleNumber,
        count: u32,
    ) -> Vec<IndexedScryptoValue> {
        let node_substates = self
            .nodes
            .get_mut(node_id)
            .and_then(|n| n.substates.get_mut(&module_num));
        if let Some(substates) = node_substates {
            let substates: Vec<IndexedScryptoValue> = substates
                .iter()
                .map(|(_key, v)| v.clone())
                .take(count.try_into().unwrap())
                .collect();

            substates
        } else {
            vec![] // TODO: should this just be an error instead?
        }
    }

    pub fn take_substates(
        &mut self,
        node_id: &NodeId,
        module_num: ModuleNumber,
        count: u32,
    ) -> Vec<IndexedScryptoValue> {
        let node_substates = self
            .nodes
            .get_mut(node_id)
            .and_then(|n| n.substates.get_mut(&module_num));
        if let Some(substates) = node_substates {
            let keys: Vec<SubstateKey> = substates
                .iter()
                .map(|(key, _)| key.clone())
                .take(count.try_into().unwrap())
                .collect();

            let mut items = Vec::new();

            for key in keys {
                let value = substates.remove(&key).unwrap();
                items.push(value);
            }

            items
        } else {
            vec![] // TODO: should this just be an error instead?
        }
    }

    /// Inserts a new node to heap.
    pub fn create_node(&mut self, node_id: NodeId, substates: NodeSubstates) {
        self.nodes.insert(
            node_id,
            HeapNode {
                substates,
                borrow_count: 0,
            },
        );
    }

    /// Removes node.
    pub fn remove_node(&mut self, node_id: &NodeId) -> Result<NodeSubstates, HeapRemoveNodeError> {
        match self
            .nodes
            .get(node_id)
            .map(|node| node.borrow_count.clone())
        {
            Some(n) => {
                if n != 0 {
                    return Err(HeapRemoveNodeError::NodeBorrowed(node_id.clone(), n));
                } else {
                }
            }
            None => return Err(HeapRemoveNodeError::NodeNotFound(node_id.clone())),
        }

        Ok(self.nodes.remove(node_id).unwrap().substates)
    }
}

#[derive(Debug)]
pub struct DroppedFungibleBucket {
    pub liquid: LiquidFungibleResource,
    pub locked: LockedFungibleResource,
}

#[derive(Debug)]
pub struct DroppedNonFungibleBucket {
    pub liquid: LiquidNonFungibleResource,
    pub locked: LockedNonFungibleResource,
}

impl Into<DroppedFungibleBucket> for Vec<Vec<u8>> {
    fn into(self) -> DroppedFungibleBucket {
        let liquid: LiquidFungibleResource =
            scrypto_decode(&self[FungibleBucketOffset::Liquid as usize]).unwrap();
        let locked: LockedFungibleResource =
            scrypto_decode(&self[FungibleBucketOffset::Locked as usize]).unwrap();

        DroppedFungibleBucket { liquid, locked }
    }
}

impl Into<DroppedNonFungibleBucket> for Vec<Vec<u8>> {
    fn into(self) -> DroppedNonFungibleBucket {
        let liquid: LiquidNonFungibleResource =
            scrypto_decode(&self[NonFungibleBucketOffset::Liquid as usize]).unwrap();
        let locked: LockedNonFungibleResource =
            scrypto_decode(&self[NonFungibleBucketOffset::Locked as usize]).unwrap();

        DroppedNonFungibleBucket { liquid, locked }
    }
}

pub struct DroppedFungibleProof {
    pub moveable: ProofMoveableSubstate,
    pub fungible_proof: FungibleProof,
}

pub struct DroppedNonFungibleProof {
    pub moveable: ProofMoveableSubstate,
    pub non_fungible_proof: NonFungibleProof,
}

impl Into<DroppedFungibleProof> for Vec<Vec<u8>> {
    fn into(self) -> DroppedFungibleProof {
        let moveable: ProofMoveableSubstate =
            scrypto_decode(&self[FungibleProofOffset::Moveable as usize]).unwrap();
        let fungible_proof: FungibleProof =
            scrypto_decode(&self[FungibleProofOffset::ProofRefs as usize]).unwrap();

        DroppedFungibleProof {
            moveable,
            fungible_proof,
        }
    }
}

impl Into<DroppedNonFungibleProof> for Vec<Vec<u8>> {
    fn into(self) -> DroppedNonFungibleProof {
        let moveable: ProofMoveableSubstate =
            scrypto_decode(&self[FungibleProofOffset::Moveable as usize]).unwrap();
        let non_fungible_proof: NonFungibleProof =
            scrypto_decode(&self[FungibleProofOffset::ProofRefs as usize]).unwrap();

        DroppedNonFungibleProof {
            moveable,
            non_fungible_proof,
        }
    }
}
