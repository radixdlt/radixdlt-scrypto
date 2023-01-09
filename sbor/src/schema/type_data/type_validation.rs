use crate::*;

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
