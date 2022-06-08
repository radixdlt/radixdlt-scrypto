#[rustfmt::skip]
pub mod test_runner;

use crate::test_runner::TestRunner;
use radix_engine::engine::*;
use radix_engine::model::{BucketError, ResourceContainerError};
use scrypto::prelude::*;
use scrypto::to_struct;
use transaction::builder::ManifestBuilder;

#[test]
fn test_bucket() {
    let mut test_runner = TestRunner::new(true);
    let (public_key, _, account) = test_runner.new_account();
    let package_address = test_runner.publish_package("bucket");

    let manifest = ManifestBuilder::new()
        .call_function(package_address, "BucketTest", "combine", to_struct!())
        .call_function(package_address, "BucketTest", "split", to_struct!())
        .call_function(package_address, "BucketTest", "borrow", to_struct!())
        .call_function(package_address, "BucketTest", "query", to_struct!())
        .call_function(
            package_address,
            "BucketTest",
            "test_restricted_transfer",
            to_struct!(),
        )
        .call_function(package_address, "BucketTest", "test_burn", to_struct!())
        .call_function(
            package_address,
            "BucketTest",
            "test_burn_freely",
            to_struct!(),
        )
        .call_function(
            package_address,
            "BucketTest",
            "create_empty_bucket_fungible",
            to_struct!(),
        )
        .call_function(
            package_address,
            "BucketTest",
            "create_empty_bucket_non_fungible",
            to_struct!(),
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![public_key]);
    receipt.expect_success();
}

#[test]
fn test_bucket_of_badges() {
    let mut test_runner = TestRunner::new(true);
    let (public_key, _, account) = test_runner.new_account();
    let package_address = test_runner.publish_package("bucket");

    let manifest = ManifestBuilder::new()
        .call_function(package_address, "BadgeTest", "combine", to_struct!())
        .call_function(package_address, "BadgeTest", "split", to_struct!())
        .call_function(package_address, "BadgeTest", "borrow", to_struct!())
        .call_function(package_address, "BadgeTest", "query", to_struct!())
        .call_method_with_all_resources(account, "deposit_batch")
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![public_key]);
    receipt.expect_success();
}

#[test]
fn test_take_with_invalid_granularity() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let (public_key, _, account) = test_runner.new_account();
    let resource_address = test_runner.create_fungible_resource(100.into(), 2, account);
    let package_address = test_runner.publish_package("bucket");

    // Act
    let manifest = ManifestBuilder::new()
        .call_function_with_abi(
            package_address,
            "BucketTest",
            "take_from_bucket",
            vec![format!("100,{}", resource_address), "1.123".to_owned()],
            Some(account),
            &test_runner.export_abi(package_address, "BucketTest"),
        )
        .unwrap()
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![public_key]);
    println!("{:?}", receipt);

    // Assert
    assert_eq!(
        receipt.result,
        Err(RuntimeError::BucketError(
            BucketError::ResourceContainerError(ResourceContainerError::InvalidAmount(
                dec!("1.123"),
                2
            ))
        ))
    );
}

#[test]
fn test_take_with_negative_amount() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let (public_key, _, account) = test_runner.new_account();
    let resource_address = test_runner.create_fungible_resource(100.into(), 2, account);
    let package_address = test_runner.publish_package("bucket");

    // Act
    let manifest = ManifestBuilder::new()
        .call_function_with_abi(
            package_address,
            "BucketTest",
            "take_from_bucket",
            vec![format!("100,{}", resource_address), "-2".to_owned()],
            Some(account),
            &test_runner.export_abi(package_address, "BucketTest"),
        )
        .unwrap()
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![public_key]);
    println!("{:?}", receipt);

    // Assert
    assert_eq!(
        receipt.result,
        Err(RuntimeError::BucketError(
            BucketError::ResourceContainerError(ResourceContainerError::InvalidAmount(
                dec!("-2"),
                2
            ))
        ))
    );
}

#[test]
fn create_empty_bucket() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let (public_key, _, account) = test_runner.new_account();

    // Act
    let manifest = ManifestBuilder::new()
        .take_from_worktop(scrypto::prelude::RADIX_TOKEN, |builder, _bucket_id| builder)
        .take_from_worktop_by_amount(
            Decimal::zero(),
            scrypto::prelude::RADIX_TOKEN,
            |builder, _bucket_id| builder,
        )
        .take_from_worktop_by_ids(
            &BTreeSet::new(),
            scrypto::prelude::RADIX_TOKEN,
            |builder, _bucket_id| builder,
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![public_key]);
    println!("{:?}", receipt);

    // Assert
    receipt.expect_success();
}
