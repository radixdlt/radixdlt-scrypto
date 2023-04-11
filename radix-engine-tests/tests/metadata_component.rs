use radix_engine::types::*;
use radix_engine_interface::api::node_modules::metadata::{MetadataEntry, MetadataValue, Url};
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;
use transaction::ecdsa_secp256k1::EcdsaSecp256k1PrivateKey;

#[test]
fn can_globalize_with_component_metadata() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/metadata_component");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_function(
            package_address,
            "MetadataComponent",
            "new",
            manifest_args!("key".to_string(), "value".to_string()),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let component_address = receipt.expect_commit(true).new_component_addresses()[0];

    // Assert
    let value = test_runner
        .get_metadata(component_address.into(), "key")
        .expect("Should exist");
    assert_eq!(
        value,
        MetadataEntry::Value(MetadataValue::String("value".to_string()))
    );
}

#[test]
fn can_set_metadata_after_globalized() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/metadata_component");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_function(
            package_address,
            "MetadataComponent",
            "new2",
            manifest_args!("key".to_string(), "value".to_string()),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let component_address = receipt.expect_commit(true).new_component_addresses()[0];

    // Assert
    receipt.expect_commit_success();
    let value = test_runner
        .get_metadata(component_address.into(), "key")
        .expect("Should exist");
    assert_eq!(
        value,
        MetadataEntry::Value(MetadataValue::String("value".to_string()))
    );
}

#[test]
fn can_remove_metadata() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/metadata_component");
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_function(
            package_address,
            "MetadataComponent",
            "new2",
            manifest_args!("key".to_string(), "value".to_string()),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let component_address = receipt.expect_commit(true).new_component_addresses()[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_function(
            package_address,
            "MetadataComponent",
            "remove_metadata",
            manifest_args!(component_address, "key".to_string()),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
    let value = test_runner.get_metadata(component_address.into(), "key");
    assert_eq!(value, None);
}

fn can_set_metadata_through_manifest(entry: MetadataEntry) {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/metadata_component");
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_function(
            package_address,
            "MetadataComponent",
            "new2",
            manifest_args!("key".to_string(), "value".to_string()),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let component_address = receipt.expect_commit(true).new_component_addresses()[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .set_metadata(component_address.into(), "key".to_string(), entry.clone())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
    let stored_entry = test_runner
        .get_metadata(component_address.into(), "key")
        .expect("Should exist");
    assert_eq!(stored_entry, entry);
}

#[test]
fn can_set_string_metadata_through_manifest() {
    can_set_metadata_through_manifest(MetadataEntry::Value(MetadataValue::String(
        "Test".to_string(),
    )));
}

#[test]
fn can_set_boolean_metadata_through_manifest() {
    can_set_metadata_through_manifest(MetadataEntry::Value(MetadataValue::Bool(true)));
}

#[test]
fn can_set_u8_metadata_through_manifest() {
    can_set_metadata_through_manifest(MetadataEntry::Value(MetadataValue::U8(1u8)));
}

#[test]
fn can_set_u32_metadata_through_manifest() {
    can_set_metadata_through_manifest(MetadataEntry::Value(MetadataValue::U32(1u32)));
}

#[test]
fn can_set_u64_metadata_through_manifest() {
    can_set_metadata_through_manifest(MetadataEntry::Value(MetadataValue::U64(1u64)));
}

#[test]
fn can_set_i32_metadata_through_manifest() {
    can_set_metadata_through_manifest(MetadataEntry::Value(MetadataValue::I32(1i32)));
}

#[test]
fn can_set_i64_metadata_through_manifest() {
    can_set_metadata_through_manifest(MetadataEntry::Value(MetadataValue::I64(1i64)));
}

#[test]
fn can_set_decimal_metadata_through_manifest() {
    can_set_metadata_through_manifest(MetadataEntry::Value(MetadataValue::Decimal(Decimal::one())));
}

#[test]
fn can_set_address_metadata_through_manifest() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/metadata_component");
    let key = EcdsaSecp256k1PrivateKey::from_u64(1u64)
        .unwrap()
        .public_key();
    let address = test_runner
        .create_non_fungible_resource(ComponentAddress::virtual_account_from_public_key(&key));
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_function(
            package_address,
            "MetadataComponent",
            "new2",
            manifest_args!("key".to_string(), "value".to_string()),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let component_address = receipt.expect_commit(true).new_component_addresses()[0];

    // Act
    let entry = MetadataEntry::Value(MetadataValue::Address(address.into()));
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .set_metadata(component_address.into(), "key".to_string(), entry.clone())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
    let stored_entry = test_runner
        .get_metadata(component_address.into(), "key")
        .expect("Should exist");
    assert_eq!(stored_entry, entry);

    can_set_metadata_through_manifest(MetadataEntry::Value(MetadataValue::Address(
        RADIX_TOKEN.into(),
    )));
}

#[test]
fn can_set_public_key_metadata_through_manifest() {
    can_set_metadata_through_manifest(MetadataEntry::Value(MetadataValue::PublicKey(
        EcdsaSecp256k1PrivateKey::from_u64(1u64)
            .unwrap()
            .public_key()
            .into(),
    )));
}

#[test]
fn can_set_url_metadata_through_manifest() {
    can_set_metadata_through_manifest(MetadataEntry::Value(MetadataValue::Url(Url(
        "test".to_string()
    ))));
}

#[test]
fn can_set_list_metadata_through_manifest() {
    can_set_metadata_through_manifest(MetadataEntry::List(vec![
        MetadataValue::Bool(true),
        MetadataValue::Address(RADIX_TOKEN.into()),
        MetadataValue::PublicKey(
            EcdsaSecp256k1PrivateKey::from_u64(1u64)
                .unwrap()
                .public_key()
                .into(),
        ),
    ]));
}
