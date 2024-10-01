use crate::state_tree::tree_store::{TypedInMemoryTreeStore, Version};
use crate::state_tree::{list_substate_hashes_at_version, put_at_next_version};
use radix_common::prelude::*;
use radix_substate_store_interface::interface::*;

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
    fn get_raw_substate_by_db_key(
        &self,
        partition_key: &DbPartitionKey,
        sort_key: &DbSortKey,
    ) -> Option<DbSubstateValue> {
        self.underlying
            .get_raw_substate_by_db_key(partition_key, sort_key)
    }

    fn list_raw_values_from_db_key(
        &self,
        partition_key: &DbPartitionKey,
        from_sort_key: Option<&DbSortKey>,
    ) -> Box<dyn Iterator<Item = PartitionEntry> + '_> {
        self.underlying
            .list_raw_values_from_db_key(partition_key, from_sort_key)
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

impl<D> StateTreeUpdatingDatabase<D>
where
    D: SubstateDatabase + ListableSubstateDatabase,
{
    pub fn validate_state_tree_matches_substate_store(
        &self,
    ) -> Result<(), StateTreeValidationError> {
        let hashes_from_tree = self.list_substate_hashes();
        if hashes_from_tree.keys().cloned().collect::<HashSet<_>>()
            != self.list_partition_keys().collect::<HashSet<_>>()
        {
            return Err(StateTreeValidationError::NotAllPartitionsAreFoundInBothHashesAndDatabase);
        }
        for (db_partition_key, by_db_sort_key) in hashes_from_tree {
            if by_db_sort_key.into_iter().collect::<HashMap<_, _>>()
                != self
                    .list_raw_values_from_db_key(&db_partition_key, None)
                    .map(|(db_sort_key, substate_value)| (db_sort_key, hash(substate_value)))
                    .collect::<HashMap<_, _>>()
            {
                return Err(StateTreeValidationError::MismatchInPartitionSubstates(
                    db_partition_key.clone(),
                ));
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StateTreeValidationError {
    NotAllPartitionsAreFoundInBothHashesAndDatabase,
    MismatchInPartitionSubstates(DbPartitionKey),
}
