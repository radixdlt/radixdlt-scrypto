use bech32::{self, ToBase32, Variant};
use scrypto::{
    address::{Bech32Decoder, Bech32Encoder, DecodeBech32AddressError},
    network::NetworkDefinition,
    prelude::*,
};

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
    let package_address = package_address(EntityType::GlobalPackage, 1);
    let bech32_encoder = Bech32Encoder::for_simulator();

    // Act
    let bech32 = bech32_encoder.encode(package_address.as_ref()).unwrap();

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
    let resource_address = resource_address(EntityType::GlobalFungibleResource, 2);
    let bech32_encoder = Bech32Encoder::for_simulator();
    let bech32_decoder = Bech32Decoder::for_simulator();

    let encoded_resource_address = bech32_encoder.encode(resource_address.as_ref()).unwrap();

    // Act
    let decoded_resource_address = bech32_decoder
        .validate_and_decode(&encoded_resource_address[..encoded_resource_address.len() - 2]);

    // Assert
    assert!(matches!(
        decoded_resource_address,
        Err(DecodeBech32AddressError::Bech32mDecodingError(_))
    ));
}

#[test]
fn decode_modified_checksum_address_fails() {
    // Arrange
    let resource_address = resource_address(EntityType::GlobalFungibleResource, 2);
    let bech32_encoder = Bech32Encoder::for_simulator();
    let bech32_decoder = Bech32Decoder::for_simulator();

    let mut encoded_resource_address = bech32_encoder.encode(resource_address.as_ref()).unwrap();

    // Act
    encoded_resource_address.push_str("qq");
    let decoded_resource_address = bech32_decoder.validate_and_decode(&encoded_resource_address);

    // Assert
    assert!(matches!(
        decoded_resource_address,
        Err(DecodeBech32AddressError::Bech32mDecodingError(_))
    ));
}

/// Tests if the decoding fails when the address is encoded in Bech32 and not Bech32m
#[test]
fn decode_invalid_bech32_variant_fails() {
    // Arrange
    let resource_address = resource_address(EntityType::GlobalFungibleResource, 2);
    let bech32_encoder = Bech32Encoder::for_simulator();
    let bech32_decoder = Bech32Decoder::for_simulator();

    // Act
    let encoded_resource_address = bech32::encode(
        bech32_encoder
            .hrp_set
            .get_entity_hrp(&EntityType::GlobalNonFungibleResource),
        resource_address.to_vec().to_base32(),
        Variant::Bech32,
    )
    .unwrap();

    let decoded_resource_address = bech32_decoder.validate_and_decode(&encoded_resource_address);

    // Assert
    assert!(matches!(
        decoded_resource_address,
        Err(DecodeBech32AddressError::InvalidVariant(Variant::Bech32))
    ));
}

#[test]
fn decode_matching_package_address_entity_id_succeeds() {
    // Arrange
    let bech32_encoder = Bech32Encoder::new(&NetworkDefinition::simulator());
    let bech32_decoder = Bech32Decoder::new(&NetworkDefinition::simulator());

    // Act
    let encoded_package_address = bech32::encode(
        bech32_encoder
            .hrp_set
            .get_entity_hrp(&EntityType::GlobalPackage),
        generate_u8_array(EntityType::GlobalPackage as u8).to_base32(),
        Variant::Bech32m,
    )
    .unwrap();

    let decoded_package_address = bech32_decoder.validate_and_decode(&encoded_package_address);

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
            .get_entity_hrp(&EntityType::GlobalAccount),
        generate_u8_array(EntityType::GlobalAccount as u8).to_base32(),
        Variant::Bech32m,
    )
    .unwrap();

    let decoded_account_address = bech32_decoder.validate_and_decode(&encoded_account_address);

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
            .get_entity_hrp(&EntityType::GlobalGenericComponent),
        generate_u8_array(EntityType::GlobalGenericComponent as u8).to_base32(),
        Variant::Bech32m,
    )
    .unwrap();

    let decoded_component_address = bech32_decoder.validate_and_decode(&encoded_component_address);

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
        bech32_encoder
            .hrp_set
            .get_entity_hrp(&EntityType::GlobalPackage),
        generate_u8_array(EntityType::GlobalNonFungibleResource as u8).to_base32(),
        Variant::Bech32m,
    )
    .unwrap();

    let decoded_package_address = bech32_decoder.validate_and_decode(&encoded_package_address);

    // Assert
    assert!(matches!(
        decoded_package_address,
        Err(DecodeBech32AddressError::InvalidHrp)
    ));
}

#[test]
fn decode_matching_resource_address_entity_id_succeeds() {
    // Arrange
    let bech32_encoder = Bech32Encoder::new(&NetworkDefinition::simulator());
    let bech32_decoder = Bech32Decoder::new(&NetworkDefinition::simulator());

    // Act
    let encoded_resource_address = bech32::encode(
        bech32_encoder
            .hrp_set
            .get_entity_hrp(&EntityType::GlobalNonFungibleResource),
        generate_u8_array(EntityType::GlobalNonFungibleResource as u8).to_base32(),
        Variant::Bech32m,
    )
    .unwrap();

    let decoded_resource_address = bech32_decoder.validate_and_decode(&encoded_resource_address);

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
        bech32_encoder
            .hrp_set
            .get_entity_hrp(&EntityType::GlobalNonFungibleResource),
        generate_u8_array(EntityType::GlobalPackage as u8).to_base32(),
        Variant::Bech32m,
    )
    .unwrap();

    let decoded_resource_address = bech32_decoder.validate_and_decode(&encoded_resource_address);

    // Assert
    assert!(matches!(
        decoded_resource_address,
        Err(DecodeBech32AddressError::InvalidHrp)
    ));
}

#[test]
fn decode_invalid_entity_specifier_fails() {
    // Arrange
    let bech32_encoder = Bech32Encoder::new(&NetworkDefinition::simulator());
    let bech32_decoder = Bech32Decoder::new(&NetworkDefinition::simulator());

    // Act
    let encoded_resource_address = bech32::encode(
        bech32_encoder
            .hrp_set
            .get_entity_hrp(&EntityType::GlobalNonFungibleResource),
        generate_u8_array(EntityType::GlobalPackage as u8).to_base32(),
        Variant::Bech32m,
    )
    .unwrap();

    let decoded_resource_address = bech32_decoder.validate_and_decode(&encoded_resource_address);

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
        bech32_encoder
            .hrp_set
            .get_entity_hrp(&EntityType::GlobalNonFungibleResource),
        generate_u8_array(EntityType::GlobalNonFungibleResource as u8).to_base32(),
        Variant::Bech32m,
    )
    .unwrap();

    let decoded_resource_address = bech32_decoder.validate_and_decode(&encoded_resource_address);

    // Assert
    assert!(matches!(decoded_resource_address, Err(_)));
}
