use super::track::Track;
use crate::blueprints::resource::*;
use crate::types::*;
use radix_engine_interface::blueprints::resource::{
    LiquidFungibleResource, LiquidNonFungibleResource, ResourceType,
};
use radix_engine_interface::math::Decimal;
use radix_engine_stores::interface::SubstateStore;

pub struct Heap {
    nodes: HashMap<NodeId, HeapNode>,
}

pub enum MoveNodeToStoreError {
    NodeNotFound(NodeId),
}

impl Heap {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }

    pub fn contains_node(&self, node_id: &NodeId) -> bool {
        self.nodes.contains_key(node_id)
    }

    pub fn get_substate(
        &self,
        node_id: &NodeId,
        module_id: TypedModuleId,
        substate_key: &SubstateKey,
    ) -> Option<&IndexedScryptoValue> {
        self.nodes
            .get_mut(node_id)
            .and_then(|node| node.substates.get(&module_id))
            .and_then(|module| module.get(substate_key))
    }

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

    pub fn insert_node(&mut self, node_id: NodeId, node: HeapNode) {
        self.nodes.insert(node_id, node);
    }

    /// Moves nodes to track.
    ///
    /// Panics if any of the nodes is not found.
    pub fn move_nodes_to_store(&mut self, track: &mut Track, nodes: &[NodeId]) {
        for node_id in nodes {
            self.move_node_to_store(track, node_id);
        }
    }

    /// Moves node to track.
    ///
    /// Panics if the node is not found.
    pub fn move_node_to_store(&mut self, track: &mut Track, node_id: &NodeId) {
        let node = self
            .nodes
            .remove(&node_id)
            .unwrap_or_else(|| panic!("Heap does not contain {:?}", node_id));
        for (module_id, module) in node.substates {
            for (substate_key, substate_value) in module {
                let owned_nodes = substate_value.owned_node_ids();
                self.move_nodes_to_store(track, owned_nodes.as_ref());
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
    /// Panics if the node is not found.
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
