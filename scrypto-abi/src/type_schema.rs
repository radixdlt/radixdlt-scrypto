use crate::v2::*;
use sbor::rust::collections::{IndexMap, IndexSet};
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::*;

#[allow(dead_code)]
type ScryptoTypeSchema<TypeLink> = TypeSchema<ScryptoCustomTypeSchema<TypeLink>, TypeLink>;

/// A schema for the values that a codec can decode / views as valid
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type")  // See https://serde.rs/enum-representations.html
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeSchema<C: CustomTypeSchema, L: TypeLink> {
    Any,

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
        element_type: L,
        length_validation: LengthValidation,
    },

    Tuple {
        element_types: Vec<L>,
    },

    Enum {
        variants: IndexMap<String, L>,
    },

    // Custom Types
    Custom(C),
}

/// Marker trait for a link between TypeSchemas:
/// - TypeRef: A global identifier for a type (well known type, or type hash)
/// - SchemaLocalTypeLink: A link in the context of a schema
pub trait TypeLink: Clone + PartialEq + Eq {}

pub trait CustomTypeSchema: Clone + PartialEq + Eq {
    type CustomTypeId: CustomTypeId;
}

// This should be implemented on CustomTypeSchema<ComplexTypeHash>
pub trait LinearizableCustomTypeSchema: CustomTypeSchema {
    type Linearized: CustomTypeSchema;

    fn linearize(self, schemas: &IndexSet<ComplexTypeHash>) -> Self::Linearized;
}

/// A schema for the values that a codec can decode / views as valid
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type")  // See https://serde.rs/enum-representations.html
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScryptoCustomTypeSchema<L: TypeLink> {
    // Global address types
    PackageAddress,
    ComponentAddress,
    ResourceAddress,
    SystemAddress,

    // RE nodes types
    Component,
    KeyValueStore { key_type: L, value_type: L },
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

impl<L: TypeLink> CustomTypeSchema for ScryptoCustomTypeSchema<L> {
    type CustomTypeId = NoCustomTypeId; // Fix this to be Scrypto
}

impl LinearizableCustomTypeSchema for ScryptoCustomTypeSchema<GlobalTypeRef> {
    type Linearized = ScryptoCustomTypeSchema<SchemaLocalTypeRef>;

    fn linearize(self, schemas: &IndexSet<ComplexTypeHash>) -> Self::Linearized {
        match self {
            Self::PackageAddress => ScryptoCustomTypeSchema::PackageAddress,
            Self::ComponentAddress => ScryptoCustomTypeSchema::ComponentAddress,
            Self::ResourceAddress => ScryptoCustomTypeSchema::ResourceAddress,
            Self::SystemAddress => ScryptoCustomTypeSchema::SystemAddress,
            Self::Component => ScryptoCustomTypeSchema::Component,
            Self::KeyValueStore {
                key_type,
                value_type,
            } => ScryptoCustomTypeSchema::KeyValueStore {
                key_type: resolve_local_type_ref(schemas, &key_type),
                value_type: resolve_local_type_ref(schemas, &value_type),
            },
            Self::Bucket => ScryptoCustomTypeSchema::Bucket,
            Self::Proof => ScryptoCustomTypeSchema::Proof,
            Self::Vault => ScryptoCustomTypeSchema::Vault,
            Self::Expression => ScryptoCustomTypeSchema::Expression,
            Self::Blob => ScryptoCustomTypeSchema::Blob,
            Self::NonFungibleAddress => ScryptoCustomTypeSchema::NonFungibleAddress,
            Self::Hash => ScryptoCustomTypeSchema::Hash,
            Self::EcdsaSecp256k1PublicKey => ScryptoCustomTypeSchema::EcdsaSecp256k1PublicKey,
            Self::EcdsaSecp256k1Signature => ScryptoCustomTypeSchema::EcdsaSecp256k1Signature,
            Self::EddsaEd25519PublicKey => ScryptoCustomTypeSchema::EddsaEd25519PublicKey,
            Self::EddsaEd25519Signature => ScryptoCustomTypeSchema::EddsaEd25519Signature,
            Self::Decimal => ScryptoCustomTypeSchema::Decimal,
            Self::PreciseDecimal => ScryptoCustomTypeSchema::PreciseDecimal,
            Self::NonFungibleId => ScryptoCustomTypeSchema::NonFungibleId,
        }
    }
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
