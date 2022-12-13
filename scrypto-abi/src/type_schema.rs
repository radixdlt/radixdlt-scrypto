use indexmap::IndexMap;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::*;

/// A schema for the values that a codec can decode / views as valid
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type")  // See https://serde.rs/enum-representations.html
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeSchema<T> {
    Any,

    // FIXED BASIC TYPES

    // Simple Types
    Unit,
    Bool,
    I8 {
        validation: NumericValidation<i8>,
    },
    I16 {
        validation: NumericValidation<i16>,
    },
    I32 {
        validation: NumericValidation<i32>,
    },
    I64 {
        validation: NumericValidation<i64>,
    },
    I128 {
        validation: NumericValidation<i128>,
    },
    U8 {
        validation: NumericValidation<u8>,
    },
    U16 {
        validation: NumericValidation<u16>,
    },
    U32 {
        validation: NumericValidation<u32>,
    },
    U64 {
        validation: NumericValidation<u64>,
    },
    U128 {
        validation: NumericValidation<u128>,
    },
    String {
        length_validation: LengthValidation,
    },

    // Composite Types
    Array {
        element_sbor_type_id: u8,
        element_type: T,
        length_validation: LengthValidation,
    },

    Tuple {
        element_types: Vec<T>,
    },

    Enum {
        variants: IndexMap<String, T>,
    },

    // CUSTOM TYPES

    // Global address types
    PackageAddress,
    ComponentAddress,
    ResourceAddress,
    SystemAddress,

    // RE nodes types
    Component,
    KeyValueStore {
        key_type: T,
        value_type: T,
    },
    Bucket,
    Proof,
    Vault,

    // Other interpreted types
    Expression,
    Blob,
    NonFungibleAddress,

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

/// Represents additional validation that should be performed on the size.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, TypeId, Decode, Encode, Default)]
pub struct LengthValidation {
    pub min: Option<u32>,
    pub max: Option<u32>,
}

impl LengthValidation {
    pub const fn none() -> Self {
        Self {
            min: None,
            max: None,
        }
    }
}

/// Represents additional validation that should be performed on the numeric value.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NumericValidation<T: Clone + PartialEq + Eq> {
    pub min: Option<T>,
    pub max: Option<T>,
}

impl<T: Clone + PartialEq + Eq> NumericValidation<T> {
    pub const fn none() -> Self {
        Self {
            min: None,
            max: None,
        }
    }
}
