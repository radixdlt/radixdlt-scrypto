use radix_engine::engine::*;
use radix_engine::ledger::TypedInMemorySubstateStore;
use radix_engine::model::{BucketError, ResourceContainerError};
use scrypto::core::NetworkDefinition;
use scrypto::prelude::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

fn test_bucket_internal(method_name: &str) {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_account();
    let package_address = test_runner.extract_and_publish_package("bucket");

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::local_simulator())
        .lock_fee(10.into(), account)
        .call_function(package_address, "BucketTest", method_name, args!())
        .call_method(
            account,
            "deposit_batch",
            args!(Expression::new("ALL_WORKTOP_RESOURCES")),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![public_key]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn test_bucket_combine() {
    test_bucket_internal("combine");
}

#[test]
fn test_bucket_split() {
    test_bucket_internal("split");
}

#[test]
fn test_bucket_borrow() {
    test_bucket_internal("borrow");
}

#[test]
fn test_bucket_query() {
    test_bucket_internal("query");
}

#[test]
fn test_bucket_restricted_transfer() {
    test_bucket_internal("test_restricted_transfer");
}

#[test]
fn test_bucket_burn() {
    test_bucket_internal("test_burn");
}

#[test]
fn test_bucket_burn_freely() {
    test_bucket_internal("test_burn_freely");
}

#[test]
fn test_bucket_empty_fungible() {
    test_bucket_internal("create_empty_bucket_fungible");
}

#[test]
fn test_bucket_empty_non_fungible() {
    test_bucket_internal("create_empty_bucket_non_fungible");
}

#[test]
fn test_bucket_of_badges() {
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_account();
    let package_address = test_runner.extract_and_publish_package("bucket");

    let manifest = ManifestBuilder::new(&NetworkDefinition::local_simulator())
        .lock_fee(10.into(), account)
        .call_function(package_address, "BadgeTest", "combine", args!())
        .call_function(package_address, "BadgeTest", "split", args!())
        .call_function(package_address, "BadgeTest", "borrow", args!())
        .call_function(package_address, "BadgeTest", "query", args!())
        .call_method(
            account,
            "deposit_batch",
            args!(Expression::new("ALL_WORKTOP_RESOURCES")),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![public_key]);
    receipt.expect_commit_success();
}

#[test]
fn test_take_with_invalid_granularity() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_account();
    let resource_address = test_runner.create_fungible_resource(100.into(), 2, account);
    let package_address = test_runner.extract_and_publish_package("bucket");

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::local_simulator())
        .lock_fee(10.into(), account)
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

    // Assert
    receipt.expect_commit_failure(|e| {
        if let RuntimeError::ApplicationError(ApplicationError::BucketError(
            BucketError::ResourceContainerError(ResourceContainerError::InvalidAmount(
                amount,
                granularity,
            )),
        )) = e
        {
            amount.eq(&dec!("1.123")) && *granularity == 2
        } else {
            false
        }
    });
}

#[test]
fn test_take_with_negative_amount() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_account();
    let resource_address = test_runner.create_fungible_resource(100.into(), 2, account);
    let package_address = test_runner.extract_and_publish_package("bucket");

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::local_simulator())
        .lock_fee(10.into(), account)
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

    // Assert
    receipt.expect_commit_failure(|e| {
        if let RuntimeError::ApplicationError(ApplicationError::BucketError(
            BucketError::ResourceContainerError(ResourceContainerError::InvalidAmount(
                amount,
                granularity,
            )),
        )) = e
        {
            amount.eq(&dec!("-2")) && *granularity == 2
        } else {
            false
        }
    });
}

#[test]
fn create_empty_bucket() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_account();

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::local_simulator())
        .lock_fee(10.into(), account)
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
        .call_method(
            account,
            "deposit_batch",
            args!(Expression::new("ALL_WORKTOP_RESOURCES")),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![public_key]);
    println!("{:?}", receipt);

    // Assert
    receipt.expect_commit_success();
}
