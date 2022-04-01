#[rustfmt::skip]
pub mod test_runner;

use crate::test_runner::TestRunner;
use radix_engine::errors::RuntimeError;
use radix_engine::ledger::InMemorySubstateStore;
use scrypto::prelude::*;

fn test_auth_rule(
    test_runner: &mut TestRunner,
    auth_rule: &AuthRule,
    signers: Vec<EcdsaPublicKey>,
    should_succeed: bool,
) {
    // Arrange
    let account = test_runner.new_account(auth_rule);
    let (_, other_account) = test_runner.new_public_key_with_account();

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .withdraw_from_account(RADIX_TOKEN, account)
        .call_method_with_all_resources(other_account, "deposit_batch")
        .build(signers)
        .unwrap();
    let receipt = test_runner.run(transaction);

    // Assert
    if should_succeed {
        receipt.result.expect("Should be okay");
    } else {
        let error = receipt.result.expect_err("Should be an error");
        assert_eq!(error, RuntimeError::NotAuthorized);
    }
}

#[test]
fn can_withdraw_from_my_1_of_2_account_with_either_key_sign() {
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (key0, auth0) = test_runner.new_public_key_and_non_fungible_address();
    let (key1, auth1) = test_runner.new_public_key_and_non_fungible_address();
    let auth_addresses = vec![auth0.clone(), auth1.clone()];
    let auths = [
        auth2!(require_any_of(auth_addresses)),
        auth2!(require(auth0) || require(auth1)),
    ];

    for auth in auths {
        for key in [key0, key1] {
            test_auth_rule(&mut test_runner, &auth, vec![key], true);
        }
    }
}

#[test]
fn can_withdraw_from_my_1_of_3_account_with_either_key_sign() {
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (key0, auth0) = test_runner.new_public_key_and_non_fungible_address();
    let (key1, auth1) = test_runner.new_public_key_and_non_fungible_address();
    let (key2, auth2) = test_runner.new_public_key_and_non_fungible_address();
    let auths = [
        auth2!(require_any_of(vec![
            auth0.clone(),
            auth1.clone(),
            auth2.clone()
        ])),
        auth2!(require(auth0.clone()) || require(auth1.clone()) || require(auth2.clone())),
        auth2!((require(auth0.clone()) || require(auth1.clone())) || require(auth2.clone())),
        auth2!(require(auth0.clone()) || (require(auth1.clone()) || require(auth2.clone()))),
    ];

    for auth in auths {
        for key in [key0, key1, key2] {
            test_auth_rule(&mut test_runner, &auth, vec![key], true);
        }
    }
}

#[test]
fn can_withdraw_from_my_2_of_2_resource_auth_account_with_both_signatures() {
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (key0, non_fungible_address0) = test_runner.new_public_key_and_non_fungible_address();
    let (key1, non_fungible_address1) = test_runner.new_public_key_and_non_fungible_address();
    let auth_addresses = vec![non_fungible_address0.clone(), non_fungible_address1.clone()];
    let auths = [
        auth2!(require_all_of(auth_addresses)),
        auth2!(require(non_fungible_address0) && require(non_fungible_address1)),
    ];

    for auth in auths {
        test_auth_rule(&mut test_runner, &auth, vec![key0, key1], true);
    }
}

#[test]
fn cannot_withdraw_from_my_2_of_2_account_with_single_signature() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (key0, non_fungible_address0) = test_runner.new_public_key_and_non_fungible_address();
    let (key1, non_fungible_address1) = test_runner.new_public_key_and_non_fungible_address();
    let auth_addresses = vec![non_fungible_address0.clone(), non_fungible_address1.clone()];
    let auths = [
        auth2!(require_all_of(auth_addresses)),
        auth2!(require(non_fungible_address0) && require(non_fungible_address1)),
    ];

    for auth in auths {
        for key in [key0, key1] {
            test_auth_rule(&mut test_runner, &auth, vec![key], false);
        }
    }
}

#[test]
fn can_withdraw_from_my_2_of_3_account_with_2_signatures() {
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (_, non_fungible_address0) = test_runner.new_public_key_and_non_fungible_address();
    let (key1, non_fungible_address1) = test_runner.new_public_key_and_non_fungible_address();
    let (key2, non_fungible_address2) = test_runner.new_public_key_and_non_fungible_address();
    let auth_addresses = vec![
        non_fungible_address0,
        non_fungible_address1,
        non_fungible_address2,
    ];
    let auth_2_of_3 = auth2!(require_n_of(2, auth_addresses));

    test_auth_rule(&mut test_runner, &auth_2_of_3, vec![key1, key2], true);
}

#[test]
fn can_withdraw_from_my_complex_account() {
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (key0, auth0) = test_runner.new_public_key_and_non_fungible_address();
    let (key1, auth1) = test_runner.new_public_key_and_non_fungible_address();
    let (key2, auth2) = test_runner.new_public_key_and_non_fungible_address();
    let auths = [
        auth2!(require(auth0.clone()) && require(auth1.clone()) || require(auth2.clone())),
        auth2!((require(auth0.clone()) && require(auth1.clone())) || require(auth2.clone())),
        auth2!((require(auth0.clone()) && (require(auth1.clone()))) || require(auth2.clone())),
        auth2!(require(auth2.clone()) || require(auth0.clone()) && require(auth1.clone())),
        auth2!(require(auth2.clone()) || (require(auth0.clone()) && require(auth1.clone()))),
    ];
    let signers_list = [vec![key2], vec![key0, key1], vec![key0, key1, key2]];

    for auth in auths {
        for signers in signers_list.clone() {
            test_auth_rule(&mut test_runner, &auth, signers, true);
        }
    }
}

#[test]
fn cannot_withdraw_from_my_complex_account() {
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (key0, auth0) = test_runner.new_public_key_and_non_fungible_address();
    let (key1, auth1) = test_runner.new_public_key_and_non_fungible_address();
    let (key2, auth2) = test_runner.new_public_key_and_non_fungible_address();
    let auths = [
        auth2!(require(auth0.clone()) && require(auth1.clone()) || require(auth2.clone())),
        auth2!((require(auth0.clone()) && require(auth1.clone())) || require(auth2.clone())),
        auth2!((require(auth0.clone()) && (require(auth1.clone()))) || require(auth2.clone())),
        auth2!(require(auth2.clone()) || require(auth0.clone()) && require(auth1.clone())),
        auth2!(require(auth2.clone()) || (require(auth0.clone()) && require(auth1.clone()))),
    ];
    let signers_list = [vec![key0], vec![key1]];

    for auth in auths {
        for signers in signers_list.clone() {
            test_auth_rule(&mut test_runner, &auth, signers, false);
        }
    }
}

#[test]
fn can_withdraw_from_my_any_xrd_auth_account_with_no_signature() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let xrd_auth = auth2!(require(RADIX_TOKEN));
    let account = test_runner.new_account(&xrd_auth);
    let (_, other_account) = test_runner.new_public_key_with_account();

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .call_method(SYSTEM_COMPONENT, "free_xrd", vec![])
        .take_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
            builder.create_proof_from_bucket(bucket_id, |builder, proof_id| {
                builder.move_to_auth_zone(proof_id);
                builder.withdraw_from_account(RADIX_TOKEN, account);
                builder.take_from_auth_zone(|builder, proof_id| builder.drop_proof(proof_id));
                builder
            });
            builder
        })
        .call_method_with_all_resources(other_account, "deposit_batch")
        .build(vec![])
        .unwrap();
    let receipt = test_runner.run(transaction);

    // Assert
    assert!(receipt.result.is_ok());
}

#[test]
fn can_withdraw_from_my_any_xrd_auth_account_with_right_amount_of_proof() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let xrd_auth = auth2!(require_amount(Decimal(1), RADIX_TOKEN));
    let account = test_runner.new_account(&xrd_auth);
    let (_, other_account) = test_runner.new_public_key_with_account();

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .call_method(SYSTEM_COMPONENT, "free_xrd", vec![])
        .take_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
            builder.create_proof_from_bucket(bucket_id, |builder, proof_id| {
                builder.move_to_auth_zone(proof_id);
                builder.withdraw_from_account(RADIX_TOKEN, account);
                builder.take_from_auth_zone(|builder, proof_id| builder.drop_proof(proof_id));
                builder
            });
            builder
        })
        .call_method_with_all_resources(other_account, "deposit_batch")
        .build(vec![])
        .unwrap();
    let receipt = test_runner.run(transaction);

    // Assert
    assert!(receipt.result.is_ok());
}

#[test]
fn cannot_withdraw_from_my_any_xrd_auth_account_with_less_than_amount_of_proof() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let xrd_auth = auth2!(require_amount(Decimal::from(1), RADIX_TOKEN));
    let account = test_runner.new_account(&xrd_auth);
    let (_, other_account) = test_runner.new_public_key_with_account();

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .call_method(SYSTEM_COMPONENT, "free_xrd", vec![])
        .take_from_worktop_by_amount(Decimal::from("0.9"), RADIX_TOKEN, |builder, bucket_id| {
            builder.create_proof_from_bucket(bucket_id, |builder, proof_id| {
                builder.move_to_auth_zone(proof_id);
                builder.withdraw_from_account(RADIX_TOKEN, account);
                builder.take_from_auth_zone(|builder, proof_id| builder.drop_proof(proof_id));
                builder
            });
            builder
        })
        .call_method_with_all_resources(other_account, "deposit_batch")
        .build(vec![])
        .unwrap();
    let receipt = test_runner.run(transaction);

    // Assert
    let error = receipt.result.expect_err("Should be an error");
    assert_eq!(error, RuntimeError::NotAuthorized);
}
