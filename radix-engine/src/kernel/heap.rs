use super::track::Track;
use crate::blueprints::resource::*;
use crate::errors::CallFrameError;
use crate::system::node_substates::{RuntimeSubstate, SubstateRef, SubstateRefMut};
use crate::types::HashMap;
use radix_engine_interface::blueprints::resource::{
    LiquidFungibleResource, LiquidNonFungibleResource, ResourceType,
};
use radix_engine_interface::math::Decimal;
use radix_engine_interface::types::{
    BucketOffset, NodeId, ProofOffset, SubstateId, SubstateKey, TypedModuleId,
};
use sbor::rust::collections::BTreeMap;
use sbor::rust::vec::Vec;

pub struct Heap {
    nodes: HashMap<NodeId, HeapRENode>,
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
        &mut self,
        node_id: &NodeId,
        module_id: TypedModuleId,
        offset: &SubstateKey,
    ) -> Result<SubstateRef, CallFrameError> {
        let node = self
            .nodes
            .get_mut(node_id)
            .ok_or_else(|| CallFrameError::RENodeNotOwned(node_id.clone()))?;

        // TODO: Will clean this up when virtual substates is cleaned up
        match (&node_id, module_id, offset) {
            (_, _, SubstateKey::KeyValueStore(..)) => {
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
        node_id: &NodeId,
        module_id: TypedModuleId,
        offset: &SubstateKey,
    ) -> Result<SubstateRefMut, CallFrameError> {
        let node = self
            .nodes
            .get_mut(node_id)
            .ok_or_else(|| CallFrameError::RENodeNotOwned(node_id.clone()))?;

        // TODO: Will clean this up when virtual substates is cleaned up
        match (&node_id, offset) {
            (_, SubstateKey::KeyValueStore(..)) => {
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

    pub fn create_node(&mut self, node_id: NodeId, node: HeapRENode) {
        self.nodes.insert(node_id, node);
    }

    pub fn move_nodes_to_store(
        &mut self,
        track: &mut Track,
        nodes: Vec<NodeId>,
    ) -> Result<(), CallFrameError> {
        for node_id in nodes {
            self.move_node_to_store(track, node_id)?;
        }

        Ok(())
    }

    pub fn move_node_to_store(
        &mut self,
        track: &mut Track,
        node_id: NodeId,
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

    pub fn remove_node(&mut self, node_id: &NodeId) -> Result<HeapRENode, CallFrameError> {
        self.nodes
            .remove(node_id)
            .ok_or_else(|| CallFrameError::RENodeNotOwned(node_id.clone()))
    }
}

#[derive(Debug)]
pub struct HeapRENode {
    pub substates: BTreeMap<(TypedModuleId, SubstateKey), RuntimeSubstate>,
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
            .remove(&(TypedModuleId::ObjectState, BucketOffset::Bucket.into()))
            .unwrap()
            .into();

        let resource = match info.resource_type {
            ResourceType::Fungible { .. } => DroppedBucketResource::Fungible(
                self.substates
                    .remove(&(TypedModuleId::ObjectState, BucketOffset::Bucket.into()))
                    .map(|s| Into::<LiquidFungibleResource>::into(s))
                    .unwrap(),
            ),
            ResourceType::NonFungible { .. } => DroppedBucketResource::NonFungible(
                self.substates
                    .remove(&(TypedModuleId::ObjectState, BucketOffset::Bucket.into()))
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
            .remove(&(TypedModuleId::ObjectState, ProofOffset::Proof.into()))
            .unwrap()
            .into()
    }
}
