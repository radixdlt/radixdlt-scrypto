use radix_common::prelude::*;
use radix_engine::errors::{RuntimeError, SystemModuleError};
use radix_engine::system::system_modules::auth::AuthError;
use radix_engine_interface::object_modules::ModuleConfig;
use radix_engine_interface::{metadata, metadata_init};
use radix_engine_tests::common::*;
use scrypto::NonFungibleData;
use scrypto_test::prelude::*;

#[test]
fn package_burn_is_only_callable_within_resource_package() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let resource_address = {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_fungible_resource(
                OwnerRole::None,
                true,
                18,
                FungibleResourceRoles::single_locked_rule(rule!(allow_all)),
                metadata!(),
                None,
            )
            .build();
        ledger
            .execute_manifest(manifest, vec![])
            .expect_commit_success()
            .new_resource_addresses()[0]
    };

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .mint_fungible(resource_address, 10)
        .take_all_from_worktop(resource_address, "bucket")
        .with_name_lookup(|builder, lookup| {
            builder.call_method(
                resource_address,
                RESOURCE_MANAGER_PACKAGE_BURN_IDENT,
                manifest_args!(lookup.bucket("bucket")),
            )
        })
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(is_auth_unauthorized_error);
}

#[test]
fn can_burn_by_amount_from_fungible_vault() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("vault"));
    let resource_address = {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_fungible_resource(
                OwnerRole::None,
                true,
                18,
                FungibleResourceRoles::single_locked_rule(rule!(allow_all)),
                metadata!(),
                None,
            )
            .build();
        ledger
            .execute_manifest(manifest, vec![])
            .expect_commit_success()
            .new_resource_addresses()[0]
    };

    let component_address = {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .mint_fungible(resource_address, 100)
            .take_all_from_worktop(resource_address, "to_burn")
            .with_name_lookup(|builder, lookup| {
                builder.call_function(
                    package_address,
                    "VaultBurn",
                    "new",
                    manifest_args!(lookup.bucket("to_burn")),
                )
            })
            .build();
        ledger
            .execute_manifest(manifest, vec![])
            .expect_commit_success()
            .new_component_addresses()[0]
    };
    let vault_id = get_vault_id(&mut ledger, component_address);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(component_address, "burn_amount", manifest_args!(dec!("50")))
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
    assert_eq!(ledger.inspect_fungible_vault(vault_id).unwrap(), dec!("50"))
}

#[test]
fn can_burn_by_amount_from_non_fungible_vault() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("vault"));
    let resource_address = {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_non_fungible_resource(
                OwnerRole::None,
                NonFungibleIdType::Integer,
                true,
                NonFungibleResourceRoles::single_locked_rule(rule!(allow_all)),
                metadata!(),
                Option::<BTreeMap<NonFungibleLocalId, EmptyStruct>>::None,
            )
            .build();
        ledger
            .execute_manifest(manifest, vec![])
            .expect_commit_success()
            .new_resource_addresses()[0]
    };

    let component_address = {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .mint_non_fungible(
                resource_address,
                btreemap!(
                    NonFungibleLocalId::integer(1) => EmptyStruct {},
                    NonFungibleLocalId::integer(2) => EmptyStruct {},
                ),
            )
            .take_all_from_worktop(resource_address, "to_burn")
            .with_name_lookup(|builder, lookup| {
                builder.call_function(
                    package_address,
                    "VaultBurn",
                    "new",
                    manifest_args!(lookup.bucket("to_burn")),
                )
            })
            .build();
        ledger
            .execute_manifest(manifest, vec![])
            .expect_commit_success()
            .new_component_addresses()[0]
    };
    let vault_id = get_vault_id(&mut ledger, component_address);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(component_address, "burn_amount", manifest_args!(dec!(1)))
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
    let (amount, _) = ledger.inspect_non_fungible_vault(vault_id).unwrap();
    assert_eq!(amount, dec!(1))
}

#[test]
fn can_burn_by_ids_from_non_fungible_vault() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("vault"));
    let resource_address = {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_non_fungible_resource(
                OwnerRole::None,
                NonFungibleIdType::Integer,
                true,
                NonFungibleResourceRoles::single_locked_rule(rule!(allow_all)),
                metadata!(),
                Option::<BTreeMap<NonFungibleLocalId, EmptyStruct>>::None,
            )
            .build();
        ledger
            .execute_manifest(manifest, vec![])
            .expect_commit_success()
            .new_resource_addresses()[0]
    };

    let component_address = {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .mint_non_fungible(
                resource_address,
                btreemap!(
                    NonFungibleLocalId::integer(1) => EmptyStruct {},
                    NonFungibleLocalId::integer(2) => EmptyStruct {},
                ),
            )
            .take_all_from_worktop(resource_address, "bucket")
            .with_name_lookup(|builder, lookup| {
                builder.call_function(
                    package_address,
                    "VaultBurn",
                    "new",
                    manifest_args!(lookup.bucket("bucket")),
                )
            })
            .build();
        ledger
            .execute_manifest(manifest, vec![])
            .expect_commit_success()
            .new_component_addresses()[0]
    };
    let vault_id = get_vault_id(&mut ledger, component_address);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            component_address,
            "burn_ids",
            manifest_args!(btreeset![NonFungibleLocalId::integer(1)]),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
    assert_eq!(
        ledger.inspect_non_fungible_vault(vault_id).unwrap().0,
        dec!(1)
    );
}

#[test]
fn can_burn_by_amount_from_fungible_vault_with_an_access_rule() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("vault"));
    let (public_key, _, _) = ledger.new_account(false);
    let virtual_signature_badge = NonFungibleGlobalId::from_public_key(&public_key);
    let virtual_signature_rule = rule!(require(virtual_signature_badge.clone()));
    let resource_address = {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_fungible_resource(
                OwnerRole::None,
                true,
                18,
                FungibleResourceRoles::single_locked_rule(virtual_signature_rule),
                metadata!(),
                None,
            )
            .build();
        ledger
            .execute_manifest(manifest, vec![virtual_signature_badge.clone()])
            .expect_commit_success()
            .new_resource_addresses()[0]
    };

    let component_address = {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .mint_fungible(resource_address, 100)
            .take_all_from_worktop(resource_address, "bucket")
            .with_name_lookup(|builder, lookup| {
                builder.call_function(
                    package_address,
                    "VaultBurn",
                    "new",
                    manifest_args!(lookup.bucket("bucket")),
                )
            })
            .build();
        ledger
            .execute_manifest(manifest, vec![virtual_signature_badge.clone()])
            .expect_commit_success()
            .new_component_addresses()[0]
    };
    let vault_id = get_vault_id(&mut ledger, component_address);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(component_address, "burn_amount", manifest_args!(dec!("50")))
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![virtual_signature_badge]);

    // Assert
    receipt.expect_commit_success();
    assert_eq!(ledger.inspect_fungible_vault(vault_id).unwrap(), dec!("50"))
}

#[test]
fn can_burn_by_amount_from_non_fungible_vault_with_an_access_rule() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("vault"));
    let (public_key, _, _) = ledger.new_account(false);
    let virtual_signature_badge = NonFungibleGlobalId::from_public_key(&public_key);
    let virtual_signature_rule = rule!(require(virtual_signature_badge.clone()));
    let resource_address = {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_non_fungible_resource(
                OwnerRole::None,
                NonFungibleIdType::Integer,
                true,
                NonFungibleResourceRoles::single_locked_rule(virtual_signature_rule),
                metadata!(),
                Option::<BTreeMap<NonFungibleLocalId, EmptyStruct>>::None,
            )
            .build();
        ledger
            .execute_manifest(manifest, vec![virtual_signature_badge.clone()])
            .expect_commit_success()
            .new_resource_addresses()[0]
    };

    let component_address = {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .mint_non_fungible(
                resource_address,
                btreemap!(
                    NonFungibleLocalId::integer(1) => EmptyStruct {},
                    NonFungibleLocalId::integer(2) => EmptyStruct {},
                ),
            )
            .take_all_from_worktop(resource_address, "bucket")
            .with_name_lookup(|builder, lookup| {
                builder.call_function(
                    package_address,
                    "VaultBurn",
                    "new",
                    manifest_args!(lookup.bucket("bucket")),
                )
            })
            .build();
        ledger
            .execute_manifest(manifest, vec![virtual_signature_badge.clone()])
            .expect_commit_success()
            .new_component_addresses()[0]
    };
    let vault_id = get_vault_id(&mut ledger, component_address);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(component_address, "burn_amount", manifest_args!(dec!(1)))
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![virtual_signature_badge]);

    // Assert
    receipt.expect_commit_success();
    let (amount, _) = ledger.inspect_non_fungible_vault(vault_id).unwrap();
    assert_eq!(amount, dec!(1))
}

#[test]
fn can_burn_by_ids_from_non_fungible_vault_with_an_access_rule() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("vault"));
    let (public_key, _, _) = ledger.new_account(false);
    let virtual_signature_badge = NonFungibleGlobalId::from_public_key(&public_key);
    let virtual_signature_rule = rule!(require(virtual_signature_badge.clone()));
    let resource_address = {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_non_fungible_resource(
                OwnerRole::None,
                NonFungibleIdType::Integer,
                true,
                NonFungibleResourceRoles::single_locked_rule(virtual_signature_rule),
                metadata!(),
                Option::<BTreeMap<NonFungibleLocalId, EmptyStruct>>::None,
            )
            .build();
        ledger
            .execute_manifest(manifest, vec![virtual_signature_badge.clone()])
            .expect_commit_success()
            .new_resource_addresses()[0]
    };

    let component_address = {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .mint_non_fungible(
                resource_address,
                btreemap!(
                    NonFungibleLocalId::integer(1) => EmptyStruct {},
                    NonFungibleLocalId::integer(2) => EmptyStruct {},
                ),
            )
            .take_all_from_worktop(resource_address, "bucket")
            .with_name_lookup(|builder, lookup| {
                builder.call_function(
                    package_address,
                    "VaultBurn",
                    "new",
                    manifest_args!(lookup.bucket("bucket")),
                )
            })
            .build();
        ledger
            .execute_manifest(manifest, vec![virtual_signature_badge.clone()])
            .expect_commit_success()
            .new_component_addresses()[0]
    };
    let vault_id = get_vault_id(&mut ledger, component_address);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            component_address,
            "burn_ids",
            manifest_args!(btreeset![NonFungibleLocalId::integer(1)]),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![virtual_signature_badge]);

    // Assert
    receipt.expect_commit_success();
    assert_eq!(
        ledger.inspect_non_fungible_vault(vault_id).unwrap().0,
        dec!(1)
    );
}

#[test]
fn cant_burn_by_amount_from_fungible_vault_with_an_access_rule_that_is_not_fulfilled() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("vault"));
    let (public_key, _, _) = ledger.new_account(false);
    let virtual_signature_badge = NonFungibleGlobalId::from_public_key(&public_key);
    let virtual_signature_rule = rule!(require(virtual_signature_badge.clone()));
    let resource_address = {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_fungible_resource(
                OwnerRole::None,
                true,
                18,
                FungibleResourceRoles::single_locked_rule(virtual_signature_rule),
                metadata!(),
                None,
            )
            .build();
        ledger
            .execute_manifest(manifest, vec![virtual_signature_badge.clone()])
            .expect_commit_success()
            .new_resource_addresses()[0]
    };

    let component_address = {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .mint_fungible(resource_address, 100)
            .take_all_from_worktop(resource_address, "bucket")
            .with_name_lookup(|builder, lookup| {
                builder.call_function(
                    package_address,
                    "VaultBurn",
                    "new",
                    manifest_args!(lookup.bucket("bucket")),
                )
            })
            .build();
        ledger
            .execute_manifest(manifest, vec![virtual_signature_badge])
            .expect_commit_success()
            .new_component_addresses()[0]
    };
    let vault_id = get_vault_id(&mut ledger, component_address);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(component_address, "burn_amount", manifest_args!(dec!("50")))
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(is_auth_unauthorized_error);
    assert_eq!(
        ledger.inspect_fungible_vault(vault_id).unwrap(),
        dec!("100")
    )
}

#[test]
fn cant_burn_by_amount_from_non_fungible_vault_with_an_access_rule_that_is_not_fulfilled() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("vault"));
    let (public_key, _, _) = ledger.new_account(false);
    let virtual_signature_badge = NonFungibleGlobalId::from_public_key(&public_key);
    let virtual_signature_rule = rule!(require(virtual_signature_badge.clone()));
    let resource_address = {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_non_fungible_resource(
                OwnerRole::None,
                NonFungibleIdType::Integer,
                true,
                NonFungibleResourceRoles::single_locked_rule(virtual_signature_rule),
                metadata!(),
                Option::<BTreeMap<NonFungibleLocalId, EmptyStruct>>::None,
            )
            .build();
        ledger
            .execute_manifest(manifest, vec![virtual_signature_badge.clone()])
            .expect_commit_success()
            .new_resource_addresses()[0]
    };

    let component_address = {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .mint_non_fungible(
                resource_address,
                btreemap!(
                    NonFungibleLocalId::integer(1) => EmptyStruct {},
                    NonFungibleLocalId::integer(2) => EmptyStruct {},
                ),
            )
            .take_all_from_worktop(resource_address, "bucket")
            .with_name_lookup(|builder, lookup| {
                builder.call_function(
                    package_address,
                    "VaultBurn",
                    "new",
                    manifest_args!(lookup.bucket("bucket")),
                )
            })
            .build();
        ledger
            .execute_manifest(manifest, vec![virtual_signature_badge])
            .expect_commit_success()
            .new_component_addresses()[0]
    };
    let vault_id = get_vault_id(&mut ledger, component_address);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(component_address, "burn_amount", manifest_args!(dec!(1)))
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(is_auth_unauthorized_error);
    let (amount, _) = ledger.inspect_non_fungible_vault(vault_id).unwrap();
    assert_eq!(amount, dec!("2"))
}

#[test]
fn cant_burn_by_ids_from_non_fungible_vault_with_an_access_rule_that_is_not_fulfilled() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("vault"));
    let (public_key, _, _) = ledger.new_account(false);
    let virtual_signature_badge = NonFungibleGlobalId::from_public_key(&public_key);
    let virtual_signature_rule = rule!(require(virtual_signature_badge.clone()));
    let resource_address = {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_non_fungible_resource(
                OwnerRole::None,
                NonFungibleIdType::Integer,
                true,
                NonFungibleResourceRoles::single_locked_rule(virtual_signature_rule),
                metadata!(),
                Option::<BTreeMap<NonFungibleLocalId, EmptyStruct>>::None,
            )
            .build();
        ledger
            .execute_manifest(manifest, vec![virtual_signature_badge.clone()])
            .expect_commit_success()
            .new_resource_addresses()[0]
    };

    let component_address = {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .mint_non_fungible(
                resource_address,
                btreemap!(
                    NonFungibleLocalId::integer(1) => EmptyStruct {},
                    NonFungibleLocalId::integer(2) => EmptyStruct {},
                ),
            )
            .take_all_from_worktop(resource_address, "bucket")
            .with_name_lookup(|builder, lookup| {
                builder.call_function(
                    package_address,
                    "VaultBurn",
                    "new",
                    manifest_args!(lookup.bucket("bucket")),
                )
            })
            .build();
        ledger
            .execute_manifest(manifest, vec![virtual_signature_badge])
            .expect_commit_success()
            .new_component_addresses()[0]
    };
    let vault_id = get_vault_id(&mut ledger, component_address);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            component_address,
            "burn_ids",
            manifest_args!(btreeset![NonFungibleLocalId::integer(1)]),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(is_auth_unauthorized_error);
    assert_eq!(
        ledger.inspect_non_fungible_vault(vault_id).unwrap().0,
        dec!("2")
    );
}

#[test]
fn can_burn_by_amount_from_fungible_vault_of_a_locked_down_resource() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("vault"));
    let resource_address = {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_fungible_resource(
                OwnerRole::None,
                true,
                18,
                FungibleResourceRoles::single_locked_rule(rule!(allow_all)),
                metadata!(),
                None,
            )
            .build();
        ledger
            .execute_manifest(manifest, vec![])
            .expect_commit_success()
            .new_resource_addresses()[0]
    };

    let component_address = {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .mint_fungible(resource_address, 100)
            .take_all_from_worktop(resource_address, "bucket")
            .with_name_lookup(|builder, lookup| {
                builder.call_function(
                    package_address,
                    "VaultBurn",
                    "new",
                    manifest_args!(lookup.bucket("bucket")),
                )
            })
            .build();
        ledger
            .execute_manifest(manifest, vec![])
            .expect_commit_success()
            .new_component_addresses()[0]
    };
    let vault_id = get_vault_id(&mut ledger, component_address);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(component_address, "burn_amount", manifest_args!(dec!("50")))
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
    assert_eq!(ledger.inspect_fungible_vault(vault_id).unwrap(), dec!("50"))
}

#[test]
fn can_burn_by_amount_from_non_fungible_vault_of_a_locked_down_resource() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("vault"));
    let resource_address = {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_non_fungible_resource(
                OwnerRole::None,
                NonFungibleIdType::Integer,
                true,
                NonFungibleResourceRoles::single_locked_rule(rule!(allow_all)),
                metadata!(),
                Option::<BTreeMap<NonFungibleLocalId, EmptyStruct>>::None,
            )
            .build();
        ledger
            .execute_manifest(manifest, vec![])
            .expect_commit_success()
            .new_resource_addresses()[0]
    };

    let component_address = {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .mint_non_fungible(
                resource_address,
                btreemap!(
                    NonFungibleLocalId::integer(1) => EmptyStruct {},
                    NonFungibleLocalId::integer(2) => EmptyStruct {},
                ),
            )
            .take_all_from_worktop(resource_address, "bucket")
            .with_name_lookup(|builder, lookup| {
                builder.call_function(
                    package_address,
                    "VaultBurn",
                    "new",
                    manifest_args!(lookup.bucket("bucket")),
                )
            })
            .build();
        ledger
            .execute_manifest(manifest, vec![])
            .expect_commit_success()
            .new_component_addresses()[0]
    };
    let vault_id = get_vault_id(&mut ledger, component_address);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(component_address, "burn_amount", manifest_args!(dec!(1)))
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
    let (amount, _) = ledger.inspect_non_fungible_vault(vault_id).unwrap();
    assert_eq!(amount, dec!(1))
}

#[test]
fn can_burn_by_ids_from_non_fungible_vault_of_a_locked_down_resource() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("vault"));
    let resource_address = {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_non_fungible_resource(
                OwnerRole::None,
                NonFungibleIdType::Integer,
                true,
                NonFungibleResourceRoles::single_locked_rule(rule!(allow_all)),
                metadata!(),
                Option::<BTreeMap<NonFungibleLocalId, EmptyStruct>>::None,
            )
            .build();
        ledger
            .execute_manifest(manifest, vec![])
            .expect_commit_success()
            .new_resource_addresses()[0]
    };

    let component_address = {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .mint_non_fungible(
                resource_address,
                btreemap!(
                    NonFungibleLocalId::integer(1) => EmptyStruct {},
                    NonFungibleLocalId::integer(2) => EmptyStruct {},
                ),
            )
            .take_all_from_worktop(resource_address, "bucket")
            .with_name_lookup(|builder, lookup| {
                builder.call_function(
                    package_address,
                    "VaultBurn",
                    "new",
                    manifest_args!(lookup.bucket("bucket")),
                )
            })
            .build();
        ledger
            .execute_manifest(manifest, vec![])
            .expect_commit_success()
            .new_component_addresses()[0]
    };
    let vault_id = get_vault_id(&mut ledger, component_address);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            component_address,
            "burn_ids",
            manifest_args!(btreeset![NonFungibleLocalId::integer(1)]),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
    assert_eq!(
        ledger.inspect_non_fungible_vault(vault_id).unwrap().0,
        dec!(1)
    );
}

#[test]
fn can_burn_by_amount_from_fungible_account_vault() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_account(false);
    let virtual_signature_badge = NonFungibleGlobalId::from_public_key(&public_key);
    let virtual_signature_rule = rule!(require(virtual_signature_badge.clone()));
    let resource_address = {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_fungible_resource(
                OwnerRole::None,
                true,
                18,
                FungibleResourceRoles::single_locked_rule(virtual_signature_rule),
                metadata!(),
                Some(100.into()),
            )
            .try_deposit_entire_worktop_or_abort(account, None)
            .build();
        ledger
            .execute_manifest(manifest, vec![virtual_signature_badge.clone()])
            .expect_commit_success()
            .new_resource_addresses()[0]
    };

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            account,
            "burn",
            manifest_args!(resource_address, dec!("50")),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![virtual_signature_badge]);

    // Assert
    receipt.expect_commit_success();
    assert_eq!(
        ledger.get_component_balance(account, resource_address),
        dec!("50")
    )
}

#[test]
fn can_burn_by_amount_from_non_fungible_account_vault() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_account(false);
    let virtual_signature_badge = NonFungibleGlobalId::from_public_key(&public_key);
    let virtual_signature_rule = rule!(require(virtual_signature_badge.clone()));
    let resource_address = {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_non_fungible_resource(
                OwnerRole::None,
                NonFungibleIdType::Integer,
                true,
                NonFungibleResourceRoles::single_locked_rule(virtual_signature_rule),
                metadata!(),
                Some(btreemap!(
                    NonFungibleLocalId::integer(1) => EmptyStruct {},
                    NonFungibleLocalId::integer(2) => EmptyStruct {},
                )),
            )
            .try_deposit_entire_worktop_or_abort(account, None)
            .build();
        ledger
            .execute_manifest(manifest, vec![virtual_signature_badge.clone()])
            .expect_commit_success()
            .new_resource_addresses()[0]
    };

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(account, "burn", manifest_args!(resource_address, dec!(1)))
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![virtual_signature_badge]);

    // Assert
    receipt.expect_commit_success();
    assert_eq!(
        ledger.get_component_balance(account, resource_address),
        dec!(1)
    )
}

#[test]
fn can_burn_by_ids_from_non_fungible_account_vault() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_account(false);
    let virtual_signature_badge = NonFungibleGlobalId::from_public_key(&public_key);
    let virtual_signature_rule = rule!(require(virtual_signature_badge.clone()));
    let resource_address = {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_non_fungible_resource(
                OwnerRole::None,
                NonFungibleIdType::Integer,
                true,
                NonFungibleResourceRoles::single_locked_rule(virtual_signature_rule),
                metadata!(),
                Some(btreemap!(
                    NonFungibleLocalId::integer(1) => EmptyStruct {},
                    NonFungibleLocalId::integer(2) => EmptyStruct {},
                )),
            )
            .try_deposit_entire_worktop_or_abort(account, None)
            .build();
        ledger
            .execute_manifest(manifest, vec![virtual_signature_badge.clone()])
            .expect_commit_success()
            .new_resource_addresses()[0]
    };

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            account,
            "burn_non_fungibles",
            manifest_args!(resource_address, indexset!(NonFungibleLocalId::integer(1))),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![virtual_signature_badge]);

    // Assert
    receipt.expect_commit_success();
    assert_eq!(
        ledger.get_component_balance(account, resource_address),
        dec!(1)
    )
}

fn get_vault_id(
    ledger: &mut DefaultLedgerSimulator,
    component_address: ComponentAddress,
) -> NodeId {
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(component_address, "vault_id", manifest_args!())
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
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
