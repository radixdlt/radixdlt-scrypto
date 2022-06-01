#[rustfmt::skip]
pub mod test_runner;

use crate::test_runner::TestRunner;
use radix_engine::engine::RuntimeError;
use radix_engine::ledger::InMemorySubstateStore;
use radix_engine::wasm::default_wasm_engine;
use scrypto::prelude::*;
use scrypto::to_struct;

#[test]
fn test_package() {
    let mut substate_store = InMemorySubstateStore::new();
    let mut wasm_engine = default_wasm_engine();
    let mut test_runner = TestRunner::new(&mut substate_store, &mut wasm_engine);
    let package = test_runner.publish_package("component");

    let transaction1 = test_runner
        .new_transaction_builder()
        .call_function(package, "PackageTest", "publish", to_struct!())
        .build(test_runner.get_nonce([]))
        .sign([]);
    let receipt1 = test_runner.validate_and_execute(&transaction1);
    assert!(receipt1.result.is_ok());
}

#[test]
fn test_component() {
    let mut substate_store = InMemorySubstateStore::new();
    let mut wasm_engine = default_wasm_engine();
    let mut test_runner = TestRunner::new(&mut substate_store, &mut wasm_engine);
    let (pk, sk, account) = test_runner.new_account();
    let package = test_runner.publish_package("component");

    // Create component
    let transaction1 = test_runner
        .new_transaction_builder()
        .call_function(package, "ComponentTest", "create_component", to_struct!())
        .build(test_runner.get_nonce([]))
        .sign([]);
    let receipt1 = test_runner.validate_and_execute(&transaction1);
    assert!(receipt1.result.is_ok());

    // Find the component address from receipt
    let component = receipt1.new_component_addresses[0];

    // Call functions & methods
    let transaction2 = test_runner
        .new_transaction_builder()
        .call_function(
            package,
            "ComponentTest",
            "get_component_info",
            to_struct!(component),
        )
        .call_method(component, "get_component_state", to_struct!())
        .call_method(component, "put_component_state", to_struct!())
        .call_method_with_all_resources(account, "deposit_batch")
        .build(test_runner.get_nonce([pk]))
        .sign([&sk]);
    let receipt2 = test_runner.validate_and_execute(&transaction2);
    receipt2.result.expect("Should be okay.");
}

#[test]
fn invalid_blueprint_name_should_cause_error() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::new();
    let mut wasm_engine = default_wasm_engine();
    let mut test_runner = TestRunner::new(&mut substate_store, &mut wasm_engine);
    let package_address = test_runner.publish_package("component");

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .call_function(
            package_address,
            "NonExistentBlueprint",
            "create_component",
            to_struct!(),
        )
        .build(test_runner.get_nonce([]))
        .sign([]);
    let receipt = test_runner.validate_and_execute(&transaction);

    // Assert
    let error = receipt.result.expect_err("Should be an error.");
    assert_eq!(
        error,
        RuntimeError::BlueprintNotFound(package_address, "NonExistentBlueprint".to_string())
    );
}

#[test]
fn reentrancy_should_not_be_possible() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::new();
    let mut wasm_engine = default_wasm_engine();
    let mut test_runner = TestRunner::new(&mut substate_store, &mut wasm_engine);
    let package_address = test_runner.publish_package("component");
    let transaction = test_runner
        .new_transaction_builder()
        .call_function(package_address, "ReentrantComponent", "new", to_struct!())
        .build(test_runner.get_nonce([]))
        .sign([]);
    let receipt = test_runner.validate_and_execute(&transaction);
    receipt.result.expect("Should be okay");
    let component_address = receipt.new_component_addresses[0];

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .call_method(component_address, "call_self", to_struct!())
        .build(test_runner.get_nonce([]))
        .sign([]);
    let receipt = test_runner.validate_and_execute(&transaction);

    // Assert
    let error = receipt.result.expect_err("Should be an error.");
    assert_eq!(error, RuntimeError::ComponentReentrancy(component_address))
}

#[test]
fn missing_component_address_should_cause_error() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::new();
    let mut wasm_engine = default_wasm_engine();
    let mut test_runner = TestRunner::new(&mut substate_store, &mut wasm_engine);
    let _ = test_runner.publish_package("component");
    let component_address =
        ComponentAddress::from_str("0200000000000000000000000000000000000000000000deadbeef")
            .unwrap();

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .call_method(component_address, "get_component_state", to_struct!())
        .build(test_runner.get_nonce([]))
        .sign([]);
    let receipt = test_runner.validate_and_execute(&transaction);

    // Assert
    let error = receipt.result.expect_err("Should be an error.");
    assert_eq!(error, RuntimeError::ComponentNotFound(component_address));
}
