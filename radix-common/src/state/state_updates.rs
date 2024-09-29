use crate::internal_prelude::*;

/// A tree-like description of all updates that happened to a stored state, to be included as a part
/// of a transaction receipt.
/// This structure is indexed (i.e. uses [`IndexMap`]s where [`Vec`]s could be used) for convenience
/// and performance, since both the source (i.e. Track) and the sink (i.e. Database and API) operate
/// on indexed structures too.
/// This structure maintains partial information on the order of operations (please see individual
/// fields for details), since the end users care about it. Please note that this means multiple
/// instances of [`StateUpdates`] can represent the same transform of state store (i.e. differing
/// only by order of some operations), and hence it is not 100% "canonical form".
#[derive(Debug, Clone, PartialEq, Eq, Sbor, Default)]
pub struct StateUpdates {
    /// Indexed Node-level updates, captured in the order of first update operation to a Node.
    pub by_node: IndexMap<NodeId, NodeStateUpdates>,
}

impl StateUpdates {
    pub fn empty() -> Self {
        Self {
            by_node: Default::default(),
        }
    }

    /// Starts a Node-level update.
    pub fn of_node(&mut self, node_id: NodeId) -> &mut NodeStateUpdates {
        self.by_node
            .entry(node_id)
            .or_insert_with(|| NodeStateUpdates::Delta {
                by_partition: index_map_new(),
            })
    }

    pub fn set_node_updates(
        mut self,
        node_id: impl Into<NodeId>,
        node_updates: NodeStateUpdates,
    ) -> Self {
        self.by_node.insert(node_id.into(), node_updates);
        self
    }

    pub fn set_substate<'a>(
        mut self,
        node_id: impl Into<NodeId>,
        partition_num: PartitionNumber,
        substate_key: impl ResolvableSubstateKey<'a>,
        new_value: impl ScryptoEncode,
    ) -> Self {
        let new_value = scrypto_encode(&new_value).expect("New substate value should be encodable");
        self.of_node(node_id.into())
            .of_partition(partition_num)
            .update_substates([(
                substate_key.into_substate_key(),
                DatabaseUpdate::Set(new_value),
            )]);
        self
    }

    pub fn rebuild_without_empty_entries(&self) -> Self {
        Self {
            by_node: self
                .by_node
                .iter()
                .filter_map(|(node_id, by_partition)| {
                    let by_partition = by_partition.without_empty_entries()?;
                    Some((*node_id, by_partition))
                })
                .collect(),
        }
    }

    pub fn into_legacy(self) -> LegacyStateUpdates {
        self.into()
    }
}

/// A description of all updates that happened to a state of a single Node.
/// Note: currently, we do not support any Node-wide changes (e.g. deleting entire Node); however,
/// we use an enum for potential future development.
#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
pub enum NodeStateUpdates {
    /// A "delta" update to a Node, touching only selected Partitions.
    /// Contains indexed Partition-level updates, captured in the order of first update operation to
    /// a Partition.
    Delta {
        by_partition: IndexMap<PartitionNumber, PartitionStateUpdates>,
    },
}

impl Default for NodeStateUpdates {
    fn default() -> Self {
        NodeStateUpdates::Delta {
            by_partition: index_map_new(),
        }
    }
}

impl NodeStateUpdates {
    pub fn empty() -> Self {
        Self::Delta {
            by_partition: Default::default(),
        }
    }

    pub fn set_substate<'a>(
        mut self,
        partition_num: PartitionNumber,
        key: impl ResolvableSubstateKey<'a>,
        value: impl ScryptoEncode,
    ) -> Self {
        let Self::Delta {
            ref mut by_partition,
        } = self;
        by_partition
            .entry(partition_num)
            .or_default()
            .set_substate(key.into_substate_key(), value);
        self
    }

    /// Starts a Partition-level update.
    pub fn of_partition(&mut self, partition_num: PartitionNumber) -> &mut PartitionStateUpdates {
        match self {
            NodeStateUpdates::Delta { by_partition } => {
                by_partition.entry(partition_num).or_default()
            }
        }
    }

    pub fn of_partition_ref(
        &self,
        partition_num: PartitionNumber,
    ) -> Option<&PartitionStateUpdates> {
        match self {
            NodeStateUpdates::Delta { by_partition } => by_partition.get(&partition_num),
        }
    }

    pub fn without_empty_entries(&self) -> Option<Self> {
        match self {
            NodeStateUpdates::Delta { by_partition } => {
                let replaced = by_partition
                    .iter()
                    .filter_map(|(partition_num, partition_state_updates)| {
                        let new_substate = partition_state_updates.without_empty_entries()?;
                        Some((*partition_num, new_substate))
                    })
                    .collect::<IndexMap<_, _>>();
                if replaced.len() > 0 {
                    Some(NodeStateUpdates::Delta {
                        by_partition: replaced,
                    })
                } else {
                    None
                }
            }
        }
    }
}

/// A description of all updates that happened to a state of a single Partition.
#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
pub enum PartitionStateUpdates {
    /// A "delta" update to a Partition, touching only selected Substates.
    /// Contains indexed Substate-level updates, captured in the order of first update operation to
    /// a Substate.
    Delta {
        by_substate: IndexMap<SubstateKey, DatabaseUpdate>,
    },
    /// A batch update.
    Batch(BatchPartitionStateUpdate),
}

impl Default for PartitionStateUpdates {
    fn default() -> Self {
        PartitionStateUpdates::Delta {
            by_substate: index_map_new(),
        }
    }
}

impl PartitionStateUpdates {
    pub fn set_substate<'a>(
        &mut self,
        key: impl ResolvableSubstateKey<'a>,
        value: impl ScryptoEncode,
    ) {
        let value = scrypto_encode(&value).expect("New substate value should be encodable");
        match self {
            PartitionStateUpdates::Delta { by_substate } => {
                by_substate.insert(key.into_substate_key(), DatabaseUpdate::Set(value));
            }
            PartitionStateUpdates::Batch(BatchPartitionStateUpdate::Reset {
                new_substate_values,
            }) => {
                new_substate_values.insert(key.into_substate_key(), value);
            }
        }
    }

    /// Resets the partition to an empty state.
    pub fn delete(&mut self) {
        *self = PartitionStateUpdates::Batch(BatchPartitionStateUpdate::Reset {
            new_substate_values: index_map_new(),
        });
    }

    pub fn contains_set_update_for(&self, key: &SubstateKey) -> bool {
        match self {
            PartitionStateUpdates::Delta { by_substate } => {
                matches!(by_substate.get(key), Some(DatabaseUpdate::Set(_)))
            }
            PartitionStateUpdates::Batch(BatchPartitionStateUpdate::Reset {
                new_substate_values,
            }) => new_substate_values.contains_key(key),
        }
    }

    /// Applies the given updates on top of the current updates to the partition.
    pub fn update_substates(
        &mut self,
        updates: impl IntoIterator<Item = (SubstateKey, DatabaseUpdate)>,
    ) {
        match self {
            PartitionStateUpdates::Delta { by_substate } => by_substate.extend(updates),
            PartitionStateUpdates::Batch(BatchPartitionStateUpdate::Reset {
                new_substate_values,
            }) => {
                for (substate_key, database_update) in updates {
                    match database_update {
                        DatabaseUpdate::Set(new_value) => {
                            new_substate_values.insert(substate_key, new_value);
                        }
                        DatabaseUpdate::Delete => {
                            let existed = new_substate_values.swap_remove(&substate_key).is_some();
                            if !existed {
                                panic!("inconsistent update: delete of substate {:?} not existing in reset partition", substate_key);
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn without_empty_entries(&self) -> Option<Self> {
        match self {
            PartitionStateUpdates::Delta { by_substate } => {
                if by_substate.len() > 0 {
                    Some(PartitionStateUpdates::Delta {
                        by_substate: by_substate.clone(),
                    })
                } else {
                    None
                }
            }
            PartitionStateUpdates::Batch(x) => {
                // We shouldn't filter out batch updates like resets, even if they set nothing new
                Some(PartitionStateUpdates::Batch(x.clone()))
            }
        }
    }

    pub fn iter_map_entries(&self) -> Box<dyn Iterator<Item = (&MapKey, DatabaseUpdateRef)> + '_> {
        match self {
            PartitionStateUpdates::Delta { by_substate } => {
                Box::new(by_substate.iter().filter_map(|(key, value)| match key {
                    SubstateKey::Map(map_key) => {
                        let value = match value {
                            DatabaseUpdate::Set(value) => DatabaseUpdateRef::Set(value),
                            DatabaseUpdate::Delete => DatabaseUpdateRef::Delete,
                        };
                        Some((map_key, value))
                    }
                    SubstateKey::Field(_) | SubstateKey::Sorted(_) => None,
                }))
            }
            PartitionStateUpdates::Batch(BatchPartitionStateUpdate::Reset {
                new_substate_values,
            }) => Box::new(
                new_substate_values
                    .iter()
                    .filter_map(|(key, value)| match key {
                        SubstateKey::Map(map_key) => Some((map_key, DatabaseUpdateRef::Set(value))),
                        SubstateKey::Field(_) | SubstateKey::Sorted(_) => None,
                    }),
            ),
        }
    }
}

/// A description of a batch update affecting an entire Partition.
#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
pub enum BatchPartitionStateUpdate {
    /// A reset, dropping all Substates of a partition and replacing them with a new set.
    /// Contains indexed new Substate values, captured in the order of creation of a Substate.
    Reset {
        new_substate_values: IndexMap<SubstateKey, DbSubstateValue>,
    },
}

/// An update of a single substate's value.
#[derive(Debug, Clone, Hash, PartialEq, Eq, Sbor, PartialOrd, Ord)]
pub enum DatabaseUpdate {
    Set(DbSubstateValue),
    Delete,
}

impl DatabaseUpdate {
    pub fn as_ref(&self) -> DatabaseUpdateRef<'_> {
        match self {
            DatabaseUpdate::Set(update) => DatabaseUpdateRef::Set(update),
            DatabaseUpdate::Delete => DatabaseUpdateRef::Delete,
        }
    }
}

/// A 1:1 counterpart of [`DatabaseUpdate`], but operating on references.
pub enum DatabaseUpdateRef<'v> {
    Set(&'v [u8]),
    Delete,
}

/// A raw substate value stored by the database.
pub type DbSubstateValue = Vec<u8>;

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
