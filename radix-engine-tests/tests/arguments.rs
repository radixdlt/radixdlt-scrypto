use radix_engine::types::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn vector_of_buckets_argument_should_succeed() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/arguments");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .take_from_worktop(RADIX_TOKEN, |builder, bucket_id1| {
            builder.take_from_worktop(RADIX_TOKEN, |builder, bucket_id2| {
                builder.call_function(
                    package_address,
                    "Arguments",
                    "vector_argument",
                    manifest_args!(vec![bucket_id1, bucket_id2,]),
                )
            })
        })
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn tuple_of_buckets_argument_should_succeed() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/arguments");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .take_from_worktop(RADIX_TOKEN, |builder, bucket_id1| {
            builder.take_from_worktop(RADIX_TOKEN, |builder, bucket_id2| {
                builder.call_function(
                    package_address,
                    "Arguments",
                    "tuple_argument",
                    manifest_args!((bucket_id1, bucket_id2,)),
                )
            })
        })
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn treemap_of_strings_and_buckets_argument_should_succeed() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/arguments");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .take_from_worktop(RADIX_TOKEN, |builder, bucket_id1| {
            builder.take_from_worktop(RADIX_TOKEN, |builder, bucket_id2| {
                let mut map = BTreeMap::new();
                map.insert("first".to_string(), bucket_id1);
                map.insert("second".to_string(), bucket_id2);

                builder.call_function(
                    package_address,
                    "Arguments",
                    "treemap_argument",
                    manifest_args!(map),
                )
            })
        })
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn hashmap_of_strings_and_buckets_argument_should_succeed() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/arguments");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .take_from_worktop(RADIX_TOKEN, |builder, bucket_id1| {
            builder.take_from_worktop(RADIX_TOKEN, |builder, bucket_id2| {
                let mut map = HashMap::new();
                map.insert("first".to_string(), bucket_id1);
                map.insert("second".to_string(), bucket_id2);

                builder.call_function(
                    package_address,
                    "Arguments",
                    "hashmap_argument",
                    manifest_args!(map),
                )
            })
        })
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn some_optional_bucket_argument_should_succeed() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/arguments");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .take_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
            builder.call_function(
                package_address,
                "Arguments",
                "option_argument",
                manifest_args!(Some(bucket_id)),
            )
        })
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn none_optional_bucket_argument_should_succeed() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/arguments");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_function(
            package_address,
            "Arguments",
            "option_argument",
            manifest_args!(Option::<ManifestBucket>::None),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}
