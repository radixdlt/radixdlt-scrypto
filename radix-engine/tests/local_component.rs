#[rustfmt::skip]
pub mod test_runner;

use crate::test_runner::TestRunner;
use radix_engine::engine::RuntimeError;
use scrypto::prelude::*;
use scrypto::to_struct;
use transaction::builder::ManifestBuilder;

#[test]
fn local_component_should_return_correct_info() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.extract_and_publish_package("local_component");

    // Act
    let manifest = ManifestBuilder::new()
        .call_function(
            package_address,
            "LocalComponent",
            "check_info_of_local_component",
            to_struct!(package_address, "LocalComponent".to_string()),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_success();
}

#[test]
fn local_component_should_be_callable_read_only() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.extract_and_publish_package("local_component");

    // Act
    let manifest = ManifestBuilder::new()
        .call_function(
            package_address,
            "LocalComponent",
            "read_local_component",
            to_struct!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_success();
}

#[test]
fn local_component_should_be_callable_with_write() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.extract_and_publish_package("local_component");

    // Act
    let manifest = ManifestBuilder::new()
        .call_function(
            package_address,
            "LocalComponent",
            "write_local_component",
            to_struct!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_success();
}

#[test]
fn local_component_with_access_rules_should_not_be_callable() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.extract_and_publish_package("local_component");
    let (public_key, _, account) = test_runner.new_account();
    let auth_resource_address = test_runner.create_non_fungible_resource(account);
    let auth_id = NonFungibleId::from_u32(1);
    let auth_address = NonFungibleAddress::new(auth_resource_address, auth_id);

    // Act
    let manifest = ManifestBuilder::new()
        .call_function(
            package_address,
            "LocalComponent",
            "try_to_read_local_component_with_auth",
            to_struct!(auth_address),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![public_key]);

    // Assert
    receipt.expect_err(|e| matches!(e, RuntimeError::AuthorizationError { .. }));
}

#[test]
fn local_component_with_access_rules_should_be_callable() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.extract_and_publish_package("local_component");
    let (public_key, _, account) = test_runner.new_account();
    let auth_resource_address = test_runner.create_non_fungible_resource(account);
    let auth_id = NonFungibleId::from_u32(1);
    let auth_address = NonFungibleAddress::new(auth_resource_address, auth_id.clone());

    // Act
    let manifest = ManifestBuilder::new()
        .call_method(
            account,
            "create_proof_by_ids",
            to_struct!(BTreeSet::from([auth_id.clone()]), auth_resource_address),
        )
        .call_function(
            package_address,
            "LocalComponent",
            "try_to_read_local_component_with_auth",
            to_struct!(auth_address),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![public_key]);

    // Assert
    receipt.expect_success()
}

#[test]
fn recursion_bomb() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let (public_key, _, account) = test_runner.new_account();
    let package_address = test_runner.extract_and_publish_package("local_component");

    // Act
    // Note: currently SEGFAULT occurs if bucket with too much in it is sent. My guess the issue is a native stack overflow.
    let manifest = ManifestBuilder::new()
        .withdraw_from_account_by_amount(Decimal::from(10), RADIX_TOKEN, account)
        .take_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
            builder.call_function(
                package_address,
                "LocalRecursionBomb",
                "recursion_bomb",
                to_struct!(scrypto::resource::Bucket(bucket_id)),
            )
        })
        .call_method_with_all_resources(account, "deposit_batch")
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![public_key]);

    // Assert
    receipt.expect_success();
}

#[test]
fn recursion_bomb_to_failure() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let (public_key, _, account) = test_runner.new_account();
    let package_address = test_runner.extract_and_publish_package("component");

    // Act
    let manifest = ManifestBuilder::new()
        .withdraw_from_account_by_amount(Decimal::from(100), RADIX_TOKEN, account)
        .take_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
            builder.call_function(
                package_address,
                "LocalRecursionBomb",
                "recursion_bomb",
                to_struct!(scrypto::resource::Bucket(bucket_id)),
            )
        })
        .call_method_with_all_resources(account, "deposit_batch")
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![public_key]);

    // Assert
    receipt.expect_err(|e| matches!(e, RuntimeError::MaxCallDepthLimitReached));
}

#[test]
fn recursion_bomb_2() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let (public_key, _, account) = test_runner.new_account();
    let package_address = test_runner.extract_and_publish_package("local_component");

    // Act
    let manifest = ManifestBuilder::new()
        .withdraw_from_account_by_amount(Decimal::from(10), RADIX_TOKEN, account)
        .take_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
            builder.call_function(
                package_address,
                "LocalRecursionBomb2",
                "recursion_bomb",
                to_struct!(scrypto::resource::Bucket(bucket_id)),
            )
        })
        .call_method_with_all_resources(account, "deposit_batch")
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![public_key]);

    // Assert
    receipt.expect_success();
}

#[test]
fn recursion_bomb_2_to_failure() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let (public_key, _, account) = test_runner.new_account();
    let package_address = test_runner.extract_and_publish_package("component");

    // Act
    let manifest = ManifestBuilder::new()
        .withdraw_from_account_by_amount(Decimal::from(100), RADIX_TOKEN, account)
        .take_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
            builder.call_function(
                package_address,
                "LocalRecursionBomb2",
                "recursion_bomb",
                to_struct!(scrypto::resource::Bucket(bucket_id)),
            )
        })
        .call_method_with_all_resources(account, "deposit_batch")
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![public_key]);

    // Assert
    receipt.expect_err(|e| matches!(e, RuntimeError::MaxCallDepthLimitReached));
}
