use sbor::rust::borrow::ToOwned;
use sbor::rust::string::String;

/// Scrypto types are special types that are Scrypto specific and may require special interpretation.
///
/// They are custom types to SBOR serialization protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScryptoTypeId {
    // Global address types
    PackageAddress,
    ComponentAddress,
    ResourceAddress,
    SystemAddress,

    // RE nodes types
    Component,
    KeyValueStore,
    Bucket,
    Proof,
    Vault,

    // Other interpreted types
    Expression,
    Blob,
    NonFungibleAddress, // for resource address contained

    // Uninterpreted, mainly for better manifest representation.
    Hash,
    EcdsaSecp256k1PublicKey,
    EcdsaSecp256k1Signature,
    EddsaEd25519PublicKey,
    EddsaEd25519Signature,
    Decimal,
    PreciseDecimal,
    NonFungibleId,
}

// Need to update `scrypto-derive/src/import.rs` after changing the table below
const MAPPING: [(ScryptoTypeId, u8, &str); 20] = [
    (ScryptoTypeId::PackageAddress, 0x80, "PackageAddress"),
    (ScryptoTypeId::ComponentAddress, 0x81, "ComponentAddress"),
    (ScryptoTypeId::ResourceAddress, 0x82, "ResourceAddress"),
    (ScryptoTypeId::SystemAddress, 0x83, "SystemAddress"),
    (ScryptoTypeId::Component, 0x90, "Component"),
    (ScryptoTypeId::KeyValueStore, 0x91, "KeyValueStore"),
    (ScryptoTypeId::Bucket, 0x92, "Bucket"),
    (ScryptoTypeId::Proof, 0x93, "Proof"),
    (ScryptoTypeId::Vault, 0x94, "Vault"),
    (ScryptoTypeId::Expression, 0xa0, "Expression"),
    (ScryptoTypeId::Blob, 0xa1, "Blob"),
    (
        ScryptoTypeId::NonFungibleAddress,
        0xa2,
        "NonFungibleAddress",
    ),
    (ScryptoTypeId::Hash, 0xb0, "Hash"),
    (
        ScryptoTypeId::EcdsaSecp256k1PublicKey,
        0xb1,
        "EcdsaSecp256k1PublicKey",
    ),
    (
        ScryptoTypeId::EcdsaSecp256k1Signature,
        0xb2,
        "EcdsaSecp256k1Signature",
    ),
    (
        ScryptoTypeId::EddsaEd25519PublicKey,
        0xb3,
        "EddsaEd25519PublicKey",
    ),
    (
        ScryptoTypeId::EddsaEd25519Signature,
        0xb4,
        "EddsaEd25519Signature",
    ),
    (ScryptoTypeId::Decimal, 0xb5, "Decimal"),
    (ScryptoTypeId::PreciseDecimal, 0xb6, "PreciseDecimal"),
    (ScryptoTypeId::NonFungibleId, 0xb7, "NonFungibleId"),
];

impl ScryptoTypeId {
    // TODO: optimize to get rid of loops

    pub fn from_id(id: u8) -> Option<ScryptoTypeId> {
        MAPPING.iter().filter(|e| e.1 == id).map(|e| e.0).next()
    }

    pub fn from_name(name: &str) -> Option<ScryptoTypeId> {
        MAPPING.iter().filter(|e| e.2 == name).map(|e| e.0).next()
    }

    pub fn id(&self) -> u8 {
        MAPPING
            .iter()
            .filter(|e| e.0 == *self)
            .map(|e| e.1)
            .next()
            .unwrap()
    }

    pub fn name(&self) -> String {
        MAPPING
            .iter()
            .filter(|e| e.0 == *self)
            .map(|e| e.2)
            .next()
            .unwrap()
            .to_owned()
    }
}
