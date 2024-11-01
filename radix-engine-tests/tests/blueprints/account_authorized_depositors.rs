use radix_common::prelude::*;
use radix_engine::blueprints::account::AccountError;
use radix_engine::errors::{ApplicationError, RuntimeError};
use radix_engine::transaction::TransactionReceipt;
use radix_engine_interface::blueprints::account::*;
use radix_engine_interface::prelude::*;
use radix_substate_store_queries::typed_substate_layout::*;
use radix_transactions::prelude::*;
use scrypto_test::prelude::LedgerSimulatorBuilder;

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
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (pk1, _, account1) = ledger.new_account(true);
    let (pk2, _, account2) = ledger.new_account(true);

    let badge = ResourceOrNonFungible::Resource(XRD);
    ledger
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .call_method(
                    account1,
                    ACCOUNT_SET_DEFAULT_DEPOSIT_RULE_IDENT,
                    AccountSetDefaultDepositRuleInput {
                        default: DefaultDepositRule::Reject,
                    },
                )
                .call_method(
                    account1,
                    ACCOUNT_ADD_AUTHORIZED_DEPOSITOR_IDENT,
                    AccountAddAuthorizedDepositorInput { badge: badge },
                )
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&pk1)],
        )
        .expect_commit_success();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .get_free_xrd_from_faucet()
        .take_all_from_worktop(XRD, "bucket")
        .with_bucket("bucket", |builder, bucket| {
            builder.call_method(
                account1,
                ACCOUNT_TRY_DEPOSIT_OR_REFUND_IDENT,
                AccountTryDepositOrRefundManifestInput {
                    bucket,
                    authorized_depositor_badge: Some(
                        ResourceOrNonFungible::Resource(VALIDATOR_OWNER_BADGE).into(),
                    ),
                },
            )
        })
        .deposit_entire_worktop(account2)
        .build();
    let receipt =
        ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&pk2)]);

    // Assert
    receipt.expect_commit_success();
    assert_eq!(ledger.get_component_balance(account2, XRD), dec!(20_000));
}

#[test]
fn try_authorized_deposit_or_refund_panics_when_badge_is_in_depositors_list_but_is_not_in_auth_zone(
) {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (pk1, _, account1) = ledger.new_account(true);
    let (pk2, _, account2) = ledger.new_account(true);

    let badge = ResourceOrNonFungible::Resource(XRD);
    ledger
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .call_method(
                    account1,
                    ACCOUNT_SET_DEFAULT_DEPOSIT_RULE_IDENT,
                    AccountSetDefaultDepositRuleInput {
                        default: DefaultDepositRule::Reject,
                    },
                )
                .call_method(
                    account1,
                    ACCOUNT_ADD_AUTHORIZED_DEPOSITOR_IDENT,
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
        .lock_fee_from_faucet()
        .withdraw_from_account(account2, XRD, 1)
        .take_all_from_worktop(XRD, "bucket")
        .with_bucket("bucket", |builder, bucket| {
            builder.call_method(
                account1,
                ACCOUNT_TRY_DEPOSIT_OR_REFUND_IDENT,
                AccountTryDepositOrRefundManifestInput {
                    bucket,
                    authorized_depositor_badge: Some(badge.into()),
                },
            )
        })
        .build();
    let receipt =
        ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&pk2)]);

    // Assert
    receipt.expect_auth_assertion_failure();
}

#[test]
fn try_authorized_deposit_or_refund_accepts_deposit_when_depositor_is_authorized_and_badge_is_in_auth_zone(
) {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (pk1, _, account1) = ledger.new_account(true);
    let (pk2, _, account2) = ledger.new_account(true);

    let badge = ResourceOrNonFungible::Resource(XRD);
    ledger
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .call_method(
                    account1,
                    ACCOUNT_SET_DEFAULT_DEPOSIT_RULE_IDENT,
                    AccountSetDefaultDepositRuleInput {
                        default: DefaultDepositRule::Reject,
                    },
                )
                .call_method(
                    account1,
                    ACCOUNT_ADD_AUTHORIZED_DEPOSITOR_IDENT,
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
        .lock_fee_from_faucet()
        .create_proof_from_account_of_amount(account2, XRD, 1)
        .withdraw_from_account(account2, XRD, 1)
        .take_all_from_worktop(XRD, "bucket")
        .with_bucket("bucket", |builder, bucket| {
            builder.call_method(
                account1,
                ACCOUNT_TRY_DEPOSIT_OR_REFUND_IDENT,
                AccountTryDepositOrRefundManifestInput {
                    bucket,
                    authorized_depositor_badge: Some(badge.into()),
                },
            )
        })
        .build();
    let receipt =
        ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&pk2)]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn authorized_depositor_can_be_removed_later() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (pk1, _, account1) = ledger.new_account(true);
    let (pk2, _, account2) = ledger.new_account(true);

    let badge = ResourceOrNonFungible::Resource(XRD);
    ledger
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .call_method(
                    account1,
                    ACCOUNT_SET_DEFAULT_DEPOSIT_RULE_IDENT,
                    AccountSetDefaultDepositRuleInput {
                        default: DefaultDepositRule::Reject,
                    },
                )
                .call_method(
                    account1,
                    ACCOUNT_ADD_AUTHORIZED_DEPOSITOR_IDENT,
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
        .lock_fee_from_faucet()
        .create_proof_from_account_of_amount(account2, XRD, 1)
        .withdraw_from_account(account2, XRD, 1)
        .take_all_from_worktop(XRD, "bucket")
        .with_bucket("bucket", |builder, bucket| {
            builder.call_method(
                account1,
                ACCOUNT_TRY_DEPOSIT_OR_REFUND_IDENT,
                AccountTryDepositOrRefundManifestInput {
                    bucket,
                    authorized_depositor_badge: Some(badge.clone().into()),
                },
            )
        })
        .build();
    let receipt =
        ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&pk2)]);

    // Assert 1
    receipt.expect_commit_success();

    // Act 2
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            account1,
            ACCOUNT_REMOVE_AUTHORIZED_DEPOSITOR_IDENT,
            AccountRemoveAuthorizedDepositorInput {
                badge: badge.clone(),
            },
        )
        .build();
    ledger
        .execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&pk1)])
        .expect_commit_success();

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_amount(account2, XRD, 1)
        .withdraw_from_account(account2, XRD, 1)
        .take_all_from_worktop(XRD, "bucket")
        .with_bucket("bucket", |builder, bucket| {
            builder.call_method(
                account1,
                ACCOUNT_TRY_DEPOSIT_OR_REFUND_IDENT,
                AccountTryDepositOrRefundManifestInput {
                    bucket,
                    authorized_depositor_badge: Some(badge.into()),
                },
            )
        })
        .build();
    let receipt =
        ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&pk2)]);

    // Assert 2
    receipt.expect_specific_failure(is_fungible_resource_manager_drop_non_empty_bucket_error);
}

#[test]
fn try_authorized_deposit_batch_or_refund_performs_a_refund_when_badge_is_not_in_depositors_list() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (pk1, _, account1) = ledger.new_account(true);
    let (pk2, _, account2) = ledger.new_account(true);

    let badge = ResourceOrNonFungible::Resource(XRD);
    ledger
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .call_method(
                    account1,
                    ACCOUNT_SET_DEFAULT_DEPOSIT_RULE_IDENT,
                    AccountSetDefaultDepositRuleInput {
                        default: DefaultDepositRule::Reject,
                    },
                )
                .call_method(
                    account1,
                    ACCOUNT_ADD_AUTHORIZED_DEPOSITOR_IDENT,
                    AccountAddAuthorizedDepositorInput { badge: badge },
                )
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&pk1)],
        )
        .expect_commit_success();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .get_free_xrd_from_faucet()
        .take_all_from_worktop(XRD, "bucket")
        .with_bucket("bucket", |builder, bucket| {
            builder.try_deposit_batch_or_refund(
                account1,
                [bucket],
                Some(ResourceOrNonFungible::Resource(VALIDATOR_OWNER_BADGE)),
            )
        })
        .deposit_entire_worktop(account2)
        .build();
    let receipt =
        ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&pk2)]);

    // Assert
    receipt.expect_commit_success();
    assert_eq!(ledger.get_component_balance(account2, XRD), dec!(20_000))
}

#[test]
fn try_authorized_deposit_batch_or_refund_panics_when_badge_is_in_depositors_list_but_is_not_in_auth_zone(
) {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (pk1, _, account1) = ledger.new_account(true);
    let (pk2, _, account2) = ledger.new_account(true);

    let badge = ResourceOrNonFungible::Resource(XRD);
    ledger
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .call_method(
                    account1,
                    ACCOUNT_SET_DEFAULT_DEPOSIT_RULE_IDENT,
                    AccountSetDefaultDepositRuleInput {
                        default: DefaultDepositRule::Reject,
                    },
                )
                .call_method(
                    account1,
                    ACCOUNT_ADD_AUTHORIZED_DEPOSITOR_IDENT,
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
        .lock_fee_from_faucet()
        .withdraw_from_account(account2, XRD, 1)
        .take_all_from_worktop(XRD, "bucket")
        .with_bucket("bucket", |builder, bucket| {
            builder.try_deposit_batch_or_refund(account1, [bucket], Some(badge.into()))
        })
        .build();
    let receipt =
        ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&pk2)]);

    // Assert
    receipt.expect_auth_assertion_failure();
}

#[test]
fn try_authorized_deposit_batch_or_refund_accepts_deposit_when_depositor_is_authorized_and_badge_is_in_auth_zone(
) {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (pk1, _, account1) = ledger.new_account(true);
    let (pk2, _, account2) = ledger.new_account(true);

    let badge = ResourceOrNonFungible::Resource(XRD);
    ledger
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .call_method(
                    account1,
                    ACCOUNT_SET_DEFAULT_DEPOSIT_RULE_IDENT,
                    AccountSetDefaultDepositRuleInput {
                        default: DefaultDepositRule::Reject,
                    },
                )
                .call_method(
                    account1,
                    ACCOUNT_ADD_AUTHORIZED_DEPOSITOR_IDENT,
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
        .lock_fee_from_faucet()
        .create_proof_from_account_of_amount(account2, XRD, 1)
        .withdraw_from_account(account2, XRD, 1)
        .take_all_from_worktop(XRD, "bucket")
        .with_bucket("bucket", |builder, bucket| {
            builder.try_deposit_batch_or_refund(account1, [bucket], Some(badge.into()))
        })
        .build();
    let receipt =
        ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&pk2)]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn try_authorized_deposit_or_abort_performs_an_abort_when_badge_is_not_in_depositors_list() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (pk1, _, account1) = ledger.new_account(true);
    let (pk2, _, account2) = ledger.new_account(true);

    let badge = ResourceOrNonFungible::Resource(XRD);
    ledger
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .call_method(
                    account1,
                    ACCOUNT_SET_DEFAULT_DEPOSIT_RULE_IDENT,
                    AccountSetDefaultDepositRuleInput {
                        default: DefaultDepositRule::Reject,
                    },
                )
                .call_method(
                    account1,
                    ACCOUNT_ADD_AUTHORIZED_DEPOSITOR_IDENT,
                    AccountAddAuthorizedDepositorInput { badge: badge },
                )
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&pk1)],
        )
        .expect_commit_success();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .withdraw_from_account(account2, XRD, 1)
        .take_all_from_worktop(XRD, "bucket")
        .with_bucket("bucket", |builder, bucket| {
            builder.call_method(
                account1,
                ACCOUNT_TRY_DEPOSIT_OR_ABORT_IDENT,
                AccountTryDepositOrAbortManifestInput {
                    bucket,
                    authorized_depositor_badge: Some(
                        ResourceOrNonFungible::Resource(VALIDATOR_OWNER_BADGE).into(),
                    ),
                },
            )
        })
        .build();
    let receipt =
        ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&pk2)]);

    // Assert
    receipt.expect_specific_failure(is_account_not_an_authorized_depositor_error);
}

#[test]
fn try_authorized_deposit_or_abort_panics_when_badge_is_in_depositors_list_but_is_not_in_auth_zone()
{
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (pk1, _, account1) = ledger.new_account(true);
    let (pk2, _, account2) = ledger.new_account(true);

    let badge = ResourceOrNonFungible::Resource(XRD);
    ledger
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .call_method(
                    account1,
                    ACCOUNT_SET_DEFAULT_DEPOSIT_RULE_IDENT,
                    AccountSetDefaultDepositRuleInput {
                        default: DefaultDepositRule::Reject,
                    },
                )
                .call_method(
                    account1,
                    ACCOUNT_ADD_AUTHORIZED_DEPOSITOR_IDENT,
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
        .lock_fee_from_faucet()
        .withdraw_from_account(account2, XRD, 1)
        .take_all_from_worktop(XRD, "bucket")
        .with_bucket("bucket", |builder, bucket| {
            builder.call_method(
                account1,
                ACCOUNT_TRY_DEPOSIT_OR_ABORT_IDENT,
                AccountTryDepositOrAbortManifestInput {
                    bucket,
                    authorized_depositor_badge: Some(badge.into()),
                },
            )
        })
        .build();
    let receipt =
        ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&pk2)]);

    // Assert
    receipt.expect_auth_assertion_failure();
}

#[test]
fn try_authorized_deposit_or_abort_accepts_deposit_when_depositor_is_authorized_and_badge_is_in_auth_zone(
) {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (pk1, _, account1) = ledger.new_account(true);
    let (pk2, _, account2) = ledger.new_account(true);

    let badge = ResourceOrNonFungible::Resource(XRD);
    ledger
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .call_method(
                    account1,
                    ACCOUNT_SET_DEFAULT_DEPOSIT_RULE_IDENT,
                    AccountSetDefaultDepositRuleInput {
                        default: DefaultDepositRule::Reject,
                    },
                )
                .call_method(
                    account1,
                    ACCOUNT_ADD_AUTHORIZED_DEPOSITOR_IDENT,
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
        .lock_fee_from_faucet()
        .create_proof_from_account_of_amount(account2, XRD, 1)
        .withdraw_from_account(account2, XRD, 1)
        .take_all_from_worktop(XRD, "bucket")
        .with_bucket("bucket", |builder, bucket| {
            builder.call_method(
                account1,
                ACCOUNT_TRY_DEPOSIT_OR_ABORT_IDENT,
                AccountTryDepositOrAbortManifestInput {
                    bucket,
                    authorized_depositor_badge: Some(badge.into()),
                },
            )
        })
        .build();
    let receipt =
        ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&pk2)]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn try_authorized_deposit_batch_or_abort_performs_an_abort_when_badge_is_not_in_depositors_list() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (pk1, _, account1) = ledger.new_account(true);
    let (pk2, _, account2) = ledger.new_account(true);

    let badge = ResourceOrNonFungible::Resource(XRD);
    ledger
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .call_method(
                    account1,
                    ACCOUNT_SET_DEFAULT_DEPOSIT_RULE_IDENT,
                    AccountSetDefaultDepositRuleInput {
                        default: DefaultDepositRule::Reject,
                    },
                )
                .call_method(
                    account1,
                    ACCOUNT_ADD_AUTHORIZED_DEPOSITOR_IDENT,
                    AccountAddAuthorizedDepositorInput { badge: badge },
                )
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&pk1)],
        )
        .expect_commit_success();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .withdraw_from_account(account2, XRD, 1)
        .take_all_from_worktop(XRD, "bucket")
        .with_bucket("bucket", |builder, bucket| {
            builder.try_deposit_batch_or_abort(
                account1,
                [bucket],
                Some(ResourceOrNonFungible::Resource(VALIDATOR_OWNER_BADGE)),
            )
        })
        .build();
    let receipt =
        ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&pk2)]);

    // Assert
    receipt.expect_specific_failure(is_account_not_an_authorized_depositor_error);
}

#[test]
fn try_authorized_deposit_batch_or_abort_panics_when_badge_is_in_depositors_list_but_is_not_in_auth_zone(
) {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (pk1, _, account1) = ledger.new_account(true);
    let (pk2, _, account2) = ledger.new_account(true);

    let badge = ResourceOrNonFungible::Resource(XRD);
    ledger
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .call_method(
                    account1,
                    ACCOUNT_SET_DEFAULT_DEPOSIT_RULE_IDENT,
                    AccountSetDefaultDepositRuleInput {
                        default: DefaultDepositRule::Reject,
                    },
                )
                .call_method(
                    account1,
                    ACCOUNT_ADD_AUTHORIZED_DEPOSITOR_IDENT,
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
        .lock_fee_from_faucet()
        .withdraw_from_account(account2, XRD, 1)
        .take_all_from_worktop(XRD, "bucket")
        .with_bucket("bucket", |builder, bucket| {
            builder.try_deposit_batch_or_abort(account1, [bucket], Some(badge.into()))
        })
        .build();
    let receipt =
        ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&pk2)]);

    // Assert
    receipt.expect_auth_assertion_failure();
}

#[test]
fn try_authorized_deposit_batch_or_abort_accepts_deposit_when_depositor_is_authorized_and_badge_is_in_auth_zone(
) {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (pk1, _, account1) = ledger.new_account(true);
    let (pk2, _, account2) = ledger.new_account(true);

    let badge = ResourceOrNonFungible::Resource(XRD);
    ledger
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .call_method(
                    account1,
                    ACCOUNT_SET_DEFAULT_DEPOSIT_RULE_IDENT,
                    AccountSetDefaultDepositRuleInput {
                        default: DefaultDepositRule::Reject,
                    },
                )
                .call_method(
                    account1,
                    ACCOUNT_ADD_AUTHORIZED_DEPOSITOR_IDENT,
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
        .lock_fee_from_faucet()
        .create_proof_from_account_of_amount(account2, XRD, 1)
        .withdraw_from_account(account2, XRD, 1)
        .take_all_from_worktop(XRD, "bucket")
        .with_bucket("bucket", |builder, bucket| {
            builder.try_deposit_batch_or_abort(account1, [bucket], Some(badge.into()))
        })
        .build();
    let receipt =
        ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&pk2)]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn authorized_depositor_badge_is_ignored_when_deposit_batch_is_permitted_without_it() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (pk, _, account) = ledger.new_account(false);

    // Act
    for method_name in [
        ACCOUNT_TRY_DEPOSIT_BATCH_OR_ABORT_IDENT,
        ACCOUNT_TRY_DEPOSIT_BATCH_OR_REFUND_IDENT,
    ] {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
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
            ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&pk)]);

        // Assert
        receipt.expect_commit_success();
    }
}

#[test]
fn authorized_depositor_badge_is_ignored_when_deposit_is_permitted_without_it() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (pk, _, account) = ledger.new_account(false);

    // Act
    for method_name in [
        ACCOUNT_TRY_DEPOSIT_OR_ABORT_IDENT,
        ACCOUNT_TRY_DEPOSIT_OR_REFUND_IDENT,
    ] {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
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
            ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&pk)]);

        // Assert
        receipt.expect_commit_success();
    }
}

#[test]
fn authorized_depositor_badge_is_checked_when_deposit_cant_go_without_it() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (pk, _, account) = ledger.new_account(false);
    let (sink_pk, _, sink) = ledger.new_account(false);

    // Act
    for (method_name, should_refund) in [
        (ACCOUNT_TRY_DEPOSIT_OR_ABORT_IDENT, false),
        (ACCOUNT_TRY_DEPOSIT_OR_REFUND_IDENT, true),
    ] {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
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
            .deposit_entire_worktop(sink)
            .build();
        let receipt = ledger.execute_manifest(
            manifest,
            [&sink_pk, &pk].map(NonFungibleGlobalId::from_public_key),
        );

        // Assert
        if should_refund {
            assert_eq!(ledger.get_component_balance(sink, XRD), dec!(20_000));
        } else {
            receipt.expect_specific_failure(is_account_not_an_authorized_depositor_error);
        }
    }
}

#[test]
fn authorized_depositor_badge_permits_caller_to_deposit() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (pk, _, account) = ledger.new_account(false);

    // Act
    for method_name in [
        ACCOUNT_TRY_DEPOSIT_OR_ABORT_IDENT,
        ACCOUNT_TRY_DEPOSIT_OR_REFUND_IDENT,
    ] {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_method(
                account,
                ACCOUNT_SET_DEFAULT_DEPOSIT_RULE_IDENT,
                AccountSetDefaultDepositRuleInput {
                    default: DefaultDepositRule::Reject,
                },
            )
            .call_method(
                account,
                ACCOUNT_ADD_AUTHORIZED_DEPOSITOR_IDENT,
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
            ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&pk)]);

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

fn is_fungible_resource_manager_drop_non_empty_bucket_error(error: &RuntimeError) -> bool {
    matches!(
        error,
        RuntimeError::ApplicationError(ApplicationError::FungibleResourceManagerError(
            FungibleResourceManagerError::DropNonEmptyBucket
        ))
    )
}

fn test_depositors_operation_method_auth(
    operation: DepositorsOperation,
    sign: bool,
    assertion: impl FnOnce(&TransactionReceipt),
) {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (pk, _, account) = ledger.new_account(true);

    let initial_proofs = if sign {
        vec![NonFungibleGlobalId::from_public_key(&pk)]
    } else {
        vec![]
    };

    // Act
    let manifest = {
        let mut builder = ManifestBuilder::new().lock_fee_from_faucet();
        builder = match operation {
            DepositorsOperation::Add { badge } => builder.call_method(
                account,
                ACCOUNT_ADD_AUTHORIZED_DEPOSITOR_IDENT,
                AccountAddAuthorizedDepositorInput { badge },
            ),
            DepositorsOperation::Remove { badge } => builder.call_method(
                account,
                ACCOUNT_REMOVE_AUTHORIZED_DEPOSITOR_IDENT,
                AccountRemoveAuthorizedDepositorInput { badge },
            ),
        };
        builder.build()
    };
    let receipt = ledger.execute_manifest(manifest, initial_proofs);

    // Assert
    assertion(&receipt)
}

enum DepositorsOperation {
    Add { badge: ResourceOrNonFungible },
    Remove { badge: ResourceOrNonFungible },
}
