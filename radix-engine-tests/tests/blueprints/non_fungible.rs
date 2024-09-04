use radix_common::prelude::*;
use radix_engine::blueprints::resource::{
    InvalidNonFungibleSchema, NonFungibleResourceManagerError,
};
use radix_engine::errors::{ApplicationError, RuntimeError, SystemError};
use radix_engine::system::system_type_checker::TypeCheckError;
use radix_engine::transaction::TransactionReceipt;
use radix_engine_interface::blueprints::transaction_processor::InstructionOutput;
use radix_engine_interface::object_modules::ModuleConfig;
use radix_engine_interface::types::FromPublicKey;
use radix_engine_tests::common::*;
use scrypto::NonFungibleData;
use scrypto_test::prelude::*;

#[test]
fn create_non_fungible_resource_with_supply_and_ruid_should_fail() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_non_fungible_resource(
            OwnerRole::None,
            NonFungibleIdType::RUID,
            false,
            NonFungibleResourceRoles::default(),
            ModuleConfig::default(),
            Some(vec![(NonFungibleLocalId::ruid([0u8; 32]), ())]),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::NonFungibleResourceManagerError(
                NonFungibleResourceManagerError::NonFungibleLocalIdProvidedForRUIDType
            ))
        )
    });
}

fn test_non_fungible_resource_with_schema<F: FnOnce(TransactionReceipt)>(
    non_fungible_schema: NonFungibleDataSchema,
    on_receipt: F,
) {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            RESOURCE_PACKAGE,
            NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
            NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT,
            NonFungibleResourceManagerCreateManifestInput {
                owner_role: OwnerRole::None,
                id_type: NonFungibleIdType::Integer,
                track_total_supply: true,
                non_fungible_schema,
                resource_roles: NonFungibleResourceRoles::default(),
                metadata: ModuleConfig::default(),
                address_reservation: None,
            },
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    on_receipt(receipt);
}

#[test]
fn create_non_fungible_resource_with_invalid_type_id_should_fail() {
    let aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();
    let schema = generate_full_schema(aggregator);
    let non_fungible_schema = NonFungibleDataSchema::Local(LocalNonFungibleDataSchema {
        schema,
        type_id: LocalTypeId::SchemaLocalIndex(64), // Invalid LocalTypeId
        mutable_fields: indexset!(),
    });

    test_non_fungible_resource_with_schema(non_fungible_schema, |receipt| {
        receipt.expect_specific_failure(|e| {
            matches!(
                e,
                RuntimeError::ApplicationError(ApplicationError::NonFungibleResourceManagerError(
                    NonFungibleResourceManagerError::InvalidNonFungibleSchema(
                        InvalidNonFungibleSchema::InvalidLocalTypeId
                    )
                ))
            )
        });
    })
}

#[test]
fn create_non_fungible_resource_with_non_tuple_type_id_should_fail() {
    let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();
    let type_id = aggregator.add_child_type_and_descendents::<String>();
    let schema = generate_full_schema(aggregator);
    let non_fungible_schema = NonFungibleDataSchema::Local(LocalNonFungibleDataSchema {
        schema,
        type_id,
        mutable_fields: indexset!(),
    });

    test_non_fungible_resource_with_schema(non_fungible_schema, |receipt| {
        receipt.expect_specific_failure(|e| {
            matches!(
                e,
                RuntimeError::ApplicationError(ApplicationError::NonFungibleResourceManagerError(
                    NonFungibleResourceManagerError::InvalidNonFungibleSchema(
                        InvalidNonFungibleSchema::NotATuple
                    )
                ))
            )
        });
    })
}

#[test]
fn create_non_fungible_resource_with_missing_mutable_field_should_fail2() {
    let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();
    let type_id = aggregator.add_child_type_and_descendents::<Sandwich>();
    let schema = generate_full_schema(aggregator);
    let non_fungible_schema = NonFungibleDataSchema::Local(LocalNonFungibleDataSchema {
        schema,
        type_id,
        mutable_fields: indexset!("missing".to_string()),
    });

    test_non_fungible_resource_with_schema(non_fungible_schema, |receipt| {
        receipt.expect_specific_failure(|e| {
            matches!(
                e,
                RuntimeError::ApplicationError(ApplicationError::NonFungibleResourceManagerError(
                    NonFungibleResourceManagerError::InvalidNonFungibleSchema(
                        InvalidNonFungibleSchema::MutableFieldDoesNotExist(..)
                    )
                ))
            )
        });
    })
}

#[test]
fn create_non_fungible_resource_with_missing_mutable_field_should_fail() {
    let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();
    let type_id = aggregator.add_child_type_and_descendents::<()>();
    let schema = generate_full_schema(aggregator);
    let non_fungible_schema = NonFungibleDataSchema::Local(LocalNonFungibleDataSchema {
        schema,
        type_id,
        mutable_fields: indexset!("missing".to_string()),
    });

    test_non_fungible_resource_with_schema(non_fungible_schema, |receipt| {
        receipt.expect_specific_failure(|e| {
            matches!(
                e,
                RuntimeError::ApplicationError(ApplicationError::NonFungibleResourceManagerError(
                    NonFungibleResourceManagerError::InvalidNonFungibleSchema(
                        InvalidNonFungibleSchema::MissingFieldNames
                    )
                ))
            )
        });
    })
}

#[test]
fn can_mint_non_fungible_with_global() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let package = ledger.publish_package_simple(PackageLoader::get("non_fungible"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package,
            "NonFungibleWithGlobalTest",
            "create_non_fungible_with_global",
            manifest_args!(),
        )
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn create_non_fungible_mutable() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let package = ledger.publish_package_simple(PackageLoader::get("non_fungible"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package,
            "NonFungibleTest",
            "create_non_fungible_mutable",
            manifest_args!(),
        )
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn can_burn_non_fungible() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let package = ledger.publish_package_simple(PackageLoader::get("non_fungible"));
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package,
            "NonFungibleTest",
            "create_burnable_non_fungible",
            manifest_args!(),
        )
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    let resource_address = receipt.expect_commit(true).new_resource_addresses()[0];
    let vault_id = ledger.get_component_vaults(account, resource_address)[0];
    let first_id = ledger
        .inspect_non_fungible_vault(vault_id)
        .unwrap()
        .1
        .next();

    let non_fungible_global_id = NonFungibleGlobalId::new(resource_address, first_id.unwrap());

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .withdraw_from_account(account, resource_address, 1)
        .burn_non_fungible_from_worktop(non_fungible_global_id.clone())
        .call_function(
            package,
            "NonFungibleTest",
            "verify_does_not_exist",
            manifest_args!(non_fungible_global_id),
        )
        .try_deposit_entire_worktop_or_abort(account, None)
        .assert_worktop_contains(resource_address, 0)
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn test_take_non_fungible() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (_, _, account) = ledger.new_allocated_account();
    let package_address = ledger.publish_package_simple(PackageLoader::get("non_fungible"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "NonFungibleTest",
            "take_non_fungible_and_put_bucket",
            manifest_args!(),
        )
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn test_take_non_fungibles() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (_, _, account) = ledger.new_allocated_account();
    let package_address = ledger.publish_package_simple(PackageLoader::get("non_fungible"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "NonFungibleTest",
            "take_non_fungibles_and_put_bucket",
            manifest_args!(),
        )
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn can_update_non_fungible_when_mutable() {
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let package_address = ledger.publish_package_simple(PackageLoader::get("non_fungible"));
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "NonFungibleTest",
            "update_non_fungible",
            manifest_args!(0_u64, "available".to_string(), true),
        )
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_commit_success();
}

#[test]
fn cannot_update_non_fungible_when_not_mutable() {
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let package_address = ledger.publish_package_simple(PackageLoader::get("non_fungible"));
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "NonFungibleTest",
            "update_non_fungible",
            manifest_args!(0_u64, "tastes_great".to_string(), false),
        )
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::NonFungibleResourceManagerError(
                NonFungibleResourceManagerError::UnknownMutableFieldName(..)
            ))
        )
    });
}

#[test]
fn cannot_update_non_fungible_when_field_does_not_exist() {
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let package_address = ledger.publish_package_simple(PackageLoader::get("non_fungible"));
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "NonFungibleTest",
            "update_non_fungible",
            manifest_args!(0_u64, "does_not_exist".to_string(), false),
        )
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::NonFungibleResourceManagerError(
                NonFungibleResourceManagerError::UnknownMutableFieldName(..)
            ))
        )
    });
}

#[test]
fn cannot_update_non_fungible_when_id_does_not_exist() {
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let package_address = ledger.publish_package_simple(PackageLoader::get("non_fungible"));
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "NonFungibleTest",
            "update_non_fungible",
            manifest_args!(666_u64, "available".to_string(), true),
        )
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::NonFungibleResourceManagerError(
                NonFungibleResourceManagerError::NonFungibleNotFound(..)
            ))
        )
    });
}

#[test]
fn cannot_get_non_fungible_when_id_does_not_exist() {
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let package_address = ledger.publish_package_simple(PackageLoader::get("non_fungible"));
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "NonFungibleTest",
            "get_non_fungible",
            manifest_args!(666_u64),
        )
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::NonFungibleResourceManagerError(
                NonFungibleResourceManagerError::NonFungibleNotFound(..)
            ))
        )
    });
}

#[test]
fn can_call_non_fungible_data_reference() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    ledger.set_metadata(
        account.into(),
        "test_key",
        "test_value",
        NonFungibleGlobalId::from_public_key(&public_key),
    );
    let package_address = ledger.publish_package_simple(PackageLoader::get("non_fungible"));
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "NonFungibleTest",
            "create_non_fungible_reference",
            manifest_args!(account),
        )
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    let resource_address = receipt.expect_commit_success().new_resource_addresses()[1];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "NonFungibleTest",
            "call_non_fungible_reference",
            manifest_args!(resource_address),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    let result = receipt.expect_commit_success().outcome.expect_success();
    assert_eq!(
        result[1],
        InstructionOutput::CallReturn(scrypto_encode("test_value").unwrap())
    );
}

#[test]
fn cannot_have_non_fungible_data_ownership() {
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let package_address = ledger.publish_package_simple(PackageLoader::get("non_fungible"));
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "NonFungibleTest",
            "update_non_fungible_with_ownership",
            manifest_args!(),
        )
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemError(SystemError::TypeCheckError(
                TypeCheckError::BlueprintPayloadValidationError(..)
            ))
        )
    });
}

#[test]
fn can_update_and_get_non_fungible() {
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let package_address = ledger.publish_package_simple(PackageLoader::get("non_fungible"));
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "NonFungibleTest",
            "update_and_get_non_fungible",
            manifest_args!(),
        )
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_commit_success();
}

#[test]
fn can_update_and_get_non_fungible_reference() {
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let package_address = ledger.publish_package_simple(PackageLoader::get("non_fungible"));
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "NonFungibleTest",
            "update_and_get_non_fungible_reference",
            manifest_args!(account),
        )
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_commit_success();
}

#[test]
fn can_check_if_contains_non_fungible_in_vault() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let package_address = ledger.publish_package_simple(PackageLoader::get("non_fungible"));
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "NonFungibleTest",
            "contains_non_fungible_vault",
            manifest_args!(),
        )
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();

    // Act
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn can_check_if_contains_non_fungible_in_bucket() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let package_address = ledger.publish_package_simple(PackageLoader::get("non_fungible"));
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "NonFungibleTest",
            "contains_non_fungible_bucket",
            manifest_args!(),
        )
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();

    // Act
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn test_non_fungible_part_1() {
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let package_address = ledger.publish_package_simple(PackageLoader::get("non_fungible"));

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "NonFungibleTest",
            "create_non_fungible_fixed",
            manifest_args!(),
        )
        .call_function(
            package_address,
            "NonFungibleTest",
            "update_and_get_non_fungible",
            manifest_args!(),
        )
        .call_function(
            package_address,
            "NonFungibleTest",
            "non_fungible_exists",
            manifest_args!(),
        )
        .call_function(
            package_address,
            "NonFungibleTest",
            "take_and_put_bucket",
            manifest_args!(),
        )
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_commit_success();
}

#[test]
fn test_non_fungible_part_2() {
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let package_address = ledger.publish_package_simple(PackageLoader::get("non_fungible"));

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "NonFungibleTest",
            "take_and_put_vault",
            manifest_args!(),
        )
        .call_function(
            package_address,
            "NonFungibleTest",
            "get_non_fungible_local_ids_bucket",
            manifest_args!(),
        )
        .call_function(
            package_address,
            "NonFungibleTest",
            "get_non_fungible_local_ids_vault",
            manifest_args!(),
        )
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_commit_success();
}

#[test]
fn test_singleton_non_fungible() {
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let package_address = ledger.publish_package_simple(PackageLoader::get("non_fungible"));

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "NonFungibleTest",
            "singleton_non_fungible",
            manifest_args!(),
        )
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_commit_success();
}

// This test was introduced in Oct 2022 to protect a regression whereby resources locked
// by a proof in a vault was accidentally committed/persisted, and locked in future transactions
#[test]
fn test_mint_update_and_withdraw() {
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let package_address = ledger.publish_package_simple(PackageLoader::get("non_fungible"));

    // create non-fungible
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "NonFungibleTest",
            "create_non_fungible_mutable",
            manifest_args!(),
        )
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    let badge_resource_address = receipt.expect_commit(true).new_resource_addresses()[0];
    let nft_resource_address = receipt.expect_commit(true).new_resource_addresses()[1];

    // update data (the NFT is referenced within a Proof)
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .withdraw_from_account(account, badge_resource_address, 1)
        .create_proof_from_account_of_non_fungibles(
            account,
            nft_resource_address,
            [NonFungibleLocalId::integer(0)],
        )
        .take_all_from_worktop(badge_resource_address, "badge")
        .pop_from_auth_zone("proof")
        .call_function_with_name_lookup(
            package_address,
            "NonFungibleTest",
            "update_nft",
            |lookup| (lookup.bucket("badge"), lookup.proof("proof")),
        )
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_commit_success();

    // Transfer
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .withdraw_from_account(account, nft_resource_address, 1)
        .assert_worktop_contains_any(nft_resource_address)
        .assert_worktop_contains(nft_resource_address, 1)
        // ID from NonFungibleTest::create_non_fungible_mutable
        .assert_worktop_contains_non_fungibles(
            nft_resource_address,
            [NonFungibleLocalId::integer(0)],
        )
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_commit_success();
}

#[test]
fn create_non_fungible_with_id_type_different_than_in_initial_supply() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let package = ledger.publish_package_simple(PackageLoader::get("non_fungible"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package,
            "NonFungibleTest",
            "create_wrong_non_fungible_local_id_type",
            manifest_args!(),
        )
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_failure();
}

#[test]
fn create_bytes_non_fungible() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (_, _, account) = ledger.new_allocated_account();
    let package = ledger.publish_package_simple(PackageLoader::get("non_fungible"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package,
            "NonFungibleTest",
            "create_bytes_non_fungible",
            manifest_args!(),
        )
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn create_string_non_fungible() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (_, _, account) = ledger.new_allocated_account();
    let package = ledger.publish_package_simple(PackageLoader::get("non_fungible"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package,
            "NonFungibleTest",
            "create_string_non_fungible",
            manifest_args!(),
        )
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn create_ruid_non_fungible() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let package = ledger.publish_package_simple(PackageLoader::get("non_fungible"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package,
            "NonFungibleTest",
            "create_ruid_non_fungible",
            manifest_args!(),
        )
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn can_get_total_supply() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package = ledger.publish_package_simple(PackageLoader::get("non_fungible"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package,
            "NonFungibleTest",
            "get_total_supply",
            manifest_args!(),
        )
        .build();

    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn cannot_get_total_supply_when_track_total_supply_disable() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package = ledger.publish_package_simple(PackageLoader::get("non_fungible"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package,
            "NonFungibleTest",
            "get_total_supply_when_track_total_supply_disabled",
            manifest_args!(),
        )
        .build();

    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn can_mint_ruid_non_fungible_in_scrypto() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let package = ledger.publish_package_simple(PackageLoader::get("non_fungible"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package,
            "NonFungibleTest",
            "create_ruid_non_fungible_and_mint",
            manifest_args!(),
        )
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn cannot_create_ruid_non_fungible_and_mint_non_ruid() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let package = ledger.publish_package_simple(PackageLoader::get("non_fungible"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package,
            "NonFungibleTest",
            "create_ruid_non_fungible_and_mint_non_ruid",
            manifest_args!(),
        )
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::NonFungibleResourceManagerError(
                NonFungibleResourceManagerError::InvalidNonFungibleIdType
            ))
        )
    });
}

#[derive(ManifestSbor, ScryptoSbor, NonFungibleData)]
pub struct Sandwich {
    pub name: String,
    #[mutable]
    pub available: bool,
    pub tastes_great: bool,
    #[mutable]
    pub reference: Option<ComponentAddress>,
    #[mutable]
    pub own: Option<()>,
}

#[test]
fn can_mint_ruid_non_fungible_with_reference_in_manifest() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (_, _, account) = ledger.new_allocated_account();
    let package = ledger.publish_package_simple(PackageLoader::get("non_fungible"));
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package,
            "NonFungibleTest",
            "create_mintable_ruid_non_fungible",
            manifest_args!(),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    let resource_address = receipt.expect_commit_success().new_resource_addresses()[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .mint_ruid_non_fungible(
            resource_address,
            vec![Sandwich {
                name: "test".to_string(),
                available: false,
                tastes_great: true,
                reference: Some(account),
                own: None,
            }],
        )
        .assert_worktop_contains_any(resource_address)
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn can_mint_ruid_non_fungible_in_manifest() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (_, _, account) = ledger.new_allocated_account();
    let package = ledger.publish_package_simple(PackageLoader::get("non_fungible"));
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package,
            "NonFungibleTest",
            "create_mintable_ruid_non_fungible",
            manifest_args!(),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    let resource_address = receipt.expect_commit(true).new_resource_addresses()[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .mint_ruid_non_fungible(
            resource_address,
            vec![Sandwich {
                name: "test".to_string(),
                available: false,
                tastes_great: true,
                reference: None,
                own: None,
            }],
        )
        .assert_worktop_contains_any(resource_address)
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn cant_burn_non_fungible_with_wrong_non_fungible_local_id_type() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let package = ledger.publish_package_simple(PackageLoader::get("non_fungible"));
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package,
            "NonFungibleTest",
            "create_burnable_non_fungible",
            manifest_args!(),
        )
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    let resource_address = receipt.expect_commit(true).new_resource_addresses()[0];
    let non_fungible_global_id =
        NonFungibleGlobalId::new(resource_address, NonFungibleLocalId::ruid([0x11; 32]));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .withdraw_from_account(account, resource_address, 1)
        .burn_non_fungible_from_worktop(non_fungible_global_id)
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_failure();
}

#[test]
fn cant_mint_non_fungible_with_different_id_type() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (_, _, _) = ledger.new_allocated_account();
    let package = ledger.publish_package_simple(PackageLoader::get("non_fungible"));
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package,
            "NonFungibleTest",
            "mint_non_fungible_with_different_id_type",
            manifest_args!(),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::NonFungibleResourceManagerError(
                NonFungibleResourceManagerError::NonFungibleIdTypeDoesNotMatch(..)
            ))
        )
    });
}

#[test]
fn cant_mint_non_fungible_with_ruid_id_type() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (_, _, _) = ledger.new_allocated_account();
    let package = ledger.publish_package_simple(PackageLoader::get("non_fungible"));
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package,
            "NonFungibleTest",
            "mint_non_fungible_with_ruid_id_type",
            manifest_args!(),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::NonFungibleResourceManagerError(
                NonFungibleResourceManagerError::NonFungibleIdTypeDoesNotMatch(..)
            ))
        )
    });
}

#[test]
fn cant_mint_ruid_non_fungible_for_non_ruid_non_fungible_resource() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (_, _, _) = ledger.new_allocated_account();
    let package = ledger.publish_package_simple(PackageLoader::get("non_fungible"));
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package,
            "NonFungibleTest",
            "mint_ruid_non_fungible_for_non_ruid_non_fungible_resource",
            manifest_args!(),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::NonFungibleResourceManagerError(
                NonFungibleResourceManagerError::InvalidNonFungibleIdType
            ))
        )
    });
}

#[test]
fn cant_mint_non_fungible_that_already_exists() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (_, _, _) = ledger.new_allocated_account();
    let package = ledger.publish_package_simple(PackageLoader::get("non_fungible"));
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package,
            "NonFungibleTest",
            "mint_non_fungible_that_already_exists",
            manifest_args!(),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::NonFungibleResourceManagerError(
                NonFungibleResourceManagerError::NonFungibleAlreadyExists(..)
            ))
        )
    });
}

#[test]
fn create_non_fungible_with_integer_address_reservation() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let package = ledger.publish_package_simple(PackageLoader::get("non_fungible"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package,
            "NonFungibleTest",
            "create_non_fungible_integer_with_address_reservation",
            manifest_args!(),
        )
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn create_non_fungible_integer() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let package = ledger.publish_package_simple(PackageLoader::get("non_fungible"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package,
            "NonFungibleTest",
            "create_non_fungible_integer",
            manifest_args!(),
        )
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn create_non_fungible_ruid_with_address_reservation() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let package = ledger.publish_package_simple(PackageLoader::get("non_fungible"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package,
            "NonFungibleTest",
            "create_non_fungible_ruid_with_address_reservation",
            manifest_args!(),
        )
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn create_non_fungible_ruid() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let package = ledger.publish_package_simple(PackageLoader::get("non_fungible"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package,
            "NonFungibleTest",
            "create_non_fungible_ruid",
            manifest_args!(),
        )
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn cant_create_non_fungible_with_id_type_does_not_match() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let package = ledger.publish_package_simple(PackageLoader::get("non_fungible"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package,
            "NonFungibleTest",
            "create_non_fungible_with_id_type_does_not_match",
            manifest_args!(),
        )
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::NonFungibleResourceManagerError(
                NonFungibleResourceManagerError::NonFungibleIdTypeDoesNotMatch(..)
            ))
        )
    });
}

#[test]
fn test_non_fungible_global_id() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("resource"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "ResourceTest",
            "non_fungible_global_id",
            manifest_args!(),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    let result = receipt.expect_commit_success();
    println!("{}", result.state_updates_string());
}
