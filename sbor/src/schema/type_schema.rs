use super::*;
use sbor::rust::collections::{IndexMap, IndexSet};
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::*;

/// A schema for the values that a codec can decode / views as valid
#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, TypeId)]
#[sbor(custom_type_id = "X")]
pub enum TypeSchema<X: CustomTypeId, C: CustomTypeSchema<CustomTypeId = X>, L: TypeLink + TypeId<X>>
{
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
    type Linearized: CustomTypeSchema<CustomTypeId = Self::CustomTypeId>;

    fn linearize(self, schemas: &IndexSet<ComplexTypeHash>) -> Self::Linearized;
}

/// Represents additional validation that should be performed on the size.
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
#[derive(Debug, Clone, PartialEq, Eq, Default, TypeId, Encode, Decode)]
pub struct NumericValidation<T> {
    pub min: Option<T>,
    pub max: Option<T>,
}

impl<T> NumericValidation<T> {
    pub const fn none() -> Self {
        Self {
            min: None,
            max: None,
        }
    }
}
