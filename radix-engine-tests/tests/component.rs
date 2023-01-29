use radix_engine::engine::{InterpreterError, RuntimeError, ScryptoFnResolvingError};
use radix_engine::types::*;
use radix_engine_interface::model::FromPublicKey;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn test_component() {
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let package = test_runner.compile_and_publish("./tests/blueprints/component");

    // Create component
    let manifest1 = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_function(package, "ComponentTest", "create_component", args!())
        .build();
    let receipt1 = test_runner.execute_manifest(manifest1, vec![]);
    receipt1.expect_commit_success();

    // Find the component address from receipt
    let component = receipt1
        .expect_commit()
        .entity_changes
        .new_component_addresses[0];

    // Call functions & methods
    let manifest2 = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_function(
            package,
            "ComponentTest",
            "get_component_info",
            args!(component),
        )
        .call_method(component, "get_component_state", args!())
        .call_method(component, "put_component_state", args!())
        .call_method(
            account,
            "deposit_batch",
            args!(ManifestExpression::EntireWorktop),
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
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_function(
            package_addr,
            "NonExistentBlueprint",
            "create_component",
            args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        if let RuntimeError::InterpreterError(InterpreterError::InvalidScryptoInvocation(
            package_address,
            blueprint_name,
            _,
            ScryptoFnResolvingError::BlueprintNotFound,
        )) = e
        {
            package_addr.eq(&package_address) && blueprint_name.eq("NonExistentBlueprint")
        } else {
            false
        }
    });
}
