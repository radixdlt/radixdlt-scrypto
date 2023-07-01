use radix_engine::errors::{ApplicationError, RuntimeError, SystemError};
use radix_engine::system::node_modules::metadata::MetadataPanicError;
use radix_engine::types::*;
use radix_engine_interface::api::node_modules::metadata::MetadataValue;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn can_get_from_scrypto() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("../assets/blueprints/metadata");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .call_function(package_address, "MetadataTest", "new", manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let component_address = receipt.expect_commit(true).new_component_addresses()[0];
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .call_method(
            component_address,
            "set_array",
            manifest_args!("key", vec![GlobalAddress::from(RADIX_TOKEN)]),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();

    // Assert
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .call_method(component_address, "get_array", manifest_args!("key"))
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let x: Vec<GlobalAddress> = receipt.expect_commit_success().output(1);
    assert_eq!(x, vec![GlobalAddress::from(RADIX_TOKEN)])
}

#[test]
fn can_set_from_scrypto() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("../assets/blueprints/metadata");
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .call_function(package_address, "MetadataTest", "new", manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let component_address = receipt.expect_commit(true).new_component_addresses()[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .call_method(
            component_address,
            "set_array",
            manifest_args!("key", vec![GlobalAddress::from(RADIX_TOKEN)]),
        )
        .build();

    // Assert
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();
}

#[test]
fn cannot_initialize_metadata_if_key_too_long() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("../assets/blueprints/metadata");

    // Act
    let key = "a".repeat(DEFAULT_MAX_METADATA_KEY_STRING_LEN + 1);
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
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
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("../assets/blueprints/metadata");
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .call_function(package_address, "MetadataTest", "new", manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let component_address = receipt.expect_commit(true).new_component_addresses()[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .set_metadata(
            component_address,
            "a".repeat(DEFAULT_MAX_METADATA_KEY_STRING_LEN + 1),
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
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("../assets/blueprints/metadata");

    // Act
    let value = "a".repeat(DEFAULT_MAX_METADATA_VALUE_SBOR_LEN + 1);
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
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
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("../assets/blueprints/metadata");
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .call_function(package_address, "MetadataTest", "new", manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let component_address = receipt.expect_commit(true).new_component_addresses()[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .set_metadata(
            component_address,
            "a",
            MetadataValue::String("a".repeat(DEFAULT_MAX_METADATA_VALUE_SBOR_LEN + 1)),
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
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("../assets/blueprints/metadata");
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .call_function(package_address, "MetadataTest", "new", manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let component_address = receipt.expect_commit(true).new_component_addresses()[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
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
