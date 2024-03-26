use radix_engine::errors::*;
use radix_engine::system::system_modules::auth::*;
use radix_engine::updates::*;
use radix_transactions::prelude::*;
use scrypto::blueprints::locker::*;
use scrypto::prelude::*;
use scrypto_test::ledger_simulator::*;

#[test]
fn account_locker_cant_be_instantiated_before_protocol_update() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new()
        .without_kernel_trace()
        .with_custom_protocol_updates(ProtocolUpdates::none().with_anemone())
        .build();
    let (_, _, account) = ledger.new_account(false);

    // Act
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_function(
                LOCKER_PACKAGE,
                ACCOUNT_LOCKER_BLUEPRINT,
                ACCOUNT_LOCKER_INSTANTIATE_SIMPLE_IDENT,
                AccountLockerInstantiateSimpleManifestInput {
                    allow_forceful_withdraws: false,
                },
            )
            .try_deposit_entire_worktop_or_abort(account, None)
            .build(),
        vec![],
    );

    // Assert
    receipt.expect_specific_rejection(|error| {
        error
            == &RejectionReason::ErrorBeforeLoanAndDeferredCostsRepaid(RuntimeError::KernelError(
                KernelError::InvalidReference(LOCKER_PACKAGE.into_node_id()),
            ))
    });
}

#[test]
fn account_locker_can_be_instantiated_after_protocol_update() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().without_kernel_trace().build();
    let (_, _, account) = ledger.new_account(false);

    // Act
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_function(
                LOCKER_PACKAGE,
                ACCOUNT_LOCKER_BLUEPRINT,
                ACCOUNT_LOCKER_INSTANTIATE_SIMPLE_IDENT,
                AccountLockerInstantiateSimpleManifestInput {
                    allow_forceful_withdraws: false,
                },
            )
            .try_deposit_entire_worktop_or_abort(account, None)
            .build(),
        vec![],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn account_locker_has_an_account_locker_entity_type() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().without_kernel_trace().build();
    let (_, _, account) = ledger.new_account(false);

    // Act
    let account_locker = ledger
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .call_function(
                    LOCKER_PACKAGE,
                    ACCOUNT_LOCKER_BLUEPRINT,
                    ACCOUNT_LOCKER_INSTANTIATE_SIMPLE_IDENT,
                    AccountLockerInstantiateSimpleManifestInput {
                        allow_forceful_withdraws: false,
                    },
                )
                .try_deposit_entire_worktop_or_abort(account, None)
                .build(),
            vec![],
        )
        .expect_commit_success()
        .new_component_addresses()
        .first()
        .copied()
        .unwrap();

    // Assert
    assert_eq!(
        account_locker.as_node_id().entity_type(),
        Some(EntityType::GlobalAccountLocker)
    );
}

#[test]
fn store_can_only_be_called_by_storer_role() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().without_kernel_trace().build();
    let (public_key, _, account) = ledger.new_account(false);

    let [owner_badge, storer_badge, recoverer_badge] =
        std::array::from_fn(|_| ledger.create_fungible_resource(dec!(1), 0, account));

    let account_locker = ledger
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .call_function(
                    LOCKER_PACKAGE,
                    ACCOUNT_LOCKER_BLUEPRINT,
                    ACCOUNT_LOCKER_INSTANTIATE_IDENT,
                    AccountLockerInstantiateManifestInput {
                        owner_role: OwnerRole::Fixed(rule!(require(owner_badge))),
                        storer_role: rule!(require(storer_badge)),
                        storer_updater_role: rule!(require(storer_badge)),
                        recoverer_role: rule!(require(recoverer_badge)),
                        recoverer_updater_role: rule!(require(recoverer_badge)),
                        address_reservation: None,
                    },
                )
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&public_key)],
        )
        .expect_commit_success()
        .new_component_addresses()
        .first()
        .copied()
        .unwrap();

    for (badge, should_succeed) in [
        (owner_badge, false),
        (storer_badge, true),
        (recoverer_badge, false),
    ] {
        // Act
        let receipt = ledger.execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .create_proof_from_account_of_amount(account, badge, dec!(1))
                .take_from_worktop(XRD, dec!(0), "bucket")
                .with_bucket("bucket", |builder, bucket| {
                    builder.call_method(
                        account_locker,
                        ACCOUNT_LOCKER_STORE_IDENT,
                        AccountLockerStoreManifestInput {
                            bucket,
                            claimant: account,
                        },
                    )
                })
                .deposit_batch(account)
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&public_key)],
        );

        // Assert
        if should_succeed {
            receipt.expect_commit_success();
        } else {
            receipt.expect_specific_failure(|error| {
                matches!(
                    error,
                    RuntimeError::SystemModuleError(SystemModuleError::AuthError(
                        AuthError::Unauthorized(..)
                    ))
                )
            });
        }
    }
}

#[test]
fn store_batch_can_only_be_called_by_storer_role() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().without_kernel_trace().build();
    let (public_key, _, account) = ledger.new_account(false);

    let [owner_badge, storer_badge, recoverer_badge] =
        std::array::from_fn(|_| ledger.create_fungible_resource(dec!(1), 0, account));

    let account_locker = ledger
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .call_function(
                    LOCKER_PACKAGE,
                    ACCOUNT_LOCKER_BLUEPRINT,
                    ACCOUNT_LOCKER_INSTANTIATE_IDENT,
                    AccountLockerInstantiateManifestInput {
                        owner_role: OwnerRole::Fixed(rule!(require(owner_badge))),
                        storer_role: rule!(require(storer_badge)),
                        storer_updater_role: rule!(require(storer_badge)),
                        recoverer_role: rule!(require(recoverer_badge)),
                        recoverer_updater_role: rule!(require(recoverer_badge)),
                        address_reservation: None,
                    },
                )
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&public_key)],
        )
        .expect_commit_success()
        .new_component_addresses()
        .first()
        .copied()
        .unwrap();

    for (badge, should_succeed) in [
        (owner_badge, false),
        (storer_badge, true),
        (recoverer_badge, false),
    ] {
        // Act
        let receipt = ledger.execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .create_proof_from_account_of_amount(account, badge, dec!(1))
                .take_from_worktop(XRD, dec!(0), "bucket")
                .with_bucket("bucket", |builder, bucket| {
                    builder.call_method(
                        account_locker,
                        ACCOUNT_LOCKER_STORE_BATCH_IDENT,
                        AccountLockerStoreBatchManifestInput {
                            bucket,
                            claimants: indexmap!(),
                        },
                    )
                })
                .deposit_batch(account)
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&public_key)],
        );

        // Assert
        if should_succeed {
            receipt.expect_commit_success();
        } else {
            receipt.expect_specific_failure(|error| {
                matches!(
                    error,
                    RuntimeError::SystemModuleError(SystemModuleError::AuthError(
                        AuthError::Unauthorized(..)
                    ))
                )
            });
        }
    }
}

#[test]
fn send_or_store_can_only_be_called_by_storer_role() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().without_kernel_trace().build();
    let (public_key, _, account) = ledger.new_account(false);

    let [owner_badge, storer_badge, recoverer_badge] =
        std::array::from_fn(|_| ledger.create_fungible_resource(dec!(1), 0, account));

    let account_locker = ledger
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .call_function(
                    LOCKER_PACKAGE,
                    ACCOUNT_LOCKER_BLUEPRINT,
                    ACCOUNT_LOCKER_INSTANTIATE_IDENT,
                    AccountLockerInstantiateManifestInput {
                        owner_role: OwnerRole::Fixed(rule!(require(owner_badge))),
                        storer_role: rule!(require(storer_badge)),
                        storer_updater_role: rule!(require(storer_badge)),
                        recoverer_role: rule!(require(recoverer_badge)),
                        recoverer_updater_role: rule!(require(recoverer_badge)),
                        address_reservation: None,
                    },
                )
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&public_key)],
        )
        .expect_commit_success()
        .new_component_addresses()
        .first()
        .copied()
        .unwrap();

    for (badge, should_succeed) in [
        (owner_badge, false),
        (storer_badge, true),
        (recoverer_badge, false),
    ] {
        // Act
        let receipt = ledger.execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .create_proof_from_account_of_amount(account, badge, dec!(1))
                .take_from_worktop(XRD, dec!(0), "bucket")
                .with_bucket("bucket", |builder, bucket| {
                    builder.call_method(
                        account_locker,
                        ACCOUNT_LOCKER_SEND_OR_STORE_IDENT,
                        AccountLockerSendOrStoreManifestInput {
                            bucket,
                            claimant: account,
                        },
                    )
                })
                .deposit_batch(account)
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&public_key)],
        );

        // Assert
        if should_succeed {
            receipt.expect_commit_success();
        } else {
            receipt.expect_specific_failure(|error| {
                matches!(
                    error,
                    RuntimeError::SystemModuleError(SystemModuleError::AuthError(
                        AuthError::Unauthorized(..)
                    ))
                )
            });
        }
    }
}

#[test]
fn send_or_store_batch_can_only_be_called_by_storer_role() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().without_kernel_trace().build();
    let (public_key, _, account) = ledger.new_account(false);

    let [owner_badge, storer_badge, recoverer_badge] =
        std::array::from_fn(|_| ledger.create_fungible_resource(dec!(1), 0, account));

    let account_locker = ledger
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .call_function(
                    LOCKER_PACKAGE,
                    ACCOUNT_LOCKER_BLUEPRINT,
                    ACCOUNT_LOCKER_INSTANTIATE_IDENT,
                    AccountLockerInstantiateManifestInput {
                        owner_role: OwnerRole::Fixed(rule!(require(owner_badge))),
                        storer_role: rule!(require(storer_badge)),
                        storer_updater_role: rule!(require(storer_badge)),
                        recoverer_role: rule!(require(recoverer_badge)),
                        recoverer_updater_role: rule!(require(recoverer_badge)),
                        address_reservation: None,
                    },
                )
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&public_key)],
        )
        .expect_commit_success()
        .new_component_addresses()
        .first()
        .copied()
        .unwrap();

    for (badge, should_succeed) in [
        (owner_badge, false),
        (storer_badge, true),
        (recoverer_badge, false),
    ] {
        // Act
        let receipt = ledger.execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .create_proof_from_account_of_amount(account, badge, dec!(1))
                .take_from_worktop(XRD, dec!(0), "bucket")
                .with_bucket("bucket", |builder, bucket| {
                    builder.call_method(
                        account_locker,
                        ACCOUNT_LOCKER_SEND_OR_STORE_BATCH_IDENT,
                        AccountLockerSendOrStoreBatchManifestInput {
                            bucket,
                            claimants: indexmap!(),
                        },
                    )
                })
                .deposit_batch(account)
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&public_key)],
        );

        // Assert
        if should_succeed {
            receipt.expect_commit_success();
        } else {
            receipt.expect_specific_failure(|error| {
                matches!(
                    error,
                    RuntimeError::SystemModuleError(SystemModuleError::AuthError(
                        AuthError::Unauthorized(..)
                    ))
                )
            });
        }
    }
}

#[test]
fn recover_can_only_be_called_by_recoverer_role() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().without_kernel_trace().build();
    let (public_key, _, account) = ledger.new_account(false);

    let [owner_badge, storer_badge, recoverer_badge] =
        std::array::from_fn(|_| ledger.create_fungible_resource(dec!(1), 0, account));

    let account_locker = ledger
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .call_function(
                    LOCKER_PACKAGE,
                    ACCOUNT_LOCKER_BLUEPRINT,
                    ACCOUNT_LOCKER_INSTANTIATE_IDENT,
                    AccountLockerInstantiateManifestInput {
                        owner_role: OwnerRole::Fixed(rule!(require(owner_badge))),
                        storer_role: rule!(require(storer_badge)),
                        storer_updater_role: rule!(require(storer_badge)),
                        recoverer_role: rule!(require(recoverer_badge)),
                        recoverer_updater_role: rule!(require(recoverer_badge)),
                        address_reservation: None,
                    },
                )
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&public_key)],
        )
        .expect_commit_success()
        .new_component_addresses()
        .first()
        .copied()
        .unwrap();

    for (badge, should_succeed) in [
        (owner_badge, false),
        (storer_badge, false),
        (recoverer_badge, true),
    ] {
        // Act
        let receipt = ledger.execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .create_proof_from_account_of_amount(account, badge, dec!(1))
                .call_method(
                    account_locker,
                    ACCOUNT_LOCKER_RECOVER_IDENT,
                    AccountLockerRecoverManifestInput {
                        claimant: account,
                        resource_address: XRD,
                        amount: dec!(0),
                    },
                )
                .deposit_batch(account)
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&public_key)],
        );

        // Assert
        if should_succeed {
            receipt.expect_commit_success();
        } else {
            receipt.expect_specific_failure(|error| {
                matches!(
                    error,
                    RuntimeError::SystemModuleError(SystemModuleError::AuthError(
                        AuthError::Unauthorized(..)
                    ))
                )
            });
        }
    }
}

#[test]
fn recover_non_fungibles_can_only_be_called_by_recoverer_role() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().without_kernel_trace().build();
    let (public_key, _, account) = ledger.new_account(false);

    let [owner_badge, storer_badge, recoverer_badge] =
        std::array::from_fn(|_| ledger.create_fungible_resource(dec!(1), 0, account));

    let account_locker = ledger
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .call_function(
                    LOCKER_PACKAGE,
                    ACCOUNT_LOCKER_BLUEPRINT,
                    ACCOUNT_LOCKER_INSTANTIATE_IDENT,
                    AccountLockerInstantiateManifestInput {
                        owner_role: OwnerRole::Fixed(rule!(require(owner_badge))),
                        storer_role: rule!(require(storer_badge)),
                        storer_updater_role: rule!(require(storer_badge)),
                        recoverer_role: rule!(require(recoverer_badge)),
                        recoverer_updater_role: rule!(require(recoverer_badge)),
                        address_reservation: None,
                    },
                )
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&public_key)],
        )
        .expect_commit_success()
        .new_component_addresses()
        .first()
        .copied()
        .unwrap();

    for (badge, should_succeed) in [
        (owner_badge, false),
        (storer_badge, false),
        (recoverer_badge, true),
    ] {
        // Act
        let receipt = ledger.execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .create_proof_from_account_of_amount(account, badge, dec!(1))
                .call_method(
                    account_locker,
                    ACCOUNT_LOCKER_RECOVER_NON_FUNGIBLES_IDENT,
                    AccountLockerRecoverNonFungiblesManifestInput {
                        claimant: account,
                        resource_address: ACCOUNT_OWNER_BADGE,
                        ids: indexset! {},
                    },
                )
                .deposit_batch(account)
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&public_key)],
        );

        // Assert
        if should_succeed {
            receipt.expect_commit_success();
        } else {
            receipt.expect_specific_failure(|error| {
                matches!(
                    error,
                    RuntimeError::SystemModuleError(SystemModuleError::AuthError(
                        AuthError::Unauthorized(..)
                    ))
                )
            });
        }
    }
}

#[test]
fn claim_is_public_and_callable_by_all() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().without_kernel_trace().build();
    let (public_key, _, account) = ledger.new_account(false);

    let [owner_badge, storer_badge, recoverer_badge] =
        std::array::from_fn(|_| ledger.create_fungible_resource(dec!(1), 0, account));

    let account_locker = ledger
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .call_function(
                    LOCKER_PACKAGE,
                    ACCOUNT_LOCKER_BLUEPRINT,
                    ACCOUNT_LOCKER_INSTANTIATE_IDENT,
                    AccountLockerInstantiateManifestInput {
                        owner_role: OwnerRole::Fixed(rule!(require(owner_badge))),
                        storer_role: rule!(require(storer_badge)),
                        storer_updater_role: rule!(require(storer_badge)),
                        recoverer_role: rule!(require(recoverer_badge)),
                        recoverer_updater_role: rule!(require(recoverer_badge)),
                        address_reservation: None,
                    },
                )
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&public_key)],
        )
        .expect_commit_success()
        .new_component_addresses()
        .first()
        .copied()
        .unwrap();

    for (badge, should_succeed) in [
        (owner_badge, true),
        (storer_badge, true),
        (recoverer_badge, true),
    ] {
        // Act
        let receipt = ledger.execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .create_proof_from_account_of_amount(account, badge, dec!(1))
                .call_method(
                    account_locker,
                    ACCOUNT_LOCKER_CLAIM_IDENT,
                    AccountLockerClaimManifestInput {
                        claimant: account,
                        resource_address: XRD,
                        amount: dec!(0),
                    },
                )
                .deposit_batch(account)
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&public_key)],
        );

        // Assert
        if should_succeed {
            receipt.expect_commit_success();
        } else {
            receipt.expect_specific_failure(|error| {
                matches!(
                    error,
                    RuntimeError::SystemModuleError(SystemModuleError::AuthError(
                        AuthError::Unauthorized(..)
                    ))
                )
            });
        }
    }
}

#[test]
fn claim_non_fungibles_is_public_and_callable_by_all() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().without_kernel_trace().build();
    let (public_key, _, account) = ledger.new_account(false);

    let [owner_badge, storer_badge, recoverer_badge] =
        std::array::from_fn(|_| ledger.create_fungible_resource(dec!(1), 0, account));

    let account_locker = ledger
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .call_function(
                    LOCKER_PACKAGE,
                    ACCOUNT_LOCKER_BLUEPRINT,
                    ACCOUNT_LOCKER_INSTANTIATE_IDENT,
                    AccountLockerInstantiateManifestInput {
                        owner_role: OwnerRole::Fixed(rule!(require(owner_badge))),
                        storer_role: rule!(require(storer_badge)),
                        storer_updater_role: rule!(require(storer_badge)),
                        recoverer_role: rule!(require(recoverer_badge)),
                        recoverer_updater_role: rule!(require(recoverer_badge)),
                        address_reservation: None,
                    },
                )
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&public_key)],
        )
        .expect_commit_success()
        .new_component_addresses()
        .first()
        .copied()
        .unwrap();

    for (badge, should_succeed) in [
        (owner_badge, true),
        (storer_badge, true),
        (recoverer_badge, true),
    ] {
        // Act
        let receipt = ledger.execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .create_proof_from_account_of_amount(account, badge, dec!(1))
                .call_method(
                    account_locker,
                    ACCOUNT_LOCKER_CLAIM_NON_FUNGIBLES_IDENT,
                    AccountLockerClaimNonFungiblesManifestInput {
                        claimant: account,
                        resource_address: ACCOUNT_OWNER_BADGE,
                        ids: indexset! {},
                    },
                )
                .deposit_batch(account)
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&public_key)],
        );

        // Assert
        if should_succeed {
            receipt.expect_commit_success();
        } else {
            receipt.expect_specific_failure(|error| {
                matches!(
                    error,
                    RuntimeError::SystemModuleError(SystemModuleError::AuthError(
                        AuthError::Unauthorized(..)
                    ))
                )
            });
        }
    }
}

#[test]
fn an_account_can_claim_its_resources_from_the_account_locker() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().without_kernel_trace().build();
    let (badge_holder_account_public_key, _, badge_holder_account) = ledger.new_account(false);
    let (user_account1_public_key, _, user_account1) = ledger.new_account(false);

    let (account_locker, account_locker_badge) = {
        let commit_result = ledger
            .execute_manifest(
                ManifestBuilder::new()
                    .lock_fee_from_faucet()
                    .call_function(
                        LOCKER_PACKAGE,
                        ACCOUNT_LOCKER_BLUEPRINT,
                        ACCOUNT_LOCKER_INSTANTIATE_SIMPLE_IDENT,
                        AccountLockerInstantiateSimpleManifestInput {
                            allow_forceful_withdraws: false,
                        },
                    )
                    .try_deposit_entire_worktop_or_abort(badge_holder_account, None)
                    .build(),
                vec![],
            )
            .expect_commit_success()
            .clone();

        let locker = commit_result
            .new_component_addresses()
            .first()
            .copied()
            .unwrap();
        let badge = commit_result
            .new_resource_addresses()
            .first()
            .copied()
            .unwrap();

        (locker, badge)
    };

    ledger
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .create_proof_from_account_of_amount(
                    badge_holder_account,
                    account_locker_badge,
                    dec!(1),
                )
                .get_free_xrd_from_faucet()
                .take_all_from_worktop(XRD, "bucket")
                .with_bucket("bucket", |builder, bucket| {
                    builder.call_method(
                        account_locker,
                        ACCOUNT_LOCKER_STORE_IDENT,
                        AccountLockerStoreManifestInput {
                            bucket,
                            claimant: user_account1,
                        },
                    )
                })
                .build(),
            vec![NonFungibleGlobalId::from_public_key(
                &badge_holder_account_public_key,
            )],
        )
        .expect_commit_success();

    // Act
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_method(
                account_locker,
                ACCOUNT_LOCKER_CLAIM_IDENT,
                AccountLockerClaimManifestInput {
                    claimant: user_account1,
                    resource_address: XRD,
                    amount: dec!(10_000),
                },
            )
            .deposit_batch(user_account1)
            .build(),
        vec![NonFungibleGlobalId::from_public_key(
            &user_account1_public_key,
        )],
    );

    // Assert
    receipt.expect_commit_success();
    assert_eq!(
        ledger.get_component_balance(user_account1, XRD),
        dec!(20_000)
    )
}

#[test]
fn an_account_cant_claim_another_accounts_resources_from_the_account_locker() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().without_kernel_trace().build();
    let (badge_holder_account_public_key, _, badge_holder_account) = ledger.new_account(false);
    let (_, _, user_account1) = ledger.new_account(false);
    let (user_account2_public_key, _, _) = ledger.new_account(false);

    let (account_locker, account_locker_badge) = {
        let commit_result = ledger
            .execute_manifest(
                ManifestBuilder::new()
                    .lock_fee_from_faucet()
                    .call_function(
                        LOCKER_PACKAGE,
                        ACCOUNT_LOCKER_BLUEPRINT,
                        ACCOUNT_LOCKER_INSTANTIATE_SIMPLE_IDENT,
                        AccountLockerInstantiateSimpleManifestInput {
                            allow_forceful_withdraws: false,
                        },
                    )
                    .try_deposit_entire_worktop_or_abort(badge_holder_account, None)
                    .build(),
                vec![],
            )
            .expect_commit_success()
            .clone();

        let locker = commit_result
            .new_component_addresses()
            .first()
            .copied()
            .unwrap();
        let badge = commit_result
            .new_resource_addresses()
            .first()
            .copied()
            .unwrap();

        (locker, badge)
    };

    ledger
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .create_proof_from_account_of_amount(
                    badge_holder_account,
                    account_locker_badge,
                    dec!(1),
                )
                .get_free_xrd_from_faucet()
                .take_all_from_worktop(XRD, "bucket")
                .with_bucket("bucket", |builder, bucket| {
                    builder.call_method(
                        account_locker,
                        ACCOUNT_LOCKER_STORE_IDENT,
                        AccountLockerStoreManifestInput {
                            bucket,
                            claimant: user_account1,
                        },
                    )
                })
                .build(),
            vec![NonFungibleGlobalId::from_public_key(
                &badge_holder_account_public_key,
            )],
        )
        .expect_commit_success();

    // Act
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_method(
                account_locker,
                ACCOUNT_LOCKER_CLAIM_IDENT,
                AccountLockerClaimManifestInput {
                    claimant: user_account1,
                    resource_address: XRD,
                    amount: dec!(10_000),
                },
            )
            .deposit_batch(user_account1)
            .build(),
        vec![NonFungibleGlobalId::from_public_key(
            &user_account2_public_key,
        )],
    );

    // Assert
    receipt.expect_specific_failure(|error| {
        matches!(
            error,
            RuntimeError::SystemError(SystemError::AssertAccessRuleFailed)
        )
    });
}

#[test]
fn account_locker_admin_can_recover_resources_from_an_account_locker() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().without_kernel_trace().build();
    let (badge_holder_account_public_key, _, badge_holder_account) = ledger.new_account(false);
    let (_, _, user_account1) = ledger.new_account(false);

    let (account_locker, account_locker_badge) = {
        let commit_result = ledger
            .execute_manifest(
                ManifestBuilder::new()
                    .lock_fee_from_faucet()
                    .call_function(
                        LOCKER_PACKAGE,
                        ACCOUNT_LOCKER_BLUEPRINT,
                        ACCOUNT_LOCKER_INSTANTIATE_SIMPLE_IDENT,
                        AccountLockerInstantiateSimpleManifestInput {
                            allow_forceful_withdraws: true,
                        },
                    )
                    .try_deposit_entire_worktop_or_abort(badge_holder_account, None)
                    .build(),
                vec![],
            )
            .expect_commit_success()
            .clone();

        let locker = commit_result
            .new_component_addresses()
            .first()
            .copied()
            .unwrap();
        let badge = commit_result
            .new_resource_addresses()
            .first()
            .copied()
            .unwrap();

        (locker, badge)
    };

    ledger
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .create_proof_from_account_of_amount(
                    badge_holder_account,
                    account_locker_badge,
                    dec!(1),
                )
                .get_free_xrd_from_faucet()
                .take_all_from_worktop(XRD, "bucket")
                .with_bucket("bucket", |builder, bucket| {
                    builder.call_method(
                        account_locker,
                        ACCOUNT_LOCKER_STORE_IDENT,
                        AccountLockerStoreManifestInput {
                            bucket,
                            claimant: user_account1,
                        },
                    )
                })
                .build(),
            vec![NonFungibleGlobalId::from_public_key(
                &badge_holder_account_public_key,
            )],
        )
        .expect_commit_success();

    // Act
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_proof_from_account_of_amount(badge_holder_account, account_locker_badge, 1)
            .call_method(
                account_locker,
                ACCOUNT_LOCKER_RECOVER_IDENT,
                AccountLockerRecoverManifestInput {
                    claimant: user_account1,
                    resource_address: XRD,
                    amount: dec!(10_000),
                },
            )
            .deposit_batch(badge_holder_account)
            .build(),
        vec![NonFungibleGlobalId::from_public_key(
            &badge_holder_account_public_key,
        )],
    );

    // Assert
    receipt.expect_commit_success();
    assert_eq!(
        ledger.get_component_balance(badge_holder_account, XRD),
        dec!(20_000)
    )
}

#[test]
fn account_locker_admin_cant_recover_resources_from_an_account_locker_when_disabled() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().without_kernel_trace().build();
    let (badge_holder_account_public_key, _, badge_holder_account) = ledger.new_account(false);
    let (_, _, user_account1) = ledger.new_account(false);

    let (account_locker, account_locker_badge) = {
        let commit_result = ledger
            .execute_manifest(
                ManifestBuilder::new()
                    .lock_fee_from_faucet()
                    .call_function(
                        LOCKER_PACKAGE,
                        ACCOUNT_LOCKER_BLUEPRINT,
                        ACCOUNT_LOCKER_INSTANTIATE_SIMPLE_IDENT,
                        AccountLockerInstantiateSimpleManifestInput {
                            allow_forceful_withdraws: false,
                        },
                    )
                    .try_deposit_entire_worktop_or_abort(badge_holder_account, None)
                    .build(),
                vec![],
            )
            .expect_commit_success()
            .clone();

        let locker = commit_result
            .new_component_addresses()
            .first()
            .copied()
            .unwrap();
        let badge = commit_result
            .new_resource_addresses()
            .first()
            .copied()
            .unwrap();

        (locker, badge)
    };

    ledger
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .create_proof_from_account_of_amount(
                    badge_holder_account,
                    account_locker_badge,
                    dec!(1),
                )
                .get_free_xrd_from_faucet()
                .take_all_from_worktop(XRD, "bucket")
                .with_bucket("bucket", |builder, bucket| {
                    builder.call_method(
                        account_locker,
                        ACCOUNT_LOCKER_STORE_IDENT,
                        AccountLockerStoreManifestInput {
                            bucket,
                            claimant: user_account1,
                        },
                    )
                })
                .build(),
            vec![NonFungibleGlobalId::from_public_key(
                &badge_holder_account_public_key,
            )],
        )
        .expect_commit_success();

    // Act
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_proof_from_account_of_amount(badge_holder_account, account_locker_badge, 1)
            .call_method(
                account_locker,
                ACCOUNT_LOCKER_RECOVER_IDENT,
                AccountLockerRecoverManifestInput {
                    claimant: user_account1,
                    resource_address: XRD,
                    amount: dec!(10_000),
                },
            )
            .deposit_batch(badge_holder_account)
            .build(),
        vec![NonFungibleGlobalId::from_public_key(
            &badge_holder_account_public_key,
        )],
    );

    // Assert
    receipt.expect_specific_failure(|error| {
        matches!(
            error,
            RuntimeError::SystemModuleError(SystemModuleError::AuthError(AuthError::Unauthorized(
                ..
            )))
        )
    });
}

#[test]
fn get_amount_method_reports_the_correct_amount_in_the_vault() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().without_kernel_trace().build();
    let (badge_holder_account_public_key, _, badge_holder_account) = ledger.new_account(false);
    let (_, _, user_account1) = ledger.new_account(false);

    let (account_locker, account_locker_badge) = {
        let commit_result = ledger
            .execute_manifest(
                ManifestBuilder::new()
                    .lock_fee_from_faucet()
                    .call_function(
                        LOCKER_PACKAGE,
                        ACCOUNT_LOCKER_BLUEPRINT,
                        ACCOUNT_LOCKER_INSTANTIATE_SIMPLE_IDENT,
                        AccountLockerInstantiateSimpleManifestInput {
                            allow_forceful_withdraws: false,
                        },
                    )
                    .try_deposit_entire_worktop_or_abort(badge_holder_account, None)
                    .build(),
                vec![],
            )
            .expect_commit_success()
            .clone();

        let locker = commit_result
            .new_component_addresses()
            .first()
            .copied()
            .unwrap();
        let badge = commit_result
            .new_resource_addresses()
            .first()
            .copied()
            .unwrap();

        (locker, badge)
    };

    ledger
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .create_proof_from_account_of_amount(
                    badge_holder_account,
                    account_locker_badge,
                    dec!(1),
                )
                .get_free_xrd_from_faucet()
                .take_all_from_worktop(XRD, "bucket")
                .with_bucket("bucket", |builder, bucket| {
                    builder.call_method(
                        account_locker,
                        ACCOUNT_LOCKER_STORE_IDENT,
                        AccountLockerStoreManifestInput {
                            bucket,
                            claimant: user_account1,
                        },
                    )
                })
                .build(),
            vec![NonFungibleGlobalId::from_public_key(
                &badge_holder_account_public_key,
            )],
        )
        .expect_commit_success();

    // Act
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_method(
                account_locker,
                ACCOUNT_LOCKER_GET_AMOUNT_IDENT,
                AccountLockerGetAmountManifestInput {
                    claimant: user_account1,
                    resource_address: XRD,
                },
            )
            .build(),
        vec![],
    );

    // Assert
    let amount = receipt.expect_commit_success().output::<Decimal>(1);
    assert_eq!(amount, dec!(10_000));
}

#[test]
fn get_non_fungible_local_ids_method_reports_the_correct_ids_in_the_vault() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().without_kernel_trace().build();
    let (badge_holder_account_public_key, _, badge_holder_account) = ledger.new_account(false);
    let (_, _, user_account1) = ledger.new_account(false);

    let non_fungible_resource =
        ledger.create_non_fungible_resource_advanced(Default::default(), badge_holder_account, 10);

    let (account_locker, account_locker_badge) = {
        let commit_result = ledger
            .execute_manifest(
                ManifestBuilder::new()
                    .lock_fee_from_faucet()
                    .call_function(
                        LOCKER_PACKAGE,
                        ACCOUNT_LOCKER_BLUEPRINT,
                        ACCOUNT_LOCKER_INSTANTIATE_SIMPLE_IDENT,
                        AccountLockerInstantiateSimpleManifestInput {
                            allow_forceful_withdraws: false,
                        },
                    )
                    .try_deposit_entire_worktop_or_abort(badge_holder_account, None)
                    .build(),
                vec![],
            )
            .expect_commit_success()
            .clone();

        let locker = commit_result
            .new_component_addresses()
            .first()
            .copied()
            .unwrap();
        let badge = commit_result
            .new_resource_addresses()
            .first()
            .copied()
            .unwrap();

        (locker, badge)
    };

    ledger
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .create_proof_from_account_of_amount(
                    badge_holder_account,
                    account_locker_badge,
                    dec!(1),
                )
                .withdraw_from_account(badge_holder_account, non_fungible_resource, dec!(10))
                .take_all_from_worktop(non_fungible_resource, "bucket")
                .with_bucket("bucket", |builder, bucket| {
                    builder.call_method(
                        account_locker,
                        ACCOUNT_LOCKER_STORE_IDENT,
                        AccountLockerStoreManifestInput {
                            bucket,
                            claimant: user_account1,
                        },
                    )
                })
                .build(),
            vec![NonFungibleGlobalId::from_public_key(
                &badge_holder_account_public_key,
            )],
        )
        .expect_commit_success();

    // Act
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_method(
                account_locker,
                ACCOUNT_LOCKER_GET_NON_FUNGIBLE_LOCAL_IDS_IDENT,
                AccountLockerGetNonFungibleLocalIdsManifestInput {
                    claimant: user_account1,
                    resource_address: non_fungible_resource,
                    limit: 100,
                },
            )
            .build(),
        vec![],
    );

    // Assert
    let amount = receipt
        .expect_commit_success()
        .output::<AccountLockerGetNonFungibleLocalIdsOutput>(1);
    assert_eq!(
        amount,
        (1..=10)
            .map(NonFungibleLocalId::integer)
            .collect::<IndexSet<_>>()
    );
}
