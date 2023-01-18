use bech32::{self, ToBase32, Variant};
use scrypto::radix_engine_interface::address::*;
use scrypto::radix_engine_interface::model::*;
use scrypto::radix_engine_interface::node::NetworkDefinition;

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
fn encode_package_address_to_string_correct_entity_type_succeeds() {
    // Arrange
    let package_address = PackageAddress::Normal([0u8; 26]);
    let bech32_encoder = Bech32Encoder::for_simulator();

    // Act
    let bech32 = bech32_encoder.encode_package_address_to_string(&package_address);

    // Assert
    assert!(bech32.starts_with("package"));
}

// Most of encoder tests are removed because entity id is no longer manually filled
// Rust compiler helps ensure all PackageAddress/ComponentAddress/ResourceAddress instances can be encoded.

// ==============
// Decoder Tests
// ==============

#[test]
fn decode_truncated_checksum_address_fails() {
    // Arrange
    let resource_address = ResourceAddress::Normal([0u8; 26]);
    let bech32_encoder = Bech32Encoder::for_simulator();
    let bech32_decoder = Bech32Decoder::for_simulator();

    let encoded_resource_address =
        bech32_encoder.encode_resource_address_to_string(&resource_address);

    // Act
    let decoded_resource_address = bech32_decoder.validate_and_decode_resource_address(
        &encoded_resource_address[..encoded_resource_address.len() - 2],
    );

    // Assert
    assert!(matches!(
        decoded_resource_address,
        Err(AddressError::Bech32mDecodingError(_))
    ));
}

#[test]
fn decode_modified_checksum_address_fails() {
    // Arrange
    let resource_address = ResourceAddress::Normal([0u8; 26]);
    let bech32_encoder = Bech32Encoder::for_simulator();
    let bech32_decoder = Bech32Decoder::for_simulator();

    let mut encoded_resource_address =
        bech32_encoder.encode_resource_address_to_string(&resource_address);

    // Act
    encoded_resource_address.push_str("qq");
    let decoded_resource_address =
        bech32_decoder.validate_and_decode_resource_address(&encoded_resource_address);

    // Assert
    assert!(matches!(
        decoded_resource_address,
        Err(AddressError::Bech32mDecodingError(_))
    ));
}

/// Tests if the decoding fails when the address is encoded in Bech32 and not Bech32m
#[test]
fn decode_invalid_bech32_variant_fails() {
    // Arrange
    let resource_address = ResourceAddress::Normal([0u8; 26]);
    let bech32_encoder = Bech32Encoder::for_simulator();
    let bech32_decoder = Bech32Decoder::for_simulator();

    // Act
    let encoded_resource_address = bech32::encode(
        bech32_encoder.hrp_set.get_entity_hrp(&EntityType::Resource),
        resource_address.to_vec().to_base32(),
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
    let bech32_encoder = Bech32Encoder::new(&NetworkDefinition::simulator());
    let bech32_decoder = Bech32Decoder::new(&NetworkDefinition::simulator());

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
    let bech32_encoder = Bech32Encoder::new(&NetworkDefinition::simulator());
    let bech32_decoder = Bech32Decoder::new(&NetworkDefinition::simulator());

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
fn decode_matching_component_address_entity_id_succeeds() {
    // Arrange
    let bech32_encoder = Bech32Encoder::new(&NetworkDefinition::simulator());
    let bech32_decoder = Bech32Decoder::new(&NetworkDefinition::simulator());

    // Act
    let encoded_component_address = bech32::encode(
        bech32_encoder
            .hrp_set
            .get_entity_hrp(&EntityType::NormalComponent),
        generate_u8_array(NORMAL_COMPONENT_ADDRESS_ENTITY_ID).to_base32(),
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
    let bech32_encoder = Bech32Encoder::new(&NetworkDefinition::simulator());
    let bech32_decoder = Bech32Decoder::new(&NetworkDefinition::simulator());

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
        Err(AddressError::InvalidHrp)
    ));
}

#[test]
fn decode_matching_resource_address_entity_id_succeeds() {
    // Arrange
    let bech32_encoder = Bech32Encoder::new(&NetworkDefinition::simulator());
    let bech32_decoder = Bech32Decoder::new(&NetworkDefinition::simulator());

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
    let bech32_encoder = Bech32Encoder::new(&NetworkDefinition::simulator());
    let bech32_decoder = Bech32Decoder::new(&NetworkDefinition::simulator());

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
        Err(AddressError::InvalidHrp)
    ));
}

#[test]
fn decode_invalid_entity_specifier_fails() {
    // Arrange
    let bech32_encoder = Bech32Encoder::new(&NetworkDefinition::simulator());
    let bech32_decoder = Bech32Decoder::new(&NetworkDefinition::simulator());

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
    let bech32_decoder = Bech32Decoder::new(&NetworkDefinition::simulator());

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
