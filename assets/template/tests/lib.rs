use radix_engine::ledger::*;
use radix_engine::model::extract_package;
use scrypto::core::NetworkDefinition;
use scrypto::prelude::*;
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
    let manifest = ManifestBuilder::new(&NetworkDefinition::local_simulator())
        .call_function(package_address, "Hello", "instantiate_hello", args!())
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![public_key]);
    println!("{:?}\n", receipt);
    receipt.expect_commit_success();
    let component = receipt.expect_commit().entity_changes.new_component_addresses[0];

    // Test the `free_token` method.
    let manifest = ManifestBuilder::new(&NetworkDefinition::local_simulator())
        .call_method(component, "free_token", args!())
        .call_method_with_all_resources(account_component, "deposit_batch")
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![public_key]);
    println!("{:?}\n", receipt);
    receipt.expect_commit_success();
}
