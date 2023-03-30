use super::track::Track;
use crate::blueprints::resource::*;
use crate::types::*;
use radix_engine_interface::blueprints::resource::{
    LiquidFungibleResource, LiquidNonFungibleResource, ResourceType,
};
use radix_engine_interface::math::Decimal;
use radix_engine_stores::interface::SubstateStore;

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
        module_id: ModuleId,
        substate_key: &SubstateKey,
    ) -> Option<&IndexedScryptoValue> {
        self.nodes
            .get_mut(node_id)
            .and_then(|node| node.substates.get(&module_id))
            .and_then(|module| module.get(substate_key))
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
            .or_insert_with(|| HeapNode::default())
            .substates
            .entry(module_id)
            .or_default()
            .insert(substate_key, substate_value);
    }

    pub fn insert_node(&mut self, node_id: NodeId, node: HeapNode) {
        self.nodes.insert(node_id, node);
    }

    /// Moves node to track.
    ///
    /// # Panics
    /// - If the node is not found.
    pub fn move_node_to_store(&mut self, track: &mut Track, node_id: &NodeId) {
        let node = self
            .nodes
            .remove(&node_id)
            .unwrap_or_else(|| panic!("Heap does not contain {:?}", node_id));
        for (module_id, module) in node.substates {
            for (substate_key, substate_value) in module {
                for node in substate_value.owned_node_ids() {
                    self.move_node_to_store(track, node);
                }
                track.insert_substate(
                    node_id.clone(),
                    module_id.into(),
                    substate_key,
                    substate_value,
                );
            }
        }
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
    pub substates: BTreeMap<ModuleId, BTreeMap<SubstateKey, IndexedScryptoValue>>,
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
        let module = self
            .substates
            .remove(&TypedModuleId::ObjectState.into())
            .unwrap();

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

impl Into<ProofInfoSubstate> for HeapNode {
    fn into(mut self) -> ProofInfoSubstate {
        let module = self
            .substates
            .remove(&TypedModuleId::ObjectState.into())
            .unwrap();

        module
            .remove(&ProofOffset::Info.into())
            .map(|x| x.as_typed().unwrap())
            .unwrap()
    }
}
