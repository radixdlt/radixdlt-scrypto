use radix_common::prelude::*;
use radix_engine::errors::{ApplicationError, RuntimeError, SystemError};
use radix_engine::object_modules::metadata::{
    MetadataError, MetadataKeyValidationError, MetadataValueValidationError,
};
use radix_engine_interface::object_modules::metadata::{
    MetadataConversionError::UnexpectedType, MetadataValue,
};
use radix_engine_tests::common::*;
use scrypto_test::prelude::*;

fn publish_metadata_package(
    ledger: &mut LedgerSimulator<NoExtension, InMemorySubstateDatabase>,
) -> PackageAddress {
    let code =
        include_workspace_asset_bytes!("radix-transaction-scenarios", "metadata.wasm").to_vec();
    let package_def = manifest_decode::<PackageDefinition>(include_workspace_asset_bytes!(
        "radix-transaction-scenarios",
        "metadata.rpd"
    ))
    .unwrap();
    ledger.publish_package_simple((code, package_def))
}

#[test]
fn can_get_from_scrypto() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = publish_metadata_package(&mut ledger);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(package_address, "MetadataTest", "new", manifest_args!())
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    let component_address = receipt.expect_commit(true).new_component_addresses()[0];
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            component_address,
            "set_array",
            manifest_args!("key", vec![GlobalAddress::from(XRD)]),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();

    // Assert
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(component_address, "get_array", manifest_args!("key"))
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    let x: Vec<GlobalAddress> = receipt.expect_commit_success().output(1);
    assert_eq!(x, vec![GlobalAddress::from(XRD)])
}

#[test]
fn can_set_from_scrypto() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = publish_metadata_package(&mut ledger);
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(package_address, "MetadataTest", "new", manifest_args!())
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
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
    let receipt = ledger.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();
}

#[test]
fn cannot_initialize_metadata_if_key_too_long() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = publish_metadata_package(&mut ledger);

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
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::MetadataError(
                MetadataError::MetadataKeyValidationError(
                    MetadataKeyValidationError::InvalidLength { .. }
                )
            ))
        )
    });
}

#[test]
fn cannot_set_metadata_if_key_too_long() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = publish_metadata_package(&mut ledger);
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(package_address, "MetadataTest", "new", manifest_args!())
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
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
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::MetadataError(
                MetadataError::MetadataKeyValidationError(
                    MetadataKeyValidationError::InvalidLength { .. }
                )
            ))
        )
    });
}

#[test]
fn cannot_initialize_metadata_if_value_too_long() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = publish_metadata_package(&mut ledger);

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
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::MetadataError(
                MetadataError::MetadataValueValidationError(
                    MetadataValueValidationError::InvalidLength { .. }
                )
            ))
        )
    });
}

#[test]
fn cannot_set_metadata_if_value_too_long() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = publish_metadata_package(&mut ledger);
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(package_address, "MetadataTest", "new", manifest_args!())
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
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
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::MetadataError(
                MetadataError::MetadataValueValidationError(
                    MetadataValueValidationError::InvalidLength { .. }
                )
            ))
        )
    });
}

#[test]
fn cannot_set_metadata_if_initialized_empty_locked() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = publish_metadata_package(&mut ledger);
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(package_address, "MetadataTest", "new", manifest_args!())
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    let component_address = receipt.expect_commit(true).new_component_addresses()[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .set_metadata(component_address, "empty_locked", MetadataValue::Bool(true))
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
fn verify_metadata_set_and_get_success() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let proof = NonFungibleGlobalId::from_public_key(&public_key);

    // add String metadata
    ledger.set_metadata(account.into(), "key", "value", proof);

    // Act
    let metadata = ledger.get_metadata(account.into(), "key").unwrap();

    // Assert
    assert!(String::from_metadata_value(metadata).is_ok());
}

#[test]
fn verify_metadata_get_fail() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let proof = NonFungibleGlobalId::from_public_key(&public_key);

    // add String metadata
    ledger.set_metadata(account.into(), "key", "value", proof);

    // Act
    let metadata = ledger.get_metadata(account.into(), "key").unwrap();

    let result = u8::from_metadata_value(metadata);

    // Assert
    assert_eq!(
        result,
        Err(UnexpectedType {
            expected_type_id: 2,
            actual_type_id: 0
        })
    );
}

#[test]
fn verify_metadata_vec_get_fail() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let proof = NonFungibleGlobalId::from_public_key(&public_key);

    // add String metadata
    ledger.set_metadata(account.into(), "key", "value", proof);

    // Act
    let metadata = ledger.get_metadata(account.into(), "key").unwrap();

    let result = Vec::<u8>::from_metadata_value(metadata);

    // Assert
    assert_eq!(
        result,
        Err(UnexpectedType {
            expected_type_id: 130,
            actual_type_id: 0
        })
    );
}

#[test]
fn verify_metadata_array_set_and_get_success() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let proof = NonFungibleGlobalId::from_public_key(&public_key);

    let value = [10u8; 10].as_ref().to_metadata_entry().unwrap();

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .set_metadata(account, String::from("key"), value)
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![proof]);
    receipt.expect_commit_success();

    // Act
    let metadata = ledger.get_metadata(account.into(), "key").unwrap();

    // Assert
    assert!(Vec::<u8>::from_metadata_value(metadata).is_ok());
}

#[test]
fn verify_metadata_array_get_fail() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let proof = NonFungibleGlobalId::from_public_key(&public_key);

    let value = [10u8; 10].as_ref().to_metadata_entry().unwrap();

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .set_metadata(account, String::from("key"), value)
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![proof]);
    receipt.expect_commit_success();

    // Act
    let metadata = ledger.get_metadata(account.into(), "key").unwrap();

    let result = u8::from_metadata_value(metadata);

    // Assert
    assert_eq!(
        result,
        Err(UnexpectedType {
            expected_type_id: 2,
            actual_type_id: 130
        })
    );
}

#[test]
fn verify_metadata_array_get_other_type_fail() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let proof = NonFungibleGlobalId::from_public_key(&public_key);

    let value = [10u8; 10].as_ref().to_metadata_entry().unwrap();

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .set_metadata(account, String::from("key"), value)
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![proof]);
    receipt.expect_commit_success();

    // Act
    let metadata = ledger.get_metadata(account.into(), "key").unwrap();

    let result = u32::from_array_metadata_value(metadata);

    // Assert
    assert_eq!(
        result,
        Err(UnexpectedType {
            expected_type_id: 131,
            actual_type_id: 130
        })
    );
}

#[test]
fn verify_metadata_array_get_vec_fail() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let proof = NonFungibleGlobalId::from_public_key(&public_key);

    let value = [10u8; 10].as_ref().to_metadata_entry().unwrap();

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .set_metadata(account, String::from("key"), value)
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![proof]);
    receipt.expect_commit_success();

    // Act
    let metadata = ledger.get_metadata(account.into(), "key").unwrap();

    let result = Vec::<u32>::from_metadata_value(metadata);

    // Assert
    assert_eq!(
        result,
        Err(UnexpectedType {
            expected_type_id: 131,
            actual_type_id: 130
        })
    );
}

#[test]
fn verify_metadata_conversion_from_various_array_and_vector_types() {
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

/// Given some value, we encode it through its [`MetadataVal`] encoding and then through its
/// [`MetadataValue`] encoding and check if both are equal. This is to detect if [`MetadataVal`]
/// is using an incorrect discriminator.
#[test]
fn encoding_metadata_through_metadata_val_is_the_same_as_metadata_value() {
    assert_metadata_val_encoding("Hello World!".to_owned());
    assert_metadata_val_encoding(false);
    assert_metadata_val_encoding(1u8);
    assert_metadata_val_encoding(1u32);
    assert_metadata_val_encoding(1u64);
    assert_metadata_val_encoding(1i32);
    assert_metadata_val_encoding(1i64);
    assert_metadata_val_encoding(dec!(1));
    assert_metadata_val_encoding(GlobalAddress::from(XRD));
    assert_metadata_val_encoding(PublicKey::Ed25519(Ed25519PublicKey(
        [0; Ed25519PublicKey::LENGTH],
    )));
    assert_metadata_val_encoding(PublicKey::Secp256k1(Secp256k1PublicKey(
        [0; Secp256k1PublicKey::LENGTH],
    )));
    assert_metadata_val_encoding(NonFungibleGlobalId::from_public_key(&Secp256k1PublicKey(
        [0; Secp256k1PublicKey::LENGTH],
    )));
    assert_metadata_val_encoding(NonFungibleGlobalId::from_public_key(&Ed25519PublicKey(
        [0; Ed25519PublicKey::LENGTH],
    )));
    assert_metadata_val_encoding(NonFungibleLocalId::integer(1));
    assert_metadata_val_encoding(NonFungibleLocalId::string("HelloWorld").unwrap());
    assert_metadata_val_encoding(NonFungibleLocalId::bytes(*b"HelloWorld").unwrap());
    assert_metadata_val_encoding(NonFungibleLocalId::ruid([0x11; 32]));
    assert_metadata_val_encoding(Instant::new(100));
    assert_metadata_val_encoding(UncheckedUrl("https://www.google.com/".into()));
    assert_metadata_val_encoding(UncheckedOrigin("https://www.google.com/".into()));
    assert_metadata_val_encoding(PublicKeyHash::Ed25519(Ed25519PublicKeyHash(
        [0; Ed25519PublicKeyHash::LENGTH],
    )));
    assert_metadata_val_encoding(PublicKeyHash::Secp256k1(Secp256k1PublicKeyHash(
        [0; Secp256k1PublicKeyHash::LENGTH],
    )));

    assert_metadata_val_encoding(vec!["Hello World!".to_owned(), "Hello World!".to_owned()]);
    assert_metadata_val_encoding(vec![false, false]);
    assert_metadata_val_encoding(vec![1u8, 1u8]);
    assert_metadata_val_encoding(vec![1u32, 1u32]);
    assert_metadata_val_encoding(vec![1u64, 1u64]);
    assert_metadata_val_encoding(vec![1i32, 1i32]);
    assert_metadata_val_encoding(vec![1i64, 1i64]);
    assert_metadata_val_encoding(vec![dec!(1), dec!(1)]);
    assert_metadata_val_encoding(vec![GlobalAddress::from(XRD), GlobalAddress::from(XRD)]);
    assert_metadata_val_encoding(vec![
        PublicKey::Ed25519(Ed25519PublicKey([0; Ed25519PublicKey::LENGTH])),
        PublicKey::Ed25519(Ed25519PublicKey([0; Ed25519PublicKey::LENGTH])),
    ]);
    assert_metadata_val_encoding(vec![
        PublicKey::Secp256k1(Secp256k1PublicKey([0; Secp256k1PublicKey::LENGTH])),
        PublicKey::Secp256k1(Secp256k1PublicKey([0; Secp256k1PublicKey::LENGTH])),
    ]);
    assert_metadata_val_encoding(vec![
        NonFungibleGlobalId::from_public_key(&Secp256k1PublicKey([0; Secp256k1PublicKey::LENGTH])),
        NonFungibleGlobalId::from_public_key(&Secp256k1PublicKey([0; Secp256k1PublicKey::LENGTH])),
    ]);
    assert_metadata_val_encoding(vec![
        NonFungibleGlobalId::from_public_key(&Ed25519PublicKey([0; Ed25519PublicKey::LENGTH])),
        NonFungibleGlobalId::from_public_key(&Ed25519PublicKey([0; Ed25519PublicKey::LENGTH])),
    ]);
    assert_metadata_val_encoding(vec![
        NonFungibleLocalId::integer(1),
        NonFungibleLocalId::integer(1),
    ]);
    assert_metadata_val_encoding(vec![
        NonFungibleLocalId::string("HelloWorld").unwrap(),
        NonFungibleLocalId::string("HelloWorld").unwrap(),
    ]);
    assert_metadata_val_encoding(vec![
        NonFungibleLocalId::bytes(*b"HelloWorld").unwrap(),
        NonFungibleLocalId::bytes(*b"HelloWorld").unwrap(),
    ]);
    assert_metadata_val_encoding(vec![
        NonFungibleLocalId::ruid([0x11; 32]),
        NonFungibleLocalId::ruid([0x11; 32]),
    ]);
    assert_metadata_val_encoding(vec![Instant::new(100), Instant::new(100)]);
    assert_metadata_val_encoding(vec![
        UncheckedUrl("https://www.google.com/".into()),
        UncheckedUrl("https://www.google.com/".into()),
    ]);
    assert_metadata_val_encoding(vec![
        UncheckedOrigin("https://www.google.com/".into()),
        UncheckedOrigin("https://www.google.com/".into()),
    ]);
    assert_metadata_val_encoding(vec![
        PublicKeyHash::Ed25519(Ed25519PublicKeyHash([0; Ed25519PublicKeyHash::LENGTH])),
        PublicKeyHash::Ed25519(Ed25519PublicKeyHash([0; Ed25519PublicKeyHash::LENGTH])),
    ]);
    assert_metadata_val_encoding(vec![
        PublicKeyHash::Secp256k1(Secp256k1PublicKeyHash([0; Secp256k1PublicKeyHash::LENGTH])),
        PublicKeyHash::Secp256k1(Secp256k1PublicKeyHash([0; Secp256k1PublicKeyHash::LENGTH])),
    ]);
}

fn assert_metadata_val_encoding<T>(item: T)
where
    T: MetadataVal + Debug + Clone,
{
    assert_eq!(
        metadata_val_encode(&item),
        scrypto_encode(&item.clone().to_metadata_value()).unwrap(),
        "Encoding is not the same: {item:#?}"
    )
}

fn metadata_val_encode<T>(value: &T) -> Vec<u8>
where
    T: MetadataVal,
{
    let mut buffer = Vec::new();
    let mut encoder =
        VecEncoder::<ScryptoCustomValueKind>::new(&mut buffer, SCRYPTO_SBOR_V1_MAX_DEPTH);
    encoder
        .write_payload_prefix(SCRYPTO_SBOR_V1_PAYLOAD_PREFIX)
        .unwrap();
    encoder.write_value_kind(ValueKind::Enum).unwrap();
    encoder.write_discriminator(T::DISCRIMINATOR).unwrap();
    encoder.write_size(1).unwrap();
    encoder.encode(&value).unwrap();
    buffer
}
