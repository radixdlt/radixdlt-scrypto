// To run tests use command:
// cargo nextest run -p radix-engine-profiling -p radix-engine-stores --no-capture --features rocksdb --release test_commit

use super::common::*;
use super::super::*;
use super::*;
use radix_engine_store_interface::{
    db_key_mapper::*,
    interface::{
        CommittableSubstateDatabase, DatabaseUpdate, DatabaseUpdates, DbPartitionKey,
        DbSortKey, PartitionUpdates, SubstateDatabase,
    },
};
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
        prepare_db(&mut substate_db, MIN_SIZE, MAX_SIZE, SIZE_STEP, COUNT, false);
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
        // 10000 substates of random size from 1B to 4MB under random partitions
        prepare_db(&mut substate_db, MIN_SIZE, MAX_SIZE, 0, 10000, true); 
    }

    // reopen database and measure commit times
    let mut substate_db = SubstateStoreWithMetrics::new_rocksdb_with_merkle_tree(path.clone());

    println!("Commit test execution");
    let mut rng = rand::thread_rng();

    let mut node_id_value = [0u8; NodeId::UUID_LENGTH];
    rng.fill(&mut node_id_value);
    let node_id = NodeId::new(EntityType::InternalKeyValueStore as u8, &node_id_value);

    let mut substate_key_value = [0u8; NodeId::LENGTH];
    rng.fill(&mut substate_key_value);
    let sort_key = SpreadPrefixKeyMapper::to_db_sort_key(&SubstateKey::Map(
        substate_key_value.into(),
    ));

    let value_size_max = 100;

    for value_size in 1..=value_size_max {
        print!("\rRound {}/{}", value_size, value_size_max );
        std::io::stdout().flush().ok();

        let mut input_data = DatabaseUpdates::new();

        let mut value_data: DbSubstateValue = vec![0u8; value_size];
        rng.fill(value_data.as_mut_slice());
        let value = DatabaseUpdate::Set(value_data);

        let mut partition = PartitionUpdates::new();
        partition.insert(sort_key.clone(), value);

        let partition_key =
            SpreadPrefixKeyMapper::to_db_partition_key(&node_id, PartitionNumber(value_size as u8));

        input_data.insert(partition_key, partition);

        substate_db.commit(&input_data);
    }
    println!("");
    // prepare output data
    // drop_highest_and_lowest_value(&substate_db, 3);
    // let rocksdb_output_data =
    //     calculate_percent_to_max_points(&substate_db.commit_metrics, 95f32);

    // prepare data for plot
    let mut rocksdb_data = Vec::with_capacity(100000);
    for (k, v) in substate_db.commit_metrics.borrow().iter() {
        for i in v {
            rocksdb_data.push((*k as f32, i.as_micros() as f32));
        }
    }
    // export results
    export_one_graph(
        "RocksDB (with Merkle tree) random commits",
        &rocksdb_data,
        "/tmp/scrypto_rocksdb_merkle_commit_1.png",
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