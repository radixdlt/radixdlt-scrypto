use scrypto_unit::*;

use scrypto::core::NetworkDefinition;
use transaction::builder::ManifestBuilder;

use radix_engine::ledger::TypedInMemorySubstateStore;
use scrypto::prelude::*;

#[test]
fn vector_of_buckets_argument_should_succeed() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.extract_and_publish_package("arguments");

    // Act
    let manifest = ManifestBuilder::new(NetworkDefinition::local_simulator())
        .lock_fee(10.into(), SYSTEM_COMPONENT)
        .take_from_worktop(RADIX_TOKEN, |builder, bucket_id1| {
            builder.take_from_worktop(RADIX_TOKEN, |builder, bucket_id2| {
                builder.call_function(
                    package_address,
                    "Arguments",
                    "vector_argument",
                    args!(vec![Bucket(bucket_id1), Bucket(bucket_id2),]),
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
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.extract_and_publish_package("arguments");

    // Act
    let manifest = ManifestBuilder::new(NetworkDefinition::local_simulator())
        .lock_fee(10.into(), SYSTEM_COMPONENT)
        .take_from_worktop(RADIX_TOKEN, |builder, bucket_id1| {
            builder.take_from_worktop(RADIX_TOKEN, |builder, bucket_id2| {
                builder.call_function(
                    package_address,
                    "Arguments",
                    "tuple_argument",
                    args!((Bucket(bucket_id1), Bucket(bucket_id2),)),
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
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.extract_and_publish_package("arguments");

    // Act
    let manifest = ManifestBuilder::new(NetworkDefinition::local_simulator())
        .lock_fee(10.into(), SYSTEM_COMPONENT)
        .take_from_worktop(RADIX_TOKEN, |builder, bucket_id1| {
            builder.take_from_worktop(RADIX_TOKEN, |builder, bucket_id2| {
                let mut map = BTreeMap::new();
                map.insert("first".to_string(), Bucket(bucket_id1));
                map.insert("second".to_string(), Bucket(bucket_id2));

                builder.call_function(package_address, "Arguments", "treemap_argument", args!(map))
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
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.extract_and_publish_package("arguments");

    // Act
    let manifest = ManifestBuilder::new(NetworkDefinition::local_simulator())
        .lock_fee(10.into(), SYSTEM_COMPONENT)
        .take_from_worktop(RADIX_TOKEN, |builder, bucket_id1| {
            builder.take_from_worktop(RADIX_TOKEN, |builder, bucket_id2| {
                let mut map = HashMap::new();
                map.insert("first".to_string(), Bucket(bucket_id1));
                map.insert("second".to_string(), Bucket(bucket_id2));

                builder.call_function(package_address, "Arguments", "hashmap_argument", args!(map))
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
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.extract_and_publish_package("arguments");

    // Act
    let manifest = ManifestBuilder::new(NetworkDefinition::local_simulator())
        .lock_fee(10.into(), SYSTEM_COMPONENT)
        .take_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
            builder.call_function(
                package_address,
                "Arguments",
                "option_argument",
                args!(Some(Bucket(bucket_id))),
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
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.extract_and_publish_package("arguments");

    // Act
    let manifest = ManifestBuilder::new(NetworkDefinition::local_simulator())
        .lock_fee(10.into(), SYSTEM_COMPONENT)
        .call_function(
            package_address,
            "Arguments",
            "option_argument",
            args!(Option::<Bucket>::None),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_success();
}
