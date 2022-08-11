// This is optional, as you may choose to use std for testing only.
#![no_std]

use radix_engine::ledger::*;
use radix_engine::model::extract_package;
use radix_engine::transaction::ExecutionConfig;
use radix_engine::transaction::TransactionExecutor;
use radix_engine::wasm::*;
use scrypto::core::Network;
use scrypto::prelude::*;
use scrypto::to_struct;
use transaction::builder::ManifestBuilder;
use transaction::model::TestTransaction;
use transaction::signing::EcdsaPrivateKey;

#[test]
fn test_say_hello() {
    // Set up environment.
    let mut substate_store = TypedInMemorySubstateStore::with_bootstrap();
    let mut wasm_engine = DefaultWasmEngine::new();
    let mut wasm_instrumenter = WasmInstrumenter::new();
    let mut executor = TransactionExecutor::new(
        &mut substate_store,
        &mut wasm_engine,
        &mut wasm_instrumenter,
    );

    // Create a key pair
    let private_key = EcdsaPrivateKey::from_u64(1).unwrap();
    let public_key = private_key.public_key();

    // Publish package
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .lock_fee(10.into(), SYSTEM_COMPONENT)
        .publish_package(extract_package(include_package!("no_std").to_vec()).unwrap())
        .build();
    let package_address = executor
        .execute_and_commit(
            &TestTransaction::new(manifest, 1, vec![public_key]),
            &ExecutionConfig::debug(),
        )
        .new_package_addresses[0];

    // Test the `say_hello` function.
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .lock_fee(10.into(), SYSTEM_COMPONENT)
        .call_function(package_address, "NoStd", "say_hello", to_struct!())
        .build();
    let receipt = executor.execute_and_commit(
        &TestTransaction::new(manifest, 2, vec![]),
        &ExecutionConfig::debug(),
    );
    receipt.expect_success();
}
