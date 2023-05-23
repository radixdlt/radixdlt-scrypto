use radix_engine_store_interface::interface::*;
use sbor::rust::prelude::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct InMemorySubstateDatabase {
    partitions: IndexMap<DbPartitionKey, BTreeMap<DbSortKey, DbSubstateValue>>,
}

impl InMemorySubstateDatabase {
    pub fn standard() -> Self {
        Self {
            partitions: index_map_new(),
        }
    }
}

impl SubstateDatabase for InMemorySubstateDatabase {
    fn get_substate(
        &self,
        partition_key: &DbPartitionKey,
        sort_key: &DbSortKey,
    ) -> Option<DbSubstateValue> {
        self.partitions
            .get(partition_key)
            .and_then(|partition| partition.get(sort_key))
            .cloned()
    }

    fn list_entries(
        &self,
        partition_key: &DbPartitionKey,
    ) -> Box<dyn Iterator<Item = PartitionEntry> + '_> {
        let iter = self
            .partitions
            .get(partition_key)
            .into_iter()
            .flat_map(|partition| partition.iter())
            .map(|(key, substate)| (key.clone(), substate.clone()));

        Box::new(iter)
    }
}

impl CommittableSubstateDatabase for InMemorySubstateDatabase {
    fn commit(&mut self, database_updates: &DatabaseUpdates) {
        for (partition_key, partition_updates) in database_updates {
            let partition = self
                .partitions
                .entry(partition_key.clone())
                .or_insert_with(|| BTreeMap::new());
            for (sort_key, update) in partition_updates {
                match update {
                    DatabaseUpdate::Set(substate_value) => {
                        partition.insert(sort_key.clone(), substate_value.clone())
                    }
                    DatabaseUpdate::Delete => partition.remove(sort_key),
                };
            }
            if partition.is_empty() {
                self.partitions.remove(partition_key);
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
