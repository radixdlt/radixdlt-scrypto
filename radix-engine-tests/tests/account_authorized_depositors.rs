use radix_engine::blueprints::account::AccountError;
use radix_engine::errors::{ApplicationError, RuntimeError};
use radix_engine::transaction::TransactionReceipt;
use radix_engine::types::*;
use radix_engine_interface::blueprints::account::*;
use scrypto_unit::TestRunnerBuilder;
use transaction::prelude::*;

#[test]
fn account_add_authorized_depositor_without_owner_auth_fails() {
    test_depositors_operation_method_auth(
        DepositorsOperation::Add {
            badge: ResourceOrNonFungible::Resource(XRD),
        },
        false,
        |receipt| receipt.expect_auth_failure(),
    )
}

#[test]
fn account_add_authorized_depositor_with_owner_auth_succeeds() {
    test_depositors_operation_method_auth(
        DepositorsOperation::Add {
            badge: ResourceOrNonFungible::Resource(XRD),
        },
        true,
        |receipt| {
            receipt.expect_commit_success();
        },
    )
}

#[test]
fn account_remove_authorized_depositor_without_owner_auth_fails() {
    test_depositors_operation_method_auth(
        DepositorsOperation::Remove {
            badge: ResourceOrNonFungible::Resource(XRD),
        },
        false,
        |receipt| receipt.expect_auth_failure(),
    )
}

#[test]
fn account_remove_authorized_depositor_with_owner_auth_succeeds() {
    test_depositors_operation_method_auth(
        DepositorsOperation::Remove {
            badge: ResourceOrNonFungible::Resource(XRD),
        },
        true,
        |receipt| {
            receipt.expect_commit_success();
        },
    )
}

#[test]
fn try_authorized_deposit_or_refund_performs_a_refund_when_badge_is_not_in_depositors_list() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (pk1, _, account1) = test_runner.new_account(true);
    let (pk2, _, account2) = test_runner.new_account(true);

    let badge = ResourceOrNonFungible::Resource(XRD);
    test_runner
        .execute_manifest_ignoring_fee(
            ManifestBuilder::new()
                .call_method(
                    account1,
                    ACCOUNT_SET_DEFAULT_DEPOSIT_RULE_IDENT,
                    AccountSetDefaultDepositRuleInput {
                        default: DefaultDepositRule::Reject,
                    },
                )
                .call_method(
                    account1,
                    ACCOUNT_ADD_AUTHORIZED_DEPOSITOR,
                    AccountAddAuthorizedDepositorInput {
                        badge: badge.clone(),
                    },
                )
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&pk1)],
        )
        .expect_commit_success();

    // Act
    let manifest = ManifestBuilder::new()
        .withdraw_from_account(account2, XRD, 1)
        .take_all_from_worktop(XRD, "bucket")
        .with_bucket("bucket", |builder, bucket| {
            builder.call_method(
                account1,
                ACCOUNT_TRY_DEPOSIT_OR_REFUND_IDENT,
                AccountTryDepositOrRefundManifestInput {
                    bucket,
                    authorized_depositor_badge: Some(ResourceOrNonFungible::Resource(
                        VALIDATOR_OWNER_BADGE,
                    )),
                },
            )
        })
        .build();
    let receipt = test_runner
        .execute_manifest_ignoring_fee(manifest, vec![NonFungibleGlobalId::from_public_key(&pk2)]);

    // Assert
    receipt.expect_specific_failure(is_account_not_an_authorized_depositor_error);
}

#[test]
fn try_authorized_deposit_or_refund_panics_when_badge_is_in_depositors_list_but_is_not_in_auth_zone(
) {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (pk1, _, account1) = test_runner.new_account(true);
    let (pk2, _, account2) = test_runner.new_account(true);

    let badge = ResourceOrNonFungible::Resource(XRD);
    test_runner
        .execute_manifest_ignoring_fee(
            ManifestBuilder::new()
                .call_method(
                    account1,
                    ACCOUNT_SET_DEFAULT_DEPOSIT_RULE_IDENT,
                    AccountSetDefaultDepositRuleInput {
                        default: DefaultDepositRule::Reject,
                    },
                )
                .call_method(
                    account1,
                    ACCOUNT_ADD_AUTHORIZED_DEPOSITOR,
                    AccountAddAuthorizedDepositorInput {
                        badge: badge.clone(),
                    },
                )
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&pk1)],
        )
        .expect_commit_success();

    // Act
    let manifest = ManifestBuilder::new()
        .withdraw_from_account(account2, XRD, 1)
        .take_all_from_worktop(XRD, "bucket")
        .with_bucket("bucket", |builder, bucket| {
            builder.call_method(
                account1,
                ACCOUNT_TRY_DEPOSIT_OR_REFUND_IDENT,
                AccountTryDepositOrRefundManifestInput {
                    bucket,
                    authorized_depositor_badge: Some(badge),
                },
            )
        })
        .build();
    let receipt = test_runner
        .execute_manifest_ignoring_fee(manifest, vec![NonFungibleGlobalId::from_public_key(&pk2)]);

    // Assert
    receipt.expect_auth_assertion_failure();
}

#[test]
fn try_authorized_deposit_or_refund_accepts_deposit_when_depositor_is_authorized_and_badge_is_in_auth_zone(
) {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (pk1, _, account1) = test_runner.new_account(true);
    let (pk2, _, account2) = test_runner.new_account(true);

    let badge = ResourceOrNonFungible::Resource(XRD);
    test_runner
        .execute_manifest_ignoring_fee(
            ManifestBuilder::new()
                .call_method(
                    account1,
                    ACCOUNT_SET_DEFAULT_DEPOSIT_RULE_IDENT,
                    AccountSetDefaultDepositRuleInput {
                        default: DefaultDepositRule::Reject,
                    },
                )
                .call_method(
                    account1,
                    ACCOUNT_ADD_AUTHORIZED_DEPOSITOR,
                    AccountAddAuthorizedDepositorInput {
                        badge: badge.clone(),
                    },
                )
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&pk1)],
        )
        .expect_commit_success();

    // Act
    let manifest = ManifestBuilder::new()
        .create_proof_from_account_of_amount(account2, XRD, 1)
        .withdraw_from_account(account2, XRD, 1)
        .take_all_from_worktop(XRD, "bucket")
        .with_bucket("bucket", |builder, bucket| {
            builder.call_method(
                account1,
                ACCOUNT_TRY_DEPOSIT_OR_REFUND_IDENT,
                AccountTryDepositOrRefundManifestInput {
                    bucket,
                    authorized_depositor_badge: Some(badge),
                },
            )
        })
        .build();
    let receipt = test_runner
        .execute_manifest_ignoring_fee(manifest, vec![NonFungibleGlobalId::from_public_key(&pk2)]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn authorized_depositor_can_be_removed_later() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (pk1, _, account1) = test_runner.new_account(true);
    let (pk2, _, account2) = test_runner.new_account(true);

    let badge = ResourceOrNonFungible::Resource(XRD);
    test_runner
        .execute_manifest_ignoring_fee(
            ManifestBuilder::new()
                .call_method(
                    account1,
                    ACCOUNT_SET_DEFAULT_DEPOSIT_RULE_IDENT,
                    AccountSetDefaultDepositRuleInput {
                        default: DefaultDepositRule::Reject,
                    },
                )
                .call_method(
                    account1,
                    ACCOUNT_ADD_AUTHORIZED_DEPOSITOR,
                    AccountAddAuthorizedDepositorInput {
                        badge: badge.clone(),
                    },
                )
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&pk1)],
        )
        .expect_commit_success();

    // Act 1
    let manifest = ManifestBuilder::new()
        .create_proof_from_account_of_amount(account2, XRD, 1)
        .withdraw_from_account(account2, XRD, 1)
        .take_all_from_worktop(XRD, "bucket")
        .with_bucket("bucket", |builder, bucket| {
            builder.call_method(
                account1,
                ACCOUNT_TRY_DEPOSIT_OR_REFUND_IDENT,
                AccountTryDepositOrRefundManifestInput {
                    bucket,
                    authorized_depositor_badge: Some(badge.clone()),
                },
            )
        })
        .build();
    let receipt = test_runner
        .execute_manifest_ignoring_fee(manifest, vec![NonFungibleGlobalId::from_public_key(&pk2)]);

    // Assert 1
    receipt.expect_commit_success();

    // Act 2
    let manifest = ManifestBuilder::new()
        .call_method(
            account1,
            ACCOUNT_REMOVE_AUTHORIZED_DEPOSITOR,
            AccountRemoveAuthorizedDepositorInput {
                badge: badge.clone(),
            },
        )
        .build();
    test_runner
        .execute_manifest_ignoring_fee(manifest, vec![NonFungibleGlobalId::from_public_key(&pk1)])
        .expect_commit_success();

    let manifest = ManifestBuilder::new()
        .create_proof_from_account_of_amount(account2, XRD, 1)
        .withdraw_from_account(account2, XRD, 1)
        .take_all_from_worktop(XRD, "bucket")
        .with_bucket("bucket", |builder, bucket| {
            builder.call_method(
                account1,
                ACCOUNT_TRY_DEPOSIT_OR_REFUND_IDENT,
                AccountTryDepositOrRefundManifestInput {
                    bucket,
                    authorized_depositor_badge: Some(badge),
                },
            )
        })
        .build();
    let receipt = test_runner
        .execute_manifest_ignoring_fee(manifest, vec![NonFungibleGlobalId::from_public_key(&pk2)]);

    // Assert 2
    receipt.expect_specific_failure(is_account_not_an_authorized_depositor_error);
}

#[test]
fn try_authorized_deposit_batch_or_refund_performs_a_refund_when_badge_is_not_in_depositors_list() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (pk1, _, account1) = test_runner.new_account(true);
    let (pk2, _, account2) = test_runner.new_account(true);

    let badge = ResourceOrNonFungible::Resource(XRD);
    test_runner
        .execute_manifest_ignoring_fee(
            ManifestBuilder::new()
                .call_method(
                    account1,
                    ACCOUNT_SET_DEFAULT_DEPOSIT_RULE_IDENT,
                    AccountSetDefaultDepositRuleInput {
                        default: DefaultDepositRule::Reject,
                    },
                )
                .call_method(
                    account1,
                    ACCOUNT_ADD_AUTHORIZED_DEPOSITOR,
                    AccountAddAuthorizedDepositorInput {
                        badge: badge.clone(),
                    },
                )
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&pk1)],
        )
        .expect_commit_success();

    // Act
    let manifest = ManifestBuilder::new()
        .withdraw_from_account(account2, XRD, 1)
        .take_all_from_worktop(XRD, "bucket")
        .with_bucket("bucket", |builder, bucket| {
            builder.call_method(
                account1,
                ACCOUNT_TRY_DEPOSIT_BATCH_OR_REFUND_IDENT,
                AccountTryDepositBatchOrRefundManifestInput {
                    buckets: vec![bucket],
                    authorized_depositor_badge: Some(ResourceOrNonFungible::Resource(
                        VALIDATOR_OWNER_BADGE,
                    )),
                },
            )
        })
        .build();
    let receipt = test_runner
        .execute_manifest_ignoring_fee(manifest, vec![NonFungibleGlobalId::from_public_key(&pk2)]);

    // Assert
    receipt.expect_specific_failure(is_account_not_an_authorized_depositor_error);
}

#[test]
fn try_authorized_deposit_batch_or_refund_panics_when_badge_is_in_depositors_list_but_is_not_in_auth_zone(
) {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (pk1, _, account1) = test_runner.new_account(true);
    let (pk2, _, account2) = test_runner.new_account(true);

    let badge = ResourceOrNonFungible::Resource(XRD);
    test_runner
        .execute_manifest_ignoring_fee(
            ManifestBuilder::new()
                .call_method(
                    account1,
                    ACCOUNT_SET_DEFAULT_DEPOSIT_RULE_IDENT,
                    AccountSetDefaultDepositRuleInput {
                        default: DefaultDepositRule::Reject,
                    },
                )
                .call_method(
                    account1,
                    ACCOUNT_ADD_AUTHORIZED_DEPOSITOR,
                    AccountAddAuthorizedDepositorInput {
                        badge: badge.clone(),
                    },
                )
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&pk1)],
        )
        .expect_commit_success();

    // Act
    let manifest = ManifestBuilder::new()
        .withdraw_from_account(account2, XRD, 1)
        .take_all_from_worktop(XRD, "bucket")
        .with_bucket("bucket", |builder, bucket| {
            builder.call_method(
                account1,
                ACCOUNT_TRY_DEPOSIT_BATCH_OR_REFUND_IDENT,
                AccountTryDepositBatchOrRefundManifestInput {
                    buckets: vec![bucket],
                    authorized_depositor_badge: Some(badge),
                },
            )
        })
        .build();
    let receipt = test_runner
        .execute_manifest_ignoring_fee(manifest, vec![NonFungibleGlobalId::from_public_key(&pk2)]);

    // Assert
    receipt.expect_auth_assertion_failure();
}

#[test]
fn try_authorized_deposit_batch_or_refund_accepts_deposit_when_depositor_is_authorized_and_badge_is_in_auth_zone(
) {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (pk1, _, account1) = test_runner.new_account(true);
    let (pk2, _, account2) = test_runner.new_account(true);

    let badge = ResourceOrNonFungible::Resource(XRD);
    test_runner
        .execute_manifest_ignoring_fee(
            ManifestBuilder::new()
                .call_method(
                    account1,
                    ACCOUNT_SET_DEFAULT_DEPOSIT_RULE_IDENT,
                    AccountSetDefaultDepositRuleInput {
                        default: DefaultDepositRule::Reject,
                    },
                )
                .call_method(
                    account1,
                    ACCOUNT_ADD_AUTHORIZED_DEPOSITOR,
                    AccountAddAuthorizedDepositorInput {
                        badge: badge.clone(),
                    },
                )
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&pk1)],
        )
        .expect_commit_success();

    // Act
    let manifest = ManifestBuilder::new()
        .create_proof_from_account_of_amount(account2, XRD, 1)
        .withdraw_from_account(account2, XRD, 1)
        .take_all_from_worktop(XRD, "bucket")
        .with_bucket("bucket", |builder, bucket| {
            builder.call_method(
                account1,
                ACCOUNT_TRY_DEPOSIT_BATCH_OR_REFUND_IDENT,
                AccountTryDepositBatchOrRefundManifestInput {
                    buckets: vec![bucket],
                    authorized_depositor_badge: Some(badge),
                },
            )
        })
        .build();
    let receipt = test_runner
        .execute_manifest_ignoring_fee(manifest, vec![NonFungibleGlobalId::from_public_key(&pk2)]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn try_authorized_deposit_or_abort_performs_an_abort_when_badge_is_not_in_depositors_list() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (pk1, _, account1) = test_runner.new_account(true);
    let (pk2, _, account2) = test_runner.new_account(true);

    let badge = ResourceOrNonFungible::Resource(XRD);
    test_runner
        .execute_manifest_ignoring_fee(
            ManifestBuilder::new()
                .call_method(
                    account1,
                    ACCOUNT_SET_DEFAULT_DEPOSIT_RULE_IDENT,
                    AccountSetDefaultDepositRuleInput {
                        default: DefaultDepositRule::Reject,
                    },
                )
                .call_method(
                    account1,
                    ACCOUNT_ADD_AUTHORIZED_DEPOSITOR,
                    AccountAddAuthorizedDepositorInput {
                        badge: badge.clone(),
                    },
                )
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&pk1)],
        )
        .expect_commit_success();

    // Act
    let manifest = ManifestBuilder::new()
        .withdraw_from_account(account2, XRD, 1)
        .take_all_from_worktop(XRD, "bucket")
        .with_bucket("bucket", |builder, bucket| {
            builder.call_method(
                account1,
                ACCOUNT_TRY_DEPOSIT_OR_ABORT_IDENT,
                AccountTryDepositOrAbortManifestInput {
                    bucket,
                    authorized_depositor_badge: Some(ResourceOrNonFungible::Resource(
                        VALIDATOR_OWNER_BADGE,
                    )),
                },
            )
        })
        .build();
    let receipt = test_runner
        .execute_manifest_ignoring_fee(manifest, vec![NonFungibleGlobalId::from_public_key(&pk2)]);

    // Assert
    receipt.expect_specific_failure(is_account_not_an_authorized_depositor_error);
}

#[test]
fn try_authorized_deposit_or_abort_panics_when_badge_is_in_depositors_list_but_is_not_in_auth_zone()
{
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (pk1, _, account1) = test_runner.new_account(true);
    let (pk2, _, account2) = test_runner.new_account(true);

    let badge = ResourceOrNonFungible::Resource(XRD);
    test_runner
        .execute_manifest_ignoring_fee(
            ManifestBuilder::new()
                .call_method(
                    account1,
                    ACCOUNT_SET_DEFAULT_DEPOSIT_RULE_IDENT,
                    AccountSetDefaultDepositRuleInput {
                        default: DefaultDepositRule::Reject,
                    },
                )
                .call_method(
                    account1,
                    ACCOUNT_ADD_AUTHORIZED_DEPOSITOR,
                    AccountAddAuthorizedDepositorInput {
                        badge: badge.clone(),
                    },
                )
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&pk1)],
        )
        .expect_commit_success();

    // Act
    let manifest = ManifestBuilder::new()
        .withdraw_from_account(account2, XRD, 1)
        .take_all_from_worktop(XRD, "bucket")
        .with_bucket("bucket", |builder, bucket| {
            builder.call_method(
                account1,
                ACCOUNT_TRY_DEPOSIT_OR_ABORT_IDENT,
                AccountTryDepositOrAbortManifestInput {
                    bucket,
                    authorized_depositor_badge: Some(badge),
                },
            )
        })
        .build();
    let receipt = test_runner
        .execute_manifest_ignoring_fee(manifest, vec![NonFungibleGlobalId::from_public_key(&pk2)]);

    // Assert
    receipt.expect_auth_assertion_failure();
}

#[test]
fn try_authorized_deposit_or_abort_accepts_deposit_when_depositor_is_authorized_and_badge_is_in_auth_zone(
) {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (pk1, _, account1) = test_runner.new_account(true);
    let (pk2, _, account2) = test_runner.new_account(true);

    let badge = ResourceOrNonFungible::Resource(XRD);
    test_runner
        .execute_manifest_ignoring_fee(
            ManifestBuilder::new()
                .call_method(
                    account1,
                    ACCOUNT_SET_DEFAULT_DEPOSIT_RULE_IDENT,
                    AccountSetDefaultDepositRuleInput {
                        default: DefaultDepositRule::Reject,
                    },
                )
                .call_method(
                    account1,
                    ACCOUNT_ADD_AUTHORIZED_DEPOSITOR,
                    AccountAddAuthorizedDepositorInput {
                        badge: badge.clone(),
                    },
                )
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&pk1)],
        )
        .expect_commit_success();

    // Act
    let manifest = ManifestBuilder::new()
        .create_proof_from_account_of_amount(account2, XRD, 1)
        .withdraw_from_account(account2, XRD, 1)
        .take_all_from_worktop(XRD, "bucket")
        .with_bucket("bucket", |builder, bucket| {
            builder.call_method(
                account1,
                ACCOUNT_TRY_DEPOSIT_OR_ABORT_IDENT,
                AccountTryDepositOrAbortManifestInput {
                    bucket,
                    authorized_depositor_badge: Some(badge),
                },
            )
        })
        .build();
    let receipt = test_runner
        .execute_manifest_ignoring_fee(manifest, vec![NonFungibleGlobalId::from_public_key(&pk2)]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn try_authorized_deposit_batch_or_abort_performs_an_abort_when_badge_is_not_in_depositors_list() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (pk1, _, account1) = test_runner.new_account(true);
    let (pk2, _, account2) = test_runner.new_account(true);

    let badge = ResourceOrNonFungible::Resource(XRD);
    test_runner
        .execute_manifest_ignoring_fee(
            ManifestBuilder::new()
                .call_method(
                    account1,
                    ACCOUNT_SET_DEFAULT_DEPOSIT_RULE_IDENT,
                    AccountSetDefaultDepositRuleInput {
                        default: DefaultDepositRule::Reject,
                    },
                )
                .call_method(
                    account1,
                    ACCOUNT_ADD_AUTHORIZED_DEPOSITOR,
                    AccountAddAuthorizedDepositorInput {
                        badge: badge.clone(),
                    },
                )
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&pk1)],
        )
        .expect_commit_success();

    // Act
    let manifest = ManifestBuilder::new()
        .withdraw_from_account(account2, XRD, 1)
        .take_all_from_worktop(XRD, "bucket")
        .with_bucket("bucket", |builder, bucket| {
            builder.call_method(
                account1,
                ACCOUNT_TRY_DEPOSIT_BATCH_OR_ABORT_IDENT,
                AccountTryDepositBatchOrAbortManifestInput {
                    buckets: vec![bucket],
                    authorized_depositor_badge: Some(ResourceOrNonFungible::Resource(
                        VALIDATOR_OWNER_BADGE,
                    )),
                },
            )
        })
        .build();
    let receipt = test_runner
        .execute_manifest_ignoring_fee(manifest, vec![NonFungibleGlobalId::from_public_key(&pk2)]);

    // Assert
    receipt.expect_specific_failure(is_account_not_an_authorized_depositor_error);
}

#[test]
fn try_authorized_deposit_batch_or_abort_panics_when_badge_is_in_depositors_list_but_is_not_in_auth_zone(
) {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (pk1, _, account1) = test_runner.new_account(true);
    let (pk2, _, account2) = test_runner.new_account(true);

    let badge = ResourceOrNonFungible::Resource(XRD);
    test_runner
        .execute_manifest_ignoring_fee(
            ManifestBuilder::new()
                .call_method(
                    account1,
                    ACCOUNT_SET_DEFAULT_DEPOSIT_RULE_IDENT,
                    AccountSetDefaultDepositRuleInput {
                        default: DefaultDepositRule::Reject,
                    },
                )
                .call_method(
                    account1,
                    ACCOUNT_ADD_AUTHORIZED_DEPOSITOR,
                    AccountAddAuthorizedDepositorInput {
                        badge: badge.clone(),
                    },
                )
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&pk1)],
        )
        .expect_commit_success();

    // Act
    let manifest = ManifestBuilder::new()
        .withdraw_from_account(account2, XRD, 1)
        .take_all_from_worktop(XRD, "bucket")
        .with_bucket("bucket", |builder, bucket| {
            builder.call_method(
                account1,
                ACCOUNT_TRY_DEPOSIT_BATCH_OR_ABORT_IDENT,
                AccountTryDepositBatchOrAbortManifestInput {
                    buckets: vec![bucket],
                    authorized_depositor_badge: Some(badge),
                },
            )
        })
        .build();
    let receipt = test_runner
        .execute_manifest_ignoring_fee(manifest, vec![NonFungibleGlobalId::from_public_key(&pk2)]);

    // Assert
    receipt.expect_auth_assertion_failure();
}

#[test]
fn try_authorized_deposit_batch_or_abort_accepts_deposit_when_depositor_is_authorized_and_badge_is_in_auth_zone(
) {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (pk1, _, account1) = test_runner.new_account(true);
    let (pk2, _, account2) = test_runner.new_account(true);

    let badge = ResourceOrNonFungible::Resource(XRD);
    test_runner
        .execute_manifest_ignoring_fee(
            ManifestBuilder::new()
                .call_method(
                    account1,
                    ACCOUNT_SET_DEFAULT_DEPOSIT_RULE_IDENT,
                    AccountSetDefaultDepositRuleInput {
                        default: DefaultDepositRule::Reject,
                    },
                )
                .call_method(
                    account1,
                    ACCOUNT_ADD_AUTHORIZED_DEPOSITOR,
                    AccountAddAuthorizedDepositorInput {
                        badge: badge.clone(),
                    },
                )
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&pk1)],
        )
        .expect_commit_success();

    // Act
    let manifest = ManifestBuilder::new()
        .create_proof_from_account_of_amount(account2, XRD, 1)
        .withdraw_from_account(account2, XRD, 1)
        .take_all_from_worktop(XRD, "bucket")
        .with_bucket("bucket", |builder, bucket| {
            builder.call_method(
                account1,
                ACCOUNT_TRY_DEPOSIT_BATCH_OR_ABORT_IDENT,
                AccountTryDepositBatchOrAbortManifestInput {
                    buckets: vec![bucket],
                    authorized_depositor_badge: Some(badge),
                },
            )
        })
        .build();
    let receipt = test_runner
        .execute_manifest_ignoring_fee(manifest, vec![NonFungibleGlobalId::from_public_key(&pk2)]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn authorized_depositor_badge_is_ignored_when_deposit_batch_is_permitted_without_it() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (pk, _, account) = test_runner.new_account(false);

    // Act
    for method_name in [
        ACCOUNT_TRY_DEPOSIT_BATCH_OR_ABORT_IDENT,
        ACCOUNT_TRY_DEPOSIT_BATCH_OR_REFUND_IDENT,
    ] {
        let manifest = ManifestBuilder::new()
            .lock_fee(FAUCET, 10)
            .call_method(FAUCET, "free", ())
            .call_method(
                account,
                method_name,
                manifest_args!(
                    ManifestExpression::EntireWorktop,
                    Option::<ResourceOrNonFungible>::Some(ResourceOrNonFungible::Resource(XRD))
                ),
            )
            .build();
        let receipt =
            test_runner.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&pk)]);

        // Assert
        receipt.expect_commit_success();
    }
}

#[test]
fn authorized_depositor_badge_is_ignored_when_deposit_is_permitted_without_it() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (pk, _, account) = test_runner.new_account(false);

    // Act
    for method_name in [
        ACCOUNT_TRY_DEPOSIT_OR_ABORT_IDENT,
        ACCOUNT_TRY_DEPOSIT_OR_REFUND_IDENT,
    ] {
        let manifest = ManifestBuilder::new()
            .lock_fee(FAUCET, 10)
            .call_method(FAUCET, "free", ())
            .take_all_from_worktop(XRD, "bucket")
            .with_bucket("bucket", |builder, bucket| {
                builder.call_method(
                    account,
                    method_name,
                    manifest_args!(
                        bucket,
                        Option::<ResourceOrNonFungible>::Some(ResourceOrNonFungible::Resource(XRD))
                    ),
                )
            })
            .build();
        let receipt =
            test_runner.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&pk)]);

        // Assert
        receipt.expect_commit_success();
    }
}

#[test]
fn authorized_depositor_badge_is_checked_when_deposit_cant_go_without_it() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (pk, _, account) = test_runner.new_account(false);

    // Act
    for method_name in [
        ACCOUNT_TRY_DEPOSIT_OR_ABORT_IDENT,
        ACCOUNT_TRY_DEPOSIT_OR_REFUND_IDENT,
    ] {
        let manifest = ManifestBuilder::new()
            .lock_fee(FAUCET, 10)
            .call_method(
                account,
                ACCOUNT_SET_DEFAULT_DEPOSIT_RULE_IDENT,
                AccountSetDefaultDepositRuleInput {
                    default: DefaultDepositRule::Reject,
                },
            )
            .call_method(FAUCET, "free", ())
            .take_all_from_worktop(XRD, "bucket")
            .with_bucket("bucket", |builder, bucket| {
                builder.call_method(
                    account,
                    method_name,
                    manifest_args!(
                        bucket,
                        Option::<ResourceOrNonFungible>::Some(ResourceOrNonFungible::Resource(XRD))
                    ),
                )
            })
            .build();
        let receipt =
            test_runner.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&pk)]);

        // Assert
        receipt.expect_specific_failure(is_account_not_an_authorized_depositor_error);
    }
}

#[test]
fn authorized_depositor_badge_permits_caller_to_deposit() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (pk, _, account) = test_runner.new_account(false);

    // Act
    for method_name in [
        ACCOUNT_TRY_DEPOSIT_OR_ABORT_IDENT,
        ACCOUNT_TRY_DEPOSIT_OR_REFUND_IDENT,
    ] {
        let manifest = ManifestBuilder::new()
            .lock_fee(FAUCET, 10)
            .call_method(
                account,
                ACCOUNT_SET_DEFAULT_DEPOSIT_RULE_IDENT,
                AccountSetDefaultDepositRuleInput {
                    default: DefaultDepositRule::Reject,
                },
            )
            .call_method(
                account,
                ACCOUNT_ADD_AUTHORIZED_DEPOSITOR,
                AccountAddAuthorizedDepositorInput {
                    badge: ResourceOrNonFungible::Resource(XRD),
                },
            )
            .create_proof_from_account_of_amount(account, XRD, 1)
            .call_method(FAUCET, "free", ())
            .take_all_from_worktop(XRD, "bucket")
            .with_bucket("bucket", |builder, bucket| {
                builder.call_method(
                    account,
                    method_name,
                    manifest_args!(
                        bucket,
                        Option::<ResourceOrNonFungible>::Some(ResourceOrNonFungible::Resource(XRD))
                    ),
                )
            })
            .build();
        let receipt =
            test_runner.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&pk)]);

        // Assert
        receipt.expect_commit_success();
    }
}

fn is_account_not_an_authorized_depositor_error(error: &RuntimeError) -> bool {
    matches!(
        error,
        RuntimeError::ApplicationError(ApplicationError::AccountError(
            AccountError::NotAnAuthorizedDepositor { .. }
        ))
    )
}

fn test_depositors_operation_method_auth(
    operation: DepositorsOperation,
    sign: bool,
    assertion: impl FnOnce(&TransactionReceipt),
) {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (pk, _, account) = test_runner.new_account(true);

    let initial_proofs = if sign {
        vec![NonFungibleGlobalId::from_public_key(&pk)]
    } else {
        vec![]
    };

    // Act
    let manifest = {
        let mut builder = ManifestBuilder::new();
        builder = match operation {
            DepositorsOperation::Add { badge } => builder.call_method(
                account,
                ACCOUNT_ADD_AUTHORIZED_DEPOSITOR,
                AccountAddAuthorizedDepositorInput { badge },
            ),
            DepositorsOperation::Remove { badge } => builder.call_method(
                account,
                ACCOUNT_REMOVE_AUTHORIZED_DEPOSITOR,
                AccountRemoveAuthorizedDepositorInput { badge },
            ),
        };
        builder.build()
    };
    let receipt = test_runner.execute_manifest_ignoring_fee(manifest, initial_proofs);

    // Assert
    assertion(&receipt)
}

enum DepositorsOperation {
    Add { badge: ResourceOrNonFungible },
    Remove { badge: ResourceOrNonFungible },
}
