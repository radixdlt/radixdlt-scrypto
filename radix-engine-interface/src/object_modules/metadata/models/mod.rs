mod discriminators;
mod origin;
mod url;

pub use self::url::*;
pub use discriminators::*;
pub use origin::*;

use crate::internal_prelude::*;
use crate::types::KeyValueStoreInit;
#[cfg(feature = "fuzzing")]
use arbitrary::Arbitrary;
use radix_common::crypto::PublicKey;
use radix_common::crypto::PublicKeyHash;
use radix_common::data::scrypto::model::NonFungibleLocalId;
use radix_common::data::scrypto::*;
use radix_common::math::Decimal;
use radix_common::time::Instant;
use radix_common::types::GlobalAddress;
use radix_common::types::NonFungibleGlobalId;
use sbor::SborEnum;

#[cfg_attr(feature = "fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
#[sbor(categorize_types = "U, O")]
pub enum GenericMetadataValue<U, O> {
    #[sbor(discriminator(METADATA_VALUE_STRING_DISCRIMINATOR))]
    String(String),
    #[sbor(discriminator(METADATA_VALUE_BOOLEAN_DISCRIMINATOR))]
    Bool(bool),
    #[sbor(discriminator(METADATA_VALUE_U8_DISCRIMINATOR))]
    U8(u8),
    #[sbor(discriminator(METADATA_VALUE_U32_DISCRIMINATOR))]
    U32(u32),
    #[sbor(discriminator(METADATA_VALUE_U64_DISCRIMINATOR))]
    U64(u64),
    #[sbor(discriminator(METADATA_VALUE_I32_DISCRIMINATOR))]
    I32(i32),
    #[sbor(discriminator(METADATA_VALUE_I64_DISCRIMINATOR))]
    I64(i64),
    #[sbor(discriminator(METADATA_VALUE_DECIMAL_DISCRIMINATOR))]
    Decimal(Decimal),
    #[sbor(discriminator(METADATA_VALUE_GLOBAL_ADDRESS_DISCRIMINATOR))]
    GlobalAddress(GlobalAddress),
    #[sbor(discriminator(METADATA_VALUE_PUBLIC_KEY_DISCRIMINATOR))]
    PublicKey(PublicKey),
    #[sbor(discriminator(METADATA_VALUE_NON_FUNGIBLE_GLOBAL_ID_DISCRIMINATOR))]
    NonFungibleGlobalId(NonFungibleGlobalId),
    #[sbor(discriminator(METADATA_VALUE_NON_FUNGIBLE_LOCAL_ID_DISCRIMINATOR))]
    NonFungibleLocalId(NonFungibleLocalId),
    #[sbor(discriminator(METADATA_VALUE_INSTANT_DISCRIMINATOR))]
    Instant(Instant),
    #[sbor(discriminator(METADATA_VALUE_URL_DISCRIMINATOR))]
    Url(U),
    #[sbor(discriminator(METADATA_VALUE_ORIGIN_DISCRIMINATOR))]
    Origin(O),
    #[sbor(discriminator(METADATA_VALUE_PUBLIC_KEY_HASH_DISCRIMINATOR))]
    PublicKeyHash(PublicKeyHash),

    #[sbor(discriminator(METADATA_VALUE_STRING_ARRAY_DISCRIMINATOR))]
    StringArray(Vec<String>),
    #[sbor(discriminator(METADATA_VALUE_BOOLEAN_ARRAY_DISCRIMINATOR))]
    BoolArray(Vec<bool>),
    #[sbor(discriminator(METADATA_VALUE_U8_ARRAY_DISCRIMINATOR))]
    U8Array(Vec<u8>),
    #[sbor(discriminator(METADATA_VALUE_U32_ARRAY_DISCRIMINATOR))]
    U32Array(Vec<u32>),
    #[sbor(discriminator(METADATA_VALUE_U64_ARRAY_DISCRIMINATOR))]
    U64Array(Vec<u64>),
    #[sbor(discriminator(METADATA_VALUE_I32_ARRAY_DISCRIMINATOR))]
    I32Array(Vec<i32>),
    #[sbor(discriminator(METADATA_VALUE_I64_ARRAY_DISCRIMINATOR))]
    I64Array(Vec<i64>),
    #[sbor(discriminator(METADATA_VALUE_DECIMAL_ARRAY_DISCRIMINATOR))]
    DecimalArray(Vec<Decimal>),
    #[sbor(discriminator(METADATA_VALUE_GLOBAL_ADDRESS_ARRAY_DISCRIMINATOR))]
    GlobalAddressArray(Vec<GlobalAddress>),
    #[sbor(discriminator(METADATA_VALUE_PUBLIC_KEY_ARRAY_DISCRIMINATOR))]
    PublicKeyArray(Vec<PublicKey>),
    #[sbor(discriminator(METADATA_VALUE_NON_FUNGIBLE_GLOBAL_ID_ARRAY_DISCRIMINATOR))]
    NonFungibleGlobalIdArray(Vec<NonFungibleGlobalId>),
    #[sbor(discriminator(METADATA_VALUE_NON_FUNGIBLE_LOCAL_ID_ARRAY_DISCRIMINATOR))]
    NonFungibleLocalIdArray(Vec<NonFungibleLocalId>),
    #[sbor(discriminator(METADATA_VALUE_INSTANT_ARRAY_DISCRIMINATOR))]
    InstantArray(Vec<Instant>),
    #[sbor(discriminator(METADATA_VALUE_URL_ARRAY_DISCRIMINATOR))]
    UrlArray(Vec<U>),
    #[sbor(discriminator(METADATA_VALUE_ORIGIN_ARRAY_DISCRIMINATOR))]
    OriginArray(Vec<O>),
    #[sbor(discriminator(METADATA_VALUE_PUBLIC_KEY_HASH_ARRAY_DISCRIMINATOR))]
    PublicKeyHashArray(Vec<PublicKeyHash>),
}

pub type MetadataValue = GenericMetadataValue<UncheckedUrl, UncheckedOrigin>;
pub type CheckedMetadataValue = GenericMetadataValue<CheckedUrl, CheckedOrigin>;

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub enum MetadataConversionError {
    UnexpectedType {
        expected_type_id: u8,
        actual_type_id: u8,
    },
}

pub trait MetadataVal: ScryptoEncode + ScryptoDecode + ToMetadataEntry {
    const DISCRIMINATOR: u8;

    fn to_metadata_value(self) -> MetadataValue;

    fn from_metadata_value(entry: MetadataValue) -> Result<Self, MetadataConversionError>;
}

pub trait ToMetadataEntry {
    fn to_metadata_entry(self) -> Option<MetadataValue>;
}

pub trait SingleMetadataVal: MetadataVal {
    fn to_array_metadata_value(vec: Vec<Self>) -> MetadataValue;

    fn from_array_metadata_value(
        entry: MetadataValue,
    ) -> Result<Vec<Self>, MetadataConversionError>;
}

pub trait ArrayMetadataVal: MetadataVal {}

macro_rules! impl_metadata_val {
    ($rust_type:ty, $metadata_type:tt, $metadata_array_type:tt, $type_id:expr) => {
        impl MetadataVal for $rust_type {
            const DISCRIMINATOR: u8 = $type_id;

            fn to_metadata_value(self) -> MetadataValue {
                MetadataValue::$metadata_type(self)
            }

            fn from_metadata_value(entry: MetadataValue) -> Result<Self, MetadataConversionError> {
                match entry {
                    MetadataValue::$metadata_type(x) => Ok(x),
                    _ => Err(MetadataConversionError::UnexpectedType {
                        expected_type_id: Self::DISCRIMINATOR,
                        actual_type_id: SborEnum::<ScryptoCustomValueKind>::get_discriminator(
                            &entry,
                        ),
                    }),
                }
            }
        }

        impl ToMetadataEntry for $rust_type {
            fn to_metadata_entry(self) -> Option<MetadataValue> {
                Some(self.to_metadata_value())
            }
        }

        impl SingleMetadataVal for $rust_type {
            fn to_array_metadata_value(vec: Vec<Self>) -> MetadataValue {
                vec.to_metadata_value()
            }

            fn from_array_metadata_value(
                entry: MetadataValue,
            ) -> Result<Vec<Self>, MetadataConversionError> {
                Vec::<Self>::from_metadata_value(entry)
            }
        }

        impl MetadataVal for Vec<$rust_type> {
            const DISCRIMINATOR: u8 = METADATA_DISCRIMINATOR_ARRAY_BASE + $type_id;

            fn to_metadata_value(self) -> MetadataValue {
                MetadataValue::$metadata_array_type(self)
            }

            fn from_metadata_value(entry: MetadataValue) -> Result<Self, MetadataConversionError> {
                match entry {
                    MetadataValue::$metadata_array_type(x) => Ok(x),
                    _ => Err(MetadataConversionError::UnexpectedType {
                        expected_type_id: Self::DISCRIMINATOR,
                        actual_type_id: SborEnum::<ScryptoCustomValueKind>::get_discriminator(
                            &entry,
                        ),
                    }),
                }
            }
        }

        impl ArrayMetadataVal for Vec<$rust_type> {}

        impl ToMetadataEntry for Vec<$rust_type> {
            fn to_metadata_entry(self) -> Option<MetadataValue> {
                Some(self.to_metadata_value())
            }
        }

        impl ToMetadataEntry for &[$rust_type] {
            fn to_metadata_entry(self) -> Option<MetadataValue> {
                Some(<$rust_type as SingleMetadataVal>::to_array_metadata_value(
                    self.iter().cloned().collect(),
                ))
            }
        }

        impl<const N: usize> ToMetadataEntry for [$rust_type; N] {
            fn to_metadata_entry(self) -> Option<MetadataValue> {
                Some(<$rust_type as SingleMetadataVal>::to_array_metadata_value(
                    self.into_iter().collect(),
                ))
            }
        }

        impl<const N: usize> ToMetadataEntry for &[$rust_type; N] {
            fn to_metadata_entry(self) -> Option<MetadataValue> {
                Some(<$rust_type as SingleMetadataVal>::to_array_metadata_value(
                    self.iter().cloned().collect(),
                ))
            }
        }
    };
}

macro_rules! impl_metadata_val_alias {
    ($metadata_rust_type:ty, $(| <$($generic:tt),+> |)? $alias_rust_type:ty) => {
        impl$(<$($generic),+>)? ToMetadataEntry for $alias_rust_type {
            fn to_metadata_entry(self) -> Option<MetadataValue> {
                Some(<$metadata_rust_type as MetadataVal>::to_metadata_value(self.into()))
            }
        }

        impl$(<$($generic),+>)? ToMetadataEntry for Vec<$alias_rust_type> {
            fn to_metadata_entry(self) -> Option<MetadataValue> {
                Some(<$metadata_rust_type as SingleMetadataVal>::to_array_metadata_value(
                    self.into_iter().map(Into::into).collect()
                ))
            }
        }

        impl$(<$($generic),+>)? ToMetadataEntry for &[$alias_rust_type] {
            fn to_metadata_entry(self) -> Option<MetadataValue> {
                Some(<$metadata_rust_type as SingleMetadataVal>::to_array_metadata_value(
                    self.iter().map(|x| (*x).into()).collect()
                ))
            }
        }

        impl<$($($generic,)+)? const N: usize> ToMetadataEntry for [$alias_rust_type; N] {
            fn to_metadata_entry(self) -> Option<MetadataValue> {
                Some(<$metadata_rust_type as SingleMetadataVal>::to_array_metadata_value(
                    self.iter().map(|x| (*x).into()).collect()
                ))
            }
        }

        impl<$($($generic,)+)? const N: usize> ToMetadataEntry for &[$alias_rust_type; N] {
            fn to_metadata_entry(self) -> Option<MetadataValue> {
                Some(<$metadata_rust_type as SingleMetadataVal>::to_array_metadata_value(
                    self.iter().map(|x| (*x).into()).collect()
                ))
            }
        }
    };
}

impl_metadata_val!(
    String,
    String,
    StringArray,
    METADATA_VALUE_STRING_DISCRIMINATOR
);
impl_metadata_val!(bool, Bool, BoolArray, METADATA_VALUE_BOOLEAN_DISCRIMINATOR);
impl_metadata_val!(u8, U8, U8Array, METADATA_VALUE_U8_DISCRIMINATOR);
impl_metadata_val!(u32, U32, U32Array, METADATA_VALUE_U32_DISCRIMINATOR);
impl_metadata_val!(u64, U64, U64Array, METADATA_VALUE_U64_DISCRIMINATOR);
impl_metadata_val!(i32, I32, I32Array, METADATA_VALUE_I32_DISCRIMINATOR);
impl_metadata_val!(i64, I64, I64Array, METADATA_VALUE_I64_DISCRIMINATOR);
impl_metadata_val!(
    Decimal,
    Decimal,
    DecimalArray,
    METADATA_VALUE_DECIMAL_DISCRIMINATOR
);
impl_metadata_val!(
    GlobalAddress,
    GlobalAddress,
    GlobalAddressArray,
    METADATA_VALUE_GLOBAL_ADDRESS_DISCRIMINATOR
);
impl_metadata_val!(
    PublicKey,
    PublicKey,
    PublicKeyArray,
    METADATA_VALUE_PUBLIC_KEY_DISCRIMINATOR
);
impl_metadata_val!(
    NonFungibleGlobalId,
    NonFungibleGlobalId,
    NonFungibleGlobalIdArray,
    METADATA_VALUE_NON_FUNGIBLE_GLOBAL_ID_DISCRIMINATOR
);
impl_metadata_val!(
    NonFungibleLocalId,
    NonFungibleLocalId,
    NonFungibleLocalIdArray,
    METADATA_VALUE_NON_FUNGIBLE_LOCAL_ID_DISCRIMINATOR
);
impl_metadata_val!(
    Instant,
    Instant,
    InstantArray,
    METADATA_VALUE_INSTANT_DISCRIMINATOR
);
impl_metadata_val!(
    UncheckedUrl,
    Url,
    UrlArray,
    METADATA_VALUE_URL_DISCRIMINATOR
);
impl_metadata_val!(
    UncheckedOrigin,
    Origin,
    OriginArray,
    METADATA_VALUE_ORIGIN_DISCRIMINATOR
);
impl_metadata_val!(
    PublicKeyHash,
    PublicKeyHash,
    PublicKeyHashArray,
    METADATA_VALUE_PUBLIC_KEY_HASH_DISCRIMINATOR
);

// Additional to metadata value implementations

impl_metadata_val_alias!(String, |<'a>| &'a str);
impl_metadata_val_alias!(GlobalAddress, ComponentAddress);
impl_metadata_val_alias!(GlobalAddress, ResourceAddress);
impl_metadata_val_alias!(GlobalAddress, PackageAddress);

impl ToMetadataEntry for MetadataValue {
    fn to_metadata_entry(self) -> Option<MetadataValue> {
        Some(self)
    }
}

impl ToMetadataEntry for Option<MetadataValue> {
    fn to_metadata_entry(self) -> Option<MetadataValue> {
        self
    }
}

pub type MetadataInit = KeyValueStoreInit<String, MetadataValue>;

impl MetadataInit {
    pub fn set_metadata<S: ToString, V: ToMetadataEntry>(&mut self, key: S, value: V) {
        match value.to_metadata_entry() {
            None => {}
            Some(value) => {
                self.set(key.to_string(), value);
            }
        }
    }

    pub fn set_and_lock_metadata<S: ToString, V: ToMetadataEntry>(&mut self, key: S, value: V) {
        match value.to_metadata_entry() {
            None => {
                self.lock_empty(key.to_string());
            }
            Some(value) => {
                self.set_and_lock(key.to_string(), value);
            }
        }
    }
}

impl From<BTreeMap<String, MetadataValue>> for MetadataInit {
    fn from(data: BTreeMap<String, MetadataValue>) -> Self {
        let mut metadata_init = MetadataInit::new();
        for (key, value) in data {
            metadata_init.set(key, value);
        }
        metadata_init
    }
}

#[macro_export]
macro_rules! metadata_init_set_entry {
    ($metadata:expr, $key:expr, $value:expr, updatable) => {{
        $metadata.set_metadata($key, $value);
    }};
    ($metadata:expr, $key:expr, $value:expr, locked) => {{
        $metadata.set_and_lock_metadata($key, $value);
    }};
}

#[macro_export]
macro_rules! metadata_init {
    () => ({
        $crate::object_modules::metadata::MetadataInit::new()
    });
    ( $($key:expr => $value:expr, $lock:ident;)* ) => ({
        let mut metadata_init = $crate::object_modules::metadata::MetadataInit::new();
        $(
            $crate::metadata_init_set_entry!(metadata_init, $key, $value, $lock);
        )*
        metadata_init
    });
}

#[macro_export]
macro_rules! metadata {
    {} => ({
        ModuleConfig {
            init: metadata_init!(),
            roles: RoleAssignmentInit::default(),
        }
    });
    {
        roles {
            $($role:ident => $rule:expr;)*
        },
        init {
            $($key:expr => $value:expr, $locked:ident;)*
        }
    } => ({
        let metadata_roles = metadata_roles!($($role => $rule;)*);
        let metadata = metadata_init!($($key => $value, $locked;)*);
        ModuleConfig {
            init: metadata,
            roles: metadata_roles
        }
    });

    {
        init {
            $($key:expr => $value:expr, $locked:ident;)*
        }
    } => ({
        let metadata = metadata_init!($($key => $value, $locked;)*);
        ModuleConfig {
            init: metadata,
            roles: RoleAssignmentInit::new(),
        }
    });

    {
        roles {
            $($role:ident => $rule:expr;)*
        }
    } => ({
        let roles = metadata_roles!($($role => $rule;)*);
        ModuleConfig {
            init: metadata_init!(),
            roles,
        }
    });
}

#[cfg(test)]
mod tests {
    use radix_common::prelude::*;

    use super::*;

    #[test]
    pub fn can_encode_and_decode_all_metadata_values() {
        encode_decode(&["Hello".to_string(), "world!".to_string()]);
        encode_decode(&[true, false]);
        encode_decode(&[1u8, 2u8]);
        encode_decode(&[2u32, 3u32]);
        encode_decode(&[3u64, 4u64]);
        encode_decode(&[4i32, 5i32]);
        encode_decode(&[5i64, 6i64]);
        encode_decode(&[dec!("1"), dec!("2.1")]);
        encode_decode(&[GlobalAddress::from(XRD)]);
        encode_decode(&[
            PublicKey::Ed25519(Ed25519PublicKey([0; Ed25519PublicKey::LENGTH])),
            PublicKey::Secp256k1(Secp256k1PublicKey([0; Secp256k1PublicKey::LENGTH])),
        ]);
        encode_decode(&[NonFungibleGlobalId::package_of_direct_caller_badge(
            POOL_PACKAGE,
        )]);
        encode_decode(&[
            NonFungibleLocalId::String(StringNonFungibleLocalId::new("Hello_world").unwrap()),
            NonFungibleLocalId::Integer(IntegerNonFungibleLocalId::new(42)),
            NonFungibleLocalId::Bytes(BytesNonFungibleLocalId::new(vec![1u8]).unwrap()),
            NonFungibleLocalId::RUID(RUIDNonFungibleLocalId::new([1; 32])),
        ]);
        encode_decode(&[Instant {
            seconds_since_unix_epoch: 1687446137,
        }]);
        encode_decode(&[UncheckedUrl::of("https://www.radixdlt.com")]);
        encode_decode(&[UncheckedOrigin::of("https://www.radixdlt.com")]);
        encode_decode(&[
            PublicKeyHash::Ed25519(Ed25519PublicKey([0; Ed25519PublicKey::LENGTH]).get_hash()),
            PublicKeyHash::Secp256k1(
                Secp256k1PublicKey([0; Secp256k1PublicKey::LENGTH]).get_hash(),
            ),
        ]);
    }

    fn encode_decode<T: SingleMetadataVal + Clone>(values: &[T]) {
        check_can_encode_decode(values[0].clone().to_metadata_value());
        check_can_encode_decode(T::to_array_metadata_value(values.to_vec()));
    }

    fn check_can_encode_decode(value: MetadataValue) {
        let bytes = scrypto_encode(&value).unwrap();
        // The outputting of bytes is for test vectors for other impls (eg the Gateway)
        #[cfg(not(feature = "alloc"))]
        println!("{}", hex::encode(&bytes));
        let decoded_value: MetadataValue = scrypto_decode(&bytes).unwrap();
        assert_eq!(decoded_value, value);
    }
}
