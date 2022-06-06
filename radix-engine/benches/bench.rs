#[macro_use]
extern crate bencher;
use bencher::Bencher;

use radix_engine::engine::TransactionExecutor;
use radix_engine::ledger::*;
use radix_engine::wasm::DefaultWasmEngine;
use scrypto::prelude::*;
use transaction::builder::ManifestBuilder;
use transaction::model::TestTransaction;
use transaction::signing::EcdsaPrivateKey;

fn bench_transfer(b: &mut Bencher) {
    // Set up environment.
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut wasm_engine = DefaultWasmEngine::new();
    let mut executor = TransactionExecutor::new(&mut substate_store, &mut wasm_engine, false);

    // Create a key pair
    let private_key = EcdsaPrivateKey::from_u64(1).unwrap();
    let public_key = private_key.public_key();

    // Create two accounts
    let manifest = ManifestBuilder::new()
        .call_method(SYSTEM_COMPONENT, call_data!(free_xrd()))
        .take_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
            builder.new_account_with_resource(
                &rule!(require(NonFungibleAddress::from_public_key(&public_key))),
                bucket_id,
            )
        })
        .build();
    let account1 = executor
        .execute(&TestTransaction::new(manifest.clone(), 3, vec![public_key]))
        .new_component_addresses[0];
    let account2 = executor
        .execute(&TestTransaction::new(manifest, 4, vec![public_key]))
        .new_component_addresses[0];

    // Create a transfer manifest
    let manifest = ManifestBuilder::new()
        .withdraw_from_account_by_amount(1.into(), RADIX_TOKEN, account1)
        .call_method_with_all_resources(account2, "deposit_batch")
        .build();

    // Loop
    let mut nonce = 5;
    b.iter(|| {
        let receipt = executor.execute(&TestTransaction::new(
            manifest.clone(),
            nonce,
            vec![public_key],
        ));
        receipt.result.expect("It should work");
        nonce += 1;
    });
}

benchmark_group!(radix_engine, bench_transfer);
benchmark_main!(radix_engine);
