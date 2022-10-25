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
    // Global address types
    PackageAddress,
    ComponentAddress,
    ResourceAddress,
    SystemAddress,

    // RE Nodes types
    // TODO: replace with `Owned(RENodeId)` and `Ref(RENodeId)`
    Component,
    KeyValueStore,
    Bucket,
    Proof,
    Vault,

    // Engine/Transaction interpreted types
    Expression,
    Blob,

    // Convenience types
    // They have no special meaning to the engine and are mainly for better manifest representation.
    Hash,
    EcdsaSecp256k1PublicKey,
    EcdsaSecp256k1Signature,
    EddsaEd25519PublicKey,
    EddsaEd25519Signature,
    Decimal,
    PreciseDecimal,
    NonFungibleId,
    NonFungibleAddress,
}

// Need to update `scrypto-derive/src/import.rs` after changing the table below
const MAPPING: [(ScryptoType, u8, &str); 20] = [
    (ScryptoType::PackageAddress, 0x80, "PackageAddress"),
    (ScryptoType::ComponentAddress, 0x81, "ComponentAddress"),
    (ScryptoType::ResourceAddress, 0x82, "ResourceAddress"),
    (ScryptoType::SystemAddress, 0x83, "SystemAddress"),
    (ScryptoType::Component, 0x90, "Component"),
    (ScryptoType::KeyValueStore, 0x91, "KeyValueStore"),
    (ScryptoType::Bucket, 0x92, "Bucket"),
    (ScryptoType::Proof, 0x93, "Proof"),
    (ScryptoType::Vault, 0x94, "Vault"),
    (ScryptoType::Expression, 0xa0, "Expression"),
    (ScryptoType::Blob, 0xa1, "Blob"),
    (ScryptoType::Hash, 0xb0, "Hash"),
    (
        ScryptoType::EcdsaSecp256k1PublicKey,
        0xb1,
        "EcdsaSecp256k1PublicKey",
    ),
    (
        ScryptoType::EcdsaSecp256k1Signature,
        0xb2,
        "EcdsaSecp256k1Signature",
    ),
    (
        ScryptoType::EddsaEd25519PublicKey,
        0xb3,
        "EddsaEd25519PublicKey",
    ),
    (
        ScryptoType::EddsaEd25519Signature,
        0xb4,
        "EddsaEd25519Signature",
    ),
    (ScryptoType::Decimal, 0xb5, "Decimal"),
    (ScryptoType::PreciseDecimal, 0xb6, "PreciseDecimal"),
    (ScryptoType::NonFungibleId, 0xb7, "NonFungibleId"),
    (ScryptoType::NonFungibleAddress, 0xb8, "NonFungibleAddress"),
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
