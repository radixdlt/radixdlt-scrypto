// To run tests use command:
// cargo nextest run -p radix-engine-profiling -p radix-engine-stores --no-capture --features rocksdb --release test_commit

use super::common::*;
use super::super::*;
use super::*;
use std::{io::Write, path::PathBuf};
use rand::Rng;


const COMMIT_REPEATS: usize = 50;


#[test]
fn test_commit() {
    // RocksDB part
    let path = PathBuf::from(r"/tmp/radix-scrypto-db");
    // clean database
    std::fs::remove_dir_all(path.clone()).ok();

    // prepare database
    {
        let mut substate_db = SubstateStoreWithMetrics::new_rocksdb(path.clone());
        prepare_db(&mut substate_db, MIN_SIZE, MAX_SIZE, SIZE_STEP, COUNT);
    }

    // reopen database and measure commit times
    let mut substate_db = SubstateStoreWithMetrics::new_rocksdb(path.clone());

    let (rocksdb_data, rocksdb_output_data) = commit_test_internal(&mut substate_db);

    // export results
    export_graph_and_print_summary(
        "RocksDB random commits",
        &rocksdb_data,
        &rocksdb_output_data,
        "/tmp/scrypto_rocksdb_commit_1.png",
        "95th percentile of commits",
        &substate_db.commit_metrics
    )
    .unwrap();
}


#[test]
fn test_commit_merkle() {
    // RocksDB part
    let path = PathBuf::from(r"/tmp/radix-scrypto-db");
    // clean database
    std::fs::remove_dir_all(path.clone()).ok();

    // prepare database
    {
        let mut substate_db = SubstateStoreWithMetrics::new_rocksdb_with_merkle_tree(path.clone());
        prepare_db(&mut substate_db, MIN_SIZE, MAX_SIZE, SIZE_STEP, COUNT);
    }

    // reopen database and measure commit times
    let mut substate_db = SubstateStoreWithMetrics::new_rocksdb_with_merkle_tree(path.clone());

    let (rocksdb_data, rocksdb_output_data) = commit_test_internal(&mut substate_db);

    // export results
    export_graph_and_print_summary(
        "RocksDB (with Merkle tree) random commits",
        &rocksdb_data,
        &rocksdb_output_data,
        "/tmp/scrypto_rocksdb_merkle_commit_1.png",
        "95th percentile of commits",
        &substate_db.commit_metrics
    )
    .unwrap();
}

fn commit_test_internal<S: SubstateDatabase + CommittableSubstateDatabase>(
    substate_db: &mut SubstateStoreWithMetrics<S>) -> (Vec<(f32, f32)>, Vec<(f32, f32)>) {
    let mut rng = rand::thread_rng();

    // prepare vector with substate sizes
    let mut size_vector: Vec<usize> = Vec::new();
    for size in (MIN_SIZE..=MAX_SIZE).step_by(SIZE_STEP) {
        size_vector.push(size);
    }

    // repeat 1 substate commit n-times
    for i in 0..COMMIT_REPEATS {
        print!("Round {}/{}\r", i, COMMIT_REPEATS);
        std::io::stdout().flush().ok();

        let mut idx_vector = size_vector.clone();

        for _ in 0..size_vector.len() {
            assert!(!idx_vector.is_empty());
            // randomize substate size
            let idx = rng.gen_range(0..idx_vector.len());

            let mut input_data = DatabaseUpdates::new();

            let (partition_key, _sort_key, partition) = generate_commit_data(&mut rng, idx_vector[idx]);

            input_data.insert(partition_key, partition);

            substate_db.commit(&input_data);

            idx_vector.remove(idx);
        }
    }

    drop_highest_and_lowest_value(&substate_db, 3);
    let rocksdb_output_data =
        calculate_percent_to_max_points(&substate_db.commit_metrics, 95f32);

    // prepare data for plot
    let mut rocksdb_data = Vec::with_capacity(100000);
    for (k, v) in substate_db.commit_metrics.borrow().iter() {
        for i in v {
            rocksdb_data.push((*k as f32, i.as_micros() as f32));
        }
    }

    (rocksdb_data, rocksdb_output_data)
}