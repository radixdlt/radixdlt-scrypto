use radix_engine::errors::{ApplicationError, RuntimeError, SystemError};
use radix_engine::system::attached_modules::metadata::MetadataError;
use radix_engine::types::*;
use radix_engine_interface::api::node_modules::metadata::{
    MetadataConversionError::UnexpectedType, MetadataValue,
};
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
    let component_address = receipt.expect_commit(true).new_component_addresses()[0];
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
    let component_address = receipt.expect_commit(true).new_component_addresses()[0];

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
                MetadataError::KeyStringExceedsMaxLength { .. }
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
    let component_address = receipt.expect_commit(true).new_component_addresses()[0];

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
                MetadataError::KeyStringExceedsMaxLength { .. }
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
                MetadataError::ValueSborExceedsMaxLength { .. }
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
    let component_address = receipt.expect_commit(true).new_component_addresses()[0];

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
                MetadataError::ValueSborExceedsMaxLength { .. }
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
    let component_address = receipt.expect_commit(true).new_component_addresses()[0];

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

#[test]
fn verify_metadata_set_and_get_fail() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let proof = NonFungibleGlobalId::from_public_key(&public_key);

    // add String metadata
    test_runner.set_metadata(account.into(), "key", "value", proof);

    // Act
    let metadata = test_runner.get_metadata(account.into(), "key").unwrap();

    assert!(String::from_metadata_value(metadata.clone()).is_ok());

    let result1 = u8::from_metadata_value(metadata.clone());
    let result2 = Vec::<u8>::from_metadata_value(metadata.clone());

    // Assert
    assert_eq!(
        result1,
        Err(UnexpectedType {
            expected_type_id: 2,
            actual_type_id: 0
        })
    );

    assert_eq!(
        result2,
        Err(UnexpectedType {
            expected_type_id: 130,
            actual_type_id: 0
        })
    );
}

#[test]
fn verify_metadata_array_set_and_get_fail() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let proof = NonFungibleGlobalId::from_public_key(&public_key);

    let value = [10u8; 10].as_ref().to_metadata_entry().unwrap();

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .set_metadata(account, String::from("key"), value)
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![proof]);
    receipt.expect_commit_success();

    // Act
    let metadata = test_runner.get_metadata(account.into(), "key").unwrap();

    assert!(Vec::<u8>::from_metadata_value(metadata.clone()).is_ok());

    let result1 = u8::from_metadata_value(metadata.clone());
    let result2 = Vec::<u32>::from_metadata_value(metadata.clone());
    let result3 = u32::from_array_metadata_value(metadata.clone());

    // Assert
    assert_eq!(
        result1,
        Err(UnexpectedType {
            expected_type_id: 2,
            actual_type_id: 130
        })
    );

    assert_eq!(
        result2,
        Err(UnexpectedType {
            expected_type_id: 131,
            actual_type_id: 130
        })
    );

    assert_eq!(
        result3,
        Err(UnexpectedType {
            expected_type_id: 131,
            actual_type_id: 130
        })
    );

    let v = [0u8; 10];
    assert!((&v).to_metadata_entry().is_some());

    let v: Vec<ComponentAddress> = vec![ComponentAddress::new_or_panic([192u8; NodeId::LENGTH])];
    assert!(v.to_metadata_entry().is_some());

    let v: Vec<ResourceAddress> = vec![ResourceAddress::new_or_panic([93u8; NodeId::LENGTH])];
    assert!(v.to_metadata_entry().is_some());

    let v: Vec<PackageAddress> = vec![PackageAddress::new_or_panic([13u8; NodeId::LENGTH])];
    assert!(v.as_slice().to_metadata_entry().is_some());

    let v = [PackageAddress::new_or_panic([13u8; NodeId::LENGTH])];
    assert!((&v).to_metadata_entry().is_some());
}
