use radix_engine::system::bootstrap::Bootstrapper;
use radix_engine::transaction::execute_and_commit_transaction;
use radix_engine::transaction::{ExecutionConfig, FeeReserveConfig};
use radix_engine::types::*;
use radix_engine::vm::wasm::WasmInstrumenter;
use radix_engine::vm::wasm::{DefaultWasmEngine, WasmMeteringConfig};
use radix_engine::vm::ScryptoVm;
use radix_engine_constants::DEFAULT_COST_UNIT_LIMIT;
use radix_engine_utils::rocks_db_metrics::*;
use scrypto_unit::*;
use std::path::PathBuf;
use transaction::builder::ManifestBuilder;
use transaction::model::TestTransaction;

#[cfg(feature = "rocksdb")]
#[test]
fn db_read_test() {
    // Set up environment.
    let mut scrypto_interpreter = ScryptoVm {
        wasm_engine: DefaultWasmEngine::default(),
        wasm_instrumenter: WasmInstrumenter::default(),
        wasm_metering_config: WasmMeteringConfig::V0,
    };

    let path = PathBuf::from(r"/tmp/radix-scrypto-db");
    // clean database
    std::fs::remove_dir_all(path.clone()).ok();

    let mut substate_db = RocksdbSubstateStoreWithMetrics::new(path);
    Bootstrapper::new(&mut substate_db, &scrypto_interpreter, false)
        .bootstrap_test_default()
        .unwrap();

    // compile and publish scrypto blueprint
    let (code, schema) = Compile::compile("./tests/blueprints/kv_store");

    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET, 100.into())
        .publish_package_advanced(
            code,
            schema,
            BTreeMap::new(),
            BTreeMap::new(),
            AccessRulesConfig::new(),
        )
        .build();
    let package_address = execute_and_commit_transaction(
        &mut substate_db,
        &mut scrypto_interpreter,
        &FeeReserveConfig::default(),
        &ExecutionConfig::default(),
        &TestTransaction::new(manifest.clone(), 1, DEFAULT_COST_UNIT_LIMIT)
            .get_executable(BTreeSet::new()),
    )
    .expect_commit(true)
    .new_package_addresses()[0];

    let max_count = 10u32;
    let cost_unit_limit = u32::MAX;

    // run scrypto blueprint - create component
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET, 1000.into())
        .call_function(package_address, "DatabaseBench", "new", manifest_args!())
        .build();
    let component = execute_and_commit_transaction(
        &mut substate_db,
        &mut scrypto_interpreter,
        &FeeReserveConfig::default(),
        &ExecutionConfig::default(),
        &TestTransaction::new(manifest.clone(), 2, cost_unit_limit)
            .get_executable(BTreeSet::new()),
    )
    .expect_commit(true)
    .new_component_addresses()[0];

    // fill KV-store with data
    let lengths = vec![ 10u32, 100u32, 500u32, 1000u32, 2000u32, 3000u32, 4000u32, 5000u32, 6000u32, 7000u32, 
        8000u32, 9000u32, 10000u32, 20000u32, 30000u32, 50000u32, 60000u32, 70000u32, 80000u32, 90000u32, 
        100000u32, 150000u32, 200000u32, 250000u32, 300000u32, 350000u32, 400000u32, 450000u32, 500000u32, 
        550000u32, 600000u32, 650000u32, 700000u32 ];
    for i in 0..lengths.len() {
        let manifest = ManifestBuilder::new()
            .lock_fee(FAUCET, 1000.into())
            .call_method(
                component,
                "insert",
                manifest_args!(lengths[i]),
            )
            .build();
        execute_and_commit_transaction(
            &mut substate_db,
            &mut scrypto_interpreter,
            &FeeReserveConfig::default(),
            &ExecutionConfig::default(),
            &TestTransaction::new(manifest.clone(), (i + 3) as u64, cost_unit_limit)
                .get_executable(BTreeSet::new()),
        )
        .expect_commit(true);
    }

    // read KV-store values
    for _ in 0..100 {
        for i in 0..lengths.len() {
            let manifest = ManifestBuilder::new()
                .lock_fee(FAUCET, 1000.into())
                .call_method(component, "read", manifest_args!(lengths[i]))
                .build();
            execute_and_commit_transaction(
                &mut substate_db,
                &mut scrypto_interpreter,
                &FeeReserveConfig::default(),
                &ExecutionConfig::default(),
                &TestTransaction::new(manifest.clone(), (max_count * 2 + i as 
                    u32 + 3) as u64, cost_unit_limit)
                    .get_executable(BTreeSet::new()),
            );
        }
    }

    // export results
    substate_db.export_graph_and_print_summary("/tmp/scrypto-rocksdb-reads-result.png").unwrap();
}
