use super::super::*;
use super::common::*;
use radix_engine_store_interface::{
    db_key_mapper::*,
    interface::{CommittableSubstateDatabase, DbPartitionKey, DbSortKey, SubstateDatabase},
};
use rand::Rng;
use std::{io::Write, path::PathBuf};

/// Range start of the measuremnts
const MIN_SIZE: usize = 1;
/// Range end of the measuremnts
const MAX_SIZE: usize = 4 * 1024 * 1024;
/// Range step
const SIZE_STEP: usize = 100 * 1024;
/// Each step write and read
const COUNT: usize = 20;
/// Multiplication of each step read (COUNT * READ_REPEATS)
const READ_REPEATS: usize = 100;

#[test]
/// Database is created in /tmp/radix-scrypto-db folder.
/// Outputs are genered in png files: /tmp/scrypto_rocksdb_1.png, /tmp/scrypto_inmem_1.png, /tmp/scrypto_diff_1.png
/// point list is printed to stdout.
/// To run test casea use command:
/// cargo test -p radix-engine-profilings -p radix-engine-stores --features rocksdb test_store_db --release -- --nocapture
/// from main radixdlt-scrypto folder.
fn test_read() {
    let read_repeats = match std::env::var("READ_REPEATS") {
        Ok(v) => usize::from_str(&v).unwrap(),
        _ => READ_REPEATS,
    };
    let min_size = match std::env::var("MIN_SIZE") {
        Ok(v) => usize::from_str(&v).unwrap(),
        _ => MIN_SIZE,
    };
    let max_size = match std::env::var("MAX_SIZE") {
        Ok(v) => usize::from_str(&v).unwrap(),
        _ => MAX_SIZE,
    };
    let size_step = match std::env::var("SIZE_STEP") {
        Ok(v) => usize::from_str(&v).unwrap(),
        _ => SIZE_STEP,
    };
    let prepare_db_write_count = match std::env::var("COUNT") {
        Ok(v) => usize::from_str(&v).unwrap(),
        _ => COUNT,
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
            size_step,
            prepare_db_write_count,
            false,
        )
    };

    // reopen database
    let mut substate_db = SubstateStoreWithMetrics::new_rocksdb_with_merkle_tree(path);
    // and run read test
    run_read_test(&mut substate_db, &data_index_vector, read_repeats);
    // run read not found test
    run_read_not_found_test(&mut substate_db, read_repeats, prepare_db_write_count);

    // prepare data for linear approximation
    discard_spikes(&mut substate_db.read_metrics.borrow_mut(), 200f32);
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
    )
    .unwrap();

    print_read_not_found_results(&substate_db);

    // InMemory DB part
    let mut substate_db = SubstateStoreWithMetrics::new_inmem();
    let data_index_vector = prepare_db(
        &mut substate_db,
        min_size,
        max_size,
        size_step,
        prepare_db_write_count,
        false,
    );
    run_read_test(&mut substate_db, &data_index_vector, read_repeats);
    // run read not found test
    run_read_not_found_test(&mut substate_db, read_repeats, prepare_db_write_count);

    // prepare data for linear approximation
    discard_spikes(&mut substate_db.read_metrics.borrow_mut(), 200f32);
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

    print_read_not_found_results(&substate_db);
}

fn run_read_test<S: SubstateDatabase + CommittableSubstateDatabase>(
    substate_db: &mut S,
    data_index_vector: &Vec<(DbPartitionKey, DbSortKey, usize)>,
    read_repeats: usize,
) {
    println!("Random read start...");

    let mut rng = rand::thread_rng();

    for i in 0..read_repeats {
        let time_start = std::time::Instant::now();
        let mut idx_vector: Vec<usize> = (0..data_index_vector.len()).collect();

        for j in 0..data_index_vector.len() {
            assert!(!idx_vector.is_empty());
            let idx = rng.gen_range(0..idx_vector.len());

            let (p, s, v) = &data_index_vector[idx_vector[idx]];

            print!("\rRead {}/{}", j + 1, data_index_vector.len());
            std::io::stdout().flush().ok();

            let read_value = substate_db.get_substate(&p, &s);

            assert!(read_value.is_some());
            assert_eq!(read_value.unwrap().len(), *v);

            idx_vector.remove(idx);
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
    count: usize,
) {
    println!("Read not found test start...");

    let mut data_index_vector_2: Vec<(DbPartitionKey, DbSortKey)> = Vec::new();
    let mut rng = rand::thread_rng();

    for _ in 0..count {
        let mut node_id_value = [0u8; NodeId::UUID_LENGTH];
        rng.fill(&mut node_id_value);
        let node_id = NodeId::new(EntityType::InternalKeyValueStore as u8, &node_id_value);
        let partition_key =
            SpreadPrefixKeyMapper::to_db_partition_key(&node_id, PartitionNumber(0u8));

        let mut substate_key_value = [0u8; NodeId::LENGTH];
        rng.fill(&mut substate_key_value);
        let sort_key =
            SpreadPrefixKeyMapper::to_db_sort_key(&SubstateKey::Map(substate_key_value.into()));

        data_index_vector_2.push((partition_key.clone(), sort_key));
    }

    for _ in 0..read_repeats {
        for (p, s) in data_index_vector_2.iter() {
            let read_value = substate_db.get_substate(&p, &s);
            assert!(read_value.is_none());
        }
    }

    println!("Read not found done");
}
