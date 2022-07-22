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
            fn encode_type(&self, encoder: &mut Encoder) {
                encoder.write_type(Self::type_id());
            }
            #[inline]
            fn encode_value(&self, encoder: &mut Encoder) {
                let bytes = self.to_vec();
                encoder.write_len(bytes.len());
                encoder.write_slice(&bytes);
            }
        }

        impl Decode for $t {
            fn decode_type(decoder: &mut Decoder) -> Result<(), DecodeError> {
                decoder.check_type(Self::type_id())
            }
            fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
                let len = decoder.read_len()?;
                let slice = decoder.read_bytes(len)?;
                Self::try_from(slice).map_err(|_| {
                    DecodeError::CustomError(::sbor::rust::format!(
                        "Failed to decode {}",
                        stringify!($t)
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
    EcdsaPublicKey,
    EcdsaSignature,
    Ed25519PublicKey,
    Ed25519Signature,

    // math
    U8,
    U16,
    U32,
    U64,
    U128,
    U256,
    U384,
    U512,
    I8,
    I16,
    I32,
    I64,
    I128,
    I256,
    I384,
    I512,
    Decimal,
    LongDecimal,

    // resource,
    Bucket,
    Proof,
    Vault,
    NonFungibleId,
    NonFungibleAddress,
    ResourceAddress,
}

// Need to update `scrypto-derive/src/import.rs` after changing the table below
const MAPPING: [(ScryptoType, u8, &str); 33] = [
    (ScryptoType::PackageAddress, 0x80, "PackageAddress"), // 128
    (ScryptoType::ComponentAddress, 0x81, "ComponentAddress"), // 129
    (ScryptoType::Component, 0x82, "ComponentAddress"),    // 130
    (ScryptoType::KeyValueStore, 0x83, "KeyValueStore"),   // 131
    (ScryptoType::Hash, 0x90, "Hash"),                     // 144
    (ScryptoType::EcdsaPublicKey, 0x91, "EcdsaPublicKey"), // 145
    (ScryptoType::EcdsaSignature, 0x92, "EcdsaSignature"), // 146
    (ScryptoType::Ed25519PublicKey, 0x93, "Ed25519PublicKey"), // 147
    (ScryptoType::Ed25519Signature, 0x94, "Ed25519Signature"), // 148
    // removed: (ScryptoType::Decimal, 0xa1, "Decimal"),   // 161
    (ScryptoType::Bucket, 0xb1, "Bucket"),                 // 177
    (ScryptoType::Proof, 0xb2, "Proof"),                   // 178
    (ScryptoType::Vault, 0xb3, "Vault"),                   // 179
    (ScryptoType::NonFungibleId, 0xb4, "NonFungibleId"),   // 180
    (ScryptoType::NonFungibleAddress, 0xb5, "NonFungibleAddress"), // 181
    (ScryptoType::ResourceAddress, 0xb6, "ResourceAddress"), // 182
    (ScryptoType::U8, 0xb7, "U8"), // 183
    (ScryptoType::U16, 0xb8, "U16"), // 184
    (ScryptoType::U32, 0xb9, "U32"), // 185
    (ScryptoType::U64, 0xba, "U64"), // 186
    (ScryptoType::U128, 0xbb, "U128"), // 187
    (ScryptoType::U256, 0xbc, "U256"), // 188
    (ScryptoType::U384, 0xbd, "U384"), // 189
    (ScryptoType::U512, 0xbe, "U512"), // 190
    (ScryptoType::I8, 0xbf, "I8"), // 191
    (ScryptoType::I16, 0xc0, "I16"), // 192
    (ScryptoType::I32, 0xc1, "I32"), // 193
    (ScryptoType::I64, 0xc2, "I64"), // 194
    (ScryptoType::I128, 0xc3, "I128"), // 195
    (ScryptoType::I256, 0xc4, "I256"), // 196
    (ScryptoType::I384, 0xc5, "I384"), // 197
    (ScryptoType::I512, 0xc6, "I512"), // 198
    (ScryptoType::Decimal, 0xc7, "Decimal"), // 199
    (ScryptoType::LongDecimal, 0xc8, "LongDecimal"), // 200
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
