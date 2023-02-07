use criterion::{criterion_group, criterion_main, Criterion};
use radix_engine::kernel::ScryptoInterpreter;
use radix_engine::ledger::*;
use radix_engine::transaction::execute_and_commit_transaction;
use radix_engine::transaction::{ExecutionConfig, FeeReserveConfig};
use radix_engine::types::*;
use radix_engine::wasm::WasmInstrumenter;
use radix_engine::wasm::{DefaultWasmEngine, WasmMeteringConfig};
use radix_engine_constants::DEFAULT_COST_UNIT_LIMIT;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::dec;
use radix_engine_interface::rule;
use scrypto_unit::TestRunner;
use transaction::builder::ManifestBuilder;
use transaction::model::TestTransaction;
use transaction::signing::EcdsaSecp256k1PrivateKey;

fn bench_transfer(c: &mut Criterion) {
    // Set up environment.
    let mut scrypto_interpreter = ScryptoInterpreter {
        wasm_engine: DefaultWasmEngine::default(),
        wasm_instrumenter: WasmInstrumenter::default(),
        wasm_metering_config: WasmMeteringConfig::V0,
    };
    let mut substate_store = TypedInMemorySubstateStore::with_bootstrap(&scrypto_interpreter);

    // Create a key pair
    let private_key = EcdsaSecp256k1PrivateKey::from_u64(1).unwrap();
    let public_key = private_key.public_key();

    // Create two accounts
    let accounts = (0..2)
        .map(|_| {
            let manifest = ManifestBuilder::new()
                .lock_fee(FAUCET_COMPONENT, 100.into())
                .new_account(&rule!(require(NonFungibleGlobalId::from_public_key(
                    &public_key
                ))))
                .build();
            let account = execute_and_commit_transaction(
                &mut substate_store,
                &mut scrypto_interpreter,
                &FeeReserveConfig::default(),
                &ExecutionConfig::default(),
                &TestTransaction::new(manifest.clone(), 1, DEFAULT_COST_UNIT_LIMIT)
                    .get_executable(vec![NonFungibleGlobalId::from_public_key(&public_key)]),
            )
            .expect_commit()
            .entity_changes
            .new_component_addresses[0];

            let manifest = ManifestBuilder::new()
                .lock_fee(FAUCET_COMPONENT, 100.into())
                .call_method(FAUCET_COMPONENT, "free", args!())
                .call_method(
                    account,
                    "deposit_batch",
                    args!(ManifestExpression::EntireWorktop),
                )
                .build();
            execute_and_commit_transaction(
                &mut substate_store,
                &mut scrypto_interpreter,
                &FeeReserveConfig::default(),
                &ExecutionConfig::default(),
                &TestTransaction::new(manifest.clone(), 1, DEFAULT_COST_UNIT_LIMIT)
                    .get_executable(vec![NonFungibleGlobalId::from_public_key(&public_key)]),
            )
            .expect_commit();

            account
        })
        .collect::<Vec<ComponentAddress>>();

    let account1 = accounts[0];
    let account2 = accounts[1];

    // Fill first account
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 100.into())
        .call_method(FAUCET_COMPONENT, "free", args!())
        .call_method(
            account1,
            "deposit_batch",
            args!(ManifestExpression::EntireWorktop),
        )
        .build();
    for nonce in 0..1000 {
        execute_and_commit_transaction(
            &mut substate_store,
            &mut scrypto_interpreter,
            &FeeReserveConfig::default(),
            &ExecutionConfig::default(),
            &TestTransaction::new(manifest.clone(), nonce, DEFAULT_COST_UNIT_LIMIT)
                .get_executable(vec![NonFungibleGlobalId::from_public_key(&public_key)]),
        )
        .expect_commit();
    }

    // Create a transfer manifest
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 100.into())
        .withdraw_from_account(account1, RADIX_TOKEN, dec!("0.000001"))
        .call_method(
            account2,
            "deposit_batch",
            args!(ManifestExpression::EntireWorktop),
        )
        .build();

    // Loop
    let mut nonce = 3;
    c.bench_function("Transfer", |b| {
        b.iter(|| {
            let receipt = execute_and_commit_transaction(
                &mut substate_store,
                &mut scrypto_interpreter,
                &FeeReserveConfig::default(),
                &ExecutionConfig::default(),
                &TestTransaction::new(manifest.clone(), nonce, DEFAULT_COST_UNIT_LIMIT)
                    .get_executable(vec![NonFungibleGlobalId::from_public_key(&public_key)]),
            );
            receipt.expect_commit_success();
            nonce += 1;
        })
    });
}

fn bench_spin_loop(c: &mut Criterion) {
    // Set up environment.
    let mut test_runner = TestRunner::builder().without_trace().build();

    let package_address = test_runner.compile_and_publish("./tests/blueprints/fee");
    let component_address = test_runner
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee(FAUCET_COMPONENT, 10u32.into())
                .call_method(FAUCET_COMPONENT, "free", args!())
                .take_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
                    builder.call_function(package_address, "Fee", "new", args!(bucket_id));
                    builder
                })
                .build(),
            vec![],
        )
        .expect_commit()
        .entity_changes
        .new_component_addresses[0];

    // Create a transfer manifest
    let manifest = ManifestBuilder::new()
        // First, lock the fee so that the loan will be repaid
        .call_method(FAUCET_COMPONENT, "lock_fee", args!(Decimal::from(10)))
        // Now spin-loop to wait for the fee loan to burn through
        .call_method(component_address, "spin_loop", args!())
        .build();

    // Loop
    c.bench_function("Spin Loop", |b| {
        b.iter(|| {
            let receipt = test_runner.execute_manifest(manifest.clone(), vec![]);
            receipt.expect_commit_failure();
        })
    });
}

criterion_group!(radix_engine, bench_transfer, bench_spin_loop);
criterion_main!(radix_engine);
