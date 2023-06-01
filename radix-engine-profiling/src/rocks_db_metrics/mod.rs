use radix_engine_interface::prelude::*;
use radix_engine_store_interface::interface::{
    CommittableSubstateDatabase, DatabaseUpdate, DatabaseUpdates, DbPartitionKey, DbSortKey,
    DbSubstateValue, PartitionEntry, SubstateDatabase,
};
use radix_engine_stores::{
    memory_db::InMemorySubstateDatabase,
    rocks_db::RocksdbSubstateStore,
    rocks_db_with_merkle_tree::{BlockBasedOptions, Options, RocksDBWithMerkleTreeSubstateStore},
};
use std::{cell::RefCell, collections::BTreeMap, path::PathBuf, time::Duration};

#[cfg(test)]
mod tests;


/// Substate store with read time measurements for RocksDB and In Memory DB.
pub struct SubstateStoreWithMetrics<S>
where
    S: SubstateDatabase + CommittableSubstateDatabase,
{
    db: S,
    pub commit_metrics: RefCell<BTreeMap<usize, Vec<Duration>>>,
    pub read_metrics: RefCell<BTreeMap<usize, Vec<Duration>>>,
    pub read_not_found_metrics: RefCell<Vec<Duration>>,
}

impl SubstateStoreWithMetrics<RocksdbSubstateStore> {
    pub fn new_rocksdb(path: PathBuf) -> Self {
        let mut factory_opts = BlockBasedOptions::default();
        factory_opts.disable_cache();

        let mut opt = Options::default();
        opt.set_disable_auto_compactions(true);
        opt.create_if_missing(true);
        opt.create_missing_column_families(true);
        opt.set_block_based_table_factory(&factory_opts);

        Self {
            db: RocksdbSubstateStore::with_options(&opt, path),
            commit_metrics: RefCell::new(BTreeMap::new()),
            read_metrics: RefCell::new(BTreeMap::new()),
            read_not_found_metrics: RefCell::new(Vec::new()),
        }
    }
}

impl SubstateStoreWithMetrics<RocksDBWithMerkleTreeSubstateStore> {
    pub fn new_rocksdb_with_merkle_tree(path: PathBuf) -> Self {
        let mut factory_opts = BlockBasedOptions::default();
        factory_opts.disable_cache();

        let mut opt = Options::default();
        opt.set_disable_auto_compactions(true);
        opt.create_if_missing(true);
        opt.create_missing_column_families(true);
        opt.set_block_based_table_factory(&factory_opts);

        Self {
            db: RocksDBWithMerkleTreeSubstateStore::with_options(&opt, path),
            commit_metrics: RefCell::new(BTreeMap::new()),
            read_metrics: RefCell::new(BTreeMap::new()),
            read_not_found_metrics: RefCell::new(Vec::new()),
        }
    }
}

impl SubstateStoreWithMetrics<InMemorySubstateDatabase> {
    pub fn new_inmem() -> Self {
        Self {
            db: InMemorySubstateDatabase::standard(),
            commit_metrics: RefCell::new(BTreeMap::new()),
            read_metrics: RefCell::new(BTreeMap::new()),
            read_not_found_metrics: RefCell::new(Vec::new()),
        }
    }
}

impl<S: SubstateDatabase + CommittableSubstateDatabase> SubstateDatabase
    for SubstateStoreWithMetrics<S>
{
    fn get_substate(
        &self,
        partition_key: &DbPartitionKey,
        sort_key: &DbSortKey,
    ) -> Option<DbSubstateValue> {
        let start = std::time::Instant::now();
        let ret = self.db.get_substate(partition_key, sort_key);
        let duration = start.elapsed();

        if let Some(value) = ret {
            let exists = self.read_metrics.borrow().get(&value.len()).is_some();
            if exists {
                self.read_metrics
                    .borrow_mut()
                    .get_mut(&value.len())
                    .unwrap()
                    .push(duration);
            } else {
                self.read_metrics
                    .borrow_mut()
                    .insert(value.len(), vec![duration]);
            }
            Some(value)
        } else {
            self.read_not_found_metrics.borrow_mut().push(duration);
            None
        }
    }

    fn list_entries(
        &self,
        partition_key: &DbPartitionKey,
    ) -> Box<dyn Iterator<Item = PartitionEntry> + '_> {
        self.db.list_entries(partition_key)
    }
}

impl<S: SubstateDatabase + CommittableSubstateDatabase> CommittableSubstateDatabase
    for SubstateStoreWithMetrics<S>
{
    fn commit(&mut self, database_updates: &DatabaseUpdates) {
        let start = std::time::Instant::now();
        self.db.commit(database_updates);
        let duration = start.elapsed();

        assert!(!database_updates.is_empty());
        let partition_update = &database_updates[0];
        assert!(!partition_update.is_empty());
        let db_update = &partition_update[0];
        match db_update {
            DatabaseUpdate::Set(value) => {
                let exists = self.commit_metrics.borrow().get(&value.len()).is_some();
                if exists {
                    self.commit_metrics
                        .borrow_mut()
                        .get_mut(&value.len())
                        .unwrap()
                        .push(duration);
                } else {
                    self.commit_metrics
                        .borrow_mut()
                        .insert(value.len(), vec![duration]);
                }
            }
            DatabaseUpdate::Delete => (), // todo
        }
    }
}
