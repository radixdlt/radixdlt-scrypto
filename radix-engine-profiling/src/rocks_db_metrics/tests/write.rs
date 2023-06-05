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
    let axis_ranges = calculate_axis_ranges(&rocksdb_data, Some(100f32), Some(5000f32));
    export_graph_and_print_summary(
        "RocksDB random commits",
        &rocksdb_data,
        &rocksdb_output_data,
        "/tmp/scrypto_rocksdb_commit_1.png",
        "95th percentile of commits",
        &substate_db.commit_metrics,
        axis_ranges
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
        // 1_000_000 substates of size 100 bytes under random partitions
        prepare_db(&mut substate_db, 100, 100, 1, 1000000, false); 
    }

    // reopen database and measure commit times
    let mut substate_db = SubstateStoreWithMetrics::new_rocksdb_with_merkle_tree(path.clone());

    println!("Commit test execution");
    let mut rng = rand::thread_rng();

    let mut node_id_value = [0u8; NodeId::UUID_LENGTH];
    rng.fill(&mut node_id_value);
    let node_id = NodeId::new(EntityType::InternalKeyValueStore as u8, &node_id_value);

    let mut rocksdb_data_intermediate: BTreeMap<usize, Vec<Duration>> = BTreeMap::new();

    let rounds_count = 1000;
    for round in 0..rounds_count {
        print!("\rRound {}/{}", round, rounds_count );
        std::io::stdout().flush().ok();

        let value_size = 100;
        for i in 1..=100 {
            let mut input_data = DatabaseUpdates::new();

            for j in 0..i {
                let mut value_data: DbSubstateValue = vec![0u8; value_size];
                rng.fill(value_data.as_mut_slice());
                let value = DatabaseUpdate::Set(value_data);

                let substate_key_value: Vec<u8> = vec![j + 1]; //[0u8; NodeId::LENGTH];
                let sort_key = SpreadPrefixKeyMapper::to_db_sort_key(&SubstateKey::Map(
                    substate_key_value.into(),
                ));

                let mut partition = PartitionUpdates::new();
                partition.insert(sort_key.clone(), value);

                let partition_key =
                    SpreadPrefixKeyMapper::to_db_partition_key(&node_id, PartitionNumber(i as u8));

                input_data.insert(partition_key, partition);
            }

            substate_db.commit(&input_data);
        }

        // prepare intermediate data
        for (_k, v) in substate_db.commit_metrics.borrow().iter() {
            for (i, val) in v.iter().enumerate() {
                let exists = rocksdb_data_intermediate.get(&(i + 1)).is_some();
                if exists {
                    rocksdb_data_intermediate
                        .get_mut(&(i + 1))
                        .unwrap()
                        .push(*val);
                } else {
                    rocksdb_data_intermediate
                        .insert(i + 1, vec![*val]);
                }
            }
        }

        substate_db.commit_metrics.borrow_mut().clear();
    }
 
    println!("");
    // prepare output data
    //drop_highest_and_lowest_value(&substate_db, 3);
    let rocksdb_data_output =
         calculate_percent_to_max_points(&mut rocksdb_data_intermediate, 95f32);
    // prepare data for plot
    let mut rocksdb_data = Vec::with_capacity(100000);
    for (k, v) in rocksdb_data_intermediate {
        for val in v {
            rocksdb_data.push((k as f32, val.as_micros() as f32));
        }
    }

    // prepare data for plot
    // let mut rocksdb_data = Vec::with_capacity(100000);
    // for (_k, v) in substate_db.commit_metrics.borrow().iter() {
    //     for (i, val) in v.iter().enumerate() {
    //         rocksdb_data.push(((i+1) as f32, val.as_micros() as f32));
    //     }
    // }

    // export results
    // export_one_graph(
    //     "RocksDB (with Merkle tree) random commits",
    //     &rocksdb_data,
    //     "/tmp/scrypto_rocksdb_merkle_commit_1.png",
    //     &substate_db.commit_metrics,
    //     Some(100f32)
    // )
    // .unwrap();

    let mut axis_ranges = calculate_axis_ranges(&rocksdb_data, None, None);
    axis_ranges.3 = 200f32;
    export_graph_and_print_summary(
        "RocksDB (with Merkle tree) random commits",
        &rocksdb_data,
        &rocksdb_data_output,
        "/tmp/scrypto_rocksdb_merkle_commit_1.png",
        "95th percentile of commits",
        &substate_db.commit_metrics,
        axis_ranges
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
        calculate_percent_to_max_points(&mut substate_db.commit_metrics.borrow_mut(), 95f32);

    // prepare data for plot
    let mut rocksdb_data = Vec::with_capacity(100000);
    for (k, v) in substate_db.commit_metrics.borrow().iter() {
        for i in v {
            rocksdb_data.push((*k as f32, i.as_micros() as f32));
        }
    }

    (rocksdb_data, rocksdb_output_data)
}