use radix_common::prelude::*;
use radix_substate_store_impls::{memory_db::*, rocks_db::*, rocks_db_with_merkle_tree::*};
use radix_substate_store_interface::interface::*;
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
            db: RocksDBWithMerkleTreeSubstateStore::with_options(&opt, path, true),
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
    fn get_raw_substate_by_db_key(
        &self,
        partition_key: &DbPartitionKey,
        sort_key: &DbSortKey,
    ) -> Option<DbSubstateValue> {
        let start = std::time::Instant::now();
        let ret = self.db.get_raw_substate_by_db_key(partition_key, sort_key);
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

    fn list_raw_values_from_db_key(
        &self,
        partition_key: &DbPartitionKey,
        from_sort_key: Option<&DbSortKey>,
    ) -> Box<dyn Iterator<Item = PartitionEntry> + '_> {
        self.db
            .list_raw_values_from_db_key(partition_key, from_sort_key)
    }
}

impl<S: SubstateDatabase + CommittableSubstateDatabase> CommittableSubstateDatabase
    for SubstateStoreWithMetrics<S>
{
    fn commit(&mut self, database_updates: &DatabaseUpdates) {
        // Validate if commit call with database_updates parameter fulfills test framework requirements
        let updated_partitions_count: usize = database_updates
            .node_updates
            .values()
            .map(|node_updates| node_updates.partition_updates.len())
            .sum();
        assert!(updated_partitions_count > 0);
        let mut set_found = false;
        let mut delete_found = false;
        let multiple_updates = updated_partitions_count > 1;
        let mut old_value_len: Option<usize> = None;
        let mut delete_value_len: usize = 0;
        for (node_key, node_updates) in &database_updates.node_updates {
            for (partition_num, partition_updates) in &node_updates.partition_updates {
                let PartitionDatabaseUpdates::Delta { substate_updates } = partition_updates else {
                    panic!("Deletes not supported while profiling")
                };
                for db_update in substate_updates {
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
                                if let Some(value) = self.get_raw_substate_by_db_key(
                                    &DbPartitionKey {
                                        node_key: node_key.clone(),
                                        partition_num: *partition_num,
                                    },
                                    &db_update.0,
                                ) {
                                    delete_value_len = value.len();
                                }
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
        let partition_updates = &database_updates.node_updates[0].partition_updates[0];
        let PartitionDatabaseUpdates::Delta { substate_updates } = partition_updates else {
            panic!("Deletes not supported while profiling")
        };
        let db_update = &substate_updates[0];
        match db_update {
            DatabaseUpdate::Set(value) => {
                self.commit_set_metrics
                    .borrow_mut()
                    .entry(value.len())
                    .or_default()
                    .push(duration / updated_partitions_count as u32);
            }
            DatabaseUpdate::Delete => {
                self.commit_delete_metrics
                    .borrow_mut()
                    .entry(delete_value_len)
                    .or_default()
                    .push(duration / updated_partitions_count as u32);
            }
        }
    }
}
