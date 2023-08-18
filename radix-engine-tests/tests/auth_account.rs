use radix_engine::errors::{ApplicationError, RuntimeError};
use radix_engine::system::node_module::royalty::ComponentRoyaltyError;
use radix_engine::types::*;
use radix_engine_interface::api::node_modules::auth::AuthAddresses;
use radix_engine_interface::rule;
use scrypto_unit::*;
use transaction::prelude::*;

fn test_auth_rule(
    test_runner: &mut DefaultTestRunner,
    auth_rule: &AccessRule,
    signer_public_keys: &[PublicKey],
    should_succeed: bool,
) {
    // Arrange
    let account = test_runner.new_account_advanced(OwnerRole::Fixed(auth_rule.clone()));
    let (_, _, other_account) = test_runner.new_allocated_account();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .withdraw_from_account(account, XRD, 1)
        .try_deposit_batch_or_abort(other_account, None)
        .build();
    let receipt =
        test_runner.execute_manifest(manifest, AuthAddresses::signer_set(signer_public_keys));

    // Assert
    if should_succeed {
        receipt.expect_commit_success();
    } else {
        receipt.expect_specific_failure(is_auth_error);
    }
}

#[test]
fn can_withdraw_from_my_1_of_2_account_with_either_key_sign() {
    let mut test_runner = TestRunnerBuilder::new().build();
    let (pk0, _, auth0) = test_runner.new_key_pair_with_auth_address();
    let (pk1, _, auth1) = test_runner.new_key_pair_with_auth_address();

    let auths = [
        rule!(require_any_of(vec![auth0.clone(), auth1.clone()])),
        rule!(require(auth0) || require(auth1)),
    ];

    for auth in auths {
        for pk in [pk0, pk1] {
            test_auth_rule(&mut test_runner, &auth, &[pk.into()], true);
        }
    }
}

#[test]
fn can_withdraw_from_my_1_of_3_account_with_either_key_sign() {
    let mut test_runner = TestRunnerBuilder::new().build();
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
        rule!(require(auth0) || (require(auth1) || require(auth2))),
    ];

    for auth in auths {
        for pk in [pk0, pk1, pk2] {
            test_auth_rule(&mut test_runner, &auth, &[pk.into()], true);
        }
    }
}

#[test]
fn can_withdraw_from_my_2_of_2_resource_auth_account_with_both_signatures() {
    let mut test_runner = TestRunnerBuilder::new().build();
    let (pk0, _, auth0) = test_runner.new_key_pair_with_auth_address();
    let (pk1, _, auth1) = test_runner.new_key_pair_with_auth_address();

    let auth = rule!(require_any_of(vec![auth0, auth1,]));

    test_auth_rule(&mut test_runner, &auth, &[pk0.into(), pk1.into()], true);
}

#[test]
fn cannot_withdraw_from_my_2_of_2_account_with_single_signature() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (pk0, _, auth0) = test_runner.new_key_pair_with_auth_address();
    let (_, _, auth1) = test_runner.new_key_pair_with_auth_address();

    let auth = rule!(require_all_of(vec![auth0, auth1]));
    test_auth_rule(&mut test_runner, &auth, &[pk0.into()], false);
}

#[test]
fn can_withdraw_from_my_2_of_3_account_with_2_signatures() {
    let mut test_runner = TestRunnerBuilder::new().build();
    let (_, _, auth0) = test_runner.new_key_pair_with_auth_address();
    let (pk1, _, auth1) = test_runner.new_key_pair_with_auth_address();
    let (pk2, _, auth2) = test_runner.new_key_pair_with_auth_address();
    let auth_2_of_3 = rule!(require_n_of(2, vec![auth0, auth1, auth2]));
    test_auth_rule(
        &mut test_runner,
        &auth_2_of_3,
        &[pk1.into(), pk2.into()],
        true,
    );
}

#[test]
fn can_withdraw_from_my_complex_account() {
    let mut test_runner = TestRunnerBuilder::new().build();
    let (pk0, _, auth0) = test_runner.new_key_pair_with_auth_address();
    let (pk1, _, auth1) = test_runner.new_key_pair_with_auth_address();
    let (pk2, _, auth2) = test_runner.new_key_pair_with_auth_address();
    let auths = [
        rule!(require(auth0.clone()) && require(auth1.clone()) || require(auth2.clone())),
        rule!((require(auth0.clone()) && require(auth1.clone())) || require(auth2.clone())),
        rule!((require(auth0.clone()) && (require(auth1.clone()))) || require(auth2.clone())),
        rule!(require(auth2.clone()) || require(auth0.clone()) && require(auth1.clone())),
        rule!(require(auth2) || (require(auth0) && require(auth1))),
    ];
    let signer_public_keys_list = [
        vec![pk2.into()],
        vec![pk0.into(), pk1.into()],
        vec![pk0.into(), pk1.into(), pk2.into()],
    ];

    for auth in auths {
        for signer_public_keys in &signer_public_keys_list {
            test_auth_rule(&mut test_runner, &auth, &signer_public_keys, true);
        }
    }
}

#[test]
fn cannot_withdraw_from_my_complex_account() {
    let mut test_runner = TestRunnerBuilder::new().build();
    let (pk0, _, auth0) = test_runner.new_key_pair_with_auth_address();
    let (pk1, _, auth1) = test_runner.new_key_pair_with_auth_address();
    let (_, _, auth2) = test_runner.new_key_pair_with_auth_address();
    let auths = [
        rule!(require(auth0.clone()) && require(auth1.clone()) || require(auth2.clone())),
        rule!((require(auth0.clone()) && require(auth1.clone())) || require(auth2.clone())),
        rule!((require(auth0.clone()) && (require(auth1.clone()))) || require(auth2.clone())),
        rule!(require(auth2.clone()) || require(auth0.clone()) && require(auth1.clone())),
        rule!(require(auth2) || (require(auth0) && require(auth1))),
    ];
    let signer_public_keys_list = [vec![pk0.into()], vec![pk1.into()]];

    for auth in auths {
        for signer_public_keys in &signer_public_keys_list {
            test_auth_rule(&mut test_runner, &auth, &signer_public_keys, false);
        }
    }
}

#[test]
fn can_withdraw_from_my_complex_account_2() {
    let mut test_runner = TestRunnerBuilder::new().build();
    let (pk0, _, auth0) = test_runner.new_key_pair_with_auth_address();
    let (pk1, _, auth1) = test_runner.new_key_pair_with_auth_address();
    let (pk2, _, auth2) = test_runner.new_key_pair_with_auth_address();
    let (pk3, _, auth3) = test_runner.new_key_pair_with_auth_address();
    let auths = [
        rule!(
            require(auth0.clone()) && require(auth1.clone()) && require(auth2.clone())
                || require(auth3.clone())
        ),
        rule!((require(auth0) && require(auth1) && require(auth2)) || require(auth3)),
    ];
    let signer_public_keys_list = [vec![pk0.into(), pk1.into(), pk2.into()], vec![pk3.into()]];

    for auth in auths {
        for signer_public_keys in &signer_public_keys_list {
            test_auth_rule(&mut test_runner, &auth, &signer_public_keys, true);
        }
    }
}

#[test]
fn cannot_withdraw_from_my_complex_account_2() {
    let mut test_runner = TestRunnerBuilder::new().build();
    let (pk0, _, auth0) = test_runner.new_key_pair_with_auth_address();
    let (pk1, _, auth1) = test_runner.new_key_pair_with_auth_address();
    let (pk2, _, auth2) = test_runner.new_key_pair_with_auth_address();
    let (_, _, auth3) = test_runner.new_key_pair_with_auth_address();
    let auths = [
        rule!(
            require(auth0.clone()) && require(auth1.clone()) && require(auth2.clone())
                || require(auth3.clone())
        ),
        rule!((require(auth0) && require(auth1) && require(auth2)) || require(auth3)),
    ];
    let signer_public_keys_list = [
        vec![pk0.into()],
        vec![pk1.into()],
        vec![pk2.into()],
        vec![pk0.into(), pk1.into()],
        vec![pk1.into(), pk2.into()],
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
    let mut test_runner = TestRunnerBuilder::new().build();
    let xrd_auth = rule!(require(XRD));
    let account = test_runner.new_account_advanced(OwnerRole::Fixed(xrd_auth));
    let (_, _, other_account) = test_runner.new_allocated_account();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .get_free_xrd_from_faucet()
        .take_all_from_worktop(XRD, "free_xrd")
        .create_proof_from_bucket_of_all("free_xrd", "proof")
        .push_to_auth_zone("proof")
        .withdraw_from_account(account, XRD, 1)
        .pop_from_auth_zone("second_proof")
        .drop_proof("second_proof")
        .return_to_worktop("free_xrd")
        .try_deposit_batch_or_abort(other_account, None)
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn can_withdraw_from_my_any_xrd_auth_account_with_right_amount_of_proof() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let xrd_auth = rule!(require_amount(Decimal(I192::from(1)), XRD));
    let account = test_runner.new_account_advanced(OwnerRole::Fixed(xrd_auth));
    let (_, _, other_account) = test_runner.new_allocated_account();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .get_free_xrd_from_faucet()
        .take_all_from_worktop(XRD, "free_xrd")
        .create_proof_from_bucket_of_all("free_xrd", "proof")
        .push_to_auth_zone("proof")
        .withdraw_from_account(account, XRD, 1)
        .pop_from_auth_zone("proof2")
        .drop_proof("proof2")
        .return_to_worktop("free_xrd")
        .try_deposit_batch_or_abort(other_account, None)
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn cannot_withdraw_from_my_any_xrd_auth_account_with_less_than_amount_of_proof() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let xrd_auth = rule!(require_amount(Decimal::from(2), XRD));
    let account = test_runner.new_account_advanced(OwnerRole::Fixed(xrd_auth));
    let (_, _, other_account) = test_runner.new_allocated_account();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .get_free_xrd_from_faucet()
        .take_from_worktop(XRD, dec!(1), "bucket")
        .create_proof_from_bucket_of_all("bucket", "proof")
        .push_to_auth_zone("proof")
        .withdraw_from_account(account, XRD, 1)
        .pop_from_auth_zone("proof2")
        .drop_proof("proof2")
        .return_to_worktop("bucket")
        .try_deposit_batch_or_abort(other_account, None)
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(is_auth_error)
}

#[test]
fn can_update_updatable_owner_role_account() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let xrd_auth = rule!(require_amount(Decimal(I192::from(1)), XRD));
    let account = test_runner.new_account_advanced(OwnerRole::Updatable(xrd_auth));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .get_free_xrd_from_faucet()
        .take_all_from_worktop(XRD, "bucket")
        .create_proof_from_bucket_of_all("bucket", "proof")
        .push_to_auth_zone("proof")
        .set_owner_role(account, AccessRule::DenyAll)
        .pop_from_auth_zone("proof2")
        .drop_proof("proof2")
        .return_to_worktop("bucket")
        .try_deposit_batch_or_abort(account, None)
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn cannot_set_royalty_on_accounts() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let account = test_runner.new_account_advanced(OwnerRole::Updatable(AccessRule::AllowAll));

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .set_component_royalty(account, "deposit", RoyaltyAmount::Free)
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::ComponentRoyaltyError(
                ComponentRoyaltyError::ComponentRoyaltyIsDisabled
            ))
        )
    });
}
