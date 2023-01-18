use sbor::*;

pub const TYPE_PACKAGE_ADDRESS: u8 = 0x80;
pub const TYPE_COMPONENT_ADDRESS: u8 = 0x81;
pub const TYPE_RESOURCE_ADDRESS: u8 = 0x82;
pub const TYPE_SYSTEM_ADDRESS: u8 = 0x83;

pub const TYPE_OWN: u8 = 0x90;
pub const TYPE_BLOB: u8 = 0x91;

pub const TYPE_BUCKET: u8 = 0xa0;
pub const TYPE_PROOF: u8 = 0xa1;
pub const TYPE_EXPRESSION: u8 = 0xa2;

pub const TYPE_HASH: u8 = 0xb0;
pub const TYPE_ECDSA_SECP256K1_PUBIC_KEY: u8 = 0xb1;
pub const TYPE_ECDSA_SECP256K1_SIGNATURE: u8 = 0xb2;
pub const TYPE_EDDSA_ED25519_PUBIC_KEY: u8 = 0xb3;
pub const TYPE_EDDSA_ED25519_SIGNATURE: u8 = 0xb4;
pub const TYPE_DECIMAL: u8 = 0xb5;
pub const TYPE_PRECISE_DECIMAL: u8 = 0xb6;
pub const TYPE_NON_FUNGIBLE_LOCAL_ID: u8 = 0xb7;

#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type")
)]
#[derive(Copy, Debug, Clone, PartialEq, Eq)]
pub enum ScryptoCustomTypeId {
    // RE interpreted types
    PackageAddress,
    ComponentAddress,
    ResourceAddress,
    SystemAddress,
    Own,

    // TX interpreted types
    Bucket,
    Proof,
    Expression,
    Blob,

    // Uninterpreted
    Hash,
    EcdsaSecp256k1PublicKey,
    EcdsaSecp256k1Signature,
    EddsaEd25519PublicKey,
    EddsaEd25519Signature,
    Decimal,
    PreciseDecimal,
    NonFungibleLocalId,
}

impl From<ScryptoCustomTypeId> for ValueKind<ScryptoCustomTypeId> {
    fn from(custom_type_id: ScryptoCustomTypeId) -> Self {
        ValueKind::Custom(custom_type_id)
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
            Self::Bucket => TYPE_BUCKET,
            Self::Proof => TYPE_PROOF,
            Self::Expression => TYPE_EXPRESSION,
            Self::Blob => TYPE_BLOB,
            Self::Hash => TYPE_HASH,
            Self::EcdsaSecp256k1PublicKey => TYPE_ECDSA_SECP256K1_PUBIC_KEY,
            Self::EcdsaSecp256k1Signature => TYPE_ECDSA_SECP256K1_SIGNATURE,
            Self::EddsaEd25519PublicKey => TYPE_EDDSA_ED25519_PUBIC_KEY,
            Self::EddsaEd25519Signature => TYPE_EDDSA_ED25519_SIGNATURE,
            Self::Decimal => TYPE_DECIMAL,
            Self::PreciseDecimal => TYPE_PRECISE_DECIMAL,
            Self::NonFungibleLocalId => TYPE_NON_FUNGIBLE_LOCAL_ID,
        }
    }

    fn from_u8(id: u8) -> Option<Self> {
        match id {
            TYPE_PACKAGE_ADDRESS => Some(ScryptoCustomTypeId::PackageAddress),
            TYPE_COMPONENT_ADDRESS => Some(ScryptoCustomTypeId::ComponentAddress),
            TYPE_RESOURCE_ADDRESS => Some(ScryptoCustomTypeId::ResourceAddress),
            TYPE_SYSTEM_ADDRESS => Some(ScryptoCustomTypeId::SystemAddress),
            TYPE_OWN => Some(ScryptoCustomTypeId::Own),
            TYPE_BUCKET => Some(ScryptoCustomTypeId::Bucket),
            TYPE_PROOF => Some(ScryptoCustomTypeId::Proof),
            TYPE_EXPRESSION => Some(ScryptoCustomTypeId::Expression),
            TYPE_BLOB => Some(ScryptoCustomTypeId::Blob),
            TYPE_HASH => Some(ScryptoCustomTypeId::Hash),
            TYPE_ECDSA_SECP256K1_PUBIC_KEY => Some(ScryptoCustomTypeId::EcdsaSecp256k1PublicKey),
            TYPE_ECDSA_SECP256K1_SIGNATURE => Some(ScryptoCustomTypeId::EcdsaSecp256k1Signature),
            TYPE_EDDSA_ED25519_PUBIC_KEY => Some(ScryptoCustomTypeId::EddsaEd25519PublicKey),
            TYPE_EDDSA_ED25519_SIGNATURE => Some(ScryptoCustomTypeId::EddsaEd25519Signature),
            TYPE_DECIMAL => Some(ScryptoCustomTypeId::Decimal),
            TYPE_PRECISE_DECIMAL => Some(ScryptoCustomTypeId::PreciseDecimal),
            TYPE_NON_FUNGIBLE_LOCAL_ID => Some(ScryptoCustomTypeId::NonFungibleLocalId),
            _ => None,
        }
    }
}
