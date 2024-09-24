use radix_common::prelude::*;
use radix_substate_store_interface::interface::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct InMemorySubstateDatabase {
    partitions: BTreeMap<DbPartitionKey, BTreeMap<DbSortKey, DbSubstateValue>>,
}

impl InMemorySubstateDatabase {
    pub fn standard() -> Self {
        Self {
            partitions: BTreeMap::new(),
        }
    }
}

impl SubstateDatabase for InMemorySubstateDatabase {
    fn get_raw_substate_by_db_key(
        &self,
        partition_key: &DbPartitionKey,
        sort_key: &DbSortKey,
    ) -> Option<DbSubstateValue> {
        self.partitions
            .get(partition_key)
            .and_then(|partition| partition.get(sort_key))
            .cloned()
    }

    fn list_raw_values_from_db_key(
        &self,
        partition_key: &DbPartitionKey,
        from_sort_key: Option<&DbSortKey>,
    ) -> Box<dyn Iterator<Item = PartitionEntry> + '_> {
        let from_sort_key = from_sort_key.cloned();
        let iter = self
            .partitions
            .get(partition_key)
            .into_iter()
            .flat_map(|partition| partition.iter())
            .skip_while(move |(key, _substate)| Some(*key) < from_sort_key.as_ref())
            .map(|(key, substate)| (key.clone(), substate.clone()));

        Box::new(iter)
    }
}

impl CommittableSubstateDatabase for InMemorySubstateDatabase {
    fn commit(&mut self, database_updates: &DatabaseUpdates) {
        for (node_key, node_updates) in &database_updates.node_updates {
            for (partition_num, partition_updates) in &node_updates.partition_updates {
                let partition_key = DbPartitionKey {
                    node_key: node_key.clone(),
                    partition_num: partition_num.clone(),
                };
                let partition = self
                    .partitions
                    .entry(partition_key.clone())
                    .or_insert_with(|| BTreeMap::new());
                match partition_updates {
                    PartitionDatabaseUpdates::Delta { substate_updates } => {
                        for (sort_key, update) in substate_updates {
                            match update {
                                DatabaseUpdate::Set(substate_value) => {
                                    partition.insert(sort_key.clone(), substate_value.clone())
                                }
                                DatabaseUpdate::Delete => partition.remove(sort_key),
                            };
                        }
                    }
                    PartitionDatabaseUpdates::Reset {
                        new_substate_values,
                    } => {
                        *partition = BTreeMap::from_iter(
                            new_substate_values
                                .iter()
                                .map(|(sort_key, value)| (sort_key.clone(), value.clone())),
                        )
                    }
                }
                if partition.is_empty() {
                    self.partitions.remove(&partition_key);
                }
            }
        }
    }
}

impl ListableSubstateDatabase for InMemorySubstateDatabase {
    fn list_partition_keys(&self) -> Box<dyn Iterator<Item = DbPartitionKey> + '_> {
        let partition_iter = self.partitions.iter().map(|(key, _)| key.clone());
        Box::new(partition_iter)
    }
}
