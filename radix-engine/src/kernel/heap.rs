use radix_engine_common::data::scrypto::{scrypto_decode, scrypto_encode, ScryptoValue};
use radix_engine_common::data::scrypto::model::ComponentAddress;
use super::track::Track;
use crate::blueprints::resource::*;
use crate::errors::{CallFrameError, OffsetDoesNotExist};
use crate::system::node_substates::{RuntimeSubstate, SubstateRef, SubstateRefMut};
use crate::types::NonIterMap;
use radix_engine_interface::api::types::{
    BucketOffset, NodeModuleId, ProofOffset, RENodeId, SubstateId, SubstateOffset,
};
use radix_engine_interface::blueprints::resource::{
    LiquidFungibleResource, LiquidNonFungibleResource, ResourceType,
};
use radix_engine_interface::math::Decimal;
use sbor::rust::boxed::Box;
use sbor::rust::collections::BTreeMap;
use sbor::rust::vec::Vec;
use crate::blueprints::epoch_manager::Validator;

pub struct Heap {
    nodes: NonIterMap<RENodeId, HeapRENode>,
}

impl Heap {
    pub fn new() -> Self {
        Self {
            nodes: NonIterMap::new(),
        }
    }

    pub fn contains_node(&self, node_id: &RENodeId) -> bool {
        self.nodes.contains_key(node_id)
    }

    pub fn get_first_in_iterable(
        &mut self,
        node_id: &RENodeId,
        module_id: &NodeModuleId,
        count: u32,
    ) -> Result<Vec<(SubstateId, RuntimeSubstate)>, CallFrameError> {
        let node = self
            .nodes
            .get_mut(node_id)
            .ok_or_else(|| CallFrameError::RENodeNotOwned(node_id.clone()))?;

        let mut items = Vec::new();

        for ((node_module_id, offset), value) in node.substates.iter() {
            if let NodeModuleId::SELF = node_module_id {
                let (address, validator) = if let RuntimeSubstate::IterableEntry(value) = value {
                    let value: (ComponentAddress, Validator) = scrypto_decode(&scrypto_encode(value).unwrap()).unwrap();
                    value
                } else {
                    panic!("oops: {:?}", value);
                };
            }
        }

        for ((node_module_id, offset), value) in node.substates.iter().take(count.try_into().unwrap()) {
            let substate_id = SubstateId(node_id.clone(), module_id.clone(), offset.clone());
            if let RuntimeSubstate::IterableEntry(value) = value {
                items.push((substate_id, RuntimeSubstate::IterableEntry(value.clone())))
            } else {
                panic!("Unexpected");
            }
        }

        Ok(items)
    }

    pub fn insert_into_iterable(
        &mut self,
        node_id: &RENodeId,
        module_id: &NodeModuleId,
        key: Vec<u8>,
        value: ScryptoValue,
    ) -> Result<(), CallFrameError> {
        let node = self
            .nodes
            .get_mut(node_id)
            .ok_or_else(|| CallFrameError::RENodeNotOwned(node_id.clone()))?;

        node.substates.insert(
            (module_id.clone(), SubstateOffset::IterableMap(key)),
            RuntimeSubstate::IterableEntry(value),
        );

        Ok(())
    }

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
                .ok_or_else(|| {
                    CallFrameError::OffsetDoesNotExist(Box::new(OffsetDoesNotExist(
                        node_id.clone(),
                        offset.clone(),
                    )))
                }),
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
                .ok_or_else(|| {
                    CallFrameError::OffsetDoesNotExist(Box::new(OffsetDoesNotExist(
                        node_id.clone(),
                        offset.clone(),
                    )))
                }),
        }
    }

    pub fn create_node(&mut self, node_id: RENodeId, node: HeapRENode) {
        self.nodes.insert(node_id, node);
    }

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
                .map_err(|e| CallFrameError::FailedToMoveSubstateToTrack(Box::new(e)))?;
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
