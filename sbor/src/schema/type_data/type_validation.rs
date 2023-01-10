use crate::*;

/// Additional validation to apply to a payload of the given type, beyond validation from the [`TypeKind`]'s type structure.
///
/// Each [`TypeKind`] typically can have either `None` or its type-specific validation.
#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, Categorize)]
pub enum TypeValidation<V: CustomTypeValidation> {
    None,

    I8(NumericValidation<i8>),
    I16(NumericValidation<i16>),
    I32(NumericValidation<i32>),
    I64(NumericValidation<i64>),
    I128(NumericValidation<i128>),
    U8(NumericValidation<u8>),
    U16(NumericValidation<u16>),
    U32(NumericValidation<u32>),
    U64(NumericValidation<u64>),
    U128(NumericValidation<u128>),
    String { length_validation: LengthValidation },
    Array { length_validation: LengthValidation },

    Custom(V),
}

/// Represents additional validation that should be performed on the size.
#[derive(Debug, Clone, PartialEq, Eq, Categorize, Decode, Encode, Default)]
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
#[derive(Debug, Clone, PartialEq, Eq, Default, Categorize, Encode, Decode)]
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
