use super::super::*;
use super::common::*;
use linreg::linear_regression_of;
use radix_substate_store_interface::db_key_mapper::*;
use rand::{seq::SliceRandom, Rng};
use std::{io::Write, path::PathBuf};

/// Range start of the measuremnts
const MIN_SIZE: usize = 1;
/// Range end of the measuremnts
const MAX_SIZE: usize = 1024 * 1024;
/// Number of different substate size in range [MIN_SIZE-MAX_SIZE]
const SIZE_COUNT: usize = 64;
/// Number of nodes written to the database in preparation step.
/// Each node has SIZE_COUNT substates of size between MIN_SIZE and MAX_SIZE in one partition.
const WRITE_NODES_COUNT: usize = 4000;

#[test]
/// Measuring deletion of substates of size from range [MIN_SIZE-MAX_SIZE].
/// Database is created in /tmp/radix-scrypto-db folder.
/// Outputs are generated in png files: /tmp/scrypto_delete_per_size_rocksdb.png, /tmp/scrypto_delete_per_size_rocksdb_JMT.png, /tmp/scrypto_delete_per_size_rocksdb_diff.png
/// point list is printed to stdout.
/// To run the test case use command:
///  cargo test -p radix-engine-profiling -p radix-substate-store-impls --features rocksdb test_delete_per_size --release -- --nocapture
/// or
///  cargo nextest run -p radix-engine-profiling -p radix-substate-store-impls --no-capture --features rocksdb --release test_delete_per_size
/// from main radixdlt-scrypto folder.
/// Test can be parametrized using environment variables: WRITE_NODES_COUNT, MIN_SIZE, MAX_SIZE, SIZE_STEP
fn test_delete_per_size() {
    let min_size = match std::env::var("MIN_SIZE") {
        Ok(v) => usize::from_str(&v).unwrap(),
        _ => MIN_SIZE,
    };
    let max_size = match std::env::var("MAX_SIZE") {
        Ok(v) => usize::from_str(&v).unwrap(),
        _ => MAX_SIZE,
    };
    let value_size_count = match std::env::var("SIZE_COUNT") {
        Ok(v) => usize::from_str(&v).unwrap(),
        _ => SIZE_COUNT,
    };
    let write_nodes_count = match std::env::var("WRITE_NODES_COUNT") {
        Ok(v) => usize::from_str(&v).unwrap(),
        _ => WRITE_NODES_COUNT,
    };

    println!("No JMT part");
    let (rocksdb_data, rocksdb_data_output, rocksdb_data_original) = test_delete_per_size_internal(
        min_size,
        max_size,
        value_size_count,
        write_nodes_count,
        |path| SubstateStoreWithMetrics::new_rocksdb(path),
    );

    let (lin_slope, lin_intercept): (f32, f32) =
        linear_regression_of(&rocksdb_data_output).unwrap();

    let axis_ranges = calculate_axis_ranges(&rocksdb_data, None, None);
    export_graph_and_print_summary(
        "RocksDB per size deletion",
        &rocksdb_data,
        &rocksdb_data_output,
        "/tmp/scrypto_delete_per_size_rocksdb.png",
        "95th percentile of deletion",
        &rocksdb_data_original,
        axis_ranges,
        Some("Size [bytes]"),
        true,
    )
    .unwrap();

    println!("JMT part");
    let (jmt_rocksdb_data, jmt_rocksdb_data_output, jmt_rocksdb_data_original) =
        test_delete_per_size_internal(
            min_size,
            max_size,
            value_size_count,
            write_nodes_count,
            |path| SubstateStoreWithMetrics::new_rocksdb_with_merkle_tree(path),
        );

    let (jmt_lin_slope, jmt_lin_intercept): (f32, f32) =
        linear_regression_of(&jmt_rocksdb_data_output).unwrap();

    let axis_ranges = calculate_axis_ranges(&jmt_rocksdb_data, None, None);
    export_graph_and_print_summary(
        "RocksDB per size deletion with JMT",
        &jmt_rocksdb_data,
        &jmt_rocksdb_data_output,
        "/tmp/scrypto_delete_per_size_rocksdb_JMT.png",
        "95th percentile of deletion",
        &jmt_rocksdb_data_original,
        axis_ranges,
        Some("Size [bytes]"),
        true,
    )
    .unwrap();

    export_graph_two_series(
        "95th percentile of deletion per size",
        &rocksdb_data_output,
        &jmt_rocksdb_data_output,
        "/tmp/scrypto_delete_per_size_rocksdb_diff.png",
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
/// Measuring partition removal of variable sizes [0-100].
/// Database is created in /tmp/radix-scrypto-db folder.
/// Outputs are generated in png files: /tmp/scrypto_delete_per_partition_rocksdb.png, /tmp/scrypto_delete_per_partition_rocksdb_JMT.png, /tmp/scrypto_delete_per_partition_rocksdb_diff.png
/// point list is printed to stdout.
/// To run the test case use command:
///  cargo test -p radix-engine-profiling -p radix-substate-store-impls --features rocksdb test_delete_per_partition --release -- --nocapture
/// or
///  cargo nextest run -p radix-engine-profiling -p radix-substate-store-impls --no-capture --features rocksdb --release test_delete_per_partition
/// from main radixdlt-scrypto folder.
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
        "/tmp/scrypto_delete_per_partition_rocksdb.png",
        "95th percentile of deletion",
        &rocksdb_data_original,
        axis_ranges,
        Some("N"),
        false,
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
        "/tmp/scrypto_delete_per_partition_rocksdb_JMT.png",
        "95th percentile of deletion",
        &jmt_rocksdb_data_original,
        axis_ranges,
        Some("N"),
        false,
    )
    .unwrap();

    export_graph_two_series(
        &format!(
            "95th percentile of deletion per partition, rounds: {}",
            ROUNDS_COUNT
        ),
        &rocksdb_data_output,
        &jmt_rocksdb_data_output,
        "/tmp/scrypto_delete_per_partition_rocksdb_diff.png",
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

    // prepare database

    // Stage 1: fill db with substates to prepare base db size
    // this will prevent from decreasing db size to 0 when deleting substates
    {
        let mut substate_db = create_store(path.clone());

        prepare_db(
            &mut substate_db,
            min_size,
            max_size,
            size_step,
            prepare_db_write_repeats,
        );
    }

    // Stage 2: reopen database and fill db with additional substates which will be deleted in next step
    let mut data: Vec<(DbPartitionKey, DbSortKey, usize)> = {
        let mut substate_db = create_store(path.clone());

        prepare_db(
            &mut substate_db,
            min_size,
            max_size,
            size_step,
            prepare_db_write_repeats,
        )
    };

    // reopen database and measure deletion times
    let mut substate_db = create_store(path.clone());

    println!("Delete test execution");
    let mut rng = rand::thread_rng();

    data.shuffle(&mut rng);

    for (partition_key, sort_key, _usize) in data {
        let mut input_data = index_map_new();

        let mut partition = index_map_new();
        partition.insert(sort_key, DatabaseUpdate::Delete);

        input_data.insert(partition_key, partition);

        substate_db.commit(&DatabaseUpdates::from_delta_maps(input_data));
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
    let data_per_round: Vec<Vec<(DbPartitionKey, Vec<DbSortKey>)>> = {
        let mut substate_db = create_store(path.clone());
        // Fill db with 1_000_000 substates of size 100 bytes under random partitions, one substate per node.
        // this will prepare base db size
        prepare_db(&mut substate_db, 100, 100, 1, 1000000);

        // fill db with additional substates which will be deleted in next step
        let value_size = 100;

        let mut data_per_round: Vec<Vec<(DbPartitionKey, Vec<DbSortKey>)>> =
            Vec::with_capacity(rounds_count);

        for _ in 0..rounds_count {
            let mut data: Vec<(DbPartitionKey, Vec<DbSortKey>)> = Vec::new();

            let mut input_data = index_map_new();
            for i in 1..=n_value {
                let mut node_id_value = [0u8; NodeId::RID_LENGTH];
                rng.fill(&mut node_id_value);
                let node_id = NodeId::new(EntityType::InternalKeyValueStore as u8, &node_id_value);

                let partition_key =
                    SpreadPrefixKeyMapper::to_db_partition_key(&node_id, PartitionNumber(0u8));
                let mut partition = index_map_new();

                let mut sort_key_data = Vec::new();
                for _ in 0..i {
                    let mut value_data: DbSubstateValue = vec![0u8; value_size];
                    rng.fill(value_data.as_mut_slice());
                    let value = DatabaseUpdate::Set(value_data);

                    let mut substate_key_value = [0u8; SUBSTATE_KEY_LENGTH];
                    rng.fill(&mut substate_key_value);
                    let sort_key = SpreadPrefixKeyMapper::to_db_sort_key(&SubstateKey::Map(
                        substate_key_value.into(),
                    ));

                    partition.insert(sort_key.clone(), value);

                    sort_key_data.push(sort_key);
                }
                data.push((partition_key.clone(), sort_key_data));
                input_data.insert(partition_key, partition);
            }
            substate_db.commit(&DatabaseUpdates::from_delta_maps(input_data));

            data_per_round.push(data);
        }

        data_per_round
    };

    // reopen database and measure commit times
    let mut substate_db = create_store(path.clone());

    println!("Delete test execution");

    let mut node_id_value = [0u8; NodeId::RID_LENGTH];
    rng.fill(&mut node_id_value);

    let mut rocksdb_data_intermediate: BTreeMap<usize, Vec<Duration>> = BTreeMap::new();

    for (idx, mut round) in data_per_round.into_iter().enumerate() {
        print!("\rRound {}/{}", idx + 1, rounds_count);
        std::io::stdout().flush().ok();

        round.shuffle(&mut rng);

        let mut idx_vector_output: Vec<usize> = Vec::with_capacity(round.len());

        for (idx, (partition_key, sort_keys)) in round.iter().enumerate() {
            // store sequence of indices for intermediate data
            idx_vector_output.push(idx);

            let mut input_data = index_map_new();
            let mut partition = index_map_new();

            for key in sort_keys {
                partition.insert(key.clone(), DatabaseUpdate::Delete);
            }

            input_data.insert(partition_key.clone(), partition);

            substate_db.commit(&DatabaseUpdates::from_delta_maps(input_data));
        }

        // prepare intermediate data
        for (_k, v) in substate_db.commit_delete_metrics.borrow().iter() {
            assert_eq!(v.len(), idx_vector_output.len());
            for (i, val) in v.iter().enumerate() {
                rocksdb_data_intermediate
                    .entry(idx_vector_output[i] + 1)
                    .or_default()
                    .push(*val);
            }
        }
        // clear metrics between rounds
        substate_db.commit_delete_metrics.borrow_mut().clear();
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
