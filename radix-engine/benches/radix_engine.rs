use criterion::{criterion_group, criterion_main, Criterion};
use radix_engine::constants::*;
use radix_engine::engine::TransactionExecutor;
use radix_engine::ledger::*;
use radix_engine::wasm::DefaultWasmEngine;
use radix_engine::wasm::WasmInstrumenter;
use scrypto::core::Network;
use scrypto::prelude::*;
use scrypto::to_struct;
use transaction::builder::ManifestBuilder;
use transaction::model::TestTransaction;
use transaction::signing::EcdsaPrivateKey;

fn bench_transfer(c: &mut Criterion) {
    // Set up environment.
    let substate_store = InMemorySubstateStore::with_bootstrap();
    let mut wasm_engine = DefaultWasmEngine::new();
    let mut wasm_instrumenter = WasmInstrumenter::new();
    let cost_unit_price = DEFAULT_COST_UNIT_PRICE.parse().unwrap();
    let max_call_depth = DEFAULT_MAX_CALL_DEPTH;
    let system_loan = DEFAULT_SYSTEM_LOAN;
    let is_system = false;
    let trace = false;
    let mut executor = TransactionExecutor::new(
        substate_store,
        &mut wasm_engine,
        &mut wasm_instrumenter,
        cost_unit_price,
        max_call_depth,
        system_loan,
        is_system,
        trace,
    );

    // Create a key pair
    let private_key = EcdsaPrivateKey::from_u64(1).unwrap();
    let public_key = private_key.public_key();

    // Create two accounts
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .call_method(SYSTEM_COMPONENT, "free_xrd", to_struct!())
        .take_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
            builder.new_account_with_resource(
                &rule!(require(NonFungibleAddress::from_public_key(&public_key))),
                bucket_id,
            )
        })
        .build();
    let account1 = executor
        .execute(&TestTransaction::new(manifest.clone(), 1, vec![public_key]))
        .new_component_addresses[0];
    let account2 = executor
        .execute(&TestTransaction::new(manifest, 2, vec![public_key]))
        .new_component_addresses[0];

    // Create a transfer manifest
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .withdraw_from_account_by_amount(1.into(), RADIX_TOKEN, account1)
        .call_method_with_all_resources(account2, "deposit_batch")
        .build();

    // Loop
    let mut nonce = 3;
    c.bench_function("Transfer", |b| {
        b.iter(|| {
            let receipt = executor.execute(&TestTransaction::new(
                manifest.clone(),
                nonce,
                vec![public_key],
            ));
            receipt.result.expect("It should work");
            nonce += 1;
        })
    });
}

criterion_group!(radix_engine, bench_transfer);
criterion_main!(radix_engine);
