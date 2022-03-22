#[rustfmt::skip]
pub mod test_runner;

use crate::test_runner::TestRunner;
use radix_engine::errors::RuntimeError;
use radix_engine::ledger::{InMemorySubstateStore, SubstateStore};
use radix_engine::model::{Component, MethodAuthorization};
use radix_engine::transaction::*;
use scrypto::{any_of, component_authorization};
use scrypto::prelude::*;

#[test]
fn cannot_make_cross_component_call_without_authorization() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (_, account) = test_runner.new_public_key_with_account();
    let auth = test_runner.create_non_fungible_resource(account.clone());
    let auth_id = NonFungibleId::from(1);
    let auth_address = NonFungibleAddress::new(auth, auth_id);
    let method_authorization = component_authorization! {
        "get_component_state" => any_of!(auth_address.clone())
    };

    let package_id = test_runner.publish_package("component");
    let transaction = test_runner
        .new_transaction_builder()
        .call_function(
            package_id,
            "CrossComponent",
            "create_component_with_auth",
            vec![scrypto_encode(&method_authorization)],
        )
        .build(vec![])
        .unwrap();
    let receipt = test_runner.run(transaction);
    assert!(receipt.result.is_ok());
    let secured_component = receipt.new_component_ids[0];

    let transaction = test_runner
        .new_transaction_builder()
        .call_function(package_id, "CrossComponent", "create_component", vec![])
        .build(vec![])
        .unwrap();
    let receipt = test_runner.run(transaction);
    assert!(receipt.result.is_ok());
    let my_component = receipt.new_component_ids[0];

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .call_method(
            my_component,
            "cross_component_call",
            vec![scrypto_encode(&secured_component)],
        )
        .build(vec![])
        .unwrap();
    let receipt = test_runner.run(transaction);

    // Assert
    let runtime_error = receipt.result.expect_err("Should be error");
    assert_eq!(runtime_error, RuntimeError::NotAuthorized);
    let component_state: Component = substate_store
        .get_decoded_substate(&secured_component)
        .map(|(c, _)| c)
        .unwrap();
    assert_eq!(
        component_state.initialize_method("get_component_state").1,
        MethodAuthorization::Protected(auth_address.into())
    );
}

#[test]
fn can_make_cross_component_call_with_authorization() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (_, account) = test_runner.new_public_key_with_account();
    let auth = test_runner.create_non_fungible_resource(account.clone());
    let auth_id = NonFungibleId::from(1);
    let auth_address = NonFungibleAddress::new(auth, auth_id.clone());
    let method_authorization = component_authorization! {
        "get_component_state" => any_of!(auth_address.clone())
    };

    let package_id = test_runner.publish_package("component");
    let transaction = test_runner
        .new_transaction_builder()
        .call_function(
            package_id,
            "CrossComponent",
            "create_component_with_auth",
            vec![scrypto_encode(&method_authorization)],
        )
        .build(vec![])
        .unwrap();
    let receipt = test_runner.run(transaction);
    assert!(receipt.result.is_ok());
    let secured_component = receipt.new_component_ids[0];

    let transaction = test_runner
        .new_transaction_builder()
        .call_function(package_id, "CrossComponent", "create_component", vec![])
        .build(vec![])
        .unwrap();
    let receipt = test_runner.run(transaction);
    assert!(receipt.result.is_ok());
    let my_component = receipt.new_component_ids[0];

    let auth_amount = ResourceSpecifier::Ids(BTreeSet::from([auth_id.clone()]), auth);
    let transaction = test_runner
        .new_transaction_builder()
        .withdraw_from_account(&auth_amount, account)
        .call_method_with_all_resources(my_component, "put_auth")
        .build(vec![])
        .unwrap();
    let receipt = test_runner.run(transaction);
    assert!(receipt.result.is_ok());

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .call_method(
            my_component,
            "cross_component_call",
            vec![scrypto_encode(&secured_component)],
        )
        .build(vec![])
        .unwrap();
    let receipt = test_runner.run(transaction);

    // Assert
    assert!(receipt.result.is_ok());
    let component_state: Component = substate_store
        .get_decoded_substate(&secured_component)
        .map(|(c, _)| c)
        .unwrap();
    assert_eq!(
        component_state.initialize_method("get_component_state").1,
        MethodAuthorization::Protected(auth_address.into())
    );
}
