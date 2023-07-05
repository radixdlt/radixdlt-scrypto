use radix_engine::types::*;
use scrypto_unit::*;
use transaction::prelude::*;

#[test]
fn vector_of_buckets_argument_should_succeed() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/arguments");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .take_all_from_worktop(XRD, "bucket1")
        .take_all_from_worktop(XRD, "bucket2")
        .with_namer(|builder, namer| {
            builder.call_function(
                package_address,
                "Arguments",
                "vector_argument",
                manifest_args!(vec![namer.bucket("bucket1"), namer.bucket("bucket2"),]),
            )
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
        .lock_fee_from_faucet()
        .take_all_from_worktop(XRD, "bucket1")
        .take_all_from_worktop(XRD, "bucket2")
        .with_namer(|builder, namer| {
            builder.call_function(
                package_address,
                "Arguments",
                "tuple_argument",
                manifest_args!((namer.bucket("bucket1"), namer.bucket("bucket2"))),
            )
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
        .lock_fee_from_faucet()
        .take_all_from_worktop(XRD, "bucket1")
        .take_all_from_worktop(XRD, "bucket2")
        .with_namer(|builder, namer| {
            let mut map = BTreeMap::new();
            map.insert("first".to_string(), namer.bucket("bucket1"));
            map.insert("second".to_string(), namer.bucket("bucket2"));
            builder.call_function(
                package_address,
                "Arguments",
                "treemap_argument",
                manifest_args!(map),
            )
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
        .lock_fee_from_faucet()
        .take_all_from_worktop(XRD, "bucket1")
        .take_all_from_worktop(XRD, "bucket2")
        .with_namer(|builder, namer| {
            let mut map = BTreeMap::new();
            map.insert("first".to_string(), namer.bucket("bucket1"));
            map.insert("second".to_string(), namer.bucket("bucket2"));
            builder.call_function(
                package_address,
                "Arguments",
                "hashmap_argument",
                manifest_args!(map),
            )
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
        .lock_fee_from_faucet()
        .take_all_from_worktop(XRD, "bucket1")
        .with_namer(|builder, namer| {
            builder.call_function(
                package_address,
                "Arguments",
                "option_argument",
                manifest_args!(Some(namer.bucket("bucket1"))),
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
        .lock_fee_from_faucet()
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
