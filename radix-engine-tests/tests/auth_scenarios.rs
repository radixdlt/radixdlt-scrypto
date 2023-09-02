use crate::node_modules::auth::RoleDefinition;
use radix_engine::errors::{ApplicationError, RuntimeError, SystemError, SystemModuleError};
use radix_engine::system::system_modules::auth::AuthError;
use radix_engine::types::node_modules::auth::ToRoleEntry;
use radix_engine::types::*;
use radix_engine::vm::NoExtension;
use radix_engine_interface::api::node_modules::auth::{AuthAddresses, ROLE_ASSIGNMENT_SET_IDENT, RoleAssignmentSetInput};
use radix_engine_interface::api::ModuleId;
use radix_engine_interface::blueprints::package::PackageDefinition;
use radix_engine_interface::rule;
use radix_engine_stores::memory_db::InMemorySubstateDatabase;
use scrypto_unit::*;
use transaction::prelude::*;

struct AuthScenariosEnv {
    acco: ComponentAddress,
    virtua_sig: NonFungibleGlobalId,
    cerb_badge: NonFungibleGlobalId,
    cerb: ResourceAddress,
    package: PackageAddress,
    big_fi: ComponentAddress,
    big_fi_badge: NonFungibleGlobalId,
    swappy: ComponentAddress,
    swappy_badge: NonFungibleGlobalId,
    cerb_vault: InternalAddress,
}

impl AuthScenariosEnv {
    fn init(
        test_runner: &mut TestRunner<NoExtension, InMemorySubstateDatabase>,
    ) -> AuthScenariosEnv {
        let (pub_key, _, acco) = test_runner.new_account(false);
        let virtua_sig = NonFungibleGlobalId::from_public_key(&pub_key);

        let cerb_badge_resource = test_runner.create_non_fungible_resource_advanced(
            NonFungibleResourceRoles::default(),
            acco,
            1,
        );
        let cerb_badge =
            NonFungibleGlobalId::new(cerb_badge_resource, NonFungibleLocalId::integer(1u64));

        let cerb = test_runner.create_non_fungible_resource_with_roles(
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
                ..Default::default()
            },
            acco,
        );

        let package_address = test_runner.compile_and_publish("./tests/blueprints/auth_scenarios");

        let manifest = ManifestBuilder::new()
            .call_function(package_address, "Swappy", "create", manifest_args!())
            .deposit_batch(acco)
            .build();
        let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![virtua_sig.clone()]);
        let result = receipt.expect_commit_success();
        let swappy = result.new_component_addresses()[0];
        let swappy_resource = result.new_resource_addresses()[0];
        let swappy_badge =
            NonFungibleGlobalId::new(swappy_resource, NonFungibleLocalId::integer(0u64));

        let manifest = ManifestBuilder::new()
            .call_function(
                package_address,
                "BigFi",
                "create",
                manifest_args!(cerb, swappy),
            )
            .deposit_batch(acco)
            .build();
        let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![virtua_sig.clone()]);
        let result = receipt.expect_commit_success();
        let big_fi = result.new_component_addresses()[0];
        let big_fi_resource = result.new_resource_addresses()[0];
        let big_fi_badge =
            NonFungibleGlobalId::new(big_fi_resource, NonFungibleLocalId::integer(0u64));

        let vault_id = test_runner.get_component_vaults(big_fi, cerb)[0];

        AuthScenariosEnv {
            acco,
            virtua_sig,
            cerb_badge,
            cerb,
            package: package_address,
            big_fi,
            big_fi_badge,
            swappy,
            swappy_badge,
            cerb_vault: InternalAddress::new_or_panic(vault_id.0),
        }
    }
}

#[test]
fn scenario_1() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let env = AuthScenariosEnv::init(&mut test_runner);

    // Act
    let manifest = ManifestBuilder::new()
        .create_proof_from_account_of_non_fungible(env.acco, env.swappy_badge)
        .call_method(env.swappy, "protected_method", manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![env.virtua_sig]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn scenario_2() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let env = AuthScenariosEnv::init(&mut test_runner);

    // Act
    let manifest = ManifestBuilder::new()
        .create_proof_from_account_of_non_fungible(env.acco, env.swappy_badge)
        .call_method(env.big_fi, "call_swappy", manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![env.virtua_sig]);

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
    let mut test_runner = TestRunnerBuilder::new().build();
    let env = AuthScenariosEnv::init(&mut test_runner);

    // Act
    let manifest = ManifestBuilder::new()
        .create_proof_from_account_of_non_fungible(env.acco, env.cerb_badge)
        .withdraw_from_account(env.acco, env.cerb, 1)
        .take_all_from_worktop(env.cerb, "cerbs")
        .with_bucket("cerbs", |builder, bucket| {
            builder.call_method(env.big_fi, "deposit_cerb", manifest_args!(bucket))
        })
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![env.virtua_sig]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn scenario_4() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let env = AuthScenariosEnv::init(&mut test_runner);

    // Act
    let manifest = ManifestBuilder::new()
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
    let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![env.virtua_sig]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn scenario_5() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let env = AuthScenariosEnv::init(&mut test_runner);

    // Act
    let manifest = ManifestBuilder::new()
        .create_proof_from_account_of_non_fungible(env.acco, env.cerb_badge)
        .call_method(env.big_fi, "mint_cerb", manifest_args!())
        .deposit_batch(env.acco)
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![env.virtua_sig]);

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
    let mut test_runner = TestRunnerBuilder::new().build();
    let env = AuthScenariosEnv::init(&mut test_runner);

    // Act
    let manifest = ManifestBuilder::new()
        .create_proof_from_account_of_non_fungible(env.acco, env.swappy_badge)
        .pop_from_auth_zone("Arnold")
        .call_method(env.swappy, "protected_method", manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![env.virtua_sig]);

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
    let mut test_runner = TestRunnerBuilder::new().build();
    let env = AuthScenariosEnv::init(&mut test_runner);

    // Act
    let manifest = ManifestBuilder::new()
        .create_proof_from_account_of_non_fungible(env.acco, env.swappy_badge)
        .pop_from_auth_zone("Arnold")
        .with_name_lookup(|builder, lookup| {
            let proof = lookup.proof("Arnold");
            builder.call_method(env.swappy, "public_method", manifest_args!(proof))
        })
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![env.virtua_sig]);

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
    let mut test_runner = TestRunnerBuilder::new().build();
    let env = AuthScenariosEnv::init(&mut test_runner);

    // Act
    let manifest = ManifestBuilder::new()
        .create_proof_from_account_of_non_fungible(env.acco, env.swappy_badge)
        .pop_from_auth_zone("Arnold")
        .with_name_lookup(|builder, lookup| {
            let proof = lookup.proof("Arnold");
            builder.call_method(env.swappy, "put_proof_in_auth_zone", manifest_args!(proof))
        })
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![env.virtua_sig]);

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
    let mut test_runner = TestRunnerBuilder::new().build();
    let env = AuthScenariosEnv::init(&mut test_runner);

    // Act
    let manifest = ManifestBuilder::new()
        .create_proof_from_account_of_non_fungible(env.acco, env.swappy_badge.clone())
        .create_proof_from_auth_zone_of_all(env.swappy_badge.resource_address(), "Bennet")
        .call_method(env.swappy, "protected_method", manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![env.virtua_sig]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn scenario_10() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let env = AuthScenariosEnv::init(&mut test_runner);

    // Act
    let manifest = ManifestBuilder::new()
        .create_proof_from_account_of_non_fungible(env.acco, env.cerb_badge)
        .call_method(env.big_fi, "recall_cerb", manifest_args!(env.cerb_vault))
        .deposit_batch(env.acco)
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![env.virtua_sig]);

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
    let mut test_runner = TestRunnerBuilder::new().build();
    let env = AuthScenariosEnv::init(&mut test_runner);

    // Act
    let manifest = ManifestBuilder::new()
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
    let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![env.virtua_sig]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn scenario_12() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let env = AuthScenariosEnv::init(&mut test_runner);

    // Act
    let manifest = ManifestBuilder::new()
        .create_proof_from_account_of_non_fungible(env.acco, env.swappy_badge.clone())
        .call_method(env.big_fi, "set_swappy_metadata", manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![env.virtua_sig]);

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
    let mut test_runner = TestRunnerBuilder::new().build();
    let env = AuthScenariosEnv::init(&mut test_runner);

    // Act
    let manifest = ManifestBuilder::new()
        .create_proof_from_account_of_non_fungible(env.acco, env.swappy_badge.clone())
        .call_method(env.swappy, "set_metadata", manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![env.virtua_sig]);

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
    let mut test_runner = TestRunnerBuilder::new().build();
    let env = AuthScenariosEnv::init(&mut test_runner);

    // Act
    let manifest = ManifestBuilder::new()
        .create_proof_from_account_of_non_fungible(env.acco, env.swappy_badge.clone())
        .call_method(env.big_fi, "some_method", manifest_args!())
        .call_function(env.package, "BigFi", "some_function", manifest_args!())
        .call_method(env.swappy, "protected_method", manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![env.virtua_sig]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn scenario_15() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let env = AuthScenariosEnv::init(&mut test_runner);

    // Act
    let manifest = ManifestBuilder::new()
        .withdraw_from_account(env.acco, env.swappy_badge.resource_address(), 1)
        .take_all_from_worktop(env.swappy_badge.resource_address(), "bucket")
        .with_bucket("bucket", |builder, bucket| {
            builder.call_method(env.swappy, "call_swappy_with_badge", manifest_args!(bucket))
        })
        .deposit_batch(env.acco)
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![env.virtua_sig]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn scenario_16() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let env = AuthScenariosEnv::init(&mut test_runner);

    // Act
    let manifest = ManifestBuilder::new()
        .create_proof_from_account_of_non_fungible(env.acco, env.swappy_badge.clone())
        .call_method(env.swappy, "another_protected_method", manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![env.virtua_sig]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn scenario_17() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let env = AuthScenariosEnv::init(&mut test_runner);

    // Act
    let manifest = ManifestBuilder::new()
        .create_proof_from_account_of_non_fungible(env.acco, env.swappy_badge.clone())
        .call_method(env.swappy, "another_protected_method2", manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![env.virtua_sig]);

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
    let mut test_runner = TestRunnerBuilder::new().build();
    let env = AuthScenariosEnv::init(&mut test_runner);

    // Act
    let manifest = ManifestBuilder::new()
        .create_proof_from_account_of_non_fungible(env.acco, env.swappy_badge.clone())
        .call_role_assignment_method(
            env.swappy,
                ROLE_ASSIGNMENT_SET_IDENT,
            RoleAssignmentSetInput {
                module: ObjectModuleId::Metadata,
                role_key: RoleKey::new("metadata_setter"),
                rule: AccessRule::AllowAll,
            }
        )
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![env.virtua_sig]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn scenario_19() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let env = AuthScenariosEnv::init(&mut test_runner);

    // Act
    let manifest = ManifestBuilder::new()
        .create_proof_from_account_of_non_fungible(env.acco, env.swappy_badge.clone())
        .call_method(env.big_fi, "update_swappy_metadata_rule", manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![env.virtua_sig]);

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
    let mut test_runner = TestRunnerBuilder::new().build();
    let env = AuthScenariosEnv::init(&mut test_runner);

    // Act
    let manifest = ManifestBuilder::new()
        .create_proof_from_account_of_non_fungible(env.acco, env.swappy_badge.clone())
        .call_method(env.swappy, "update_metadata_rule", manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![env.virtua_sig]);

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
    let mut test_runner = TestRunnerBuilder::new().build();
    let env = AuthScenariosEnv::init(&mut test_runner);

    // Act
    let manifest = ManifestBuilder::new()
        .create_proof_from_account_of_non_fungible(env.acco, env.cerb_badge.clone())
        .call_role_assignment_method(
            env.cerb,
            ROLE_ASSIGNMENT_SET_IDENT,
            RoleAssignmentSetInput {
                module: ObjectModuleId::Main,
                role_key: RoleKey::new("withdrawer"),
                rule: AccessRule::AllowAll,
            }
        )
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![env.virtua_sig]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn scenario_22() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let env = AuthScenariosEnv::init(&mut test_runner);

    // Act
    let manifest = ManifestBuilder::new()
        .create_proof_from_account_of_non_fungible(env.acco, env.swappy_badge.clone())
        .call_method(env.big_fi, "update_cerb_rule", manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![env.virtua_sig]);

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
    let mut test_runner = TestRunnerBuilder::new().build();
    let env = AuthScenariosEnv::init(&mut test_runner);

    // Act
    let manifest = ManifestBuilder::new()
        .create_proof_from_account_of_non_fungible(env.acco, env.swappy_badge)
        .call_function(env.package, "BigFi", "call_swappy_func", manifest_args!(env.swappy))
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![env.virtua_sig]);

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
    let mut test_runner = TestRunnerBuilder::new().build();
    let env = AuthScenariosEnv::init(&mut test_runner);

    // Act
    let manifest = ManifestBuilder::new()
        .create_proof_from_account_of_amount(env.acco, XRD, 1)
        .call_method(env.big_fi, "call_swappy_function", manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![env.virtua_sig]);

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