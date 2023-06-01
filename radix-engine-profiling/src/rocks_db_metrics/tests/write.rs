use super::common::*;
use super::super::*;
use super::*;
#[allow(unused_imports)]
use std::{io::Write, path::PathBuf};



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
    // repeat commits of 1 substate writes
    let commit_repeats = 50;
    for i in 0..commit_repeats {
        print!("Round {}/{}   ", i, commit_repeats);
        prepare_db(&mut substate_db, MIN_SIZE, MAX_SIZE, SIZE_STEP, 1);
    }

    drop_highest_and_lowest_value(&mut substate_db, 3);
    let rocksdb_output_data =
        calculate_percent_to_max_points(&substate_db.commit_metrics, 95f32);

    // prepare data for plot
    let mut rocksdb_data = Vec::with_capacity(100000);
    for (k, v) in substate_db.commit_metrics.borrow().iter() {
        for i in v {
            rocksdb_data.push((*k as f32, i.as_micros() as f32));
        }
    }

    // export results
    export_graph_and_print_summary(
        &mut substate_db,
        "RocksDB random commits",
        &rocksdb_data,
        &rocksdb_output_data,
        "/tmp/scrypto_rocksdb_commit_1.png",
        "95th percentile of commits",
    )
    .unwrap();
}

