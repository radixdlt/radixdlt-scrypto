use radix_engine::errors::{RuntimeError, SystemModuleError};
use radix_engine::system::system_modules::auth::AuthError;
use radix_engine::types::*;
use radix_engine_interface::api::node_modules::ModuleConfig;
use radix_engine_interface::{metadata, metadata_init};
use scrypto::NonFungibleData;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn package_burn_is_only_callable_within_resource_package() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let resource_address = {
        let access_rules = btreemap!(
            Mint => (AccessRule::AllowAll, AccessRule::DenyAll),
            Burn => (AccessRule::AllowAll, AccessRule::DenyAll),
            Withdraw => (AccessRule::AllowAll, AccessRule::DenyAll),
            Deposit => (AccessRule::AllowAll, AccessRule::DenyAll),
            Recall => (AccessRule::AllowAll, AccessRule::DenyAll),
            UpdateNonFungibleData => (AccessRule::AllowAll, AccessRule::DenyAll),
        );
        let manifest = ManifestBuilder::new()
            .create_fungible_resource(OwnerRole::None, true, 18, metadata!(), access_rules, None)
            .build();
        test_runner
            .execute_manifest_ignoring_fee(manifest, vec![])
            .expect_commit_success()
            .new_resource_addresses()
            .get(0)
            .unwrap()
            .clone()
    };

    // Act
    let manifest = ManifestBuilder::new()
        .mint_fungible(resource_address, 10.into())
        .take_all_from_worktop(resource_address, |builder, bucket| {
            builder.call_method(
                resource_address,
                RESOURCE_MANAGER_PACKAGE_BURN_IDENT,
                manifest_args!(bucket),
            )
        })
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(is_auth_unauthorized_error);
}

#[test]
fn can_burn_by_amount_from_fungible_vault() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/vault");
    let resource_address = {
        let access_rules = btreemap!(
            Mint => (AccessRule::AllowAll, AccessRule::DenyAll),
            Burn => (AccessRule::AllowAll, AccessRule::DenyAll),
            Withdraw => (AccessRule::AllowAll, AccessRule::DenyAll),
            Deposit => (AccessRule::AllowAll, AccessRule::DenyAll),
            Recall => (AccessRule::AllowAll, AccessRule::DenyAll),
            UpdateNonFungibleData => (AccessRule::AllowAll, AccessRule::DenyAll),
        );
        let manifest = ManifestBuilder::new()
            .create_fungible_resource(OwnerRole::None, true, 18, metadata!(), access_rules, None)
            .build();
        test_runner
            .execute_manifest_ignoring_fee(manifest, vec![])
            .expect_commit_success()
            .new_resource_addresses()
            .get(0)
            .unwrap()
            .clone()
    };

    let component_address = {
        let manifest = ManifestBuilder::new()
            .mint_fungible(resource_address, 100.into())
            .take_all_from_worktop(resource_address, |builder, bucket| {
                builder.call_function(package_address, "VaultBurn", "new", manifest_args!(bucket))
            })
            .build();
        test_runner
            .execute_manifest_ignoring_fee(manifest, vec![])
            .expect_commit_success()
            .new_component_addresses()
            .get(0)
            .unwrap()
            .clone()
    };
    let vault_id = get_vault_id(&mut test_runner, component_address);

    // Act
    let manifest = ManifestBuilder::new()
        .call_method(component_address, "burn_amount", manifest_args!(dec!("50")))
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
    assert_eq!(
        test_runner.inspect_fungible_vault(vault_id).unwrap(),
        dec!("50")
    )
}

#[test]
fn can_burn_by_amount_from_non_fungible_vault() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/vault");
    let resource_address = {
        let access_rules = btreemap!(
            Mint => (AccessRule::AllowAll, AccessRule::DenyAll),
            Burn => (AccessRule::AllowAll, AccessRule::DenyAll),
            Withdraw => (AccessRule::AllowAll, AccessRule::DenyAll),
            Deposit => (AccessRule::AllowAll, AccessRule::DenyAll),
            Recall => (AccessRule::AllowAll, AccessRule::DenyAll),
            UpdateNonFungibleData => (AccessRule::AllowAll, AccessRule::DenyAll),
        );
        let manifest = ManifestBuilder::new()
            .create_non_fungible_resource(
                OwnerRole::None,
                NonFungibleIdType::Integer,
                true,
                metadata!(),
                access_rules,
                Option::<BTreeMap<NonFungibleLocalId, EmptyStruct>>::None,
            )
            .build();
        test_runner
            .execute_manifest_ignoring_fee(manifest, vec![])
            .expect_commit_success()
            .new_resource_addresses()
            .get(0)
            .unwrap()
            .clone()
    };

    let component_address = {
        let manifest = ManifestBuilder::new()
            .mint_non_fungible(
                resource_address,
                btreemap!(
                    NonFungibleLocalId::integer(1) => EmptyStruct {},
                    NonFungibleLocalId::integer(2) => EmptyStruct {},
                ),
            )
            .take_all_from_worktop(resource_address, |builder, bucket| {
                builder.call_function(package_address, "VaultBurn", "new", manifest_args!(bucket))
            })
            .build();
        test_runner
            .execute_manifest_ignoring_fee(manifest, vec![])
            .expect_commit_success()
            .new_component_addresses()
            .get(0)
            .unwrap()
            .clone()
    };
    let vault_id = get_vault_id(&mut test_runner, component_address);

    // Act
    let manifest = ManifestBuilder::new()
        .call_method(component_address, "burn_amount", manifest_args!(dec!("1")))
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
    assert_eq!(
        test_runner.inspect_non_fungible_vault(vault_id).unwrap().0,
        dec!("1")
    )
}

#[test]
fn can_burn_by_ids_from_non_fungible_vault() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/vault");
    let resource_address = {
        let access_rules = btreemap!(
            Mint => (AccessRule::AllowAll, AccessRule::DenyAll),
            Burn => (AccessRule::AllowAll, AccessRule::DenyAll),
            Withdraw => (AccessRule::AllowAll, AccessRule::DenyAll),
            Deposit => (AccessRule::AllowAll, AccessRule::DenyAll),
            Recall => (AccessRule::AllowAll, AccessRule::DenyAll),
            UpdateNonFungibleData => (AccessRule::AllowAll, AccessRule::DenyAll),
        );
        let manifest = ManifestBuilder::new()
            .create_non_fungible_resource(
                OwnerRole::None,
                NonFungibleIdType::Integer,
                true,
                metadata!(),
                access_rules,
                Option::<BTreeMap<NonFungibleLocalId, EmptyStruct>>::None,
            )
            .build();
        test_runner
            .execute_manifest_ignoring_fee(manifest, vec![])
            .expect_commit_success()
            .new_resource_addresses()
            .get(0)
            .unwrap()
            .clone()
    };

    let component_address = {
        let manifest = ManifestBuilder::new()
            .mint_non_fungible(
                resource_address,
                btreemap!(
                    NonFungibleLocalId::integer(1) => EmptyStruct {},
                    NonFungibleLocalId::integer(2) => EmptyStruct {},
                ),
            )
            .take_all_from_worktop(resource_address, |builder, bucket| {
                builder.call_function(package_address, "VaultBurn", "new", manifest_args!(bucket))
            })
            .build();
        test_runner
            .execute_manifest_ignoring_fee(manifest, vec![])
            .expect_commit_success()
            .new_component_addresses()
            .get(0)
            .unwrap()
            .clone()
    };
    let vault_id = get_vault_id(&mut test_runner, component_address);

    // Act
    let manifest = ManifestBuilder::new()
        .call_method(
            component_address,
            "burn_ids",
            manifest_args!(btreeset![NonFungibleLocalId::integer(1)]),
        )
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
    assert_eq!(
        test_runner.inspect_non_fungible_vault(vault_id).unwrap().0,
        dec!("1")
    );
}

#[test]
fn can_burn_by_amount_from_fungible_vault_with_an_access_rule() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/vault");
    let (public_key, _, _) = test_runner.new_account(false);
    let virtual_signature_badge = NonFungibleGlobalId::from_public_key(&public_key);
    let virtual_signature_rule = rule!(require(virtual_signature_badge.clone()));
    let resource_address = {
        let access_rules = btreemap!(
            Mint => (virtual_signature_rule.clone(), AccessRule::DenyAll),
            Burn => (virtual_signature_rule.clone(), AccessRule::DenyAll),
            Withdraw => (virtual_signature_rule.clone(), AccessRule::DenyAll),
            Deposit => (virtual_signature_rule.clone(), AccessRule::DenyAll),
            Recall => (virtual_signature_rule.clone(), AccessRule::DenyAll),
            UpdateNonFungibleData => (virtual_signature_rule.clone(), AccessRule::DenyAll),
        );
        let manifest = ManifestBuilder::new()
            .create_fungible_resource(OwnerRole::None, true, 18, metadata!(), access_rules, None)
            .build();
        test_runner
            .execute_manifest_ignoring_fee(manifest, vec![virtual_signature_badge.clone()])
            .expect_commit_success()
            .new_resource_addresses()
            .get(0)
            .unwrap()
            .clone()
    };

    let component_address = {
        let manifest = ManifestBuilder::new()
            .mint_fungible(resource_address, 100.into())
            .take_all_from_worktop(resource_address, |builder, bucket| {
                builder.call_function(package_address, "VaultBurn", "new", manifest_args!(bucket))
            })
            .build();
        test_runner
            .execute_manifest_ignoring_fee(manifest, vec![virtual_signature_badge.clone()])
            .expect_commit_success()
            .new_component_addresses()
            .get(0)
            .unwrap()
            .clone()
    };
    let vault_id = get_vault_id(&mut test_runner, component_address);

    // Act
    let manifest = ManifestBuilder::new()
        .call_method(component_address, "burn_amount", manifest_args!(dec!("50")))
        .build();
    let receipt =
        test_runner.execute_manifest_ignoring_fee(manifest, vec![virtual_signature_badge]);

    // Assert
    receipt.expect_commit_success();
    assert_eq!(
        test_runner.inspect_fungible_vault(vault_id).unwrap(),
        dec!("50")
    )
}

#[test]
fn can_burn_by_amount_from_non_fungible_vault_with_an_access_rule() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/vault");
    let (public_key, _, _) = test_runner.new_account(false);
    let virtual_signature_badge = NonFungibleGlobalId::from_public_key(&public_key);
    let virtual_signature_rule = rule!(require(virtual_signature_badge.clone()));
    let resource_address = {
        let access_rules = btreemap!(
            Mint => (virtual_signature_rule.clone(), AccessRule::DenyAll),
            Burn => (virtual_signature_rule.clone(), AccessRule::DenyAll),
            Withdraw => (virtual_signature_rule.clone(), AccessRule::DenyAll),
            Deposit => (virtual_signature_rule.clone(), AccessRule::DenyAll),
            Recall => (virtual_signature_rule.clone(), AccessRule::DenyAll),
            UpdateNonFungibleData => (virtual_signature_rule.clone(), AccessRule::DenyAll),
        );
        let manifest = ManifestBuilder::new()
            .create_non_fungible_resource(
                OwnerRole::None,
                NonFungibleIdType::Integer,
                true,
                metadata!(),
                access_rules,
                Option::<BTreeMap<NonFungibleLocalId, EmptyStruct>>::None,
            )
            .build();
        test_runner
            .execute_manifest_ignoring_fee(manifest, vec![virtual_signature_badge.clone()])
            .expect_commit_success()
            .new_resource_addresses()
            .get(0)
            .unwrap()
            .clone()
    };

    let component_address = {
        let manifest = ManifestBuilder::new()
            .mint_non_fungible(
                resource_address,
                btreemap!(
                    NonFungibleLocalId::integer(1) => EmptyStruct {},
                    NonFungibleLocalId::integer(2) => EmptyStruct {},
                ),
            )
            .take_all_from_worktop(resource_address, |builder, bucket| {
                builder.call_function(package_address, "VaultBurn", "new", manifest_args!(bucket))
            })
            .build();
        test_runner
            .execute_manifest_ignoring_fee(manifest, vec![virtual_signature_badge.clone()])
            .expect_commit_success()
            .new_component_addresses()
            .get(0)
            .unwrap()
            .clone()
    };
    let vault_id = get_vault_id(&mut test_runner, component_address);

    // Act
    let manifest = ManifestBuilder::new()
        .call_method(component_address, "burn_amount", manifest_args!(dec!("1")))
        .build();
    let receipt =
        test_runner.execute_manifest_ignoring_fee(manifest, vec![virtual_signature_badge.clone()]);

    // Assert
    receipt.expect_commit_success();
    assert_eq!(
        test_runner.inspect_non_fungible_vault(vault_id).unwrap().0,
        dec!("1")
    )
}

#[test]
fn can_burn_by_ids_from_non_fungible_vault_with_an_access_rule() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/vault");
    let (public_key, _, _) = test_runner.new_account(false);
    let virtual_signature_badge = NonFungibleGlobalId::from_public_key(&public_key);
    let virtual_signature_rule = rule!(require(virtual_signature_badge.clone()));
    let resource_address = {
        let access_rules = btreemap!(
            Mint => (virtual_signature_rule.clone(), AccessRule::DenyAll),
            Burn => (virtual_signature_rule.clone(), AccessRule::DenyAll),
            Withdraw => (virtual_signature_rule.clone(), AccessRule::DenyAll),
            Deposit => (virtual_signature_rule.clone(), AccessRule::DenyAll),
            Recall => (virtual_signature_rule.clone(), AccessRule::DenyAll),
            UpdateNonFungibleData => (virtual_signature_rule.clone(), AccessRule::DenyAll),
        );
        let manifest = ManifestBuilder::new()
            .create_non_fungible_resource(
                OwnerRole::None,
                NonFungibleIdType::Integer,
                true,
                metadata!(),
                access_rules,
                Option::<BTreeMap<NonFungibleLocalId, EmptyStruct>>::None,
            )
            .build();
        test_runner
            .execute_manifest_ignoring_fee(manifest, vec![virtual_signature_badge.clone()])
            .expect_commit_success()
            .new_resource_addresses()
            .get(0)
            .unwrap()
            .clone()
    };

    let component_address = {
        let manifest = ManifestBuilder::new()
            .mint_non_fungible(
                resource_address,
                btreemap!(
                    NonFungibleLocalId::integer(1) => EmptyStruct {},
                    NonFungibleLocalId::integer(2) => EmptyStruct {},
                ),
            )
            .take_all_from_worktop(resource_address, |builder, bucket| {
                builder.call_function(package_address, "VaultBurn", "new", manifest_args!(bucket))
            })
            .build();
        test_runner
            .execute_manifest_ignoring_fee(manifest, vec![virtual_signature_badge.clone()])
            .expect_commit_success()
            .new_component_addresses()
            .get(0)
            .unwrap()
            .clone()
    };
    let vault_id = get_vault_id(&mut test_runner, component_address);

    // Act
    let manifest = ManifestBuilder::new()
        .call_method(
            component_address,
            "burn_ids",
            manifest_args!(btreeset![NonFungibleLocalId::integer(1)]),
        )
        .build();
    let receipt =
        test_runner.execute_manifest_ignoring_fee(manifest, vec![virtual_signature_badge.clone()]);

    // Assert
    receipt.expect_commit_success();
    assert_eq!(
        test_runner.inspect_non_fungible_vault(vault_id).unwrap().0,
        dec!("1")
    );
}

#[test]
fn cant_burn_by_amount_from_fungible_vault_with_an_access_rule_that_is_not_fulfilled() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/vault");
    let (public_key, _, _) = test_runner.new_account(false);
    let virtual_signature_badge = NonFungibleGlobalId::from_public_key(&public_key);
    let virtual_signature_rule = rule!(require(virtual_signature_badge.clone()));
    let resource_address = {
        let access_rules = btreemap!(
            Mint => (virtual_signature_rule.clone(), AccessRule::DenyAll),
            Burn => (virtual_signature_rule.clone(), AccessRule::DenyAll),
            Withdraw => (virtual_signature_rule.clone(), AccessRule::DenyAll),
            Deposit => (virtual_signature_rule.clone(), AccessRule::DenyAll),
            Recall => (virtual_signature_rule.clone(), AccessRule::DenyAll),
            UpdateNonFungibleData => (virtual_signature_rule.clone(), AccessRule::DenyAll),
        );
        let manifest = ManifestBuilder::new()
            .create_fungible_resource(OwnerRole::None, true, 18, metadata!(), access_rules, None)
            .build();
        test_runner
            .execute_manifest_ignoring_fee(manifest, vec![virtual_signature_badge.clone()])
            .expect_commit_success()
            .new_resource_addresses()
            .get(0)
            .unwrap()
            .clone()
    };

    let component_address = {
        let manifest = ManifestBuilder::new()
            .mint_fungible(resource_address, 100.into())
            .take_all_from_worktop(resource_address, |builder, bucket| {
                builder.call_function(package_address, "VaultBurn", "new", manifest_args!(bucket))
            })
            .build();
        test_runner
            .execute_manifest_ignoring_fee(manifest, vec![virtual_signature_badge.clone()])
            .expect_commit_success()
            .new_component_addresses()
            .get(0)
            .unwrap()
            .clone()
    };
    let vault_id = get_vault_id(&mut test_runner, component_address);

    // Act
    let manifest = ManifestBuilder::new()
        .call_method(component_address, "burn_amount", manifest_args!(dec!("50")))
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(is_auth_unauthorized_error);
    assert_eq!(
        test_runner.inspect_fungible_vault(vault_id).unwrap(),
        dec!("100")
    )
}

#[test]
fn cant_burn_by_amount_from_non_fungible_vault_with_an_access_rule_that_is_not_fulfilled() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/vault");
    let (public_key, _, _) = test_runner.new_account(false);
    let virtual_signature_badge = NonFungibleGlobalId::from_public_key(&public_key);
    let virtual_signature_rule = rule!(require(virtual_signature_badge.clone()));
    let resource_address = {
        let access_rules = btreemap!(
            Mint => (virtual_signature_rule.clone(), AccessRule::DenyAll),
            Burn => (virtual_signature_rule.clone(), AccessRule::DenyAll),
            Withdraw => (virtual_signature_rule.clone(), AccessRule::DenyAll),
            Deposit => (virtual_signature_rule.clone(), AccessRule::DenyAll),
            Recall => (virtual_signature_rule.clone(), AccessRule::DenyAll),
            UpdateNonFungibleData => (virtual_signature_rule.clone(), AccessRule::DenyAll),
        );
        let manifest = ManifestBuilder::new()
            .create_non_fungible_resource(
                OwnerRole::None,
                NonFungibleIdType::Integer,
                true,
                metadata!(),
                access_rules,
                Option::<BTreeMap<NonFungibleLocalId, EmptyStruct>>::None,
            )
            .build();
        test_runner
            .execute_manifest_ignoring_fee(manifest, vec![virtual_signature_badge.clone()])
            .expect_commit_success()
            .new_resource_addresses()
            .get(0)
            .unwrap()
            .clone()
    };

    let component_address = {
        let manifest = ManifestBuilder::new()
            .mint_non_fungible(
                resource_address,
                btreemap!(
                    NonFungibleLocalId::integer(1) => EmptyStruct {},
                    NonFungibleLocalId::integer(2) => EmptyStruct {},
                ),
            )
            .take_all_from_worktop(resource_address, |builder, bucket| {
                builder.call_function(package_address, "VaultBurn", "new", manifest_args!(bucket))
            })
            .build();
        test_runner
            .execute_manifest_ignoring_fee(manifest, vec![virtual_signature_badge.clone()])
            .expect_commit_success()
            .new_component_addresses()
            .get(0)
            .unwrap()
            .clone()
    };
    let vault_id = get_vault_id(&mut test_runner, component_address);

    // Act
    let manifest = ManifestBuilder::new()
        .call_method(component_address, "burn_amount", manifest_args!(dec!("1")))
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(is_auth_unauthorized_error);
    assert_eq!(
        test_runner.inspect_non_fungible_vault(vault_id).unwrap().0,
        dec!("2")
    )
}

#[test]
fn cant_burn_by_ids_from_non_fungible_vault_with_an_access_rule_that_is_not_fulfilled() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/vault");
    let (public_key, _, _) = test_runner.new_account(false);
    let virtual_signature_badge = NonFungibleGlobalId::from_public_key(&public_key);
    let virtual_signature_rule = rule!(require(virtual_signature_badge.clone()));
    let resource_address = {
        let access_rules = btreemap!(
            Mint => (virtual_signature_rule.clone(), AccessRule::DenyAll),
            Burn => (virtual_signature_rule.clone(), AccessRule::DenyAll),
            Withdraw => (virtual_signature_rule.clone(), AccessRule::DenyAll),
            Deposit => (virtual_signature_rule.clone(), AccessRule::DenyAll),
            Recall => (virtual_signature_rule.clone(), AccessRule::DenyAll),
            UpdateNonFungibleData => (virtual_signature_rule.clone(), AccessRule::DenyAll),
        );
        let manifest = ManifestBuilder::new()
            .create_non_fungible_resource(
                OwnerRole::None,
                NonFungibleIdType::Integer,
                true,
                metadata!(),
                access_rules,
                Option::<BTreeMap<NonFungibleLocalId, EmptyStruct>>::None,
            )
            .build();
        test_runner
            .execute_manifest_ignoring_fee(manifest, vec![virtual_signature_badge.clone()])
            .expect_commit_success()
            .new_resource_addresses()
            .get(0)
            .unwrap()
            .clone()
    };

    let component_address = {
        let manifest = ManifestBuilder::new()
            .mint_non_fungible(
                resource_address,
                btreemap!(
                    NonFungibleLocalId::integer(1) => EmptyStruct {},
                    NonFungibleLocalId::integer(2) => EmptyStruct {},
                ),
            )
            .take_all_from_worktop(resource_address, |builder, bucket| {
                builder.call_function(package_address, "VaultBurn", "new", manifest_args!(bucket))
            })
            .build();
        test_runner
            .execute_manifest_ignoring_fee(manifest, vec![virtual_signature_badge.clone()])
            .expect_commit_success()
            .new_component_addresses()
            .get(0)
            .unwrap()
            .clone()
    };
    let vault_id = get_vault_id(&mut test_runner, component_address);

    // Act
    let manifest = ManifestBuilder::new()
        .call_method(
            component_address,
            "burn_ids",
            manifest_args!(btreeset![NonFungibleLocalId::integer(1)]),
        )
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(is_auth_unauthorized_error);
    assert_eq!(
        test_runner.inspect_non_fungible_vault(vault_id).unwrap().0,
        dec!("2")
    );
}

#[test]
fn can_burn_by_amount_from_fungible_vault_of_a_locked_down_resource() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/vault");
    let resource_address = {
        let access_rules = btreemap!(
            Mint => (AccessRule::AllowAll, AccessRule::DenyAll),
            Burn => (AccessRule::AllowAll, AccessRule::DenyAll),
            Withdraw => (AccessRule::DenyAll, AccessRule::DenyAll),
            Deposit => (AccessRule::AllowAll, AccessRule::DenyAll),
            Recall => (AccessRule::DenyAll, AccessRule::DenyAll),
            UpdateNonFungibleData => (AccessRule::DenyAll, AccessRule::DenyAll),
        );
        let manifest = ManifestBuilder::new()
            .create_fungible_resource(OwnerRole::None, true, 18, metadata!(), access_rules, None)
            .build();
        test_runner
            .execute_manifest_ignoring_fee(manifest, vec![])
            .expect_commit_success()
            .new_resource_addresses()
            .get(0)
            .unwrap()
            .clone()
    };

    let component_address = {
        let manifest = ManifestBuilder::new()
            .mint_fungible(resource_address, 100.into())
            .take_all_from_worktop(resource_address, |builder, bucket| {
                builder.call_function(package_address, "VaultBurn", "new", manifest_args!(bucket))
            })
            .build();
        test_runner
            .execute_manifest_ignoring_fee(manifest, vec![])
            .expect_commit_success()
            .new_component_addresses()
            .get(0)
            .unwrap()
            .clone()
    };
    let vault_id = get_vault_id(&mut test_runner, component_address);

    // Act
    let manifest = ManifestBuilder::new()
        .call_method(component_address, "burn_amount", manifest_args!(dec!("50")))
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
    assert_eq!(
        test_runner.inspect_fungible_vault(vault_id).unwrap(),
        dec!("50")
    )
}

#[test]
fn can_burn_by_amount_from_non_fungible_vault_of_a_locked_down_resource() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/vault");
    let resource_address = {
        let access_rules = btreemap!(
            Mint => (AccessRule::AllowAll, AccessRule::DenyAll),
            Burn => (AccessRule::AllowAll, AccessRule::DenyAll),
            Withdraw => (AccessRule::DenyAll, AccessRule::DenyAll),
            Deposit => (AccessRule::AllowAll, AccessRule::DenyAll),
            Recall => (AccessRule::DenyAll, AccessRule::DenyAll),
            UpdateNonFungibleData => (AccessRule::DenyAll, AccessRule::DenyAll),
        );
        let manifest = ManifestBuilder::new()
            .create_non_fungible_resource(
                OwnerRole::None,
                NonFungibleIdType::Integer,
                true,
                metadata!(),
                access_rules,
                Option::<BTreeMap<NonFungibleLocalId, EmptyStruct>>::None,
            )
            .build();
        test_runner
            .execute_manifest_ignoring_fee(manifest, vec![])
            .expect_commit_success()
            .new_resource_addresses()
            .get(0)
            .unwrap()
            .clone()
    };

    let component_address = {
        let manifest = ManifestBuilder::new()
            .mint_non_fungible(
                resource_address,
                btreemap!(
                    NonFungibleLocalId::integer(1) => EmptyStruct {},
                    NonFungibleLocalId::integer(2) => EmptyStruct {},
                ),
            )
            .take_all_from_worktop(resource_address, |builder, bucket| {
                builder.call_function(package_address, "VaultBurn", "new", manifest_args!(bucket))
            })
            .build();
        test_runner
            .execute_manifest_ignoring_fee(manifest, vec![])
            .expect_commit_success()
            .new_component_addresses()
            .get(0)
            .unwrap()
            .clone()
    };
    let vault_id = get_vault_id(&mut test_runner, component_address);

    // Act
    let manifest = ManifestBuilder::new()
        .call_method(component_address, "burn_amount", manifest_args!(dec!("1")))
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
    assert_eq!(
        test_runner.inspect_non_fungible_vault(vault_id).unwrap().0,
        dec!("1")
    )
}

#[test]
fn can_burn_by_ids_from_non_fungible_vault_of_a_locked_down_resource() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/vault");
    let resource_address = {
        let access_rules = btreemap!(
            Mint => (AccessRule::AllowAll, AccessRule::DenyAll),
            Burn => (AccessRule::AllowAll, AccessRule::DenyAll),
            Withdraw => (AccessRule::DenyAll, AccessRule::DenyAll),
            Deposit => (AccessRule::AllowAll, AccessRule::DenyAll),
            Recall => (AccessRule::DenyAll, AccessRule::DenyAll),
            UpdateNonFungibleData => (AccessRule::DenyAll, AccessRule::DenyAll),
        );
        let manifest = ManifestBuilder::new()
            .create_non_fungible_resource(
                OwnerRole::None,
                NonFungibleIdType::Integer,
                true,
                metadata!(),
                access_rules,
                Option::<BTreeMap<NonFungibleLocalId, EmptyStruct>>::None,
            )
            .build();
        test_runner
            .execute_manifest_ignoring_fee(manifest, vec![])
            .expect_commit_success()
            .new_resource_addresses()
            .get(0)
            .unwrap()
            .clone()
    };

    let component_address = {
        let manifest = ManifestBuilder::new()
            .mint_non_fungible(
                resource_address,
                btreemap!(
                    NonFungibleLocalId::integer(1) => EmptyStruct {},
                    NonFungibleLocalId::integer(2) => EmptyStruct {},
                ),
            )
            .take_all_from_worktop(resource_address, |builder, bucket| {
                builder.call_function(package_address, "VaultBurn", "new", manifest_args!(bucket))
            })
            .build();
        test_runner
            .execute_manifest_ignoring_fee(manifest, vec![])
            .expect_commit_success()
            .new_component_addresses()
            .get(0)
            .unwrap()
            .clone()
    };
    let vault_id = get_vault_id(&mut test_runner, component_address);

    // Act
    let manifest = ManifestBuilder::new()
        .call_method(
            component_address,
            "burn_ids",
            manifest_args!(btreeset![NonFungibleLocalId::integer(1)]),
        )
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
    assert_eq!(
        test_runner.inspect_non_fungible_vault(vault_id).unwrap().0,
        dec!("1")
    );
}

#[test]
fn can_burn_by_amount_from_fungible_account_vault() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_account(false);
    let virtual_signature_badge = NonFungibleGlobalId::from_public_key(&public_key);
    let virtual_signature_rule = rule!(require(virtual_signature_badge.clone()));
    let resource_address = {
        let access_rules = btreemap!(
            Mint => (virtual_signature_rule.clone(), AccessRule::DenyAll),
            Burn => (virtual_signature_rule.clone(), AccessRule::DenyAll),
            Withdraw => (virtual_signature_rule.clone(), AccessRule::DenyAll),
            Deposit => (virtual_signature_rule.clone(), AccessRule::DenyAll),
            Recall => (virtual_signature_rule.clone(), AccessRule::DenyAll),
            UpdateNonFungibleData => (virtual_signature_rule.clone(), AccessRule::DenyAll),
        );
        let manifest = ManifestBuilder::new()
            .create_fungible_resource(
                OwnerRole::None,
                true,
                18,
                metadata!(),
                access_rules,
                Some(100.into()),
            )
            .try_deposit_batch_or_abort(account)
            .build();
        test_runner
            .execute_manifest_ignoring_fee(manifest, vec![virtual_signature_badge.clone()])
            .expect_commit_success()
            .new_resource_addresses()
            .get(0)
            .unwrap()
            .clone()
    };

    // Act
    let manifest = ManifestBuilder::new()
        .call_method(
            account,
            "burn",
            manifest_args!(resource_address, dec!("50")),
        )
        .build();
    let receipt =
        test_runner.execute_manifest_ignoring_fee(manifest, vec![virtual_signature_badge.clone()]);

    // Assert
    receipt.expect_commit_success();
    assert_eq!(
        test_runner
            .account_balance(account, resource_address)
            .unwrap(),
        dec!("50")
    )
}

#[test]
fn can_burn_by_amount_from_non_fungible_account_vault() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_account(false);
    let virtual_signature_badge = NonFungibleGlobalId::from_public_key(&public_key);
    let virtual_signature_rule = rule!(require(virtual_signature_badge.clone()));
    let resource_address = {
        let access_rules = btreemap!(
            Mint => (virtual_signature_rule.clone(), AccessRule::DenyAll),
            Burn => (virtual_signature_rule.clone(), AccessRule::DenyAll),
            Withdraw => (virtual_signature_rule.clone(), AccessRule::DenyAll),
            Deposit => (virtual_signature_rule.clone(), AccessRule::DenyAll),
            Recall => (virtual_signature_rule.clone(), AccessRule::DenyAll),
            UpdateNonFungibleData => (virtual_signature_rule.clone(), AccessRule::DenyAll),
        );
        let manifest = ManifestBuilder::new()
            .create_non_fungible_resource(
                OwnerRole::None,
                NonFungibleIdType::Integer,
                true,
                metadata!(),
                access_rules,
                Some(btreemap!(
                    NonFungibleLocalId::integer(1) => EmptyStruct {},
                    NonFungibleLocalId::integer(2) => EmptyStruct {},
                )),
            )
            .try_deposit_batch_or_abort(account)
            .build();
        test_runner
            .execute_manifest_ignoring_fee(manifest, vec![virtual_signature_badge.clone()])
            .expect_commit_success()
            .new_resource_addresses()
            .get(0)
            .unwrap()
            .clone()
    };

    // Act
    let manifest = ManifestBuilder::new()
        .call_method(account, "burn", manifest_args!(resource_address, dec!("1")))
        .build();
    let receipt =
        test_runner.execute_manifest_ignoring_fee(manifest, vec![virtual_signature_badge.clone()]);

    // Assert
    receipt.expect_commit_success();
    assert_eq!(
        test_runner
            .account_balance(account, resource_address)
            .unwrap(),
        dec!("1")
    )
}

#[test]
fn can_burn_by_ids_from_non_fungible_account_vault() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_account(false);
    let virtual_signature_badge = NonFungibleGlobalId::from_public_key(&public_key);
    let virtual_signature_rule = rule!(require(virtual_signature_badge.clone()));
    let resource_address = {
        let access_rules = btreemap!(
            Mint => (virtual_signature_rule.clone(), AccessRule::DenyAll),
            Burn => (virtual_signature_rule.clone(), AccessRule::DenyAll),
            Withdraw => (virtual_signature_rule.clone(), AccessRule::DenyAll),
            Deposit => (virtual_signature_rule.clone(), AccessRule::DenyAll),
            Recall => (virtual_signature_rule.clone(), AccessRule::DenyAll),
            UpdateNonFungibleData => (virtual_signature_rule.clone(), AccessRule::DenyAll),
        );
        let manifest = ManifestBuilder::new()
            .create_non_fungible_resource(
                OwnerRole::None,
                NonFungibleIdType::Integer,
                true,
                metadata!(),
                access_rules,
                Some(btreemap!(
                    NonFungibleLocalId::integer(1) => EmptyStruct {},
                    NonFungibleLocalId::integer(2) => EmptyStruct {},
                )),
            )
            .try_deposit_batch_or_abort(account)
            .build();
        test_runner
            .execute_manifest_ignoring_fee(manifest, vec![virtual_signature_badge.clone()])
            .expect_commit_success()
            .new_resource_addresses()
            .get(0)
            .unwrap()
            .clone()
    };

    // Act
    let manifest = ManifestBuilder::new()
        .call_method(
            account,
            "burn_non_fungibles",
            manifest_args!(resource_address, btreeset!(NonFungibleLocalId::integer(1))),
        )
        .build();
    let receipt =
        test_runner.execute_manifest_ignoring_fee(manifest, vec![virtual_signature_badge.clone()]);

    // Assert
    receipt.expect_commit_success();
    assert_eq!(
        test_runner
            .account_balance(account, resource_address)
            .unwrap(),
        dec!("1")
    )
}

fn get_vault_id(test_runner: &mut TestRunner, component_address: ComponentAddress) -> NodeId {
    let manifest = ManifestBuilder::new()
        .call_method(component_address, "vault_id", manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![]);
    receipt.expect_commit_success().output(1)
}

#[derive(NonFungibleData, ScryptoSbor, ManifestSbor)]
struct EmptyStruct {}

fn is_auth_unauthorized_error(error: &RuntimeError) -> bool {
    matches!(
        error,
        RuntimeError::SystemModuleError(SystemModuleError::AuthError(
            AuthError::Unauthorized { .. }
        ))
    )
}
