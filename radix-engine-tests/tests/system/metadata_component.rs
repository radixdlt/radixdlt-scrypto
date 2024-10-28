use radix_common::prelude::*;
use radix_engine::{
    errors::{ApplicationError, RuntimeError, SystemError},
    object_modules::metadata::{MetadataError, MetadataValueValidationError},
};
use radix_engine_tests::common::*;
use scrypto_test::prelude::*;

#[test]
fn cannot_create_metadata_with_invalid_value() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("metadata_component"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "MetadataComponent",
            "create_metadata_with_invalid_url",
            manifest_args!(),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::MetadataError(
                MetadataError::MetadataValueValidationError(
                    MetadataValueValidationError::InvalidURL(_)
                )
            ))
        )
    });
}

#[test]
fn cannot_set_metadata_with_invalid_value() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("metadata_component"));
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "MetadataComponent",
            "new2",
            manifest_args!("key".to_string(), "value".to_string()),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    let component_address = receipt.expect_commit(true).new_component_addresses()[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "MetadataComponent",
            "set_metadata_with_invalid_url",
            manifest_args!(component_address, "key".to_string()),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::MetadataError(
                MetadataError::MetadataValueValidationError(
                    MetadataValueValidationError::InvalidURL(_)
                )
            ))
        )
    });

    // Act 2
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "MetadataComponent",
            "set_metadata_with_invalid_origin",
            manifest_args!(component_address, "key".to_string()),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert 2
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::MetadataError(
                MetadataError::MetadataValueValidationError(
                    MetadataValueValidationError::InvalidOrigin(_)
                )
            ))
        )
    });
}

#[test]
fn can_globalize_with_component_metadata() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("metadata_component"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "MetadataComponent",
            "new",
            manifest_args!("key".to_string(), "value".to_string()),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    let component_address = receipt.expect_commit(true).new_component_addresses()[0];

    // Assert
    let value = ledger
        .get_metadata(component_address.into(), "key")
        .expect("Should exist");
    assert_eq!(value, MetadataValue::String("value".to_string()));
}

#[test]
fn can_set_metadata_after_globalized() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("metadata_component"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "MetadataComponent",
            "new2",
            manifest_args!("key".to_string(), "value".to_string()),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    let component_address = receipt.expect_commit(true).new_component_addresses()[0];

    // Assert
    receipt.expect_commit_success();
    let value = ledger
        .get_metadata(component_address.into(), "key")
        .expect("Should exist");
    assert_eq!(value, MetadataValue::String("value".to_string()));
}

#[test]
fn can_remove_metadata() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("metadata_component"));
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "MetadataComponent",
            "new2",
            manifest_args!("key".to_string(), "value".to_string()),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    let component_address = receipt.expect_commit(true).new_component_addresses()[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "MetadataComponent",
            "remove_metadata",
            manifest_args!(component_address, "key".to_string()),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
    let value = ledger.get_metadata(component_address.into(), "key");
    assert_eq!(value, None);
}

fn can_set_metadata_through_manifest(entry: MetadataValue) {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("metadata_component"));
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "MetadataComponent",
            "new2",
            manifest_args!("key".to_string(), "value".to_string()),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    let component_address = receipt.expect_commit(true).new_component_addresses()[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .set_metadata(component_address, "key".to_string(), entry.clone())
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
    let stored_entry = ledger
        .get_metadata(component_address.into(), "key")
        .expect("Should exist");
    assert_eq!(stored_entry, entry);
}

#[test]
fn can_set_string_metadata_through_manifest() {
    can_set_metadata_through_manifest(MetadataValue::String("Test".to_string()));
}

#[test]
fn can_set_boolean_metadata_through_manifest() {
    can_set_metadata_through_manifest(MetadataValue::Bool(true));
}

#[test]
fn can_set_u8_metadata_through_manifest() {
    can_set_metadata_through_manifest(MetadataValue::U8(1u8));
}

#[test]
fn can_set_u32_metadata_through_manifest() {
    can_set_metadata_through_manifest(MetadataValue::U32(1u32));
}

#[test]
fn can_set_u64_metadata_through_manifest() {
    can_set_metadata_through_manifest(MetadataValue::U64(1u64));
}

#[test]
fn can_set_i32_metadata_through_manifest() {
    can_set_metadata_through_manifest(MetadataValue::I32(1i32));
}

#[test]
fn can_set_i64_metadata_through_manifest() {
    can_set_metadata_through_manifest(MetadataValue::I64(1i64));
}

#[test]
fn can_set_decimal_metadata_through_manifest() {
    can_set_metadata_through_manifest(MetadataValue::Decimal(Decimal::one()));
}

#[test]
fn can_set_address_metadata_through_manifest() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("metadata_component"));
    let key = Secp256k1PrivateKey::from_u64(1u64).unwrap().public_key();
    let address = ledger
        .create_non_fungible_resource(ComponentAddress::preallocated_account_from_public_key(&key));
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "MetadataComponent",
            "new2",
            manifest_args!("key".to_string(), "value".to_string()),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    let component_address = receipt.expect_commit(true).new_component_addresses()[0];

    // Act
    let entry = MetadataValue::GlobalAddress(address.into());
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .set_metadata(component_address, "key".to_string(), entry.clone())
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
    let stored_entry = ledger
        .get_metadata(component_address.into(), "key")
        .expect("Should exist");
    assert_eq!(stored_entry, entry);

    can_set_metadata_through_manifest(MetadataValue::GlobalAddress(XRD.into()));
}

#[test]
fn cannot_set_address_metadata_after_freezing() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("metadata_component"));
    let key = Secp256k1PrivateKey::from_u64(1u64).unwrap().public_key();
    let address = ledger
        .create_non_fungible_resource(ComponentAddress::preallocated_account_from_public_key(&key));
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "MetadataComponent",
            "new2",
            manifest_args!("key".to_string(), "value".to_string()),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    let component_address = receipt.expect_commit(true).new_component_addresses()[0];
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .lock_metadata(component_address, "other_key")
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();

    // Act
    let entry = MetadataValue::GlobalAddress(address.into());
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .set_metadata(component_address, "other_key", entry)
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemError(SystemError::KeyValueEntryLocked)
        )
    });
}

#[test]
fn can_set_public_key_metadata_through_manifest() {
    can_set_metadata_through_manifest(MetadataValue::PublicKey(
        Secp256k1PrivateKey::from_u64(1u64)
            .unwrap()
            .public_key()
            .into(),
    ));
}

#[test]
fn can_set_instant_metadata_through_manifest() {
    can_set_metadata_through_manifest(MetadataValue::Instant(Instant {
        seconds_since_unix_epoch: 51,
    }));
}

#[test]
fn can_set_url_metadata_through_manifest() {
    can_set_metadata_through_manifest(MetadataValue::Url(UncheckedUrl::of(
        "https://radixdlt.com/index.html",
    )));
}

#[test]
fn can_set_origin_metadata_through_manifest() {
    can_set_metadata_through_manifest(MetadataValue::Origin(UncheckedOrigin::of(
        "https://radixdlt.com",
    )));
}

#[test]
fn can_set_public_key_hash_metadata_through_manifest() {
    can_set_metadata_through_manifest(MetadataValue::PublicKeyHash(PublicKeyHash::Secp256k1(
        Secp256k1PublicKeyHash([0; 29]),
    )));
}

#[test]
fn can_set_list_metadata_through_manifest() {
    can_set_metadata_through_manifest(MetadataValue::BoolArray(vec![true, false]));
}
