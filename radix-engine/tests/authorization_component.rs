#[rustfmt::skip]
pub mod test_runner;

use crate::test_runner::TestRunner;
use radix_engine::engine::RuntimeError;
use radix_engine::ledger::InMemorySubstateStore;
use radix_engine::wasm::default_wasm_engine;
use scrypto::call_data;
use scrypto::prelude::*;

#[test]
fn cannot_make_cross_component_call_without_authorization() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::new();
    let wasm_engine = default_wasm_engine();
    let mut test_runner = TestRunner::new(&mut substate_store, wasm_engine);
    let (_, _, account) = test_runner.new_account();
    let auth = test_runner.create_non_fungible_resource(account.clone());
    let auth_id = NonFungibleId::from_u32(1);
    let auth_address = NonFungibleAddress::new(auth, auth_id);
    let authorization =
        AccessRules::new().method("get_component_state", rule!(require(auth_address.clone())));

    let package_address = test_runner.publish_package("component");
    let transaction = test_runner
        .new_transaction_builder()
        .call_function(
            package_address,
            "CrossComponent",
            call_data!(create_component_with_auth(authorization)),
        )
        .build(test_runner.get_nonce([]))
        .sign([]);
    let receipt = test_runner.validate_and_execute(&transaction);
    receipt.result.expect("Should be okay");
    let secured_component = receipt.new_component_addresses[0];

    let transaction = test_runner
        .new_transaction_builder()
        .call_function(
            package_address,
            "CrossComponent",
            call_data!(create_component()),
        )
        .build(test_runner.get_nonce([]))
        .sign([]);
    let receipt = test_runner.validate_and_execute(&transaction);
    assert!(receipt.result.is_ok());
    let my_component = receipt.new_component_addresses[0];

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .call_method(
            my_component,
            call_data!(cross_component_call(secured_component)),
        )
        .build(test_runner.get_nonce([]))
        .sign([]);
    let receipt = test_runner.validate_and_execute(&transaction);

    // Assert
    let error = receipt.result.expect_err("Should be error");
    assert_auth_error!(error);
}

#[test]
fn can_make_cross_component_call_with_authorization() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::new();
    let wasm_engine = default_wasm_engine();
    let mut test_runner = TestRunner::new(&mut substate_store, wasm_engine);
    let (key, sk, account) = test_runner.new_account();
    let auth = test_runner.create_non_fungible_resource(account.clone());
    let auth_id = NonFungibleId::from_u32(1);
    let auth_address = NonFungibleAddress::new(auth, auth_id.clone());
    let authorization =
        AccessRules::new().method("get_component_state", rule!(require(auth_address.clone())));

    let package_address = test_runner.publish_package("component");
    let transaction = test_runner
        .new_transaction_builder()
        .call_function(
            package_address,
            "CrossComponent",
            call_data!(create_component_with_auth(authorization)),
        )
        .build(test_runner.get_nonce([]))
        .sign([]);
    let receipt = test_runner.validate_and_execute(&transaction);
    receipt.result.expect("Should be okay");
    let secured_component = receipt.new_component_addresses[0];

    let transaction = test_runner
        .new_transaction_builder()
        .call_function(
            package_address,
            "CrossComponent",
            call_data!(create_component()),
        )
        .build(test_runner.get_nonce([]))
        .sign([]);
    let receipt = test_runner.validate_and_execute(&transaction);
    receipt.result.expect("Should be okay.");
    let my_component = receipt.new_component_addresses[0];

    let transaction = test_runner
        .new_transaction_builder()
        .withdraw_from_account_by_ids(&BTreeSet::from([auth_id.clone()]), auth, account)
        .call_method_with_all_resources(my_component, "put_auth")
        .build(test_runner.get_nonce([key]))
        .sign([&sk]);
    let receipt = test_runner.validate_and_execute(&transaction);
    receipt.result.expect("Should be okay.");

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .call_method(
            my_component,
            call_data!(cross_component_call(secured_component)),
        )
        .build(test_runner.get_nonce([]))
        .sign([]);
    let receipt = test_runner.validate_and_execute(&transaction);

    // Assert
    receipt.result.expect("Should be okay");
}
