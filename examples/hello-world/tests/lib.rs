use radix_engine_interface::model::FromPublicKey;
use radix_engine_interface::node::NetworkDefinition;
use scrypto::prelude::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn test_hello() {
    // Setup the environment
    let mut test_runner = TestRunner::new(true);

    // Create an account
    let (public_key, _private_key, account_component) = test_runner.new_allocated_account();

    // Publish package
    let package_address = test_runner.compile_and_publish(this_package!());

    // Test the `instantiate_hello` function.
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .call_function(package_address, "Hello", "instantiate_hello", args!())
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleAddress::from_public_key(&public_key)],
    );
    println!("{:?}\n", receipt);
    receipt.expect_commit_success();
    let component = receipt
        .expect_commit()
        .entity_changes
        .new_component_addresses[0];

    // Test the `free_token` method.
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .call_method(component, "free_token", args!())
        .call_method(
            account_component,
            "deposit_batch",
            args!(Expression::entire_worktop()),
        )
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleAddress::from_public_key(&public_key)],
    );
    println!("{:?}\n", receipt);
    receipt.expect_commit_success();
}
