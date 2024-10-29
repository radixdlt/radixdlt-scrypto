use radix_common::prelude::*;
use radix_engine::blueprints::access_controller::v2::*;
use radix_engine::blueprints::access_controller::*;
use radix_engine::system::system_db_reader::*;
use radix_engine::updates::*;
use radix_substate_store_queries::typed_native_events::*;
use radix_substate_store_queries::typed_substate_layout::*;
use scrypto_test::prelude::*;

/// The state of the access controller changes with the bottlenose protocol update where we're
/// adding a new XRD vault to the state. This test ensures that we don't have any regression from
/// the refactoring and that the package definition for the v1.0 access controller package remains
/// the same.
#[test]
fn access_controller_package_definition_v1_0_matches_expected() {
    // Arrange
    let expected_package_definition = manifest_decode::<PackageDefinition>(include_bytes!(
        "../../assets/access_controller_v1_package_definition.rpd"
    ))
    .unwrap();

    // Act
    let package_definition = v1::AccessControllerV1NativePackage::definition();

    // Assert
    assert_eq!(package_definition, expected_package_definition);
}

#[test]
fn access_controller_instantiated_before_protocol_update_has_v1_state() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new()
        .with_custom_protocol(|builder| builder.only_babylon())
        .build();

    let access_controller = ledger
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .get_free_xrd_from_faucet()
                .take_all_from_worktop(XRD, "xrd")
                .create_access_controller(
                    "xrd",
                    rule!(allow_all),
                    rule!(allow_all),
                    rule!(allow_all),
                    None,
                )
                .build(),
            vec![],
        )
        .expect_commit_success()
        .new_component_addresses()
        .first()
        .copied()
        .unwrap();

    // Act
    let state = read_access_controller_state(ledger.substate_db(), access_controller)
        .into_content()
        .into_versions();

    // Assert
    assert_matches!(state, AccessControllerV2StateVersions::V1(..))
}

#[test]
fn access_controller_instantiated_after_protocol_update_has_v2_state() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();

    let access_controller = ledger
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .get_free_xrd_from_faucet()
                .take_all_from_worktop(XRD, "xrd")
                .create_access_controller(
                    "xrd",
                    rule!(allow_all),
                    rule!(allow_all),
                    rule!(allow_all),
                    None,
                )
                .build(),
            vec![],
        )
        .expect_commit_success()
        .new_component_addresses()
        .first()
        .copied()
        .unwrap();

    // Act
    let state = read_access_controller_state(ledger.substate_db(), access_controller)
        .into_content()
        .into_versions();

    // Assert
    assert_matches!(state, AccessControllerV2StateVersions::V2(..))
}

#[test]
fn before_protocol_update_calling_any_method_on_an_access_controller_with_v1_state_doesnt_update_state(
) {
    // Arrange
    let rule_set = RuleSet {
        primary_role: rule!(allow_all),
        recovery_role: rule!(allow_all),
        confirmation_role: rule!(allow_all),
    };
    let invocations = [
        (
            ACCESS_CONTROLLER_CREATE_PROOF_IDENT,
            to_manifest_value_and_unwrap!(&AccessControllerCreateProofInput {}),
        ),
        (
            ACCESS_CONTROLLER_INITIATE_RECOVERY_AS_PRIMARY_IDENT,
            to_manifest_value_and_unwrap!(&AccessControllerInitiateRecoveryAsPrimaryInput {
                rule_set: rule_set.clone(),
                timed_recovery_delay_in_minutes: Some(0)
            }),
        ),
        (
            ACCESS_CONTROLLER_INITIATE_RECOVERY_AS_RECOVERY_IDENT,
            to_manifest_value_and_unwrap!(&AccessControllerInitiateRecoveryAsRecoveryInput {
                rule_set: rule_set.clone(),
                timed_recovery_delay_in_minutes: Some(0)
            }),
        ),
        (
            ACCESS_CONTROLLER_INITIATE_BADGE_WITHDRAW_ATTEMPT_AS_PRIMARY_IDENT,
            to_manifest_value_and_unwrap!(
                &AccessControllerInitiateBadgeWithdrawAttemptAsPrimaryInput {}
            ),
        ),
        (
            ACCESS_CONTROLLER_INITIATE_BADGE_WITHDRAW_ATTEMPT_AS_RECOVERY_IDENT,
            to_manifest_value_and_unwrap!(
                &AccessControllerInitiateBadgeWithdrawAttemptAsRecoveryInput {}
            ),
        ),
        (
            ACCESS_CONTROLLER_QUICK_CONFIRM_PRIMARY_ROLE_RECOVERY_PROPOSAL_IDENT,
            to_manifest_value_and_unwrap!(
                &AccessControllerQuickConfirmPrimaryRoleRecoveryProposalInput {
                    rule_set: rule_set.clone(),
                    timed_recovery_delay_in_minutes: Some(0)
                }
            ),
        ),
        (
            ACCESS_CONTROLLER_QUICK_CONFIRM_RECOVERY_ROLE_RECOVERY_PROPOSAL_IDENT,
            to_manifest_value_and_unwrap!(
                &AccessControllerQuickConfirmRecoveryRoleRecoveryProposalInput {
                    rule_set: rule_set.clone(),
                    timed_recovery_delay_in_minutes: Some(0)
                }
            ),
        ),
        (
            ACCESS_CONTROLLER_QUICK_CONFIRM_PRIMARY_ROLE_BADGE_WITHDRAW_ATTEMPT_IDENT,
            to_manifest_value_and_unwrap!(
                &AccessControllerQuickConfirmPrimaryRoleBadgeWithdrawAttemptInput {}
            ),
        ),
        (
            ACCESS_CONTROLLER_QUICK_CONFIRM_RECOVERY_ROLE_BADGE_WITHDRAW_ATTEMPT_IDENT,
            to_manifest_value_and_unwrap!(
                &AccessControllerQuickConfirmRecoveryRoleBadgeWithdrawAttemptInput {}
            ),
        ),
        (
            ACCESS_CONTROLLER_TIMED_CONFIRM_RECOVERY_IDENT,
            to_manifest_value_and_unwrap!(&AccessControllerTimedConfirmRecoveryInput {
                rule_set: rule_set.clone(),
                timed_recovery_delay_in_minutes: Some(0)
            }),
        ),
        (
            ACCESS_CONTROLLER_CANCEL_PRIMARY_ROLE_RECOVERY_PROPOSAL_IDENT,
            to_manifest_value_and_unwrap!(
                &AccessControllerCancelPrimaryRoleRecoveryProposalInput {}
            ),
        ),
        (
            ACCESS_CONTROLLER_CANCEL_RECOVERY_ROLE_RECOVERY_PROPOSAL_IDENT,
            to_manifest_value_and_unwrap!(
                &AccessControllerCancelRecoveryRoleRecoveryProposalInput {}
            ),
        ),
        (
            ACCESS_CONTROLLER_CANCEL_PRIMARY_ROLE_BADGE_WITHDRAW_ATTEMPT_IDENT,
            to_manifest_value_and_unwrap!(
                &AccessControllerCancelPrimaryRoleBadgeWithdrawAttemptInput {}
            ),
        ),
        (
            ACCESS_CONTROLLER_CANCEL_RECOVERY_ROLE_BADGE_WITHDRAW_ATTEMPT_IDENT,
            to_manifest_value_and_unwrap!(
                &AccessControllerCancelRecoveryRoleBadgeWithdrawAttemptInput {}
            ),
        ),
        (
            ACCESS_CONTROLLER_LOCK_PRIMARY_ROLE_IDENT,
            to_manifest_value_and_unwrap!(&AccessControllerLockPrimaryRoleInput {}),
        ),
        (
            ACCESS_CONTROLLER_UNLOCK_PRIMARY_ROLE_IDENT,
            to_manifest_value_and_unwrap!(&AccessControllerUnlockPrimaryRoleInput {}),
        ),
        (
            ACCESS_CONTROLLER_STOP_TIMED_RECOVERY_IDENT,
            to_manifest_value_and_unwrap!(&AccessControllerStopTimedRecoveryInput {
                rule_set: rule_set.clone(),
                timed_recovery_delay_in_minutes: Some(0)
            }),
        ),
        (
            ACCESS_CONTROLLER_MINT_RECOVERY_BADGES_IDENT,
            to_manifest_value_and_unwrap!(&AccessControllerMintRecoveryBadgesInput {
                non_fungible_local_ids: Default::default()
            }),
        ),
    ];

    for (method_name, args) in invocations {
        let mut ledger = LedgerSimulatorBuilder::new()
            .with_custom_protocol(|builder| builder.only_babylon())
            .build();
        let (_, _, account) = ledger.new_account(false);

        let access_controller = ledger
            .execute_manifest(
                ManifestBuilder::new()
                    .lock_fee_from_faucet()
                    .get_free_xrd_from_faucet()
                    .take_all_from_worktop(XRD, "xrd")
                    .create_access_controller(
                        "xrd",
                        rule!(allow_all),
                        rule!(allow_all),
                        rule!(allow_all),
                        Some(0),
                    )
                    .build(),
                vec![],
            )
            .expect_commit_success()
            .new_component_addresses()
            .first()
            .copied()
            .unwrap();

        let setup_manifest = match method_name {
            ACCESS_CONTROLLER_QUICK_CONFIRM_PRIMARY_ROLE_RECOVERY_PROPOSAL_IDENT
            | ACCESS_CONTROLLER_CANCEL_PRIMARY_ROLE_RECOVERY_PROPOSAL_IDENT => Some(
                ManifestBuilder::new()
                    .lock_fee_from_faucet()
                    .call_method(
                        access_controller,
                        ACCESS_CONTROLLER_INITIATE_RECOVERY_AS_PRIMARY_IDENT,
                        AccessControllerInitiateRecoveryAsPrimaryInput {
                            rule_set: rule_set.clone(),
                            timed_recovery_delay_in_minutes: Some(0),
                        },
                    )
                    .build(),
            ),
            ACCESS_CONTROLLER_QUICK_CONFIRM_RECOVERY_ROLE_RECOVERY_PROPOSAL_IDENT
            | ACCESS_CONTROLLER_CANCEL_RECOVERY_ROLE_RECOVERY_PROPOSAL_IDENT
            | ACCESS_CONTROLLER_TIMED_CONFIRM_RECOVERY_IDENT
            | ACCESS_CONTROLLER_STOP_TIMED_RECOVERY_IDENT => Some(
                ManifestBuilder::new()
                    .lock_fee_from_faucet()
                    .call_method(
                        access_controller,
                        ACCESS_CONTROLLER_INITIATE_RECOVERY_AS_RECOVERY_IDENT,
                        AccessControllerInitiateRecoveryAsRecoveryInput {
                            rule_set: rule_set.clone(),
                            timed_recovery_delay_in_minutes: Some(0),
                        },
                    )
                    .build(),
            ),
            ACCESS_CONTROLLER_QUICK_CONFIRM_PRIMARY_ROLE_BADGE_WITHDRAW_ATTEMPT_IDENT
            | ACCESS_CONTROLLER_CANCEL_PRIMARY_ROLE_BADGE_WITHDRAW_ATTEMPT_IDENT => Some(
                ManifestBuilder::new()
                    .lock_fee_from_faucet()
                    .call_method(
                        access_controller,
                        ACCESS_CONTROLLER_INITIATE_BADGE_WITHDRAW_ATTEMPT_AS_PRIMARY_IDENT,
                        AccessControllerInitiateBadgeWithdrawAttemptAsPrimaryInput {},
                    )
                    .build(),
            ),
            ACCESS_CONTROLLER_QUICK_CONFIRM_RECOVERY_ROLE_BADGE_WITHDRAW_ATTEMPT_IDENT
            | ACCESS_CONTROLLER_CANCEL_RECOVERY_ROLE_BADGE_WITHDRAW_ATTEMPT_IDENT => Some(
                ManifestBuilder::new()
                    .lock_fee_from_faucet()
                    .call_method(
                        access_controller,
                        ACCESS_CONTROLLER_INITIATE_BADGE_WITHDRAW_ATTEMPT_AS_RECOVERY_IDENT,
                        AccessControllerInitiateBadgeWithdrawAttemptAsRecoveryInput {},
                    )
                    .build(),
            ),
            _ => None,
        };
        if let Some(setup_manifest) = setup_manifest {
            ledger
                .execute_manifest(setup_manifest, vec![])
                .expect_commit_success();
        }

        // Act
        let manifest = {
            let mut manifest_builder = ManifestBuilder::new().lock_fee_from_faucet();

            if method_name == ACCESS_CONTROLLER_CONTRIBUTE_RECOVERY_FEE_IDENT {
                manifest_builder = manifest_builder
                    .get_free_xrd_from_faucet()
                    .take_all_from_worktop(XRD, "xrd")
            }

            manifest_builder
                .call_method(
                    access_controller,
                    method_name,
                    ManifestArgs::new_from_tuple_or_panic(args),
                )
                .try_deposit_entire_worktop_or_abort(account, None)
                .build()
        };
        ledger
            .execute_manifest(manifest, vec![])
            .expect_commit_success();

        // Assert
        let state = read_access_controller_state(ledger.substate_db(), access_controller);
        assert!(
            matches!(state, AccessControllerV2StateVersions::V1(..)),
            "Invocation {method_name} failed"
        );
    }
}

#[test]
fn after_protocol_update_calling_any_method_on_an_access_controller_with_v1_state_updates_its_state_from_v1_to_v2(
) {
    // Arrange
    let rule_set = RuleSet {
        primary_role: rule!(allow_all),
        recovery_role: rule!(allow_all),
        confirmation_role: rule!(allow_all),
    };
    let invocations = [
        (
            ACCESS_CONTROLLER_CREATE_PROOF_IDENT,
            to_manifest_value_and_unwrap!(&AccessControllerCreateProofInput {}),
        ),
        (
            ACCESS_CONTROLLER_INITIATE_RECOVERY_AS_PRIMARY_IDENT,
            to_manifest_value_and_unwrap!(&AccessControllerInitiateRecoveryAsPrimaryInput {
                rule_set: rule_set.clone(),
                timed_recovery_delay_in_minutes: Some(0)
            }),
        ),
        (
            ACCESS_CONTROLLER_INITIATE_RECOVERY_AS_RECOVERY_IDENT,
            to_manifest_value_and_unwrap!(&AccessControllerInitiateRecoveryAsRecoveryInput {
                rule_set: rule_set.clone(),
                timed_recovery_delay_in_minutes: Some(0)
            }),
        ),
        (
            ACCESS_CONTROLLER_INITIATE_BADGE_WITHDRAW_ATTEMPT_AS_PRIMARY_IDENT,
            to_manifest_value_and_unwrap!(
                &AccessControllerInitiateBadgeWithdrawAttemptAsPrimaryInput {}
            ),
        ),
        (
            ACCESS_CONTROLLER_INITIATE_BADGE_WITHDRAW_ATTEMPT_AS_RECOVERY_IDENT,
            to_manifest_value_and_unwrap!(
                &AccessControllerInitiateBadgeWithdrawAttemptAsRecoveryInput {}
            ),
        ),
        (
            ACCESS_CONTROLLER_QUICK_CONFIRM_PRIMARY_ROLE_RECOVERY_PROPOSAL_IDENT,
            to_manifest_value_and_unwrap!(
                &AccessControllerQuickConfirmPrimaryRoleRecoveryProposalInput {
                    rule_set: rule_set.clone(),
                    timed_recovery_delay_in_minutes: Some(0)
                }
            ),
        ),
        (
            ACCESS_CONTROLLER_QUICK_CONFIRM_RECOVERY_ROLE_RECOVERY_PROPOSAL_IDENT,
            to_manifest_value_and_unwrap!(
                &AccessControllerQuickConfirmRecoveryRoleRecoveryProposalInput {
                    rule_set: rule_set.clone(),
                    timed_recovery_delay_in_minutes: Some(0)
                }
            ),
        ),
        (
            ACCESS_CONTROLLER_QUICK_CONFIRM_PRIMARY_ROLE_BADGE_WITHDRAW_ATTEMPT_IDENT,
            to_manifest_value_and_unwrap!(
                &AccessControllerQuickConfirmPrimaryRoleBadgeWithdrawAttemptInput {}
            ),
        ),
        (
            ACCESS_CONTROLLER_QUICK_CONFIRM_RECOVERY_ROLE_BADGE_WITHDRAW_ATTEMPT_IDENT,
            to_manifest_value_and_unwrap!(
                &AccessControllerQuickConfirmRecoveryRoleBadgeWithdrawAttemptInput {}
            ),
        ),
        (
            ACCESS_CONTROLLER_TIMED_CONFIRM_RECOVERY_IDENT,
            to_manifest_value_and_unwrap!(&AccessControllerTimedConfirmRecoveryInput {
                rule_set: rule_set.clone(),
                timed_recovery_delay_in_minutes: Some(0)
            }),
        ),
        (
            ACCESS_CONTROLLER_CANCEL_PRIMARY_ROLE_RECOVERY_PROPOSAL_IDENT,
            to_manifest_value_and_unwrap!(
                &AccessControllerCancelPrimaryRoleRecoveryProposalInput {}
            ),
        ),
        (
            ACCESS_CONTROLLER_CANCEL_RECOVERY_ROLE_RECOVERY_PROPOSAL_IDENT,
            to_manifest_value_and_unwrap!(
                &AccessControllerCancelRecoveryRoleRecoveryProposalInput {}
            ),
        ),
        (
            ACCESS_CONTROLLER_CANCEL_PRIMARY_ROLE_BADGE_WITHDRAW_ATTEMPT_IDENT,
            to_manifest_value_and_unwrap!(
                &AccessControllerCancelPrimaryRoleBadgeWithdrawAttemptInput {}
            ),
        ),
        (
            ACCESS_CONTROLLER_CANCEL_RECOVERY_ROLE_BADGE_WITHDRAW_ATTEMPT_IDENT,
            to_manifest_value_and_unwrap!(
                &AccessControllerCancelRecoveryRoleBadgeWithdrawAttemptInput {}
            ),
        ),
        (
            ACCESS_CONTROLLER_LOCK_PRIMARY_ROLE_IDENT,
            to_manifest_value_and_unwrap!(&AccessControllerLockPrimaryRoleInput {}),
        ),
        (
            ACCESS_CONTROLLER_UNLOCK_PRIMARY_ROLE_IDENT,
            to_manifest_value_and_unwrap!(&AccessControllerUnlockPrimaryRoleInput {}),
        ),
        (
            ACCESS_CONTROLLER_STOP_TIMED_RECOVERY_IDENT,
            to_manifest_value_and_unwrap!(&AccessControllerStopTimedRecoveryInput {
                rule_set: rule_set.clone(),
                timed_recovery_delay_in_minutes: Some(0)
            }),
        ),
        (
            ACCESS_CONTROLLER_MINT_RECOVERY_BADGES_IDENT,
            to_manifest_value_and_unwrap!(&AccessControllerMintRecoveryBadgesInput {
                non_fungible_local_ids: Default::default()
            }),
        ),
        (
            ACCESS_CONTROLLER_CONTRIBUTE_RECOVERY_FEE_IDENT,
            to_manifest_value_and_unwrap!(&AccessControllerContributeRecoveryFeeManifestInput {
                bucket: ManifestBucket(0)
            }),
        ),
        /*
        Commented out intentionally since they're not very easy to test. To call these we must
        first call contribute which will in on itself update the state. These methods are capable
        of updating the state on their own but I can't see any scenario where they ever will since
        they require that the vault be there to succeed.
        */
        // (
        //     ACCESS_CONTROLLER_LOCK_RECOVERY_FEE_IDENT,
        //     to_manifest_value_and_unwrap!(&AccessControllerLockRecoveryFeeInput {
        //         amount: dec!(0)
        //     }),
        // ),
        // (
        //     ACCESS_CONTROLLER_WITHDRAW_RECOVERY_FEE_IDENT,
        //     to_manifest_value_and_unwrap!(&AccessControllerWithdrawRecoveryFeeInput {
        //         amount: dec!(0)
        //     }),
        // ),
    ];

    for (method_name, args) in invocations {
        let mut ledger = LedgerSimulatorBuilder::new()
            .with_custom_protocol(|builder| builder.only_babylon())
            .build();
        let (_, _, account) = ledger.new_account(false);

        let access_controller = ledger
            .execute_manifest(
                ManifestBuilder::new()
                    .lock_fee_from_faucet()
                    .get_free_xrd_from_faucet()
                    .take_all_from_worktop(XRD, "xrd")
                    .create_access_controller(
                        "xrd",
                        rule!(allow_all),
                        rule!(allow_all),
                        rule!(allow_all),
                        Some(0),
                    )
                    .build(),
                vec![],
            )
            .expect_commit_success()
            .new_component_addresses()
            .first()
            .copied()
            .unwrap();

        let setup_manifest = match method_name {
            ACCESS_CONTROLLER_QUICK_CONFIRM_PRIMARY_ROLE_RECOVERY_PROPOSAL_IDENT
            | ACCESS_CONTROLLER_CANCEL_PRIMARY_ROLE_RECOVERY_PROPOSAL_IDENT => Some(
                ManifestBuilder::new()
                    .lock_fee_from_faucet()
                    .call_method(
                        access_controller,
                        ACCESS_CONTROLLER_INITIATE_RECOVERY_AS_PRIMARY_IDENT,
                        AccessControllerInitiateRecoveryAsPrimaryInput {
                            rule_set: rule_set.clone(),
                            timed_recovery_delay_in_minutes: Some(0),
                        },
                    )
                    .build(),
            ),
            ACCESS_CONTROLLER_QUICK_CONFIRM_RECOVERY_ROLE_RECOVERY_PROPOSAL_IDENT
            | ACCESS_CONTROLLER_CANCEL_RECOVERY_ROLE_RECOVERY_PROPOSAL_IDENT
            | ACCESS_CONTROLLER_TIMED_CONFIRM_RECOVERY_IDENT
            | ACCESS_CONTROLLER_STOP_TIMED_RECOVERY_IDENT => Some(
                ManifestBuilder::new()
                    .lock_fee_from_faucet()
                    .call_method(
                        access_controller,
                        ACCESS_CONTROLLER_INITIATE_RECOVERY_AS_RECOVERY_IDENT,
                        AccessControllerInitiateRecoveryAsRecoveryInput {
                            rule_set: rule_set.clone(),
                            timed_recovery_delay_in_minutes: Some(0),
                        },
                    )
                    .build(),
            ),
            ACCESS_CONTROLLER_QUICK_CONFIRM_PRIMARY_ROLE_BADGE_WITHDRAW_ATTEMPT_IDENT
            | ACCESS_CONTROLLER_CANCEL_PRIMARY_ROLE_BADGE_WITHDRAW_ATTEMPT_IDENT => Some(
                ManifestBuilder::new()
                    .lock_fee_from_faucet()
                    .call_method(
                        access_controller,
                        ACCESS_CONTROLLER_INITIATE_BADGE_WITHDRAW_ATTEMPT_AS_PRIMARY_IDENT,
                        AccessControllerInitiateBadgeWithdrawAttemptAsPrimaryInput {},
                    )
                    .build(),
            ),
            ACCESS_CONTROLLER_QUICK_CONFIRM_RECOVERY_ROLE_BADGE_WITHDRAW_ATTEMPT_IDENT
            | ACCESS_CONTROLLER_CANCEL_RECOVERY_ROLE_BADGE_WITHDRAW_ATTEMPT_IDENT => Some(
                ManifestBuilder::new()
                    .lock_fee_from_faucet()
                    .call_method(
                        access_controller,
                        ACCESS_CONTROLLER_INITIATE_BADGE_WITHDRAW_ATTEMPT_AS_RECOVERY_IDENT,
                        AccessControllerInitiateBadgeWithdrawAttemptAsRecoveryInput {},
                    )
                    .build(),
            ),
            _ => None,
        };
        if let Some(setup_manifest) = setup_manifest {
            ledger
                .execute_manifest(setup_manifest, vec![])
                .expect_commit_success();
        }

        ProtocolBuilder::for_simulator()
            .from_to(ProtocolVersion::Babylon, ProtocolVersion::Bottlenose)
            .commit_each_protocol_update(ledger.substate_db_mut());

        // Act
        let manifest = {
            let mut manifest_builder = ManifestBuilder::new().lock_fee_from_faucet();

            if method_name == ACCESS_CONTROLLER_CONTRIBUTE_RECOVERY_FEE_IDENT {
                manifest_builder = manifest_builder
                    .get_free_xrd_from_faucet()
                    .take_all_from_worktop(XRD, "xrd")
            }

            manifest_builder
                .call_method(
                    access_controller,
                    method_name,
                    ManifestArgs::new_from_tuple_or_panic(args),
                )
                .try_deposit_entire_worktop_or_abort(account, None)
                .build()
        };
        ledger
            .execute_manifest(manifest, vec![])
            .expect_commit_success();

        // Assert
        let state = read_access_controller_state(ledger.substate_db(), access_controller);
        assert!(
            matches!(state, AccessControllerV2StateVersions::V2(..)),
            "Invocation {method_name} failed"
        );
    }
}

#[test]
fn lock_recovery_fee_is_only_callable_by_primary_recovery_or_confirmation() {
    // Arrange
    let expectations = [
        (Role::None, false),
        (Role::Primary, true),
        (Role::Recovery, true),
        (Role::Confirmation, true),
    ];
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (pk, _, account) = ledger.new_account(false);

    let primary_badge_resource = ledger.create_fungible_resource(dec!(1), 0, account);
    let recovery_badge_resource = ledger.create_fungible_resource(dec!(1), 0, account);
    let confirmation_badge_resource = ledger.create_fungible_resource(dec!(1), 0, account);

    let access_controller = ledger
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .get_free_xrd_from_faucet()
                .take_all_from_worktop(XRD, "xrd")
                .create_access_controller(
                    "xrd",
                    rule!(require(primary_badge_resource)),
                    rule!(require(recovery_badge_resource)),
                    rule!(require(confirmation_badge_resource)),
                    None,
                )
                .build(),
            vec![],
        )
        .expect_commit_success()
        .new_component_addresses()
        .first()
        .copied()
        .unwrap();

    ledger
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .get_free_xrd_from_faucet()
                .take_all_from_worktop(XRD, "xrd")
                .with_bucket("xrd", |builder, bucket| {
                    builder.call_method(
                        access_controller,
                        ACCESS_CONTROLLER_CONTRIBUTE_RECOVERY_FEE_IDENT,
                        manifest_args!(bucket),
                    )
                })
                .build(),
            vec![],
        )
        .expect_commit_success();

    for (role, should_succeed) in expectations {
        // Act
        let manifest = {
            let mut manifest_builder = ManifestBuilder::new().lock_fee_from_faucet();
            manifest_builder = match role {
                Role::None => manifest_builder,
                Role::Primary => manifest_builder.create_proof_from_account_of_amount(
                    account,
                    primary_badge_resource,
                    1,
                ),
                Role::Recovery => manifest_builder.create_proof_from_account_of_amount(
                    account,
                    recovery_badge_resource,
                    1,
                ),
                Role::Confirmation => manifest_builder.create_proof_from_account_of_amount(
                    account,
                    confirmation_badge_resource,
                    1,
                ),
            };
            manifest_builder
                .call_method(
                    access_controller,
                    ACCESS_CONTROLLER_LOCK_RECOVERY_FEE_IDENT,
                    AccessControllerLockRecoveryFeeInput { amount: dec!(100) },
                )
                .build()
        };
        let receipt =
            ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&pk)]);

        // Assert
        if should_succeed {
            receipt.expect_commit_success();
        } else {
            receipt.expect_specific_failure(|error| {
                matches!(
                    error,
                    RuntimeError::SystemModuleError(SystemModuleError::AuthError(
                        AuthError::Unauthorized(unauthorized)
                    )) if unauthorized.fn_identifier.ident == ACCESS_CONTROLLER_LOCK_RECOVERY_FEE_IDENT
                )
            })
        };
    }
}

#[test]
fn withdraw_recovery_fee_is_only_callable_by_primary() {
    // Arrange
    let expectations = [
        (Role::None, false),
        (Role::Primary, true),
        (Role::Recovery, false),
        (Role::Confirmation, false),
    ];
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (pk, _, account) = ledger.new_account(false);

    let primary_badge_resource = ledger.create_fungible_resource(dec!(1), 0, account);
    let recovery_badge_resource = ledger.create_fungible_resource(dec!(1), 0, account);
    let confirmation_badge_resource = ledger.create_fungible_resource(dec!(1), 0, account);

    let access_controller = ledger
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .get_free_xrd_from_faucet()
                .take_all_from_worktop(XRD, "xrd")
                .create_access_controller(
                    "xrd",
                    rule!(require(primary_badge_resource)),
                    rule!(require(recovery_badge_resource)),
                    rule!(require(confirmation_badge_resource)),
                    None,
                )
                .build(),
            vec![],
        )
        .expect_commit_success()
        .new_component_addresses()
        .first()
        .copied()
        .unwrap();

    ledger
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .get_free_xrd_from_faucet()
                .take_all_from_worktop(XRD, "xrd")
                .with_bucket("xrd", |builder, bucket| {
                    builder.call_method(
                        access_controller,
                        ACCESS_CONTROLLER_CONTRIBUTE_RECOVERY_FEE_IDENT,
                        manifest_args!(bucket),
                    )
                })
                .build(),
            vec![],
        )
        .expect_commit_success();

    for (role, should_succeed) in expectations {
        // Act
        let manifest = {
            let mut manifest_builder = ManifestBuilder::new().lock_fee_from_faucet();
            manifest_builder = match role {
                Role::None => manifest_builder,
                Role::Primary => manifest_builder.create_proof_from_account_of_amount(
                    account,
                    primary_badge_resource,
                    1,
                ),
                Role::Recovery => manifest_builder.create_proof_from_account_of_amount(
                    account,
                    recovery_badge_resource,
                    1,
                ),
                Role::Confirmation => manifest_builder.create_proof_from_account_of_amount(
                    account,
                    confirmation_badge_resource,
                    1,
                ),
            };
            manifest_builder
                .call_method(
                    access_controller,
                    ACCESS_CONTROLLER_WITHDRAW_RECOVERY_FEE_IDENT,
                    AccessControllerWithdrawRecoveryFeeInput { amount: dec!(1) },
                )
                .try_deposit_entire_worktop_or_abort(account, None)
                .build()
        };
        let receipt =
            ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&pk)]);

        // Assert
        if should_succeed {
            receipt.expect_commit_success();
        } else {
            receipt.expect_specific_failure(|error| {
                matches!(
                    error,
                    RuntimeError::SystemModuleError(SystemModuleError::AuthError(
                        AuthError::Unauthorized(unauthorized)
                    )) if unauthorized.fn_identifier.ident == ACCESS_CONTROLLER_WITHDRAW_RECOVERY_FEE_IDENT
                )
            })
        };
    }
}

#[test]
fn contribute_recovery_fee_is_callable_without_auth() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();

    let access_controller = ledger
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .get_free_xrd_from_faucet()
                .take_all_from_worktop(XRD, "xrd")
                .create_access_controller(
                    "xrd",
                    rule!(allow_all),
                    rule!(allow_all),
                    rule!(allow_all),
                    None,
                )
                .build(),
            vec![],
        )
        .expect_commit_success()
        .new_component_addresses()
        .first()
        .copied()
        .unwrap();

    // Act
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .get_free_xrd_from_faucet()
            .take_all_from_worktop(XRD, "xrd")
            .with_bucket("xrd", |builder, bucket| {
                builder.call_method(
                    access_controller,
                    ACCESS_CONTROLLER_CONTRIBUTE_RECOVERY_FEE_IDENT,
                    manifest_args!(bucket),
                )
            })
            .build(),
        vec![],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn deposit_event_is_emitted_when_recovery_xrd_is_contributed() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();

    let access_controller = ledger
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .get_free_xrd_from_faucet()
                .take_all_from_worktop(XRD, "xrd")
                .create_access_controller(
                    "xrd",
                    rule!(allow_all),
                    rule!(allow_all),
                    rule!(allow_all),
                    None,
                )
                .build(),
            vec![],
        )
        .expect_commit_success()
        .new_component_addresses()
        .first()
        .copied()
        .unwrap();

    // Act
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .get_free_xrd_from_faucet()
            .take_all_from_worktop(XRD, "xrd")
            .with_bucket("xrd", |builder, bucket| {
                builder.call_method(
                    access_controller,
                    ACCESS_CONTROLLER_CONTRIBUTE_RECOVERY_FEE_IDENT,
                    manifest_args!(bucket),
                )
            })
            .build(),
        vec![],
    );

    // Assert
    let commit_result = receipt.expect_commit_success();
    let deposit_events = commit_result
        .application_events
        .iter()
        .map(|(identifier, data)| to_typed_native_event(identifier, data).unwrap())
        .filter_map(|typed_event| {
            if let TypedNativeEvent::AccessController(
                TypedAccessControllerPackageEvent::AccessController(
                    TypedAccessControllerBlueprintEvent::DepositRecoveryXrdEvent(event),
                ),
            ) = typed_event
            {
                Some(event)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let [event] = deposit_events.as_slice() else {
        panic!("More than one deposit event!")
    };

    assert_eq!(
        event,
        &DepositRecoveryXrdEvent {
            amount: dec!(10_000)
        }
    )
}

#[test]
fn withdraw_event_is_emitted_when_recovery_xrd_is_withdrawn() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (_, _, account) = ledger.new_account(false);

    let access_controller = ledger
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .get_free_xrd_from_faucet()
                .take_all_from_worktop(XRD, "xrd")
                .create_access_controller(
                    "xrd",
                    rule!(allow_all),
                    rule!(allow_all),
                    rule!(allow_all),
                    None,
                )
                .build(),
            vec![],
        )
        .expect_commit_success()
        .new_component_addresses()
        .first()
        .copied()
        .unwrap();

    ledger
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .get_free_xrd_from_faucet()
                .take_all_from_worktop(XRD, "xrd")
                .with_bucket("xrd", |builder, bucket| {
                    builder.call_method(
                        access_controller,
                        ACCESS_CONTROLLER_CONTRIBUTE_RECOVERY_FEE_IDENT,
                        manifest_args!(bucket),
                    )
                })
                .build(),
            vec![],
        )
        .expect_commit_success();

    // Act
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_method(
                access_controller,
                ACCESS_CONTROLLER_WITHDRAW_RECOVERY_FEE_IDENT,
                AccessControllerWithdrawRecoveryFeeInput { amount: dec!(1) },
            )
            .try_deposit_entire_worktop_or_abort(account, None)
            .build(),
        vec![],
    );

    // Assert
    let commit_result = receipt.expect_commit_success();
    let withdraw_events = commit_result
        .application_events
        .iter()
        .map(|(identifier, data)| to_typed_native_event(identifier, data).unwrap())
        .filter_map(|typed_event| {
            if let TypedNativeEvent::AccessController(
                TypedAccessControllerPackageEvent::AccessController(
                    TypedAccessControllerBlueprintEvent::WithdrawRecoveryXrdEvent(event),
                ),
            ) = typed_event
            {
                Some(event)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let [event] = withdraw_events.as_slice() else {
        panic!("More than one deposit event!")
    };

    assert_eq!(event, &WithdrawRecoveryXrdEvent { amount: dec!(1) })
}

#[test]
fn fees_can_be_locked_from_an_access_controller_with_a_badge_primary_role() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (pk, _, account) = ledger.new_account(false);

    let primary_badge_resource = ledger.create_fungible_resource(dec!(1), 0, account);

    let access_controller = ledger
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .get_free_xrd_from_faucet()
                .take_all_from_worktop(XRD, "xrd")
                .create_access_controller(
                    "xrd",
                    rule!(require(primary_badge_resource)),
                    rule!(allow_all),
                    rule!(allow_all),
                    None,
                )
                .build(),
            vec![],
        )
        .expect_commit_success()
        .new_component_addresses()
        .first()
        .copied()
        .unwrap();

    ledger
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .get_free_xrd_from_faucet()
                .take_all_from_worktop(XRD, "xrd")
                .with_bucket("xrd", |builder, bucket| {
                    builder.call_method(
                        access_controller,
                        ACCESS_CONTROLLER_CONTRIBUTE_RECOVERY_FEE_IDENT,
                        manifest_args!(bucket),
                    )
                })
                .build(),
            vec![],
        )
        .expect_commit_success();

    // Act
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .create_proof_from_account_of_amount(account, primary_badge_resource, 1)
            .call_method(
                access_controller,
                ACCESS_CONTROLLER_LOCK_RECOVERY_FEE_IDENT,
                AccessControllerLockRecoveryFeeInput { amount: dec!(100) },
            )
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&pk)],
    );

    // Assert
    receipt.expect_commit_success();
}

#[derive(Clone, Debug)]
pub enum Role {
    None,
    Primary,
    Recovery,
    Confirmation,
}

fn read_access_controller_state<S>(
    db: &S,
    component_address: ComponentAddress,
) -> AccessControllerV2StateVersions
where
    S: SubstateDatabase,
{
    SystemDatabaseReader::new(db)
        .read_object_field(
            component_address.as_node_id(),
            ModuleId::Main,
            AccessControllerV2Field::State.field_index(),
        )
        .unwrap()
        .as_typed::<AccessControllerV2StateFieldPayload>()
        .unwrap()
        .into_content()
        .into_versions()
}
