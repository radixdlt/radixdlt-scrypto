use crate::*;

/// Additional validation to apply to a payload of the given type, beyond validation from the [`TypeKind`]'s type structure.
///
/// Each [`TypeKind`] typically can have either `None` or its type-specific validation.
#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
pub enum TypeValidation<E: CustomTypeValidation> {
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

    Custom(E),
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

    pub fn compare(base: &Self, compared: &Self) -> ValidationChange {
        // Massage it into an equivalent numeric validation comparison
        NumericValidation::compare(
            &NumericValidation::with_bounds(base.min, base.max),
            &NumericValidation::with_bounds(compared.min, compared.max),
        )
    }
}

/// Represents additional validation that should be performed on the numeric value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Sbor)]
pub struct NumericValidation<T: NumericValidationBound> {
    pub min: Option<T>,
    pub max: Option<T>,
}

impl<T: NumericValidationBound> NumericValidation<T> {
    pub const fn with_bounds(min: Option<T>, max: Option<T>) -> Self {
        Self { min, max }
    }

    pub const fn none() -> Self {
        Self {
            min: None,
            max: None,
        }
    }

    pub fn compare(base: &Self, compared: &Self) -> ValidationChange {
        // Slight warning - `compare` takes the opposite argument order to `cmp`.
        // This is to be consistent with the schema comparison arg ordering.
        let min_change = match compared.effective_min().cmp(&base.effective_min()) {
            core::cmp::Ordering::Less => ValidationChange::Weakened, // Min has decreased
            core::cmp::Ordering::Equal => ValidationChange::Unchanged,
            core::cmp::Ordering::Greater => ValidationChange::Strengthened, // Min has increased
        };
        let max_change = match compared.effective_max().cmp(&base.effective_max()) {
            core::cmp::Ordering::Less => ValidationChange::Strengthened, // Max has decreased
            core::cmp::Ordering::Equal => ValidationChange::Unchanged,
            core::cmp::Ordering::Greater => ValidationChange::Weakened, // Max has increased
        };
        ValidationChange::combine(min_change, max_change)
    }

    pub fn is_valid(&self, value: T) -> bool {
        self.effective_min() <= value && value <= self.effective_max()
    }

    pub fn effective_min(&self) -> T {
        self.min.unwrap_or(T::MIN_VALUE)
    }

    pub fn effective_max(&self) -> T {
        self.max.unwrap_or(T::MAX_VALUE)
    }
}

pub trait NumericValidationBound: Ord + Copy {
    const MAX_VALUE: Self;
    const MIN_VALUE: Self;
}

impl NumericValidationBound for i8 {
    const MAX_VALUE: Self = Self::MAX;
    const MIN_VALUE: Self = Self::MIN;
}

impl NumericValidationBound for i16 {
    const MAX_VALUE: Self = Self::MAX;
    const MIN_VALUE: Self = Self::MIN;
}

impl NumericValidationBound for i32 {
    const MAX_VALUE: Self = Self::MAX;
    const MIN_VALUE: Self = Self::MIN;
}

impl NumericValidationBound for i64 {
    const MAX_VALUE: Self = Self::MAX;
    const MIN_VALUE: Self = Self::MIN;
}

impl NumericValidationBound for i128 {
    const MAX_VALUE: Self = Self::MAX;
    const MIN_VALUE: Self = Self::MIN;
}

impl NumericValidationBound for u8 {
    const MAX_VALUE: Self = Self::MAX;
    const MIN_VALUE: Self = Self::MIN;
}

impl NumericValidationBound for u16 {
    const MAX_VALUE: Self = Self::MAX;
    const MIN_VALUE: Self = Self::MIN;
}

impl NumericValidationBound for u32 {
    const MAX_VALUE: Self = Self::MAX;
    const MIN_VALUE: Self = Self::MIN;
}

impl NumericValidationBound for u64 {
    const MAX_VALUE: Self = Self::MAX;
    const MIN_VALUE: Self = Self::MIN;
}

impl NumericValidationBound for u128 {
    const MAX_VALUE: Self = Self::MAX;
    const MIN_VALUE: Self = Self::MIN;
}
