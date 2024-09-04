use radix_common::prelude::*;
use radix_engine::errors::{ApplicationError, RuntimeError, SystemError, SystemModuleError};
use radix_engine::system::system_modules::auth::AuthError;
use radix_engine::vm::NoExtension;
use radix_engine_interface::object_modules::role_assignment::{
    RoleAssignmentSetInput, ROLE_ASSIGNMENT_SET_IDENT,
};
use radix_engine_interface::rule;
use radix_engine_tests::common::*;
use radix_substate_store_impls::memory_db::InMemorySubstateDatabase;
use scrypto_test::prelude::*;

pub struct AuthScenariosEnv {
    acco: ComponentAddress,
    virtua_sig: NonFungibleGlobalId,
    cerb_badge: NonFungibleGlobalId,
    cerb: ResourceAddress,
    package: PackageAddress,
    big_fi: ComponentAddress,
    swappy: ComponentAddress,
    swappy_badge: NonFungibleGlobalId,
    cerb_vault: InternalAddress,
}

impl AuthScenariosEnv {
    fn init(
        ledger: &mut LedgerSimulator<NoExtension, InMemorySubstateDatabase>,
    ) -> AuthScenariosEnv {
        let (pub_key, _, acco) = ledger.new_account(false);
        let virtua_sig = NonFungibleGlobalId::from_public_key(&pub_key);

        let cerb_badge_resource = ledger.create_non_fungible_resource_advanced(
            NonFungibleResourceRoles::default(),
            acco,
            1,
        );
        let cerb_badge =
            NonFungibleGlobalId::new(cerb_badge_resource, NonFungibleLocalId::integer(1u64));

        let cerb = ledger.create_non_fungible_resource_with_roles(
            NonFungibleResourceRoles {
                burn_roles: burn_roles! {
                    burner => rule!(require(cerb_badge.clone()));
                    burner_updater => rule!(deny_all);
                },
                recall_roles: recall_roles! {
                    recaller => rule!(require(cerb_badge.clone()));
                    recaller_updater => rule!(deny_all);
                },
                freeze_roles: freeze_roles! {
                    freezer => rule!(require(cerb_badge.clone()));
                    freezer_updater => rule!(deny_all);
                },
                withdraw_roles: withdraw_roles! {
                    withdrawer => rule!(require(cerb_badge.clone()));
                    withdrawer_updater => rule!(require(cerb_badge.clone()));
                },
                deposit_roles: deposit_roles! {
                    depositor => rule!(allow_all);
                    depositor_updater => rule!(allow_all);
                },
                ..Default::default()
            },
            acco,
        );
        ledger.execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .call_role_assignment_method(
                    cerb,
                    ROLE_ASSIGNMENT_SET_IDENT,
                    RoleAssignmentSetInput {
                        module: ObjectModuleId::Main,
                        role_key: RoleKey::new("depositor"),
                        rule: rule!(require(cerb_badge.clone())),
                    },
                )
                .build(),
            vec![],
        );

        let package_address = ledger.publish_package_simple(PackageLoader::get("auth_scenarios"));

        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_function(package_address, "Swappy", "create", manifest_args!(cerb))
            .deposit_entire_worktop(acco)
            .build();
        let receipt = ledger.execute_manifest(manifest, vec![virtua_sig.clone()]);
        let result = receipt.expect_commit_success();
        let swappy = result.new_component_addresses()[0];
        let swappy_resource = result.new_resource_addresses()[0];
        let swappy_badge =
            NonFungibleGlobalId::new(swappy_resource, NonFungibleLocalId::integer(0u64));

        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_function(
                package_address,
                "BigFi",
                "create",
                manifest_args!(cerb, swappy),
            )
            .deposit_entire_worktop(acco)
            .build();
        let receipt = ledger.execute_manifest(manifest, vec![virtua_sig.clone()]);
        let result = receipt.expect_commit_success();
        let big_fi = result.new_component_addresses()[0];
        let vault_id = ledger.get_component_vaults(big_fi, cerb)[0];

        AuthScenariosEnv {
            acco,
            virtua_sig,
            cerb_badge,
            cerb,
            package: package_address,
            big_fi,
            swappy,
            swappy_badge,
            cerb_vault: InternalAddress::new_or_panic(vault_id.0),
        }
    }
}

#[test]
fn scenario_1() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let env = AuthScenariosEnv::init(&mut ledger);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_non_fungible(env.acco, env.swappy_badge)
        .call_method(env.swappy, "protected_method", manifest_args!())
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![env.virtua_sig]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn scenario_1_with_injected_costing_error() {
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let env = AuthScenariosEnv::init(&mut ledger);

    let mut inject_err_after_count = 1u64;

    loop {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_proof_from_account_of_non_fungible(env.acco, env.swappy_badge.clone())
            .call_method(env.swappy, "protected_method", manifest_args!())
            .build();
        let receipt = ledger.execute_manifest_with_injected_error(
            manifest,
            vec![env.virtua_sig.clone()],
            inject_err_after_count,
        );
        if receipt.is_commit_success() {
            break;
        }

        inject_err_after_count += 1u64;
    }

    println!("Count: {:?}", inject_err_after_count);
}

#[test]
fn scenario_2() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let env = AuthScenariosEnv::init(&mut ledger);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_non_fungible(env.acco, env.swappy_badge)
        .call_method(env.big_fi, "call_swappy", manifest_args!())
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![env.virtua_sig]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemModuleError(SystemModuleError::AuthError(AuthError::Unauthorized(
                ..
            )))
        )
    });
}

#[test]
fn scenario_3() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let env = AuthScenariosEnv::init(&mut ledger);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_non_fungible(env.acco, env.cerb_badge)
        .withdraw_from_account(env.acco, env.cerb, 1)
        .take_all_from_worktop(env.cerb, "cerbs")
        .with_bucket("cerbs", |builder, bucket| {
            builder.call_method(env.big_fi, "deposit_cerb", manifest_args!(bucket))
        })
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![env.virtua_sig]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn scenario_4() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let env = AuthScenariosEnv::init(&mut ledger);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_non_fungible(env.acco, env.cerb_badge)
        .withdraw_from_account(env.acco, env.cerb, 1)
        .take_all_from_worktop(env.cerb, "cerbs")
        .with_bucket("cerbs", |builder, bucket| {
            builder.call_method(
                env.big_fi,
                "deposit_cerb_into_subservio",
                manifest_args!(bucket),
            )
        })
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![env.virtua_sig]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn scenario_5() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let env = AuthScenariosEnv::init(&mut ledger);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_non_fungible(env.acco, env.cerb_badge)
        .call_method(env.big_fi, "mint_cerb", manifest_args!())
        .deposit_entire_worktop(env.acco)
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![env.virtua_sig]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemModuleError(SystemModuleError::AuthError(AuthError::Unauthorized(
                ..
            )))
        )
    });
}

#[test]
fn scenario_6() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let env = AuthScenariosEnv::init(&mut ledger);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_non_fungible(env.acco, env.swappy_badge)
        .pop_from_auth_zone("Arnold")
        .call_method(env.swappy, "protected_method", manifest_args!())
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![env.virtua_sig]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemModuleError(SystemModuleError::AuthError(AuthError::Unauthorized(
                ..
            )))
        )
    });
}

#[test]
fn scenario_7() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let env = AuthScenariosEnv::init(&mut ledger);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_non_fungible(env.acco, env.swappy_badge)
        .pop_from_auth_zone("Arnold")
        .with_name_lookup(|builder, lookup| {
            let proof = lookup.proof("Arnold");
            builder.call_method(env.swappy, "public_method", manifest_args!(proof))
        })
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![env.virtua_sig]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemError(SystemError::AssertAccessRuleFailed)
        )
    });
}

#[test]
fn scenario_8() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let env = AuthScenariosEnv::init(&mut ledger);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_non_fungible(env.acco, env.swappy_badge)
        .pop_from_auth_zone("Arnold")
        .with_name_lookup(|builder, lookup| {
            let proof = lookup.proof("Arnold");
            builder.call_method(env.swappy, "put_proof_in_auth_zone", manifest_args!(proof))
        })
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![env.virtua_sig]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::PanicMessage(..))
        )
    });
}

#[test]
fn scenario_9() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let env = AuthScenariosEnv::init(&mut ledger);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_non_fungible(env.acco, env.swappy_badge.clone())
        .create_proof_from_auth_zone_of_all(env.swappy_badge.resource_address(), "Bennet")
        .call_method(env.swappy, "protected_method", manifest_args!())
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![env.virtua_sig]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn scenario_10() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let env = AuthScenariosEnv::init(&mut ledger);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_non_fungible(env.acco, env.cerb_badge)
        .call_method(env.big_fi, "recall_cerb", manifest_args!(env.cerb_vault))
        .deposit_entire_worktop(env.acco)
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![env.virtua_sig]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemModuleError(SystemModuleError::AuthError(AuthError::Unauthorized(
                ..
            )))
        )
    });
}

#[test]
fn scenario_11() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let env = AuthScenariosEnv::init(&mut ledger);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_non_fungible(env.acco, env.swappy_badge.clone())
        .call_metadata_method(
            env.swappy,
            METADATA_SET_IDENT,
            MetadataSetInput {
                key: "key".to_string(),
                value: MetadataValue::String("value".to_string()),
            },
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![env.virtua_sig]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn scenario_12() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let env = AuthScenariosEnv::init(&mut ledger);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_non_fungible(env.acco, env.swappy_badge.clone())
        .call_method(env.big_fi, "set_swappy_metadata", manifest_args!())
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![env.virtua_sig]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemModuleError(SystemModuleError::AuthError(AuthError::Unauthorized(
                ..
            )))
        )
    });
}

#[test]
fn scenario_13() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let env = AuthScenariosEnv::init(&mut ledger);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_non_fungible(env.acco, env.swappy_badge.clone())
        .call_method(env.swappy, "set_metadata", manifest_args!())
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![env.virtua_sig]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemModuleError(SystemModuleError::AuthError(AuthError::Unauthorized(
                ..
            )))
        )
    });
}

#[test]
fn scenario_14() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let env = AuthScenariosEnv::init(&mut ledger);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_non_fungible(env.acco, env.swappy_badge.clone())
        .call_method(env.big_fi, "some_method", manifest_args!())
        .call_function(env.package, "BigFi", "some_function", manifest_args!())
        .call_method(env.swappy, "protected_method", manifest_args!())
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![env.virtua_sig]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn scenario_15() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let env = AuthScenariosEnv::init(&mut ledger);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .withdraw_from_account(env.acco, env.swappy_badge.resource_address(), 1)
        .take_all_from_worktop(env.swappy_badge.resource_address(), "bucket")
        .with_bucket("bucket", |builder, bucket| {
            builder.call_method(env.big_fi, "call_swappy_with_badge", manifest_args!(bucket))
        })
        .deposit_entire_worktop(env.acco)
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![env.virtua_sig]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn scenario_16() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let env = AuthScenariosEnv::init(&mut ledger);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_non_fungible(env.acco, env.swappy_badge.clone())
        .call_method(env.swappy, "another_protected_method", manifest_args!())
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![env.virtua_sig]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn scenario_17() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let env = AuthScenariosEnv::init(&mut ledger);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_non_fungible(env.acco, env.swappy_badge.clone())
        .call_method(env.swappy, "another_protected_method2", manifest_args!())
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![env.virtua_sig]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemModuleError(SystemModuleError::AuthError(AuthError::Unauthorized(
                ..
            )))
        )
    });
}

#[test]
fn scenario_18() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let env = AuthScenariosEnv::init(&mut ledger);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_non_fungible(env.acco, env.swappy_badge.clone())
        .call_role_assignment_method(
            env.swappy,
            ROLE_ASSIGNMENT_SET_IDENT,
            RoleAssignmentSetInput {
                module: ObjectModuleId::Metadata,
                role_key: RoleKey::new("metadata_setter"),
                rule: AccessRule::AllowAll,
            },
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![env.virtua_sig]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn scenario_19() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let env = AuthScenariosEnv::init(&mut ledger);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_non_fungible(env.acco, env.swappy_badge.clone())
        .call_method(env.big_fi, "update_swappy_metadata_rule", manifest_args!())
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![env.virtua_sig]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemModuleError(SystemModuleError::AuthError(AuthError::Unauthorized(
                ..
            )))
        )
    });
}

#[test]
fn scenario_20() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let env = AuthScenariosEnv::init(&mut ledger);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_non_fungible(env.acco, env.swappy_badge.clone())
        .call_method(env.swappy, "update_metadata_rule", manifest_args!())
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![env.virtua_sig]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemModuleError(SystemModuleError::AuthError(AuthError::Unauthorized(
                ..
            )))
        )
    });
}

#[test]
fn scenario_21() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let env = AuthScenariosEnv::init(&mut ledger);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_non_fungible(env.acco, env.cerb_badge.clone())
        .call_role_assignment_method(
            env.cerb,
            ROLE_ASSIGNMENT_SET_IDENT,
            RoleAssignmentSetInput {
                module: ObjectModuleId::Main,
                role_key: RoleKey::new("withdrawer"),
                rule: AccessRule::AllowAll,
            },
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![env.virtua_sig]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn scenario_22() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let env = AuthScenariosEnv::init(&mut ledger);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_non_fungible(env.acco, env.swappy_badge.clone())
        .call_method(env.big_fi, "update_cerb_rule", manifest_args!())
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![env.virtua_sig]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemModuleError(SystemModuleError::AuthError(AuthError::Unauthorized(
                ..
            )))
        )
    });
}

#[test]
fn scenario_23() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let env = AuthScenariosEnv::init(&mut ledger);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_non_fungible(env.acco, env.swappy_badge)
        .call_function(
            env.package,
            "BigFi",
            "call_swappy_func",
            manifest_args!(env.swappy),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![env.virtua_sig]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemModuleError(SystemModuleError::AuthError(AuthError::Unauthorized(
                ..
            )))
        )
    });
}

#[test]
fn scenario_24() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let env = AuthScenariosEnv::init(&mut ledger);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_amount(env.acco, XRD, 1)
        .call_method(env.big_fi, "call_swappy_function", manifest_args!())
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![env.virtua_sig]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemModuleError(SystemModuleError::AuthError(AuthError::Unauthorized(
                ..
            )))
        )
    });
}

#[test]
fn scenario_25() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let env = AuthScenariosEnv::init(&mut ledger);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_non_fungible(env.acco, env.cerb_badge)
        .withdraw_from_account(env.acco, env.cerb, 1)
        .take_all_from_worktop(env.cerb, "bucket")
        .with_bucket("bucket", |builder, bucket| {
            builder.call_method(env.big_fi, "burn_bucket", manifest_args!(bucket))
        })
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![env.virtua_sig]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemModuleError(SystemModuleError::AuthError(AuthError::Unauthorized(
                ..
            )))
        )
    });
}

#[test]
fn scenario_26() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let env = AuthScenariosEnv::init(&mut ledger);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_non_fungible(env.acco, env.cerb_badge)
        .withdraw_from_account(env.acco, env.cerb, 1)
        .take_all_from_worktop(env.cerb, "cerbs")
        .with_bucket("cerbs", |builder, bucket| {
            builder.call_method(env.big_fi, "deposit_cerb", manifest_args!(bucket))
        })
        .call_method(env.big_fi, "burn_vault", manifest_args!())
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![env.virtua_sig]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn scenario_27() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let env = AuthScenariosEnv::init(&mut ledger);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_non_fungible(env.acco, env.cerb_badge)
        .withdraw_from_account(env.acco, env.cerb, 1)
        .withdraw_from_account(env.acco, env.swappy_badge.resource_address(), 1)
        .take_all_from_worktop(env.cerb, "cerbs")
        .take_all_from_worktop(env.swappy_badge.resource_address(), "swappy")
        .with_bucket("cerbs", |builder, bucket| {
            builder.call_method(env.big_fi, "deposit_cerb", manifest_args!(bucket))
        })
        .with_bucket("swappy", |builder, bucket| {
            builder.call_method(env.big_fi, "assert_in_subservio", manifest_args!(bucket))
        })
        .deposit_entire_worktop(env.acco)
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![env.virtua_sig]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemError(SystemError::AssertAccessRuleFailed)
        )
    });
}

#[test]
fn scenario_28() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let env = AuthScenariosEnv::init(&mut ledger);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_non_fungible(env.acco, env.cerb_badge)
        .withdraw_from_account(env.acco, env.cerb, 1)
        .withdraw_from_account(env.acco, env.swappy_badge.resource_address(), 1)
        .take_all_from_worktop(env.cerb, "cerbs")
        .take_all_from_worktop(env.swappy_badge.resource_address(), "swappy")
        .with_bucket("cerbs", |builder, bucket| {
            builder.call_method(env.big_fi, "deposit_cerb", manifest_args!(bucket))
        })
        .with_bucket("swappy", |builder, bucket| {
            builder.call_method(
                env.big_fi,
                "call_swappy_in_subservio",
                manifest_args!(bucket),
            )
        })
        .deposit_entire_worktop(env.acco)
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![env.virtua_sig]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn scenario_29() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let env = AuthScenariosEnv::init(&mut ledger);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_non_fungible(
            env.acco,
            NonFungibleGlobalId::new(env.cerb, NonFungibleLocalId::integer(1)),
        )
        .create_proof_from_auth_zone_of_all(env.cerb, "cerb_proof")
        .with_name_lookup(|builder, lookup| {
            builder.call_method(
                env.big_fi,
                "pass_proof",
                manifest_args!(lookup.proof("cerb_proof")),
            )
        })
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![env.virtua_sig]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn scenario_30() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let env = AuthScenariosEnv::init(&mut ledger);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_non_fungible(env.acco, env.cerb_badge)
        .withdraw_from_account(env.acco, env.cerb, 3)
        .take_all_from_worktop(env.cerb, "cerbs")
        .with_bucket("cerbs", |builder, bucket| {
            builder.call_method(env.big_fi, "deposit_cerb", manifest_args!(bucket))
        })
        .call_method(env.big_fi, "create_and_pass_proof", manifest_args!())
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![env.virtua_sig]);

    // Assert
    receipt.expect_commit_success();
}
