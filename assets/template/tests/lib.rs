use radix_engine::engine::TransactionExecutor;
use radix_engine::ledger::*;
use radix_engine::model::extract_package;
use radix_engine::wasm::DefaultWasmEngine;
use scrypto::prelude::*;
use transaction::builder::ManifestBuilder;
use transaction::signing::EcdsaPrivateKey;
use transaction::validation::TestTransaction;

#[test]
fn test_hello() {
    // TODO: Make TestRunner publicly available

    // Set up environment.
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut wasm_engine = DefaultWasmEngine::new();
    let mut executor = TransactionExecutor::new(&mut substate_store, &mut wasm_engine, false);

    // Create a key pair
    let private_key = EcdsaPrivateKey::from_u64(1).unwrap();
    let public_key = private_key.public_key();

    // Publish package
    let manifest = ManifestBuilder::new()
        .publish_package(extract_package(compile_package!()).unwrap())
        .build();
    let package_address = executor
        .execute(&TestTransaction::new(manifest, 1, vec![public_key]))
        .new_package_addresses[0];

    // Create an account
    let manifest = ManifestBuilder::new()
        .call_method(SYSTEM_COMPONENT, call_data!(free_xrd()))
        .take_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
            builder.new_account_with_resource(
                &rule!(require(NonFungibleAddress::from_public_key(&public_key))),
                bucket_id,
            )
        })
        .build();
    let account = executor
        .execute(&TestTransaction::new(manifest, 3, vec![public_key]))
        .new_component_addresses[0];

    // Test the `instantiate_hello` function.
    let manifest = ManifestBuilder::new()
        .call_function(package_address, "Hello", call_data!(instantiate_hello()))
        .build();
    let receipt = executor.execute(&TestTransaction::new(manifest, 3, vec![public_key]));
    println!("{:?}\n", receipt);
    receipt.result.expect("Should be okay.");

    // Test the `free_token` method.
    let manifest = ManifestBuilder::new()
        .call_method(receipt.new_component_addresses[0], call_data!(free_token()))
        .call_method_with_all_resources(account, "deposit_batch")
        .build();
    let receipt = executor.execute(&TestTransaction::new(manifest, 4, vec![public_key]));
    println!("{:?}\n", receipt);
    receipt.result.expect("Should be okay.");
}
