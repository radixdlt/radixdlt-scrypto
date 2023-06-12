#![allow(unused_imports)]

use super::super::*;
use super::common::*;
use super::*;
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
use std::{cmp::Ordering, io::Write, path::PathBuf};

#[test]
// to run this test use following command in the main repository folder:
// cargo nextest run -p radix-engine-profiling -p radix-engine-stores --no-capture --features rocksdb --release test_delete_per_size
fn test_delete_per_size() {
    const ROUNDS_COUNT: usize = 50;
    const MIN_SIZE: usize = 1;
    const MAX_SIZE: usize = 4 * 1024 * 1024;
    const SIZE_STEP: usize = 100 * 1024;
    const PREPARE_DB_WRITE_REPEATS: usize = ROUNDS_COUNT * 2;

    println!("No JMT part");
    let (rocksdb_data, rocksdb_data_output, rocksdb_data_original) = test_delete_per_size_internal(
        ROUNDS_COUNT,
        MIN_SIZE,
        MAX_SIZE,
        SIZE_STEP,
        PREPARE_DB_WRITE_REPEATS,
        |path| SubstateStoreWithMetrics::new_rocksdb(path),
    );

    let (lin_slope, lin_intercept): (f32, f32) =
        linear_regression_of(&rocksdb_data_output).unwrap();

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

    println!("JMT part");
    let (jmt_rocksdb_data, jmt_rocksdb_data_output, jmt_rocksdb_data_original) =
        test_delete_per_size_internal(
            ROUNDS_COUNT,
            MIN_SIZE,
            MAX_SIZE,
            SIZE_STEP,
            PREPARE_DB_WRITE_REPEATS,
            |path| SubstateStoreWithMetrics::new_rocksdb_with_merkle_tree(path),
        );

    let (jmt_lin_slope, jmt_lin_intercept): (f32, f32) =
        linear_regression_of(&jmt_rocksdb_data_output).unwrap();

    let axis_ranges = calculate_axis_ranges(&jmt_rocksdb_data, None, None);
    export_graph_and_print_summary(
        &format!(
            "RocksDB per size deletion with JMT, rounds: {}",
            ROUNDS_COUNT
        ),
        &jmt_rocksdb_data,
        &jmt_rocksdb_data_output,
        "/tmp/scrypto_rocksdb_per_size_deletion_JMT.png",
        "95th percentile of deletion",
        &jmt_rocksdb_data_original,
        axis_ranges,
        Some("Size [bytes]"),
    )
    .unwrap();

    export_graph_two_series(
        &format!(
            "95th percentile of deletion per size, rounds: {}",
            ROUNDS_COUNT
        ),
        &rocksdb_data_output,
        &jmt_rocksdb_data_output,
        "/tmp/scrypto_rocksdb_per_size_deletion_diff.png",
        "Size [bytes]",
        "Duration [µs]",
        "Series 1: deletion",
        "Series 2: deletion with JMT",
        (lin_slope, lin_intercept),
        (jmt_lin_slope, jmt_lin_intercept),
    )
    .unwrap();
}

#[test]
fn test_delete_per_partition() {
    const N: usize = 100;
    const ROUNDS_COUNT: usize = 50;

    println!("No JMT part");
    let (rocksdb_data, rocksdb_data_output, rocksdb_data_original) =
        test_delete_per_partition_internal(N, ROUNDS_COUNT, |path| {
            SubstateStoreWithMetrics::new_rocksdb(path)
        });

    let (lin_slope, lin_intercept): (f32, f32) =
        linear_regression_of(&rocksdb_data_output).unwrap();

    let axis_ranges = calculate_axis_ranges(&rocksdb_data, None, None);
    export_graph_and_print_summary(
        &format!(
            "RocksDB per partition deletion (N=1..{}) rounds: {}",
            N, ROUNDS_COUNT
        ),
        &rocksdb_data,
        &rocksdb_data_output,
        "/tmp/scrypto_rocksdb_per_partition_deletion.png",
        "95th percentile of deletion",
        &rocksdb_data_original,
        axis_ranges,
        Some("N"),
    )
    .unwrap();

    println!("JMT part");
    let (jmt_rocksdb_data, jmt_rocksdb_data_output, jmt_rocksdb_data_original) =
        test_delete_per_partition_internal(N, ROUNDS_COUNT, |path| {
            SubstateStoreWithMetrics::new_rocksdb_with_merkle_tree(path)
        });

    let (jmt_lin_slope, jmt_lin_intercept): (f32, f32) =
        linear_regression_of(&jmt_rocksdb_data_output).unwrap();

    let axis_ranges = calculate_axis_ranges(&jmt_rocksdb_data, None, None);
    export_graph_and_print_summary(
        &format!(
            "RocksDB per partition deletion with JMT (N=1..{}) rounds: {}",
            N, ROUNDS_COUNT
        ),
        &jmt_rocksdb_data,
        &jmt_rocksdb_data_output,
        "/tmp/scrypto_rocksdb_per_partition_deletion_JMT.png",
        "95th percentile of deletion",
        &jmt_rocksdb_data_original,
        axis_ranges,
        Some("N"),
    )
    .unwrap();

    export_graph_two_series(
        &format!(
            "95th percentile of deletion per partition, rounds: {}",
            ROUNDS_COUNT
        ),
        &rocksdb_data_output,
        &jmt_rocksdb_data_output,
        "/tmp/scrypto_rocksdb_per_partition_deletion_diff.png",
        "N",
        "Duration [µs]",
        "Series 1: deletion",
        "Series 2: deletion with JMT",
        (lin_slope, lin_intercept),
        (jmt_lin_slope, jmt_lin_intercept),
    )
    .unwrap();
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
        prepare_db(
            &mut substate_db,
            min_size,
            max_size,
            size_step,
            prepare_db_write_repeats,
            false,
        )
    };

    // reopen database and measure deletion times
    let mut substate_db = create_store(path.clone());

    println!("Delete test execution");
    let mut rng = rand::thread_rng();

    // repeat 1 substate commit n-times
    for i in 0..rounds_count {
        print!("Round {}/{}\r", i + 1, rounds_count);
        std::io::stdout().flush().ok();

        // prepare vector with indices of data to draw from
        let mut size_vector: Vec<usize> = Vec::new();
        for j in (i..data.len()).step_by(prepare_db_write_repeats) {
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

fn test_delete_per_partition_internal<F, S>(
    n_value: usize,
    rounds_count: usize,
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

    let mut rng = rand::thread_rng();

    // prepare database
    let data: Vec<(DbPartitionKey, Vec<DbSortKey>)> = {
        let mut substate_db = create_store(path.clone());
        // 1_000_000 substates of size 100 bytes under random partitions
        prepare_db(&mut substate_db, 100, 100, 1, 1000000, false);

        let value_size = 100;

        let mut data_index_vector: Vec<(DbPartitionKey, Vec<DbSortKey>)> =
            Vec::with_capacity(n_value * n_value);

        for _ in 0..rounds_count {
            let mut input_data = DatabaseUpdates::new();
            for i in 1..=n_value {
                let mut node_id_value = [0u8; NodeId::UUID_LENGTH];
                rng.fill(&mut node_id_value);
                let node_id = NodeId::new(EntityType::InternalKeyValueStore as u8, &node_id_value);

                let partition_key =
                    SpreadPrefixKeyMapper::to_db_partition_key(&node_id, PartitionNumber(0u8));
                let mut partition = PartitionUpdates::new();

                let mut sort_key_data = Vec::new();
                for _ in 0..i {
                    let mut value_data: DbSubstateValue = vec![0u8; value_size];
                    rng.fill(value_data.as_mut_slice());
                    let value = DatabaseUpdate::Set(value_data);

                    let mut substate_key_value = [0u8; NodeId::LENGTH];
                    rng.fill(&mut substate_key_value);
                    let sort_key = SpreadPrefixKeyMapper::to_db_sort_key(&SubstateKey::Map(
                        substate_key_value.into(),
                    ));

                    partition.insert(sort_key.clone(), value);

                    sort_key_data.push(sort_key);
                }
                data_index_vector.push((partition_key.clone(), sort_key_data));
                input_data.insert(partition_key, partition);
            }
            substate_db.commit(&input_data);
        }

        data_index_vector
    };

    // reopen database and measure commit times
    let mut substate_db = create_store(path.clone());

    println!("Delete test execution");

    let mut node_id_value = [0u8; NodeId::UUID_LENGTH];
    rng.fill(&mut node_id_value);

    let mut rocksdb_data_intermediate: BTreeMap<usize, Vec<Duration>> = BTreeMap::new();

    for (idx, item) in data.iter().enumerate() {
        if (idx + 1) % n_value == 0 {
            print!("\rRound {}/{}", idx / n_value + 1, rounds_count);
            std::io::stdout().flush().ok();
        }

        let mut input_data = DatabaseUpdates::new();

        let mut partition = PartitionUpdates::new();
        for j in &item.1 {
            partition.insert(j.clone(), DatabaseUpdate::Delete);
        }

        input_data.insert(item.0.clone(), partition);

        substate_db.commit(&input_data);

        if (idx + 1) % n_value == 0 {
            // prepare intermediate data
            for (_k, v) in substate_db.commit_delete_metrics.borrow().iter() {
                for (i, val) in v.iter().enumerate() {
                    let exists = rocksdb_data_intermediate.get(&(i + 1)).is_some();
                    if exists {
                        rocksdb_data_intermediate
                            .get_mut(&(i + 1))
                            .unwrap()
                            .push(*val);
                    } else {
                        rocksdb_data_intermediate.insert(i + 1, vec![*val]);
                    }
                }
            }

            substate_db.commit_delete_metrics.borrow_mut().clear();
        }
    }

    println!("");
    // prepare output data
    discard_spikes(&mut rocksdb_data_intermediate, 200f32);
    let rocksdb_data_output =
        calculate_percent_to_max_points(&mut rocksdb_data_intermediate, 95f32);
    // prepare data for plot
    let mut rocksdb_data = Vec::with_capacity(100000);
    for (k, v) in &rocksdb_data_intermediate {
        for val in v {
            rocksdb_data.push((*k as f32, val.as_micros() as f32));
        }
    }

    (rocksdb_data, rocksdb_data_output, rocksdb_data_intermediate)
}
