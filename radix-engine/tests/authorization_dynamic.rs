#[rustfmt::skip]
pub mod test_runner;

use scrypto::call_data;
use crate::test_runner::TestRunner;
use radix_engine::errors::RuntimeError;
use radix_engine::ledger::InMemorySubstateStore;
use scrypto::prelude::*;

fn test_dynamic_auth(
    num_keys: usize,
    initial_auth: usize,
    update_auth: Option<usize>,
    signers: &[usize],
    should_succeed: bool,
) {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let key_and_addresses: Vec<(EcdsaPublicKey, EcdsaPrivateKey, NonFungibleAddress)> = (0
        ..num_keys)
        .map(|_| test_runner.new_key_pair_with_pk_address())
        .collect();
    let addresses: Vec<NonFungibleAddress> = key_and_addresses
        .iter()
        .map(|(_, _, addr)| addr.clone())
        .collect();
    let pks: Vec<EcdsaPublicKey> = signers
        .iter()
        .map(|index| key_and_addresses.get(*index).unwrap().0)
        .collect();
    let sks: Vec<&EcdsaPrivateKey> = signers
        .iter()
        .map(|index| &key_and_addresses.get(*index).unwrap().1)
        .collect();

    let package = test_runner.publish_package("component");
    let transaction1 = test_runner
        .new_transaction_builder()
        .call_function(
            package,
            "AuthComponent",
            call_data![create_component(addresses.get(initial_auth).unwrap().clone())],
        )
        .build(test_runner.get_nonce([]))
        .sign([]);
    let receipt1 = test_runner.validate_and_execute(&transaction1);
    receipt1.result.expect("Should be okay.");
    let component = receipt1.new_component_addresses[0];

    if let Some(next_auth) = update_auth {
        let update_txn = test_runner
            .new_transaction_builder()
            .call_method(
                component,
                call_data![update_auth(addresses.get(next_auth).unwrap().clone())],
            )
            .build(test_runner.get_nonce([]))
            .sign([]);
        test_runner
            .validate_and_execute(&update_txn)
            .result
            .expect("Should be okay.");
    }

    // Act
    let transaction2 = test_runner
        .new_transaction_builder()
        .call_method(component, call_data![get_secret()])
        .build(test_runner.get_nonce(pks))
        .sign(sks);
    let receipt2 = test_runner.validate_and_execute(&transaction2);

    // Assert
    if should_succeed {
        receipt2.result.expect("Should be okay.");
    } else {
        let error = receipt2.result.expect_err("Should be an error.");
        assert_auth_error!(error);
    }
}

fn test_dynamic_authlist(
    list_size: usize,
    auth_rule: AccessRule,
    signers: &[usize],
    should_succeed: bool,
) {
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let key_and_addresses: Vec<(EcdsaPublicKey, EcdsaPrivateKey, NonFungibleAddress)> = (0
        ..list_size)
        .map(|_| test_runner.new_key_pair_with_pk_address())
        .collect();
    let list: Vec<NonFungibleAddress> = key_and_addresses
        .iter()
        .map(|(_, _, addr)| addr.clone())
        .collect();
    let pks: Vec<EcdsaPublicKey> = signers
        .iter()
        .map(|index| key_and_addresses.get(*index).unwrap().0)
        .collect();
    let sks: Vec<&EcdsaPrivateKey> = signers
        .iter()
        .map(|index| &key_and_addresses.get(*index).unwrap().1)
        .collect();
    let authorization = AccessRules::new().method("get_secret", auth_rule);

    // Arrange
    let package = test_runner.publish_package("component");
    let transaction1 = test_runner
        .new_transaction_builder()
        .call_function(
            package,
            "AuthListComponent",
            call_data![create_component(2u8, list, authorization)],
        )
        .build(test_runner.get_nonce([]))
        .sign([]);
    let receipt0 = test_runner.validate_and_execute(&transaction1);
    receipt0.result.expect("Should be okay.");
    let component = receipt0.new_component_addresses[0];

    // Act
    let transaction2 = test_runner
        .new_transaction_builder()
        .call_method(component, call_data!(get_secret()))
        .build(test_runner.get_nonce(pks))
        .sign(sks);
    let receipt = test_runner.validate_and_execute(&transaction2);

    // Assert
    if should_succeed {
        receipt.result.expect("Should be okay.");
    } else {
        let error = receipt.result.expect_err("Should be an error.");
        assert_auth_error!(error);
    }
}

#[test]
fn dynamic_auth_should_allow_me_to_call_method_when_signed() {
    test_dynamic_auth(1, 0, None, &[0], true);
}

#[test]
fn dynamic_auth_should_not_allow_me_to_call_method_when_signed_by_another_key() {
    test_dynamic_auth(2, 0, None, &[1], false);
}

#[test]
fn dynamic_auth_should_not_allow_me_to_call_method_when_change_auth() {
    test_dynamic_auth(2, 0, Some(1), &[0], false);
}

#[test]
fn dynamic_auth_should_allow_me_to_call_method_when_change_auth() {
    test_dynamic_auth(2, 0, Some(1), &[1], true);
}

#[test]
fn dynamic_require_should_fail_on_dynamic_list() {
    test_dynamic_authlist(3, rule!(require("auth")), &[0, 1, 2], false);
}

#[test]
fn dynamic_all_of_should_fail_on_nonexistent_resource() {
    test_dynamic_authlist(3, rule!(require("does_not_exist")), &[0, 1, 2], false);
}

#[test]
fn dynamic_min_n_of_should_allow_me_to_call_method() {
    let auths = [
        rule!(require_n_of(2, "auth")),
        rule!(require_n_of("count", "auth")),
    ];

    for auth in auths {
        test_dynamic_authlist(3, auth, &[0, 1], true);
    }
}

#[test]
fn dynamic_min_n_of_should_fail_if_not_signed_enough() {
    let auths = [
        rule!(require_n_of(2, "auth")),
        rule!(require_n_of("count", "auth")),
    ];

    for auth in auths {
        test_dynamic_authlist(3, auth, &[0], false);
    }
}

#[test]
fn dynamic_min_n_of_should_fail_if_path_does_not_exist() {
    test_dynamic_authlist(3, rule!(require_n_of(1, "does_not_exist")), &[0, 1], false);
}

#[test]
fn dynamic_all_of_should_allow_me_to_call_method() {
    test_dynamic_authlist(3, rule!(require_all_of("auth")), &[0, 1, 2], true);
}

#[test]
fn dynamic_all_of_should_fail_if_not_signed_enough() {
    test_dynamic_authlist(3, rule!(require_all_of("auth")), &[0, 1], false);
}

#[test]
fn dynamic_all_of_should_fail_if_path_does_not_exist() {
    test_dynamic_authlist(3, rule!(require_all_of("does_not_exist")), &[0, 1], false);
}

#[test]
fn dynamic_any_of_should_allow_me_to_call_method() {
    test_dynamic_authlist(3, rule!(require_any_of("auth")), &[1], true);
}

#[test]
fn dynamic_any_of_should_fail_if_path_does_not_exist() {
    test_dynamic_authlist(3, rule!(require_any_of("does_not_exist")), &[0, 1], false);
}

#[test]
fn chess_should_not_allow_second_player_to_move_if_first_player_didnt_move() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (pk, _, _) = test_runner.new_account();
    let (other_pk, other_sk, _) = test_runner.new_account();
    let package = test_runner.publish_package("component");
    let non_fungible_address =
        NonFungibleAddress::new(ECDSA_TOKEN, NonFungibleId::from_bytes(pk.to_vec()));
    let other_non_fungible_address =
        NonFungibleAddress::new(ECDSA_TOKEN, NonFungibleId::from_bytes(other_pk.to_vec()));
    let players = [non_fungible_address, other_non_fungible_address];
    let transaction1 = test_runner
        .new_transaction_builder()
        .call_function(
            package,
            "Chess",
            call_data![create_game(players)],
        )
        .build(test_runner.get_nonce([]))
        .sign([]);
    let receipt1 = test_runner.validate_and_execute(&transaction1);
    receipt1.result.expect("Should be okay.");
    let component = receipt1.new_component_addresses[0];

    // Act
    let transaction2 = test_runner
        .new_transaction_builder()
        .call_method(component, call_data!(make_move()))
        .build(test_runner.get_nonce([other_pk]))
        .sign([&other_sk]);
    let receipt = test_runner.validate_and_execute(&transaction2);

    // Assert
    let error = receipt.result.expect_err("Should be an error");
    assert_auth_error!(error);
}

#[test]
fn chess_should_allow_second_player_to_move_after_first_player() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (pk, sk, _) = test_runner.new_account();
    let (other_pk, other_sk, _) = test_runner.new_account();
    let package = test_runner.publish_package("component");
    let non_fungible_address =
        NonFungibleAddress::new(ECDSA_TOKEN, NonFungibleId::from_bytes(pk.to_vec()));
    let other_non_fungible_address =
        NonFungibleAddress::new(ECDSA_TOKEN, NonFungibleId::from_bytes(other_pk.to_vec()));
    let players = [non_fungible_address, other_non_fungible_address];
    let transaction1 = test_runner
        .new_transaction_builder()
        .call_function(
            package,
            "Chess",
            call_data![create_game(players)],
        )
        .build(test_runner.get_nonce([]))
        .sign([]);
    let receipt1 = test_runner.validate_and_execute(&transaction1);
    receipt1.result.expect("Should be okay.");
    let component = receipt1.new_component_addresses[0];
    let transaction2 = test_runner
        .new_transaction_builder()
        .call_method(component, call_data!(make_move()))
        .build(test_runner.get_nonce([pk]))
        .sign([&sk]);
    test_runner
        .validate_and_execute(&transaction2)
        .result
        .expect("Should be okay.");

    // Act
    let transaction3 = test_runner
        .new_transaction_builder()
        .call_method(component, call_data!(make_move()))
        .build(test_runner.get_nonce([other_pk]))
        .sign([&other_sk]);
    let receipt = test_runner.validate_and_execute(&transaction3);

    // Assert
    receipt.result.expect("Should be okay.");
}
