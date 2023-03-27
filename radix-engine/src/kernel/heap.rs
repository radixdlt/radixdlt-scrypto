use super::track::Track;
use crate::blueprints::resource::*;
use crate::errors::CallFrameError;
use crate::system::node_substates::{RuntimeSubstate, SubstateRef, SubstateRefMut};
use crate::types::HashMap;
use radix_engine_interface::api::types::{
    BucketOffset, NodeModuleId, ProofOffset, RENodeId, SubstateId, SubstateOffset,
};
use radix_engine_interface::blueprints::resource::{
    LiquidFungibleResource, LiquidNonFungibleResource, ResourceType,
};
use radix_engine_interface::math::Decimal;
use sbor::rust::collections::BTreeMap;
use sbor::rust::vec::Vec;
use resources_tracker_macro::trace_resources;

pub struct Heap {
    nodes: HashMap<RENodeId, HeapRENode>,
}

impl Heap {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }

    pub fn contains_node(&self, node_id: &RENodeId) -> bool {
        self.nodes.contains_key(node_id)
    }

    #[trace_resources(info="Heap")]
    pub fn get_substate(
        &mut self,
        node_id: &RENodeId,
        module_id: NodeModuleId,
        offset: &SubstateOffset,
    ) -> Result<SubstateRef, CallFrameError> {
        let node = self
            .nodes
            .get_mut(node_id)
            .ok_or_else(|| CallFrameError::RENodeNotOwned(node_id.clone()))?;

        // TODO: Will clean this up when virtual substates is cleaned up
        match (&node_id, module_id, offset) {
            (_, _, SubstateOffset::KeyValueStore(..)) => {
                let entry = node
                    .substates
                    .entry((module_id, offset.clone()))
                    .or_insert(RuntimeSubstate::KeyValueStoreEntry(Option::None));
                Ok(entry.to_ref())
            }
            _ => node
                .substates
                .get(&(module_id, offset.clone()))
                .map(|s| s.to_ref())
                .ok_or_else(|| CallFrameError::OffsetDoesNotExist(node_id.clone(), offset.clone())),
        }
    }

    pub fn get_substate_mut(
        &mut self,
        node_id: &RENodeId,
        module_id: NodeModuleId,
        offset: &SubstateOffset,
    ) -> Result<SubstateRefMut, CallFrameError> {
        let node = self
            .nodes
            .get_mut(node_id)
            .ok_or_else(|| CallFrameError::RENodeNotOwned(node_id.clone()))?;

        // TODO: Will clean this up when virtual substates is cleaned up
        match (&node_id, offset) {
            (_, SubstateOffset::KeyValueStore(..)) => {
                let entry = node
                    .substates
                    .entry((module_id, offset.clone()))
                    .or_insert(RuntimeSubstate::KeyValueStoreEntry(Option::None));
                Ok(entry.to_ref_mut())
            }
            _ => node
                .substates
                .get_mut(&(module_id, offset.clone()))
                .map(|s| s.to_ref_mut())
                .ok_or_else(|| CallFrameError::OffsetDoesNotExist(node_id.clone(), offset.clone())),
        }
    }

    #[trace_resources(info="Heap")]
    pub fn create_node(&mut self, node_id: RENodeId, node: HeapRENode) {
        self.nodes.insert(node_id, node);
    }

    #[trace_resources]
    pub fn move_nodes_to_store(
        &mut self,
        track: &mut Track,
        nodes: Vec<RENodeId>,
    ) -> Result<(), CallFrameError> {
        for node_id in nodes {
            self.move_node_to_store(track, node_id)?;
        }

        Ok(())
    }

    #[trace_resources]
    pub fn move_node_to_store(
        &mut self,
        track: &mut Track,
        node_id: RENodeId,
    ) -> Result<(), CallFrameError> {
        let node = self
            .nodes
            .remove(&node_id)
            .ok_or_else(|| CallFrameError::RENodeNotOwned(node_id))?;
        for ((module_id, offset), substate) in node.substates {
            let (_, owned_nodes) = substate.to_ref().references_and_owned_nodes();
            self.move_nodes_to_store(track, owned_nodes)?;
            track
                .insert_substate(SubstateId(node_id, module_id, offset), substate)
                .map_err(|e| CallFrameError::FailedToMoveSubstateToTrack(e))?;
        }

        Ok(())
    }

    pub fn remove_node(&mut self, node_id: &RENodeId) -> Result<HeapRENode, CallFrameError> {
        self.nodes
            .remove(node_id)
            .ok_or_else(|| CallFrameError::RENodeNotOwned(node_id.clone()))
    }
}

#[derive(Debug)]
pub struct HeapRENode {
    pub substates: BTreeMap<(NodeModuleId, SubstateOffset), RuntimeSubstate>,
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

impl Into<DroppedBucket> for HeapRENode {
    fn into(mut self) -> DroppedBucket {
        let info: BucketInfoSubstate = self
            .substates
            .remove(&(
                NodeModuleId::SELF,
                SubstateOffset::Bucket(BucketOffset::Info),
            ))
            .unwrap()
            .into();

        let resource = match info.resource_type {
            ResourceType::Fungible { .. } => DroppedBucketResource::Fungible(
                self.substates
                    .remove(&(
                        NodeModuleId::SELF,
                        SubstateOffset::Bucket(BucketOffset::LiquidFungible),
                    ))
                    .map(|s| Into::<LiquidFungibleResource>::into(s))
                    .unwrap(),
            ),
            ResourceType::NonFungible { .. } => DroppedBucketResource::NonFungible(
                self.substates
                    .remove(&(
                        NodeModuleId::SELF,
                        SubstateOffset::Bucket(BucketOffset::LiquidNonFungible),
                    ))
                    .map(|s| Into::<LiquidNonFungibleResource>::into(s))
                    .unwrap(),
            ),
        };

        DroppedBucket { info, resource }
    }
}

impl Into<ProofInfoSubstate> for HeapRENode {
    fn into(mut self) -> ProofInfoSubstate {
        self.substates
            .remove(&(NodeModuleId::SELF, SubstateOffset::Proof(ProofOffset::Info)))
            .unwrap()
            .into()
    }
}
