#[rustfmt::skip]
pub mod test_runner;

use crate::test_runner::TestRunner;
use radix_engine::ledger::InMemorySubstateStore;
use scrypto::prelude::*;
use radix_engine::errors::RuntimeError;

#[test]
fn dynamic_auth_should_allow_me_to_call_method_when_signed() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (key, _) = test_runner.new_public_key_with_account();
    let package = test_runner.publish_package("component");
    let non_fungible_address = NonFungibleAddress::new(ECDSA_TOKEN, NonFungibleId::new(key.to_vec()));
    let transaction1 = test_runner
        .new_transaction_builder()
        .call_function(package, "AuthComponent", "create_component", vec![scrypto_encode(&non_fungible_address)])
        .build(vec![])
        .unwrap();
    let receipt1 = test_runner.run(transaction1);
    receipt1.result.expect("Should be okay.");
    let component = receipt1.new_component_ids[0];

    // Act
    let transaction2 = test_runner
        .new_transaction_builder()
        .call_method(component, "get_secret", vec![])
        .build(vec![key])
        .unwrap();
    let receipt2 = test_runner.run(transaction2);

    // Assert
    receipt2.result.expect("Should be okay.");
}

#[test]
fn dynamic_auth_should_not_allow_me_to_call_method_when_signed_by_another_key() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (key, _) = test_runner.new_public_key_with_account();
    let (other_key, _) = test_runner.new_public_key_with_account();
    let package = test_runner.publish_package("component");
    let non_fungible_address = NonFungibleAddress::new(ECDSA_TOKEN, NonFungibleId::new(key.to_vec()));
    let transaction1 = test_runner
        .new_transaction_builder()
        .call_function(package, "AuthComponent", "create_component", vec![scrypto_encode(&non_fungible_address)])
        .build(vec![])
        .unwrap();
    let receipt1 = test_runner.run(transaction1);
    receipt1.result.expect("Should be okay.");
    let component = receipt1.new_component_ids[0];

    // Act
    let transaction2 = test_runner
        .new_transaction_builder()
        .call_method(component, "get_secret", vec![])
        .build(vec![other_key])
        .unwrap();
    let receipt2 = test_runner.run(transaction2);

    // Assert
    let error = receipt2.result.expect_err("Should be an error");
    assert_eq!(error, RuntimeError::NotAuthorized);
}

#[test]
fn dynamic_auth_should_not_allow_me_to_call_method_when_change_auth() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (key, _) = test_runner.new_public_key_with_account();
    let (other_key, _) = test_runner.new_public_key_with_account();
    let package = test_runner.publish_package("component");
    let non_fungible_address = NonFungibleAddress::new(ECDSA_TOKEN, NonFungibleId::new(key.to_vec()));
    let other_non_fungible_address = NonFungibleAddress::new(ECDSA_TOKEN, NonFungibleId::new(other_key.to_vec()));
    let transaction1 = test_runner
        .new_transaction_builder()
        .call_function(package, "AuthComponent", "create_component", vec![scrypto_encode(&non_fungible_address)])
        .build(vec![])
        .unwrap();
    let receipt1 = test_runner.run(transaction1);
    receipt1.result.expect("Should be okay.");
    let component = receipt1.new_component_ids[0];

    // Act
    let transaction2 = test_runner
        .new_transaction_builder()
        .call_method(component, "get_secret", vec![])
        .call_method(component, "update_auth", vec![scrypto_encode(&other_non_fungible_address)])
        .call_method(component, "get_secret", vec![])
        .build(vec![key])
        .unwrap();
    let receipt2 = test_runner.run(transaction2);

    // Assert
    let error = receipt2.result.expect_err("Should be an error");
    assert_eq!(error, RuntimeError::NotAuthorized);
}

#[test]
fn dynamic_auth_should_allow_me_to_call_method_when_change_auth() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (key, _) = test_runner.new_public_key_with_account();
    let (other_key, _) = test_runner.new_public_key_with_account();
    let package = test_runner.publish_package("component");
    let non_fungible_address = NonFungibleAddress::new(ECDSA_TOKEN, NonFungibleId::new(key.to_vec()));
    let other_non_fungible_address = NonFungibleAddress::new(ECDSA_TOKEN, NonFungibleId::new(other_key.to_vec()));
    let transaction1 = test_runner
        .new_transaction_builder()
        .call_function(package, "AuthComponent", "create_component", vec![scrypto_encode(&non_fungible_address)])
        .build(vec![])
        .unwrap();
    let receipt0 = test_runner.run(transaction1);
    receipt0.result.expect("Should be okay.");
    let component = receipt0.new_component_ids[0];
    let transaction2 = test_runner
        .new_transaction_builder()
        .call_method(component, "update_auth", vec![scrypto_encode(&other_non_fungible_address)])
        .build(vec![key])
        .unwrap();
    test_runner.run(transaction2).result.expect("Should be okay.");

    // Act
    let transaction3 = test_runner
        .new_transaction_builder()
        .call_method(component, "get_secret", vec![])
        .build(vec![other_key])
        .unwrap();
    let receipt = test_runner.run(transaction3);

    // Assert
    receipt.result.expect("Should be okay.");
}

#[test]
fn chess_should_not_allow_second_player_to_move_if_first_player_didnt_move() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (key, _) = test_runner.new_public_key_with_account();
    let (other_key, _) = test_runner.new_public_key_with_account();
    let package = test_runner.publish_package("component");
    let non_fungible_address = NonFungibleAddress::new(ECDSA_TOKEN, NonFungibleId::new(key.to_vec()));
    let other_non_fungible_address = NonFungibleAddress::new(ECDSA_TOKEN, NonFungibleId::new(other_key.to_vec()));
    let players = [non_fungible_address, other_non_fungible_address];
    let transaction1 = test_runner
        .new_transaction_builder()
        .call_function(package, "Chess", "create_game", vec![scrypto_encode(&players)])
        .build(vec![])
        .unwrap();
    let receipt1 = test_runner.run(transaction1);
    receipt1.result.expect("Should be okay.");
    let component = receipt1.new_component_ids[0];

    // Act
    let transaction2 = test_runner
        .new_transaction_builder()
        .call_method(component, "make_move", vec![])
        .build(vec![other_key])
        .unwrap();
    let receipt = test_runner.run(transaction2);

    // Assert
    let error = receipt.result.expect_err("Should be an error");
    assert_eq!(error, RuntimeError::NotAuthorized);
}

#[test]
fn chess_should_allow_second_player_to_move_after_first_player() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (key, _) = test_runner.new_public_key_with_account();
    let (other_key, _) = test_runner.new_public_key_with_account();
    let package = test_runner.publish_package("component");
    let non_fungible_address = NonFungibleAddress::new(ECDSA_TOKEN, NonFungibleId::new(key.to_vec()));
    let other_non_fungible_address = NonFungibleAddress::new(ECDSA_TOKEN, NonFungibleId::new(other_key.to_vec()));
    let players = [non_fungible_address, other_non_fungible_address];
    let transaction1 = test_runner
        .new_transaction_builder()
        .call_function(package, "Chess", "create_game", vec![scrypto_encode(&players)])
        .build(vec![])
        .unwrap();
    let receipt1 = test_runner.run(transaction1);
    receipt1.result.expect("Should be okay.");
    let component = receipt1.new_component_ids[0];
    let transaction2 = test_runner
        .new_transaction_builder()
        .call_method(component, "make_move", vec![])
        .build(vec![key])
        .unwrap();
    test_runner.run(transaction2).result.expect("Should be okay.");

    // Act
    let transaction3 = test_runner
        .new_transaction_builder()
        .call_method(component, "make_move", vec![])
        .build(vec![other_key])
        .unwrap();
    let receipt = test_runner.run(transaction3);

    // Assert
    receipt.result.expect("Should be okay.");
}