#[rustfmt::skip]
pub mod test_runner;

use crate::test_runner::TestRunner;
use radix_engine::errors::RuntimeError;
use radix_engine::ledger::InMemorySubstateStore;
use scrypto::prelude::*;

#[test]
fn test_package() {
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let package = test_runner.publish_package("component");

    let transaction1 = test_runner
        .new_transaction_builder()
        .call_function(package, "PackageTest", "publish", vec![])
        .build(vec![])
        .unwrap();
    let receipt1 = test_runner.run(transaction1);
    assert!(receipt1.result.is_ok());
}

#[test]
fn test_component() {
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (key, account) = test_runner.new_public_key_with_account();
    let package = test_runner.publish_package("component");

    // Create component
    let transaction1 = test_runner
        .new_transaction_builder()
        .call_function(package, "ComponentTest", "create_component", vec![])
        .build(vec![])
        .unwrap();
    let receipt1 = test_runner.run(transaction1);
    assert!(receipt1.result.is_ok());

    // Find the component ID from receipt
    let component = receipt1.new_component_ids[0];

    // Call functions & methods
    let transaction2 = test_runner
        .new_transaction_builder()
        .call_function(
            package,
            "ComponentTest",
            "get_component_info",
            vec![scrypto_encode(&component)],
        )
        .call_method(component, "get_component_state", vec![])
        .call_method(component, "put_component_state", vec![])
        .call_method_with_all_resources(account, "deposit_batch")
        .build(vec![key])
        .unwrap();
    let receipt2 = test_runner.run(transaction2);
    assert!(receipt2.result.is_ok());
}

#[test]
fn invalid_blueprint_name_should_cause_error() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let package_id = test_runner.publish_package("component");

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .call_function(
            package_id,
            "NonExistentBlueprint",
            "create_component",
            vec![],
        )
        .build(vec![])
        .unwrap();
    let receipt = test_runner.run(transaction);

    // Assert
    let error = receipt.result.expect_err("Should be an error.");
    assert_eq!(
        error,
        RuntimeError::BlueprintNotFound(package_id, "NonExistentBlueprint".to_string())
    );
}
