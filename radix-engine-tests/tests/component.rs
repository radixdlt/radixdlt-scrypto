use radix_engine::errors::{InterpreterError, RuntimeError};
use radix_engine::types::*;
use radix_engine_interface::blueprints::resource::FromPublicKey;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn test_component() {
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let package = test_runner.compile_and_publish("./tests/blueprints/component");

    // Create component
    let manifest1 = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_function(
            package,
            "ComponentTest",
            "create_component",
            manifest_args!(),
        )
        .build();
    let receipt1 = test_runner.execute_manifest(manifest1, vec![]);
    receipt1.expect_commit_success();

    // Find the component address from receipt
    let component = receipt1.expect_commit(true).new_component_addresses()[0];

    // Call functions & methods
    let manifest2 = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_function(
            package,
            "ComponentTest",
            "get_component_info",
            manifest_args!(component),
        )
        .call_method(component, "get_component_state", manifest_args!())
        .call_method(component, "put_component_state", manifest_args!())
        .call_method(
            account,
            "deposit_batch",
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();
    let receipt2 = test_runner.execute_manifest(
        manifest2,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt2.expect_commit_success();
}

#[test]
fn invalid_blueprint_name_should_cause_error() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_addr = test_runner.compile_and_publish("./tests/blueprints/component");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_function(
            package_addr,
            "NonExistentBlueprint",
            "create_component",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        if let RuntimeError::InterpreterError(InterpreterError::ScryptoBlueprintNotFound(
            blueprint,
        )) = e
        {
            package_addr.eq(&blueprint.package_address)
                && blueprint.blueprint_name.eq("NonExistentBlueprint")
        } else {
            false
        }
    });
}
