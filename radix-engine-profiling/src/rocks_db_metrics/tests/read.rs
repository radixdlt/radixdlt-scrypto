use super::super::*;
use super::common::*;
use radix_substate_store_interface::{
    db_key_mapper::*,
    interface::{CommittableSubstateDatabase, DbPartitionKey, DbSortKey, SubstateDatabase},
};
use rand::{seq::SliceRandom, Rng};
use std::{io::Write, path::PathBuf};

/// Substate size range start
const MIN_SIZE: usize = 1;
/// Substate size range end
const MAX_SIZE: usize = 1024 * 1024;
/// Number of different substate size in range [MIN_SIZE-MAX_SIZE]
const SIZE_COUNT: usize = 64;
/// Number of nodes written to the database in preparation step.
/// Each node has SIZE_COUNT substates of size between MIN_SIZE and MAX_SIZE in one partition.
const WRITE_NODES_COUNT: usize = 4000;
/// Number of repated reads of each node previously written to the database.
const READ_NODES_REPEAT_COUNT: usize = 100;
/// Number of substates to read in 'read not found' test
const READ_NOT_FOUND_SUBSTATES_COUNT: usize = 100;
/// Max size of read substates to use
const FILTER_READ_MAX_SIZE: usize = MAX_SIZE;
/// Filter to use to discard spikes (only value in range median +/- filter is used)
const FILTER_SPIKES: f32 = 500f32;

#[test]
/// Measuring read of node substates of size in range [MIN_SIZE-MAX_SIZE] and measuring substates not found in database.
/// Database is created in /tmp/radix-scrypto-db folder.
/// Outputs are generated in png files: /tmp/scrypto_read_rocksdb.png, /tmp/scrypto_read_inmem.png, /tmp/scrypto_read_diff.png
/// point list is printed to stdout.
/// To run the test case use command:
///  cargo test -p radix-engine-profiling -p radix-substate-store-impls --features rocksdb test_read --release -- --nocapture
/// or
///  cargo nextest run -p radix-engine-profiling -p radix-substate-store-impls --no-capture --features rocksdb --release test_read
/// from main radixdlt-scrypto folder.
/// Test can be parametrized using environment variables: READ_NODES_REPEAT_COUNT, MIN_SIZE, MAX_SIZE, SIZE_COUNT,
///  WRITE_NODES_COUNT, FILTER_READ_MAX_SIZE, FILTER_SPIKES, READ_NOT_FOUND_SUBSTATES_COUNT
fn test_read() {
    let read_repeats = match std::env::var("READ_NODES_REPEAT_COUNT") {
        Ok(v) => usize::from_str(&v).unwrap(),
        _ => READ_NODES_REPEAT_COUNT,
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
    let read_max_size = match std::env::var("FILTER_READ_MAX_SIZE") {
        Ok(v) => usize::from_str(&v).unwrap(),
        _ => FILTER_READ_MAX_SIZE,
    };
    let filter = match std::env::var("FILTER_SPIKES") {
        Ok(v) => f32::from_str(&v).unwrap(),
        _ => FILTER_SPIKES,
    };
    let read_not_fund_count = match std::env::var("READ_NOT_FOUND_SUBSTATES_COUNT") {
        Ok(v) => usize::from_str(&v).unwrap(),
        _ => READ_NOT_FOUND_SUBSTATES_COUNT,
    };

    // RocksDB part
    let path = PathBuf::from(r"/tmp/radix-scrypto-db");
    // clean database
    std::fs::remove_dir_all(path.clone()).ok();

    // prepare database
    let data_index_vector = {
        let mut substate_db = SubstateStoreWithMetrics::new_rocksdb_with_merkle_tree(path.clone());
        prepare_db(
            &mut substate_db,
            min_size,
            max_size,
            value_size_count,
            write_nodes_count,
        )
    };

    let mut data_index_vector = data_index_vector
        .into_iter()
        .filter(|x| x.2 <= read_max_size)
        .collect();

    // reopen database
    let mut substate_db = SubstateStoreWithMetrics::new_rocksdb_with_merkle_tree(path);

    // and run read test
    run_read_test(&mut substate_db, &mut data_index_vector, read_repeats);
    // run read not found test
    run_read_not_found_test(&mut substate_db, read_repeats, read_not_fund_count);

    // prepare data for linear approximation
    discard_spikes(&mut substate_db.read_metrics.borrow_mut(), filter);
    let rocksdb_output_data =
        calculate_percent_to_max_points(&mut substate_db.read_metrics.borrow_mut(), 95f32);

    // prepare data for plot
    let mut rocksdb_data = Vec::with_capacity(100000);
    for (k, v) in substate_db.read_metrics.borrow().iter() {
        for i in v {
            rocksdb_data.push((*k as f32, i.as_micros() as f32));
        }
    }

    // export results
    let axis_ranges = calculate_axis_ranges(&rocksdb_data, None, None);
    export_graph_and_print_summary(
        "RocksDB random reads",
        &rocksdb_data,
        &rocksdb_output_data,
        "/tmp/scrypto_read_rocksdb.png",
        "95th percentile of reads",
        &substate_db.read_metrics.borrow(),
        axis_ranges,
        None,
        true,
    )
    .unwrap();

    export_graph_and_print_summary_read_not_found_results(
        &substate_db,
        "/tmp/scrypto_not_found_read_rocksdb.png",
        "RocksDB read not existing substates",
    )
    .unwrap();

    // InMemory DB part
    let mut substate_db = SubstateStoreWithMetrics::new_inmem();
    let data_index_vector = prepare_db(
        &mut substate_db,
        min_size,
        max_size,
        value_size_count,
        write_nodes_count,
    );

    let mut data_index_vector = data_index_vector
        .into_iter()
        .filter(|x| x.2 <= read_max_size)
        .collect();

    run_read_test(&mut substate_db, &mut data_index_vector, read_repeats);
    // run read not found test
    run_read_not_found_test(&mut substate_db, read_repeats, read_not_fund_count);

    // prepare data for linear approximation
    discard_spikes(&mut substate_db.read_metrics.borrow_mut(), filter);
    let inmem_output_data =
        calculate_percent_to_max_points(&mut substate_db.read_metrics.borrow_mut(), 95f32);

    // prepare data for plot
    let mut inmem_data = Vec::with_capacity(100000);
    for (k, v) in substate_db.read_metrics.borrow().iter() {
        for i in v {
            inmem_data.push((*k as f32, i.as_micros() as f32));
        }
    }

    // export results
    let axis_ranges = calculate_axis_ranges(&inmem_data, None, None);
    export_graph_and_print_summary(
        "InMemoryDB random reads",
        &inmem_data,
        &inmem_output_data,
        "/tmp/scrypto_read_inmem.png",
        "95th percentile of reads",
        &substate_db.read_metrics.borrow(),
        axis_ranges,
        None,
        true,
    )
    .unwrap();

    // Calculate RocksDB - InMemory diff and export results
    export_graph_and_print_summary_for_two_series(
        "RocksDB - InMemoryDB random reads",
        &rocksdb_output_data,
        &inmem_output_data,
        "/tmp/scrypto_read_diff.png",
    )
    .unwrap();

    export_graph_and_print_summary_read_not_found_results(
        &substate_db,
        "/tmp/scrypto_not_found_read_inmem.png",
        "InMemoryDB read not existing substates",
    )
    .unwrap();
}

fn run_read_test<S: SubstateDatabase + CommittableSubstateDatabase>(
    substate_db: &mut S,
    data_index_vector: &mut Vec<(DbPartitionKey, DbSortKey, usize)>,
    read_repeats: usize,
) {
    println!("Random read start...");
    assert!(!data_index_vector.is_empty());

    let mut rng = rand::thread_rng();

    for i in 0..read_repeats {
        let time_start = std::time::Instant::now();

        data_index_vector.shuffle(&mut rng);

        for (j, (p, s, v)) in data_index_vector.iter().enumerate() {
            print!("\rRead {}/{}  ", j + 1, data_index_vector.len());
            std::io::stdout().flush().ok();

            let read_value = substate_db.get_raw_substate_by_db_key(&p, &s);

            assert!(read_value.is_some());
            assert_eq!(read_value.unwrap().len(), *v);
        }

        let time_end = std::time::Instant::now();
        let mut duration = time_end
            .checked_duration_since(time_start)
            .unwrap()
            .as_secs();
        if duration == 0 {
            duration = 1;
        }
        println!(
            "\rRound {}/{}  read time: {} s, left: {} s\r",
            i + 1,
            read_repeats,
            duration,
            (read_repeats - (i + 1)) * duration as usize
        );
    }

    println!("Read done");
}

fn run_read_not_found_test<S: SubstateDatabase + CommittableSubstateDatabase>(
    substate_db: &mut S,
    read_repeats: usize,
    prepare_count: usize,
) {
    println!("Read not found test start...");

    let mut data_index_vector: Vec<(DbPartitionKey, DbSortKey)> = Vec::with_capacity(prepare_count);
    let mut rng = rand::thread_rng();

    // prepare list of partition_keys/sort_keys to qeury database
    for _ in 0..prepare_count {
        let mut node_id_value = [0u8; NodeId::RID_LENGTH];
        rng.fill(&mut node_id_value);
        let node_id = NodeId::new(EntityType::InternalKeyValueStore as u8, &node_id_value);
        let partition_key =
            SpreadPrefixKeyMapper::to_db_partition_key(&node_id, PartitionNumber(0u8));

        let mut substate_key_value = [0u8; SUBSTATE_KEY_LENGTH];
        rng.fill(&mut substate_key_value);
        let sort_key =
            SpreadPrefixKeyMapper::to_db_sort_key(&SubstateKey::Map(substate_key_value.into()));

        data_index_vector.push((partition_key.clone(), sort_key));
    }

    for _ in 0..read_repeats {
        data_index_vector.shuffle(&mut rng);

        for (p, s) in data_index_vector.iter() {
            let read_value = substate_db.get_raw_substate_by_db_key(&p, &s);
            assert!(read_value.is_none());
        }
    }

    println!("Read not found done");
}
