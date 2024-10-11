use criterion::{criterion_group, criterion_main, Criterion};
use radix_common::prelude::*;
use radix_engine::transaction::execute_and_commit_transaction;
use radix_engine::transaction::ExecutionConfig;
use radix_engine::updates::ProtocolBuilder;
use radix_engine::vm::*;
use radix_engine_interface::prelude::*;
use radix_engine_interface::rule;
use radix_substate_store_impls::memory_db::InMemorySubstateDatabase;
use radix_transactions::model::TestTransaction;
use radix_transactions::prelude::*;
use radix_transactions::validation::TransactionValidator;

fn bench_transfer(c: &mut Criterion) {
    // Set up environment.
    let mut substate_db = InMemorySubstateDatabase::standard();
    let network = NetworkDefinition::simulator();
    let vm_modules = VmModules::default();
    ProtocolBuilder::for_network(&network)
        .from_bootstrap_to_latest()
        .commit_each_protocol_update(&mut substate_db);
    let validator = TransactionValidator::new(&substate_db, &network);

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
                &vm_modules,
                &ExecutionConfig::for_notarized_transaction(NetworkDefinition::simulator()),
                TestTransaction::new_v1_from_nonce(
                    manifest,
                    1,
                    btreeset![NonFungibleGlobalId::from_public_key(&public_key)],
                )
                .into_executable(&validator)
                .unwrap(),
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
            &vm_modules,
            &ExecutionConfig::for_notarized_transaction(NetworkDefinition::simulator()),
            TestTransaction::new_v1_from_nonce(
                manifest.clone(),
                nonce,
                btreeset![NonFungibleGlobalId::from_public_key(&public_key)],
            )
            .into_executable(&validator)
            .unwrap(),
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
                &vm_modules,
                &ExecutionConfig::for_notarized_transaction(NetworkDefinition::simulator()),
                TestTransaction::new_v1_from_nonce(
                    manifest.clone(),
                    nonce,
                    btreeset![NonFungibleGlobalId::from_public_key(&public_key)],
                )
                .into_executable(&validator)
                .unwrap(),
            );
            receipt.expect_commit_success();
            nonce += 1;
        })
    });
}

criterion_group!(transfer, bench_transfer);
criterion_main!(transfer);
