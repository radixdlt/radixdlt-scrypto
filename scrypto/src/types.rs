macro_rules! custom_type {
    ($t:ty, $ct:expr, $generics: expr) => {
        impl TypeId for $t {
            #[inline]
            fn type_id() -> u8 {
                $ct.id()
            }
        }

        impl Encode for $t {
            fn encode_value(&self, encoder: &mut Encoder) {
                let bytes = self.to_vec();
                encoder.write_len(bytes.len());
                encoder.write_slice(&bytes);
            }
        }

        impl Decode for $t {
            fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
                let len = decoder.read_len()?;
                let slice = decoder.read_bytes(len)?;
                Self::try_from(slice).map_err(|_| DecodeError::InvalidCustomData($ct.id()))
            }
        }

        impl Describe for $t {
            fn describe() -> Type {
                Type::Custom {
                    name: $ct.name(),
                    generics: $generics,
                }
            }
        }
    };
}

pub(crate) use custom_type;

/// Scrypto types that are encoded as custom SBOR types.
///
/// Any encode-able type in Scrypto library that requires special interpretation
/// must be declared as a custom type.
///
/// Custom types must be encoded as `[length + bytes]`.
pub enum CustomType {
    // core
    Package,
    Component,
    LazyMap,

    // crypto
    Hash,

    // math
    Decimal,
    BigDecimal,

    // resource,
    Bucket,
    BucketRef,
    Vault,
    NonFungibleKey,
    ResourceDef,
}

impl CustomType {
    pub fn id(&self) -> u8 {
        match self {
            // core
            CustomType::Package => 0x80,
            CustomType::Component => 0x81,
            CustomType::LazyMap => 0x83,
            // crypto
            CustomType::Hash => 0x90,
            // math
            CustomType::Decimal => 0xa0,
            CustomType::BigDecimal => 0xa1,
            // resource
            CustomType::Bucket => 0xb0,
            CustomType::BucketRef => 0xb1,
            CustomType::Vault => 0xb2,
            CustomType::NonFungibleKey => 0xb3,
            CustomType::ResourceDef => 0xb4,
        }
    }

    pub fn name(&self) -> String {
        match self {
            // core
            CustomType::Package => "scrypto::core::Package",
            CustomType::Component => "scrypto::core::Component",
            CustomType::LazyMap => "scrypto::core::LazyMap",
            // crypto
            CustomType::Hash => "scrypto::crypto::Hash",
            // math
            CustomType::Decimal => "scrypto::math::Decimal",
            CustomType::BigDecimal => "scrypto::math::BigDecimal",
            // resource
            CustomType::Bucket => "scrypto::resource::Bucket",
            CustomType::BucketRef => "scrypto::resource::BucketRef",
            CustomType::Vault => "scrypto::resource::Vault",
            CustomType::NonFungibleKey => "scrypto::resource::NonFungibleKey",
            CustomType::ResourceDef => "scrypto::resource::ResourceDef",
        }
        .to_owned()
    }
}
