use radix_engine::blueprints::resource::FungibleResourceManagerError;
use radix_engine::errors::{ApplicationError, RuntimeError, SystemModuleError};
use radix_engine::system::system_modules::auth::AuthError;
use radix_engine::types::*;
use radix_engine_interface::api::node_modules::ModuleConfig;
use radix_engine_interface::blueprints::resource::FromPublicKey;
use radix_engine_interface::{metadata, metadata_init};
use scrypto_unit::*;
use transaction::prelude::*;

#[test]
#[ignore]
fn cannot_get_total_supply_of_xrd() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            XRD,
            RESOURCE_MANAGER_GET_TOTAL_SUPPLY_IDENT,
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    let commit = receipt.expect_commit_success();
    let output: Option<Decimal> = commit.output(1);
    assert!(output.is_none());
}

#[test]
fn test_set_mintable_with_self_resource_address() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (public_key, _, _) = test_runner.new_allocated_account();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/resource");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "ResourceTest",
            "set_mintable_with_self_resource_address",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn test_resource_manager() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/resource");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "ResourceTest",
            "create_fungible",
            manifest_args!(),
        )
        .call_function(package_address, "ResourceTest", "query", manifest_args!())
        .call_function(package_address, "ResourceTest", "burn", manifest_args!())
        .call_function(
            package_address,
            "ResourceTest",
            "update_resource_metadata",
            manifest_args!(),
        )
        .try_deposit_batch_or_abort(account)
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn mint_with_bad_granularity_should_fail() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/resource");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "ResourceTest",
            "create_fungible_and_mint",
            manifest_args!(0u8, dec!("0.1")),
        )
        .try_deposit_batch_or_abort(account)
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        if let RuntimeError::ApplicationError(ApplicationError::FungibleResourceManagerError(
            FungibleResourceManagerError::InvalidAmount(amount, granularity),
        )) = e
        {
            amount.eq(&dec!("0.1")) && *granularity == 0
        } else {
            false
        }
    });
}

#[test]
fn create_fungible_too_high_granularity_should_fail() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (public_key, _, _) = test_runner.new_allocated_account();
    let _package_address = test_runner.compile_and_publish("./tests/blueprints/resource");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_fungible_resource(
            OwnerRole::None,
            false,
            23u8,
            FungibleResourceRoles::default(),
            metadata!(),
            Some(dec!("100")),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        if let RuntimeError::ApplicationError(ApplicationError::FungibleResourceManagerError(
            FungibleResourceManagerError::InvalidDivisibility(granularity),
        )) = e
        {
            *granularity == 23u8
        } else {
            false
        }
    });
}

#[test]
fn mint_too_much_should_fail() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/resource");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "ResourceTest",
            "create_fungible_and_mint",
            manifest_args!(
                0u8,
                // 2^160 subunits rounded up to full units (so should exceed the limit)
                dec!("1461501637330902918203684832717")
            ),
        )
        .try_deposit_batch_or_abort(account)
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::FungibleResourceManagerError(
                FungibleResourceManagerError::MaxMintAmountExceeded
            ))
        )
    })
}

#[test]
fn can_mint_with_proof_in_root() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/resource");
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(package_address, "AuthResource", "create", manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();
    let (admin_token, resource) = test_runner.create_mintable_burnable_fungible_resource(account);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_non_fungibles(
            account,
            admin_token,
            &btreeset!(NonFungibleLocalId::integer(1)),
        )
        .mint_fungible(resource, 1)
        .try_deposit_batch_or_abort(account)
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn cannot_mint_in_component_with_proof_in_root() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/resource");
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(package_address, "AuthResource", "create", manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let component = receipt.expect_commit(true).new_component_addresses()[0];
    let (admin_token, resource) = test_runner.create_mintable_burnable_fungible_resource(account);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_non_fungibles(
            account,
            admin_token,
            &btreeset!(NonFungibleLocalId::integer(1)),
        )
        .call_method(component, "mint", manifest_args!(resource))
        .try_deposit_batch_or_abort(account)
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

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
fn can_burn_with_proof_in_root() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/resource");
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(package_address, "AuthResource", "create", manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();
    let (admin_token, resource) = test_runner.create_mintable_burnable_fungible_resource(account);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_non_fungibles(
            account,
            admin_token,
            &btreeset!(NonFungibleLocalId::integer(1)),
        )
        .mint_fungible(resource, 1)
        .burn_all_from_worktop(resource)
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn cannot_burn_in_component_with_proof_in_root() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/resource");
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(package_address, "AuthResource", "create", manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let component = receipt.expect_commit(true).new_component_addresses()[0];
    let (admin_token, resource) = test_runner.create_mintable_burnable_fungible_resource(account);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_non_fungibles(
            account,
            admin_token,
            &btreeset!(NonFungibleLocalId::integer(1)),
        )
        .mint_fungible(resource, 1)
        .take_all_from_worktop(resource, "to_burn")
        .with_name_lookup(|builder, lookup| {
            builder.call_method(component, "burn", manifest_args!(lookup.bucket("to_burn")))
        })
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

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
fn test_fungible_resource_amount_for_withdrawal() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (_, _, account) = test_runner.new_allocated_account();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/resource");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "RoundingTest",
            "fungible_resource_amount_for_withdrawal",
            manifest_args!(),
        )
        .try_deposit_batch_or_abort(account)
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn test_non_fungible_resource_amount_for_withdrawal() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (_, _, account) = test_runner.new_allocated_account();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/resource");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "RoundingTest",
            "non_fungible_resource_amount_for_withdrawal",
            manifest_args!(),
        )
        .try_deposit_batch_or_abort(account)
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn test_fungible_resource_take_advanced() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (_, _, account) = test_runner.new_allocated_account();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/resource");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "RoundingTest",
            "fungible_resource_take_advanced",
            manifest_args!(),
        )
        .try_deposit_batch_or_abort(account)
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn test_non_fungible_resource_take_advanced() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (_, _, account) = test_runner.new_allocated_account();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/resource");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "RoundingTest",
            "non_fungible_resource_take_advanced",
            manifest_args!(),
        )
        .try_deposit_batch_or_abort(account)
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn can_use_fungible_types_in_interface() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (_, _, account) = test_runner.new_allocated_account();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/resource");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "ResourceTypes",
            "test_fungible_types",
            manifest_args!(),
        )
        .try_deposit_batch_or_abort(account)
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn can_use_non_fungible_types_in_interface() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (_, _, account) = test_runner.new_allocated_account();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/resource");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "ResourceTypes",
            "test_non_fungible_types",
            manifest_args!(),
        )
        .try_deposit_batch_or_abort(account)
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}
