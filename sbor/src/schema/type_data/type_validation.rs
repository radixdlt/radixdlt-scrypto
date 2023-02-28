use crate::*;

/// Additional validation to apply to a payload of the given type, beyond validation from the [`TypeKind`]'s type structure.
///
/// Each [`TypeKind`] typically can have either `None` or its type-specific validation.
#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
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

    String(LengthValidation),

    Array(LengthValidation),
    Map(LengthValidation),

    Custom(V),
}

/// Represents additional validation that should be performed on the size.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Sbor)]
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

    pub fn is_valid(&self, length: usize) -> bool {
        self.min.unwrap_or(0) as usize <= length && length <= self.max.unwrap_or(u32::MAX) as usize
    }
}

/// Represents additional validation that should be performed on the numeric value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Sbor)]
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

impl NumericValidation<i8> {
    pub fn is_valid(&self, value: i8) -> bool {
        self.min.unwrap_or(i8::MIN) <= value && value <= self.max.unwrap_or(i8::MAX)
    }
}

impl NumericValidation<i16> {
    pub fn is_valid(&self, value: i16) -> bool {
        self.min.unwrap_or(i16::MIN) <= value && value <= self.max.unwrap_or(i16::MAX)
    }
}

impl NumericValidation<i32> {
    pub fn is_valid(&self, value: i32) -> bool {
        self.min.unwrap_or(i32::MIN) <= value && value <= self.max.unwrap_or(i32::MAX)
    }
}

impl NumericValidation<i64> {
    pub fn is_valid(&self, value: i64) -> bool {
        self.min.unwrap_or(i64::MIN) <= value && value <= self.max.unwrap_or(i64::MAX)
    }
}

impl NumericValidation<i128> {
    pub fn is_valid(&self, value: i128) -> bool {
        self.min.unwrap_or(i128::MIN) <= value && value <= self.max.unwrap_or(i128::MAX)
    }
}

impl NumericValidation<u8> {
    pub fn is_valid(&self, value: u8) -> bool {
        self.min.unwrap_or(u8::MIN) <= value && value <= self.max.unwrap_or(u8::MAX)
    }
}

impl NumericValidation<u16> {
    pub fn is_valid(&self, value: u16) -> bool {
        self.min.unwrap_or(u16::MIN) <= value && value <= self.max.unwrap_or(u16::MAX)
    }
}

impl NumericValidation<u32> {
    pub fn is_valid(&self, value: u32) -> bool {
        self.min.unwrap_or(u32::MIN) <= value && value <= self.max.unwrap_or(u32::MAX)
    }
}

impl NumericValidation<u64> {
    pub fn is_valid(&self, value: u64) -> bool {
        self.min.unwrap_or(u64::MIN) <= value && value <= self.max.unwrap_or(u64::MAX)
    }
}

impl NumericValidation<u128> {
    pub fn is_valid(&self, value: u128) -> bool {
        self.min.unwrap_or(u128::MIN) <= value && value <= self.max.unwrap_or(u128::MAX)
    }
}
