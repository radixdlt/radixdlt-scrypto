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
    pub commit_set_metrics: RefCell<BTreeMap<usize, Vec<Duration>>>,
    pub commit_delete_metrics: RefCell<BTreeMap<usize, Vec<Duration>>>,
    pub read_metrics: RefCell<BTreeMap<usize, Vec<Duration>>>,
    pub read_not_found_metrics: RefCell<Vec<Duration>>,
}

impl SubstateStoreWithMetrics<RocksdbSubstateStore> {
    pub fn new_rocksdb(path: PathBuf) -> Self {
        let mut factory_opts = BlockBasedOptions::default();
        factory_opts.disable_cache();

        let mut opt = Options::default();
        opt.create_if_missing(true);
        opt.create_missing_column_families(true);
        opt.set_block_based_table_factory(&factory_opts);

        Self {
            db: RocksdbSubstateStore::with_options(&opt, path),
            commit_set_metrics: RefCell::new(BTreeMap::new()),
            commit_delete_metrics: RefCell::new(BTreeMap::new()),
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
        opt.create_if_missing(true);
        opt.create_missing_column_families(true);
        opt.set_block_based_table_factory(&factory_opts);

        Self {
            db: RocksDBWithMerkleTreeSubstateStore::with_options(&opt, path),
            commit_set_metrics: RefCell::new(BTreeMap::new()),
            commit_delete_metrics: RefCell::new(BTreeMap::new()),
            read_metrics: RefCell::new(BTreeMap::new()),
            read_not_found_metrics: RefCell::new(Vec::new()),
        }
    }
}

impl SubstateStoreWithMetrics<InMemorySubstateDatabase> {
    pub fn new_inmem() -> Self {
        Self {
            db: InMemorySubstateDatabase::standard(),
            commit_set_metrics: RefCell::new(BTreeMap::new()),
            commit_delete_metrics: RefCell::new(BTreeMap::new()),
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
            self.read_metrics
                .borrow_mut()
                .entry(value.len())
                .or_default()
                .push(duration);
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
        // Validate if commit call with database_updates parameter fulfills test framework requirements
        assert!(!database_updates.is_empty());
        let mut set_found = false;
        let mut delete_found = false;
        let multiple_updates = database_updates.len() > 1;
        let mut old_value_len: Option<usize> = None;
        let mut delete_value_len: usize = 0;
        for partition_update in database_updates {
            for db_update in partition_update.1 {
                match db_update.1 {
                    DatabaseUpdate::Set(value) => {
                        if delete_found {
                            panic!(
                                "Mixed DatabaseUpdate (Set & Delete) not supported while profiling"
                            )
                        } else {
                            set_found = true;
                            if multiple_updates {
                                if old_value_len.is_some() {
                                    if old_value_len.unwrap() != value.len() {
                                        panic!("For multiple DatabaseUpdate value size must be the same");
                                    }
                                } else {
                                    old_value_len = Some(value.len());
                                }
                            }
                        }
                    }
                    DatabaseUpdate::Delete => {
                        if set_found {
                            panic!(
                                "Mixed DatabaseUpdate (Set & Delete) not supported while profiling"
                            )
                        } else {
                            delete_found = true;
                            if let Some(value) =
                                self.get_substate(&partition_update.0, &db_update.0)
                            {
                                delete_value_len = value.len();
                            }
                        }
                    }
                }
            }
        }

        // call commit on database and measure execution time
        let start = std::time::Instant::now();
        self.db.commit(database_updates);
        let duration = start.elapsed();

        // Commit profiling tests are divided to two types:
        // - per size - test invokes only database_update per commit (that is why we can use 1st item only here)
        // - per partition - test invokes commits for particular partition size, so value length is not important here (still it is safe to use 1st item only)
        let partition_update = &database_updates[0];
        let db_update = &partition_update[0];
        match db_update {
            DatabaseUpdate::Set(value) => {
                self.commit_set_metrics
                    .borrow_mut()
                    .entry(value.len())
                    .or_default()
                    .push(duration / database_updates.len() as u32);
            }
            DatabaseUpdate::Delete => {
                self.commit_delete_metrics
                    .borrow_mut()
                    .entry(delete_value_len)
                    .or_default()
                    .push(duration / database_updates.len() as u32);
            }
        }
    }
}
