use radix_engine::errors::{ApplicationError, RuntimeError, SystemError};
use radix_engine::system::node_modules::metadata::MetadataPanicError;
use radix_engine::types::*;
use radix_engine_interface::api::node_modules::metadata::MetadataValue;
use scrypto_unit::*;
use transaction::prelude::*;

#[test]
fn can_get_from_scrypto() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let package_address = test_runner.compile_and_publish("../assets/blueprints/metadata");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(package_address, "MetadataTest", "new", manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let component_address = receipt
        .expect_commit(true)
        .new_component_addresses()[0];
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            component_address,
            "set_array",
            manifest_args!("key", vec![GlobalAddress::from(XRD)]),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();

    // Assert
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(component_address, "get_array", manifest_args!("key"))
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let x: Vec<GlobalAddress> = receipt.expect_commit_success().output(1);
    assert_eq!(x, vec![GlobalAddress::from(XRD)])
}

#[test]
fn can_set_from_scrypto() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let package_address = test_runner.compile_and_publish("../assets/blueprints/metadata");
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(package_address, "MetadataTest", "new", manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let component_address = receipt
        .expect_commit(true)
        .new_component_addresses()[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            component_address,
            "set_array",
            manifest_args!("key", vec![GlobalAddress::from(XRD)]),
        )
        .build();

    // Assert
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();
}

#[test]
fn cannot_initialize_metadata_if_key_too_long() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let package_address = test_runner.compile_and_publish("../assets/blueprints/metadata");

    // Act
    let key = "a".repeat(MAX_METADATA_KEY_STRING_LEN + 1);
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "MetadataTest",
            "new_with_initial_metadata",
            manifest_args!(key, "some_value".to_string()),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::MetadataError(
                MetadataPanicError::KeyStringExceedsMaxLength { .. }
            ))
        )
    });
}

#[test]
fn cannot_set_metadata_if_key_too_long() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let package_address = test_runner.compile_and_publish("../assets/blueprints/metadata");
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(package_address, "MetadataTest", "new", manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let component_address = receipt
        .expect_commit(true)
        .new_component_addresses()[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .set_metadata(
            component_address,
            "a".repeat(MAX_METADATA_KEY_STRING_LEN + 1),
            MetadataValue::Bool(true),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::MetadataError(
                MetadataPanicError::KeyStringExceedsMaxLength { .. }
            ))
        )
    });
}

#[test]
fn cannot_initialize_metadata_if_value_too_long() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let package_address = test_runner.compile_and_publish("../assets/blueprints/metadata");

    // Act
    let value = "a".repeat(MAX_METADATA_VALUE_SBOR_LEN + 1);
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "MetadataTest",
            "new_with_initial_metadata",
            manifest_args!("a".to_string(), value),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::MetadataError(
                MetadataPanicError::ValueSborExceedsMaxLength { .. }
            ))
        )
    });
}

#[test]
fn cannot_set_metadata_if_value_too_long() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let package_address = test_runner.compile_and_publish("../assets/blueprints/metadata");
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(package_address, "MetadataTest", "new", manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let component_address = receipt
        .expect_commit(true)
        .new_component_addresses()[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .set_metadata(
            component_address,
            "a",
            MetadataValue::String("a".repeat(MAX_METADATA_VALUE_SBOR_LEN + 1)),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::MetadataError(
                MetadataPanicError::ValueSborExceedsMaxLength { .. }
            ))
        )
    });
}

#[test]
fn cannot_set_metadata_if_initialized_empty_locked() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let package_address = test_runner.compile_and_publish("../assets/blueprints/metadata");
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(package_address, "MetadataTest", "new", manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let component_address = receipt
        .expect_commit(true)
        .new_component_addresses()[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .set_metadata(component_address, "empty_locked", MetadataValue::Bool(true))
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemError(SystemError::MutatingImmutableSubstate)
        )
    });
}
