// This is optional, as you may choose to use std for testing only.
#![no_std]

use radix_engine::engine::TransactionExecutor;
use radix_engine::ledger::*;
use radix_engine::model::extract_package;
use radix_engine::wasm::DefaultWasmEngine;
use scrypto::call_data;
use scrypto::prelude::*;
use transaction::builder::ManifestBuilder;
use transaction::signing::EcdsaPrivateKey;
use transaction::validation::TestTransaction;

#[test]
fn test_say_hello() {
    // Set up environment.
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut wasm_engine = DefaultWasmEngine::new();
    let mut executor = TransactionExecutor::new(&mut substate_store, &mut wasm_engine, false);

    // Create a key pair
    let private_key = EcdsaPrivateKey::from_u64(1).unwrap();
    let public_key = private_key.public_key();

    // Publish package
    let manifest = ManifestBuilder::new()
        .publish_package(extract_package(include_package!("no_std").to_vec()).unwrap())
        .build();
    let package_address = executor
        .execute(&TestTransaction::new(manifest, 1, vec![public_key]))
        .new_package_addresses[0];

    // Test the `say_hello` function.
    let manifest = ManifestBuilder::new()
        .call_function(package_address, "NoStd", call_data!(say_hello()))
        .build();
    let receipt = executor.execute(&TestTransaction::new(manifest, 2, vec![]));
    receipt.result.expect("Should be okay.");
}
