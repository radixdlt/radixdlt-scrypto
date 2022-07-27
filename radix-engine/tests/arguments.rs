#[rustfmt::skip]
pub mod test_runner;
use crate::test_runner::TestRunner;

use scrypto::core::Network;
use transaction::builder::ManifestBuilder;

use radix_engine::ledger::InMemorySubstateStore;
use scrypto::prelude::*;
use scrypto::to_struct;

#[test]
fn vector_of_buckets_argument_should_succeed() {
    // Arrange
    let mut store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.extract_and_publish_package("arguments");

    // Act
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .pay_fee(10.into(), SYSTEM_COMPONENT)
        .take_from_worktop(RADIX_TOKEN, |builder, bucket_id1| {
            builder.take_from_worktop(RADIX_TOKEN, |builder, bucket_id2| {
                builder.call_function(
                    package_address,
                    "Arguments",
                    "vector_argument",
                    to_struct!(vec![Bucket(bucket_id1), Bucket(bucket_id2),]),
                )
            })
        })
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_success();
}

#[test]
fn tuple_of_buckets_argument_should_succeed() {
    // Arrange
    let mut store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.extract_and_publish_package("arguments");

    // Act
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .pay_fee(10.into(), SYSTEM_COMPONENT)
        .take_from_worktop(RADIX_TOKEN, |builder, bucket_id1| {
            builder.take_from_worktop(RADIX_TOKEN, |builder, bucket_id2| {
                builder.call_function(
                    package_address,
                    "Arguments",
                    "tuple_argument",
                    to_struct!((Bucket(bucket_id1), Bucket(bucket_id2),)),
                )
            })
        })
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_success();
}

#[test]
fn treemap_of_strings_and_buckets_argument_should_succeed() {
    // Arrange
    let mut store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.extract_and_publish_package("arguments");

    // Act
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .pay_fee(10.into(), SYSTEM_COMPONENT)
        .take_from_worktop(RADIX_TOKEN, |builder, bucket_id1| {
            builder.take_from_worktop(RADIX_TOKEN, |builder, bucket_id2| {
                let mut map = BTreeMap::new();
                map.insert("first".to_string(), Bucket(bucket_id1));
                map.insert("second".to_string(), Bucket(bucket_id2));

                builder.call_function(
                    package_address,
                    "Arguments",
                    "treemap_argument",
                    to_struct!(map),
                )
            })
        })
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_success();
}

#[test]
fn hashmap_of_strings_and_buckets_argument_should_succeed() {
    // Arrange
    let mut store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.extract_and_publish_package("arguments");

    // Act
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .pay_fee(10.into(), SYSTEM_COMPONENT)
        .take_from_worktop(RADIX_TOKEN, |builder, bucket_id1| {
            builder.take_from_worktop(RADIX_TOKEN, |builder, bucket_id2| {
                let mut map = HashMap::new();
                map.insert("first".to_string(), Bucket(bucket_id1));
                map.insert("second".to_string(), Bucket(bucket_id2));

                builder.call_function(
                    package_address,
                    "Arguments",
                    "hashmap_argument",
                    to_struct!(map),
                )
            })
        })
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_success();
}

#[test]
fn some_optional_bucket_argument_should_succeed() {
    // Arrange
    let mut store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.extract_and_publish_package("arguments");

    // Act
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .pay_fee(10.into(), SYSTEM_COMPONENT)
        .take_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
            builder.call_function(
                package_address,
                "Arguments",
                "option_argument",
                to_struct!(Some(Bucket(bucket_id))),
            )
        })
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_success();
}

#[test]
fn none_optional_bucket_argument_should_succeed() {
    // Arrange
    let mut store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.extract_and_publish_package("arguments");

    // Act
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .pay_fee(10.into(), SYSTEM_COMPONENT)
        .call_function(
            package_address,
            "Arguments",
            "option_argument",
            to_struct!(Option::<Bucket>::None),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_success();
}
