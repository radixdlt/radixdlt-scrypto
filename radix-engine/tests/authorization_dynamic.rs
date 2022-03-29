#[rustfmt::skip]
pub mod test_runner;

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
    let key_and_addresses: Vec<(EcdsaPublicKey, NonFungibleAddress)> = (0..num_keys)
        .map(|_| test_runner.new_public_key_and_non_fungible_address())
        .collect();
    let addresses: Vec<NonFungibleAddress> = key_and_addresses
        .iter()
        .map(|(_, addr)| addr.clone())
        .collect();
    let key_signers = signers
        .iter()
        .map(|index| key_and_addresses.get(*index).unwrap().0)
        .collect();

    let package = test_runner.publish_package("component");
    let transaction1 = test_runner
        .new_transaction_builder()
        .call_function(
            package,
            "AuthComponent",
            "create_component",
            vec![scrypto_encode(addresses.get(initial_auth).unwrap())],
        )
        .build(vec![])
        .unwrap();
    let receipt1 = test_runner.run(transaction1);
    receipt1.result.expect("Should be okay.");
    let component = receipt1.new_component_ids[0];

    if let Some(next_auth) = update_auth {
        let update_txn = test_runner
            .new_transaction_builder()
            .call_method(
                component,
                "update_auth",
                vec![scrypto_encode(addresses.get(next_auth).unwrap())],
            )
            .build(vec![])
            .unwrap();
        test_runner.run(update_txn).result.expect("Should be okay.");
    }

    // Act
    let transaction2 = test_runner
        .new_transaction_builder()
        .call_method(component, "get_secret", vec![])
        .build(key_signers)
        .unwrap();
    let receipt2 = test_runner.run(transaction2);

    // Assert
    if should_succeed {
        receipt2.result.expect("Should be okay.");
    } else {
        let error = receipt2.result.expect_err("Should be an error.");
        assert_eq!(error, RuntimeError::NotAuthorized);
    }
}

fn test_dynamic_authlist(
    list_size: usize,
    rule: ProofRule,
    signers: &[usize],
    should_succeed: bool,
) {
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let key_and_addresses: Vec<(EcdsaPublicKey, NonFungibleAddress)> = (0..list_size)
        .map(|_| test_runner.new_public_key_and_non_fungible_address())
        .collect();
    let list: Vec<NonFungibleAddress> = key_and_addresses
        .iter()
        .map(|(_, addr)| addr.clone())
        .collect();
    let key_signers = signers
        .iter()
        .map(|index| key_and_addresses.get(*index).unwrap().0)
        .collect();
    let authorization = component_authorization! {
        "get_secret" => rule
    };

    // Arrange
    let package = test_runner.publish_package("component");
    let transaction1 = test_runner
        .new_transaction_builder()
        .call_function(
            package,
            "AuthListComponent",
            "create_component",
            args!(list, authorization),
        )
        .build(vec![])
        .unwrap();
    let receipt0 = test_runner.run(transaction1);
    receipt0.result.expect("Should be okay.");
    let component = receipt0.new_component_ids[0];

    // Act
    let transaction2 = test_runner
        .new_transaction_builder()
        .call_method(component, "get_secret", vec![])
        .build(key_signers)
        .unwrap();
    let receipt = test_runner.run(transaction2);

    // Assert
    if should_succeed {
        receipt.result.expect("Should be okay.");
    } else {
        let error = receipt.result.expect_err("Should be an error.");
        assert_eq!(error, RuntimeError::NotAuthorized);
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
fn dynamic_this_should_fail_on_dynamic_list() {
    test_dynamic_authlist(
        3,
        this!(SchemaPath::new().field("auth")),
        &[0, 1, 2],
        false
    );
}

#[test]
fn dynamic_all_of_should_fail_on_nonexistent_resource() {
    test_dynamic_authlist(
        3,
        all_of!(resource_list!(SchemaPath::new().field("does_not_exist"))),
        &[0, 1, 2],
        false,
    );
}

#[test]
fn dynamic_min_n_of_should_allow_me_to_call_method() {
    test_dynamic_authlist(
        3,
        min_n_of!(2, SchemaPath::new().field("auth")),
        &[0, 1],
        true,
    );
}

#[test]
fn dynamic_min_n_of_should_fail_if_not_signed_enough() {
    test_dynamic_authlist(
        3,
        min_n_of!(2, SchemaPath::new().field("auth")),
        &[0],
        false,
    );
}

#[test]
fn dynamic_min_n_of_should_fail_if_path_does_not_exist() {
    test_dynamic_authlist(
        3,
        min_n_of!(1, SchemaPath::new().field("does_not_exist")),
        &[0, 1],
        false,
    );
}

#[test]
fn dynamic_all_of_should_allow_me_to_call_method() {
    test_dynamic_authlist(
        3,
        all_of!(SchemaPath::new().field("auth")),
        &[0, 1, 2],
        true,
    );
}

#[test]
fn dynamic_all_of_should_fail_if_not_signed_enough() {
    test_dynamic_authlist(3, all_of!(SchemaPath::new().field("auth")), &[0, 1], false);
}

#[test]
fn dynamic_all_of_should_fail_if_path_does_not_exist() {
    test_dynamic_authlist(3, all_of!(SchemaPath::new().field("does_not_exist")), &[0, 1], false);
}

#[test]
fn dynamic_any_of_should_allow_me_to_call_method() {
    test_dynamic_authlist(3, any_of!(SchemaPath::new().field("auth")), &[1], true);
}

#[test]
fn dynamic_any_of_should_fail_if_path_does_not_exist() {
    test_dynamic_authlist(3, any_of!(SchemaPath::new().field("does_not_exist")), &[0, 1], false);
}

#[test]
fn chess_should_not_allow_second_player_to_move_if_first_player_didnt_move() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (key, _) = test_runner.new_public_key_with_account();
    let (other_key, _) = test_runner.new_public_key_with_account();
    let package = test_runner.publish_package("component");
    let non_fungible_address =
        NonFungibleAddress::new(ECDSA_TOKEN, NonFungibleId::new(key.to_vec()));
    let other_non_fungible_address =
        NonFungibleAddress::new(ECDSA_TOKEN, NonFungibleId::new(other_key.to_vec()));
    let players = [non_fungible_address, other_non_fungible_address];
    let transaction1 = test_runner
        .new_transaction_builder()
        .call_function(
            package,
            "Chess",
            "create_game",
            vec![scrypto_encode(&players)],
        )
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
    let non_fungible_address =
        NonFungibleAddress::new(ECDSA_TOKEN, NonFungibleId::new(key.to_vec()));
    let other_non_fungible_address =
        NonFungibleAddress::new(ECDSA_TOKEN, NonFungibleId::new(other_key.to_vec()));
    let players = [non_fungible_address, other_non_fungible_address];
    let transaction1 = test_runner
        .new_transaction_builder()
        .call_function(
            package,
            "Chess",
            "create_game",
            vec![scrypto_encode(&players)],
        )
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
    test_runner
        .run(transaction2)
        .result
        .expect("Should be okay.");

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
