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



#[test]
fn db_read_test() {
    println!("starting");
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
    .expect_commit(true).new_package_addresses()[0];

    // run scrypto blueprint
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET, 10.into())
        .call_function(package_address, "Basic", "multiple_reads", manifest_args!())
        .build();

    for i in 0..10000 {
        execute_and_commit_transaction(
            &mut substate_db,
            &mut scrypto_interpreter,
            &FeeReserveConfig::default(),
            &ExecutionConfig::default(),
            &TestTransaction::new(manifest.clone(), i + 1, DEFAULT_COST_UNIT_LIMIT)
                .get_executable(BTreeSet::new()),
        )
        .expect_commit(true);
    }

    substate_db.export_mft().unwrap();
}
