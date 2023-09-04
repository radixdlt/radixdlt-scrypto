use radix_engine::types::*;
use radix_engine_store_interface::interface::{
    CommittableSubstateDatabase, DatabaseUpdate, DatabaseUpdates, DbPartitionKey, DbSortKey,
    DbSubstateValue, ListableSubstateDatabase, PartitionEntry, SubstateDatabase,
};
use radix_engine_stores::hash_tree::tree_store::{TypedInMemoryTreeStore, Version};
use radix_engine_stores::hash_tree::{
    list_substate_hashes_at_version, put_at_next_version, SubstateHashChange,
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct HashTreeUpdatingDatabase<D> {
    underlying: D,
    tree_store: TypedInMemoryTreeStore,
    current_version: Version,
    current_hash: Hash,
}

impl<D> HashTreeUpdatingDatabase<D> {
    pub fn new(underlying: D) -> Self {
        HashTreeUpdatingDatabase {
            underlying,
            tree_store: TypedInMemoryTreeStore::new(),
            current_version: 0,
            current_hash: Hash([0; Hash::LENGTH]),
        }
    }

    pub fn get_current(&self) -> Hash {
        self.current_hash
    }

    pub fn list_hashes(&mut self) -> IndexMap<DbPartitionKey, IndexMap<DbSortKey, Hash>> {
        list_substate_hashes_at_version(&mut self.tree_store, self.current_version)
    }

    fn update_with(&mut self, db_updates: &DatabaseUpdates) {
        let mut hash_changes = Vec::new();
        for (db_partition_key, partition_update) in db_updates {
            for (db_sort_key, db_update) in partition_update {
                let hash_change = SubstateHashChange::new(
                    (db_partition_key.clone(), db_sort_key.clone()),
                    match db_update {
                        DatabaseUpdate::Set(v) => Some(hash(v)),
                        DatabaseUpdate::Delete => None,
                    },
                );
                hash_changes.push(hash_change);
            }
        }

        self.current_hash = put_at_next_version(
            &mut self.tree_store,
            Some(self.current_version).filter(|version| *version > 0),
            hash_changes,
        );
        self.current_version += 1;
    }
}

impl<D: SubstateDatabase> SubstateDatabase for HashTreeUpdatingDatabase<D> {
    fn get_substate(
        &self,
        partition_key: &DbPartitionKey,
        sort_key: &DbSortKey,
    ) -> Option<DbSubstateValue> {
        self.underlying.get_substate(partition_key, sort_key)
    }

    fn list_entries(
        &self,
        partition_key: &DbPartitionKey,
    ) -> Box<dyn Iterator<Item = PartitionEntry> + '_> {
        self.underlying.list_entries(partition_key)
    }
}

impl<D: ListableSubstateDatabase> ListableSubstateDatabase for HashTreeUpdatingDatabase<D> {
    fn list_partition_keys(&self) -> Box<dyn Iterator<Item = DbPartitionKey> + '_> {
        self.underlying.list_partition_keys()
    }
}

impl<D: CommittableSubstateDatabase> CommittableSubstateDatabase for HashTreeUpdatingDatabase<D> {
    fn commit(&mut self, database_updates: &DatabaseUpdates) {
        self.underlying.commit(database_updates);
        self.update_with(database_updates);
    }
}
