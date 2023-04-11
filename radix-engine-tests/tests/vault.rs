use radix_engine::blueprints::resource::VaultError;
use radix_engine::errors::{ApplicationError, CallFrameError, KernelError, RuntimeError};
use radix_engine::kernel::call_frame::{MoveError, UnlockSubstateError};
use radix_engine::types::*;
use scrypto::prelude::FromPublicKey;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn non_existent_vault_in_component_creation_should_fail() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/vault");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10u32.into())
        .call_function(
            package_address,
            "NonExistentVault",
            "create_component_with_non_existent_vault",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::KernelError(KernelError::CallFrameError(
                CallFrameError::UnlockSubstateError(UnlockSubstateError::MoveError(
                    MoveError::OwnNotFound(_)
                ))
            ))
        )
    });
}

#[test]
fn non_existent_vault_in_committed_component_should_fail() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/vault");
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10u32.into())
        .call_function(package_address, "NonExistentVault", "new", manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let component_address = receipt.expect_commit(true).new_component_addresses()[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10u32.into())
        .call_method(
            component_address,
            "create_non_existent_vault",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::KernelError(KernelError::CallFrameError(
                CallFrameError::UnlockSubstateError(UnlockSubstateError::MoveError(
                    MoveError::OwnNotFound(_)
                ))
            ))
        )
    });
}

#[test]
fn non_existent_vault_in_kv_store_creation_should_fail() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/vault");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10u32.into())
        .call_function(
            package_address,
            "NonExistentVault",
            "create_kv_store_with_non_existent_vault",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::KernelError(KernelError::CallFrameError(
                CallFrameError::UnlockSubstateError(UnlockSubstateError::MoveError(
                    MoveError::OwnNotFound(_)
                ))
            ))
        )
    });
}

#[test]
fn non_existent_vault_in_committed_kv_store_should_fail() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/vault");
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10u32.into())
        .call_function(package_address, "NonExistentVault", "new", manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let component_address = receipt.expect_commit(true).new_component_addresses()[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10u32.into())
        .call_method(
            component_address,
            "create_non_existent_vault_in_kv_store",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::KernelError(KernelError::CallFrameError(
                CallFrameError::UnlockSubstateError(UnlockSubstateError::MoveError(
                    MoveError::OwnNotFound(_)
                ))
            ))
        )
    });
}

#[test]
fn create_mutable_vault_into_map() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/vault");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_function(
            package_address,
            "VaultTest",
            "new_vault_into_map",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn invalid_double_ownership_of_vault() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/vault");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10u32.into())
        .call_function(
            package_address,
            "VaultTest",
            "invalid_double_ownership_of_vault",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::KernelError(KernelError::CallFrameError(
                CallFrameError::UnlockSubstateError(UnlockSubstateError::MoveError(
                    MoveError::OwnNotFound(_)
                ))
            ))
        )
    });
}

#[test]
fn create_mutable_vault_into_map_and_referencing_before_storing() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/vault");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_function(
            package_address,
            "VaultTest",
            "new_vault_into_map_then_get",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn cannot_overwrite_vault_in_map() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/vault");
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10u32.into())
        .call_function(
            package_address,
            "VaultTest",
            "new_vault_into_map",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let component_address = receipt.expect_commit(true).new_component_addresses()[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10u32.into())
        .call_method(
            component_address,
            "overwrite_vault_in_map",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::KernelError(KernelError::CallFrameError(
                CallFrameError::UnlockSubstateError(UnlockSubstateError::CantDropNodeInStore(_))
            ))
        )
    });
}

#[test]
fn create_mutable_vault_into_vector() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/vault");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_function(
            package_address,
            "VaultTest",
            "new_vault_into_vector",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn cannot_remove_vaults() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/vault");
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10u32.into())
        .call_function(
            package_address,
            "VaultTest",
            "new_vault_into_vector",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let component_address = receipt.expect_commit(true).new_component_addresses()[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10u32.into())
        .call_method(component_address, "clear_vector", manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::KernelError(KernelError::CallFrameError(
                CallFrameError::UnlockSubstateError(UnlockSubstateError::CantDropNodeInStore(_))
            ))
        )
    });
}

#[test]
fn can_push_vault_into_vector() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/vault");
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_function(
            package_address,
            "VaultTest",
            "new_vault_into_vector",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let component_address = receipt.expect_commit(true).new_component_addresses()[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_method(
            component_address,
            "push_vault_into_vector",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn create_mutable_vault_with_take() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/vault");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_function(
            package_address,
            "VaultTest",
            "new_vault_with_take",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn create_mutable_vault_with_take_non_fungible() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/vault");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_function(
            package_address,
            "VaultTest",
            "new_vault_with_take_non_fungible",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn create_mutable_vault_with_get_nonfungible_ids() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/vault");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_function(
            package_address,
            "VaultTest",
            "new_vault_with_get_non_fungible_local_ids",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn create_mutable_vault_with_get_nonfungible_id() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/vault");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_function(
            package_address,
            "VaultTest",
            "new_vault_with_get_non_fungible_local_id",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn create_mutable_vault_with_get_amount() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/vault");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_function(
            package_address,
            "VaultTest",
            "new_vault_with_get_amount",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn create_mutable_vault_with_get_resource_manager() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/vault");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_function(
            package_address,
            "VaultTest",
            "new_vault_with_get_resource_manager",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn withdraw_with_over_specified_divisibility_should_result_in_error() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (pk, _, account) = test_runner.new_allocated_account();
    let resource_address = test_runner.create_fungible_resource(100u32.into(), 4, account);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .withdraw_from_account(account, resource_address, dec!("5.55555"))
        .call_method(
            account,
            "deposit_batch",
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();
    let receipt =
        test_runner.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&pk)]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::VaultError(VaultError::InvalidAmount))
        )
    });
}

#[test]
fn create_proof_with_over_specified_divisibility_should_result_in_error() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (pk, _, account) = test_runner.new_allocated_account();
    let resource_address = test_runner.create_fungible_resource(100u32.into(), 4, account);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .create_proof_from_account_by_amount(account, resource_address, dec!("5.55555"))
        .build();
    let receipt =
        test_runner.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&pk)]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::VaultError(VaultError::InvalidAmount))
        )
    });
}
