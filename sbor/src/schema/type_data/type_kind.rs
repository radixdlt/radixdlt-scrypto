use super::*;
use crate::rust::collections::BTreeMap;
use crate::rust::string::String;
use crate::rust::vec::Vec;

/// A schema for the values that a codec can decode / views as valid
#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, TypeId)]
#[sbor(custom_type_id = "X")]
pub enum TypeKind<
    X: CustomTypeId,
    C: CustomTypeKind<L, CustomTypeId = X>,
    L: SchemaTypeLink + TypeId<X>,
> {
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
        element_type: L,
        length_validation: LengthValidation,
    },

    Tuple {
        field_types: Vec<L>,
    },

    Enum {
        variants: BTreeMap<String, Vec<L>>,
    },

    // Custom Types
    Custom(C),
}
