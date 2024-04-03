use crate::state_tree::tree_store::{TypedInMemoryTreeStore, Version};
use crate::state_tree::{list_substate_hashes_at_version, put_at_next_version};
use radix_common::prelude::*;
use radix_substate_store_interface::interface::{
    CommittableSubstateDatabase, DatabaseUpdates, DbPartitionKey, DbSortKey, DbSubstateValue,
    ListableSubstateDatabase, PartitionEntry, SubstateDatabase,
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct StateTreeUpdatingDatabase<D> {
    underlying: D,
    tree_store: TypedInMemoryTreeStore,
    current_version: Version,
    current_hash: Hash,
}

impl<D> StateTreeUpdatingDatabase<D> {
    pub fn new(underlying: D) -> Self {
        StateTreeUpdatingDatabase {
            underlying,
            tree_store: TypedInMemoryTreeStore::new().with_pruning_enabled(),
            current_version: 0,
            current_hash: Hash([0; Hash::LENGTH]),
        }
    }

    pub fn get_current_root_hash(&self) -> Hash {
        self.current_hash
    }

    pub fn get_current_version(&self) -> Version {
        self.current_version
    }

    pub fn list_substate_hashes(&self) -> IndexMap<DbPartitionKey, IndexMap<DbSortKey, Hash>> {
        list_substate_hashes_at_version(&self.tree_store, self.current_version)
    }

    fn update_with(&mut self, db_updates: &DatabaseUpdates) {
        self.current_hash = put_at_next_version(
            &mut self.tree_store,
            Some(self.current_version).filter(|version| *version > 0),
            db_updates,
        );
        self.current_version += 1;
    }
}

impl<D: SubstateDatabase> SubstateDatabase for StateTreeUpdatingDatabase<D> {
    fn get_substate(
        &self,
        partition_key: &DbPartitionKey,
        sort_key: &DbSortKey,
    ) -> Option<DbSubstateValue> {
        self.underlying.get_substate(partition_key, sort_key)
    }

    fn list_entries_from(
        &self,
        partition_key: &DbPartitionKey,
        from_sort_key: Option<&DbSortKey>,
    ) -> Box<dyn Iterator<Item = PartitionEntry> + '_> {
        self.underlying
            .list_entries_from(partition_key, from_sort_key)
    }
}

impl<D: ListableSubstateDatabase> ListableSubstateDatabase for StateTreeUpdatingDatabase<D> {
    fn list_partition_keys(&self) -> Box<dyn Iterator<Item = DbPartitionKey> + '_> {
        self.underlying.list_partition_keys()
    }
}

impl<D: CommittableSubstateDatabase> CommittableSubstateDatabase for StateTreeUpdatingDatabase<D> {
    fn commit(&mut self, database_updates: &DatabaseUpdates) {
        self.underlying.commit(database_updates);
        self.update_with(database_updates);
    }
}
