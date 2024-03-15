use crate::internal_prelude::*;
use crate::track::{
    BatchPartitionStateUpdate, NodeStateUpdates, PartitionStateUpdates, StateUpdates,
};
use radix_engine_common::types::{NodeId, PartitionNumber, SubstateKey};
use radix_rust::prelude::{index_map_new, index_set_new, IndexMap, IndexSet};
use radix_substate_store_interface::interface::DatabaseUpdate;

/// A legacy format capturing the same information as new [`StateUpdates`].
/// Note to migrators: this struct will live only temporarily. The new one should be preferred (and
/// should be the only persisted one). Please use the [`From`] utilities below for easy migration.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, Default)]
pub struct LegacyStateUpdates {
    /// A set of partitions that were entirely deleted.
    /// Note: this batch changes should be applied *before* the [`system_updates`] below (i.e.
    /// allowing a latter individual substate creation to be applied).
    pub partition_deletions: IndexSet<(NodeId, PartitionNumber)>,

    /// A set of individual substate updates (indexed by partition and substate key).
    /// Note to migrators: the type seen below used to be named `SystemUpdates`.
    pub system_updates: IndexMap<(NodeId, PartitionNumber), IndexMap<SubstateKey, DatabaseUpdate>>,
}

impl From<LegacyStateUpdates> for StateUpdates {
    fn from(legacy_state_updates: LegacyStateUpdates) -> Self {
        let mut state_updates = StateUpdates {
            by_node: index_map_new(),
        };
        for (node_id, partition_num) in legacy_state_updates.partition_deletions {
            state_updates
                .of_node(node_id)
                .of_partition(partition_num)
                .delete();
        }
        for ((node_id, partition_num), by_substate) in legacy_state_updates.system_updates {
            state_updates
                .of_node(node_id)
                .of_partition(partition_num)
                .update_substates(by_substate);
        }
        state_updates
    }
}

impl From<StateUpdates> for LegacyStateUpdates {
    fn from(state_updates: StateUpdates) -> Self {
        let mut partition_deletions = index_set_new();
        let mut system_updates = index_map_new();
        for (node_id, node_state_updates) in state_updates.by_node {
            match node_state_updates {
                NodeStateUpdates::Delta { by_partition } => {
                    for (partition_num, partition_state_updates) in by_partition {
                        let node_partition = (node_id.clone(), partition_num);
                        match partition_state_updates {
                            PartitionStateUpdates::Delta { by_substate } => {
                                system_updates.insert(node_partition, by_substate);
                            }
                            PartitionStateUpdates::Batch(batch) => match batch {
                                BatchPartitionStateUpdate::Reset {
                                    new_substate_values,
                                } => {
                                    partition_deletions.insert(node_partition.clone());
                                    let as_updates = new_substate_values
                                        .into_iter()
                                        .map(|(substate_key, value)| {
                                            (substate_key, DatabaseUpdate::Set(value))
                                        })
                                        .collect();
                                    system_updates.insert(node_partition, as_updates);
                                }
                            },
                        }
                    }
                }
            }
        }
        LegacyStateUpdates {
            partition_deletions,
            system_updates,
        }
    }
}
