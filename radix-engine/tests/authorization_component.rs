#[rustfmt::skip]
pub mod test_runner;

use crate::test_runner::TestRunner;
use radix_engine::errors::RuntimeError;
use radix_engine::ledger::InMemorySubstateStore;
use scrypto::prelude::*;

#[test]
fn cannot_make_cross_component_call_without_authorization() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
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
            "create_component_with_auth",
            vec![scrypto_encode(&authorization)],
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
            "create_component",
            vec![],
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
            "cross_component_call",
            vec![scrypto_encode(&secured_component)],
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
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
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
            "create_component_with_auth",
            vec![scrypto_encode(&authorization)],
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
            "create_component",
            vec![],
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
            "cross_component_call",
            vec![scrypto_encode(&secured_component)],
        )
        .build(test_runner.get_nonce([]))
        .sign([]);
    let receipt = test_runner.validate_and_execute(&transaction);

    // Assert
    receipt.result.expect("Should be okay");
}
