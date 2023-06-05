use super::common::*;
use super::super::*;
use super::*;
use radix_engine_store_interface::{
    db_key_mapper::*,
    interface::{
        CommittableSubstateDatabase,DbPartitionKey,
        DbSortKey, SubstateDatabase,
    },
};
use rand::Rng;
use std::{io::Write, path::PathBuf};


#[test]
/// Database is created in /tmp/radix-scrypto-db folder.
/// Outputs are genered in png files: /tmp/scrypto_rocksdb_1.png, /tmp/scrypto_inmem_1.png, /tmp/scrypto_diff_1.png
/// point list is printed to stdout.
/// To run test casea use command:
/// cargo test -p radix-engine-profilings -p radix-engine-stores --features rocksdb test_store_db --release -- --nocapture
/// from main radixdlt-scrypto folder.
fn test_read() {
    // RocksDB part
    let path = PathBuf::from(r"/tmp/radix-scrypto-db");
    // clean database
    std::fs::remove_dir_all(path.clone()).ok();

    // prepare database
    let data_index_vector = {
        let mut substate_db = SubstateStoreWithMetrics::new_rocksdb(path.clone());
        prepare_db(&mut substate_db, MIN_SIZE, MAX_SIZE, SIZE_STEP, COUNT, false)
    };

    // reopen database
    let mut substate_db = SubstateStoreWithMetrics::new_rocksdb(path);
    // and run read test
    run_read_test(&mut substate_db, &data_index_vector);
    // run read not found test
    run_read_not_found_test(&mut substate_db);

    // prepare data for linear approximation
    drop_highest_and_lowest_value(&mut substate_db, 3);
    let rocksdb_output_data = calculate_percent_to_max_points(&mut substate_db.read_metrics.borrow_mut(), 95f32);

    // prepare data for plot
    let mut rocksdb_data = Vec::with_capacity(100000);
    for (k, v) in substate_db.read_metrics.borrow().iter() {
        for i in v {
            rocksdb_data.push((*k as f32, i.as_micros() as f32));
        }
    }

    // export results
    let axis_ranges = calculate_axis_ranges(&rocksdb_data, Some(100f32), Some(5000f32));
    export_graph_and_print_summary(
        "RocksDB random reads",
        &rocksdb_data,
        &rocksdb_output_data,
        "/tmp/scrypto_rocksdb_1.png",
        "95th percentile of reads",
        &substate_db.read_metrics.borrow(),
        axis_ranges,
        None,
    )
    .unwrap();

    print_read_not_found_results(&substate_db);

    // InMemory DB part
    let mut substate_db = SubstateStoreWithMetrics::new_inmem();
    let data_index_vector = prepare_db(&mut substate_db, MIN_SIZE, MAX_SIZE, SIZE_STEP, COUNT, false);
    run_read_test(&mut substate_db, &data_index_vector);
    // run read not found test
    run_read_not_found_test(&mut substate_db);

    // prepare data for linear approximation
    drop_highest_and_lowest_value(&mut substate_db, 3);
    let inmem_output_data = calculate_percent_to_max_points(&mut substate_db.read_metrics.borrow_mut(), 95f32);

    // prepare data for plot
    let mut inmem_data = Vec::with_capacity(100000);
    for (k, v) in substate_db.read_metrics.borrow().iter() {
        for i in v {
            inmem_data.push((*k as f32, i.as_micros() as f32));
        }
    }

    // export results
    let axis_ranges = calculate_axis_ranges(&rocksdb_data, Some(100f32), Some(5000f32));
    export_graph_and_print_summary(
        "InMemoryDB random reads",
        &inmem_data,
        &inmem_output_data,
        "/tmp/scrypto_inmem_1.png",
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
        "/tmp/scrypto_diff_1.png"
    )
    .unwrap();

    print_read_not_found_results(&substate_db);
}

fn run_read_test<S: SubstateDatabase + CommittableSubstateDatabase>(
    substate_db: &mut S,
    data_index_vector: &Vec<(DbPartitionKey, DbSortKey, usize)>,
) {
    println!("Random read start...");

    let mut rng = rand::thread_rng();

    //let mut p_key_cnt = 1u32;
    for i in 0..READ_REPEATS {
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
            READ_REPEATS,
            duration,
            (READ_REPEATS - (i + 1)) * duration as usize
        );
    }

    println!("Read done");
}

fn run_read_not_found_test<S: SubstateDatabase + CommittableSubstateDatabase>(
    substate_db: &mut S,
) {
    println!("Read not found test start...");

    let mut data_index_vector_2: Vec<(DbPartitionKey, DbSortKey)> = Vec::new();
    let mut rng = rand::thread_rng();

    for _ in 0..COUNT {
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

    for _ in 0..READ_REPEATS {
        for (p, s) in data_index_vector_2.iter() {
            let read_value = substate_db.get_substate(&p, &s);
            assert!(read_value.is_none());
        }
    }

    println!("Read done");
}

