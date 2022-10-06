use radix_engine::engine::*;
use radix_engine::ledger::TypedInMemorySubstateStore;
use radix_engine::model::{BucketError, ResourceOperationError};
use radix_engine::types::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

fn test_bucket_internal(method_name: &str) {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_account();
    let package_address = test_runner.compile_and_publish("./tests/bucket");

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), account)
        .call_scrypto_function(package_address, "BucketTest", method_name, args!())
        .call_method(
            account,
            "deposit_batch",
            args!(Expression::entire_worktop()),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleAddress::from_public_key(&public_key)],
    );

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
    let package_address = test_runner.compile_and_publish("./tests/bucket");

    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), account)
        .call_scrypto_function(package_address, "BadgeTest", "combine", args!())
        .call_scrypto_function(package_address, "BadgeTest", "split", args!())
        .call_scrypto_function(package_address, "BadgeTest", "borrow", args!())
        .call_scrypto_function(package_address, "BadgeTest", "query", args!())
        .call_method(
            account,
            "deposit_batch",
            args!(Expression::entire_worktop()),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleAddress::from_public_key(&public_key)],
    );
    receipt.expect_commit_success();
}

#[test]
fn test_take_with_invalid_granularity() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_account();
    let resource_address = test_runner.create_fungible_resource(100.into(), 2, account);
    let resource_address_str =
        Bech32Encoder::for_simulator().encode_resource_address_to_string(&resource_address);
    let package_address = test_runner.compile_and_publish("./tests/bucket");

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), account)
        .call_function_with_abi(
            package_address,
            "BucketTest",
            "take_from_bucket",
            vec![format!("100,{}", resource_address_str), "1.123".to_owned()],
            Some(account),
            &test_runner.export_abi(package_address, "BucketTest"),
        )
        .unwrap()
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleAddress::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        if let RuntimeError::ApplicationError(ApplicationError::BucketError(
            BucketError::ResourceOperationError(ResourceOperationError::InvalidAmount(
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
    let resource_address_str =
        Bech32Encoder::for_simulator().encode_resource_address_to_string(&resource_address);
    let package_address = test_runner.compile_and_publish("./tests/bucket");

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), account)
        .call_function_with_abi(
            package_address,
            "BucketTest",
            "take_from_bucket",
            vec![format!("100,{}", resource_address_str), "-2".to_owned()],
            Some(account),
            &test_runner.export_abi(package_address, "BucketTest"),
        )
        .unwrap()
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleAddress::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        if let RuntimeError::ApplicationError(ApplicationError::BucketError(
            BucketError::ResourceOperationError(ResourceOperationError::InvalidAmount(
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
    let non_fungible_resource = test_runner.create_non_fungible_resource(account);

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), account)
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
        vec![NonFungibleAddress::from_public_key(&public_key)],
    );
    println!("{}", receipt.displayable(&Bech32Encoder::for_simulator()));

    // Assert
    receipt.expect_commit_success();
}
