#[rustfmt::skip]
pub mod test_runner;

use crate::test_runner::TestRunner;
use radix_engine::errors::RuntimeError;
use radix_engine::ledger::InMemorySubstateStore;
use scrypto::prelude::*;

fn test_auth_rule(
    test_runner: &mut TestRunner,
    auth_rule: &AuthRule,
    pks: Vec<EcdsaPublicKey>,
    sks: Vec<EcdsaPrivateKey>,
    should_succeed: bool,
) {
    // Arrange
    let account_address = test_runner.new_account_with_auth_rule(auth_rule);
    let (_, _, other_account) = test_runner.new_account();

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .withdraw_from_account(RADIX_TOKEN, account_address)
        .call_method_with_all_resources(other_account, "deposit_batch")
        .build(pks)
        .unwrap()
        .sign(sks);
    let receipt = test_runner.validate_and_execute(&transaction);

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
    let (pk0, sk0, auth0) = test_runner.new_key_pair_with_pk_address();
    let (pk1, sk1, auth1) = test_runner.new_key_pair_with_pk_address();

    let auths = [
        auth!(require_any_of(vec![auth0.clone(), auth1.clone()])),
        auth!(require(auth0) || require(auth1)),
    ];

    for auth in auths {
        for (pk, sk) in [(pk0, sk0), (pk1, sk1)] {
            test_auth_rule(&mut test_runner, &auth, vec![pk], vec![sk], true);
        }
    }
}

#[test]
fn can_withdraw_from_my_1_of_3_account_with_either_key_sign() {
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (pk0, sk0, auth0) = test_runner.new_key_pair_with_pk_address();
    let (pk1, sk1, auth1) = test_runner.new_key_pair_with_pk_address();
    let (pk2, sk2, auth2) = test_runner.new_key_pair_with_pk_address();
    let auths = [
        auth!(require_any_of(vec![
            auth0.clone(),
            auth1.clone(),
            auth2.clone()
        ])),
        auth!(require(auth0.clone()) || require(auth1.clone()) || require(auth2.clone())),
        auth!((require(auth0.clone()) || require(auth1.clone())) || require(auth2.clone())),
        auth!(require(auth0.clone()) || (require(auth1.clone()) || require(auth2.clone()))),
    ];

    for auth in auths {
        for (pk, sk) in [(pk0, sk0), (pk1, sk1), (pk2, sk2)] {
            test_auth_rule(&mut test_runner, &auth, vec![pk], vec![sk], true);
        }
    }
}

#[test]
fn can_withdraw_from_my_2_of_2_resource_auth_account_with_both_signatures() {
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (pk0, sk0, auth0) = test_runner.new_key_pair_with_pk_address();
    let (pk1, sk1, auth1) = test_runner.new_key_pair_with_pk_address();

    let auth = auth!(require_any_of(vec![auth0, auth1,]));

    test_auth_rule(
        &mut test_runner,
        &auth,
        vec![pk0, pk1],
        vec![sk0, sk1],
        true,
    );
}

#[test]
fn cannot_withdraw_from_my_2_of_2_account_with_single_signature() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (pk0, sk0, auth0) = test_runner.new_key_pair_with_pk_address();
    let (_, _, auth1) = test_runner.new_key_pair_with_pk_address();

    let auth = auth!(require_all_of(vec![auth0, auth1]));
    test_auth_rule(&mut test_runner, &auth, vec![pk0], vec![sk0], false);
}

#[test]
fn can_withdraw_from_my_2_of_3_account_with_2_signatures() {
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (_, _, auth0) = test_runner.new_key_pair_with_pk_address();
    let (pk1, sk1, auth1) = test_runner.new_key_pair_with_pk_address();
    let (pk2, sk2, auth2) = test_runner.new_key_pair_with_pk_address();
    let auth_2_of_3 = auth!(require_n_of(2, vec![auth0, auth1, auth2]));
    test_auth_rule(
        &mut test_runner,
        &auth_2_of_3,
        vec![pk1, pk2],
        vec![sk1, sk2],
        true,
    );
}

#[test]
fn can_withdraw_from_my_complex_account() {
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (pk0, sk0, auth0) = test_runner.new_key_pair_with_pk_address();
    let (pk1, sk1, auth1) = test_runner.new_key_pair_with_pk_address();
    let (pk2, sk2, auth2) = test_runner.new_key_pair_with_pk_address();
    let auths = [
        auth!(require(auth0.clone()) && require(auth1.clone()) || require(auth2.clone())),
        auth!((require(auth0.clone()) && require(auth1.clone())) || require(auth2.clone())),
        auth!((require(auth0.clone()) && (require(auth1.clone()))) || require(auth2.clone())),
        auth!(require(auth2.clone()) || require(auth0.clone()) && require(auth1.clone())),
        auth!(require(auth2.clone()) || (require(auth0.clone()) && require(auth1.clone()))),
    ];
    let signers_list = [
        (vec![pk2], vec![sk2]),
        (vec![pk0, pk1], vec![sk0, sk1]),
        (vec![pk0, pk1, pk2], vec![sk0, sk1, sk2]),
    ];

    for auth in auths {
        for signers in signers_list.clone() {
            test_auth_rule(&mut test_runner, &auth, signers.0, signers.1, true);
        }
    }
}

#[test]
fn cannot_withdraw_from_my_complex_account() {
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (pk0, sk0, auth0) = test_runner.new_key_pair_with_pk_address();
    let (pk1, sk1, auth1) = test_runner.new_key_pair_with_pk_address();
    let (_, _, auth2) = test_runner.new_key_pair_with_pk_address();
    let auths = [
        auth!(require(auth0.clone()) && require(auth1.clone()) || require(auth2.clone())),
        auth!((require(auth0.clone()) && require(auth1.clone())) || require(auth2.clone())),
        auth!((require(auth0.clone()) && (require(auth1.clone()))) || require(auth2.clone())),
        auth!(require(auth2.clone()) || require(auth0.clone()) && require(auth1.clone())),
        auth!(require(auth2.clone()) || (require(auth0.clone()) && require(auth1.clone()))),
    ];
    let signers_list = [(vec![pk0], vec![sk0]), (vec![pk1], vec![sk1])];

    for auth in auths {
        for signers in signers_list.clone() {
            test_auth_rule(&mut test_runner, &auth, signers.0, signers.1, false);
        }
    }
}

#[test]
fn can_withdraw_from_my_complex_account_2() {
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (pk0, sk0, auth0) = test_runner.new_key_pair_with_pk_address();
    let (pk1, sk1, auth1) = test_runner.new_key_pair_with_pk_address();
    let (pk2, sk2, auth2) = test_runner.new_key_pair_with_pk_address();
    let (pk3, sk3, auth3) = test_runner.new_key_pair_with_pk_address();
    let auths = [
        auth!(
            require(auth0.clone()) && require(auth1.clone()) && require(auth2.clone())
                || require(auth3.clone())
        ),
        auth!(
            (require(auth0.clone()) && require(auth1.clone()) && require(auth2.clone()))
                || require(auth3.clone())
        ),
    ];
    let signers_list = [
        (vec![pk0, pk1, pk2], vec![sk0, sk1, sk2]),
        (vec![pk3], vec![sk3]),
    ];

    for auth in auths {
        for signers in signers_list.clone() {
            test_auth_rule(&mut test_runner, &auth, signers.0, signers.1, true);
        }
    }
}

#[test]
fn cannot_withdraw_from_my_complex_account_2() {
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (pk0, sk0, auth0) = test_runner.new_key_pair_with_pk_address();
    let (pk1, sk1, auth1) = test_runner.new_key_pair_with_pk_address();
    let (pk2, sk2, auth2) = test_runner.new_key_pair_with_pk_address();
    let (_, _, auth3) = test_runner.new_key_pair_with_pk_address();
    let auths = [
        auth!(
            require(auth0.clone()) && require(auth1.clone()) && require(auth2.clone())
                || require(auth3.clone())
        ),
        auth!(
            (require(auth0.clone()) && require(auth1.clone()) && require(auth2.clone()))
                || require(auth3.clone())
        ),
    ];
    let signers_list = [
        (vec![pk0], vec![sk0]),
        (vec![pk1], vec![sk1]),
        (vec![pk2], vec![sk2]),
        (vec![pk0, pk1], vec![sk0, sk1]),
        (vec![pk1, pk2], vec![sk1, sk2]),
    ];

    for auth in auths {
        for signers in signers_list.clone() {
            test_auth_rule(&mut test_runner, &auth, signers.0, signers.1, false);
        }
    }
}

#[test]
fn can_withdraw_from_my_any_xrd_auth_account_with_no_signature() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let xrd_auth = auth!(require(RADIX_TOKEN));
    let account = test_runner.new_account_with_auth_rule(&xrd_auth);
    let (_, _, other_account) = test_runner.new_account();

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .call_method(SYSTEM_COMPONENT, "free_xrd", vec![])
        .take_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
            builder.create_proof_from_bucket(bucket_id, |builder, proof_id| {
                builder.push_to_auth_zone(proof_id);
                builder.withdraw_from_account(RADIX_TOKEN, account);
                builder.pop_from_auth_zone(|builder, proof_id| builder.drop_proof(proof_id));
                builder
            });
            builder
        })
        .call_method_with_all_resources(other_account, "deposit_batch")
        .build(&[])
        .unwrap()
        .sign(&[]);
    let receipt = test_runner.validate_and_execute(&transaction);

    // Assert
    assert!(receipt.result.is_ok());
}

#[test]
fn can_withdraw_from_my_any_xrd_auth_account_with_right_amount_of_proof() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let xrd_auth = auth!(require_amount(Decimal(1), RADIX_TOKEN));
    let account = test_runner.new_account_with_auth_rule(&xrd_auth);
    let (_, _, other_account) = test_runner.new_account();

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .call_method(SYSTEM_COMPONENT, "free_xrd", vec![])
        .take_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
            builder.create_proof_from_bucket(bucket_id, |builder, proof_id| {
                builder.push_to_auth_zone(proof_id);
                builder.withdraw_from_account(RADIX_TOKEN, account);
                builder.pop_from_auth_zone(|builder, proof_id| builder.drop_proof(proof_id));
                builder
            });
            builder
        })
        .call_method_with_all_resources(other_account, "deposit_batch")
        .build(&[])
        .unwrap()
        .sign(&[]);
    let receipt = test_runner.validate_and_execute(&transaction);

    // Assert
    assert!(receipt.result.is_ok());
}

#[test]
fn cannot_withdraw_from_my_any_xrd_auth_account_with_less_than_amount_of_proof() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let xrd_auth = auth!(require_amount(Decimal::from(1), RADIX_TOKEN));
    let account = test_runner.new_account_with_auth_rule(&xrd_auth);
    let (_, _, other_account) = test_runner.new_account();

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .call_method(SYSTEM_COMPONENT, "free_xrd", vec![])
        .take_from_worktop_by_amount(Decimal::from("0.9"), RADIX_TOKEN, |builder, bucket_id| {
            builder.create_proof_from_bucket(bucket_id, |builder, proof_id| {
                builder.push_to_auth_zone(proof_id);
                builder.withdraw_from_account(RADIX_TOKEN, account);
                builder.pop_from_auth_zone(|builder, proof_id| builder.drop_proof(proof_id));
                builder
            });
            builder
        })
        .call_method_with_all_resources(other_account, "deposit_batch")
        .build(&[])
        .unwrap()
        .sign(&[]);
    let receipt = test_runner.validate_and_execute(&transaction);

    // Assert
    let error = receipt.result.expect_err("Should be an error");
    assert_eq!(error, RuntimeError::NotAuthorized);
}
