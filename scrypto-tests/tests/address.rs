use scrypto::address::{
    AddressError, Bech32Decoder, Bech32Encoder, EntityType, ACCOUNT_COMPONENT_ADDRESS_ENTITY_ID,
    COMPONENT_ADDRESS_ENTITY_ID, PACKAGE_ADDRESS_ENTITY_ID, RESOURCE_ADDRESS_ENTITY_ID,
    SYSTEM_COMPONENT_ADDRESS_ENTITY_ID,
};
use scrypto::core::NetworkDefinition;
use scrypto::prelude::{ComponentAddress, PackageAddress, ResourceAddress};

use bech32::{self, ToBase32, Variant};

fn generate_u8_array(entity_byte: u8) -> [u8; 27] {
    [
        entity_byte,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
    ]
}

// ==============
// Encoder Tests
// ==============

#[test]
fn encode_package_address_correct_entity_type_succeeds() {
    // Arrange
    let package_address = PackageAddress(generate_u8_array(PACKAGE_ADDRESS_ENTITY_ID));
    let bech32_encoder = Bech32Encoder::new(&NetworkDefinition::local_simulator());

    // Act
    let encoded_package_address = bech32_encoder.encode_package_address(&package_address);

    // Assert
    assert!(matches!(encoded_package_address, Ok(_)));
}

#[test]
fn encode_package_address_incorrect_entity_type_fails() {
    // Arrange
    let package_address = PackageAddress(generate_u8_array(RESOURCE_ADDRESS_ENTITY_ID));
    let bech32_encoder = Bech32Encoder::new(&NetworkDefinition::local_simulator());

    // Act
    let encoded_package_address = bech32_encoder.encode_package_address(&package_address);

    // Assert
    assert!(matches!(
        encoded_package_address,
        Err(AddressError::InvalidEntityTypeId(
            RESOURCE_ADDRESS_ENTITY_ID
        ))
    ));
}

#[test]
fn encode_component_address_component_entity_type_succeeds() {
    // Arrange
    let component_address = ComponentAddress(generate_u8_array(COMPONENT_ADDRESS_ENTITY_ID));
    let bech32_encoder = Bech32Encoder::new(&NetworkDefinition::local_simulator());

    // Act
    let encoded_component_address = bech32_encoder.encode_component_address(&component_address);

    // Assert
    assert!(matches!(encoded_component_address, Ok(_)));
}

#[test]
fn encode_component_address_account_component_entity_type_succeeds() {
    // Arrange
    let component_address =
        ComponentAddress(generate_u8_array(ACCOUNT_COMPONENT_ADDRESS_ENTITY_ID));
    let bech32_encoder = Bech32Encoder::new(&NetworkDefinition::local_simulator());

    // Act
    let encoded_component_address = bech32_encoder.encode_component_address(&component_address);

    // Assert
    assert!(matches!(encoded_component_address, Ok(_)));
}

#[test]
fn encode_component_address_system_component_entity_type_succeeds() {
    // Arrange
    let component_address = ComponentAddress(generate_u8_array(SYSTEM_COMPONENT_ADDRESS_ENTITY_ID));
    let bech32_encoder = Bech32Encoder::new(&NetworkDefinition::local_simulator());

    // Act
    let encoded_component_address = bech32_encoder.encode_component_address(&component_address);

    // Assert
    assert!(matches!(encoded_component_address, Ok(_)));
}

#[test]
fn encode_component_address_incorrect_entity_type_fails() {
    // Arrange
    let component_address = ComponentAddress(generate_u8_array(RESOURCE_ADDRESS_ENTITY_ID));
    let bech32_encoder = Bech32Encoder::new(&NetworkDefinition::local_simulator());

    // Act
    let encoded_component_address = bech32_encoder.encode_component_address(&component_address);

    // Assert
    assert!(matches!(
        encoded_component_address,
        Err(AddressError::InvalidEntityTypeId(
            RESOURCE_ADDRESS_ENTITY_ID
        ))
    ));
}

#[test]
fn encode_resource_address_correct_entity_type_succeeds() {
    // Arrange
    let resource_address = ResourceAddress(generate_u8_array(RESOURCE_ADDRESS_ENTITY_ID));
    let bech32_encoder = Bech32Encoder::new(&NetworkDefinition::local_simulator());

    // Act
    let encoded_resource_address = bech32_encoder.encode_resource_address(&resource_address);

    // Assert
    assert!(matches!(encoded_resource_address, Ok(_)));
}

#[test]
fn encode_resource_address_incorrect_entity_type_fails() {
    // Arrange
    let resource_address = ResourceAddress(generate_u8_array(PACKAGE_ADDRESS_ENTITY_ID));
    let bech32_encoder = Bech32Encoder::new(&NetworkDefinition::local_simulator());

    // Act
    let encoded_resource_address = bech32_encoder.encode_resource_address(&resource_address);

    // Assert
    assert!(matches!(
        encoded_resource_address,
        Err(AddressError::InvalidEntityTypeId(PACKAGE_ADDRESS_ENTITY_ID))
    ));
}

// ==============
// Decoder Tests
// ==============

#[test]
fn decode_truncated_checksum_address_fails() {
    // Arrange
    let resource_address = ResourceAddress(generate_u8_array(RESOURCE_ADDRESS_ENTITY_ID));
    let bech32_encoder = Bech32Encoder::new(&NetworkDefinition::local_simulator());
    let bech32_decoder = Bech32Decoder::new(&NetworkDefinition::local_simulator());

    let encoded_resource_address = bech32_encoder
        .encode_resource_address(&resource_address)
        .unwrap();

    // Act
    let decoded_resource_address = bech32_decoder.validate_and_decode_resource_address(
        &encoded_resource_address[..encoded_resource_address.len() - 2],
    );

    // Assert
    assert!(matches!(
        decoded_resource_address,
        Err(AddressError::DecodingError(_))
    ));
}

#[test]
fn decode_modified_checksum_address_fails() {
    // Arrange
    let resource_address = ResourceAddress(generate_u8_array(RESOURCE_ADDRESS_ENTITY_ID));
    let bech32_encoder = Bech32Encoder::new(&NetworkDefinition::local_simulator());
    let bech32_decoder = Bech32Decoder::new(&NetworkDefinition::local_simulator());

    let mut encoded_resource_address = bech32_encoder
        .encode_resource_address(&resource_address)
        .unwrap();

    // Act
    encoded_resource_address.push_str("qq");
    let decoded_resource_address =
        bech32_decoder.validate_and_decode_resource_address(&encoded_resource_address);

    // Assert
    assert!(matches!(
        decoded_resource_address,
        Err(AddressError::DecodingError(_))
    ));
}

/// Tests if the decoding fails when the address is encoded in Bech32 and not Bech32m
#[test]
fn decode_invalid_bech32_variant_fails() {
    // Arrange
    let resource_address = ResourceAddress(generate_u8_array(RESOURCE_ADDRESS_ENTITY_ID));
    let bech32_encoder = Bech32Encoder::new(&NetworkDefinition::local_simulator());
    let bech32_decoder = Bech32Decoder::new(&NetworkDefinition::local_simulator());

    // Act
    let encoded_resource_address = bech32::encode(
        bech32_encoder.hrp_set.get_entity_hrp(&EntityType::Resource),
        resource_address.0.to_base32(),
        Variant::Bech32,
    )
    .unwrap();

    let decoded_resource_address =
        bech32_decoder.validate_and_decode_resource_address(&encoded_resource_address);

    // Assert
    assert!(matches!(
        decoded_resource_address,
        Err(AddressError::InvalidVariant(Variant::Bech32))
    ));
}

#[test]
fn decode_matching_package_address_entity_id_succeeds() {
    // Arrange
    let bech32_encoder = Bech32Encoder::new(&NetworkDefinition::local_simulator());
    let bech32_decoder = Bech32Decoder::new(&NetworkDefinition::local_simulator());

    // Act
    let encoded_package_address = bech32::encode(
        bech32_encoder.hrp_set.get_entity_hrp(&EntityType::Package),
        generate_u8_array(PACKAGE_ADDRESS_ENTITY_ID).to_base32(),
        Variant::Bech32m,
    )
    .unwrap();

    let decoded_package_address =
        bech32_decoder.validate_and_decode_package_address(&encoded_package_address);

    // Assert
    assert!(matches!(decoded_package_address, Ok(_)));
}

#[test]
fn decode_matching_account_address_entity_id_succeeds() {
    // Arrange
    let bech32_encoder = Bech32Encoder::new(&NetworkDefinition::local_simulator());
    let bech32_decoder = Bech32Decoder::new(&NetworkDefinition::local_simulator());

    // Act
    let encoded_account_address = bech32::encode(
        bech32_encoder
            .hrp_set
            .get_entity_hrp(&EntityType::AccountComponent),
        generate_u8_array(ACCOUNT_COMPONENT_ADDRESS_ENTITY_ID).to_base32(),
        Variant::Bech32m,
    )
    .unwrap();

    let decoded_account_address =
        bech32_decoder.validate_and_decode_component_address(&encoded_account_address);

    // Assert
    assert!(matches!(decoded_account_address, Ok(_)));
}

#[test]
fn decode_matching_system_address_entity_id_succeeds() {
    // Arrange
    let bech32_encoder = Bech32Encoder::new(&NetworkDefinition::local_simulator());
    let bech32_decoder = Bech32Decoder::new(&NetworkDefinition::local_simulator());

    // Act
    let encoded_system_address = bech32::encode(
        bech32_encoder
            .hrp_set
            .get_entity_hrp(&EntityType::SystemComponent),
        generate_u8_array(SYSTEM_COMPONENT_ADDRESS_ENTITY_ID).to_base32(),
        Variant::Bech32m,
    )
    .unwrap();

    let decoded_system_address =
        bech32_decoder.validate_and_decode_component_address(&encoded_system_address);

    // Assert
    assert!(matches!(decoded_system_address, Ok(_)));
}

#[test]
fn decode_matching_component_address_entity_id_succeeds() {
    // Arrange
    let bech32_encoder = Bech32Encoder::new(&NetworkDefinition::local_simulator());
    let bech32_decoder = Bech32Decoder::new(&NetworkDefinition::local_simulator());

    // Act
    let encoded_component_address = bech32::encode(
        bech32_encoder
            .hrp_set
            .get_entity_hrp(&EntityType::Component),
        generate_u8_array(COMPONENT_ADDRESS_ENTITY_ID).to_base32(),
        Variant::Bech32m,
    )
    .unwrap();

    let decoded_component_address =
        bech32_decoder.validate_and_decode_component_address(&encoded_component_address);

    // Assert
    assert!(matches!(decoded_component_address, Ok(_)));
}

#[test]
fn decode_mismatched_package_address_entity_id_fails() {
    // Arrange
    let bech32_encoder = Bech32Encoder::new(&NetworkDefinition::local_simulator());
    let bech32_decoder = Bech32Decoder::new(&NetworkDefinition::local_simulator());

    // Act
    let encoded_package_address = bech32::encode(
        bech32_encoder.hrp_set.get_entity_hrp(&EntityType::Package),
        generate_u8_array(RESOURCE_ADDRESS_ENTITY_ID).to_base32(),
        Variant::Bech32m,
    )
    .unwrap();

    let decoded_package_address =
        bech32_decoder.validate_and_decode_package_address(&encoded_package_address);

    // Assert
    assert!(matches!(
        decoded_package_address,
        Err(AddressError::InvalidEntityTypeId(
            RESOURCE_ADDRESS_ENTITY_ID
        ))
    ));
}

#[test]
fn decode_matching_resource_address_entity_id_succeeds() {
    // Arrange
    let bech32_encoder = Bech32Encoder::new(&NetworkDefinition::local_simulator());
    let bech32_decoder = Bech32Decoder::new(&NetworkDefinition::local_simulator());

    // Act
    let encoded_resource_address = bech32::encode(
        bech32_encoder.hrp_set.get_entity_hrp(&EntityType::Resource),
        generate_u8_array(RESOURCE_ADDRESS_ENTITY_ID).to_base32(),
        Variant::Bech32m,
    )
    .unwrap();

    let decoded_resource_address =
        bech32_decoder.validate_and_decode_resource_address(&encoded_resource_address);

    // Assert
    assert!(matches!(decoded_resource_address, Ok(_)));
}

#[test]
fn decode_mismatched_resource_address_entity_id_fails() {
    // Arrange
    let bech32_encoder = Bech32Encoder::new(&NetworkDefinition::local_simulator());
    let bech32_decoder = Bech32Decoder::new(&NetworkDefinition::local_simulator());

    // Act
    let encoded_resource_address = bech32::encode(
        bech32_encoder.hrp_set.get_entity_hrp(&EntityType::Resource),
        generate_u8_array(PACKAGE_ADDRESS_ENTITY_ID).to_base32(),
        Variant::Bech32m,
    )
    .unwrap();

    let decoded_resource_address =
        bech32_decoder.validate_and_decode_resource_address(&encoded_resource_address);

    // Assert
    assert!(matches!(
        decoded_resource_address,
        Err(AddressError::InvalidEntityTypeId(PACKAGE_ADDRESS_ENTITY_ID))
    ));
}

#[test]
fn decode_invalid_entity_specifier_fails() {
    // Arrange
    let bech32_encoder = Bech32Encoder::new(&NetworkDefinition::local_simulator());
    let bech32_decoder = Bech32Decoder::new(&NetworkDefinition::local_simulator());

    // Act
    let encoded_resource_address = bech32::encode(
        bech32_encoder.hrp_set.get_entity_hrp(&EntityType::Resource),
        generate_u8_array(PACKAGE_ADDRESS_ENTITY_ID).to_base32(),
        Variant::Bech32m,
    )
    .unwrap();

    let decoded_resource_address =
        bech32_decoder.validate_and_decode_resource_address(&encoded_resource_address);

    // Assert
    assert!(matches!(decoded_resource_address, Err(_)));
}

#[test]
fn decode_invalid_network_specifier_fails() {
    // Arrange
    let bech32_encoder = Bech32Encoder::new(&NetworkDefinition::mainnet());
    let bech32_decoder = Bech32Decoder::new(&NetworkDefinition::local_simulator());

    // Act
    let encoded_resource_address = bech32::encode(
        bech32_encoder.hrp_set.get_entity_hrp(&EntityType::Resource),
        generate_u8_array(RESOURCE_ADDRESS_ENTITY_ID).to_base32(),
        Variant::Bech32m,
    )
    .unwrap();

    let decoded_resource_address =
        bech32_decoder.validate_and_decode_resource_address(&encoded_resource_address);

    // Assert
    assert!(matches!(decoded_resource_address, Err(_)));
}
