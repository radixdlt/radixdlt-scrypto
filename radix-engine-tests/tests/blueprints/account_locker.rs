use radix_engine::errors::*;
use radix_engine::system::system_modules::auth::*;
use radix_engine::system::system_modules::limits::TransactionLimitsError;
use radix_engine::system::system_type_checker::*;
use radix_engine::transaction::*;
use radix_engine::updates::*;
use radix_substate_store_queries::typed_substate_layout::*;
use radix_transactions::prelude::*;
use scrypto::blueprints::account::*;
use scrypto::blueprints::locker::*;
use scrypto::prelude::*;
use scrypto_test::ledger_simulator::*;

#[test]
fn account_locker_cant_be_instantiated_before_protocol_update() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new()
        .with_custom_protocol(|builder| builder.from_bootstrap_to(ProtocolVersion::Anemone))
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
                    allow_recover: false,
                },
            )
            .try_deposit_entire_worktop_or_abort(account, None)
            .build(),
        vec![],
    );

    // Assert
    receipt.expect_specific_rejection(|error| {
        error
            == &RejectionReason::BootloadingError(BootloadingError::ReferencedNodeDoesNotExist(
                LOCKER_PACKAGE.into_node_id().into(),
            ))
    });
}

#[test]
fn account_locker_can_be_instantiated_after_protocol_update() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
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
                    allow_recover: false,
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
    let mut ledger = LedgerSimulatorBuilder::new().build();
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
                        allow_recover: false,
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
fn account_locker_component_address_have_the_expected_bech32m_encoding() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
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
                        allow_recover: false,
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
    let encoded = AddressBech32Encoder::new(&NetworkDefinition::mainnet())
        .encode(&account_locker.as_node_id().0)
        .unwrap();

    // Assert
    assert!(encoded.starts_with("locker_rdx1"),);
}

#[test]
fn store_can_only_be_called_by_storer_role() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
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
                            claimant: account.into(),
                            try_direct_send: false,
                        },
                    )
                })
                .deposit_entire_worktop(account)
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
fn airdrop_can_only_be_called_by_storer_role() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
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
                        ACCOUNT_LOCKER_AIRDROP_IDENT,
                        AccountLockerAirdropManifestInput {
                            bucket,
                            claimants: indexmap!(),
                            try_direct_send: false,
                        },
                    )
                })
                .deposit_entire_worktop(account)
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
    let mut ledger = LedgerSimulatorBuilder::new().build();
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
                        claimant: account.into(),
                        resource_address: XRD.into(),
                        amount: dec!(0),
                    },
                )
                .deposit_entire_worktop(account)
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
    let mut ledger = LedgerSimulatorBuilder::new().build();
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
                        claimant: account.into(),
                        resource_address: ACCOUNT_OWNER_BADGE.into(),
                        ids: indexset! {},
                    },
                )
                .deposit_entire_worktop(account)
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
fn send_or_store_stores_the_resources_if_the_account_rejects_the_deposit_and_the_locker_is_not_and_authorized_depositor(
) {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (badge_holder_public_key, _, badge_holder_account) = ledger.new_account(false);
    let (user_account_public_key, _, user_account) = ledger.new_account(false);

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
                            allow_recover: false,
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
                .call_method(
                    user_account,
                    ACCOUNT_SET_RESOURCE_PREFERENCE_IDENT,
                    AccountSetResourcePreferenceInput {
                        resource_address: XRD.into(),
                        resource_preference: ResourcePreference::Disallowed,
                    },
                )
                .build(),
            vec![NonFungibleGlobalId::from_public_key(
                &user_account_public_key,
            )],
        )
        .expect_commit_success();

    // Act
    let receipt = ledger.execute_manifest(
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
                        claimant: user_account.into(),
                        bucket,
                        try_direct_send: true,
                    },
                )
            })
            .build(),
        vec![NonFungibleGlobalId::from_public_key(
            &badge_holder_public_key,
        )],
    );

    // Assert
    receipt.expect_commit_success();
    assert_eq!(
        ledger.get_component_balance(user_account, XRD),
        dec!(10_000) // The initial 10_000 we get when we create a new account. Nothing more.
    )
}

#[test]
fn send_or_store_sends_the_resources_if_the_locker_is_an_authorized_depositor() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (badge_holder_public_key, _, badge_holder_account) = ledger.new_account(false);
    let (user_account_public_key, _, user_account) = ledger.new_account(false);

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
                            allow_recover: false,
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
                .call_method(
                    user_account,
                    ACCOUNT_SET_RESOURCE_PREFERENCE_IDENT,
                    AccountSetResourcePreferenceInput {
                        resource_address: XRD.into(),
                        resource_preference: ResourcePreference::Disallowed,
                    },
                )
                .call_method(
                    user_account,
                    ACCOUNT_ADD_AUTHORIZED_DEPOSITOR_IDENT,
                    AccountAddAuthorizedDepositorInput {
                        badge: global_caller(account_locker),
                    },
                )
                .build(),
            vec![NonFungibleGlobalId::from_public_key(
                &user_account_public_key,
            )],
        )
        .expect_commit_success();

    // Act
    let receipt = ledger.execute_manifest(
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
                        claimant: user_account.into(),
                        bucket,
                        try_direct_send: true,
                    },
                )
            })
            .build(),
        vec![NonFungibleGlobalId::from_public_key(
            &badge_holder_public_key,
        )],
    );

    // Assert
    receipt.expect_commit_success();
    assert_eq!(
        ledger.get_component_balance(user_account, XRD),
        dec!(20_000) // The initial 10_000 we get when we create a new account. Nothing more.
    )
}

#[test]
fn claim_is_public_and_callable_by_all() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
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
                        claimant: account.into(),
                        resource_address: XRD.into(),
                        amount: dec!(0),
                    },
                )
                .deposit_entire_worktop(account)
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
    let mut ledger = LedgerSimulatorBuilder::new().build();
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
                        claimant: account.into(),
                        resource_address: ACCOUNT_OWNER_BADGE.into(),
                        ids: indexset! {},
                    },
                )
                .deposit_entire_worktop(account)
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
    let mut ledger = LedgerSimulatorBuilder::new().build();
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
                            allow_recover: false,
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
                            claimant: user_account1.into(),
                            try_direct_send: false,
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
                    claimant: user_account1.into(),
                    resource_address: XRD.into(),
                    amount: dec!(10_000),
                },
            )
            .deposit_entire_worktop(user_account1)
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
    let mut ledger = LedgerSimulatorBuilder::new().build();
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
                            allow_recover: false,
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
                            claimant: user_account1.into(),
                            try_direct_send: false,
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
                    claimant: user_account1.into(),
                    resource_address: XRD.into(),
                    amount: dec!(10_000),
                },
            )
            .deposit_entire_worktop(user_account1)
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
    let mut ledger = LedgerSimulatorBuilder::new().build();
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
                            allow_recover: true,
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
                            claimant: user_account1.into(),
                            try_direct_send: false,
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
                    claimant: user_account1.into(),
                    resource_address: XRD.into(),
                    amount: dec!(10_000),
                },
            )
            .deposit_entire_worktop(badge_holder_account)
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
    let mut ledger = LedgerSimulatorBuilder::new().build();
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
                            allow_recover: false,
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
                            claimant: user_account1.into(),
                            try_direct_send: false,
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
                    claimant: user_account1.into(),
                    resource_address: XRD.into(),
                    amount: dec!(10_000),
                },
            )
            .deposit_entire_worktop(badge_holder_account)
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
    let mut ledger = LedgerSimulatorBuilder::new().build();
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
                            allow_recover: false,
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
                            claimant: user_account1.into(),
                            try_direct_send: false,
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
                    claimant: user_account1.into(),
                    resource_address: XRD.into(),
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
    let mut ledger = LedgerSimulatorBuilder::new().build();
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
                            allow_recover: false,
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
                            claimant: user_account1.into(),
                            try_direct_send: false,
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
                    claimant: user_account1.into(),
                    resource_address: non_fungible_resource.into(),
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

#[test]
fn state_of_the_account_locker_can_be_reconciled_from_events_alone() {
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (badge_holder_account_public_key, _, badge_holder_account) = ledger.new_account(false);

    let [(user_account1_public_key, _, user_account1), (user_account2_public_key, _, user_account2), (user_account3_public_key, _, user_account3)] =
        std::array::from_fn(|_| ledger.new_account(false));
    let [fungible_resource1, fungible_resource2, fungible_resource3] = std::array::from_fn(|_| {
        ledger.create_freely_mintable_and_burnable_fungible_resource(
            OwnerRole::None,
            None,
            18,
            badge_holder_account,
        )
    });
    let [non_fungible_resource1, non_fungible_resource2, non_fungible_resource3] =
        std::array::from_fn(|_| {
            ledger.create_freely_mintable_and_burnable_non_fungible_resource(
                OwnerRole::None,
                NonFungibleIdType::Integer,
                None::<Vec<(_, ())>>,
                badge_holder_account,
            )
        });

    trait ManifestBuilderExt {
        fn set_disallowed_preference(
            self,
            account: ComponentAddress,
            resource: ResourceAddress,
        ) -> Self;
    }

    impl ManifestBuilderExt for ManifestBuilder {
        fn set_disallowed_preference(
            self,
            account: ComponentAddress,
            resource: ResourceAddress,
        ) -> Self {
            self.call_method(
                account,
                ACCOUNT_SET_RESOURCE_PREFERENCE_IDENT,
                AccountSetResourcePreferenceInput {
                    resource_address: resource.into(),
                    resource_preference: ResourcePreference::Disallowed,
                },
            )
        }
    }

    ledger
        .execute_manifest_with_enabled_modules(
            ManifestBuilder::new()
                .set_disallowed_preference(user_account1, fungible_resource1)
                .set_disallowed_preference(user_account1, non_fungible_resource3)
                .set_disallowed_preference(user_account2, fungible_resource2)
                .set_disallowed_preference(user_account2, non_fungible_resource2)
                .set_disallowed_preference(user_account3, fungible_resource3)
                .set_disallowed_preference(user_account3, non_fungible_resource1)
                .build(),
            true,
            true,
        )
        .expect_commit_success();

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
                            allow_recover: true,
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

    // A vector of actions to perform and what the state after them is expected to be.
    let action_and_state_after = vec![
        //=======
        // Store
        //=======
        Item {
            action: LockerAction::Store {
                claimant: user_account1.into(),
                resource_to_mint: fungible_resource1,
                items_to_mint: ResourceSpecifier::Fungible(dec!(1)),
            },
            state_after: btreemap! {
                user_account1 => btreemap! {
                    fungible_resource1 => ResourceSpecifier::Fungible(dec!(1))
                }
            },
        },
        Item {
            action: LockerAction::Store {
                claimant: user_account2.into(),
                resource_to_mint: fungible_resource1,
                items_to_mint: ResourceSpecifier::Fungible(dec!(1)),
            },
            state_after: btreemap! {
                user_account1 => btreemap! {
                    fungible_resource1 => ResourceSpecifier::Fungible(dec!(1))
                },
                user_account2 => btreemap! {
                    fungible_resource1 => ResourceSpecifier::Fungible(dec!(1))
                }
            },
        },
        //==============
        // Store Batch
        //==============
        Item {
            action: LockerAction::StoreBatch {
                claimants: btreemap! {
                    user_account1 => ResourceSpecifier::Fungible(dec!(1)),
                    user_account2 => ResourceSpecifier::Fungible(dec!(1)),
                    user_account3 => ResourceSpecifier::Fungible(dec!(1)),
                },
                resource_to_mint: fungible_resource2,
            },
            state_after: btreemap! {
                user_account1 => btreemap! {
                    fungible_resource1 => ResourceSpecifier::Fungible(dec!(1)),
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(1)),
                },
                user_account2 => btreemap! {
                    fungible_resource1 => ResourceSpecifier::Fungible(dec!(1)),
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(1))
                },
                user_account3 => btreemap! {
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(1))
                },
            },
        },
        //===============
        // Send or Store
        //===============
        Item {
            action: LockerAction::SendOrStore {
                claimant: user_account1.into(),
                resource_to_mint: fungible_resource1,
                items_to_mint: ResourceSpecifier::Fungible(dec!(1)),
            },
            state_after: btreemap! {
                // User Account 1 rejects the deposits of fungible resource 1. So, the locker stores
                // it.
                user_account1 => btreemap! {
                    fungible_resource1 => ResourceSpecifier::Fungible(dec!(2)),
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(1)),
                },
                user_account2 => btreemap! {
                    fungible_resource1 => ResourceSpecifier::Fungible(dec!(1)),
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(1))
                },
                user_account3 => btreemap! {
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(1))
                },
            },
        },
        Item {
            action: LockerAction::SendOrStore {
                claimant: user_account1.into(),
                resource_to_mint: fungible_resource2,
                items_to_mint: ResourceSpecifier::Fungible(dec!(1)),
            },
            state_after: btreemap! {
                user_account1 => btreemap! {
                    fungible_resource1 => ResourceSpecifier::Fungible(dec!(2)),
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(1)),
                },
                user_account2 => btreemap! {
                    fungible_resource1 => ResourceSpecifier::Fungible(dec!(1)),
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(1))
                },
                user_account3 => btreemap! {
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(1))
                },
            },
        },
        Item {
            action: LockerAction::SendOrStore {
                claimant: user_account1.into(),
                resource_to_mint: fungible_resource3,
                items_to_mint: ResourceSpecifier::Fungible(dec!(1)),
            },
            state_after: btreemap! {
                user_account1 => btreemap! {
                    fungible_resource1 => ResourceSpecifier::Fungible(dec!(2)),
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(1)),
                },
                user_account2 => btreemap! {
                    fungible_resource1 => ResourceSpecifier::Fungible(dec!(1)),
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(1))
                },
                user_account3 => btreemap! {
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(1))
                },
            },
        },
        Item {
            action: LockerAction::SendOrStore {
                claimant: user_account2.into(),
                resource_to_mint: fungible_resource1,
                items_to_mint: ResourceSpecifier::Fungible(dec!(1)),
            },
            state_after: btreemap! {
                user_account1 => btreemap! {
                    fungible_resource1 => ResourceSpecifier::Fungible(dec!(2)),
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(1)),
                },
                user_account2 => btreemap! {
                    fungible_resource1 => ResourceSpecifier::Fungible(dec!(1)),
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(1))
                },
                user_account3 => btreemap! {
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(1))
                },
            },
        },
        Item {
            action: LockerAction::SendOrStore {
                claimant: user_account2.into(),
                resource_to_mint: fungible_resource2,
                items_to_mint: ResourceSpecifier::Fungible(dec!(1)),
            },
            state_after: btreemap! {
                user_account1 => btreemap! {
                    fungible_resource1 => ResourceSpecifier::Fungible(dec!(2)),
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(1)),
                },
                // User Account 2 rejects the deposits of fungible resource 2. So, the locker stores
                // it.
                user_account2 => btreemap! {
                    fungible_resource1 => ResourceSpecifier::Fungible(dec!(1)),
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(2))
                },
                user_account3 => btreemap! {
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(1))
                },
            },
        },
        Item {
            action: LockerAction::SendOrStore {
                claimant: user_account2.into(),
                resource_to_mint: fungible_resource3,
                items_to_mint: ResourceSpecifier::Fungible(dec!(1)),
            },
            state_after: btreemap! {
                user_account1 => btreemap! {
                    fungible_resource1 => ResourceSpecifier::Fungible(dec!(2)),
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(1)),
                },
                user_account2 => btreemap! {
                    fungible_resource1 => ResourceSpecifier::Fungible(dec!(1)),
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(2))
                },
                user_account3 => btreemap! {
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(1))
                },
            },
        },
        Item {
            action: LockerAction::SendOrStore {
                claimant: user_account3.into(),
                resource_to_mint: fungible_resource1,
                items_to_mint: ResourceSpecifier::Fungible(dec!(1)),
            },
            state_after: btreemap! {
                user_account1 => btreemap! {
                    fungible_resource1 => ResourceSpecifier::Fungible(dec!(2)),
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(1)),
                },
                user_account2 => btreemap! {
                    fungible_resource1 => ResourceSpecifier::Fungible(dec!(1)),
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(2))
                },
                user_account3 => btreemap! {
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(1))
                },
            },
        },
        Item {
            action: LockerAction::SendOrStore {
                claimant: user_account3.into(),
                resource_to_mint: fungible_resource2,
                items_to_mint: ResourceSpecifier::Fungible(dec!(1)),
            },
            state_after: btreemap! {
                user_account1 => btreemap! {
                    fungible_resource1 => ResourceSpecifier::Fungible(dec!(2)),
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(1)),
                },
                user_account2 => btreemap! {
                    fungible_resource1 => ResourceSpecifier::Fungible(dec!(1)),
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(2))
                },
                user_account3 => btreemap! {
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(1))
                },
            },
        },
        Item {
            action: LockerAction::SendOrStore {
                claimant: user_account3.into(),
                resource_to_mint: fungible_resource3,
                items_to_mint: ResourceSpecifier::Fungible(dec!(1)),
            },
            state_after: btreemap! {
                user_account1 => btreemap! {
                    fungible_resource1 => ResourceSpecifier::Fungible(dec!(2)),
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(1)),
                },
                user_account2 => btreemap! {
                    fungible_resource1 => ResourceSpecifier::Fungible(dec!(1)),
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(2))
                },
                // User Account 3 rejects the deposits of fungible resource 3. So, the locker stores
                // it.
                user_account3 => btreemap! {
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(1)),
                    fungible_resource3 => ResourceSpecifier::Fungible(dec!(1)),
                },
            },
        },
        //=====================
        // Send or Store Batch
        //=====================
        Item {
            action: LockerAction::SendOrStoreBatch {
                claimants: btreemap! {
                    user_account1 => ResourceSpecifier::Fungible(dec!(1)),
                    user_account2 => ResourceSpecifier::Fungible(dec!(1)),
                    user_account3 => ResourceSpecifier::Fungible(dec!(1)),
                },
                resource_to_mint: fungible_resource1,
            },
            state_after: btreemap! {
                // User Account 1 rejects the deposits of fungible resource 1. So, the locker stores
                // it.
                user_account1 => btreemap! {
                    fungible_resource1 => ResourceSpecifier::Fungible(dec!(3)),
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(1)),
                },
                user_account2 => btreemap! {
                    fungible_resource1 => ResourceSpecifier::Fungible(dec!(1)),
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(2))
                },
                user_account3 => btreemap! {
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(1)),
                    fungible_resource3 => ResourceSpecifier::Fungible(dec!(1)),
                },
            },
        },
        Item {
            action: LockerAction::SendOrStoreBatch {
                claimants: btreemap! {
                    user_account1 => ResourceSpecifier::Fungible(dec!(1)),
                    user_account2 => ResourceSpecifier::Fungible(dec!(1)),
                    user_account3 => ResourceSpecifier::Fungible(dec!(1)),
                },
                resource_to_mint: fungible_resource2,
            },
            state_after: btreemap! {
                user_account1 => btreemap! {
                    fungible_resource1 => ResourceSpecifier::Fungible(dec!(3)),
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(1)),
                },
                // User Account 2 rejects the deposits of fungible resource 2. So, the locker stores
                // it.
                user_account2 => btreemap! {
                    fungible_resource1 => ResourceSpecifier::Fungible(dec!(1)),
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(3))
                },
                user_account3 => btreemap! {
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(1)),
                    fungible_resource3 => ResourceSpecifier::Fungible(dec!(1)),
                },
            },
        },
        Item {
            action: LockerAction::SendOrStoreBatch {
                claimants: btreemap! {
                    user_account1 => ResourceSpecifier::Fungible(dec!(1)),
                    user_account2 => ResourceSpecifier::Fungible(dec!(1)),
                    user_account3 => ResourceSpecifier::Fungible(dec!(1)),
                },
                resource_to_mint: fungible_resource3,
            },
            state_after: btreemap! {
                user_account1 => btreemap! {
                    fungible_resource1 => ResourceSpecifier::Fungible(dec!(3)),
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(1)),
                },
                user_account2 => btreemap! {
                    fungible_resource1 => ResourceSpecifier::Fungible(dec!(1)),
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(3))
                },
                // User Account 3 rejects the deposits of fungible resource 3. So, the locker stores
                // it.
                user_account3 => btreemap! {
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(1)),
                    fungible_resource3 => ResourceSpecifier::Fungible(dec!(2)),
                },
            },
        },
        //=========
        // Recover
        //=========
        Item {
            action: LockerAction::Recover {
                claimant: user_account1.into(),
                resource_to_recover: fungible_resource1,
                items_to_recover: ResourceSpecifier::Fungible(dec!(1)),
            },
            state_after: btreemap! {
                user_account1 => btreemap! {
                    fungible_resource1 => ResourceSpecifier::Fungible(dec!(2)),
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(1)),
                },
                user_account2 => btreemap! {
                    fungible_resource1 => ResourceSpecifier::Fungible(dec!(1)),
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(3))
                },
                user_account3 => btreemap! {
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(1)),
                    fungible_resource3 => ResourceSpecifier::Fungible(dec!(2)),
                },
            },
        },
        //=======
        // Claim
        //=======
        Item {
            action: LockerAction::Claim {
                claimant: user_account2.into(),
                resource_to_claim: fungible_resource2,
                items_to_claim: ResourceSpecifier::Fungible(dec!(1)),
            },
            state_after: btreemap! {
                user_account1 => btreemap! {
                    fungible_resource1 => ResourceSpecifier::Fungible(dec!(2)),
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(1)),
                },
                user_account2 => btreemap! {
                    fungible_resource1 => ResourceSpecifier::Fungible(dec!(1)),
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(2))
                },
                user_account3 => btreemap! {
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(1)),
                    fungible_resource3 => ResourceSpecifier::Fungible(dec!(2)),
                },
            },
        },
        //=======
        // Store
        //=======
        Item {
            action: LockerAction::Store {
                claimant: user_account1.into(),
                resource_to_mint: non_fungible_resource1,
                items_to_mint: ResourceSpecifier::NonFungible(indexset!(
                    NonFungibleLocalId::integer(1),
                    NonFungibleLocalId::integer(2),
                )),
            },
            state_after: btreemap! {
                user_account1 => btreemap! {
                    fungible_resource1 => ResourceSpecifier::Fungible(dec!(2)),
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(1)),
                    non_fungible_resource1 => ResourceSpecifier::NonFungible(indexset!(
                            NonFungibleLocalId::integer(1),
                            NonFungibleLocalId::integer(2),
                        ),
                    )
                },
                user_account2 => btreemap! {
                    fungible_resource1 => ResourceSpecifier::Fungible(dec!(1)),
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(2))
                },
                user_account3 => btreemap! {
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(1)),
                    fungible_resource3 => ResourceSpecifier::Fungible(dec!(2)),
                },
            },
        },
        //===============
        // Send or Store
        //===============
        Item {
            action: LockerAction::SendOrStore {
                claimant: user_account1.into(),
                resource_to_mint: non_fungible_resource1,
                items_to_mint: ResourceSpecifier::NonFungible(indexset!(
                    NonFungibleLocalId::integer(3),
                    NonFungibleLocalId::integer(4),
                )),
            },
            state_after: btreemap! {
                user_account1 => btreemap! {
                    fungible_resource1 => ResourceSpecifier::Fungible(dec!(2)),
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(1)),
                    non_fungible_resource1 => ResourceSpecifier::NonFungible(indexset!(
                            NonFungibleLocalId::integer(1),
                            NonFungibleLocalId::integer(2),
                        ),
                    )
                },
                user_account2 => btreemap! {
                    fungible_resource1 => ResourceSpecifier::Fungible(dec!(1)),
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(2))
                },
                user_account3 => btreemap! {
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(1)),
                    fungible_resource3 => ResourceSpecifier::Fungible(dec!(2)),
                },
            },
        },
        Item {
            action: LockerAction::SendOrStore {
                claimant: user_account2.into(),
                resource_to_mint: non_fungible_resource1,
                items_to_mint: ResourceSpecifier::NonFungible(indexset!(
                    NonFungibleLocalId::integer(5),
                    NonFungibleLocalId::integer(6),
                )),
            },
            state_after: btreemap! {
                user_account1 => btreemap! {
                    fungible_resource1 => ResourceSpecifier::Fungible(dec!(2)),
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(1)),
                    non_fungible_resource1 => ResourceSpecifier::NonFungible(indexset!(
                            NonFungibleLocalId::integer(1),
                            NonFungibleLocalId::integer(2),
                        ),
                    )
                },
                user_account2 => btreemap! {
                    fungible_resource1 => ResourceSpecifier::Fungible(dec!(1)),
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(2)),
                },
                user_account3 => btreemap! {
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(1)),
                    fungible_resource3 => ResourceSpecifier::Fungible(dec!(2)),
                },
            },
        },
        Item {
            action: LockerAction::SendOrStore {
                claimant: user_account3.into(),
                resource_to_mint: non_fungible_resource1,
                items_to_mint: ResourceSpecifier::NonFungible(indexset!(
                    NonFungibleLocalId::integer(7),
                    NonFungibleLocalId::integer(8),
                )),
            },
            state_after: btreemap! {
                user_account1 => btreemap! {
                    fungible_resource1 => ResourceSpecifier::Fungible(dec!(2)),
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(1)),
                    non_fungible_resource1 => ResourceSpecifier::NonFungible(indexset!(
                            NonFungibleLocalId::integer(1),
                            NonFungibleLocalId::integer(2),
                        ),
                    )
                },
                user_account2 => btreemap! {
                    fungible_resource1 => ResourceSpecifier::Fungible(dec!(1)),
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(2)),
                },
                // User Account 3 rejects the deposits of non-fungible resource 3. So, the locker
                // stores it.
                user_account3 => btreemap! {
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(1)),
                    fungible_resource3 => ResourceSpecifier::Fungible(dec!(2)),
                    non_fungible_resource1 => ResourceSpecifier::NonFungible(indexset!(
                            NonFungibleLocalId::integer(7),
                            NonFungibleLocalId::integer(8),
                        ),
                    )
                },
            },
        },
        Item {
            action: LockerAction::SendOrStore {
                claimant: user_account1.into(),
                resource_to_mint: non_fungible_resource2,
                items_to_mint: ResourceSpecifier::NonFungible(indexset!(
                    NonFungibleLocalId::integer(1),
                    NonFungibleLocalId::integer(2),
                )),
            },
            state_after: btreemap! {
                user_account1 => btreemap! {
                    fungible_resource1 => ResourceSpecifier::Fungible(dec!(2)),
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(1)),
                    non_fungible_resource1 => ResourceSpecifier::NonFungible(indexset!(
                            NonFungibleLocalId::integer(1),
                            NonFungibleLocalId::integer(2),
                        ),
                    )
                },
                user_account2 => btreemap! {
                    fungible_resource1 => ResourceSpecifier::Fungible(dec!(1)),
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(2)),
                },
                user_account3 => btreemap! {
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(1)),
                    fungible_resource3 => ResourceSpecifier::Fungible(dec!(2)),
                    non_fungible_resource1 => ResourceSpecifier::NonFungible(indexset!(
                            NonFungibleLocalId::integer(7),
                            NonFungibleLocalId::integer(8),
                        ),
                    )
                },
            },
        },
        Item {
            action: LockerAction::SendOrStore {
                claimant: user_account2.into(),
                resource_to_mint: non_fungible_resource2,
                items_to_mint: ResourceSpecifier::NonFungible(indexset!(
                    NonFungibleLocalId::integer(3),
                    NonFungibleLocalId::integer(4),
                )),
            },
            state_after: btreemap! {
                user_account1 => btreemap! {
                    fungible_resource1 => ResourceSpecifier::Fungible(dec!(2)),
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(1)),
                    non_fungible_resource1 => ResourceSpecifier::NonFungible(indexset!(
                            NonFungibleLocalId::integer(1),
                            NonFungibleLocalId::integer(2),
                        ),
                    )
                },
                // User Account 2 rejects the deposits of non-fungible resource 2. So, the locker
                // stores it.
                user_account2 => btreemap! {
                    fungible_resource1 => ResourceSpecifier::Fungible(dec!(1)),
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(2)),
                    non_fungible_resource2 => ResourceSpecifier::NonFungible(indexset!(
                            NonFungibleLocalId::integer(3),
                            NonFungibleLocalId::integer(4),
                        ),
                    )
                },
                user_account3 => btreemap! {
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(1)),
                    fungible_resource3 => ResourceSpecifier::Fungible(dec!(2)),
                    non_fungible_resource1 => ResourceSpecifier::NonFungible(indexset!(
                            NonFungibleLocalId::integer(7),
                            NonFungibleLocalId::integer(8),
                        ),
                    )
                },
            },
        },
        Item {
            action: LockerAction::SendOrStore {
                claimant: user_account3.into(),
                resource_to_mint: non_fungible_resource2,
                items_to_mint: ResourceSpecifier::NonFungible(indexset!(
                    NonFungibleLocalId::integer(5),
                    NonFungibleLocalId::integer(6),
                )),
            },
            state_after: btreemap! {
                user_account1 => btreemap! {
                    fungible_resource1 => ResourceSpecifier::Fungible(dec!(2)),
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(1)),
                    non_fungible_resource1 => ResourceSpecifier::NonFungible(indexset!(
                            NonFungibleLocalId::integer(1),
                            NonFungibleLocalId::integer(2),
                        ),
                    )
                },
                user_account2 => btreemap! {
                    fungible_resource1 => ResourceSpecifier::Fungible(dec!(1)),
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(2)),
                    non_fungible_resource2 => ResourceSpecifier::NonFungible(indexset!(
                            NonFungibleLocalId::integer(3),
                            NonFungibleLocalId::integer(4),
                        ),
                    )
                },
                user_account3 => btreemap! {
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(1)),
                    fungible_resource3 => ResourceSpecifier::Fungible(dec!(2)),
                    non_fungible_resource1 => ResourceSpecifier::NonFungible(indexset!(
                            NonFungibleLocalId::integer(7),
                            NonFungibleLocalId::integer(8),
                        ),
                    )
                },
            },
        },
        Item {
            action: LockerAction::SendOrStore {
                claimant: user_account1.into(),
                resource_to_mint: non_fungible_resource3,
                items_to_mint: ResourceSpecifier::NonFungible(indexset!(
                    NonFungibleLocalId::integer(1),
                    NonFungibleLocalId::integer(2),
                )),
            },
            state_after: btreemap! {
                // User Account 1 rejects the deposits of non-fungible resource 3. So, the locker
                // stores it.
                user_account1 => btreemap! {
                    fungible_resource1 => ResourceSpecifier::Fungible(dec!(2)),
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(1)),
                    non_fungible_resource1 => ResourceSpecifier::NonFungible(indexset!(
                            NonFungibleLocalId::integer(1),
                            NonFungibleLocalId::integer(2),
                        ),
                    ),
                    non_fungible_resource3 => ResourceSpecifier::NonFungible(indexset!(
                            NonFungibleLocalId::integer(1),
                            NonFungibleLocalId::integer(2),
                        ),
                    )
                },
                user_account2 => btreemap! {
                    fungible_resource1 => ResourceSpecifier::Fungible(dec!(1)),
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(2)),
                    non_fungible_resource2 => ResourceSpecifier::NonFungible(indexset!(
                            NonFungibleLocalId::integer(3),
                            NonFungibleLocalId::integer(4),
                        ),
                    )
                },
                user_account3 => btreemap! {
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(1)),
                    fungible_resource3 => ResourceSpecifier::Fungible(dec!(2)),
                    non_fungible_resource1 => ResourceSpecifier::NonFungible(indexset!(
                            NonFungibleLocalId::integer(7),
                            NonFungibleLocalId::integer(8),
                        ),
                    )
                },
            },
        },
        Item {
            action: LockerAction::SendOrStore {
                claimant: user_account2.into(),
                resource_to_mint: non_fungible_resource3,
                items_to_mint: ResourceSpecifier::NonFungible(indexset!(
                    NonFungibleLocalId::integer(3),
                    NonFungibleLocalId::integer(4),
                )),
            },
            state_after: btreemap! {
                user_account1 => btreemap! {
                    fungible_resource1 => ResourceSpecifier::Fungible(dec!(2)),
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(1)),
                    non_fungible_resource1 => ResourceSpecifier::NonFungible(indexset!(
                            NonFungibleLocalId::integer(1),
                            NonFungibleLocalId::integer(2),
                        ),
                    ),
                    non_fungible_resource3 => ResourceSpecifier::NonFungible(indexset!(
                            NonFungibleLocalId::integer(1),
                            NonFungibleLocalId::integer(2),
                        ),
                    )
                },
                user_account2 => btreemap! {
                    fungible_resource1 => ResourceSpecifier::Fungible(dec!(1)),
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(2)),
                    non_fungible_resource2 => ResourceSpecifier::NonFungible(indexset!(
                            NonFungibleLocalId::integer(3),
                            NonFungibleLocalId::integer(4),
                        ),
                    )
                },
                user_account3 => btreemap! {
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(1)),
                    fungible_resource3 => ResourceSpecifier::Fungible(dec!(2)),
                    non_fungible_resource1 => ResourceSpecifier::NonFungible(indexset!(
                            NonFungibleLocalId::integer(7),
                            NonFungibleLocalId::integer(8),
                        ),
                    )
                },
            },
        },
        Item {
            action: LockerAction::SendOrStore {
                claimant: user_account3.into(),
                resource_to_mint: non_fungible_resource3,
                items_to_mint: ResourceSpecifier::NonFungible(indexset!(
                    NonFungibleLocalId::integer(5),
                    NonFungibleLocalId::integer(6),
                )),
            },
            state_after: btreemap! {
                user_account1 => btreemap! {
                    fungible_resource1 => ResourceSpecifier::Fungible(dec!(2)),
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(1)),
                    non_fungible_resource1 => ResourceSpecifier::NonFungible(indexset!(
                            NonFungibleLocalId::integer(1),
                            NonFungibleLocalId::integer(2),
                        ),
                    ),
                    non_fungible_resource3 => ResourceSpecifier::NonFungible(indexset!(
                            NonFungibleLocalId::integer(1),
                            NonFungibleLocalId::integer(2),
                        ),
                    )
                },
                user_account2 => btreemap! {
                    fungible_resource1 => ResourceSpecifier::Fungible(dec!(1)),
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(2)),
                    non_fungible_resource2 => ResourceSpecifier::NonFungible(indexset!(
                            NonFungibleLocalId::integer(3),
                            NonFungibleLocalId::integer(4),
                        ),
                    )
                },
                user_account3 => btreemap! {
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(1)),
                    fungible_resource3 => ResourceSpecifier::Fungible(dec!(2)),
                    non_fungible_resource1 => ResourceSpecifier::NonFungible(indexset!(
                            NonFungibleLocalId::integer(7),
                            NonFungibleLocalId::integer(8),
                        ),
                    )
                },
            },
        },
        //==============
        // Store Batch
        //==============
        Item {
            action: LockerAction::StoreBatch {
                claimants: btreemap! {
                    user_account1 => ResourceSpecifier::NonFungible(indexset!(
                            NonFungibleLocalId::integer(10),
                            NonFungibleLocalId::integer(11),
                        )
                    ),
                    user_account2 => ResourceSpecifier::NonFungible(indexset!(
                            NonFungibleLocalId::integer(12),
                            NonFungibleLocalId::integer(13),
                        )
                    ),
                    user_account3 => ResourceSpecifier::NonFungible(indexset!(
                            NonFungibleLocalId::integer(14),
                            NonFungibleLocalId::integer(15),
                        )
                    ),
                },
                resource_to_mint: non_fungible_resource1,
            },
            state_after: btreemap! {
                user_account1 => btreemap! {
                    fungible_resource1 => ResourceSpecifier::Fungible(dec!(2)),
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(1)),
                    non_fungible_resource1 => ResourceSpecifier::NonFungible(indexset!(
                            NonFungibleLocalId::integer(1),
                            NonFungibleLocalId::integer(2),
                            NonFungibleLocalId::integer(10),
                            NonFungibleLocalId::integer(11),
                        ),
                    ),
                    non_fungible_resource3 => ResourceSpecifier::NonFungible(indexset!(
                            NonFungibleLocalId::integer(1),
                            NonFungibleLocalId::integer(2),
                        ),
                    )
                },
                user_account2 => btreemap! {
                    fungible_resource1 => ResourceSpecifier::Fungible(dec!(1)),
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(2)),
                    non_fungible_resource1 => ResourceSpecifier::NonFungible(indexset!(
                            NonFungibleLocalId::integer(12),
                            NonFungibleLocalId::integer(13),
                        ),
                    ),
                    non_fungible_resource2 => ResourceSpecifier::NonFungible(indexset!(
                            NonFungibleLocalId::integer(3),
                            NonFungibleLocalId::integer(4),
                        ),
                    )
                },
                user_account3 => btreemap! {
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(1)),
                    fungible_resource3 => ResourceSpecifier::Fungible(dec!(2)),
                    non_fungible_resource1 => ResourceSpecifier::NonFungible(indexset!(
                            NonFungibleLocalId::integer(7),
                            NonFungibleLocalId::integer(8),
                            NonFungibleLocalId::integer(14),
                            NonFungibleLocalId::integer(15),
                        ),
                    )
                },
            },
        },
        //=====================
        // Send or Store Batch
        //=====================
        Item {
            action: LockerAction::SendOrStoreBatch {
                claimants: btreemap! {
                    user_account1 => ResourceSpecifier::NonFungible(indexset!(
                            NonFungibleLocalId::integer(16),
                            NonFungibleLocalId::integer(17),
                        )
                    ),
                    user_account2 => ResourceSpecifier::NonFungible(indexset!(
                            NonFungibleLocalId::integer(18),
                            NonFungibleLocalId::integer(19),
                        )
                    ),
                    user_account3 => ResourceSpecifier::NonFungible(indexset!(
                            NonFungibleLocalId::integer(20),
                            NonFungibleLocalId::integer(21),
                        )
                    ),
                },
                resource_to_mint: non_fungible_resource1,
            },
            state_after: btreemap! {
                user_account1 => btreemap! {
                    fungible_resource1 => ResourceSpecifier::Fungible(dec!(2)),
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(1)),
                    non_fungible_resource1 => ResourceSpecifier::NonFungible(indexset!(
                            NonFungibleLocalId::integer(1),
                            NonFungibleLocalId::integer(2),
                            NonFungibleLocalId::integer(10),
                            NonFungibleLocalId::integer(11),
                        ),
                    ),
                    non_fungible_resource3 => ResourceSpecifier::NonFungible(indexset!(
                            NonFungibleLocalId::integer(1),
                            NonFungibleLocalId::integer(2),
                        ),
                    )
                },
                user_account2 => btreemap! {
                    fungible_resource1 => ResourceSpecifier::Fungible(dec!(1)),
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(2)),
                    non_fungible_resource1 => ResourceSpecifier::NonFungible(indexset!(
                            NonFungibleLocalId::integer(12),
                            NonFungibleLocalId::integer(13),
                        ),
                    ),
                    non_fungible_resource2 => ResourceSpecifier::NonFungible(indexset!(
                            NonFungibleLocalId::integer(3),
                            NonFungibleLocalId::integer(4),
                        ),
                    )
                },
                user_account3 => btreemap! {
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(1)),
                    fungible_resource3 => ResourceSpecifier::Fungible(dec!(2)),
                    non_fungible_resource1 => ResourceSpecifier::NonFungible(indexset!(
                            NonFungibleLocalId::integer(7),
                            NonFungibleLocalId::integer(8),
                            NonFungibleLocalId::integer(14),
                            NonFungibleLocalId::integer(15),
                            NonFungibleLocalId::integer(20),
                            NonFungibleLocalId::integer(21),
                        ),
                    )
                },
            },
        },
        //=========
        // Recover
        //=========
        Item {
            action: LockerAction::Recover {
                claimant: user_account3.into(),
                resource_to_recover: non_fungible_resource1,
                items_to_recover: ResourceSpecifier::NonFungible(indexset!(
                    NonFungibleLocalId::integer(7),
                    NonFungibleLocalId::integer(8),
                )),
            },
            state_after: btreemap! {
                user_account1 => btreemap! {
                    fungible_resource1 => ResourceSpecifier::Fungible(dec!(2)),
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(1)),
                    non_fungible_resource1 => ResourceSpecifier::NonFungible(indexset!(
                            NonFungibleLocalId::integer(1),
                            NonFungibleLocalId::integer(2),
                            NonFungibleLocalId::integer(10),
                            NonFungibleLocalId::integer(11),
                        ),
                    ),
                    non_fungible_resource3 => ResourceSpecifier::NonFungible(indexset!(
                            NonFungibleLocalId::integer(1),
                            NonFungibleLocalId::integer(2),
                        ),
                    )
                },
                user_account2 => btreemap! {
                    fungible_resource1 => ResourceSpecifier::Fungible(dec!(1)),
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(2)),
                    non_fungible_resource1 => ResourceSpecifier::NonFungible(indexset!(
                            NonFungibleLocalId::integer(12),
                            NonFungibleLocalId::integer(13),
                        ),
                    ),
                    non_fungible_resource2 => ResourceSpecifier::NonFungible(indexset!(
                            NonFungibleLocalId::integer(3),
                            NonFungibleLocalId::integer(4),
                        ),
                    )
                },
                user_account3 => btreemap! {
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(1)),
                    fungible_resource3 => ResourceSpecifier::Fungible(dec!(2)),
                    non_fungible_resource1 => ResourceSpecifier::NonFungible(indexset!(
                            NonFungibleLocalId::integer(14),
                            NonFungibleLocalId::integer(15),
                            NonFungibleLocalId::integer(20),
                            NonFungibleLocalId::integer(21),
                        ),
                    )
                },
            },
        },
        Item {
            action: LockerAction::Recover {
                claimant: user_account3.into(),
                resource_to_recover: non_fungible_resource1,
                items_to_recover: ResourceSpecifier::Fungible(dec!(1)),
            },
            state_after: btreemap! {
                user_account1 => btreemap! {
                    fungible_resource1 => ResourceSpecifier::Fungible(dec!(2)),
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(1)),
                    non_fungible_resource1 => ResourceSpecifier::NonFungible(indexset!(
                            NonFungibleLocalId::integer(1),
                            NonFungibleLocalId::integer(2),
                            NonFungibleLocalId::integer(10),
                            NonFungibleLocalId::integer(11),
                        ),
                    ),
                    non_fungible_resource3 => ResourceSpecifier::NonFungible(indexset!(
                            NonFungibleLocalId::integer(1),
                            NonFungibleLocalId::integer(2),
                        ),
                    )
                },
                user_account2 => btreemap! {
                    fungible_resource1 => ResourceSpecifier::Fungible(dec!(1)),
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(2)),
                    non_fungible_resource1 => ResourceSpecifier::NonFungible(indexset!(
                            NonFungibleLocalId::integer(12),
                            NonFungibleLocalId::integer(13),
                        ),
                    ),
                    non_fungible_resource2 => ResourceSpecifier::NonFungible(indexset!(
                            NonFungibleLocalId::integer(3),
                            NonFungibleLocalId::integer(4),
                        ),
                    )
                },
                user_account3 => btreemap! {
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(1)),
                    fungible_resource3 => ResourceSpecifier::Fungible(dec!(2)),
                    non_fungible_resource1 => ResourceSpecifier::NonFungible(indexset!(
                            NonFungibleLocalId::integer(14),
                            NonFungibleLocalId::integer(15),
                            NonFungibleLocalId::integer(20),
                        ),
                    )
                },
            },
        },
        //=======
        // Claim
        //=======
        Item {
            action: LockerAction::Claim {
                claimant: user_account3.into(),
                resource_to_claim: non_fungible_resource1,
                items_to_claim: ResourceSpecifier::NonFungible(indexset!(
                    NonFungibleLocalId::integer(20)
                )),
            },
            state_after: btreemap! {
                user_account1 => btreemap! {
                    fungible_resource1 => ResourceSpecifier::Fungible(dec!(2)),
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(1)),
                    non_fungible_resource1 => ResourceSpecifier::NonFungible(indexset!(
                            NonFungibleLocalId::integer(1),
                            NonFungibleLocalId::integer(2),
                            NonFungibleLocalId::integer(10),
                            NonFungibleLocalId::integer(11),
                        ),
                    ),
                    non_fungible_resource3 => ResourceSpecifier::NonFungible(indexset!(
                            NonFungibleLocalId::integer(1),
                            NonFungibleLocalId::integer(2),
                        ),
                    )
                },
                user_account2 => btreemap! {
                    fungible_resource1 => ResourceSpecifier::Fungible(dec!(1)),
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(2)),
                    non_fungible_resource1 => ResourceSpecifier::NonFungible(indexset!(
                            NonFungibleLocalId::integer(12),
                            NonFungibleLocalId::integer(13),
                        ),
                    ),
                    non_fungible_resource2 => ResourceSpecifier::NonFungible(indexset!(
                            NonFungibleLocalId::integer(3),
                            NonFungibleLocalId::integer(4),
                        ),
                    )
                },
                user_account3 => btreemap! {
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(1)),
                    fungible_resource3 => ResourceSpecifier::Fungible(dec!(2)),
                    non_fungible_resource1 => ResourceSpecifier::NonFungible(indexset!(
                            NonFungibleLocalId::integer(14),
                            NonFungibleLocalId::integer(15),
                        ),
                    )
                },
            },
        },
        Item {
            action: LockerAction::Claim {
                claimant: user_account3.into(),
                resource_to_claim: non_fungible_resource1,
                items_to_claim: ResourceSpecifier::Fungible(dec!(1)),
            },
            state_after: btreemap! {
                user_account1 => btreemap! {
                    fungible_resource1 => ResourceSpecifier::Fungible(dec!(2)),
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(1)),
                    non_fungible_resource1 => ResourceSpecifier::NonFungible(indexset!(
                            NonFungibleLocalId::integer(1),
                            NonFungibleLocalId::integer(2),
                            NonFungibleLocalId::integer(10),
                            NonFungibleLocalId::integer(11),
                        ),
                    ),
                    non_fungible_resource3 => ResourceSpecifier::NonFungible(indexset!(
                            NonFungibleLocalId::integer(1),
                            NonFungibleLocalId::integer(2),
                        ),
                    )
                },
                user_account2 => btreemap! {
                    fungible_resource1 => ResourceSpecifier::Fungible(dec!(1)),
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(2)),
                    non_fungible_resource1 => ResourceSpecifier::NonFungible(indexset!(
                            NonFungibleLocalId::integer(12),
                            NonFungibleLocalId::integer(13),
                        ),
                    ),
                    non_fungible_resource2 => ResourceSpecifier::NonFungible(indexset!(
                            NonFungibleLocalId::integer(3),
                            NonFungibleLocalId::integer(4),
                        ),
                    )
                },
                user_account3 => btreemap! {
                    fungible_resource2 => ResourceSpecifier::Fungible(dec!(1)),
                    fungible_resource3 => ResourceSpecifier::Fungible(dec!(2)),
                    non_fungible_resource1 => ResourceSpecifier::NonFungible(indexset!(
                            NonFungibleLocalId::integer(14),
                        ),
                    )
                },
            },
        },
    ];

    let mut state_reconciled_from_events =
        BTreeMap::<ComponentAddress, BTreeMap<ResourceAddress, ResourceSpecifier>>::new();
    for Item {
        action,
        state_after,
    } in action_and_state_after
    {
        // Perform the action
        let receipt = match action {
            // Mint the resources  and store them in the account locker.
            LockerAction::Store {
                claimant,
                resource_to_mint,
                items_to_mint: ResourceSpecifier::Fungible(amount),
            } => ledger.execute_manifest(
                ManifestBuilder::new()
                    .lock_fee_from_faucet()
                    .create_proof_from_account_of_amount(
                        badge_holder_account,
                        account_locker_badge,
                        dec!(1),
                    )
                    .mint_fungible(resource_to_mint, amount)
                    .take_all_from_worktop(resource_to_mint, "bucket")
                    .with_bucket("bucket", |builder, bucket| {
                        builder.call_method(
                            account_locker,
                            ACCOUNT_LOCKER_STORE_IDENT,
                            AccountLockerStoreManifestInput {
                                bucket,
                                claimant: claimant.into(),
                                try_direct_send: false,
                            },
                        )
                    })
                    .build(),
                vec![NonFungibleGlobalId::from_public_key(
                    &badge_holder_account_public_key,
                )],
            ),
            LockerAction::Store {
                claimant,
                resource_to_mint,
                items_to_mint: ResourceSpecifier::NonFungible(ids),
            } => ledger.execute_manifest(
                ManifestBuilder::new()
                    .lock_fee_from_faucet()
                    .create_proof_from_account_of_amount(
                        badge_holder_account,
                        account_locker_badge,
                        dec!(1),
                    )
                    .mint_non_fungible(resource_to_mint, ids.into_iter().map(|id| (id, ())))
                    .take_all_from_worktop(resource_to_mint, "bucket")
                    .with_bucket("bucket", |builder, bucket| {
                        builder.call_method(
                            account_locker,
                            ACCOUNT_LOCKER_STORE_IDENT,
                            AccountLockerStoreManifestInput {
                                bucket,
                                claimant: claimant.into(),
                                try_direct_send: false,
                            },
                        )
                    })
                    .build(),
                vec![NonFungibleGlobalId::from_public_key(
                    &badge_holder_account_public_key,
                )],
            ),
            LockerAction::SendOrStore {
                claimant,
                resource_to_mint,
                items_to_mint: ResourceSpecifier::Fungible(amount),
            } => ledger.execute_manifest(
                ManifestBuilder::new()
                    .lock_fee_from_faucet()
                    .create_proof_from_account_of_amount(
                        badge_holder_account,
                        account_locker_badge,
                        dec!(1),
                    )
                    .mint_fungible(resource_to_mint, amount)
                    .take_all_from_worktop(resource_to_mint, "bucket")
                    .with_bucket("bucket", |builder, bucket| {
                        builder.call_method(
                            account_locker,
                            ACCOUNT_LOCKER_STORE_IDENT,
                            AccountLockerStoreManifestInput {
                                bucket,
                                claimant: claimant.into(),
                                try_direct_send: true,
                            },
                        )
                    })
                    .build(),
                vec![NonFungibleGlobalId::from_public_key(
                    &badge_holder_account_public_key,
                )],
            ),
            LockerAction::SendOrStore {
                claimant,
                resource_to_mint,
                items_to_mint: ResourceSpecifier::NonFungible(ids),
            } => ledger.execute_manifest(
                ManifestBuilder::new()
                    .lock_fee_from_faucet()
                    .create_proof_from_account_of_amount(
                        badge_holder_account,
                        account_locker_badge,
                        dec!(1),
                    )
                    .mint_non_fungible(resource_to_mint, ids.into_iter().map(|id| (id, ())))
                    .take_all_from_worktop(resource_to_mint, "bucket")
                    .with_bucket("bucket", |builder, bucket| {
                        builder.call_method(
                            account_locker,
                            ACCOUNT_LOCKER_STORE_IDENT,
                            AccountLockerStoreManifestInput {
                                bucket,
                                claimant: claimant.into(),
                                try_direct_send: true,
                            },
                        )
                    })
                    .build(),
                vec![NonFungibleGlobalId::from_public_key(
                    &badge_holder_account_public_key,
                )],
            ),
            LockerAction::StoreBatch {
                claimants,
                resource_to_mint,
            } => ledger.execute_manifest(
                ManifestBuilder::new()
                    .lock_fee_from_faucet()
                    .create_proof_from_account_of_amount(
                        badge_holder_account,
                        account_locker_badge,
                        dec!(1),
                    )
                    .then(|builder| {
                        claimants.values().fold(builder, |acc, item| match item {
                            ResourceSpecifier::Fungible(amount) => {
                                acc.mint_fungible(resource_to_mint, *amount)
                            }
                            ResourceSpecifier::NonFungible(ids) => acc.mint_non_fungible(
                                resource_to_mint,
                                ids.into_iter().map(|id| (id.clone(), ())),
                            ),
                        })
                    })
                    .take_all_from_worktop(resource_to_mint, "bucket")
                    .with_bucket("bucket", |builder, bucket| {
                        builder.call_method(
                            account_locker,
                            ACCOUNT_LOCKER_AIRDROP_IDENT,
                            AccountLockerAirdropManifestInput {
                                bucket,
                                claimants: claimants
                                    .into_iter()
                                    .map(|(k, v)| (k.into(), v))
                                    .collect(),
                                try_direct_send: false,
                            },
                        )
                    })
                    .build(),
                vec![NonFungibleGlobalId::from_public_key(
                    &badge_holder_account_public_key,
                )],
            ),
            LockerAction::SendOrStoreBatch {
                claimants,
                resource_to_mint,
            } => ledger.execute_manifest(
                ManifestBuilder::new()
                    .lock_fee_from_faucet()
                    .create_proof_from_account_of_amount(
                        badge_holder_account,
                        account_locker_badge,
                        dec!(1),
                    )
                    .then(|builder| {
                        claimants.values().fold(builder, |acc, item| match item {
                            ResourceSpecifier::Fungible(amount) => {
                                acc.mint_fungible(resource_to_mint, *amount)
                            }
                            ResourceSpecifier::NonFungible(ids) => acc.mint_non_fungible(
                                resource_to_mint,
                                ids.into_iter().map(|id| (id.clone(), ())),
                            ),
                        })
                    })
                    .take_all_from_worktop(resource_to_mint, "bucket")
                    .with_bucket("bucket", |builder, bucket| {
                        builder.call_method(
                            account_locker,
                            ACCOUNT_LOCKER_AIRDROP_IDENT,
                            AccountLockerAirdropManifestInput {
                                bucket,
                                claimants: claimants
                                    .into_iter()
                                    .map(|(k, v)| (k.into(), v))
                                    .collect(),
                                try_direct_send: true,
                            },
                        )
                    })
                    .build(),
                vec![NonFungibleGlobalId::from_public_key(
                    &badge_holder_account_public_key,
                )],
            ),
            LockerAction::Recover {
                claimant,
                resource_to_recover,
                items_to_recover: ResourceSpecifier::Fungible(amount),
            } => ledger.execute_manifest(
                ManifestBuilder::new()
                    .lock_fee_from_faucet()
                    .create_proof_from_account_of_amount(
                        badge_holder_account,
                        account_locker_badge,
                        dec!(1),
                    )
                    .call_method(
                        account_locker,
                        ACCOUNT_LOCKER_RECOVER_IDENT,
                        AccountLockerRecoverManifestInput {
                            claimant: claimant.into(),
                            resource_address: resource_to_recover.into(),
                            amount,
                        },
                    )
                    .try_deposit_entire_worktop_or_abort(badge_holder_account, None)
                    .build(),
                vec![NonFungibleGlobalId::from_public_key(
                    &badge_holder_account_public_key,
                )],
            ),
            LockerAction::Recover {
                claimant,
                resource_to_recover,
                items_to_recover: ResourceSpecifier::NonFungible(ids),
            } => ledger.execute_manifest(
                ManifestBuilder::new()
                    .lock_fee_from_faucet()
                    .create_proof_from_account_of_amount(
                        badge_holder_account,
                        account_locker_badge,
                        dec!(1),
                    )
                    .call_method(
                        account_locker,
                        ACCOUNT_LOCKER_RECOVER_NON_FUNGIBLES_IDENT,
                        AccountLockerRecoverNonFungiblesManifestInput {
                            claimant: claimant.into(),
                            resource_address: resource_to_recover.into(),
                            ids,
                        },
                    )
                    .try_deposit_entire_worktop_or_abort(badge_holder_account, None)
                    .build(),
                vec![NonFungibleGlobalId::from_public_key(
                    &badge_holder_account_public_key,
                )],
            ),
            LockerAction::Claim {
                claimant,
                resource_to_claim,
                items_to_claim: ResourceSpecifier::Fungible(amount),
            } => ledger.execute_manifest(
                ManifestBuilder::new()
                    .lock_fee_from_faucet()
                    .call_method(
                        account_locker,
                        ACCOUNT_LOCKER_CLAIM_IDENT,
                        AccountLockerClaimManifestInput {
                            claimant: claimant.into(),
                            resource_address: resource_to_claim.into(),
                            amount,
                        },
                    )
                    .deposit_entire_worktop(claimant)
                    .build(),
                [
                    &user_account1_public_key,
                    &user_account2_public_key,
                    &user_account3_public_key,
                ]
                .map(NonFungibleGlobalId::from_public_key),
            ),
            LockerAction::Claim {
                claimant,
                resource_to_claim,
                items_to_claim: ResourceSpecifier::NonFungible(ids),
            } => ledger.execute_manifest(
                ManifestBuilder::new()
                    .lock_fee_from_faucet()
                    .call_method(
                        account_locker,
                        ACCOUNT_LOCKER_CLAIM_NON_FUNGIBLES_IDENT,
                        AccountLockerClaimNonFungiblesManifestInput {
                            claimant: claimant.into(),
                            resource_address: resource_to_claim.into(),
                            ids,
                        },
                    )
                    .deposit_entire_worktop(claimant)
                    .build(),
                [
                    &user_account1_public_key,
                    &user_account2_public_key,
                    &user_account3_public_key,
                ]
                .map(NonFungibleGlobalId::from_public_key),
            ),
        };
        receipt.expect_commit_success();

        // Reconcile the state from events.
        let events = receipt
            .expect_commit_success()
            .application_events
            .iter()
            .filter_map(|(EventTypeIdentifier(emitter, event_name), event_data)| {
                AccountLockerEvent::new(emitter, event_name, event_data)
            })
            .collect::<Vec<_>>();

        for event in events {
            match event {
                AccountLockerEvent::StoreEvent(StoreEvent {
                    claimant,
                    resource_address,
                    resources,
                }) => {
                    let entry = state_reconciled_from_events
                        .entry(claimant.0)
                        .or_default()
                        .entry(resource_address)
                        .or_insert(ResourceSpecifier::new_empty(resource_address));
                    *entry = entry.checked_add(&resources).expect("Can't fail!");
                }
                AccountLockerEvent::RecoveryEvent(RecoverEvent {
                    claimant,
                    resource_address,
                    resources,
                })
                | AccountLockerEvent::ClaimEvent(ClaimEvent {
                    claimant,
                    resource_address,
                    resources,
                }) => {
                    let entry = state_reconciled_from_events
                        .entry(claimant.0)
                        .or_default()
                        .entry(resource_address)
                        .or_insert(ResourceSpecifier::new_empty(resource_address));
                    *entry = entry.checked_sub(&resources).expect("Can't fail!");
                }
            }
        }

        // Assert that the state reconciled from the events is the same as what we expect it to
        // be.
        assert_eq!(
            state_reconciled_from_events, state_after,
            "Events State: {:#?}\nExpected: {:#?}",
            state_reconciled_from_events, state_after
        );
    }
}

pub struct Item {
    action: LockerAction,
    state_after: BTreeMap<ComponentAddress, BTreeMap<ResourceAddress, ResourceSpecifier>>,
}

#[derive(Clone, Debug)]
pub enum LockerAction {
    Store {
        claimant: ComponentAddress,
        resource_to_mint: ResourceAddress,
        items_to_mint: ResourceSpecifier,
    },
    SendOrStore {
        claimant: ComponentAddress,
        resource_to_mint: ResourceAddress,
        items_to_mint: ResourceSpecifier,
    },
    StoreBatch {
        claimants: BTreeMap<ComponentAddress, ResourceSpecifier>,
        resource_to_mint: ResourceAddress,
    },
    SendOrStoreBatch {
        claimants: BTreeMap<ComponentAddress, ResourceSpecifier>,
        resource_to_mint: ResourceAddress,
    },
    Recover {
        claimant: ComponentAddress,
        resource_to_recover: ResourceAddress,
        items_to_recover: ResourceSpecifier,
    },
    Claim {
        claimant: ComponentAddress,
        resource_to_claim: ResourceAddress,
        items_to_claim: ResourceSpecifier,
    },
}

#[derive(Clone, Debug)]
pub enum AccountLockerEvent {
    StoreEvent(StoreEvent),
    RecoveryEvent(RecoverEvent),
    ClaimEvent(ClaimEvent),
}

impl AccountLockerEvent {
    pub fn new(
        emitter: &Emitter,
        event_name: &str,
        event_data: &[u8],
    ) -> Option<AccountLockerEvent> {
        if let Emitter::Method(node_id, ModuleId::Main) = emitter {
            if node_id
                .entity_type()
                .is_some_and(|entity_type| entity_type == EntityType::GlobalAccountLocker)
            {
                match event_name {
                    StoreEvent::EVENT_NAME => scrypto_decode(event_data)
                        .map(AccountLockerEvent::StoreEvent)
                        .ok(),
                    ClaimEvent::EVENT_NAME => scrypto_decode(event_data)
                        .map(AccountLockerEvent::ClaimEvent)
                        .ok(),
                    RecoverEvent::EVENT_NAME => scrypto_decode(event_data)
                        .map(AccountLockerEvent::RecoveryEvent)
                        .ok(),
                    _ => None,
                }
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[extend::ext]
pub impl DefaultLedgerSimulator {
    fn execute_manifest_without_auth(
        &mut self,
        manifest: TransactionManifestV1,
    ) -> TransactionReceipt {
        self.execute_manifest_with_enabled_modules(manifest, true, false)
    }

    fn execute_manifest_with_enabled_modules(
        &mut self,
        manifest: TransactionManifestV1,
        disable_auth: bool,
        disable_costing: bool,
    ) -> TransactionReceipt {
        let mut execution_config =
            ExecutionConfig::for_notarized_transaction(NetworkDefinition::mainnet());
        execution_config.system_overrides = Some(SystemOverrides {
            disable_auth,
            disable_costing,
            ..SystemOverrides::with_network(NetworkDefinition::mainnet())
        });

        let nonce = self.next_transaction_nonce();
        let test_transaction =
            TestTransaction::new_v1_from_nonce(manifest, nonce, Default::default());
        self.execute_transaction(test_transaction, execution_config)
    }
}

#[test]
fn send_does_not_accept_an_address_that_is_not_an_account() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (badge_holder_account_public_key, _, badge_holder_account) = ledger.new_account(false);

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
                            allow_recover: false,
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

    // Act
    let receipt = ledger.execute_manifest(
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
                        claimant: FAUCET.into(),
                        try_direct_send: false,
                    },
                )
            })
            .build(),
        vec![NonFungibleGlobalId::from_public_key(
            &badge_holder_account_public_key,
        )],
    );

    // Assert
    receipt.expect_specific_failure(|error| {
        matches!(
            error,
            RuntimeError::SystemError(SystemError::TypeCheckError(
                TypeCheckError::BlueprintPayloadValidationError(
                    _,
                    BlueprintPayloadIdentifier::Function(func_name, InputOrOutput::Input),
                    _
                )
            )) if func_name == ACCOUNT_LOCKER_STORE_IDENT
        )
    });
}

#[test]
fn airdrop_does_not_accept_an_address_that_is_not_an_account() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (badge_holder_account_public_key, _, badge_holder_account) = ledger.new_account(false);

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
                            allow_recover: false,
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

    // Act
    let receipt = ledger.execute_manifest(
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
                    ACCOUNT_LOCKER_AIRDROP_IDENT,
                    AccountLockerAirdropManifestInput {
                        bucket,
                        claimants: indexmap! {
                            FAUCET.into() => ResourceSpecifier::Fungible(dec!(1))
                        },
                        try_direct_send: false,
                    },
                )
            })
            .build(),
        vec![NonFungibleGlobalId::from_public_key(
            &badge_holder_account_public_key,
        )],
    );

    // Assert
    receipt.expect_specific_failure(|error| {
        matches!(
            error,
            RuntimeError::SystemError(SystemError::TypeCheckError(
                TypeCheckError::BlueprintPayloadValidationError(
                    _,
                    BlueprintPayloadIdentifier::Function(func_name, InputOrOutput::Input),
                    _
                )
            )) if func_name == ACCOUNT_LOCKER_AIRDROP_IDENT
        )
    });
}

#[test]
fn claim_does_not_accept_an_address_that_is_not_an_account() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (badge_holder_account_public_key, _, badge_holder_account) = ledger.new_account(false);

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
                            allow_recover: false,
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

    // Act
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_proof_from_account_of_amount(
                badge_holder_account,
                account_locker_badge,
                dec!(1),
            )
            .call_method(
                account_locker,
                ACCOUNT_LOCKER_CLAIM_IDENT,
                AccountLockerClaimManifestInput {
                    claimant: FAUCET.into(),
                    resource_address: XRD.into(),
                    amount: dec!(1),
                },
            )
            .build(),
        vec![NonFungibleGlobalId::from_public_key(
            &badge_holder_account_public_key,
        )],
    );

    // Assert
    receipt.expect_specific_failure(|error| {
        matches!(
            error,
            RuntimeError::SystemError(SystemError::TypeCheckError(
                TypeCheckError::BlueprintPayloadValidationError(
                    _,
                    BlueprintPayloadIdentifier::Function(func_name, InputOrOutput::Input),
                    _
                )
            )) if func_name == ACCOUNT_LOCKER_CLAIM_IDENT
        )
    });
}

#[test]
fn recover_does_not_accept_an_address_that_is_not_an_account() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (badge_holder_account_public_key, _, badge_holder_account) = ledger.new_account(false);

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
                            allow_recover: true,
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

    // Act
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_proof_from_account_of_amount(
                badge_holder_account,
                account_locker_badge,
                dec!(1),
            )
            .call_method(
                account_locker,
                ACCOUNT_LOCKER_RECOVER_IDENT,
                AccountLockerRecoverManifestInput {
                    claimant: FAUCET.into(),
                    resource_address: XRD.into(),
                    amount: dec!(1),
                },
            )
            .build(),
        vec![NonFungibleGlobalId::from_public_key(
            &badge_holder_account_public_key,
        )],
    );

    // Assert
    receipt.expect_specific_failure(|error| {
        matches!(
            error,
            RuntimeError::SystemError(SystemError::TypeCheckError(
                TypeCheckError::BlueprintPayloadValidationError(
                    _,
                    BlueprintPayloadIdentifier::Function(func_name, InputOrOutput::Input),
                    _
                )
            )) if func_name == ACCOUNT_LOCKER_RECOVER_IDENT
        )
    });
}

#[test]
fn exceeding_one_of_the_limits_when_airdropping_returns_the_expected_error() {
    for airdrops in 1u64.. {
        let mut ledger = LedgerSimulatorBuilder::new().build();
        let (pk, _, account) = ledger.new_account(false);

        let keys_and_accounts = (1..=airdrops)
            .map(|num| Secp256k1PrivateKey::from_u64(num).unwrap())
            .map(|private_key| {
                let address = ComponentAddress::preallocated_account_from_public_key(
                    &private_key.public_key(),
                );
                (private_key, address)
            })
            .collect::<Vec<_>>();

        let manifest = keys_and_accounts
            .iter()
            .map(|(_, account)| account)
            .copied()
            .fold(
                ManifestBuilder::new().lock_fee_from_faucet(),
                |builder, account| {
                    builder.call_method(
                        account,
                        ACCOUNT_SET_DEFAULT_DEPOSIT_RULE_IDENT,
                        AccountSetDefaultDepositRuleInput {
                            default: DefaultDepositRule::Reject,
                        },
                    )
                },
            )
            .build();
        ledger
            .execute_manifest_without_auth(manifest)
            .expect_commit_success();

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
                                allow_recover: false,
                            },
                        )
                        .try_deposit_entire_worktop_or_abort(account, None)
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

        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .get_free_xrd_from_faucet()
            .create_proof_from_account_of_amount(account, account_locker_badge, 1)
            .take_all_from_worktop(XRD, "bucket")
            .with_bucket("bucket", |builder, bucket| {
                builder.call_method(
                    account_locker,
                    ACCOUNT_LOCKER_AIRDROP_IDENT,
                    AccountLockerAirdropManifestInput {
                        claimants: keys_and_accounts
                            .iter()
                            .map(|entry| (entry.1.into(), ResourceSpecifier::Fungible(dec!(1))))
                            .collect(),
                        bucket,
                        try_direct_send: true,
                    },
                )
            })
            .try_deposit_entire_worktop_or_abort(account, None)
            .build();
        let receipt =
            ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&pk)]);
        if receipt.is_commit_failure() {
            receipt.expect_specific_failure(|error| {
                matches!(
                    error,
                    RuntimeError::SystemModuleError(SystemModuleError::TransactionLimitsError(
                        TransactionLimitsError::TooManyEvents
                    ))
                )
            });
            break;
        }
    }
}
