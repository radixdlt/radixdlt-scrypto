use radix_engine::blueprints::resource::VaultError;
use radix_engine::errors::{
    ApplicationError, CallFrameError, KernelError, RuntimeError, SystemError,
};
use radix_engine::kernel::call_frame::{CloseSubstateError, CreateNodeError, TakeNodeError};
use radix_engine::types::*;
use radix_engine_interface::api::node_modules::ModuleConfig;
use radix_engine_interface::{metadata, metadata_init};
use scrypto::prelude::FromPublicKey;
use scrypto::NonFungibleData;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn non_existent_vault_in_component_creation_should_fail() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/vault");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
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
            RuntimeError::SystemError(SystemError::PayloadValidationAgainstSchemaError(..))
        )
    });
}

#[test]
fn non_existent_vault_in_committed_component_should_fail() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/vault");
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .call_function(package_address, "NonExistentVault", "new", manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let component_address = receipt.expect_commit(true).new_component_addresses()[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
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
            RuntimeError::SystemError(SystemError::PayloadValidationAgainstSchemaError(..))
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
        .lock_fee(test_runner.faucet_component(), 500u32.into())
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
            RuntimeError::SystemError(SystemError::InvalidSubstateWrite(_))
        )
    });
}

#[test]
fn non_existent_vault_in_committed_kv_store_should_fail() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/vault");
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .call_function(package_address, "NonExistentVault", "new", manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let component_address = receipt.expect_commit(true).new_component_addresses()[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
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
            RuntimeError::SystemError(SystemError::InvalidSubstateWrite(_))
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
        .lock_fee(test_runner.faucet_component(), 500u32.into())
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
        .lock_fee(test_runner.faucet_component(), 500u32.into())
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
                CallFrameError::CreateNodeError(CreateNodeError::TakeNodeError(
                    TakeNodeError::OwnNotFound(_)
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
        .lock_fee(test_runner.faucet_component(), 500u32.into())
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
        .lock_fee(test_runner.faucet_component(), 500u32.into())
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
        .lock_fee(test_runner.faucet_component(), 500u32.into())
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
                CallFrameError::CloseSubstateError(CloseSubstateError::CantDropNodeInStore(_))
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
        .lock_fee(test_runner.faucet_component(), 500u32.into())
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
        .lock_fee(test_runner.faucet_component(), 500u32.into())
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
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .call_method(component_address, "clear_vector", manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::KernelError(KernelError::CallFrameError(
                CallFrameError::CloseSubstateError(CloseSubstateError::CantDropNodeInStore(_))
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
        .lock_fee(test_runner.faucet_component(), 500u32.into())
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
        .lock_fee(test_runner.faucet_component(), 500u32.into())
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
fn create_fungible_vault_with_take() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/vault");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .call_function(
            package_address,
            "VaultTest",
            "new_fungible_vault_with_take",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn create_non_fungible_vault_with_take() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/vault");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .call_function(
            package_address,
            "NonFungibleVault",
            "new_non_fungible_vault_with_take",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn create_non_fungible_vault_with_take_twice() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/vault");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .call_function(
            package_address,
            "NonFungibleVault",
            "new_non_fungible_vault_with_take_twice",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn create_non_fungible_vault_with_take_non_fungible() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/vault");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .call_function(
            package_address,
            "NonFungibleVault",
            "new_non_fungible_vault_with_take_non_fungible",
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
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .call_function(
            package_address,
            "NonFungibleVault",
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
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .call_function(
            package_address,
            "NonFungibleVault",
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
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .call_function(
            package_address,
            "NonFungibleVault",
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
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .call_function(
            package_address,
            "NonFungibleVault",
            "new_vault_with_get_resource_manager",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn take_on_non_fungible_vault() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/vault");
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .call_function(
            package_address,
            "NonFungibleVault",
            "new_non_fungible_vault",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let address = receipt.expect_commit_success().new_component_addresses()[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .call_method(address, "take", manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn take_twice_on_non_fungible_vault() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/vault");
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .call_function(
            package_address,
            "NonFungibleVault",
            "new_non_fungible_vault",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let address = receipt.expect_commit_success().new_component_addresses()[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .call_method(address, "take_twice", manifest_args!())
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
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .withdraw_from_account(account, resource_address, dec!("5.55555"))
        .call_method(
            account,
            "try_deposit_batch_or_abort",
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
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .create_proof_from_account_of_amount(account, resource_address, dec!("5.55555"))
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
fn taking_resource_from_non_fungible_vault_should_reduce_the_contained_amount() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/vault");
    let (_, _, account) = test_runner.new_account(false);
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
            "take_ids",
            manifest_args!(btreeset![NonFungibleLocalId::integer(1)]),
        )
        .try_deposit_batch_or_abort(account)
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
    assert_eq!(
        test_runner.inspect_non_fungible_vault(vault_id).unwrap().0,
        dec!("1")
    );
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
