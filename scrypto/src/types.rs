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
    PackageRef,
    ComponentRef,
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
    ResourceDefRef,
}

impl CustomType {
    pub fn of(id: u8) -> Option<CustomType> {
        match id {
            // core
            0x80 => Some(CustomType::PackageRef),
            0x81 => Some(CustomType::ComponentRef),
            0x82 => Some(CustomType::LazyMap),
            // crypto
            0x90 => Some(CustomType::Hash),
            // math
            0xa0 => Some(CustomType::Decimal),
            0xa1 => Some(CustomType::BigDecimal),
            // resource
            0xb0 => Some(CustomType::Bucket),
            0xb1 => Some(CustomType::BucketRef),
            0xb2 => Some(CustomType::Vault),
            0xb3 => Some(CustomType::NonFungibleKey),
            0xb4 => Some(CustomType::ResourceDefRef),
            _ => None,
        }
    }

    pub fn id(&self) -> u8 {
        match self {
            // core
            CustomType::PackageRef => 0x80,
            CustomType::ComponentRef => 0x81,
            CustomType::LazyMap => 0x82,
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
            CustomType::ResourceDefRef => 0xb4,
        }
    }

    pub fn name(&self) -> String {
        match self {
            // core
            CustomType::PackageRef => "scrypto::core::PackageRef",
            CustomType::ComponentRef => "scrypto::core::ComponentRef",
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
            CustomType::ResourceDefRef => "scrypto::resource::ResourceDefRef",
        }
        .to_owned()
    }
}
