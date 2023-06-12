#![allow(unused_imports)]

use super::super::*;
use super::*;
use super::common::*;
use linreg::linear_regression_of;
use plotters::prelude::IntoLinspace;
use radix_engine_store_interface::{
    db_key_mapper::*,
    interface::{
        CommittableSubstateDatabase, DatabaseUpdate, DatabaseUpdates, PartitionUpdates,
        SubstateDatabase,
    },
};
use rand::Rng;
use std::{io::Write, path::PathBuf, cmp::Ordering};

#[test]
fn test_delete_per_size() {
    const ROUNDS_COUNT: usize = 20;
    const MIN_SIZE: usize = 1;
    const MAX_SIZE: usize = 4 * 1024 * 1024;
    const SIZE_STEP: usize = 100 * 1024;
    const PREPARE_DB_WRITE_REPEATS: usize = ROUNDS_COUNT * 2;

    println!("No JMT part");
    let (rocksdb_data, rocksdb_data_output, rocksdb_data_original) =
        test_delete_per_size_internal(ROUNDS_COUNT, MIN_SIZE, MAX_SIZE, SIZE_STEP, PREPARE_DB_WRITE_REPEATS, 
            |path| SubstateStoreWithMetrics::new_rocksdb(path) );

    let axis_ranges = calculate_axis_ranges(&rocksdb_data, None, None);
    export_graph_and_print_summary(
        &format!("RocksDB per size deletion, rounds: {}", ROUNDS_COUNT),
        &rocksdb_data,
        &rocksdb_data_output,
        "/tmp/scrypto_rocksdb_per_size_deletion.png",
        "95th percentile of deletion",
        &rocksdb_data_original,
        axis_ranges,
        Some("Size [bytes]"),
    )
    .unwrap();
}

#[test]
fn test_delete_per_partition() {
}



fn test_delete_per_size_internal<F, S>(
    rounds_count: usize,
    min_size: usize,
    max_size: usize,
    size_step: usize,
    prepare_db_write_repeats: usize,
    create_store: F,
) -> (
    Vec<(f32, f32)>,
    Vec<(f32, f32)>,
    BTreeMap<usize, Vec<Duration>>,
)
where
    F: Fn(PathBuf) -> SubstateStoreWithMetrics<S>,
    S: SubstateDatabase + CommittableSubstateDatabase,
{
    // RocksDB part
    let path = PathBuf::from(r"/tmp/radix-scrypto-db");
    // clean database
    std::fs::remove_dir_all(path.clone()).ok();

    // prepare database with maxium size
    let data: Vec<(DbPartitionKey, DbSortKey, usize)> = {
        let mut substate_db = create_store(path.clone());
        prepare_db(&mut substate_db, min_size, max_size, size_step, prepare_db_write_repeats, false)
    };

    // reopen database and measure deletion times
    let mut substate_db = create_store(path.clone());
    let mut rng = rand::thread_rng();


    // repeat 1 substate commit n-times
    for i in 0..rounds_count {
        print!("Round {}/{}\r", i + 1, rounds_count);
        std::io::stdout().flush().ok();

        // prepare vector with indices of data to draw from
        let mut size_vector: Vec<usize> = Vec::new();
        for j in (i..data.len()).step_by(prepare_db_write_repeats) {
            //println!(" idx: {}", j);
            size_vector.push(j);
        }

        let mut idx_vector = size_vector.clone();

        for _ in 0..size_vector.len() {
            assert!(!idx_vector.is_empty());
            // randomize substate size
            let idx = rng.gen_range(0..idx_vector.len());

            let mut input_data = DatabaseUpdates::new();

            let mut partition = PartitionUpdates::new();
            partition.insert(data[idx_vector[idx]].1.clone(), DatabaseUpdate::Delete);

            input_data.insert(data[idx_vector[idx]].0.clone(), partition);
            //println!(" -> del: {} {}", idx, idx_vector[idx]);

            substate_db.commit(&input_data);

            idx_vector.remove(idx);
        }
    }

    discard_spikes(&mut substate_db.commit_delete_metrics.borrow_mut(), 100f32);
    let rocksdb_output_data =
        calculate_percent_to_max_points(&mut substate_db.commit_delete_metrics.borrow_mut(), 95f32);

    // prepare data for plot
    let mut rocksdb_data = Vec::with_capacity(100000);
    for (k, v) in substate_db.commit_delete_metrics.borrow().iter() {
        for i in v {
            rocksdb_data.push((*k as f32, i.as_micros() as f32));
        }
    }
    let original_data = substate_db.commit_delete_metrics.borrow().clone();

    (rocksdb_data, rocksdb_output_data, original_data)
}


