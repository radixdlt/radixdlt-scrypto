use sbor::rust::borrow::ToOwned;
use sbor::rust::string::String;

/// A macro to help create a Scrypto-specific type.
#[macro_export]
macro_rules! scrypto_type {
    ($t:ty, $ct:expr, $generics: expr) => {
        impl TypeId for $t {
            #[inline]
            fn type_id() -> u8 {
                $ct.id()
            }
        }

        impl Encode for $t {
            #[inline]
            fn encode_type_id(encoder: &mut Encoder) {
                encoder.write_type_id(Self::type_id());
            }
            #[inline]
            fn encode_value(&self, encoder: &mut Encoder) {
                let bytes = self.to_vec();
                encoder.write_dynamic_size(bytes.len());
                encoder.write_slice(&bytes);
            }
        }

        impl Decode for $t {
            fn check_type_id(decoder: &mut Decoder) -> Result<(), DecodeError> {
                decoder.check_type_id(Self::type_id())
            }
            fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
                let len = decoder.read_dynamic_size()?;
                let slice = decoder.read_bytes(len)?;
                Self::try_from(slice).map_err(|err| {
                    DecodeError::CustomError(::sbor::rust::format!(
                        "Failed to decode {}: {:?}",
                        stringify!($t),
                        err
                    ))
                })
            }
        }

        impl Describe for $t {
            fn describe() -> sbor::describe::Type {
                sbor::describe::Type::Custom {
                    type_id: $ct.id(),
                    generics: $generics,
                }
            }
        }
    };
}

/// Scrypto types are special types that are Scrypto specific and may require special interpretation.
///
/// They are custom types to SBOR serialization protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScryptoType {
    // component
    PackageAddress,
    ComponentAddress,
    Component,
    KeyValueStore,

    // crypto
    Hash,
    EcdsaSecp256k1PublicKey,
    EcdsaSecp256k1Signature,
    EddsaEd25519PublicKey,
    EddsaEd25519Signature,

    // math
    Decimal,
    PreciseDecimal,

    // resource,
    Bucket,
    Proof,
    Vault,
    NonFungibleId,
    NonFungibleAddress,
    ResourceAddress,

    Expression,
    Blob,
}

// Need to update `scrypto-derive/src/import.rs` after changing the table below
const MAPPING: [(ScryptoType, u8, &str); 19] = [
    (ScryptoType::PackageAddress, 0x80, "PackageAddress"), // 128
    (ScryptoType::ComponentAddress, 0x81, "ComponentAddress"), // 129
    (ScryptoType::Component, 0x82, "ComponentAddress"),    // 130
    (ScryptoType::KeyValueStore, 0x83, "KeyValueStore"),   // 131
    (ScryptoType::Hash, 0x90, "Hash"),                     // 144
    (
        ScryptoType::EcdsaSecp256k1PublicKey,
        0x91,
        "EcdsaSecp256k1PublicKey",
    ), // 145
    (
        ScryptoType::EcdsaSecp256k1Signature,
        0x92,
        "EcdsaSecp256k1Signature",
    ), // 146
    (
        ScryptoType::EddsaEd25519PublicKey,
        0x93,
        "EddsaEd25519PublicKey",
    ), // 147
    (
        ScryptoType::EddsaEd25519Signature,
        0x94,
        "EddsaEd25519Signature",
    ), // 148
    (ScryptoType::Decimal, 0xa1, "Decimal"),               // 161
    (ScryptoType::PreciseDecimal, 0xa2, "PreciseDecimal"), // 162
    (ScryptoType::Bucket, 0xb1, "Bucket"),                 // 177
    (ScryptoType::Proof, 0xb2, "Proof"),                   // 178
    (ScryptoType::Vault, 0xb3, "Vault"),                   // 179
    (ScryptoType::NonFungibleId, 0xb4, "NonFungibleId"),   // 180
    (ScryptoType::NonFungibleAddress, 0xb5, "NonFungibleAddress"), // 181
    (ScryptoType::ResourceAddress, 0xb6, "ResourceAddress"), // 182
    (ScryptoType::Expression, 0xc1, "Expression"),         // 193
    (ScryptoType::Blob, 0xc2, "Blob"),                     // 194
];

impl ScryptoType {
    // TODO: optimize to get rid of loops

    pub fn from_id(id: u8) -> Option<ScryptoType> {
        MAPPING.iter().filter(|e| e.1 == id).map(|e| e.0).next()
    }

    pub fn from_name(name: &str) -> Option<ScryptoType> {
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
