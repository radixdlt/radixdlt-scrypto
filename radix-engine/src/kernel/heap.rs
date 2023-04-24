use crate::blueprints::resource::*;
use crate::types::*;
use radix_engine_interface::blueprints::resource::{
    LiquidFungibleResource, LiquidNonFungibleResource, LockedFungibleResource,
    LockedNonFungibleResource, ResourceType,
};
use radix_engine_stores::interface::NodeSubstates;
use sbor::rust::collections::btree_map::Entry;

pub struct Heap {
    nodes: NonIterMap<NodeId, NodeSubstates>,
}

pub enum MoveNodeToStoreError {
    NodeNotFound(NodeId),
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

    pub fn get_substate_virtualize<F: FnOnce() -> Option<IndexedScryptoValue>>(
        &mut self,
        node_id: &NodeId,
        module_id: ModuleId,
        substate_key: &SubstateKey,
        virtualize: F,
    ) -> Option<&IndexedScryptoValue> {
        let entry = self
            .nodes
            .entry(*node_id)
            .or_insert(BTreeMap::new())
            .entry(module_id)
            .or_insert(BTreeMap::new())
            .entry(substate_key.clone());
        if let Entry::Vacant(e) = entry {
            let value = virtualize();
            if let Some(value) = value {
                e.insert(value);
            }
        }

        self.nodes
            .get(node_id)
            .and_then(|node_substates| node_substates.get(&module_id))
            .and_then(|module_substates| module_substates.get(substate_key))
    }

    /// Reads a substate
    pub fn get_substate(
        &mut self,
        node_id: &NodeId,
        module_id: ModuleId,
        substate_key: &SubstateKey,
    ) -> Option<&IndexedScryptoValue> {
        self.nodes
            .get(node_id)
            .and_then(|node_substates| node_substates.get(&module_id))
            .and_then(|module_substates| module_substates.get(substate_key))
    }

    /// Inserts or overwrites a substate
    pub fn put_substate(
        &mut self,
        node_id: NodeId,
        module_id: ModuleId,
        substate_key: SubstateKey,
        substate_value: IndexedScryptoValue,
    ) {
        self.nodes
            .entry(node_id)
            .or_insert_with(|| NodeSubstates::default())
            .entry(module_id)
            .or_default()
            .insert(substate_key, substate_value);
    }

    /// Inserts a new node to heap.
    pub fn create_node(&mut self, node_id: NodeId, node: NodeSubstates) {
        self.nodes.insert(node_id, node);
    }

    /// Removes node.
    ///
    /// # Panics
    /// - If the node is not found.
    pub fn remove_node(&mut self, node_id: &NodeId) -> NodeSubstates {
        self.nodes
            .remove(node_id)
            .unwrap_or_else(|| panic!("Heap does not contain {:?}", node_id))
    }
}

#[derive(Debug)]
pub struct DroppedFungibleBucket {
    pub info: BucketInfoSubstate,
    pub liquid: LiquidFungibleResource,
    pub locked: LockedFungibleResource,
}

#[derive(Debug)]
pub struct DroppedNonFungibleBucket {
    pub info: BucketInfoSubstate,
    pub liquid: LiquidNonFungibleResource,
    pub locked: LockedNonFungibleResource,
}

impl Into<DroppedFungibleBucket> for NodeSubstates {
    fn into(mut self) -> DroppedFungibleBucket {
        let mut module = self.remove(&SysModuleId::Object.into()).unwrap();

        DroppedFungibleBucket {
            info: module
                .remove(&BucketOffset::Info.into())
                .map(|x| x.as_typed().unwrap())
                .unwrap(),
            liquid: module
                .remove(&BucketOffset::Liquid.into())
                .map(|x| x.as_typed().unwrap())
                .unwrap(),
            locked: module
                .remove(&BucketOffset::Locked.into())
                .map(|x| x.as_typed().unwrap())
                .unwrap(),
        }
    }
}

impl Into<DroppedNonFungibleBucket> for NodeSubstates {
    fn into(mut self) -> DroppedNonFungibleBucket {
        let mut module = self.remove(&SysModuleId::Object.into()).unwrap();

        DroppedNonFungibleBucket {
            info: module
                .remove(&BucketOffset::Info.into())
                .map(|x| x.as_typed().unwrap())
                .unwrap(),
            liquid: module
                .remove(&BucketOffset::Liquid.into())
                .map(|x| x.as_typed().unwrap())
                .unwrap(),
            locked: module
                .remove(&BucketOffset::Locked.into())
                .map(|x| x.as_typed().unwrap())
                .unwrap(),
        }
    }
}

pub struct DroppedProof {
    pub info: ProofInfoSubstate,
    pub proof: DroppedProofResource,
}

pub enum DroppedProofResource {
    Fungible(FungibleProof),
    NonFungible(NonFungibleProof),
}

impl Into<DroppedProof> for NodeSubstates {
    fn into(mut self) -> DroppedProof {
        let mut module = self.remove(&SysModuleId::Object.into()).unwrap();

        let info: ProofInfoSubstate = module
            .remove(&ProofOffset::Info.into())
            .map(|x| x.as_typed().unwrap())
            .unwrap();

        let proof = match info.resource_type {
            ResourceType::Fungible { .. } => DroppedProofResource::Fungible(
                module
                    .remove(&ProofOffset::Fungible.into())
                    .map(|x| x.as_typed().unwrap())
                    .unwrap(),
            ),
            ResourceType::NonFungible { .. } => DroppedProofResource::NonFungible(
                module
                    .remove(&ProofOffset::NonFungible.into())
                    .map(|x| x.as_typed().unwrap())
                    .unwrap(),
            ),
        };

        DroppedProof { info, proof }
    }
}
