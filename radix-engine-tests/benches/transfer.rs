use criterion::{criterion_group, criterion_main, Criterion};
use radix_engine::system::bootstrap::Bootstrapper;
use radix_engine::transaction::execute_and_commit_transaction;
use radix_engine::transaction::{ExecutionConfig, FeeReserveConfig};
use radix_engine::types::*;
use radix_engine::vm::wasm::WasmInstrumenter;
use radix_engine::vm::wasm::{DefaultWasmEngine, WasmMeteringConfig};
use radix_engine::vm::ScryptoVm;
use radix_engine_interface::dec;
use radix_engine_interface::rule;
use radix_engine_stores::memory_db::InMemorySubstateDatabase;
use transaction::builder::ManifestBuilder;
use transaction::ecdsa_secp256k1::EcdsaSecp256k1PrivateKey;
use transaction::model::TestTransaction;

fn bench_transfer(c: &mut Criterion) {
    // Set up environment.
    let mut scrypto_interpreter = ScryptoVm {
        wasm_engine: DefaultWasmEngine::default(),
        wasm_instrumenter: WasmInstrumenter::default(),
        wasm_metering_config: WasmMeteringConfig::V0,
    };
    let mut substate_db = InMemorySubstateDatabase::standard();
    Bootstrapper::new(&mut substate_db, &scrypto_interpreter, false)
        .bootstrap_test_default()
        .unwrap();

    // Create a key pair
    let private_key = EcdsaSecp256k1PrivateKey::from_u64(1).unwrap();
    let public_key = private_key.public_key();

    // Create two accounts
    let accounts = (0..2)
        .map(|_| {
            let mut roles = Roles::new();
            roles.define_role(
                "owner",
                rule!(require(NonFungibleGlobalId::from_public_key(&public_key))),
                ["owner"],
            );
            let manifest = ManifestBuilder::new()
                .lock_fee(FAUCET, 100.into())
                .new_account_advanced(authority_rules)
                .build();
            let account = execute_and_commit_transaction(
                &mut substate_db,
                &mut scrypto_interpreter,
                &FeeReserveConfig::default(),
                &ExecutionConfig::default(),
                &TestTransaction::new_from_nonce(manifest.clone(), 1)
                    .prepare()
                    .unwrap()
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
            &TestTransaction::new_from_nonce(manifest.clone(), nonce)
                .prepare()
                .unwrap()
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
                &TestTransaction::new_from_nonce(manifest.clone(), nonce)
                    .prepare()
                    .unwrap()
                    .get_executable(btreeset![NonFungibleGlobalId::from_public_key(&public_key)]),
            );
            receipt.expect_commit_success();
            nonce += 1;
        })
    });
}

criterion_group!(transfer, bench_transfer);
criterion_main!(transfer);
