use super::*;
use sbor::*;

use well_known_scrypto_custom_types::*;

pub mod well_known_scrypto_custom_types {
    use super::*;

    pub const PACKAGE_ADDRESS_ID: u8 = VALUE_KIND_PACKAGE_ADDRESS;
    pub const COMPONENT_ADDRESS_ID: u8 = VALUE_KIND_COMPONENT_ADDRESS;
    pub const RESOURCE_ADDRESS_ID: u8 = VALUE_KIND_RESOURCE_ADDRESS;

    pub const OWN_ID: u8 = VALUE_KIND_OWN;
    // We skip KeyValueStore because it has generic parameters

    pub const HASH_ID: u8 = VALUE_KIND_HASH;
    pub const ECDSA_SECP256K1_PUBLIC_KEY_ID: u8 = VALUE_KIND_ECDSA_SECP256K1_PUBLIC_KEY;
    pub const ECDSA_SECP256K1_SIGNATURE_ID: u8 = VALUE_KIND_ECDSA_SECP256K1_SIGNATURE;
    pub const EDDSA_ED25519_PUBLIC_KEY_ID: u8 = VALUE_KIND_EDDSA_ED25519_PUBLIC_KEY;
    pub const EDDSA_ED25519_SIGNATURE_ID: u8 = VALUE_KIND_EDDSA_ED25519_SIGNATURE;
    pub const DECIMAL_ID: u8 = VALUE_KIND_DECIMAL;
    pub const PRECISE_DECIMAL_ID: u8 = VALUE_KIND_PRECISE_DECIMAL;
    pub const NON_FUNGIBLE_LOCAL_ID_ID: u8 = VALUE_KIND_NON_FUNGIBLE_LOCAL_ID;
}

pub(crate) fn resolve_scrypto_custom_well_known_type(
    well_known_index: u8,
) -> Option<TypeData<ScryptoCustomTypeKind<LocalTypeIndex>, LocalTypeIndex>> {
    let (name, custom_type_kind) = match well_known_index {
        PACKAGE_ADDRESS_ID => ("PackageAddress", ScryptoCustomTypeKind::PackageAddress),
        COMPONENT_ADDRESS_ID => ("ComponentAddress", ScryptoCustomTypeKind::ComponentAddress),
        RESOURCE_ADDRESS_ID => ("ResourceAddress", ScryptoCustomTypeKind::ResourceAddress),

        OWN_ID => ("Own", ScryptoCustomTypeKind::Own),

        HASH_ID => ("Hash", ScryptoCustomTypeKind::Hash),
        ECDSA_SECP256K1_PUBLIC_KEY_ID => (
            "EcdsaSecp256k1PublicKey",
            ScryptoCustomTypeKind::EcdsaSecp256k1PublicKey,
        ),
        ECDSA_SECP256K1_SIGNATURE_ID => (
            "EcdsaSecp256k1Signature",
            ScryptoCustomTypeKind::EcdsaSecp256k1Signature,
        ),
        EDDSA_ED25519_PUBLIC_KEY_ID => (
            "EddsaEd25519PublicKey",
            ScryptoCustomTypeKind::EddsaEd25519PublicKey,
        ),
        EDDSA_ED25519_SIGNATURE_ID => (
            "EddsaEd25519Signature",
            ScryptoCustomTypeKind::EddsaEd25519Signature,
        ),
        DECIMAL_ID => ("Decimal", ScryptoCustomTypeKind::Decimal),
        PRECISE_DECIMAL_ID => ("PreciseDecimal", ScryptoCustomTypeKind::PreciseDecimal),
        NON_FUNGIBLE_LOCAL_ID_ID => (
            "NonFungibleLocalId",
            ScryptoCustomTypeKind::NonFungibleLocalId,
        ),
        _ => return None,
    };

    Some(TypeData::named_no_child_names(
        name,
        TypeKind::Custom(custom_type_kind),
    ))
}
