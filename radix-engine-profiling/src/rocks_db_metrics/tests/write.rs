use super::super::*;
use super::common::*;
use linreg::linear_regression_of;
use radix_substate_store_interface::db_key_mapper::*;
use rand::{seq::SliceRandom, Rng};
use std::{io::Write, path::PathBuf};

/// Number of nodes writes during test execution - measured.
const WRITE_NODES_REPEAT_COUNT: usize = 4000;
/// Substate size range start
const MIN_SIZE: usize = 1;
/// Substate size range end
const MAX_SIZE: usize = 1024 * 1024;
/// Number of different substate size in range [MIN_SIZE-MAX_SIZE]
const SIZE_COUNT: usize = 64;
/// Number of nodes written to the database in preparation step.
/// Each node has SIZE_COUNT substates of size between MIN_SIZE and MAX_SIZE in one partition.
const WRITE_NODES_COUNT: usize = 4000;

#[test]
/// Measuring the writing nodes with SIZE_COUNT substates from range [MIN_SIZE-MAX_SIZE] each.
/// Database is created in /tmp/radix-scrypto-db folder.
/// Outputs are generated in png files: /tmp/scrypto_commit_per_size_rocksdb.png, /tmp/scrypto_commit_per_size_rocksdb_JMT.png, /tmp/scrypto_commit_per_size_rocksdb_diff.png
/// point list is printed to stdout.
/// To run the test case use command:
///  cargo test -p radix-engine-profiling -p radix-substate-store-impls --features rocksdb test_commit_per_size --release -- --nocapture
/// or
///  cargo nextest run -p radix-engine-profiling -p radix-substate-store-impls --no-capture --features rocksdb --release test_commit_per_size
/// from main radixdlt-scrypto folder.
/// Test can be parametrized using environment variables: READ_NODES_REPEAT_COUNT, MIN_SIZE, MAX_SIZE, SIZE_STEP, WRITE_NODES_COUNT
fn test_commit_per_size() {
    let write_nodes_repeat_count = match std::env::var("WRITE_NODES_REPEAT_COUNT") {
        Ok(v) => usize::from_str(&v).unwrap(),
        _ => WRITE_NODES_REPEAT_COUNT,
    };
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
    let (rocksdb_data, rocksdb_data_output, rocksdb_data_original) = test_commit_per_size_internal(
        write_nodes_repeat_count,
        min_size,
        max_size,
        value_size_count,
        write_nodes_count,
        |path| SubstateStoreWithMetrics::new_rocksdb(path),
    );

    let (lin_slope, lin_intercept): (f32, f32) =
        linear_regression_of(&rocksdb_data_output).unwrap();

    let mut axis_ranges = calculate_axis_ranges(&rocksdb_data, None, None);
    axis_ranges.3 = (lin_slope * max_size as f32 + lin_intercept) * 1.2f32;
    export_graph_and_print_summary(
        &format!(
            "RocksDB per size commits, rounds: {}",
            write_nodes_repeat_count
        ),
        &rocksdb_data,
        &rocksdb_data_output,
        "/tmp/scrypto_commit_per_size_rocksdb.png",
        "95th percentile of commits",
        &rocksdb_data_original,
        axis_ranges,
        Some("Size [bytes]"),
        true,
    )
    .unwrap();

    println!("JMT part");
    let (jmt_rocksdb_data, jmt_rocksdb_data_output, jmt_rocksdb_data_original) =
        test_commit_per_size_internal(
            write_nodes_repeat_count,
            min_size,
            max_size,
            value_size_count,
            write_nodes_count,
            |path| SubstateStoreWithMetrics::new_rocksdb_with_merkle_tree(path),
        );

    let (jmt_lin_slope, jmt_lin_intercept): (f32, f32) =
        linear_regression_of(&jmt_rocksdb_data_output).unwrap();

    let mut axis_ranges = calculate_axis_ranges(&jmt_rocksdb_data, None, None);
    axis_ranges.3 = (jmt_lin_slope * max_size as f32 + jmt_lin_intercept) * 1.2f32;
    export_graph_and_print_summary(
        &format!(
            "RocksDB with Merkle tree per size commits, rounds: {}",
            write_nodes_repeat_count
        ),
        &jmt_rocksdb_data,
        &jmt_rocksdb_data_output,
        "/tmp/scrypto_commit_per_size_rocksdb_JMT.png",
        "95th percentile of commits",
        &jmt_rocksdb_data_original,
        axis_ranges,
        Some("Size [bytes]"),
        true,
    )
    .unwrap();

    export_graph_two_series(
        &format!(
            "95th percentile of commits per size, rounds: {}",
            write_nodes_repeat_count
        ),
        &rocksdb_data_output,
        &jmt_rocksdb_data_output,
        "/tmp/scrypto_commit_per_size_rocksdb_diff.png",
        "Size [bytes]",
        "Duration [µs]",
        "Series 1: commits",
        "Series 2: commits with JMT",
        (lin_slope, lin_intercept),
        (jmt_lin_slope, jmt_lin_intercept),
    )
    .unwrap();
}

#[test]
/// Measuring the writing partition of variable sizes [0-100].
/// Database is created in /tmp/radix-scrypto-db folder.
/// Outputs are generated in png files: /tmp/scrypto_commit_per_partition_rocksdb.png, /tmp/scrypto_commit_per_partition_rocksdb_JMT.png, /tmp/scrypto_commit_per_partition_rocksdb_diff.png
/// point list is printed to stdout.
/// To run the test case use command:
///  cargo test -p radix-engine-profiling -p radix-substate-store-impls --features rocksdb test_commit_per_partition --release -- --nocapture
/// or
///  cargo nextest run -p radix-engine-profiling -p radix-substate-store-impls --no-capture --features rocksdb --release test_commit_per_partition
/// from main radixdlt-scrypto folder.
fn test_commit_per_partition() {
    const N: usize = 100;
    const ROUNDS_COUNT: usize = 100;

    println!("No JMT part");
    let (rocksdb_data, rocksdb_data_output, rocksdb_data_original) =
        test_commit_per_partition_internal(N, ROUNDS_COUNT, |path| {
            SubstateStoreWithMetrics::new_rocksdb(path)
        });

    let (lin_slope, lin_intercept): (f32, f32) =
        linear_regression_of(&rocksdb_data_output).unwrap();

    let mut axis_ranges = calculate_axis_ranges(&rocksdb_data, None, None);
    axis_ranges.3 = rocksdb_data_output.last().unwrap().1 * 1.2f32;
    export_graph_and_print_summary(
        &format!(
            "RocksDB per partition commits (N=1..{}) rounds: {}",
            N, ROUNDS_COUNT
        ),
        &rocksdb_data,
        &rocksdb_data_output,
        "/tmp/scrypto_commit_per_partition_rocksdb.png",
        "95th percentile of commits",
        &rocksdb_data_original,
        axis_ranges,
        Some("N"),
        false,
    )
    .unwrap();

    println!("JMT part");
    let (jmt_rocksdb_data, jmt_rocksdb_data_output, jmt_rocksdb_data_original) =
        test_commit_per_partition_internal(N, ROUNDS_COUNT, |path| {
            SubstateStoreWithMetrics::new_rocksdb_with_merkle_tree(path)
        });

    let (jmt_lin_slope, jmt_lin_intercept): (f32, f32) =
        linear_regression_of(&jmt_rocksdb_data_output).unwrap();

    let mut axis_ranges = calculate_axis_ranges(&jmt_rocksdb_data, None, None);
    axis_ranges.3 = jmt_rocksdb_data_output.last().unwrap().1 * 1.2f32;
    export_graph_and_print_summary(
        &format!(
            "RocksDB with Merkle tree per partition commits (N=1..{}) rounds: {}",
            N, ROUNDS_COUNT
        ),
        &jmt_rocksdb_data,
        &jmt_rocksdb_data_output,
        "/tmp/scrypto_commit_per_partition_rocksdb_JMT.png",
        "95th percentile of commits",
        &jmt_rocksdb_data_original,
        axis_ranges,
        Some("N"),
        false,
    )
    .unwrap();

    export_graph_two_series(
        &format!(
            "95th percentile of commits per pertition (N=1..{}) rounds: {}",
            N, ROUNDS_COUNT
        ),
        &rocksdb_data_output,
        &jmt_rocksdb_data_output,
        "/tmp/scrypto_commit_per_partition_rocksdb_diff.png",
        "N",
        "Duration [µs]",
        "Series 1: commits",
        "Series 2: commits with JMT",
        (lin_slope, lin_intercept),
        (jmt_lin_slope, jmt_lin_intercept),
    )
    .unwrap();
}

fn test_commit_per_partition_internal<F, S>(
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

    // prepare database
    {
        let mut substate_db = create_store(path.clone());
        // Generate 1_000_000 substates of size 100 bytes, one substate per node.
        prepare_db(&mut substate_db, 100, 100, 1, 1000000);
    }

    // reopen database and measure commit times
    let mut substate_db = create_store(path.clone());

    println!("Commit test execution");
    let mut rng = rand::thread_rng();

    let mut node_id_value = [0u8; NodeId::RID_LENGTH];
    rng.fill(&mut node_id_value);
    let node_id = NodeId::new(EntityType::InternalKeyValueStore as u8, &node_id_value);

    let mut rocksdb_data_intermediate: BTreeMap<usize, Vec<Duration>> = BTreeMap::new();

    for round in 0..rounds_count {
        print!("\rRound {}/{}  ", round + 1, rounds_count);
        std::io::stdout().flush().ok();

        let value_size = 100;
        for n in 1..=n_value {
            let mut input_data = index_map_new();
            let mut partition = index_map_new();

            for j in 0..n {
                let mut value_data: DbSubstateValue = vec![0u8; value_size];
                rng.fill(value_data.as_mut_slice());
                let value = DatabaseUpdate::Set(value_data);
                let substate_key_value: Vec<u8> = (j + 1).to_be_bytes().to_vec();
                let sort_key = SpreadPrefixKeyMapper::to_db_sort_key(&SubstateKey::Map(
                    substate_key_value.into(),
                ));

                partition.insert(sort_key.clone(), value);
            }

            let partition_key =
                SpreadPrefixKeyMapper::to_db_partition_key(&node_id, PartitionNumber(n as u8));

            input_data.insert(partition_key, partition);

            substate_db.commit(&DatabaseUpdates::from_delta_maps(input_data));
        }

        // prepare intermediate data
        for (_k, v) in substate_db.commit_set_metrics.borrow().iter() {
            for (i, val) in v.iter().enumerate() {
                rocksdb_data_intermediate
                    .entry(i + 1)
                    .or_default()
                    .push(*val);
            }
        }

        substate_db.commit_set_metrics.borrow_mut().clear();
    }

    println!("");
    // prepare output data
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

fn test_commit_per_size_internal<F, S>(
    rounds_count: usize,
    min_size: usize,
    max_size: usize,
    value_size_count: usize,
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

    // prepare database with maximum size
    {
        let mut substate_db = create_store(path.clone());
        prepare_db(
            &mut substate_db,
            min_size,
            max_size,
            value_size_count,
            prepare_db_write_repeats,
        );
    }

    // reopen database and measure commit times
    let mut substate_db = create_store(path.clone());
    let mut rng = rand::thread_rng();

    // prepare vector with substate sizes
    let mut size_vector = generate_range(min_size, max_size, value_size_count);

    // repeat 1 commit of substate of various size into different nodes and partitions n-times
    for i in 0..rounds_count {
        print!("Round {}/{}  \r", i + 1, rounds_count);
        std::io::stdout().flush().ok();

        let mut node_id_value = [0u8; NodeId::RID_LENGTH];
        rng.fill(&mut node_id_value);
        let node_id = NodeId::new(EntityType::InternalKeyValueStore as u8, &node_id_value);
        let partition_key =
            SpreadPrefixKeyMapper::to_db_partition_key(&node_id, PartitionNumber(0u8));

        size_vector.shuffle(&mut rng);

        for substate_size in size_vector.iter() {
            let mut input_data = index_map_new();
            let mut partition = index_map_new();

            generate_commit_data(&mut partition, &mut rng, *substate_size);

            input_data.insert(partition_key.clone(), partition);

            substate_db.commit(&DatabaseUpdates::from_delta_maps(input_data));
        }
    }

    discard_spikes(&mut substate_db.commit_set_metrics.borrow_mut(), 5000f32);
    let rocksdb_output_data =
        calculate_percent_to_max_points(&mut substate_db.commit_set_metrics.borrow_mut(), 95f32);

    // prepare data for plot
    let mut rocksdb_data = Vec::with_capacity(100000);
    for (k, v) in substate_db.commit_set_metrics.borrow().iter() {
        for i in v {
            rocksdb_data.push((*k as f32, i.as_micros() as f32));
        }
    }
    let original_data = substate_db.commit_set_metrics.borrow().clone();

    (rocksdb_data, rocksdb_output_data, original_data)
}
