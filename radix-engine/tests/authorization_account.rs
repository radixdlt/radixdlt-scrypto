extern crate core;

use radix_engine::ledger::{
    ReadableSubstateStore, TypedInMemorySubstateStore, WriteableSubstateStore,
};
use scrypto::core::NetworkDefinition;
use scrypto::prelude::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

fn test_auth_rule<'s, S: ReadableSubstateStore + WriteableSubstateStore>(
    test_runner: &mut TestRunner<'s, S>,
    auth_rule: &AccessRule,
    signer_public_keys: &[EcdsaPublicKey],
    should_succeed: bool,
) {
    // Arrange
    let account = test_runner.new_account_with_auth_rule(auth_rule);
    let (_, _, other_account) = test_runner.new_account();

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::local_simulator())
        .lock_fee(10.into(), SYSTEM_COMPONENT)
        .withdraw_from_account(RADIX_TOKEN, account)
        .call_method_with_all_resources(other_account, "deposit_batch")
        .build();
    let receipt = test_runner.execute_manifest(manifest, signer_public_keys.to_vec());

    // Assert
    if should_succeed {
        receipt.expect_success();
    } else {
        receipt.expect_failure(is_auth_error);
    }
}

#[test]
fn can_withdraw_from_my_1_of_2_account_with_either_key_sign() {
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (pk0, _, auth0) = test_runner.new_key_pair_with_auth_address();
    let (pk1, _, auth1) = test_runner.new_key_pair_with_auth_address();

    let auths = [
        rule!(require_any_of(vec![auth0.clone(), auth1.clone()])),
        rule!(require(auth0) || require(auth1)),
    ];

    for auth in auths {
        for pk in [pk0, pk1] {
            test_auth_rule(&mut test_runner, &auth, &[pk], true);
        }
    }
}

#[test]
fn can_withdraw_from_my_1_of_3_account_with_either_key_sign() {
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (pk0, _, auth0) = test_runner.new_key_pair_with_auth_address();
    let (pk1, _, auth1) = test_runner.new_key_pair_with_auth_address();
    let (pk2, _, auth2) = test_runner.new_key_pair_with_auth_address();
    let auths = [
        rule!(require_any_of(vec![
            auth0.clone(),
            auth1.clone(),
            auth2.clone()
        ])),
        rule!(require(auth0.clone()) || require(auth1.clone()) || require(auth2.clone())),
        rule!((require(auth0.clone()) || require(auth1.clone())) || require(auth2.clone())),
        rule!(require(auth0.clone()) || (require(auth1.clone()) || require(auth2.clone()))),
    ];

    for auth in auths {
        for pk in [pk0, pk1, pk2] {
            test_auth_rule(&mut test_runner, &auth, &[pk], true);
        }
    }
}

#[test]
fn can_withdraw_from_my_2_of_2_resource_auth_account_with_both_signatures() {
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (pk0, _, auth0) = test_runner.new_key_pair_with_auth_address();
    let (pk1, _, auth1) = test_runner.new_key_pair_with_auth_address();

    let auth = rule!(require_any_of(vec![auth0, auth1,]));

    test_auth_rule(&mut test_runner, &auth, &[pk0, pk1], true);
}

#[test]
fn cannot_withdraw_from_my_2_of_2_account_with_single_signature() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (pk0, _, auth0) = test_runner.new_key_pair_with_auth_address();
    let (_, _, auth1) = test_runner.new_key_pair_with_auth_address();

    let auth = rule!(require_all_of(vec![auth0, auth1]));
    test_auth_rule(&mut test_runner, &auth, &[pk0], false);
}

#[test]
fn can_withdraw_from_my_2_of_3_account_with_2_signatures() {
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (_, _, auth0) = test_runner.new_key_pair_with_auth_address();
    let (pk1, _, auth1) = test_runner.new_key_pair_with_auth_address();
    let (pk2, _, auth2) = test_runner.new_key_pair_with_auth_address();
    let auth_2_of_3 = rule!(require_n_of(2, vec![auth0, auth1, auth2]));
    test_auth_rule(&mut test_runner, &auth_2_of_3, &[pk1, pk2], true);
}

#[test]
fn can_withdraw_from_my_complex_account() {
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (pk0, _, auth0) = test_runner.new_key_pair_with_auth_address();
    let (pk1, _, auth1) = test_runner.new_key_pair_with_auth_address();
    let (pk2, _, auth2) = test_runner.new_key_pair_with_auth_address();
    let auths = [
        rule!(require(auth0.clone()) && require(auth1.clone()) || require(auth2.clone())),
        rule!((require(auth0.clone()) && require(auth1.clone())) || require(auth2.clone())),
        rule!((require(auth0.clone()) && (require(auth1.clone()))) || require(auth2.clone())),
        rule!(require(auth2.clone()) || require(auth0.clone()) && require(auth1.clone())),
        rule!(require(auth2.clone()) || (require(auth0.clone()) && require(auth1.clone()))),
    ];
    let signer_public_keys_list = [vec![pk2], vec![pk0, pk1], vec![pk0, pk1, pk2]];

    for auth in auths {
        for signer_public_keys in &signer_public_keys_list {
            test_auth_rule(&mut test_runner, &auth, &signer_public_keys, true);
        }
    }
}

#[test]
fn cannot_withdraw_from_my_complex_account() {
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (pk0, _, auth0) = test_runner.new_key_pair_with_auth_address();
    let (pk1, _, auth1) = test_runner.new_key_pair_with_auth_address();
    let (_, _, auth2) = test_runner.new_key_pair_with_auth_address();
    let auths = [
        rule!(require(auth0.clone()) && require(auth1.clone()) || require(auth2.clone())),
        rule!((require(auth0.clone()) && require(auth1.clone())) || require(auth2.clone())),
        rule!((require(auth0.clone()) && (require(auth1.clone()))) || require(auth2.clone())),
        rule!(require(auth2.clone()) || require(auth0.clone()) && require(auth1.clone())),
        rule!(require(auth2.clone()) || (require(auth0.clone()) && require(auth1.clone()))),
    ];
    let signer_public_keys_list = [vec![pk0], vec![pk1]];

    for auth in auths {
        for signer_public_keys in &signer_public_keys_list {
            test_auth_rule(&mut test_runner, &auth, &signer_public_keys, false);
        }
    }
}

#[test]
fn can_withdraw_from_my_complex_account_2() {
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (pk0, _, auth0) = test_runner.new_key_pair_with_auth_address();
    let (pk1, _, auth1) = test_runner.new_key_pair_with_auth_address();
    let (pk2, _, auth2) = test_runner.new_key_pair_with_auth_address();
    let (pk3, _, auth3) = test_runner.new_key_pair_with_auth_address();
    let auths = [
        rule!(
            require(auth0.clone()) && require(auth1.clone()) && require(auth2.clone())
                || require(auth3.clone())
        ),
        rule!(
            (require(auth0.clone()) && require(auth1.clone()) && require(auth2.clone()))
                || require(auth3.clone())
        ),
    ];
    let signer_public_keys_list = [vec![pk0, pk1, pk2], vec![pk3]];

    for auth in auths {
        for signer_public_keys in &signer_public_keys_list {
            test_auth_rule(&mut test_runner, &auth, &signer_public_keys, true);
        }
    }
}

#[test]
fn cannot_withdraw_from_my_complex_account_2() {
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (pk0, _, auth0) = test_runner.new_key_pair_with_auth_address();
    let (pk1, _, auth1) = test_runner.new_key_pair_with_auth_address();
    let (pk2, _, auth2) = test_runner.new_key_pair_with_auth_address();
    let (_, _, auth3) = test_runner.new_key_pair_with_auth_address();
    let auths = [
        rule!(
            require(auth0.clone()) && require(auth1.clone()) && require(auth2.clone())
                || require(auth3.clone())
        ),
        rule!(
            (require(auth0.clone()) && require(auth1.clone()) && require(auth2.clone()))
                || require(auth3.clone())
        ),
    ];
    let signer_public_keys_list = [
        vec![pk0],
        vec![pk1],
        vec![pk2],
        vec![pk0, pk1],
        vec![pk1, pk2],
    ];

    for auth in auths {
        for signer_public_keys in &signer_public_keys_list {
            test_auth_rule(&mut test_runner, &auth, &signer_public_keys, false);
        }
    }
}

#[test]
fn can_withdraw_from_my_any_xrd_auth_account_with_no_signature() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let xrd_auth = rule!(require(RADIX_TOKEN));
    let account = test_runner.new_account_with_auth_rule(&xrd_auth);
    let (_, _, other_account) = test_runner.new_account();

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::local_simulator())
        .lock_fee(10.into(), SYSTEM_COMPONENT)
        .call_method(SYSTEM_COMPONENT, "free_xrd", args!())
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
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_success();
}

#[test]
fn can_withdraw_from_my_any_xrd_auth_account_with_right_amount_of_proof() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let xrd_auth = rule!(require_amount(Decimal(I256::from(1)), RADIX_TOKEN));
    let account = test_runner.new_account_with_auth_rule(&xrd_auth);
    let (_, _, other_account) = test_runner.new_account();

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::local_simulator())
        .lock_fee(10.into(), SYSTEM_COMPONENT)
        .call_method(SYSTEM_COMPONENT, "free_xrd", args!())
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
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_success();
}

#[test]
fn cannot_withdraw_from_my_any_xrd_auth_account_with_less_than_amount_of_proof() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let xrd_auth = rule!(require_amount(Decimal::from(1), RADIX_TOKEN));
    let account = test_runner.new_account_with_auth_rule(&xrd_auth);
    let (_, _, other_account) = test_runner.new_account();

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::local_simulator())
        .lock_fee(10.into(), SYSTEM_COMPONENT)
        .call_method(SYSTEM_COMPONENT, "free_xrd", args!())
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
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_failure(is_auth_error)
}
