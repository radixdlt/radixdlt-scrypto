use criterion::{criterion_group, criterion_main, Criterion};
use radix_common::prelude::*;
use radix_engine::system::bootstrap::Bootstrapper;
use radix_engine::transaction::execute_and_commit_transaction;
use radix_engine::transaction::ExecutionConfig;
use radix_engine::vm::wasm::{DefaultWasmEngine, WasmValidatorConfigV1};
use radix_engine::vm::{NoExtension, ScryptoVm, VmInit};
use radix_engine_interface::prelude::*;
use radix_engine_interface::rule;
use radix_substate_store_impls::memory_db::InMemorySubstateDatabase;
use radix_transactions::model::TestTransaction;
use radix_transactions::prelude::*;

fn bench_transfer(c: &mut Criterion) {
    // Set up environment.
    let scrypto_vm = ScryptoVm {
        wasm_engine: DefaultWasmEngine::default(),
        wasm_validator_config: WasmValidatorConfigV1::new(),
    };
    let vm_init = VmInit::new(&scrypto_vm, NoExtension);
    let mut substate_db = InMemorySubstateDatabase::standard();
    Bootstrapper::new(
        NetworkDefinition::simulator(),
        &mut substate_db,
        vm_init.clone(),
        false,
    )
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
                .new_account_advanced(owner_role, None)
                .build();
            let account = execute_and_commit_transaction(
                &mut substate_db,
                vm_init.clone(),
                &ExecutionConfig::for_notarized_transaction(NetworkDefinition::simulator()),
                TestTransaction::new_from_nonce(manifest, 1)
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
        .try_deposit_entire_worktop_or_abort(account1, None)
        .build();
    for nonce in 0..1000 {
        execute_and_commit_transaction(
            &mut substate_db,
            vm_init.clone(),
            &ExecutionConfig::for_notarized_transaction(NetworkDefinition::simulator()),
            TestTransaction::new_from_nonce(manifest.clone(), nonce)
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
        .try_deposit_entire_worktop_or_abort(account2, None)
        .build();

    // Loop
    let mut nonce = 3;
    c.bench_function("transaction::transfer", |b| {
        b.iter(|| {
            let receipt = execute_and_commit_transaction(
                &mut substate_db,
                vm_init.clone(),
                &ExecutionConfig::for_notarized_transaction(NetworkDefinition::simulator()),
                TestTransaction::new_from_nonce(manifest.clone(), nonce)
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
