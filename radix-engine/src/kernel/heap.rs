use crate::blueprints::resource::*;
use crate::types::*;
use radix_engine_interface::blueprints::resource::{
    LiquidFungibleResource, LiquidNonFungibleResource, ResourceType,
};
use radix_engine_interface::math::Decimal;
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

    pub fn get_substate_virtualize<F: FnOnce() -> IndexedScryptoValue>(
        &mut self,
        node_id: &NodeId,
        module_id: ModuleId,
        substate_key: &SubstateKey,
        virtualize: F,
    ) -> &IndexedScryptoValue {
        let entry = self
            .nodes
            .entry(*node_id)
            .or_insert(BTreeMap::new())
            .entry(module_id)
            .or_insert(BTreeMap::new())
            .entry(substate_key.clone());
        if let Entry::Vacant(e) = entry {
            let value = virtualize();
            e.insert(value);
        }

        self.nodes
            .get(node_id)
            .and_then(|node_substates| node_substates.get(&module_id))
            .and_then(|module_substates| module_substates.get(substate_key))
            .unwrap()
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
    pub fn set_substate(
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

    pub fn delete_substate(
        &mut self,
        node_id: &NodeId,
        module_id: ModuleId,
        substate_key: &SubstateKey,
    ) -> Option<IndexedScryptoValue> {
        self.nodes
            .get_mut(node_id)
            .and_then(|n| n.get_mut(&module_id))
            .and_then(|s| s.remove(substate_key))
    }

    pub fn scan_substates(
        &mut self,
        node_id: &NodeId,
        module_id: ModuleId,
        count: u32,
    ) -> Vec<IndexedScryptoValue> {
        let node_substates = self
            .nodes
            .get_mut(node_id)
            .and_then(|n| n.get_mut(&module_id));
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
        module_id: ModuleId,
        count: u32,
    ) -> Vec<IndexedScryptoValue> {
        let node_substates = self
            .nodes
            .get_mut(node_id)
            .and_then(|n| n.get_mut(&module_id));
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

impl Into<DroppedBucket> for NodeSubstates {
    fn into(mut self) -> DroppedBucket {
        let mut module_substates = self.remove(&SysModuleId::User.into()).unwrap();

        let info: BucketInfoSubstate = module_substates
            .remove(&BucketOffset::Info.into())
            .map(|x| x.as_typed().unwrap())
            .unwrap();

        let resource = match info.resource_type {
            ResourceType::Fungible { .. } => DroppedBucketResource::Fungible(
                module_substates
                    .remove(&BucketOffset::LiquidFungible.into())
                    .map(|x| x.as_typed().unwrap())
                    .unwrap(),
            ),
            ResourceType::NonFungible { .. } => DroppedBucketResource::NonFungible(
                module_substates
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

impl Into<DroppedProof> for NodeSubstates {
    fn into(mut self) -> DroppedProof {
        let mut module = self.remove(&SysModuleId::User.into()).unwrap();

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
