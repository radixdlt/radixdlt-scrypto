use crate::internal_prelude::*;
use crate::track::interface::IOAccess;
use crate::track::interface::{CallbackError, CanonicalSubstateKey, NodeSubstates};

pub struct Heap {
    nodes: NonIterMap<NodeId, NodeSubstates>,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum HeapRemovePartitionError {
    NodeNotFound(error_models::ReferencedNodeId),
    ModuleNotFound(PartitionNumber),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum HeapRemoveNodeError {
    NodeNotFound(error_models::ReferencedNodeId),
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

    pub fn remove_partition<E, F: FnMut(&Heap, IOAccess) -> Result<(), E>>(
        &mut self,
        node_id: &NodeId,
        partition_number: PartitionNumber,
        on_io_access: &mut F,
    ) -> Result<
        BTreeMap<SubstateKey, IndexedScryptoValue>,
        CallbackError<HeapRemovePartitionError, E>,
    > {
        if let Some(substates) = self.nodes.get_mut(node_id) {
            let partition = substates
                .remove(&partition_number)
                .ok_or(CallbackError::Error(
                    HeapRemovePartitionError::ModuleNotFound(partition_number),
                ))?;

            for (substate_key, substate_value) in &partition {
                on_io_access(
                    self,
                    IOAccess::HeapSubstateUpdated {
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

            Ok(partition)
        } else {
            Err(CallbackError::Error(
                HeapRemovePartitionError::NodeNotFound(node_id.clone().into()),
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
            .and_then(|partition_substates| partition_substates.get(substate_key))
    }

    /// Inserts or overwrites a substate
    pub fn set_substate<E, F: FnMut(&Heap, IOAccess) -> Result<(), E>>(
        &mut self,
        node_id: NodeId,
        partition_number: PartitionNumber,
        substate_key: SubstateKey,
        substate_value: IndexedScryptoValue,
        on_io_access: &mut F,
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

        on_io_access(
            self,
            IOAccess::HeapSubstateUpdated {
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

    pub fn remove_substate<E, F: FnMut(&Heap, IOAccess) -> Result<(), E>>(
        &mut self,
        node_id: &NodeId,
        partition_number: PartitionNumber,
        substate_key: &SubstateKey,
        on_io_access: &mut F,
    ) -> Result<Option<IndexedScryptoValue>, E> {
        let substate_value = self
            .nodes
            .get_mut(node_id)
            .and_then(|n| n.get_mut(&partition_number))
            .and_then(|s| s.remove(substate_key));

        if let Some(value) = &substate_value {
            on_io_access(
                self,
                IOAccess::HeapSubstateUpdated {
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
        &self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        count: u32,
    ) -> Vec<SubstateKey> {
        let node_substates = self.nodes.get(node_id).and_then(|n| n.get(&partition_num));
        if let Some(substates) = node_substates {
            let substate_keys: Vec<SubstateKey> = substates
                .iter()
                .map(|(key, _value)| key.clone())
                .take(count.try_into().unwrap())
                .collect();

            substate_keys
        } else {
            vec![]
        }
    }

    /// Drains the substates from a node's partition. On an non-existing node/partition, this
    /// will return an empty vector
    pub fn drain_substates<E, F: FnMut(&Heap, IOAccess) -> Result<(), E>>(
        &mut self,
        node_id: &NodeId,
        partition_number: PartitionNumber,
        count: u32,
        on_io_access: &mut F,
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
                on_io_access(
                    self,
                    IOAccess::HeapSubstateUpdated {
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
    pub fn create_node<E, F: FnMut(&Heap, IOAccess) -> Result<(), E>>(
        &mut self,
        node_id: NodeId,
        substates: NodeSubstates,
        on_io_access: &mut F,
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
                on_io_access(
                    self,
                    IOAccess::HeapSubstateUpdated {
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
    pub fn remove_node<E, F: FnMut(&Heap, IOAccess) -> Result<(), E>>(
        &mut self,
        node_id: &NodeId,
        on_io_access: &mut F,
    ) -> Result<NodeSubstates, CallbackError<HeapRemoveNodeError, E>> {
        let node_substates = match self.nodes.remove(node_id) {
            Some(node_substates) => node_substates,
            None => Err(CallbackError::Error(HeapRemoveNodeError::NodeNotFound(
                node_id.clone().into(),
            )))?,
        };

        for (partition_number, partition) in &node_substates {
            for (substate_key, substate_value) in partition {
                on_io_access(
                    self,
                    IOAccess::HeapSubstateUpdated {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heap_size_accounting() {
        let mut heap = Heap::new();
        let mut total_size = 0;

        let mut on_io_access = |_: &_, io_access| {
            match io_access {
                IOAccess::HeapSubstateUpdated {
                    canonical_substate_key,
                    old_size,
                    new_size,
                } => {
                    if old_size.is_none() {
                        total_size += canonical_substate_key.len();
                    }
                    if new_size.is_none() {
                        total_size -= canonical_substate_key.len();
                    }

                    total_size += new_size.unwrap_or_default();
                    total_size -= old_size.unwrap_or_default();
                }
                _ => {}
            }

            Result::<(), ()>::Ok(())
        };

        let node_id = NodeId([0u8; NodeId::LENGTH]);
        let partition_number1 = PartitionNumber(5);
        let partition_number2 = PartitionNumber(6);
        let key1 = SubstateKey::Map(scrypto_encode(&"1").unwrap());
        let key2 = SubstateKey::Map(scrypto_encode(&"2").unwrap());
        heap.create_node(
            NodeId([0u8; NodeId::LENGTH]),
            btreemap!(
                partition_number1 => btreemap!(
                    key1.clone() => IndexedScryptoValue::from_typed("A"),
                    key2.clone() => IndexedScryptoValue::from_typed("B"),
                ),
                partition_number2 => btreemap!(
                    key1.clone() => IndexedScryptoValue::from_typed("C"),
                    key2.clone() => IndexedScryptoValue::from_typed("D"),
                )
            ),
            &mut on_io_access,
        )
        .unwrap();
        heap.set_substate(
            node_id,
            partition_number1,
            key1,
            IndexedScryptoValue::from_typed("E"),
            &mut on_io_access,
        )
        .unwrap();
        heap.drain_substates(&node_id, partition_number1, 1, &mut on_io_access)
            .unwrap();
        heap.remove_substate(&node_id, partition_number2, &key2, &mut on_io_access)
            .unwrap();
        heap.remove_partition(&node_id, partition_number2, &mut on_io_access)
            .unwrap();
        heap.remove_node(&node_id, &mut on_io_access).unwrap();
        assert_eq!(total_size, 0);
    }
}
