use crate::blueprints::resource::*;
use crate::types::*;
use radix_engine_interface::blueprints::resource::{
    LiquidFungibleResource, LiquidNonFungibleResource, LockedFungibleResource,
    LockedNonFungibleResource, ResourceType,
};

pub struct Heap {
    nodes: NonIterMap<NodeId, HeapNode>,
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

    /// Reads a substate
    pub fn get_substate(
        &self,
        node_id: &NodeId,
        module_id: SysModuleId,
        substate_key: &SubstateKey,
    ) -> Option<&IndexedScryptoValue> {
        self.nodes
            .get(node_id)
            .and_then(|node| node.substates.get(&module_id))
            .and_then(|module| module.get(substate_key))
    }

    /// Inserts or overwrites a substate
    pub fn put_substate(
        &mut self,
        node_id: NodeId,
        module_id: SysModuleId,
        substate_key: SubstateKey,
        substate_value: IndexedScryptoValue,
    ) {
        self.nodes
            .entry(node_id)
            .or_insert_with(|| HeapNode::default())
            .substates
            .entry(module_id)
            .or_default()
            .insert(substate_key, substate_value);
    }

    /// Inserts a new node to heap.
    pub fn insert_node(&mut self, node_id: NodeId, node: HeapNode) {
        self.nodes.insert(node_id, node);
    }

    /// Removes node.
    ///
    /// # Panics
    /// - If the node is not found.
    pub fn remove_node(&mut self, node_id: &NodeId) -> HeapNode {
        self.nodes
            .remove(node_id)
            .unwrap_or_else(|| panic!("Heap does not contain {:?}", node_id))
    }
}

// TODO: Remove
#[derive(Debug, Default)]
pub struct HeapNode {
    pub substates: BTreeMap<SysModuleId, BTreeMap<SubstateKey, IndexedScryptoValue>>,
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

impl Into<DroppedFungibleBucket> for HeapNode {
    fn into(mut self) -> DroppedFungibleBucket {
        let mut module = self.substates.remove(&SysModuleId::ObjectTuple).unwrap();

        DroppedFungibleBucket {
            info: module
                .remove(&BucketOffset::Info.into())
                .map(|x| x.as_typed().unwrap())
                .unwrap(),
            liquid: module
                .remove(&BucketOffset::LiquidFungible.into())
                .map(|x| x.as_typed().unwrap())
                .unwrap(),
            locked: module
                .remove(&BucketOffset::LockedFungible.into())
                .map(|x| x.as_typed().unwrap())
                .unwrap(),
        }
    }
}

impl Into<DroppedNonFungibleBucket> for HeapNode {
    fn into(mut self) -> DroppedNonFungibleBucket {
        let mut module = self.substates.remove(&SysModuleId::ObjectTuple).unwrap();

        DroppedNonFungibleBucket {
            info: module
                .remove(&BucketOffset::Info.into())
                .map(|x| x.as_typed().unwrap())
                .unwrap(),
            liquid: module
                .remove(&BucketOffset::LiquidNonFungible.into())
                .map(|x| x.as_typed().unwrap())
                .unwrap(),
            locked: module
                .remove(&BucketOffset::LockedNonFungible.into())
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

impl Into<DroppedProof> for HeapNode {
    fn into(mut self) -> DroppedProof {
        let mut module = self.substates.remove(&SysModuleId::ObjectTuple).unwrap();

        let info: ProofInfoSubstate = module
            .remove(&ProofOffset::Info.into())
            .map(|x| x.as_typed().unwrap())
            .unwrap();

        let resource = match info.resource_type {
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

        DroppedProof {
            info,
            proof: resource,
        }
    }
}
