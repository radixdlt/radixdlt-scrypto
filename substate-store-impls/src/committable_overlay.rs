use crate::memory_db::*;
use itertools::Itertools;
use sbor::prelude::*;
use substate_store_interface::interface::*;

pub struct CommittableOverlay<T> {
    overlay: InMemorySubstateDatabase,
    db: T,
}

impl<T> CommittableOverlay<T> {
    pub fn new(db: T) -> Self {
        Self {
            overlay: InMemorySubstateDatabase::standard(),
            db,
        }
    }

    pub fn underlying_db(&self) -> &T {
        &self.db
    }

    pub fn into_inner(self) -> T {
        self.db
    }
}

impl<T> SubstateDatabase for CommittableOverlay<T>
where
    T: SubstateDatabase,
{
    fn get_substate(
        &self,
        partition_key: &DbPartitionKey,
        sort_key: &DbSortKey,
    ) -> Option<DbSubstateValue> {
        self.overlay
            .get_substate(partition_key, sort_key)
            .or_else(|| self.db.get_substate(partition_key, sort_key))
    }

    fn list_entries_from(
        &self,
        partition_key: &DbPartitionKey,
        from_sort_key: Option<&DbSortKey>,
    ) -> Box<dyn Iterator<Item = PartitionEntry> + '_> {
        let overlay_iterator = self.overlay.list_entries_from(partition_key, from_sort_key);
        let db_iterator = self.db.list_entries_from(partition_key, from_sort_key);

        // The unique_by method retains the first items that it encounters and ignores other items.
        // So, we start with the overlay iterator and then the db iterator so that if a substate is
        // in both the database and the overlay then the overlay substate will be included in the
        // output.
        Box::new(
            overlay_iterator
                .chain(db_iterator)
                .unique_by(|(key, _)| key.clone()),
        )
    }
}

impl<T> CommittableSubstateDatabase for CommittableOverlay<T> {
    fn commit(&mut self, database_updates: &DatabaseUpdates) {
        self.overlay.commit(database_updates)
    }
}

impl<T> ListableSubstateDatabase for CommittableOverlay<T>
where
    T: ListableSubstateDatabase,
{
    fn list_partition_keys(&self) -> Box<dyn Iterator<Item = DbPartitionKey> + '_> {
        let overlay_iterator = self.overlay.list_partition_keys();
        let db_iterator = self.db.list_partition_keys();

        Box::new(overlay_iterator.chain(db_iterator).unique())
    }
}
