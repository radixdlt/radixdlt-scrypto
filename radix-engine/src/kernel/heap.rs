use crate::track::interface::{CallbackError, CanonicalSubstateKey, NodeSubstates};
use crate::types::*;
use crate::{blueprints::resource::*, track::interface::StoreAccess};
use radix_engine_interface::blueprints::resource::{
    LiquidFungibleResource, LiquidNonFungibleResource, LockedFungibleResource,
    LockedNonFungibleResource,
};

pub struct Heap {
    nodes: NonIterMap<NodeId, NodeSubstates>,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum HeapRemovePartitionError {
    NodeNotFound(NodeId),
    ModuleNotFound(PartitionNumber),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum HeapRemoveNodeError {
    NodeNotFound(NodeId),
}

impl Heap {
    pub fn new() -> Self {
        Self {
            nodes: NonIterMap::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn remove_module<'x, E: 'x, F: FnMut(&Heap, StoreAccess) -> Result<(), E> + 'x>(
        &mut self,
        node_id: &NodeId,
        partition_number: PartitionNumber,
        on_store_access: &'x mut F,
    ) -> Result<
        BTreeMap<SubstateKey, IndexedScryptoValue>,
        CallbackError<HeapRemovePartitionError, E>,
    > {
        if let Some(modules) = self.nodes.get_mut(node_id) {
            let module = modules
                .remove(&partition_number)
                .ok_or(CallbackError::Error(
                    HeapRemovePartitionError::ModuleNotFound(partition_number),
                ))?;

            for (substate_key, substate_value) in &module {
                on_store_access(
                    self,
                    StoreAccess::SubstateUpdatedInHeap {
                        canonical_substate_key: CanonicalSubstateKey {
                            node_id: *node_id,
                            partition_number,
                            substate_key: substate_key.clone(),
                        },
                        old_size: Some(substate_value.len()),
                        new_size: None,
                    },
                )
                .map_err(CallbackError::CallbackError)?;
            }

            Ok(module)
        } else {
            Err(CallbackError::Error(
                HeapRemovePartitionError::NodeNotFound(node_id.clone()),
            ))
        }
    }

    /// Reads a substate
    pub fn get_substate(
        &self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        substate_key: &SubstateKey,
    ) -> Option<&IndexedScryptoValue> {
        self.nodes
            .get(node_id)
            .and_then(|node| node.get(&partition_num))
            .and_then(|module_substates| module_substates.get(substate_key))
    }

    /// Inserts or overwrites a substate
    pub fn set_substate<'x, E: 'x, F: FnMut(&Heap, StoreAccess) -> Result<(), E> + 'x>(
        &mut self,
        node_id: NodeId,
        partition_number: PartitionNumber,
        substate_key: SubstateKey,
        substate_value: IndexedScryptoValue,
        on_store_access: &'x mut F,
    ) -> Result<(), E> {
        let entry = self
            .nodes
            .entry(node_id)
            .or_insert_with(|| NodeSubstates::default())
            .entry(partition_number)
            .or_default()
            .entry(substate_key.clone());

        let old_size;
        let new_size = Some(substate_value.len());
        match entry {
            btree_map::Entry::Vacant(e) => {
                old_size = None;
                e.insert(substate_value);
            }
            btree_map::Entry::Occupied(mut e) => {
                old_size = Some(e.get().len());
                e.insert(substate_value);
            }
        }

        on_store_access(
            self,
            StoreAccess::SubstateUpdatedInHeap {
                canonical_substate_key: CanonicalSubstateKey {
                    node_id,
                    partition_number,
                    substate_key,
                },
                old_size,
                new_size,
            },
        )?;

        Ok(())
    }

    pub fn remove_substate<'x, E: 'x, F: FnMut(&Heap, StoreAccess) -> Result<(), E> + 'x>(
        &mut self,
        node_id: &NodeId,
        partition_number: PartitionNumber,
        substate_key: &SubstateKey,
        on_store_access: &'x mut F,
    ) -> Result<Option<IndexedScryptoValue>, E> {
        let substate_value = self
            .nodes
            .get_mut(node_id)
            .and_then(|n| n.get_mut(&partition_number))
            .and_then(|s| s.remove(substate_key));

        if let Some(value) = &substate_value {
            on_store_access(
                self,
                StoreAccess::SubstateUpdatedInHeap {
                    canonical_substate_key: CanonicalSubstateKey {
                        node_id: *node_id,
                        partition_number,
                        substate_key: substate_key.clone(),
                    },
                    old_size: Some(value.len()),
                    new_size: None,
                },
            )?;
        }

        Ok(substate_value)
    }

    /// Scans the keys of a node's partition. On an non-existing node/partition, this
    /// will return an empty vector
    pub fn scan_keys(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        count: u32,
    ) -> Vec<SubstateKey> {
        let node_substates = self
            .nodes
            .get_mut(node_id)
            .and_then(|n| n.get_mut(&partition_num));
        if let Some(substates) = node_substates {
            let substates: Vec<SubstateKey> = substates
                .iter()
                .map(|(key, _value)| key.clone())
                .take(count.try_into().unwrap())
                .collect();

            substates
        } else {
            vec![]
        }
    }

    /// Drains the substates from a node's partition. On an non-existing node/partition, this
    /// will return an empty vector
    pub fn drain_substates<'x, E: 'x, F: FnMut(&Heap, StoreAccess) -> Result<(), E> + 'x>(
        &mut self,
        node_id: &NodeId,
        partition_number: PartitionNumber,
        count: u32,
        on_store_access: &'x mut F,
    ) -> Result<Vec<(SubstateKey, IndexedScryptoValue)>, E> {
        let node_substates = self
            .nodes
            .get_mut(node_id)
            .and_then(|n| n.get_mut(&partition_number));
        if let Some(substates) = node_substates {
            let keys: Vec<SubstateKey> = substates
                .iter()
                .map(|(key, _)| key.clone())
                .take(count.try_into().unwrap())
                .collect();

            let mut items = Vec::new();

            for key in keys {
                let value = substates.remove(&key).unwrap();
                items.push((key, value));
            }

            for (key, value) in &items {
                on_store_access(
                    self,
                    StoreAccess::SubstateUpdatedInHeap {
                        canonical_substate_key: CanonicalSubstateKey {
                            node_id: *node_id,
                            partition_number,
                            substate_key: key.clone(),
                        },
                        old_size: Some(value.len()),
                        new_size: None,
                    },
                )?;
            }

            Ok(items)
        } else {
            Ok(vec![])
        }
    }

    /// Inserts a new node to heap.
    pub fn create_node<'x, E: 'x, F: FnMut(&Heap, StoreAccess) -> Result<(), E> + 'x>(
        &mut self,
        node_id: NodeId,
        substates: NodeSubstates,
        on_store_access: &'x mut F,
    ) -> Result<(), E> {
        assert!(!self.nodes.contains_key(&node_id));

        let sizes: IndexMap<PartitionNumber, IndexMap<SubstateKey, usize>> = substates
            .iter()
            .map(|(k, v)| {
                (
                    k.clone(),
                    v.iter().map(|(k, v)| (k.clone(), v.len())).collect(),
                )
            })
            .collect();

        self.nodes.insert(node_id, substates);

        for (partition_number, partition) in sizes {
            for (substate_key, substate_size) in partition {
                on_store_access(
                    self,
                    StoreAccess::SubstateUpdatedInHeap {
                        canonical_substate_key: CanonicalSubstateKey {
                            node_id,
                            partition_number,
                            substate_key: substate_key.clone(),
                        },
                        old_size: None,
                        new_size: Some(substate_size),
                    },
                )?;
            }
        }

        Ok(())
    }

    /// Removes node.
    pub fn remove_node<'x, E: 'x, F: FnMut(&Heap, StoreAccess) -> Result<(), E> + 'x>(
        &mut self,
        node_id: &NodeId,
        on_store_access: &'x mut F,
    ) -> Result<NodeSubstates, CallbackError<HeapRemoveNodeError, E>> {
        let node_substates = match self.nodes.remove(node_id) {
            Some(node_substates) => node_substates,
            None => Err(CallbackError::Error(HeapRemoveNodeError::NodeNotFound(
                node_id.clone(),
            )))?,
        };

        for (partition_number, partition) in &node_substates {
            for (substate_key, substate_value) in partition {
                on_store_access(
                    self,
                    StoreAccess::SubstateUpdatedInHeap {
                        canonical_substate_key: CanonicalSubstateKey {
                            node_id: *node_id,
                            partition_number: *partition_number,
                            substate_key: substate_key.clone(),
                        },
                        old_size: Some(substate_value.len()),
                        new_size: None,
                    },
                )
                .map_err(CallbackError::CallbackError)?;
            }
        }

        Ok(node_substates)
    }
}

#[derive(Debug)]
pub struct DroppedFungibleBucket {
    pub liquid: LiquidFungibleResource,
    pub locked: LockedFungibleResource,
}

#[derive(Debug)]
pub struct DroppedNonFungibleBucket {
    pub liquid: LiquidNonFungibleResource,
    pub locked: LockedNonFungibleResource,
}

impl Into<DroppedFungibleBucket> for Vec<Vec<u8>> {
    fn into(self) -> DroppedFungibleBucket {
        let liquid: LiquidFungibleResource =
            scrypto_decode(&self[FungibleBucketField::Liquid as usize]).unwrap();
        let locked: LockedFungibleResource =
            scrypto_decode(&self[FungibleBucketField::Locked as usize]).unwrap();

        DroppedFungibleBucket { liquid, locked }
    }
}

impl Into<DroppedNonFungibleBucket> for Vec<Vec<u8>> {
    fn into(self) -> DroppedNonFungibleBucket {
        let liquid: LiquidNonFungibleResource =
            scrypto_decode(&self[NonFungibleBucketField::Liquid as usize]).unwrap();
        let locked: LockedNonFungibleResource =
            scrypto_decode(&self[NonFungibleBucketField::Locked as usize]).unwrap();

        DroppedNonFungibleBucket { liquid, locked }
    }
}

pub struct DroppedFungibleProof {
    pub moveable: ProofMoveableSubstate,
    pub fungible_proof: FungibleProofSubstate,
}

pub struct DroppedNonFungibleProof {
    pub moveable: ProofMoveableSubstate,
    pub non_fungible_proof: NonFungibleProofSubstate,
}

impl Into<DroppedFungibleProof> for Vec<Vec<u8>> {
    fn into(self) -> DroppedFungibleProof {
        let moveable: ProofMoveableSubstate =
            scrypto_decode(&self[FungibleProofField::Moveable as usize]).unwrap();
        let fungible_proof: FungibleProofSubstate =
            scrypto_decode(&self[FungibleProofField::ProofRefs as usize]).unwrap();

        DroppedFungibleProof {
            moveable,
            fungible_proof,
        }
    }
}

impl Into<DroppedNonFungibleProof> for Vec<Vec<u8>> {
    fn into(self) -> DroppedNonFungibleProof {
        let moveable: ProofMoveableSubstate =
            scrypto_decode(&self[FungibleProofField::Moveable as usize]).unwrap();
        let non_fungible_proof: NonFungibleProofSubstate =
            scrypto_decode(&self[FungibleProofField::ProofRefs as usize]).unwrap();

        DroppedNonFungibleProof {
            moveable,
            non_fungible_proof,
        }
    }
}
