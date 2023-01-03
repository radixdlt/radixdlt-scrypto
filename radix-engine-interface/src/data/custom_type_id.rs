use sbor::*;

pub const TYPE_PACKAGE_ADDRESS: u8 = 0x80;
pub const TYPE_COMPONENT_ADDRESS: u8 = 0x81;
pub const TYPE_RESOURCE_ADDRESS: u8 = 0x82;
pub const TYPE_SYSTEM_ADDRESS: u8 = 0x83;

pub const TYPE_OWN: u8 = 0x94;
pub const TYPE_NON_FUNGIBLE_ADDRESS: u8 = 0xa2;
pub const TYPE_COMPONENT: u8 = 0x90;
pub const TYPE_KEY_VALUE_STORE: u8 = 0x91;
pub const TYPE_BLOB: u8 = 0xa1;

pub const TYPE_BUCKET: u8 = 0x92;
pub const TYPE_PROOF: u8 = 0x93;
pub const TYPE_EXPRESSION: u8 = 0xa0;

pub const TYPE_HASH: u8 = 0xb0;
pub const TYPE_ECDSA_SECP256K1_PUBIC_KEY: u8 = 0xb1;
pub const TYPE_ECDSA_SECP256K1_SIGNATURE: u8 = 0xb2;
pub const TYPE_EDDSA_ED25519_PUBIC_KEY: u8 = 0xb3;
pub const TYPE_EDDSA_ED25519_SIGNATURE: u8 = 0xb4;
pub const TYPE_DECIMAL: u8 = 0xb5;
pub const TYPE_PRECISE_DECIMAL: u8 = 0xb6;
pub const TYPE_NON_FUNGIBLE_ID: u8 = 0xb7;

#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type")
)]
#[derive(Copy, Debug, Clone, PartialEq, Eq)]
pub enum ScryptoCustomTypeId {
    // RE global address types
    PackageAddress,
    ComponentAddress,
    ResourceAddress,
    SystemAddress,

    // RE interpreted types
    Own,
    Component,
    KeyValueStore,
    NonFungibleAddress, // for resource address contained
    Blob,

    // TX interpreted types
    Bucket,     // super::types::ManifestBucket, also interpreted by engine at the moment
    Proof,      // super::types::ManifestProof, also interpreted by engine at the moment
    Expression, // super::types::Expression

    // Uninterpreted
    Hash,
    EcdsaSecp256k1PublicKey,
    EcdsaSecp256k1Signature,
    EddsaEd25519PublicKey,
    EddsaEd25519Signature,
    Decimal,
    PreciseDecimal,
    NonFungibleId,
}

impl From<ScryptoCustomTypeId> for SborTypeId<ScryptoCustomTypeId> {
    fn from(custom_type_id: ScryptoCustomTypeId) -> Self {
        SborTypeId::Custom(custom_type_id)
    }
}

impl CustomTypeId for ScryptoCustomTypeId {
    fn as_u8(&self) -> u8 {
        match self {
            Self::PackageAddress => TYPE_PACKAGE_ADDRESS,
            Self::ComponentAddress => TYPE_COMPONENT_ADDRESS,
            Self::ResourceAddress => TYPE_RESOURCE_ADDRESS,
            Self::SystemAddress => TYPE_SYSTEM_ADDRESS,
            Self::Own => TYPE_OWN,
            Self::Component => TYPE_COMPONENT,
            Self::KeyValueStore => TYPE_KEY_VALUE_STORE,
            Self::Bucket => TYPE_BUCKET,
            Self::Proof => TYPE_PROOF,
            Self::Expression => TYPE_EXPRESSION,
            Self::Blob => TYPE_BLOB,
            Self::NonFungibleAddress => TYPE_NON_FUNGIBLE_ADDRESS,
            Self::Hash => TYPE_HASH,
            Self::EcdsaSecp256k1PublicKey => TYPE_ECDSA_SECP256K1_PUBIC_KEY,
            Self::EcdsaSecp256k1Signature => TYPE_ECDSA_SECP256K1_SIGNATURE,
            Self::EddsaEd25519PublicKey => TYPE_EDDSA_ED25519_PUBIC_KEY,
            Self::EddsaEd25519Signature => TYPE_EDDSA_ED25519_SIGNATURE,
            Self::Decimal => TYPE_DECIMAL,
            Self::PreciseDecimal => TYPE_PRECISE_DECIMAL,
            Self::NonFungibleId => TYPE_NON_FUNGIBLE_ID,
        }
    }

    fn from_u8(id: u8) -> Option<Self> {
        match id {
            TYPE_PACKAGE_ADDRESS => Some(ScryptoCustomTypeId::PackageAddress),
            TYPE_COMPONENT_ADDRESS => Some(ScryptoCustomTypeId::ComponentAddress),
            TYPE_RESOURCE_ADDRESS => Some(ScryptoCustomTypeId::ResourceAddress),
            TYPE_SYSTEM_ADDRESS => Some(ScryptoCustomTypeId::SystemAddress),
            TYPE_OWN => Some(ScryptoCustomTypeId::Own),
            TYPE_COMPONENT => Some(ScryptoCustomTypeId::Component),
            TYPE_KEY_VALUE_STORE => Some(ScryptoCustomTypeId::KeyValueStore),
            TYPE_BUCKET => Some(ScryptoCustomTypeId::Bucket),
            TYPE_PROOF => Some(ScryptoCustomTypeId::Proof),
            TYPE_EXPRESSION => Some(ScryptoCustomTypeId::Expression),
            TYPE_BLOB => Some(ScryptoCustomTypeId::Blob),
            TYPE_NON_FUNGIBLE_ADDRESS => Some(ScryptoCustomTypeId::NonFungibleAddress),
            TYPE_HASH => Some(ScryptoCustomTypeId::Hash),
            TYPE_ECDSA_SECP256K1_PUBIC_KEY => Some(ScryptoCustomTypeId::EcdsaSecp256k1PublicKey),
            TYPE_ECDSA_SECP256K1_SIGNATURE => Some(ScryptoCustomTypeId::EcdsaSecp256k1Signature),
            TYPE_EDDSA_ED25519_PUBIC_KEY => Some(ScryptoCustomTypeId::EddsaEd25519PublicKey),
            TYPE_EDDSA_ED25519_SIGNATURE => Some(ScryptoCustomTypeId::EddsaEd25519Signature),
            TYPE_DECIMAL => Some(ScryptoCustomTypeId::Decimal),
            TYPE_PRECISE_DECIMAL => Some(ScryptoCustomTypeId::PreciseDecimal),
            TYPE_NON_FUNGIBLE_ID => Some(ScryptoCustomTypeId::NonFungibleId),
            _ => None,
        }
    }
}
