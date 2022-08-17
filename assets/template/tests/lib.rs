use radix_engine::ledger::*;
use radix_engine::model::extract_package;
use scrypto::core::Network;
use scrypto::prelude::*;
use scrypto::to_struct;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn test_hello() {
    // Setup the environment
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);

    // Create an account
    let (public_key, _private_key, account_component) = test_runner.new_account();

    // Publish package
    let package_address = test_runner.publish_package(extract_package(compile_package!()).unwrap());

    // Test the `instantiate_hello` function.
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .call_function(package_address, "Hello", "instantiate_hello", to_struct!())
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![public_key]);
    println!("{:?}\n", receipt);
    receipt.expect_success();
    let component = receipt.new_component_addresses[0];

    // Test the `free_token` method.
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .call_method(component, "free_token", to_struct!())
        .call_method_with_all_resources(account_component, "deposit_batch")
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![public_key]);
    println!("{:?}\n", receipt);
    receipt.expect_success();
}
