use std::collections::BTreeSet;
use super::track::Track;
use crate::blueprints::resource::*;
use crate::errors::{CallFrameError, OffsetDoesNotExist};
use crate::system::node_substates::{RuntimeSubstate, SubstateRef, SubstateRefMut};
use crate::types::NonIterMap;
use radix_engine_common::data::scrypto::ScryptoValue;
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

    pub fn remove_first_in_iterable(
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

        let substates = node
            .substates
            .entry(module_id.clone())
            .or_insert(BTreeMap::new());

        let mut offsets = BTreeSet::new();
        for (offset, value) in substates.iter().take(count.try_into().unwrap()) {
            offsets.insert(offset.clone());
        }

        for offset in offsets {
            let substate = substates.remove(&offset).unwrap();
            let substate_id = SubstateId(node_id.clone(), module_id.clone(), offset);
            if let RuntimeSubstate::IterableEntry(value) = substate {
                items.push((substate_id, RuntimeSubstate::IterableEntry(value.clone())))
            } else {
                panic!("Unexpected");
            }
        }

        Ok(items)
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

        let substates = node
            .substates
            .entry(module_id.clone())
            .or_insert(BTreeMap::new());

        for (offset, value) in substates.iter().take(count.try_into().unwrap()) {
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

        let substates = node
            .substates
            .entry(module_id.clone())
            .or_insert(BTreeMap::new());

        substates.insert(
            SubstateOffset::IterableMap(key),
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
                    .entry(module_id)
                    .or_insert(BTreeMap::new())
                    .entry(offset.clone())
                    .or_insert(RuntimeSubstate::KeyValueStoreEntry(Option::None));
                Ok(entry.to_ref())
            }
            _ => {
                let substates = node.substates.get(&module_id).ok_or_else(|| {
                    CallFrameError::OffsetDoesNotExist(Box::new(OffsetDoesNotExist(
                        node_id.clone(),
                        offset.clone(),
                    )))
                })?;

                substates.get(offset).map(|s| s.to_ref()).ok_or_else(|| {
                    CallFrameError::OffsetDoesNotExist(Box::new(OffsetDoesNotExist(
                        node_id.clone(),
                        offset.clone(),
                    )))
                })
            }
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
                let substates = node.substates.entry(module_id).or_insert(BTreeMap::new());
                let entry = substates
                    .entry(offset.clone())
                    .or_insert(RuntimeSubstate::KeyValueStoreEntry(Option::None));
                Ok(entry.to_ref_mut())
            }
            _ => {
                let substates = node.substates.get_mut(&module_id).ok_or_else(|| {
                    CallFrameError::OffsetDoesNotExist(Box::new(OffsetDoesNotExist(
                        node_id.clone(),
                        offset.clone(),
                    )))
                })?;

                substates
                    .get_mut(offset)
                    .map(|s| s.to_ref_mut())
                    .ok_or_else(|| {
                        CallFrameError::OffsetDoesNotExist(Box::new(OffsetDoesNotExist(
                            node_id.clone(),
                            offset.clone(),
                        )))
                    })
            }
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
        for (module_id, substates) in node.substates {
            match module_id {
                NodeModuleId::Iterable => {
                    track.insert_iterable(&node_id, &module_id);
                    for (offset, substate) in substates {
                        match (offset, substate) {
                            (
                                SubstateOffset::IterableMap(key),
                                RuntimeSubstate::IterableEntry(value),
                            ) => {
                                track.insert_into_iterable(&node_id, &module_id, key, value);
                            }
                            _ => panic!("Unexpected"),
                        }
                    }
                }
                _ => {
                    for (offset, substate) in substates {
                        let (_, owned_nodes) = substate.to_ref().references_and_owned_nodes();
                        self.move_nodes_to_store(track, owned_nodes)?;
                        track
                            .insert_substate(SubstateId(node_id, module_id, offset), substate)
                            .map_err(|e| {
                                CallFrameError::FailedToMoveSubstateToTrack(Box::new(e))
                            })?;
                    }
                }
            }
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
    pub substates: BTreeMap<NodeModuleId, BTreeMap<SubstateOffset, RuntimeSubstate>>,
}

impl HeapRENode {
    pub fn new(
        substates: BTreeMap<NodeModuleId, BTreeMap<SubstateOffset, RuntimeSubstate>>,
    ) -> Self {
        Self { substates }
    }
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
        let mut self_substates = self.substates.remove(&NodeModuleId::SELF).unwrap();
        let info: BucketInfoSubstate = self_substates
            .remove(&SubstateOffset::Bucket(BucketOffset::Info))
            .unwrap()
            .into();

        let resource = match info.resource_type {
            ResourceType::Fungible { .. } => DroppedBucketResource::Fungible(
                self_substates
                    .remove(&SubstateOffset::Bucket(BucketOffset::LiquidFungible))
                    .map(|s| Into::<LiquidFungibleResource>::into(s))
                    .unwrap(),
            ),
            ResourceType::NonFungible { .. } => DroppedBucketResource::NonFungible(
                self_substates
                    .remove(&SubstateOffset::Bucket(BucketOffset::LiquidNonFungible))
                    .map(|s| Into::<LiquidNonFungibleResource>::into(s))
                    .unwrap(),
            ),
        };

        DroppedBucket { info, resource }
    }
}

impl Into<ProofInfoSubstate> for HeapRENode {
    fn into(mut self) -> ProofInfoSubstate {
        let mut self_substates = self.substates.remove(&NodeModuleId::SELF).unwrap();
        self_substates
            .remove(&SubstateOffset::Proof(ProofOffset::Info))
            .unwrap()
            .into()
    }
}
