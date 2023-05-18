use criterion::{criterion_group, criterion_main, Criterion};
use radix_engine::system::bootstrap::Bootstrapper;
use radix_engine::transaction::execute_and_commit_transaction;
use radix_engine::transaction::{ExecutionConfig, FeeReserveConfig};
use radix_engine::types::*;
use radix_engine::vm::wasm::WasmInstrumenter;
use radix_engine::vm::wasm::{DefaultWasmEngine, WasmMeteringConfig};
use radix_engine::vm::ScryptoVm;
use radix_engine_constants::DEFAULT_COST_UNIT_LIMIT;
use radix_engine_interface::dec;
use radix_engine_interface::rule;
use radix_engine_store_interface::interface::{SubstateDatabase, DbPartitionKey, DbSortKey, DbSubstateValue, PartitionEntry, CommittableSubstateDatabase, DatabaseUpdates};
use transaction::builder::ManifestBuilder;
use transaction::ecdsa_secp256k1::EcdsaSecp256k1PrivateKey;
use transaction::model::TestTransaction;
use std::path::PathBuf;
use std::time::Duration;
use radix_engine_stores::rocks_db::RocksdbSubstateStore;


struct RocksdbSubstateStoreWithMetrics {
    db: RocksdbSubstateStore,
    read_metrics: RefCell<HashMap<usize, Duration>>
}

impl RocksdbSubstateStoreWithMetrics {
    pub fn new(path: PathBuf) -> Self {
        Self {
            db: RocksdbSubstateStore::standard(path),
            read_metrics: RefCell::new(HashMap::with_capacity(1000))
        }
    }
}

impl SubstateDatabase for RocksdbSubstateStoreWithMetrics {
    fn get_substate(
        &self,
        partition_key: &DbPartitionKey,
        sort_key: &DbSortKey,
    ) -> Option<DbSubstateValue> {

        let start = std::time::Instant::now();
        let ret = self.db.get_substate(partition_key,sort_key);
        let duration = start.elapsed();

        if let Some(value) = ret {
            self.read_metrics.borrow_mut().insert(value.len(), duration);
            Some(value)
        } else {
            None
        }
    }

    fn list_entries(
        &self,
        partition_key: &DbPartitionKey,
    ) -> Box<dyn Iterator<Item = PartitionEntry> + '_> {
        self.db.list_entries(partition_key)
    }
}

impl CommittableSubstateDatabase for RocksdbSubstateStoreWithMetrics {
    fn commit(&mut self, database_updates: &DatabaseUpdates) {
        self.db.commit(database_updates)
    }
}


fn db_rw_test(c: &mut Criterion) {
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

    // Create a key pair
    let private_key = EcdsaSecp256k1PrivateKey::from_u64(1).unwrap();
    let public_key = private_key.public_key();

    // Create two accounts
    let accounts = (0..2)
        .map(|_| {
            let config = AccessRulesConfig::new().default(
                rule!(require(NonFungibleGlobalId::from_public_key(&public_key))),
                rule!(require(NonFungibleGlobalId::from_public_key(&public_key))),
            );
            let manifest = ManifestBuilder::new()
                .lock_fee(FAUCET, 100.into())
                .new_account_advanced(config)
                .build();
            let account = execute_and_commit_transaction(
                &mut substate_db,
                &mut scrypto_interpreter,
                &FeeReserveConfig::default(),
                &ExecutionConfig::default(),
                &TestTransaction::new(manifest.clone(), 1, DEFAULT_COST_UNIT_LIMIT)
                    .get_executable(btreeset![NonFungibleGlobalId::from_public_key(&public_key)]),
            )
            .expect_commit(true)
            .new_component_addresses()[0];

            account
        })
        .collect::<Vec<ComponentAddress>>();

    let account1 = accounts[0];
    let account2 = accounts[1];

    // Fill first account
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET, 100.into())
        .call_method(FAUCET, "free", manifest_args!())
        .call_method(
            account1,
            "deposit_batch",
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();
    for nonce in 0..1000 {
        execute_and_commit_transaction(
            &mut substate_db,
            &mut scrypto_interpreter,
            &FeeReserveConfig::default(),
            &ExecutionConfig::default(),
            &TestTransaction::new(manifest.clone(), nonce, DEFAULT_COST_UNIT_LIMIT)
                .get_executable(btreeset![NonFungibleGlobalId::from_public_key(&public_key)]),
        )
        .expect_commit(true);
    }

    // Create a transfer manifest
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET, 100.into())
        .withdraw_from_account(account1, RADIX_TOKEN, dec!("0.000001"))
        .call_method(
            account2,
            "deposit_batch",
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();

    // Loop
    let mut nonce = 3;
    c.bench_function("Transfer::run", |b| {
        b.iter(|| {
            let receipt = execute_and_commit_transaction(
                &mut substate_db,
                &mut scrypto_interpreter,
                &FeeReserveConfig::default(),
                &ExecutionConfig::default(),
                &TestTransaction::new(manifest.clone(), nonce, DEFAULT_COST_UNIT_LIMIT)
                    .get_executable(btreeset![NonFungibleGlobalId::from_public_key(&public_key)]),
            );
            receipt.expect_commit_success();
            nonce += 1;
        })
    });
}

criterion_group!(database, db_rw_test);
criterion_main!(database);
