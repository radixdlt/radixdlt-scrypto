use criterion::{criterion_group, criterion_main, Criterion};
use radix_engine::system::bootstrap::Bootstrapper;
use radix_engine::transaction::execute_and_commit_transaction;
use radix_engine::transaction::{ExecutionConfig, FeeReserveConfig};
use radix_engine::types::*;
use radix_engine::vm::wasm::{DefaultWasmEngine, WasmValidatorConfigV1};
use radix_engine::vm::{DefaultNativeVm, ScryptoVm, Vm};
use radix_engine_interface::dec;
use radix_engine_interface::rule;
use radix_engine_stores::memory_db::InMemorySubstateDatabase;
use transaction::model::TestTransaction;
use transaction::prelude::*;
use transaction::signing::secp256k1::Secp256k1PrivateKey;

fn bench_transfer(c: &mut Criterion) {
    // Set up environment.
    let scrypto_vm = ScryptoVm {
        wasm_engine: DefaultWasmEngine::default(),
        wasm_validator_config: WasmValidatorConfigV1::new(),
    };
    let native_vm = DefaultNativeVm::new();
    let vm = Vm::new(&scrypto_vm, native_vm);
    let mut substate_db = InMemorySubstateDatabase::standard();
    Bootstrapper::new(&mut substate_db, vm.clone(), false)
        .bootstrap_test_default()
        .unwrap();

    // Create a key pair
    let private_key = Secp256k1PrivateKey::from_u64(1).unwrap();
    let public_key = private_key.public_key();

    // Create two accounts
    let accounts = (0..2)
        .map(|_| {
            let owner_role = OwnerRole::Updatable(rule!(require(
                NonFungibleGlobalId::from_public_key(&public_key)
            )));
            let manifest = ManifestBuilder::new()
                .lock_fee_from_faucet()
                .new_account_advanced(owner_role)
                .build();
            let account = execute_and_commit_transaction(
                &mut substate_db,
                vm.clone(),
                &FeeReserveConfig::default(),
                &ExecutionConfig::for_notarized_transaction(),
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
        .lock_fee_from_faucet()
        .get_free_xrd_from_faucet()
        .try_deposit_batch_or_abort(account1)
        .build();
    for nonce in 0..1000 {
        execute_and_commit_transaction(
            &mut substate_db,
            vm.clone(),
            &FeeReserveConfig::default(),
            &ExecutionConfig::for_notarized_transaction(),
            &TestTransaction::new_from_nonce(manifest.clone(), nonce)
                .prepare()
                .unwrap()
                .get_executable(btreeset![NonFungibleGlobalId::from_public_key(&public_key)]),
        )
        .expect_commit(true);
    }

    // Create a transfer manifest
    let manifest = ManifestBuilder::new()
        .lock_standard_test_fee(account1)
        .withdraw_from_account(account1, XRD, dec!("0.000001"))
        .try_deposit_batch_or_abort(account2)
        .build();

    // Loop
    let mut nonce = 3;
    c.bench_function("Transfer::run", |b| {
        b.iter(|| {
            let receipt = execute_and_commit_transaction(
                &mut substate_db,
                vm.clone(),
                &FeeReserveConfig::default(),
                &ExecutionConfig::for_notarized_transaction(),
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
