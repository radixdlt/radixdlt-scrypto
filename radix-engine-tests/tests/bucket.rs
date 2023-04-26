use radix_engine::{
    blueprints::resource::BucketError,
    errors::{ApplicationError, RuntimeError},
    types::*,
};
use radix_engine_interface::blueprints::resource::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;
use utils::ContextualDisplay;

fn test_bucket_internal(method_name: &str, args: ManifestValue, expect_success: bool) {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/bucket");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(account, 10.into())
        .call_function(package_address, "BucketTest", method_name, args)
        .call_method(
            account,
            "deposit_batch",
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    if expect_success {
        receipt.expect_commit_success();
    } else {
        receipt.expect_commit_failure();
    }
}

#[test]
fn test_drop_bucket() {
    test_bucket_internal("drop_bucket", manifest_args!(), false);
}

#[test]
fn test_bucket_drop_empty() {
    test_bucket_internal("drop_empty", manifest_args!(0u32), true);
}

#[test]
fn test_bucket_drop_not_empty() {
    test_bucket_internal("drop_empty", manifest_args!(1u32), false);
}

#[test]
fn test_bucket_combine() {
    test_bucket_internal("combine", manifest_args!(), true);
}

#[test]
fn test_bucket_split() {
    test_bucket_internal("split", manifest_args!(), true);
}

#[test]
fn test_bucket_borrow() {
    test_bucket_internal("borrow", manifest_args!(), true);
}

#[test]
fn test_bucket_query() {
    test_bucket_internal("query", manifest_args!(), true);
}

#[test]
fn test_bucket_restricted_transfer() {
    test_bucket_internal("test_restricted_transfer", manifest_args!(), true);
}

#[test]
fn test_bucket_burn() {
    test_bucket_internal("test_burn", manifest_args!(), true);
}

#[test]
fn test_bucket_burn_freely() {
    test_bucket_internal("test_burn_freely", manifest_args!(), true);
}

#[test]
fn test_bucket_empty_fungible() {
    test_bucket_internal("create_empty_bucket_fungible", manifest_args!(), true);
}

#[test]
fn test_bucket_empty_non_fungible() {
    test_bucket_internal("create_empty_bucket_non_fungible", manifest_args!(), true);
}

#[test]
fn test_bucket_of_badges() {
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/bucket");

    let manifest = ManifestBuilder::new()
        .lock_fee(account, 10.into())
        .call_function(package_address, "BadgeTest", "combine", manifest_args!())
        .call_function(package_address, "BadgeTest", "split", manifest_args!())
        .call_function(package_address, "BadgeTest", "borrow", manifest_args!())
        .call_function(package_address, "BadgeTest", "query", manifest_args!())
        .call_method(
            account,
            "deposit_batch",
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_commit_success();
}

#[test]
fn test_take_with_invalid_granularity() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let resource_address = test_runner.create_fungible_resource(100.into(), 2, account);
    let package_address = test_runner.compile_and_publish("./tests/blueprints/bucket");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(account, 10.into())
        .withdraw_from_account(account, resource_address, 100.into())
        .take_from_worktop(resource_address, |builder, bucket_id| {
            let bucket = bucket_id;
            builder.call_function(
                package_address,
                "BucketTest",
                "take_from_bucket",
                manifest_args!(bucket, dec!("1.123")),
            )
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
            RuntimeError::ApplicationError(ApplicationError::BucketError(
                BucketError::InvalidAmount,
            ))
        )
    });
}

#[test]
fn test_take_with_negative_amount() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let resource_address = test_runner.create_fungible_resource(100.into(), 2, account);
    let package_address = test_runner.compile_and_publish("./tests/blueprints/bucket");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(account, 10.into())
        .withdraw_from_account(account, resource_address, 100.into())
        .take_from_worktop(resource_address, |builder, bucket_id| {
            let bucket = bucket_id;
            builder.call_function(
                package_address,
                "BucketTest",
                "take_from_bucket",
                manifest_args!(bucket, dec!("-2")),
            )
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
            RuntimeError::ApplicationError(ApplicationError::BucketError(
                BucketError::InvalidAmount,
            ))
        )
    });
}

#[test]
fn create_empty_bucket() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let non_fungible_resource = test_runner.create_non_fungible_resource(account);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(account, 10.into())
        .take_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
            builder.return_to_worktop(bucket_id)
        })
        .take_from_worktop_by_amount(Decimal::zero(), RADIX_TOKEN, |builder, bucket_id| {
            builder.return_to_worktop(bucket_id)
        })
        .take_from_worktop_by_ids(
            &BTreeSet::new(),
            non_fungible_resource,
            |builder, bucket_id| builder.return_to_worktop(bucket_id),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    println!("{}", receipt.display(&Bech32Encoder::for_simulator()));

    // Assert
    receipt.expect_commit_success();
}
