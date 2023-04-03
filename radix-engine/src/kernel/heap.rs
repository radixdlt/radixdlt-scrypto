use crate::blueprints::resource::*;
use crate::types::*;
use radix_engine_interface::blueprints::resource::{
    LiquidFungibleResource, LiquidNonFungibleResource, ResourceType,
};
use radix_engine_interface::math::Decimal;

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
        module_id: TypedModuleId,
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
        module_id: TypedModuleId,
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

#[derive(Debug, Default)]
pub struct HeapNode {
    pub substates: BTreeMap<TypedModuleId, BTreeMap<SubstateKey, IndexedScryptoValue>>,
}

pub struct DroppedBucket {
    pub info: BucketInfoSubstate,
    pub resource: DroppedBucketResource,
}

pub enum DroppedBucketResource {
    Fungible(LiquidFungibleResource),
    NonFungible(LiquidNonFungibleResource),
}

impl DroppedBucket {
    pub fn amount(&self) -> Decimal {
        match &self.resource {
            DroppedBucketResource::Fungible(f) => f.amount(),
            DroppedBucketResource::NonFungible(f) => f.amount(),
        }
    }
}

impl Into<DroppedBucket> for HeapNode {
    fn into(mut self) -> DroppedBucket {
        let mut module = self.substates.remove(&TypedModuleId::ObjectState).unwrap();

        let info: BucketInfoSubstate = module
            .remove(&BucketOffset::Info.into())
            .map(|x| x.as_typed().unwrap())
            .unwrap();

        let resource = match info.resource_type {
            ResourceType::Fungible { .. } => DroppedBucketResource::Fungible(
                module
                    .remove(&BucketOffset::LiquidFungible.into())
                    .map(|x| x.as_typed().unwrap())
                    .unwrap(),
            ),
            ResourceType::NonFungible { .. } => DroppedBucketResource::NonFungible(
                module
                    .remove(&BucketOffset::LiquidNonFungible.into())
                    .map(|x| x.as_typed().unwrap())
                    .unwrap(),
            ),
        };

        DroppedBucket { info, resource }
    }
}

pub struct DroppedProof {
    pub info: ProofInfoSubstate,
    pub resource: DroppedProofResource,
}

pub enum DroppedProofResource {
    Fungible(FungibleProof),
    NonFungible(NonFungibleProof),
}

impl Into<DroppedProof> for HeapNode {
    fn into(mut self) -> DroppedProof {
        let mut module = self.substates.remove(&TypedModuleId::ObjectState).unwrap();

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

        DroppedProof { info, resource }
    }
}
