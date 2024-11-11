use crate::internal_prelude::*;
#[cfg(feature = "fuzzing")]
use arbitrary::Arbitrary;
use core::cmp::Ordering;
use core::ops::*;
use num_bigint::BigInt;
use num_traits::{Pow, Zero};
#[cfg(feature = "fuzzing")]
use serde::{Deserialize, Serialize};

use crate::data::manifest::ManifestCustomValueKind;
use crate::data::scrypto::*;
use crate::math::bnum_integer::*;
use crate::math::rounding_mode::*;
use crate::math::traits::*;
use crate::math::PreciseDecimal;
use crate::well_known_scrypto_custom_type;
use crate::*;

use super::CheckedTruncate;

/// `Decimal` represents a 192 bit representation of a fixed-scale decimal number.
///
/// The finite set of values are of the form `m / 10^18`, where `m` is
/// an integer such that `-2^(192 - 1) <= m < 2^(192 - 1)`.
///
/// ```text
/// Fractional part: ~60 bits/18 digits
/// Integer part   : 132 bits /40 digits
/// Max            :  3138550867693340381917894711603833208051.177722232017256447
/// Min            : -3138550867693340381917894711603833208051.177722232017256448
/// ```
///
/// Unless otherwise specified, all operations will panic if there is underflow/overflow.
///
/// To create a Decimal with a certain number of `10^(-18)` subunits, use
/// [`Decimal::from_attos`] or equivalently [`Decimal::from_subunits`].
#[cfg_attr(feature = "fuzzing", derive(Arbitrary, Serialize, Deserialize))]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Decimal(InnerDecimal);

pub type InnerDecimal = I192;

impl Default for Decimal {
    fn default() -> Self {
        Self::zero()
    }
}

// TODO come up with some smarter formatting depending on Decimal::Scale
macro_rules! fmt_remainder {
    () => {
        "{:018}"
    };
}

impl Decimal {
    /// The min value of `Decimal`.
    pub const MIN: Self = Self(I192::MIN);

    /// The max value of `Decimal`.
    pub const MAX: Self = Self(I192::MAX);

    /// The bit length of number storing `Decimal`.
    pub const BITS: usize = I192::BITS as usize;

    /// The fixed scale used by `Decimal`.
    pub const SCALE: u32 = 18;

    pub const ZERO: Self = Self(I192::ZERO);

    pub const ONE_ATTO: Self = Self(I192::ONE);
    pub const ONE_SUBUNIT: Self = Self::ONE_ATTO;
    pub const ONE_HUNDREDTH: Self = Self(I192::from_digits([10_u64.pow(Decimal::SCALE - 2), 0, 0]));
    pub const ONE_TENTH: Self = Self(I192::from_digits([10_u64.pow(Decimal::SCALE - 1), 0, 0]));
    pub const ONE: Self = Self(I192::from_digits([10_u64.pow(Decimal::SCALE), 0, 0]));
    pub const TEN: Self = Self(I192::from_digits([10_u64.pow(Decimal::SCALE + 1), 0, 0]));
    pub const ONE_HUNDRED: Self = Self(I192::from_digits([7766279631452241920, 0x5, 0]));

    /// Constructs a [`Decimal`] from its underlying `10^(-18)` subunits.
    pub const fn from_attos(attos: I192) -> Self {
        Self(attos)
    }

    /// Constructs a [`Decimal`] from its underlying `10^(-18)` subunits.
    ///
    /// This is an alias of [`from_attos`][Self::from_attos], for consistency with [`PreciseDecimal::from_precise_subunits`].
    pub const fn from_subunits(subunits: I192) -> Self {
        Self(subunits)
    }

    /// Returns the underlying `10^(-18)` subunits of the [`Decimal`].
    pub const fn attos(self) -> I192 {
        self.0
    }

    /// Returns the underlying `10^(-18)` subunits of the [`Decimal`].
    ///
    /// This is an alias of [`attos`][Self::attos], for consistency with [`PreciseDecimal::precise_subunits`].
    pub const fn subunits(self) -> I192 {
        self.0
    }

    /// Returns a [`Decimal`] with value 0.
    pub const fn zero() -> Self {
        Self::ZERO
    }

    /// Returns a [`Decimal`] with value 1.
    pub const fn one() -> Self {
        Self::ONE
    }

    /// Whether this value is zero.
    pub fn is_zero(&self) -> bool {
        self.0 == I192::ZERO
    }

    /// Whether this value is positive.
    pub fn is_positive(&self) -> bool {
        self.0 > I192::ZERO
    }

    /// Whether this value is negative.
    pub fn is_negative(&self) -> bool {
        self.0 < I192::ZERO
    }

    /// Returns the absolute value.
    pub fn checked_abs(&self) -> Option<Self> {
        if *self != Self::MIN {
            Some(Self(self.0.abs()))
        } else {
            None
        }
    }

    /// Returns the largest integer that is equal to or less than this number.
    pub fn checked_floor(&self) -> Option<Self> {
        self.checked_round(0, RoundingMode::ToNegativeInfinity)
    }

    /// Returns the smallest integer that is equal to or greater than this number.
    pub fn checked_ceiling(&self) -> Option<Self> {
        self.checked_round(0, RoundingMode::ToPositiveInfinity)
    }

    /// Rounds this number to the specified decimal places.
    ///
    /// # Panics
    /// - Panic if the number of decimal places is not within [0..SCALE]
    pub fn checked_round<T: Into<i32>>(
        &self,
        decimal_places: T,
        mode: RoundingMode,
    ) -> Option<Self> {
        let decimal_places = decimal_places.into();
        assert!(decimal_places <= Self::SCALE as i32);
        assert!(decimal_places >= 0);

        let n = Self::SCALE - decimal_places as u32;
        let divisor: I192 = I192::TEN.pow(n);
        let positive_remainder = {
            // % is the "C" style remainder operator, rather than the mathematical modulo operator,
            // So we fix that here https://internals.rust-lang.org/t/mathematical-modulo-operator/5952
            let remainder = self.0 % divisor;
            match remainder.cmp(&I192::ZERO) {
                Ordering::Less => divisor + remainder,
                Ordering::Equal => return Some(*self),
                Ordering::Greater => remainder,
            }
        };

        let resolved_strategy =
            ResolvedRoundingStrategy::from_mode(mode, self.is_positive(), || {
                let midpoint = divisor >> 1; // Half the divisor
                positive_remainder.cmp(&midpoint)
            });

        let rounded_subunits = match resolved_strategy {
            ResolvedRoundingStrategy::RoundUp => {
                let to_add = divisor
                    .checked_sub(positive_remainder)
                    .expect("Always safe");
                self.0.checked_add(to_add)?
            }
            ResolvedRoundingStrategy::RoundDown => self.0.checked_sub(positive_remainder)?,
            ResolvedRoundingStrategy::RoundToEven => {
                let double_divisor = divisor << 1; // Double the divisor
                if self.is_positive() {
                    // If positive, we try rounding down first (to avoid accidental overflow)
                    let rounded_down = self.0.checked_sub(positive_remainder)?;
                    if rounded_down % double_divisor == I192::ZERO {
                        rounded_down
                    } else {
                        rounded_down.checked_add(divisor)?
                    }
                } else {
                    // If negative, we try rounding up first (to avoid accidental overflow)
                    let to_add = divisor
                        .checked_sub(positive_remainder)
                        .expect("Always safe");
                    let rounded_up = self.0.checked_add(to_add)?;
                    if rounded_up % double_divisor == I192::ZERO {
                        rounded_up
                    } else {
                        rounded_up.checked_sub(divisor)?
                    }
                }
            }
        };

        Some(Self(rounded_subunits))
    }

    /// Calculates power using exponentiation by squaring".
    pub fn checked_powi(&self, exp: i64) -> Option<Self> {
        let one_256 = I256::from(Self::ONE.0);
        let base_256 = I256::from(self.0);
        let div = |x: i64, y: i64| x.checked_div(y);
        let sub = |x: i64, y: i64| x.checked_sub(y);
        let mul = |x: i64, y: i64| x.checked_mul(y);

        if exp < 0 {
            let dec_192 = I192::try_from((one_256 * one_256).checked_div(base_256)?).ok()?;
            let exp = mul(exp, -1)?;
            return Self(dec_192).checked_powi(exp);
        }
        if exp == 0 {
            return Some(Self::ONE);
        }
        if exp == 1 {
            return Some(*self);
        }
        if exp % 2 == 0 {
            let dec_192 = I192::try_from(base_256.checked_mul(base_256)? / one_256).ok()?;
            let exp = div(exp, 2)?;
            Self(dec_192).checked_powi(exp)
        } else {
            let dec_192 = I192::try_from(base_256.checked_mul(base_256)? / one_256).ok()?;
            let sub_dec = Self(dec_192);
            let exp = div(sub(exp, 1)?, 2)?;
            let b = sub_dec.checked_powi(exp)?;
            self.checked_mul(b)
        }
    }

    /// Square root of a Decimal
    pub fn checked_sqrt(&self) -> Option<Self> {
        if self.is_negative() {
            return None;
        }
        if self.is_zero() {
            return Some(Self::ZERO);
        }

        // The I192 i associated to a Decimal d is : i = d*10^18.
        // Therefore, taking sqrt yields sqrt(i) = sqrt(d)*10^9 => We lost precision
        // To get the right precision, we compute : sqrt(i*10^18) = sqrt(d)*10^18
        let self_256 = I256::from(self.0);
        let correct_nb = self_256 * I256::from(Self::ONE.0);
        let sqrt = I192::try_from(correct_nb.sqrt()).ok()?;
        Some(Self(sqrt))
    }

    /// Cubic root of a Decimal
    pub fn checked_cbrt(&self) -> Option<Self> {
        if self.is_zero() {
            return Some(Self::ZERO);
        }

        // By reasoning in the same way as before, we realise that we need to multiply by 10^36
        let self_320 = I320::from(self.0);
        let correct_nb = self_320 * I320::from(Self::ONE.0).pow(2);
        let cbrt = I192::try_from(correct_nb.cbrt()).ok()?;
        Some(Self(cbrt))
    }

    /// Nth root of a Decimal
    pub fn checked_nth_root(&self, n: u32) -> Option<Self> {
        if (self.is_negative() && n % 2 == 0) || n == 0 {
            None
        } else if n == 1 {
            Some(*self)
        } else {
            if self.is_zero() {
                return Some(Self::ZERO);
            }

            // By induction, we need to multiply by the (n-1)th power of 10^18.
            // To not overflow, we use BigInts
            let self_bigint = BigInt::from(self.0);
            let correct_nb = self_bigint * BigInt::from(Self::ONE.0).pow(n - 1);
            let nth_root = I192::try_from(correct_nb.nth_root(n)).unwrap();
            Some(Decimal(nth_root))
        }
    }
}

macro_rules! from_primitive_type {
    ($($type:ident),*) => {
        $(
            impl From<$type> for Decimal {
                fn from(val: $type) -> Self {
                    Self(I192::from(val) * Self::ONE.0)
                }
            }
        )*
    };
}
macro_rules! to_primitive_type {
    ($($type:ident),*) => {
        $(
            impl TryFrom<Decimal> for $type {
                type Error = ParseDecimalError;

                fn try_from(val: Decimal) -> Result<Self, Self::Error> {
                    let rounded = val.checked_round(0, RoundingMode::ToZero).ok_or(ParseDecimalError::Overflow)?;
                    let fraction = val.checked_sub(rounded).ok_or(Self::Error::Overflow)?;
                    if !fraction.is_zero() {
                        Err(Self::Error::InvalidDigit)
                    }
                    else {
                        let i_192 = rounded.0 / I192::TEN.pow(Decimal::SCALE);
                        $type::try_from(i_192)
                            .map_err(|_| Self::Error::Overflow)
                    }
                }
            }

            impl TryFrom<&Decimal> for $type {
                type Error = ParseDecimalError;

                fn try_from(val: &Decimal) -> Result<Self, Self::Error> {
                    $type::try_from(*val)
                }
            }
        )*
    }
}

from_primitive_type!(i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);
to_primitive_type!(i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);

resolvable_with_try_into_impls!(Decimal);

// from_str() should be enough, but we want to have try_from() to simplify dec! macro
impl TryFrom<&str> for Decimal {
    type Error = ParseDecimalError;

    fn try_from(val: &str) -> Result<Self, Self::Error> {
        Self::from_str(val)
    }
}

impl TryFrom<String> for Decimal {
    type Error = ParseDecimalError;

    fn try_from(val: String) -> Result<Self, Self::Error> {
        Self::from_str(&val)
    }
}

impl From<bool> for Decimal {
    fn from(val: bool) -> Self {
        if val {
            Self::ONE
        } else {
            Self::ZERO
        }
    }
}

impl CheckedNeg<Decimal> for Decimal {
    type Output = Self;

    #[inline]
    fn checked_neg(self) -> Option<Self::Output> {
        let c = self.0.checked_neg();
        c.map(Self)
    }
}

impl CheckedAdd<Decimal> for Decimal {
    type Output = Self;

    #[inline]
    fn checked_add(self, other: Self) -> Option<Self::Output> {
        let a = self.0;
        let b = other.0;
        let c = a.checked_add(b);
        c.map(Self)
    }
}

impl SaturatingAdd<Decimal> for Decimal {
    type Output = Self;

    #[inline]
    fn saturating_add(self, other: Self) -> Self::Output {
        Self(self.0.saturating_add(other.0))
    }
}

impl CheckedSub<Decimal> for Decimal {
    type Output = Self;

    #[inline]
    fn checked_sub(self, other: Self) -> Option<Self::Output> {
        let a = self.0;
        let b = other.0;
        let c = a.checked_sub(b);
        c.map(Self)
    }
}

impl CheckedMul<Decimal> for Decimal {
    type Output = Self;

    #[inline]
    fn checked_mul(self, other: Self) -> Option<Self> {
        // Use I256 (BInt<4>) to not overflow.
        let a = I256::from(self.0);
        let b = I256::from(other.0);
        let mut c = a.checked_mul(b)?;
        c = c.checked_div(I256::from(Self::ONE.0))?;

        let c_192 = I192::try_from(c).ok();
        c_192.map(Self)
    }
}

impl CheckedDiv<Decimal> for Decimal {
    type Output = Self;

    #[inline]
    fn checked_div(self, other: Self) -> Option<Self> {
        // Use I256 (BInt<4>) to not overflow.
        let a = I256::from(self.0);
        let b = I256::from(other.0);
        let mut c = a.checked_mul(I256::from(Self::ONE.0))?;
        c = c.checked_div(b)?;

        let c_192 = I192::try_from(c).ok();
        c_192.map(Self)
    }
}

impl Neg for Decimal {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self::Output {
        self.checked_neg().expect("Overflow")
    }
}

impl Add<Decimal> for Decimal {
    type Output = Self;

    #[inline]
    fn add(self, other: Self) -> Self::Output {
        self.checked_add(other).expect("Overflow")
    }
}

impl Sub<Decimal> for Decimal {
    type Output = Self;

    #[inline]
    fn sub(self, other: Self) -> Self::Output {
        self.checked_sub(other).expect("Overflow")
    }
}

impl Mul<Decimal> for Decimal {
    type Output = Self;

    #[inline]
    fn mul(self, other: Self) -> Self::Output {
        self.checked_mul(other).expect("Overflow")
    }
}

impl Div<Decimal> for Decimal {
    type Output = Self;

    #[inline]
    fn div(self, other: Self) -> Self::Output {
        self.checked_div(other)
            .expect("Overflow or division by zero")
    }
}

impl AddAssign<Decimal> for Decimal {
    #[inline]
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl SubAssign<Decimal> for Decimal {
    #[inline]
    fn sub_assign(&mut self, other: Self) {
        *self = *self - other;
    }
}

impl MulAssign<Decimal> for Decimal {
    #[inline]
    fn mul_assign(&mut self, other: Self) {
        *self = *self * other;
    }
}

impl DivAssign<Decimal> for Decimal {
    #[inline]
    fn div_assign(&mut self, other: Self) {
        *self = *self / other;
    }
}

macro_rules! impl_arith_ops {
    ($type:ident) => {
        impl CheckedAdd<$type> for Decimal {
            type Output = Self;

            fn checked_add(self, other: $type) -> Option<Self::Output> {
                self.checked_add(Self::try_from(other).ok()?)
            }
        }

        impl CheckedSub<$type> for Decimal {
            type Output = Self;

            fn checked_sub(self, other: $type) -> Option<Self::Output> {
                self.checked_sub(Self::try_from(other).ok()?)
            }
        }

        impl CheckedMul<$type> for Decimal {
            type Output = Self;

            fn checked_mul(self, other: $type) -> Option<Self::Output> {
                self.checked_mul(Self::try_from(other).ok()?)
            }
        }

        impl CheckedDiv<$type> for Decimal {
            type Output = Self;

            fn checked_div(self, other: $type) -> Option<Self::Output> {
                self.checked_div(Self::try_from(other).ok()?)
            }
        }

        impl Add<$type> for Decimal {
            type Output = Self;

            #[inline]
            fn add(self, other: $type) -> Self::Output {
                self.checked_add(other).expect("Overflow")
            }
        }

        impl Sub<$type> for Decimal {
            type Output = Self;

            #[inline]
            fn sub(self, other: $type) -> Self::Output {
                self.checked_sub(other).expect("Overflow")
            }
        }

        impl Mul<$type> for Decimal {
            type Output = Self;

            #[inline]
            fn mul(self, other: $type) -> Self::Output {
                self.checked_mul(other).expect("Overflow")
            }
        }

        impl Div<$type> for Decimal {
            type Output = Self;

            #[inline]
            fn div(self, other: $type) -> Self::Output {
                self.checked_div(other)
                    .expect("Overflow or division by zero")
            }
        }

        impl Add<Decimal> for $type {
            type Output = Decimal;

            #[inline]
            fn add(self, other: Decimal) -> Self::Output {
                other + self
            }
        }

        impl Sub<Decimal> for $type {
            type Output = Decimal;

            #[inline]
            fn sub(self, other: Decimal) -> Self::Output {
                // Cannot use self.checked_sub directly.
                // It conflicts with already defined checked_sub for primitive types.
                Decimal::try_from(self)
                    .expect("Overflow")
                    .checked_sub(other)
                    .expect("Overflow")
            }
        }

        impl Mul<Decimal> for $type {
            type Output = Decimal;

            #[inline]
            fn mul(self, other: Decimal) -> Self::Output {
                other * self
            }
        }

        impl Div<Decimal> for $type {
            type Output = Decimal;

            #[inline]
            fn div(self, other: Decimal) -> Self::Output {
                // Cannot use self.checked_div directly.
                // It conflicts with already defined checked_sub for primitive types.
                Decimal::try_from(self)
                    .expect("Overflow")
                    .checked_div(other)
                    .expect("Overflow or division by zero")
            }
        }

        impl AddAssign<$type> for Decimal {
            #[inline]
            fn add_assign(&mut self, other: $type) {
                *self = *self + other;
            }
        }

        impl SubAssign<$type> for Decimal {
            #[inline]
            fn sub_assign(&mut self, other: $type) {
                *self = *self - other;
            }
        }

        impl MulAssign<$type> for Decimal {
            #[inline]
            fn mul_assign(&mut self, other: $type) {
                *self = *self * other;
            }
        }

        impl DivAssign<$type> for Decimal {
            #[inline]
            fn div_assign(&mut self, other: $type) {
                *self = *self / other;
            }
        }
    };
}
impl_arith_ops!(u8);
impl_arith_ops!(u16);
impl_arith_ops!(u32);
impl_arith_ops!(u64);
impl_arith_ops!(u128);
impl_arith_ops!(usize);
impl_arith_ops!(i8);
impl_arith_ops!(i16);
impl_arith_ops!(i32);
impl_arith_ops!(i64);
impl_arith_ops!(i128);
impl_arith_ops!(isize);
impl_arith_ops!(I192);
impl_arith_ops!(I256);
impl_arith_ops!(I320);
impl_arith_ops!(I448);
impl_arith_ops!(I512);
impl_arith_ops!(U192);
impl_arith_ops!(U256);
impl_arith_ops!(U320);
impl_arith_ops!(U448);
impl_arith_ops!(U512);

// Below implements CheckedX traits for given type with Decimal as an argument.
// It cannot be used for primitive types, since they already implement these traits
// but with different argument type.
macro_rules! impl_arith_ops_non_primitives {
    ($type:ident) => {
        impl CheckedAdd<Decimal> for $type {
            type Output = Decimal;

            #[inline]
            fn checked_add(self, other: Decimal) -> Option<Self::Output> {
                other.checked_add(self)
            }
        }

        impl CheckedSub<Decimal> for $type {
            type Output = Decimal;

            fn checked_sub(self, other: Decimal) -> Option<Self::Output> {
                Decimal::try_from(self).ok()?.checked_sub(other)
            }
        }

        impl CheckedMul<Decimal> for $type {
            type Output = Decimal;

            #[inline]
            fn checked_mul(self, other: Decimal) -> Option<Self::Output> {
                other.checked_mul(self)
            }
        }

        impl CheckedDiv<Decimal> for $type {
            type Output = Decimal;

            fn checked_div(self, other: Decimal) -> Option<Self::Output> {
                Decimal::try_from(self).ok()?.checked_div(other)
            }
        }
    };
}
impl_arith_ops_non_primitives!(I192);
impl_arith_ops_non_primitives!(I256);
impl_arith_ops_non_primitives!(I320);
impl_arith_ops_non_primitives!(I448);
impl_arith_ops_non_primitives!(I512);
impl_arith_ops_non_primitives!(U192);
impl_arith_ops_non_primitives!(U256);
impl_arith_ops_non_primitives!(U320);
impl_arith_ops_non_primitives!(U448);
impl_arith_ops_non_primitives!(U512);

//========
// binary
//========

impl TryFrom<&[u8]> for Decimal {
    type Error = ParseDecimalError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        if slice.len() == Self::BITS / 8 {
            let val = I192::try_from(slice).expect("Length should have already been checked.");
            Ok(Self(val))
        } else {
            Err(ParseDecimalError::InvalidLength(slice.len()))
        }
    }
}

impl Decimal {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_le_bytes().to_vec()
    }
}

well_known_scrypto_custom_type!(
    Decimal,
    ScryptoCustomValueKind::Decimal,
    Type::Decimal,
    Decimal::BITS / 8,
    DECIMAL_TYPE,
    decimal_type_data
);

manifest_type!(Decimal, ManifestCustomValueKind::Decimal, Decimal::BITS / 8);

//======
// text
//======

impl FromStr for Decimal {
    type Err = ParseDecimalError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let v: Vec<&str> = s.split('.').collect();

        if v.len() > 2 {
            return Err(ParseDecimalError::MoreThanOneDecimalPoint);
        }

        let integer_part = match I192::from_str(v[0]) {
            Ok(val) => val,
            Err(err) => match err {
                ParseI192Error::NegativeToUnsigned => {
                    unreachable!("NegativeToUnsigned is only for parsing unsigned types, not I192")
                }
                ParseI192Error::Overflow => return Err(ParseDecimalError::Overflow),
                ParseI192Error::InvalidLength => {
                    unreachable!("InvalidLength is only for parsing &[u8], not &str")
                }
                ParseI192Error::InvalidDigit => return Err(ParseDecimalError::InvalidDigit),
                // We have decided to be restrictive to force people to type "0.123" instead of ".123"
                // for clarity, and to align with how rust's float literal works
                ParseI192Error::Empty => return Err(ParseDecimalError::EmptyIntegralPart),
            },
        };

        let mut subunits = integer_part
            .checked_mul(Self::ONE.0)
            .ok_or(ParseDecimalError::Overflow)?;

        if v.len() == 2 {
            let scale = if let Some(scale) = Self::SCALE.checked_sub(v[1].len() as u32) {
                Ok(scale)
            } else {
                Err(Self::Err::MoreThanEighteenDecimalPlaces)
            }?;

            let fractional_part = match I192::from_str(v[1]) {
                Ok(val) => val,
                Err(err) => match err {
                    ParseI192Error::NegativeToUnsigned => {
                        unreachable!(
                            "NegativeToUnsigned is only for parsing unsigned types, no I192"
                        )
                    }
                    ParseI192Error::Overflow => return Err(ParseDecimalError::Overflow),
                    ParseI192Error::InvalidLength => {
                        unreachable!("InvalidLength is only for parsing &[u8], not &str")
                    }
                    ParseI192Error::InvalidDigit => return Err(ParseDecimalError::InvalidDigit),
                    ParseI192Error::Empty => return Err(ParseDecimalError::EmptyFractionalPart),
                },
            };

            // The product of these must be less than Self::SCALE
            let fractional_subunits = fractional_part
                .checked_mul(I192::TEN.pow(scale))
                .expect("No overflow possible");

            // if input is -0. then from_str returns 0 and we loose '-' sign.
            // Therefore check for '-' in input directly
            if integer_part.is_negative() || v[0].starts_with('-') {
                subunits = subunits
                    .checked_sub(fractional_subunits)
                    .ok_or(ParseDecimalError::Overflow)?;
            } else {
                subunits = subunits
                    .checked_add(fractional_subunits)
                    .ok_or(ParseDecimalError::Overflow)?;
            }
        }
        Ok(Self(subunits))
    }
}

impl fmt::Display for Decimal {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        const MULTIPLIER: I192 = Decimal::ONE.0;
        let quotient = self.0 / MULTIPLIER;
        let remainder = self.0 % MULTIPLIER;

        if !remainder.is_zero() {
            // print remainder with leading zeroes
            let mut sign = "".to_string();

            // take care of sign in case quotient == zere and remainder < 0,
            // eg.
            //  self.0=-100000000000000000 -> -0.1
            if remainder < I192::ZERO && quotient == I192::ZERO {
                sign.push('-');
            }
            let rem_str = format!(fmt_remainder!(), remainder.abs());
            write!(f, "{}{}.{}", sign, quotient, &rem_str.trim_end_matches('0'))
        } else {
            write!(f, "{}", quotient)
        }
    }
}

impl fmt::Debug for Decimal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

//========
// ParseDecimalError, ParsePreciseDecimalError
//========

/// Represents an error when parsing Decimal from another type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseDecimalError {
    InvalidDigit,
    Overflow,
    EmptyIntegralPart,
    EmptyFractionalPart,
    MoreThanEighteenDecimalPlaces,
    MoreThanOneDecimalPoint,
    InvalidLength(usize),
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseDecimalError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseDecimalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl TryFrom<PreciseDecimal> for Decimal {
    type Error = ParseDecimalError;

    fn try_from(val: PreciseDecimal) -> Result<Self, Self::Error> {
        val.checked_truncate(RoundingMode::ToZero)
            .ok_or(ParseDecimalError::Overflow)
    }
}

macro_rules! try_from_integer {
    ($($t:ident),*) => {
        $(
            impl TryFrom<$t> for Decimal {
                type Error = ParseDecimalError;

                fn try_from(val: $t) -> Result<Self, Self::Error> {
                    match I192::try_from(val) {
                        Ok(val) => {
                            match val.checked_mul(Self::ONE.0) {
                                Some(mul) => Ok(Self(mul)),
                                None => Err(ParseDecimalError::Overflow),
                            }
                        },
                        Err(_) => Err(ParseDecimalError::Overflow),
                    }
                }
            }
        )*
    };
}
try_from_integer!(I192, I256, I320, I448, I512, U192, U256, U320, U448, U512);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::internal_prelude::*;
    use paste::paste;

    macro_rules! test_dec {
        // NOTE: Decimal arithmetic operation safe unwrap.
        // In general, it is assumed that reasonable literals are provided.
        // If not then something is definitely wrong and panic is fine.
        ($x:literal) => {
            $crate::math::Decimal::try_from($x).unwrap()
        };
    }

    #[test]
    fn test_format_decimal() {
        assert_eq!(Decimal(1i128.into()).to_string(), "0.000000000000000001");
        assert_eq!(
            Decimal(123456789123456789i128.into()).to_string(),
            "0.123456789123456789"
        );
        assert_eq!(Decimal(1000000000000000000i128.into()).to_string(), "1");
        assert_eq!(Decimal(123000000000000000000i128.into()).to_string(), "123");
        assert_eq!(
            Decimal(123456789123456789000000000000000000i128.into()).to_string(),
            "123456789123456789"
        );
        assert_eq!(
            Decimal::MAX.to_string(),
            "3138550867693340381917894711603833208051.177722232017256447"
        );
        assert!(Decimal::MIN.is_negative());
        assert_eq!(
            Decimal::MIN.to_string(),
            "-3138550867693340381917894711603833208051.177722232017256448"
        );
    }

    #[test]
    fn test_parse_decimal() {
        assert_eq!(
            Decimal::from_str("0.000000000000000001").unwrap(),
            Decimal(1i128.into()),
        );
        assert_eq!(
            Decimal::from_str("0.0000000000000000001"),
            Err(ParseDecimalError::MoreThanEighteenDecimalPlaces),
        );
        assert_eq!(
            Decimal::from_str("0.123456789123456789").unwrap(),
            Decimal(123456789123456789i128.into()),
        );
        assert_eq!(
            Decimal::from_str("1").unwrap(),
            Decimal(1000000000000000000i128.into()),
        );
        assert_eq!(
            Decimal::from_str("123456789123456789").unwrap(),
            Decimal(123456789123456789000000000000000000i128.into()),
        );
        assert_eq!(
            Decimal::from_str("3138550867693340381917894711603833208051.177722232017256447")
                .unwrap(),
            Decimal::MAX,
        );
        assert_eq!(
            Decimal::from_str("3138550867693340381917894711603833208051.177722232017256448"),
            Err(ParseDecimalError::Overflow),
        );
        assert_eq!(
            Decimal::from_str("3138550867693340381917894711603833208052.177722232017256447"),
            Err(ParseDecimalError::Overflow),
        );
        assert_eq!(
            Decimal::from_str("-3138550867693340381917894711603833208051.177722232017256448")
                .unwrap(),
            Decimal::MIN,
        );
        assert_eq!(
            Decimal::from_str("-3138550867693340381917894711603833208051.177722232017256449"),
            Err(ParseDecimalError::Overflow),
        );
        assert_eq!(
            Decimal::from_str(".000000000000000231"),
            Err(ParseDecimalError::EmptyIntegralPart),
        );
        assert_eq!(
            Decimal::from_str("231."),
            Err(ParseDecimalError::EmptyFractionalPart),
        );

        assert_eq!(test_dec!("0"), Decimal::ZERO);
        assert_eq!(test_dec!("1"), Decimal::ONE);
        assert_eq!(test_dec!("0.1"), Decimal::ONE_TENTH);
        assert_eq!(test_dec!("10"), Decimal::TEN);
        assert_eq!(test_dec!("100"), Decimal::ONE_HUNDRED);
        assert_eq!(test_dec!("0.01"), Decimal::ONE_HUNDREDTH);
        assert_eq!(test_dec!("0.000000000000000001"), Decimal::ONE_ATTO);

        assert_eq!("0", Decimal::ZERO.to_string());
        assert_eq!("1", Decimal::ONE.to_string());
        assert_eq!("0.1", Decimal::ONE_TENTH.to_string());
        assert_eq!("10", Decimal::TEN.to_string());
        assert_eq!("100", Decimal::ONE_HUNDRED.to_string());
        assert_eq!("0.01", Decimal::ONE_HUNDREDTH.to_string());
        assert_eq!("0.000000000000000001", Decimal::ONE_ATTO.to_string());
    }

    #[test]
    fn test_add_decimal() {
        let a = Decimal::from(5u32);
        let b = Decimal::from(7u32);
        assert_eq!(a.checked_add(b).unwrap().to_string(), "12");
    }

    #[test]
    fn test_add_overflow_decimal() {
        assert!(Decimal::MAX.checked_add(Decimal::ONE).is_none());
    }

    #[test]
    fn test_sub_decimal() {
        let a = Decimal::from(5u32);
        let b = Decimal::from(7u32);
        assert_eq!(a.checked_sub(b).unwrap().to_string(), "-2");
        assert_eq!(b.checked_sub(a).unwrap().to_string(), "2");
    }

    #[test]
    fn test_sub_overflow_decimal() {
        assert!(Decimal::MIN.checked_sub(Decimal::ONE).is_none());
    }

    #[test]
    fn test_mul_decimal() {
        let a = Decimal::from(5u32);
        let b = Decimal::from(7u32);
        assert_eq!(a.checked_mul(b).unwrap().to_string(), "35");
        let a = Decimal::from_str("1000000000").unwrap();
        let b = Decimal::from_str("1000000000").unwrap();
        assert_eq!(a.checked_mul(b).unwrap().to_string(), "1000000000000000000");
        let a = Decimal::MAX;
        let b = test_dec!(1);
        assert_eq!(a.checked_mul(b).unwrap(), Decimal::MAX);
    }

    #[test]
    fn test_mul_to_max_decimal() {
        let a = Decimal::MAX.checked_sqrt().unwrap();
        a.checked_mul(a).unwrap();
    }

    #[test]
    fn test_mul_to_minimum_overflow_decimal() {
        let a = Decimal::MAX.checked_sqrt().unwrap();
        assert!(a.checked_mul(a + Decimal(I192::ONE)).is_none());
    }

    #[test]
    fn test_mul_overflow_by_small_decimal() {
        assert!(Decimal::MAX
            .checked_mul(test_dec!("1.000000000000000001"))
            .is_none());
    }

    #[test]
    fn test_mul_overflow_by_a_lot_decimal() {
        assert!(Decimal::MAX.checked_mul(test_dec!("1.1")).is_none());
    }

    #[test]
    fn test_mul_neg_overflow_decimal() {
        assert!(Decimal::MAX
            .checked_neg()
            .unwrap()
            .checked_mul(test_dec!("-1.000000000000000001"))
            .is_none());
    }

    #[test]
    fn test_div_by_zero_decimal() {
        let a = Decimal::from(5u32);
        let b = Decimal::from(0u32);
        assert!(a.checked_div(b).is_none());
    }

    #[test]
    fn test_powi_exp_overflow_decimal() {
        let a = Decimal::from(5u32);
        let b = i64::MIN;
        assert!(a.checked_powi(b).is_none());
    }

    #[test]
    fn test_1_powi_max_decimal() {
        let a = Decimal::from(1u32);
        let b = i64::MAX;
        assert_eq!(a.checked_powi(b).unwrap().to_string(), "1");
    }

    #[test]
    fn test_1_powi_min_decimal() {
        let a = Decimal::from(1u32);
        let b = i64::MAX - 1;
        assert_eq!(a.checked_powi(b).unwrap().to_string(), "1");
    }

    #[test]
    fn test_powi_max_decimal() {
        let _max = Decimal::MAX.checked_powi(1);
        let _max_sqrt = Decimal::MAX.checked_sqrt().unwrap();
        let _max_cbrt = Decimal::MAX.checked_cbrt().unwrap();
        let _max_dec_2 = _max_sqrt.checked_powi(2).unwrap();
        let _max_dec_3 = _max_cbrt.checked_powi(3).unwrap();
    }

    #[test]
    fn test_div_decimal() {
        let a = Decimal::from(5u32);
        let b = Decimal::from(7u32);
        assert_eq!(
            a.checked_div(b).unwrap().to_string(),
            "0.714285714285714285"
        );
        assert_eq!(b.checked_div(a).unwrap().to_string(), "1.4");
        assert_eq!(
            Decimal::MAX.checked_div(test_dec!(1)).unwrap(),
            Decimal::MAX
        );
    }

    #[test]
    fn test_div_negative_decimal() {
        let a = Decimal::from(-42);
        let b = Decimal::from(2);
        assert_eq!(a.checked_div(b).unwrap().to_string(), "-21");
    }

    #[test]
    fn test_0_pow_0_decimal() {
        let a = test_dec!("0");
        assert_eq!((a.checked_powi(0).unwrap()).to_string(), "1");
    }

    #[test]
    fn test_0_powi_1_decimal() {
        let a = test_dec!("0");
        assert_eq!((a.checked_powi(1).unwrap()).to_string(), "0");
    }

    #[test]
    fn test_0_powi_10_decimal() {
        let a = test_dec!("0");
        assert_eq!((a.checked_powi(10).unwrap()).to_string(), "0");
    }

    #[test]
    fn test_1_powi_0_decimal() {
        let a = test_dec!(1);
        assert_eq!((a.checked_powi(0).unwrap()).to_string(), "1");
    }

    #[test]
    fn test_1_powi_1_decimal() {
        let a = test_dec!(1);
        assert_eq!((a.checked_powi(1).unwrap()).to_string(), "1");
    }

    #[test]
    fn test_1_powi_10_decimal() {
        let a = test_dec!(1);
        assert_eq!((a.checked_powi(10).unwrap()).to_string(), "1");
    }

    #[test]
    fn test_2_powi_0_decimal() {
        let a = test_dec!("2");
        assert_eq!(a.checked_powi(0).unwrap().to_string(), "1");
    }

    #[test]
    fn test_2_powi_3724_decimal() {
        let a = test_dec!("1.000234891009084238");
        assert_eq!(
            a.checked_powi(3724).unwrap().to_string(),
            "2.397991232254669619"
        );
    }

    #[test]
    fn test_2_powi_2_decimal() {
        let a = test_dec!("2");
        assert_eq!(a.checked_powi(2).unwrap().to_string(), "4");
    }

    #[test]
    fn test_2_powi_3_decimal() {
        let a = test_dec!("2");
        assert_eq!(a.checked_powi(3).unwrap().to_string(), "8");
    }

    #[test]
    fn test_10_powi_3_decimal() {
        let a = test_dec!("10");
        assert_eq!(a.checked_powi(3).unwrap().to_string(), "1000");
    }

    #[test]
    fn test_5_powi_2_decimal() {
        let a = test_dec!("5");
        assert_eq!(a.checked_powi(2).unwrap().to_string(), "25");
    }

    #[test]
    fn test_5_powi_minus2_decimal() {
        let a = test_dec!("5");
        assert_eq!(a.checked_powi(-2).unwrap().to_string(), "0.04");
    }

    #[test]
    fn test_10_powi_minus3_decimal() {
        let a = test_dec!("10");
        assert_eq!(a.checked_powi(-3).unwrap().to_string(), "0.001");
    }

    #[test]
    fn test_minus10_powi_minus3_decimal() {
        let a = test_dec!("-10");
        assert_eq!(a.checked_powi(-3).unwrap().to_string(), "-0.001");
    }

    #[test]
    fn test_minus10_powi_minus2_decimal() {
        let a = test_dec!("-10");
        assert_eq!(a.checked_powi(-2).unwrap().to_string(), "0.01");
    }

    #[test]
    fn test_minus05_powi_minus2_decimal() {
        let a = test_dec!("-0.5");
        assert_eq!(a.checked_powi(-2).unwrap().to_string(), "4");
    }
    #[test]
    fn test_minus05_powi_minus3_decimal() {
        let a = test_dec!("-0.5");
        assert_eq!(a.checked_powi(-3).unwrap().to_string(), "-8");
    }

    #[test]
    fn test_10_powi_15_decimal() {
        let a = test_dec!(10i128);
        assert_eq!(a.checked_powi(15).unwrap().to_string(), "1000000000000000");
    }

    #[test]
    fn test_10_powi_16_decimal() {
        let a = Decimal(10i128.into());
        assert_eq!(a.checked_powi(16).unwrap().to_string(), "0");
    }

    #[test]
    fn test_one_and_zero_decimal() {
        assert_eq!(Decimal::one().to_string(), "1");
        assert_eq!(Decimal::zero().to_string(), "0");
    }

    #[test]
    fn test_dec_string_decimal_decimal() {
        assert_eq!(
            test_dec!("1.123456789012345678").to_string(),
            "1.123456789012345678"
        );
        assert_eq!(test_dec!("-5.6").to_string(), "-5.6");
    }

    #[test]
    fn test_dec_string_decimal() {
        assert_eq!(test_dec!(1).to_string(), "1");
        assert_eq!(test_dec!("0").to_string(), "0");
    }

    #[test]
    fn test_dec_int_decimal() {
        assert_eq!(test_dec!(1).to_string(), "1");
        assert_eq!(test_dec!(5).to_string(), "5");
    }

    #[test]
    fn test_dec_bool_decimal() {
        assert_eq!((test_dec!(false)).to_string(), "0");
    }

    #[test]
    fn test_floor_decimal() {
        assert_eq!(
            Decimal::MAX.checked_floor().unwrap(),
            test_dec!("3138550867693340381917894711603833208051")
        );
        assert_eq!(test_dec!("1.2").checked_floor().unwrap(), test_dec!("1"));
        assert_eq!(test_dec!("1.0").checked_floor().unwrap(), test_dec!("1"));
        assert_eq!(test_dec!("0.9").checked_floor().unwrap(), test_dec!("0"));
        assert_eq!(test_dec!("0").checked_floor().unwrap(), test_dec!("0"));
        assert_eq!(test_dec!("-0.1").checked_floor().unwrap(), test_dec!("-1"));
        assert_eq!(test_dec!("-1").checked_floor().unwrap(), test_dec!("-1"));
        assert_eq!(test_dec!("-5.2").checked_floor().unwrap(), test_dec!("-6"));

        assert_eq!(
            test_dec!("-3138550867693340381917894711603833208050.177722232017256448") // Decimal::MIN+1
                .checked_floor()
                .unwrap(),
            test_dec!("-3138550867693340381917894711603833208051")
        );
        assert_eq!(
            test_dec!("-3138550867693340381917894711603833208050.000000000000000001")
                .checked_floor()
                .unwrap(),
            test_dec!("-3138550867693340381917894711603833208051")
        );
        assert_eq!(
            test_dec!("-3138550867693340381917894711603833208051.000000000000000000")
                .checked_floor()
                .unwrap(),
            test_dec!("-3138550867693340381917894711603833208051")
        );

        // below shall return None due to overflow
        assert!(Decimal::MIN.checked_floor().is_none());

        assert!(
            test_dec!("-3138550867693340381917894711603833208051.000000000000000001")
                .checked_floor()
                .is_none()
        );
    }

    #[test]
    fn test_abs_decimal() {
        assert_eq!(test_dec!(-2).checked_abs().unwrap(), test_dec!(2));
        assert_eq!(test_dec!(2).checked_abs().unwrap(), test_dec!(2));
        assert_eq!(test_dec!(0).checked_abs().unwrap(), test_dec!(0));
        assert_eq!(Decimal::MAX.checked_abs().unwrap(), Decimal::MAX);

        // below shall return None due to overflow
        assert!(Decimal::MIN.checked_abs().is_none());
    }

    #[test]
    fn test_ceiling_decimal() {
        assert_eq!(test_dec!("1.2").checked_ceiling().unwrap(), test_dec!("2"));
        assert_eq!(test_dec!("1.0").checked_ceiling().unwrap(), test_dec!("1"));
        assert_eq!(test_dec!("0.9").checked_ceiling().unwrap(), test_dec!("1"));
        assert_eq!(test_dec!("0").checked_ceiling().unwrap(), test_dec!("0"));
        assert_eq!(test_dec!("-0.1").checked_ceiling().unwrap(), test_dec!("0"));
        assert_eq!(test_dec!("-1").checked_ceiling().unwrap(), test_dec!("-1"));
        assert_eq!(
            test_dec!("-5.2").checked_ceiling().unwrap(),
            test_dec!("-5")
        );
        assert_eq!(
            Decimal::MIN.checked_ceiling().unwrap(),
            test_dec!("-3138550867693340381917894711603833208051")
        );
        assert_eq!(
            test_dec!("3138550867693340381917894711603833208050.177722232017256447") // Decimal::MAX-1
                .checked_ceiling()
                .unwrap(),
            test_dec!("3138550867693340381917894711603833208051")
        );
        assert_eq!(
            test_dec!("3138550867693340381917894711603833208050.000000000000000000")
                .checked_ceiling()
                .unwrap(),
            test_dec!("3138550867693340381917894711603833208050")
        );

        // below shall return None due to overflow
        assert!(Decimal::MAX.checked_ceiling().is_none());
        assert!(
            test_dec!("3138550867693340381917894711603833208051.000000000000000001")
                .checked_ceiling()
                .is_none()
        );
    }

    #[test]
    fn test_rounding_to_zero_decimal() {
        let mode = RoundingMode::ToZero;
        assert_eq!(
            test_dec!("1.2").checked_round(0, mode).unwrap(),
            test_dec!("1")
        );
        assert_eq!(
            test_dec!("1.0").checked_round(0, mode).unwrap(),
            test_dec!("1")
        );
        assert_eq!(
            test_dec!("0.9").checked_round(0, mode).unwrap(),
            test_dec!("0")
        );
        assert_eq!(
            test_dec!("0").checked_round(0, mode).unwrap(),
            test_dec!("0")
        );
        assert_eq!(
            test_dec!("-0.1").checked_round(0, mode).unwrap(),
            test_dec!("0")
        );
        assert_eq!(
            test_dec!("-1").checked_round(0, mode).unwrap(),
            test_dec!("-1")
        );
        assert_eq!(
            test_dec!("-5.2").checked_round(0, mode).unwrap(),
            test_dec!("-5")
        );
        assert_eq!(
            Decimal::MAX.checked_round(0, mode).unwrap(),
            test_dec!("3138550867693340381917894711603833208051")
        );
        assert_eq!(
            Decimal::MIN.checked_round(0, mode).unwrap(),
            test_dec!("-3138550867693340381917894711603833208051")
        );
    }

    #[test]
    fn test_rounding_away_from_zero_decimal() {
        let mode = RoundingMode::AwayFromZero;
        assert_eq!(
            test_dec!("1.2").checked_round(0, mode).unwrap(),
            test_dec!("2")
        );
        assert_eq!(
            test_dec!("1.0").checked_round(0, mode).unwrap(),
            test_dec!("1")
        );
        assert_eq!(
            test_dec!("0.9").checked_round(0, mode).unwrap(),
            test_dec!("1")
        );
        assert_eq!(
            test_dec!("0").checked_round(0, mode).unwrap(),
            test_dec!("0")
        );
        assert_eq!(
            test_dec!("-0.1").checked_round(0, mode).unwrap(),
            test_dec!("-1")
        );
        assert_eq!(
            test_dec!("-1").checked_round(0, mode).unwrap(),
            test_dec!("-1")
        );
        assert_eq!(
            test_dec!("-5.2").checked_round(0, mode).unwrap(),
            test_dec!("-6")
        );

        assert_eq!(
            test_dec!("-3138550867693340381917894711603833208050.9")
                .checked_round(0, mode)
                .unwrap(),
            test_dec!("-3138550867693340381917894711603833208051")
        );
        assert_eq!(
            test_dec!("3138550867693340381917894711603833208050.9")
                .checked_round(0, mode)
                .unwrap(),
            test_dec!("3138550867693340381917894711603833208051")
        );

        // below shall return None due to overflow
        assert!(Decimal::MIN.checked_round(0, mode).is_none());
        assert!(test_dec!("-3138550867693340381917894711603833208051.1")
            .checked_round(0, mode)
            .is_none());
        assert!(Decimal::MAX.checked_round(0, mode).is_none());
        assert!(test_dec!("3138550867693340381917894711603833208051.1")
            .checked_round(0, mode)
            .is_none());
    }

    #[test]
    fn test_rounding_midpoint_toward_zero_decimal() {
        let mode = RoundingMode::ToNearestMidpointTowardZero;
        assert_eq!(
            test_dec!("5.5").checked_round(0, mode).unwrap(),
            test_dec!("5")
        );
        assert_eq!(
            test_dec!("2.5").checked_round(0, mode).unwrap(),
            test_dec!("2")
        );
        assert_eq!(
            test_dec!("1.6").checked_round(0, mode).unwrap(),
            test_dec!("2")
        );
        assert_eq!(
            test_dec!("1.1").checked_round(0, mode).unwrap(),
            test_dec!("1")
        );
        assert_eq!(
            test_dec!("1.0").checked_round(0, mode).unwrap(),
            test_dec!("1")
        );
        assert_eq!(
            test_dec!("-1.0").checked_round(0, mode).unwrap(),
            test_dec!("-1")
        );
        assert_eq!(
            test_dec!("-1.1").checked_round(0, mode).unwrap(),
            test_dec!("-1")
        );
        assert_eq!(
            test_dec!("-1.6").checked_round(0, mode).unwrap(),
            test_dec!("-2")
        );
        assert_eq!(
            test_dec!("-2.5").checked_round(0, mode).unwrap(),
            test_dec!("-2")
        );
        assert_eq!(
            test_dec!("-5.5").checked_round(0, mode).unwrap(),
            test_dec!("-5")
        );
        assert_eq!(
            Decimal::MAX.checked_round(0, mode).unwrap(),
            test_dec!("3138550867693340381917894711603833208051")
        );
        assert_eq!(
            Decimal::MIN.checked_round(0, mode).unwrap(),
            test_dec!("-3138550867693340381917894711603833208051")
        );
    }

    #[test]
    fn test_rounding_midpoint_away_from_zero_decimal() {
        let mode = RoundingMode::ToNearestMidpointAwayFromZero;
        assert_eq!(
            test_dec!("5.5").checked_round(0, mode).unwrap(),
            test_dec!("6")
        );
        assert_eq!(
            test_dec!("2.5").checked_round(0, mode).unwrap(),
            test_dec!("3")
        );
        assert_eq!(
            test_dec!("1.6").checked_round(0, mode).unwrap(),
            test_dec!("2")
        );
        assert_eq!(
            test_dec!("1.1").checked_round(0, mode).unwrap(),
            test_dec!("1")
        );
        assert_eq!(
            test_dec!("1.0").checked_round(0, mode).unwrap(),
            test_dec!("1")
        );
        assert_eq!(
            test_dec!("-1.0").checked_round(0, mode).unwrap(),
            test_dec!("-1")
        );
        assert_eq!(
            test_dec!("-1.1").checked_round(0, mode).unwrap(),
            test_dec!("-1")
        );
        assert_eq!(
            test_dec!("-1.6").checked_round(0, mode).unwrap(),
            test_dec!("-2")
        );
        assert_eq!(
            test_dec!("-2.5").checked_round(0, mode).unwrap(),
            test_dec!("-3")
        );
        assert_eq!(
            test_dec!("-5.5").checked_round(0, mode).unwrap(),
            test_dec!("-6")
        );
        assert_eq!(
            Decimal::MAX.checked_round(0, mode).unwrap(),
            test_dec!("3138550867693340381917894711603833208051")
        );
        assert_eq!(
            Decimal::MIN.checked_round(0, mode).unwrap(),
            test_dec!("-3138550867693340381917894711603833208051")
        );
    }

    #[test]
    fn test_rounding_midpoint_nearest_even_zero_decimal() {
        let mode = RoundingMode::ToNearestMidpointToEven;
        assert_eq!(
            test_dec!("5.5").checked_round(0, mode).unwrap(),
            test_dec!("6")
        );
        assert_eq!(
            test_dec!("2.5").checked_round(0, mode).unwrap(),
            test_dec!("2")
        );
        assert_eq!(
            test_dec!("1.6").checked_round(0, mode).unwrap(),
            test_dec!("2")
        );
        assert_eq!(
            test_dec!("1.1").checked_round(0, mode).unwrap(),
            test_dec!("1")
        );
        assert_eq!(
            test_dec!("1.0").checked_round(0, mode).unwrap(),
            test_dec!("1")
        );
        assert_eq!(
            test_dec!("-1.0").checked_round(0, mode).unwrap(),
            test_dec!("-1")
        );
        assert_eq!(
            test_dec!("-1.1").checked_round(0, mode).unwrap(),
            test_dec!("-1")
        );
        assert_eq!(
            test_dec!("-1.6").checked_round(0, mode).unwrap(),
            test_dec!("-2")
        );
        assert_eq!(
            test_dec!("-2.5").checked_round(0, mode).unwrap(),
            test_dec!("-2")
        );
        assert_eq!(
            test_dec!("-5.5").checked_round(0, mode).unwrap(),
            test_dec!("-6")
        );

        assert_eq!(
            Decimal::MAX.checked_round(0, mode).unwrap(),
            test_dec!("3138550867693340381917894711603833208051")
        );
        assert_eq!(
            Decimal::MIN.checked_round(0, mode).unwrap(),
            test_dec!("-3138550867693340381917894711603833208051")
        );
    }

    #[test]
    fn test_various_decimal_places_decimal() {
        let num = test_dec!("2.4595");
        let mode = RoundingMode::AwayFromZero;
        assert_eq!(num.checked_round(0, mode).unwrap(), test_dec!("3"));
        assert_eq!(num.checked_round(1, mode).unwrap(), test_dec!("2.5"));
        assert_eq!(num.checked_round(2, mode).unwrap(), test_dec!("2.46"));
        assert_eq!(num.checked_round(3, mode).unwrap(), test_dec!("2.46"));

        assert_eq!(
            test_dec!("3138550867693340381917894711603833208050.177722232017256447")
                .checked_round(1, mode)
                .unwrap(),
            test_dec!("3138550867693340381917894711603833208050.2")
        );
        assert_eq!(
            test_dec!("-3138550867693340381917894711603833208050.177722232017256448")
                .checked_round(1, mode)
                .unwrap(),
            test_dec!("-3138550867693340381917894711603833208050.2")
        );

        let mode = RoundingMode::ToZero;
        assert_eq!(num.checked_round(0, mode).unwrap(), test_dec!("2"));
        assert_eq!(num.checked_round(1, mode).unwrap(), test_dec!("2.4"));
        assert_eq!(num.checked_round(2, mode).unwrap(), test_dec!("2.45"));
        assert_eq!(num.checked_round(3, mode).unwrap(), test_dec!("2.459"));
        let mode = RoundingMode::ToPositiveInfinity;
        assert_eq!(num.checked_round(0, mode).unwrap(), test_dec!("3"));
        assert_eq!(num.checked_round(1, mode).unwrap(), test_dec!("2.5"));
        assert_eq!(num.checked_round(2, mode).unwrap(), test_dec!("2.46"));
        assert_eq!(num.checked_round(3, mode).unwrap(), test_dec!("2.46"));
        let mode = RoundingMode::ToNegativeInfinity;
        assert_eq!(num.checked_round(0, mode).unwrap(), test_dec!("2"));
        assert_eq!(num.checked_round(1, mode).unwrap(), test_dec!("2.4"));
        assert_eq!(num.checked_round(2, mode).unwrap(), test_dec!("2.45"));
        assert_eq!(num.checked_round(3, mode).unwrap(), test_dec!("2.459"));
        let mode = RoundingMode::ToNearestMidpointAwayFromZero;
        assert_eq!(num.checked_round(0, mode).unwrap(), test_dec!("2"));
        assert_eq!(num.checked_round(1, mode).unwrap(), test_dec!("2.5"));
        assert_eq!(num.checked_round(2, mode).unwrap(), test_dec!("2.46"));
        assert_eq!(num.checked_round(3, mode).unwrap(), test_dec!("2.46"));
        let mode = RoundingMode::ToNearestMidpointTowardZero;
        assert_eq!(num.checked_round(0, mode).unwrap(), test_dec!("2"));
        assert_eq!(num.checked_round(1, mode).unwrap(), test_dec!("2.5"));
        assert_eq!(num.checked_round(2, mode).unwrap(), test_dec!("2.46"));
        assert_eq!(num.checked_round(3, mode).unwrap(), test_dec!("2.459"));
        let mode = RoundingMode::ToNearestMidpointToEven;
        assert_eq!(num.checked_round(0, mode).unwrap(), test_dec!("2"));
        assert_eq!(num.checked_round(1, mode).unwrap(), test_dec!("2.5"));
        assert_eq!(num.checked_round(2, mode).unwrap(), test_dec!("2.46"));
        assert_eq!(num.checked_round(3, mode).unwrap(), test_dec!("2.46"));

        let num = test_dec!("-2.4595");
        let mode = RoundingMode::AwayFromZero;
        assert_eq!(num.checked_round(0, mode).unwrap(), test_dec!("-3"));
        assert_eq!(num.checked_round(1, mode).unwrap(), test_dec!("-2.5"));
        assert_eq!(num.checked_round(2, mode).unwrap(), test_dec!("-2.46"));
        assert_eq!(num.checked_round(3, mode).unwrap(), test_dec!("-2.46"));
        let mode = RoundingMode::ToZero;
        assert_eq!(num.checked_round(0, mode).unwrap(), test_dec!("-2"));
        assert_eq!(num.checked_round(1, mode).unwrap(), test_dec!("-2.4"));
        assert_eq!(num.checked_round(2, mode).unwrap(), test_dec!("-2.45"));
        assert_eq!(num.checked_round(3, mode).unwrap(), test_dec!("-2.459"));
        let mode = RoundingMode::ToPositiveInfinity;
        assert_eq!(num.checked_round(0, mode).unwrap(), test_dec!("-2"));
        assert_eq!(num.checked_round(1, mode).unwrap(), test_dec!("-2.4"));
        assert_eq!(num.checked_round(2, mode).unwrap(), test_dec!("-2.45"));
        assert_eq!(num.checked_round(3, mode).unwrap(), test_dec!("-2.459"));
        let mode = RoundingMode::ToNegativeInfinity;
        assert_eq!(num.checked_round(0, mode).unwrap(), test_dec!("-3"));
        assert_eq!(num.checked_round(1, mode).unwrap(), test_dec!("-2.5"));
        assert_eq!(num.checked_round(2, mode).unwrap(), test_dec!("-2.46"));
        assert_eq!(num.checked_round(3, mode).unwrap(), test_dec!("-2.46"));
        let mode = RoundingMode::ToNearestMidpointAwayFromZero;
        assert_eq!(num.checked_round(0, mode).unwrap(), test_dec!("-2"));
        assert_eq!(num.checked_round(1, mode).unwrap(), test_dec!("-2.5"));
        assert_eq!(num.checked_round(2, mode).unwrap(), test_dec!("-2.46"));
        assert_eq!(num.checked_round(3, mode).unwrap(), test_dec!("-2.46"));
        let mode = RoundingMode::ToNearestMidpointTowardZero;
        assert_eq!(num.checked_round(0, mode).unwrap(), test_dec!("-2"));
        assert_eq!(num.checked_round(1, mode).unwrap(), test_dec!("-2.5"));
        assert_eq!(num.checked_round(2, mode).unwrap(), test_dec!("-2.46"));
        assert_eq!(num.checked_round(3, mode).unwrap(), test_dec!("-2.459"));
        let mode = RoundingMode::ToNearestMidpointToEven;
        assert_eq!(num.checked_round(0, mode).unwrap(), test_dec!("-2"));
        assert_eq!(num.checked_round(1, mode).unwrap(), test_dec!("-2.5"));
        assert_eq!(num.checked_round(2, mode).unwrap(), test_dec!("-2.46"));
        assert_eq!(num.checked_round(3, mode).unwrap(), test_dec!("-2.46"));
    }

    #[test]
    fn test_encode_decimal_value_decimal() {
        let dec = test_dec!("0");
        let bytes = scrypto_encode(&dec).unwrap();
        assert_eq!(bytes, {
            let mut a = [0; 26];
            a[0] = SCRYPTO_SBOR_V1_PAYLOAD_PREFIX;
            a[1] = ScryptoValueKind::Custom(ScryptoCustomValueKind::Decimal).as_u8();
            a
        });
    }

    #[test]
    fn test_decode_decimal_value_decimal() {
        let dec = test_dec!("1.23456789");
        let bytes = scrypto_encode(&dec).unwrap();
        let decoded: Decimal = scrypto_decode(&bytes).unwrap();
        assert_eq!(decoded, test_dec!("1.23456789"));
    }

    #[test]
    fn test_from_str_decimal() {
        let dec = Decimal::from_str("5.0").unwrap();
        assert_eq!(dec.to_string(), "5");
    }

    #[test]
    fn test_from_str_failure_decimal() {
        let dec = Decimal::from_str("non_decimal_value");
        assert_eq!(dec, Err(ParseDecimalError::InvalidDigit));
    }

    macro_rules! test_from_into_precise_decimal_decimal {
        ($(($from:expr, $expected:expr, $suffix:expr)),*) => {
            paste!{
            $(
                #[test]
                fn [<test_from_into_precise_decimal_decimal_ $suffix>]() {
                    let pdec = PreciseDecimal::try_from($from).unwrap();
                    let dec = Decimal::try_from(pdec).unwrap();
                    assert_eq!(dec.to_string(), $expected);

                    let dec: Decimal = pdec.try_into().unwrap();
                    assert_eq!(dec.to_string(), $expected);
                }
            )*
            }
        };
    }

    test_from_into_precise_decimal_decimal! {
        ("12345678.123456789012345678901234567890123456", "12345678.123456789012345678", 1),
        ("12345678.123456789012345678101234567890123456", "12345678.123456789012345678", 2),
        ("-12345678.123456789012345678901234567890123456", "-12345678.123456789012345678", 3),
        ("-12345678.123456789012345678101234567890123456", "-12345678.123456789012345678", 4),
        ("0.000000000000000000000000008901234567", "0", 5),
        ("-0.000000000000000000000000008901234567", "0", 6),
        ("5", "5", 7),
        ("12345678.1", "12345678.1", 8)
    }

    macro_rules! test_from_precise_decimal_decimal_overflow {
        ($(($from:expr, $suffix:expr)),*) => {
            paste!{
            $(
                #[test]
                fn [<test_from_precise_decimal_decimal_overflow_ $suffix>]() {
                    let err = Decimal::try_from($from).unwrap_err();
                    assert_eq!(err, ParseDecimalError::Overflow);
                }
            )*
            }
        };
    }

    test_from_precise_decimal_decimal_overflow! {
        (PreciseDecimal::MAX, 1),
        (PreciseDecimal::MIN, 2),
        (PreciseDecimal::from(Decimal::MAX).checked_add(Decimal::ONE).unwrap(), 3),
        (PreciseDecimal::from(Decimal::MIN).checked_sub(Decimal::ONE).unwrap(), 4)
    }

    macro_rules! test_try_from_integer_overflow {
        ($(($from:expr, $suffix:expr)),*) => {
            paste!{
            $(
                #[test]
                fn [<test_try_from_integer_overflow_ $suffix>]() {
                    let err = Decimal::try_from($from).unwrap_err();
                    assert_eq!(err, ParseDecimalError::Overflow)
                }
            )*
            }
        };
    }

    test_try_from_integer_overflow! {
        (I192::MAX, 1),
        (I192::MIN, 2),
        // maximal Decimal integer part + 1
        (I192::MAX/(I192::from(10).pow(Decimal::SCALE)) + I192::ONE, 3),
        // minimal Decimal integer part - 1
        (I192::MIN/(I192::from(10).pow(Decimal::SCALE)) - I192::ONE, 4),
        (I256::MAX, 5),
        (I256::MIN, 6),
        (I320::MAX, 7),
        (I320::MIN, 8),
        (I448::MAX, 9),
        (I448::MIN, 10),
        (I512::MAX, 11),
        (I512::MIN, 12),
        (U192::MAX, 13),
        (U256::MAX, 14),
        (U320::MAX, 15),
        (U448::MAX, 16),
        (U512::MAX, 17)
    }

    macro_rules! test_try_from_integer {
        ($(($from:expr, $expected:expr, $suffix:expr)),*) => {
            paste!{
            $(
                #[test]
                fn [<test_try_from_integer_ $suffix>]() {
                    let dec = Decimal::try_from($from).unwrap();
                    assert_eq!(dec.to_string(), $expected)
                }
            )*
            }
        };
    }

    test_try_from_integer! {
        (I192::ONE, "1", 1),
        (-I192::ONE, "-1", 2),
        (I256::ONE, "1", 3),
        (-I256::ONE, "-1", 4),
        (I320::ONE, "1", 5),
        (-I320::ONE, "-1", 6),
        (I448::ONE, "1", 7),
        (-I448::ONE, "-1", 8),
        (I512::ONE, "1", 9),
        (-I512::ONE, "-1", 10),
        // maximal Decimal integer part
        (I192::MAX/(I192::from(10_u64.pow(Decimal::SCALE))), "3138550867693340381917894711603833208051", 11),
        // minimal Decimal integer part
        (I192::MIN/(I192::from(10_u64.pow(Decimal::SCALE))), "-3138550867693340381917894711603833208051", 12),
        (U192::MIN, "0", 13),
        (U256::MIN, "0", 14),
        (U320::MIN, "0", 15),
        (U448::MIN, "0", 16),
        (U512::MIN, "0", 17)
    }

    #[test]
    fn test_sqrt() {
        let sqrt_of_42 = test_dec!(42).checked_sqrt();
        let sqrt_of_0 = test_dec!(0).checked_sqrt();
        let sqrt_of_negative = test_dec!("-1").checked_sqrt();
        let sqrt_max = Decimal::MAX.checked_sqrt();
        assert_eq!(sqrt_of_42.unwrap(), test_dec!("6.48074069840786023"));
        assert_eq!(sqrt_of_0.unwrap(), test_dec!(0));
        assert_eq!(sqrt_of_negative, None);
        assert_eq!(
            sqrt_max.unwrap(),
            test_dec!("56022770974786139918.731938227458171762")
        );
    }

    #[test]
    fn test_cbrt() {
        let cbrt_of_42 = test_dec!(42).checked_cbrt().unwrap();
        let cbrt_of_0 = test_dec!(0).checked_cbrt().unwrap();
        let cbrt_of_negative_42 = test_dec!("-42").checked_cbrt().unwrap();
        let cbrt_max = Decimal::MAX.checked_cbrt().unwrap();
        assert_eq!(cbrt_of_42, test_dec!("3.476026644886449786"));
        assert_eq!(cbrt_of_0, test_dec!("0"));
        assert_eq!(cbrt_of_negative_42, test_dec!("-3.476026644886449786"));
        assert_eq!(cbrt_max, test_dec!("14641190473997.345813510937532903"));
    }

    #[test]
    fn test_nth_root() {
        let root_4_42 = test_dec!(42).checked_nth_root(4);
        let root_5_42 = test_dec!(42).checked_nth_root(5);
        let root_42_42 = test_dec!(42).checked_nth_root(42);
        let root_neg_4_42 = test_dec!("-42").checked_nth_root(4);
        let root_neg_5_42 = test_dec!("-42").checked_nth_root(5);
        let root_0 = test_dec!(42).checked_nth_root(0);
        assert_eq!(root_4_42.unwrap(), test_dec!("2.545729895021830518"));
        assert_eq!(root_5_42.unwrap(), test_dec!("2.111785764966753912"));
        assert_eq!(root_42_42.unwrap(), test_dec!("1.093072057934823618"));
        assert_eq!(root_neg_4_42, None);
        assert_eq!(root_neg_5_42.unwrap(), test_dec!("-2.111785764966753912"));
        assert_eq!(root_0, None);
    }

    #[test]
    fn no_panic_with_18_decimal_places() {
        // Arrange
        let string = "1.111111111111111111";

        // Act
        let decimal = Decimal::from_str(string);

        // Assert
        assert!(decimal.is_ok())
    }

    #[test]
    fn no_panic_with_19_decimal_places() {
        // Arrange
        let string = "1.1111111111111111111";

        // Act
        let decimal = Decimal::from_str(string);

        // Assert
        assert_matches!(
            decimal,
            Err(ParseDecimalError::MoreThanEighteenDecimalPlaces)
        );
    }

    #[test]
    fn test_neg_decimal() {
        let d = Decimal::ONE;
        assert_eq!(-d, test_dec!("-1"));
        let d = Decimal::MAX;
        assert_eq!(-d, Decimal(I192::MIN + I192::ONE));
    }

    #[test]
    #[should_panic(expected = "Overflow")]
    fn test_neg_decimal_panic() {
        let d = Decimal::MIN;
        let _ = -d;
    }

    // These tests make sure that any basic arithmetic operation
    // between primitive type and Decimal produces a Decimal.
    // Example:
    //   Decimal(10) * 10_u32 -> Decimal(100)
    macro_rules! test_arith_decimal_primitive {
        ($type:ident) => {
            paste! {
                #[test]
                fn [<test_arith_decimal_$type>]() {
                    let d1 = test_dec!("2");
                    let u1 = $type::try_from(4).unwrap();
                    assert_eq!(d1.checked_add(u1).unwrap(), test_dec!("6"));
                    assert_eq!(d1.checked_sub(u1).unwrap(), test_dec!("-2"));
                    assert_eq!(d1.checked_mul(u1).unwrap(), test_dec!("8"));
                    assert_eq!(d1.checked_div(u1).unwrap(), test_dec!("0.5"));

                    let d1 = test_dec!("2");
                    let u1 = $type::MAX;
                    let d2 = Decimal::from($type::MAX);
                    assert_eq!(d1.checked_add(u1).unwrap(), d1.checked_add(d2).unwrap());
                    assert_eq!(d1.checked_sub(u1).unwrap(), d1.checked_sub(d2).unwrap());
                    assert_eq!(d1.checked_mul(u1).unwrap(), d1.checked_mul(d2).unwrap());
                    assert_eq!(d1.checked_div(u1).unwrap(), d1.checked_div(d2).unwrap());

                    let d1 = Decimal::from($type::MIN);
                    let u1 = 2 as $type;
                    let d2 = test_dec!("2");
                    assert_eq!(d1.checked_add(u1).unwrap(), d1.checked_add(d2).unwrap());
                    assert_eq!(d1.checked_sub(u1).unwrap(), d1.checked_sub(d2).unwrap());
                    assert_eq!(d1.checked_mul(u1).unwrap(), d1.checked_mul(d2).unwrap());
                    assert_eq!(d1.checked_div(u1).unwrap(), d1.checked_div(d2).unwrap());
                }
            }
        };
    }
    test_arith_decimal_primitive!(u8);
    test_arith_decimal_primitive!(u16);
    test_arith_decimal_primitive!(u32);
    test_arith_decimal_primitive!(u64);
    test_arith_decimal_primitive!(u128);
    test_arith_decimal_primitive!(usize);
    test_arith_decimal_primitive!(i8);
    test_arith_decimal_primitive!(i16);
    test_arith_decimal_primitive!(i32);
    test_arith_decimal_primitive!(i64);
    test_arith_decimal_primitive!(i128);
    test_arith_decimal_primitive!(isize);

    macro_rules! test_arith_decimal_integer {
        ($type:ident) => {
            paste! {
                #[test]
                fn [<test_arith_decimal_$type:lower>]() {
                    let d1 = test_dec!("2");
                    let u1 = $type::try_from(4).unwrap();
                    let u2 = $type::try_from(2).unwrap();
                    let d2 = test_dec!("4");
                    assert_eq!(d1.checked_add(u1).unwrap(), u2.checked_add(d2).unwrap());
                    assert_eq!(d1.checked_sub(u1).unwrap(), u2.checked_sub(d2).unwrap());
                    assert_eq!(d1.checked_mul(u1).unwrap(), u2.checked_mul(d2).unwrap());
                    assert_eq!(d1.checked_div(u1).unwrap(), u2.checked_div(d2).unwrap());

                    let d1 = test_dec!("2");
                    let u1 = $type::MAX;
                    assert!(d1.checked_add(u1).is_none());
                    assert!(d1.checked_sub(u1).is_none());
                    assert!(d1.checked_mul(u1).is_none());
                    assert!(d1.checked_div(u1).is_none());

                    let d1 = Decimal::MAX;
                    let u1 = $type::try_from(2).unwrap();
                    assert_eq!(d1.checked_add(u1), None);
                    assert_eq!(d1.checked_sub(u1).unwrap(), Decimal::MAX - test_dec!("2"));
                    assert_eq!(d1.checked_mul(u1), None);
                    assert_eq!(d1.checked_div(u1).unwrap(), Decimal::MAX / test_dec!("2"));

                }
            }
        };
    }
    test_arith_decimal_integer!(I192);
    test_arith_decimal_integer!(I256);
    test_arith_decimal_integer!(I512);
    test_arith_decimal_integer!(U192);
    test_arith_decimal_integer!(U256);
    test_arith_decimal_integer!(U512);

    macro_rules! test_math_operands_decimal {
        ($type:ident) => {
            paste! {
                #[test]
                fn [<test_math_operands_decimal_$type:lower>]() {
                    let d1 = test_dec!("2");
                    let u1 = $type::try_from(4).unwrap();
                    assert_eq!(d1 + u1, test_dec!("6"));
                    assert_eq!(d1 - u1, test_dec!("-2"));
                    assert_eq!(d1 * u1, test_dec!("8"));
                    assert_eq!(d1 / u1, test_dec!("0.5"));

                    let u1 = $type::try_from(2).unwrap();
                    let d1 = test_dec!("4");
                    assert_eq!(u1 + d1, test_dec!("6"));
                    assert_eq!(u1 - d1, test_dec!("-2"));
                    assert_eq!(u1 * d1, test_dec!("8"));
                    assert_eq!(u1 / d1, test_dec!("0.5"));

                    let u1 = $type::try_from(4).unwrap();

                    let mut d1 = test_dec!("2");
                    d1 += u1;
                    assert_eq!(d1, test_dec!("6"));

                    let mut d1 = test_dec!("2");
                    d1 -= u1;
                    assert_eq!(d1, test_dec!("-2"));

                    let mut d1 = test_dec!("2");
                    d1 *= u1;
                    assert_eq!(d1, test_dec!("8"));

                    let mut d1 = test_dec!("2");
                    d1 /= u1;
                    assert_eq!(d1, test_dec!("0.5"));
                }

                #[test]
                #[should_panic(expected = "Overflow")]
                fn [<test_math_add_decimal_$type:lower _panic>]() {
                    let d1 = Decimal::MAX;
                    let u1 = $type::try_from(1).unwrap();
                    let _ = d1 + u1;
                }

                #[test]
                #[should_panic(expected = "Overflow")]
                fn [<test_math_add_$type:lower _xdecimal_panic>]() {
                    let d1 = Decimal::MAX;
                    let u1 = $type::try_from(1).unwrap();
                    let _ = u1 + d1;
                }

                #[test]
                #[should_panic(expected = "Overflow")]
                fn [<test_math_sub_decimal_$type:lower _panic>]() {
                    let d1 = Decimal::MIN;
                    let u1 = $type::try_from(1).unwrap();
                    let _ = d1 - u1;
                }

                #[test]
                #[should_panic(expected = "Overflow")]
                fn [<test_math_sub_$type:lower _xdecimal_panic>]() {
                    let d1 = Decimal::MIN;
                    let u1 = $type::try_from(1).unwrap();
                    let _ = u1 - d1;
                }

                #[test]
                #[should_panic(expected = "Overflow")]
                fn [<test_math_mul_decimal_$type:lower _panic>]() {
                    let d1 = Decimal::MAX;
                    let u1 = $type::try_from(2).unwrap();
                    let _ = d1 * u1;
                }

                #[test]
                #[should_panic(expected = "Overflow")]
                fn [<test_math_mul_$type:lower _xdecimal_panic>]() {
                    let d1 = Decimal::MAX;
                    let u1 = $type::try_from(2).unwrap();
                    let _ = u1 * d1;
                }

                #[test]
                #[should_panic(expected = "Overflow")]
                fn [<test_math_div_zero_decimal_$type:lower _panic>]() {
                    let d1 = Decimal::MAX;
                    let u1 = $type::try_from(0).unwrap();
                    let _ = d1 / u1;
                }

                #[test]
                #[should_panic(expected = "Overflow or division by zero")]
                fn [<test_math_div_zero_$type:lower _xdecimal_panic>]() {
                    let d1 = Decimal::ZERO;
                    let u1 = $type::try_from(1).unwrap();
                    let _ = u1 / d1;
                }

                #[test]
                #[should_panic(expected = "Overflow")]
                fn [<test_math_add_assign_decimal_$type:lower _panic>]() {
                    let mut d1 = Decimal::MAX;
                    let u1 = $type::try_from(1).unwrap();
                    d1 += u1;
                }

                #[test]
                #[should_panic(expected = "Overflow")]
                fn [<test_math_sub_assign_decimal_$type:lower _panic>]() {
                    let mut d1 = Decimal::MIN;
                    let u1 = $type::try_from(1).unwrap();
                    d1 -= u1;
                }

                #[test]
                #[should_panic(expected = "Overflow")]
                fn [<test_math_mul_assign_decimal_$type:lower _panic>]() {
                    let mut d1 = Decimal::MAX;
                    let u1 = $type::try_from(2).unwrap();
                    d1 *= u1;
                }

                #[test]
                #[should_panic(expected = "Overflow or division by zero")]
                fn [<test_math_div_assign_decimal_$type:lower _panic>]() {
                    let mut d1 = Decimal::MAX;
                    let u1 = $type::try_from(0).unwrap();
                    d1 /= u1;
                }
            }
        };
    }
    test_math_operands_decimal!(Decimal);
    test_math_operands_decimal!(u8);
    test_math_operands_decimal!(u16);
    test_math_operands_decimal!(u32);
    test_math_operands_decimal!(u64);
    test_math_operands_decimal!(u128);
    test_math_operands_decimal!(usize);
    test_math_operands_decimal!(i8);
    test_math_operands_decimal!(i16);
    test_math_operands_decimal!(i32);
    test_math_operands_decimal!(i64);
    test_math_operands_decimal!(i128);
    test_math_operands_decimal!(isize);
    test_math_operands_decimal!(I192);
    test_math_operands_decimal!(I256);
    test_math_operands_decimal!(I320);
    test_math_operands_decimal!(I448);
    test_math_operands_decimal!(I512);
    test_math_operands_decimal!(U192);
    test_math_operands_decimal!(U256);
    test_math_operands_decimal!(U320);
    test_math_operands_decimal!(U448);
    test_math_operands_decimal!(U512);

    macro_rules! test_from_primitive_type {
        ($type:ident) => {
            paste! {
                #[test]
                fn [<test_decimal_from_primitive_$type>]() {
                    let v = $type::try_from(1).unwrap();
                    assert_eq!(Decimal::from(v), test_dec!(1));

                    if $type::MIN != 0 {
                        let v = $type::try_from(-1).unwrap();
                        assert_eq!(Decimal::from(v), test_dec!(-1));
                    }

                    let v = $type::MAX;
                    assert_eq!(Decimal::from(v), Decimal::from_str(&v.to_string()).unwrap());

                    let v = $type::MIN;
                    assert_eq!(Decimal::from(v), Decimal::from_str(&v.to_string()).unwrap());
                }
            }
        };
    }
    test_from_primitive_type!(u8);
    test_from_primitive_type!(u16);
    test_from_primitive_type!(u32);
    test_from_primitive_type!(u64);
    test_from_primitive_type!(u128);
    test_from_primitive_type!(usize);
    test_from_primitive_type!(i8);
    test_from_primitive_type!(i16);
    test_from_primitive_type!(i32);
    test_from_primitive_type!(i64);
    test_from_primitive_type!(i128);
    test_from_primitive_type!(isize);

    macro_rules! test_to_primitive_type {
        ($type:ident) => {
            paste! {
                #[test]
                fn [<test_decimal_to_primitive_$type>]() {
                    let d = test_dec!(1);
                    let v = $type::try_from(1).unwrap();
                    assert_eq!($type::try_from(d).unwrap(), v);

                    if $type::MIN != 0 {
                        let d = test_dec!(-1);
                        let v = $type::try_from(-1).unwrap();
                        assert_eq!($type::try_from(d).unwrap(), v);
                    }

                    let v = $type::MAX;
                    let d = Decimal::from(v);
                    assert_eq!($type::try_from(d).unwrap(), v);

                    let v = $type::MIN;
                    let d = Decimal::from(v);
                    assert_eq!($type::try_from(d).unwrap(), v);

                    let d = Decimal::MAX;
                    let err = $type::try_from(d).unwrap_err();
                    assert_eq!(err, ParseDecimalError::InvalidDigit);

                    let v = $type::MAX;
                    let d = Decimal::from(v).checked_add(1).unwrap();
                    let err = $type::try_from(d).unwrap_err();
                    assert_eq!(err, ParseDecimalError::Overflow);

                    let v = $type::MIN;
                    let d = Decimal::from(v).checked_sub(1).unwrap();
                    let err = $type::try_from(d).unwrap_err();
                    assert_eq!(err, ParseDecimalError::Overflow);

                    let d = test_dec!("1.1");
                    let err = $type::try_from(d).unwrap_err();
                    assert_eq!(err, ParseDecimalError::InvalidDigit);
                }
            }
        };
    }
    test_to_primitive_type!(u8);
    test_to_primitive_type!(u16);
    test_to_primitive_type!(u32);
    test_to_primitive_type!(u64);
    test_to_primitive_type!(u128);
    test_to_primitive_type!(usize);
    test_to_primitive_type!(i8);
    test_to_primitive_type!(i16);
    test_to_primitive_type!(i32);
    test_to_primitive_type!(i64);
    test_to_primitive_type!(i128);
    test_to_primitive_type!(isize);
}
