#[cfg(feature = "radix_engine_fuzzing")]
use arbitrary::Arbitrary;
use core::cmp::Ordering;
use num_bigint::BigInt;
use num_traits::{Pow, Zero};
use sbor::rust::convert::{TryFrom, TryInto};
use sbor::rust::fmt;
use sbor::rust::format;
use sbor::rust::iter;
use sbor::rust::str::FromStr;
use sbor::rust::string::String;
use sbor::rust::string::ToString;
use sbor::rust::vec::Vec;
use sbor::*;

use crate::data::manifest::ManifestCustomValueKind;
use crate::data::scrypto::*;
use crate::math::bnum_integer::*;
use crate::math::decimal::*;
use crate::math::rounding_mode::*;
use crate::math::traits::*;
use crate::well_known_scrypto_custom_type;
use crate::*;

/// `PreciseDecimal` represents a 256 bit representation of a fixed-scale decimal number.
///
/// The finite set of values are of the form `m / 10^36`, where `m` is
/// an integer such that `-2^(256 - 1) <= m < 2^(256 - 1)`.
///
/// Fractional part: ~120 bits / 36 digits
/// Integer part   : 136 bits / 41 digits
/// Max            :  57896044618658097711785492504343953926634.992332820282019728792003956564819967
/// Min            : -57896044618658097711785492504343953926634.992332820282019728792003956564819968
///
/// Unless otherwise specified, all operations will panic if underflow/overflow.
#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PreciseDecimal(pub I256);

impl Default for PreciseDecimal {
    fn default() -> Self {
        Self::zero()
    }
}

impl iter::Sum for PreciseDecimal {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut sum = PreciseDecimal::zero();
        iter.for_each(|d| sum = sum.safe_add(d).expect("Overflow"));
        sum
    }
}

// TODO come up with some smarter formatting depending on PreciseDecimal::Scale
macro_rules! fmt_remainder {
    () => {
        "{:036}"
    };
}

impl PreciseDecimal {
    /// The min value of `PreciseDecimal`.
    pub const MIN: Self = Self(I256::MIN);

    /// The max value of `PreciseDecimal`.
    pub const MAX: Self = Self(I256::MAX);

    /// The bit length of number storing `PreciseDecimal`.
    pub const BITS: usize = I256::BITS as usize;

    /// The fixed scale used by `PreciseDecimal`.
    pub const SCALE: u32 = 36;

    pub const ZERO: Self = Self(I256::ZERO);

    pub const ONE: Self = Self(I256::from_digits([
        12919594847110692864,
        54210108624275221,
        0,
        0,
    ]));

    /// Returns `PreciseDecimal` of 0.
    pub fn zero() -> Self {
        Self::ZERO
    }

    /// Returns `PreciseDecimal` of 1.
    pub fn one() -> Self {
        Self::ONE
    }

    /// Whether this decimal is zero.
    pub fn is_zero(&self) -> bool {
        self.0 == I256::zero()
    }

    /// Whether this decimal is positive.
    pub fn is_positive(&self) -> bool {
        self.0 > I256::zero()
    }

    /// Whether this decimal is negative.
    pub fn is_negative(&self) -> bool {
        self.0 < I256::zero()
    }

    /// Returns the absolute value.
    pub fn abs(&self) -> PreciseDecimal {
        PreciseDecimal(self.0.abs())
    }

    /// Returns the largest integer that is equal to or less than this number.
    pub fn floor(&self) -> Self {
        self.round(0, RoundingMode::ToNegativeInfinity)
    }

    /// Returns the smallest integer that is equal to or greater than this number.
    pub fn ceiling(&self) -> Self {
        self.round(0, RoundingMode::ToPositiveInfinity)
    }

    /// Rounds this number to the specified decimal places.
    ///
    /// # Panics
    /// - Panic if the number of decimal places is not within [0..SCALE]
    pub fn round<T: Into<i32>>(&self, decimal_places: T, mode: RoundingMode) -> Self {
        let decimal_places = decimal_places.into();
        assert!(decimal_places <= Self::SCALE as i32);
        assert!(decimal_places >= 0);

        let n = Self::SCALE - decimal_places as u32;
        let divisor: I256 = I256::TEN.pow(n);
        match mode {
            RoundingMode::ToPositiveInfinity => {
                if self.0 % divisor == I256::ZERO {
                    *self
                } else if self.is_negative() {
                    Self(self.0 / divisor * divisor)
                } else {
                    Self((self.0 / divisor + I256::ONE) * divisor)
                }
            }
            RoundingMode::ToNegativeInfinity => {
                if self.0 % divisor == I256::ZERO {
                    *self
                } else if self.is_negative() {
                    Self((self.0 / divisor - I256::ONE) * divisor)
                } else {
                    Self(self.0 / divisor * divisor)
                }
            }
            RoundingMode::ToZero => {
                if self.0 % divisor == I256::ZERO {
                    *self
                } else {
                    Self(self.0 / divisor * divisor)
                }
            }
            RoundingMode::AwayFromZero => {
                if self.0 % divisor == I256::ZERO {
                    *self
                } else if self.is_negative() {
                    Self((self.0 / divisor - I256::ONE) * divisor)
                } else {
                    Self((self.0 / divisor + I256::ONE) * divisor)
                }
            }
            RoundingMode::ToNearestMidpointTowardZero => {
                let remainder = (self.0 % divisor).abs();
                if remainder == I256::ZERO {
                    *self
                } else {
                    let mid_point = divisor / I256::from(2);
                    if remainder > mid_point {
                        if self.is_negative() {
                            Self((self.0 / divisor - I256::ONE) * divisor)
                        } else {
                            Self((self.0 / divisor + I256::ONE) * divisor)
                        }
                    } else {
                        Self(self.0 / divisor * divisor)
                    }
                }
            }
            RoundingMode::ToNearestMidpointAwayFromZero => {
                let remainder = (self.0 % divisor).abs();
                if remainder == I256::ZERO {
                    *self
                } else {
                    let mid_point = divisor / I256::from(2);
                    if remainder >= mid_point {
                        if self.is_negative() {
                            Self((self.0 / divisor - I256::ONE) * divisor)
                        } else {
                            Self((self.0 / divisor + I256::ONE) * divisor)
                        }
                    } else {
                        Self(self.0 / divisor * divisor)
                    }
                }
            }
            RoundingMode::ToNearestMidpointToEven => {
                let remainder = (self.0 % divisor).abs();
                if remainder == I256::ZERO {
                    *self
                } else {
                    let mid_point = divisor / I256::from(2);
                    match remainder.cmp(&mid_point) {
                        Ordering::Greater => {
                            if self.is_negative() {
                                Self((self.0 / divisor - I256::ONE) * divisor)
                            } else {
                                Self((self.0 / divisor + I256::ONE) * divisor)
                            }
                        }
                        Ordering::Equal => {
                            if self.0 / divisor % I256::from(2) == I256::ZERO {
                                Self(self.0 / divisor * divisor)
                            } else if self.is_negative() {
                                Self((self.0 / divisor - I256::ONE) * divisor)
                            } else {
                                Self((self.0 / divisor + I256::ONE) * divisor)
                            }
                        }
                        Ordering::Less => Self(self.0 / divisor * divisor),
                    }
                }
            }
        }
    }

    /// Calculates power using exponentiation by squaring.
    pub fn safe_powi(&self, exp: i64) -> Option<Self> {
        let one_384 = I384::from(Self::ONE.0);
        let base_384 = I384::from(self.0);
        let div = |x: i64, y: i64| x.checked_div(y);
        let sub = |x: i64, y: i64| x.checked_sub(y);
        let mul = |x: i64, y: i64| x.checked_mul(y);

        if exp < 0 {
            let sub_384 = one_384 * one_384 / base_384;
            let sub_256 = I256::try_from(sub_384).ok()?;
            let exp = mul(exp, -1)?;
            return Self(sub_256).safe_powi(exp);
        }
        if exp == 0 {
            return Some(Self::ONE);
        }
        if exp == 1 {
            return Some(*self);
        }
        if exp % 2 == 0 {
            let sub_384 = base_384 * base_384 / one_384;
            let sub_256 = I256::try_from(sub_384).ok()?;
            let exp = div(exp, 2)?;
            Self(sub_256).safe_powi(exp)
        } else {
            let sub_384 = base_384 * base_384 / one_384;
            let sub_256 = I256::try_from(sub_384).ok()?;
            let sub_pdec = Self(sub_256);
            let exp = div(sub(exp, 1)?, 2)?;
            let b = sub_pdec.safe_powi(exp)?;
            self.safe_mul(b)
        }
    }

    /// Square root of a PreciseDecimal
    pub fn sqrt(&self) -> Option<Self> {
        if self.is_negative() {
            return None;
        }
        if self.is_zero() {
            return Some(Self::ZERO);
        }

        // The I256 i associated to a Decimal d is : i = d*10^36.
        // Therefore, taking sqrt yields sqrt(i) = sqrt(d)*10^32 => We lost precision
        // To get the right precision, we compute : sqrt(i*10^36) = sqrt(d)*10^36
        let self_384 = I384::from(self.0);
        let correct_nb = self_384 * I384::from(Self::ONE.0);
        let sqrt = I256::try_from(correct_nb.sqrt()).expect("Overflow");
        Some(Self(sqrt))
    }

    /// Cubic root of a PreciseDecimal
    pub fn cbrt(&self) -> Self {
        if self.is_zero() {
            return Self::ZERO;
        }

        // By reasoning in the same way as before, we realise that we need to multiply by 10^36
        let self_bigint = BigInt::from(self.0);
        let correct_nb: BigInt = self_bigint * BigInt::from(Self::ONE.0).pow(2_u32);
        let cbrt = I256::try_from(correct_nb.cbrt()).unwrap();
        Self(cbrt)
    }

    /// Nth root of a PreciseDecimal
    pub fn nth_root(&self, n: u32) -> Option<Self> {
        if (self.is_negative() && n % 2 == 0) || n == 0 {
            None
        } else if n == 1 {
            Some(*self)
        } else {
            if self.is_zero() {
                return Some(Self::ZERO);
            }

            // By induction, we need to multiply by the (n-1)th power of 10^36.
            // To not overflow, we use BigInt
            let self_integer = BigInt::from(self.0);
            let correct_nb = self_integer * BigInt::from(Self::ONE.0).pow(n - 1);
            let nth_root = I256::try_from(correct_nb.nth_root(n)).unwrap();
            Some(Self(nth_root))
        }
    }
}

macro_rules! from_int {
    ($type:ident) => {
        impl From<$type> for PreciseDecimal {
            fn from(val: $type) -> Self {
                Self(I256::from(val) * Self::ONE.0)
            }
        }
    };
}
from_int!(u8);
from_int!(u16);
from_int!(u32);
from_int!(u64);
from_int!(u128);
from_int!(usize);
from_int!(i8);
from_int!(i16);
from_int!(i32);
from_int!(i64);
from_int!(i128);
from_int!(isize);

// from_str() should be enough, but we want to have try_from() to simplify pdec! macro
impl TryFrom<&str> for PreciseDecimal {
    type Error = ParsePreciseDecimalError;

    fn try_from(val: &str) -> Result<Self, Self::Error> {
        Self::from_str(val)
    }
}

impl TryFrom<String> for PreciseDecimal {
    type Error = ParsePreciseDecimalError;

    fn try_from(val: String) -> Result<Self, Self::Error> {
        Self::from_str(&val)
    }
}

impl From<bool> for PreciseDecimal {
    fn from(val: bool) -> Self {
        if val {
            Self::from(1u8)
        } else {
            Self::from(0u8)
        }
    }
}

impl SafeNeg<PreciseDecimal> for PreciseDecimal {
    type Output = Self;

    #[inline]
    fn safe_neg(self) -> Option<Self::Output> {
        let c = self.0.safe_neg();
        c.map(Self)
    }
}

impl SafeAdd<PreciseDecimal> for PreciseDecimal {
    type Output = Self;

    #[inline]
    fn safe_add(self, other: Self) -> Option<Self::Output> {
        let a = self.0;
        let b = other.0;
        let c = a.safe_add(b);
        c.map(Self)
    }
}

impl SafeSub<PreciseDecimal> for PreciseDecimal {
    type Output = Self;

    #[inline]
    fn safe_sub(self, other: Self) -> Option<Self::Output> {
        let a = self.0;
        let b = other.0;
        let c = a.safe_sub(b);
        c.map(Self)
    }
}

impl SafeMul<PreciseDecimal> for PreciseDecimal {
    type Output = Self;

    #[inline]
    fn safe_mul(self, other: Self) -> Option<Self> {
        // Use I384 (BInt<6>) to not overflow.
        let a = I384::from(self.0);
        let b = I384::from(other.0);

        let c = a.safe_mul(b)?;
        let c = c.safe_div(I384::from(Self::ONE.0))?;

        let c_256 = I256::try_from(c).ok();
        c_256.map(Self)
    }
}

impl SafeDiv<PreciseDecimal> for PreciseDecimal {
    type Output = Self;

    #[inline]
    fn safe_div(self, other: Self) -> Option<Self> {
        // Use I384 (BInt<6>) to not overflow.
        let a = I384::from(self.0);
        let b = I384::from(other.0);

        let c = a.safe_mul(I384::from(Self::ONE.0))?;
        let c = c.safe_div(b)?;

        let c_256 = I256::try_from(c).ok();
        c_256.map(Self)
    }
}
/*
impl SafeAdd<Decimal> for PreciseDecimal {
    type Output = Self;

    fn safe_add(self, other: Self) -> Option<Self::Output> {
        self.safe_add(Self::from(other))
    }
}

impl SafeSub<PreciseDecimal> for Decimal {
    type Output = PreciseDecimal;

    fn safe_sub(self, other: Self) -> Option<Self::Output> {
        PreciseDecimal::from(self).safe_sub(other)
    }
}

impl SafeMul<PreciseDecimal> for Decimal {
    type Output = PreciseDecimal;

    fn safe_mul(self, other: Self) -> Option<Self> {
        PreciseDecimal::from(self).safe_mul(other)
    }
}

impl SafeDiv<PreciseDecimal> for Decimal {
    type Output = PreciseDecimal;

    fn safe_div(self, other: Self) -> Option<Self> {
        PreciseDecimal::from(self).safe_div(other)
    }
}
*/

// TODO below should be checked ops as well, but cannot overload safe_ops for primitived type
// eg. below is not working, safe_sub expects i32
// 1_i32.safe_sub(d: Decimal) -> Option<Decimal>
macro_rules! impl_arith_ops {
    ($type:ident) => {
        impl SafeAdd<$type> for PreciseDecimal {
            type Output = Self;

            fn safe_add(self, other: $type) -> Option<Self::Output> {
                self.safe_add(Self::from(other))
            }
        }

        impl SafeSub<$type> for PreciseDecimal {
            type Output = Self;

            fn safe_sub(self, other: $type) -> Option<Self::Output> {
                self.safe_sub(Self::from(other))
            }
        }

        impl SafeMul<$type> for PreciseDecimal {
            type Output = Self;

            fn safe_mul(self, other: $type) -> Option<Self::Output> {
                self.safe_mul(Self::from(other))
            }
        }

        impl SafeDiv<$type> for PreciseDecimal {
            type Output = Self;

            fn safe_div(self, other: $type) -> Option<Self::Output> {
                self.safe_div(Self::from(other))
            }
        }

        impl SafeAdd<PreciseDecimal> for $type {
            type Output = PreciseDecimal;

            #[inline]
            fn safe_add(self, other: PreciseDecimal) -> Option<Self::Output> {
                other.safe_add(self)
            }
        }

        impl SafeSub<PreciseDecimal> for $type {
            type Output = PreciseDecimal;

            fn safe_sub(self, other: PreciseDecimal) -> Option<Self::Output> {
                PreciseDecimal::from(self).safe_sub(other)
            }
        }

        impl SafeMul<PreciseDecimal> for $type {
            type Output = PreciseDecimal;

            #[inline]
            fn safe_mul(self, other: PreciseDecimal) -> Option<Self::Output> {
                other.safe_mul(self)
            }
        }

        impl SafeDiv<PreciseDecimal> for $type {
            type Output = PreciseDecimal;

            fn safe_div(self, other: PreciseDecimal) -> Option<Self::Output> {
                PreciseDecimal::from(self).safe_div(other)
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
impl_arith_ops!(Decimal);

//========
// binary
//========

impl TryFrom<&[u8]> for PreciseDecimal {
    type Error = ParsePreciseDecimalError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        if slice.len() == Self::BITS / 8 {
            match I256::try_from(slice) {
                Ok(val) => Ok(Self(val)),
                Err(_) => Err(ParsePreciseDecimalError::Overflow),
            }
        } else {
            Err(ParsePreciseDecimalError::InvalidLength(slice.len()))
        }
    }
}

impl PreciseDecimal {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_le_bytes().to_vec()
    }
}

well_known_scrypto_custom_type!(
    PreciseDecimal,
    ScryptoCustomValueKind::PreciseDecimal,
    Type::PreciseDecimal,
    PreciseDecimal::BITS / 8,
    PRECISE_DECIMAL_TYPE,
    precise_decimal_type_data
);

manifest_type!(
    PreciseDecimal,
    ManifestCustomValueKind::PreciseDecimal,
    PreciseDecimal::BITS / 8
);

//======
// text
//======

impl FromStr for PreciseDecimal {
    type Err = ParsePreciseDecimalError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tens = I256::from(10);
        let v: Vec<&str> = s.split('.').collect();

        let mut int = match I256::from_str(v[0]) {
            Ok(val) => val,
            Err(_) => return Err(ParsePreciseDecimalError::InvalidDigit),
        };

        int *= tens.pow(Self::SCALE);

        if v.len() == 2 {
            let scale = if let Some(scale) = Self::SCALE.checked_sub(v[1].len() as u32) {
                Ok(scale)
            } else {
                Err(Self::Err::UnsupportedDecimalPlace)
            }?;

            let frac = match I256::from_str(v[1]) {
                Ok(val) => val,
                Err(_) => return Err(ParsePreciseDecimalError::InvalidDigit),
            };

            // if input is -0. then from_str returns 0 and we loose '-' sign.
            // Therefore check for '-' in input directly
            if int.is_negative() || v[0].starts_with('-') {
                int -= frac * tens.pow(scale);
            } else {
                int += frac * tens.pow(scale);
            }
        }
        Ok(Self(int))
    }
}

impl fmt::Display for PreciseDecimal {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        const MULTIPLIER: I256 = PreciseDecimal::ONE.0;
        let quotient = self.0 / MULTIPLIER;
        let remainder = self.0 % MULTIPLIER;

        if !remainder.is_zero() {
            // print remainder with leading zeroes
            let mut sign = "".to_string();

            // take care of sign in case quotient == zere and remainder < 0,
            // eg.
            //  self.0=-100000000000000000 -> -0.1
            if remainder < I256::ZERO && quotient == I256::ZERO {
                sign.push('-');
            }
            let rem_str = format!(fmt_remainder!(), remainder.abs());
            write!(f, "{}{}.{}", sign, quotient, &rem_str.trim_end_matches('0'))
        } else {
            write!(f, "{}", quotient)
        }
    }
}

impl fmt::Debug for PreciseDecimal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

//========
// ParseDecimalError, ParsePreciseDecimalError
//========

/// Represents an error when parsing PreciseDecimal from another type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsePreciseDecimalError {
    InvalidDecimal(String),
    InvalidChar(char),
    InvalidDigit,
    UnsupportedDecimalPlace,
    InvalidLength(usize),
    Overflow,
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParsePreciseDecimalError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParsePreciseDecimalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<Decimal> for PreciseDecimal {
    fn from(val: Decimal) -> Self {
        Self(I256::try_from(val.0).unwrap() * I256::from(10i8).pow(Self::SCALE - Decimal::SCALE))
    }
}

pub trait Truncate<T> {
    type Output;
    fn truncate(self) -> Self::Output;
}

impl Truncate<Decimal> for PreciseDecimal {
    type Output = Decimal;

    fn truncate(self) -> Self::Output {
        Decimal(
            (self.0 / I256::from(10i8).pow(PreciseDecimal::SCALE - Decimal::SCALE))
                .try_into()
                .expect("Overflow"),
        )
    }
}

macro_rules! from_integer {
    ($($t:ident),*) => {
        $(
            impl From<$t> for PreciseDecimal {
                fn from(val: $t) -> Self {
                    Self(I256::from(val) * Self::ONE.0)
                }
            }
        )*
    };
}
macro_rules! try_from_integer {
    ($($t:ident),*) => {
        $(
            impl TryFrom<$t> for PreciseDecimal {
                type Error = ParsePreciseDecimalError;

                fn try_from(val: $t) -> Result<Self, Self::Error> {
                    match I256::try_from(val) {
                        Ok(val) => {
                            match val.safe_mul(Self::ONE.0) {
                                Some(mul) => Ok(Self(mul)),
                                None => Err(ParsePreciseDecimalError::Overflow),
                            }
                        },
                        Err(_) => Err(ParsePreciseDecimalError::Overflow),
                    }
                }
            }
        )*
    };
}

from_integer!(I192, U192);
try_from_integer!(I256, I320, I384, I448, I512);
try_from_integer!(U256, U320, U384, U448, U512);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dec;
    use crate::math::precise_decimal::RoundingMode;
    use crate::pdec;
    use paste::paste;
    use sbor::rust::vec;

    #[test]
    fn test_format_precise_decimal() {
        assert_eq!(
            PreciseDecimal(1i128.into()).to_string(),
            "0.000000000000000000000000000000000001"
        );
        assert_eq!(
            PreciseDecimal(123456789123456789i128.into()).to_string(),
            "0.000000000000000000123456789123456789"
        );
        assert_eq!(
            PreciseDecimal(I256::from(10).pow(PreciseDecimal::SCALE)).to_string(),
            "1"
        );
        assert_eq!(
            PreciseDecimal(I256::from(10).pow(PreciseDecimal::SCALE) * I256::from(123)).to_string(),
            "123"
        );
        assert_eq!(
            PreciseDecimal(
                I256::from_str("123456789000000000000000000000000000000000000").unwrap()
            )
            .to_string(),
            "123456789"
        );
        assert_eq!(
            PreciseDecimal::MAX.to_string(),
            "57896044618658097711785492504343953926634.992332820282019728792003956564819967"
        );
        assert_eq!(PreciseDecimal::MIN.is_negative(), true);
        assert_eq!(
            PreciseDecimal::MIN.to_string(),
            "-57896044618658097711785492504343953926634.992332820282019728792003956564819968"
        );
    }

    #[test]
    fn test_parse_precise_decimal() {
        assert_eq!(
            PreciseDecimal::from_str("0.000000000000000001").unwrap(),
            PreciseDecimal(I256::from(10).pow(18)),
        );
        assert_eq!(
            PreciseDecimal::from_str("0.123456789123456789").unwrap(),
            PreciseDecimal(I256::from(123456789123456789i128) * I256::from(10i8).pow(18)),
        );
        assert_eq!(
            PreciseDecimal::from_str("1").unwrap(),
            PreciseDecimal(I256::from(10).pow(PreciseDecimal::SCALE)),
        );
        assert_eq!(
            PreciseDecimal::from_str("123456789123456789").unwrap(),
            PreciseDecimal(
                I256::from(123456789123456789i128) * I256::from(10).pow(PreciseDecimal::SCALE)
            ),
        );
        assert_eq!(
            PreciseDecimal::from_str(
                "57896044618658097711785492504343953926634.992332820282019728792003956564819967"
            )
            .unwrap(),
            PreciseDecimal::MAX,
        );
        assert_eq!(
            PreciseDecimal::from_str(
                "-57896044618658097711785492504343953926634.992332820282019728792003956564819968"
            )
            .unwrap(),
            PreciseDecimal::MIN,
        );
    }

    #[test]
    fn test_add_precise_decimal() {
        let a = PreciseDecimal::from(5u32);
        let b = PreciseDecimal::from(7u32);
        assert_eq!(a.safe_add(b).unwrap().to_string(), "12");
    }

    #[test]
    #[should_panic(expected = "Overflow")]
    fn test_add_overflow_precise_decimal() {
        let _ = PreciseDecimal::MAX
            .safe_add(PreciseDecimal::ONE)
            .expect("Overflow");
    }

    #[test]
    fn test_sub_precise_decimal() {
        let a = PreciseDecimal::from(5u32);
        let b = PreciseDecimal::from(7u32);
        assert_eq!(a.safe_sub(b).unwrap().to_string(), "-2");
        assert_eq!(b.safe_sub(a).unwrap().to_string(), "2");
    }

    #[test]
    #[should_panic(expected = "Overflow")]
    fn test_sub_overflow_precise_decimal() {
        let _ = PreciseDecimal::MIN
            .safe_sub(PreciseDecimal::ONE)
            .expect("Overflow");
    }

    #[test]
    fn test_mul_precise_decimal() {
        let a = PreciseDecimal::from(5u32);
        let b = PreciseDecimal::from(7u32);
        assert_eq!(a.safe_mul(b).unwrap().to_string(), "35");
        let a = PreciseDecimal::from_str("1000000000").unwrap();
        let b = PreciseDecimal::from_str("1000000000").unwrap();
        assert_eq!(a.safe_mul(b).unwrap().to_string(), "1000000000000000000");

        let a = PreciseDecimal::MAX.safe_div(pdec!(2)).unwrap();
        let b = PreciseDecimal::from(2);
        assert_eq!(
            a.safe_mul(b).unwrap(),
            pdec!("57896044618658097711785492504343953926634.992332820282019728792003956564819966")
        );
    }

    #[test]
    #[should_panic(expected = "Overflow")]
    fn test_mul_overflow_by_small_precise_decimal() {
        let _ = PreciseDecimal::MAX
            .safe_mul(pdec!("1.000000000000000000000000000000000001"))
            .expect("Overflow");
    }

    #[test]
    #[should_panic(expected = "Overflow")]
    fn test_mul_overflow_by_a_lot_precise_decimal() {
        let _ = PreciseDecimal::MAX
            .safe_mul(pdec!("1.1"))
            .expect("Overflow");
    }

    #[test]
    #[should_panic(expected = "Overflow")]
    fn test_mul_neg_overflow_precise_decimal() {
        let p = pdec!("-1.000000000000000000000000000000000001");
        println!("p = {}", p);
        let _ = PreciseDecimal::MAX
            .safe_neg()
            .unwrap()
            .safe_mul(pdec!("-1.000000000000000000000000000000000001"))
            .expect("Overflow");
    }

    #[test]
    #[should_panic]
    fn test_div_by_zero_precise_decimal() {
        let a = PreciseDecimal::from(5u32);
        let b = PreciseDecimal::from(0u32);
        a.safe_div(b).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_powi_exp_overflow_precise_decimal() {
        let a = PreciseDecimal::from(5u32);
        let b = i64::MIN;
        assert_eq!(a.safe_powi(b).unwrap().to_string(), "0");
    }

    #[test]
    fn test_1_powi_max_precise_decimal() {
        let a = PreciseDecimal::from(1u32);
        let b = i64::MAX;
        assert_eq!(a.safe_powi(b).unwrap().to_string(), "1");
    }

    #[test]
    fn test_1_powi_min_precise_decimal() {
        let a = PreciseDecimal::from(1u32);
        let b = i64::MAX - 1;
        assert_eq!(a.safe_powi(b).unwrap().to_string(), "1");
    }

    #[test]
    fn test_powi_max_precise_decimal() {
        let _max = PreciseDecimal::MAX.safe_powi(1).unwrap();
        let _max_sqrt = PreciseDecimal::MAX.sqrt().unwrap();
        let _max_cbrt = PreciseDecimal::MAX.cbrt();
        let _max_dec_2 = _max_sqrt.safe_powi(2).unwrap();
        let _max_dec_3 = _max_cbrt.safe_powi(3).unwrap();
    }

    #[test]
    fn test_div_precise_decimal() {
        let a = PreciseDecimal::from(5u32);
        let b = PreciseDecimal::from(7u32);
        assert_eq!(
            a.safe_div(b).unwrap().to_string(),
            "0.714285714285714285714285714285714285"
        );
        assert_eq!(b.safe_div(a).unwrap().to_string(), "1.4");
        let a = PreciseDecimal::MAX;
        let b = PreciseDecimal::from(2);
        assert_eq!(
            a.safe_div(b).unwrap(),
            pdec!("28948022309329048855892746252171976963317.496166410141009864396001978282409983")
        );
    }

    #[test]
    fn test_div_negative_precise_decimal() {
        let a = PreciseDecimal::from(-42);
        let b = PreciseDecimal::from(2);
        assert_eq!(a.safe_div(b).unwrap().to_string(), "-21");
    }

    #[test]
    fn test_0_pow_0_precise_decimal() {
        let a = pdec!("0");
        assert_eq!(a.safe_powi(0).unwrap().to_string(), "1");
    }

    #[test]
    fn test_0_powi_1_precise_decimal() {
        let a = pdec!("0");
        assert_eq!(a.safe_powi(1).unwrap().to_string(), "0");
    }

    #[test]
    fn test_0_powi_10_precise_decimal() {
        let a = pdec!("0");
        assert_eq!(a.safe_powi(10).unwrap().to_string(), "0");
    }

    #[test]
    fn test_1_powi_0_precise_decimal() {
        let a = pdec!(1);
        assert_eq!(a.safe_powi(0).unwrap().to_string(), "1");
    }

    #[test]
    fn test_1_powi_1_precise_decimal() {
        let a = pdec!(1);
        assert_eq!(a.safe_powi(1).unwrap().to_string(), "1");
    }

    #[test]
    fn test_1_powi_10_precise_decimal() {
        let a = pdec!(1);
        assert_eq!(a.safe_powi(10).unwrap().to_string(), "1");
    }

    #[test]
    fn test_2_powi_0_precise_decimal() {
        let a = pdec!("2");
        assert_eq!(a.safe_powi(0).unwrap().to_string(), "1");
    }

    #[test]
    fn test_2_powi_3724_precise_decimal() {
        let a = pdec!("1.000234891009084238");
        assert_eq!(
            a.safe_powi(3724).unwrap().to_string(),
            "2.3979912322546748642222795591580985"
        );
    }

    #[test]
    fn test_2_powi_2_precise_decimal() {
        let a = pdec!("2");
        assert_eq!(a.safe_powi(2).unwrap().to_string(), "4");
    }

    #[test]
    fn test_2_powi_3_precise_decimal() {
        let a = pdec!("2");
        assert_eq!(a.safe_powi(3).unwrap().to_string(), "8");
    }

    #[test]
    fn test_10_powi_3_precise_decimal() {
        let a = pdec!("10");
        assert_eq!(a.safe_powi(3).unwrap().to_string(), "1000");
    }

    #[test]
    fn test_5_powi_2_precise_decimal() {
        let a = pdec!("5");
        assert_eq!(a.safe_powi(2).unwrap().to_string(), "25");
    }

    #[test]
    fn test_5_powi_minus2_precise_decimal() {
        let a = pdec!("5");
        assert_eq!(a.safe_powi(-2).unwrap().to_string(), "0.04");
    }

    #[test]
    fn test_10_powi_minus3_precise_decimal() {
        let a = pdec!("10");
        assert_eq!(a.safe_powi(-3).unwrap().to_string(), "0.001");
    }

    #[test]
    fn test_minus10_powi_minus3_precise_decimal() {
        let a = pdec!("-10");
        assert_eq!(a.safe_powi(-3).unwrap().to_string(), "-0.001");
    }

    #[test]
    fn test_minus10_powi_minus2_precise_decimal() {
        let a = pdec!("-10");
        assert_eq!(a.safe_powi(-2).unwrap().to_string(), "0.01");
    }

    #[test]
    fn test_minus05_powi_minus2_precise_decimal() {
        let a = pdec!("-0.5");
        assert_eq!(a.safe_powi(-2).unwrap().to_string(), "4");
    }
    #[test]
    fn test_minus05_powi_minus3_precise_decimal() {
        let a = pdec!("-0.5");
        assert_eq!(a.safe_powi(-3).unwrap().to_string(), "-8");
    }

    #[test]
    fn test_10_powi_15_precise_decimal() {
        let a = pdec!(10i128);
        assert_eq!(a.safe_powi(15).unwrap().to_string(), "1000000000000000");
    }

    #[test]
    #[should_panic]
    fn test_10_powi_16_precise_decimal() {
        let a = PreciseDecimal(10i128.into());
        assert_eq!(
            a.safe_powi(16).unwrap().to_string(),
            "1000000000000000000000"
        );
    }

    #[test]
    fn test_one_and_zero_precise_decimal() {
        assert_eq!(PreciseDecimal::one().to_string(), "1");
        assert_eq!(PreciseDecimal::zero().to_string(), "0");
    }

    #[test]
    fn test_dec_string_decimal_precise_decimal() {
        assert_eq!(
            pdec!("1.123456789012345678").to_string(),
            "1.123456789012345678"
        );
        assert_eq!(pdec!("-5.6").to_string(), "-5.6");
    }

    #[test]
    fn test_dec_string_precise_decimal() {
        assert_eq!(pdec!(1).to_string(), "1");
        assert_eq!(pdec!("0").to_string(), "0");
    }

    #[test]
    fn test_dec_int_precise_decimal() {
        assert_eq!(pdec!(1).to_string(), "1");
        assert_eq!(pdec!(5).to_string(), "5");
    }

    #[test]
    fn test_dec_bool_precise_decimal() {
        assert_eq!((pdec!(false)).to_string(), "0");
    }

    #[test]
    fn test_dec_rational_precise_decimal() {
        assert_eq!((pdec!(11235, 0)).to_string(), "11235");
        assert_eq!((pdec!(11235, -2)).to_string(), "112.35");
        assert_eq!((pdec!(11235, 2)).to_string(), "1123500");

        //        assert_eq!(
        //            pdec!("1120000000000000000000000000000000000000000000000000000000000000001", -64).to_string(),
        //            "112.0000000000000000000000000000000000000000000000000000000000000001"
        //        );
    }

    #[test]
    #[should_panic(expected = "Shift overflow")]
    fn test_shift_overflow_precise_decimal() {
        // u32::MAX + 1
        pdec!(1, 4_294_967_296i128); // use explicit type to defer error to runtime
    }

    #[test]
    fn test_floor_precise_decimal() {
        assert_eq!(
            PreciseDecimal::MAX.floor().to_string(),
            "57896044618658097711785492504343953926634"
        );
        assert_eq!(pdec!("1.2").floor().to_string(), "1");
        assert_eq!(pdec!("1.0").floor().to_string(), "1");
        assert_eq!(pdec!("0.9").floor().to_string(), "0");
        assert_eq!(pdec!("0").floor().to_string(), "0");
        assert_eq!(pdec!("-0.1").floor().to_string(), "-1");
        assert_eq!(pdec!("-1").floor().to_string(), "-1");
        assert_eq!(pdec!("-5.2").floor().to_string(), "-6");
    }

    #[test]
    #[should_panic]
    fn test_floor_overflow_precise_decimal() {
        PreciseDecimal::MIN.floor();
    }

    #[test]
    fn test_ceiling_precise_decimal() {
        assert_eq!(pdec!("1.2").ceiling().to_string(), "2");
        assert_eq!(pdec!("1.0").ceiling().to_string(), "1");
        assert_eq!(pdec!("0.9").ceiling().to_string(), "1");
        assert_eq!(pdec!("0").ceiling().to_string(), "0");
        assert_eq!(pdec!("-0.1").ceiling().to_string(), "0");
        assert_eq!(pdec!("-1").ceiling().to_string(), "-1");
        assert_eq!(pdec!("-5.2").ceiling().to_string(), "-5");
        assert_eq!(
            PreciseDecimal::MIN.ceiling().to_string(),
            "-57896044618658097711785492504343953926634"
        );
    }

    #[test]
    #[should_panic]
    fn test_ceiling_overflow_precise_decimal() {
        PreciseDecimal::MAX.ceiling();
    }

    #[test]
    fn test_rounding_to_zero_precise_decimal() {
        let mode = RoundingMode::ToZero;
        assert_eq!(pdec!("1.2").round(0, mode).to_string(), "1");
        assert_eq!(pdec!("1.0").round(0, mode).to_string(), "1");
        assert_eq!(pdec!("0.9").round(0, mode).to_string(), "0");
        assert_eq!(pdec!("0").round(0, mode).to_string(), "0");
        assert_eq!(pdec!("-0.1").round(0, mode).to_string(), "0");
        assert_eq!(pdec!("-1").round(0, mode).to_string(), "-1");
        assert_eq!(pdec!("-5.2").round(0, mode).to_string(), "-5");
    }

    #[test]
    fn test_rounding_away_from_zero_precise_decimal() {
        let mode = RoundingMode::AwayFromZero;
        assert_eq!(pdec!("1.2").round(0, mode).to_string(), "2");
        assert_eq!(pdec!("1.0").round(0, mode).to_string(), "1");
        assert_eq!(pdec!("0.9").round(0, mode).to_string(), "1");
        assert_eq!(pdec!("0").round(0, mode).to_string(), "0");
        assert_eq!(pdec!("-0.1").round(0, mode).to_string(), "-1");
        assert_eq!(pdec!("-1").round(0, mode).to_string(), "-1");
        assert_eq!(pdec!("-5.2").round(0, mode).to_string(), "-6");
    }

    #[test]
    fn test_rounding_midpoint_toward_zero_precise_decimal() {
        let mode = RoundingMode::ToNearestMidpointTowardZero;
        assert_eq!(pdec!("5.5").round(0, mode).to_string(), "5");
        assert_eq!(pdec!("2.5").round(0, mode).to_string(), "2");
        assert_eq!(pdec!("1.6").round(0, mode).to_string(), "2");
        assert_eq!(pdec!("1.1").round(0, mode).to_string(), "1");
        assert_eq!(pdec!("1.0").round(0, mode).to_string(), "1");
        assert_eq!(pdec!("-1.0").round(0, mode).to_string(), "-1");
        assert_eq!(pdec!("-1.1").round(0, mode).to_string(), "-1");
        assert_eq!(pdec!("-1.6").round(0, mode).to_string(), "-2");
        assert_eq!(pdec!("-2.5").round(0, mode).to_string(), "-2");
        assert_eq!(pdec!("-5.5").round(0, mode).to_string(), "-5");
    }

    #[test]
    fn test_rounding_midpoint_away_from_zero_precise_decimal() {
        let mode = RoundingMode::ToNearestMidpointAwayFromZero;
        assert_eq!(pdec!("5.5").round(0, mode).to_string(), "6");
        assert_eq!(pdec!("2.5").round(0, mode).to_string(), "3");
        assert_eq!(pdec!("1.6").round(0, mode).to_string(), "2");
        assert_eq!(pdec!("1.1").round(0, mode).to_string(), "1");
        assert_eq!(pdec!("1.0").round(0, mode).to_string(), "1");
        assert_eq!(pdec!("-1.0").round(0, mode).to_string(), "-1");
        assert_eq!(pdec!("-1.1").round(0, mode).to_string(), "-1");
        assert_eq!(pdec!("-1.6").round(0, mode).to_string(), "-2");
        assert_eq!(pdec!("-2.5").round(0, mode).to_string(), "-3");
        assert_eq!(pdec!("-5.5").round(0, mode).to_string(), "-6");
    }

    #[test]
    fn test_rounding_midpoint_nearest_even_precise_decimal() {
        let mode = RoundingMode::ToNearestMidpointToEven;
        assert_eq!(pdec!("5.5").round(0, mode).to_string(), "6");
        assert_eq!(pdec!("2.5").round(0, mode).to_string(), "2");
        assert_eq!(pdec!("1.6").round(0, mode).to_string(), "2");
        assert_eq!(pdec!("1.1").round(0, mode).to_string(), "1");
        assert_eq!(pdec!("1.0").round(0, mode).to_string(), "1");
        assert_eq!(pdec!("-1.0").round(0, mode).to_string(), "-1");
        assert_eq!(pdec!("-1.1").round(0, mode).to_string(), "-1");
        assert_eq!(pdec!("-1.6").round(0, mode).to_string(), "-2");
        assert_eq!(pdec!("-2.5").round(0, mode).to_string(), "-2");
        assert_eq!(pdec!("-5.5").round(0, mode).to_string(), "-6");
    }

    #[test]
    fn test_various_decimal_places_precise_decimal() {
        let num = pdec!("2.4595");
        let mode = RoundingMode::AwayFromZero;
        assert_eq!(num.round(0, mode).to_string(), "3");
        assert_eq!(num.round(1, mode).to_string(), "2.5");
        assert_eq!(num.round(2, mode).to_string(), "2.46");
        assert_eq!(num.round(3, mode).to_string(), "2.46");
        let mode = RoundingMode::ToZero;
        assert_eq!(num.round(0, mode).to_string(), "2");
        assert_eq!(num.round(1, mode).to_string(), "2.4");
        assert_eq!(num.round(2, mode).to_string(), "2.45");
        assert_eq!(num.round(3, mode).to_string(), "2.459");
        let mode = RoundingMode::ToPositiveInfinity;
        assert_eq!(num.round(0, mode).to_string(), "3");
        assert_eq!(num.round(1, mode).to_string(), "2.5");
        assert_eq!(num.round(2, mode).to_string(), "2.46");
        assert_eq!(num.round(3, mode).to_string(), "2.46");
        let mode = RoundingMode::ToNegativeInfinity;
        assert_eq!(num.round(0, mode).to_string(), "2");
        assert_eq!(num.round(1, mode).to_string(), "2.4");
        assert_eq!(num.round(2, mode).to_string(), "2.45");
        assert_eq!(num.round(3, mode).to_string(), "2.459");
        let mode = RoundingMode::ToNearestMidpointAwayFromZero;
        assert_eq!(num.round(0, mode).to_string(), "2");
        assert_eq!(num.round(1, mode).to_string(), "2.5");
        assert_eq!(num.round(2, mode).to_string(), "2.46");
        assert_eq!(num.round(3, mode).to_string(), "2.46");
        let mode = RoundingMode::ToNearestMidpointTowardZero;
        assert_eq!(num.round(0, mode).to_string(), "2");
        assert_eq!(num.round(1, mode).to_string(), "2.5");
        assert_eq!(num.round(2, mode).to_string(), "2.46");
        assert_eq!(num.round(3, mode).to_string(), "2.459");
        let mode = RoundingMode::ToNearestMidpointToEven;
        assert_eq!(num.round(0, mode).to_string(), "2");
        assert_eq!(num.round(1, mode).to_string(), "2.5");
        assert_eq!(num.round(2, mode).to_string(), "2.46");
        assert_eq!(num.round(3, mode).to_string(), "2.46");

        let num = pdec!("-2.4595");
        let mode = RoundingMode::AwayFromZero;
        assert_eq!(num.round(0, mode).to_string(), "-3");
        assert_eq!(num.round(1, mode).to_string(), "-2.5");
        assert_eq!(num.round(2, mode).to_string(), "-2.46");
        assert_eq!(num.round(3, mode).to_string(), "-2.46");
        let mode = RoundingMode::ToZero;
        assert_eq!(num.round(0, mode).to_string(), "-2");
        assert_eq!(num.round(1, mode).to_string(), "-2.4");
        assert_eq!(num.round(2, mode).to_string(), "-2.45");
        assert_eq!(num.round(3, mode).to_string(), "-2.459");
        let mode = RoundingMode::ToPositiveInfinity;
        assert_eq!(num.round(0, mode).to_string(), "-2");
        assert_eq!(num.round(1, mode).to_string(), "-2.4");
        assert_eq!(num.round(2, mode).to_string(), "-2.45");
        assert_eq!(num.round(3, mode).to_string(), "-2.459");
        let mode = RoundingMode::ToNegativeInfinity;
        assert_eq!(num.round(0, mode).to_string(), "-3");
        assert_eq!(num.round(1, mode).to_string(), "-2.5");
        assert_eq!(num.round(2, mode).to_string(), "-2.46");
        assert_eq!(num.round(3, mode).to_string(), "-2.46");
        let mode = RoundingMode::ToNearestMidpointAwayFromZero;
        assert_eq!(num.round(0, mode).to_string(), "-2");
        assert_eq!(num.round(1, mode).to_string(), "-2.5");
        assert_eq!(num.round(2, mode).to_string(), "-2.46");
        assert_eq!(num.round(3, mode).to_string(), "-2.46");
        let mode = RoundingMode::ToNearestMidpointTowardZero;
        assert_eq!(num.round(0, mode).to_string(), "-2");
        assert_eq!(num.round(1, mode).to_string(), "-2.5");
        assert_eq!(num.round(2, mode).to_string(), "-2.46");
        assert_eq!(num.round(3, mode).to_string(), "-2.459");
        let mode = RoundingMode::ToNearestMidpointToEven;
        assert_eq!(num.round(0, mode).to_string(), "-2");
        assert_eq!(num.round(1, mode).to_string(), "-2.5");
        assert_eq!(num.round(2, mode).to_string(), "-2.46");
        assert_eq!(num.round(3, mode).to_string(), "-2.46");
    }

    #[test]
    fn test_sum_precise_decimal() {
        let decimals = vec![pdec!(1), pdec!("2"), pdec!("3")];
        // two syntax
        let sum1: PreciseDecimal = decimals.iter().copied().sum();
        let sum2: PreciseDecimal = decimals.into_iter().sum();
        assert_eq!(sum1, pdec!("6"));
        assert_eq!(sum2, pdec!("6"));
    }

    #[test]
    fn test_encode_decimal_value_precise_decimal() {
        let pdec = pdec!("0");
        let bytes = scrypto_encode(&pdec).unwrap();
        assert_eq!(bytes, {
            let mut a = [0; 34];
            a[0] = SCRYPTO_SBOR_V1_PAYLOAD_PREFIX;
            a[1] = ScryptoValueKind::Custom(ScryptoCustomValueKind::PreciseDecimal).as_u8();
            a
        });
    }

    #[test]
    fn test_decode_decimal_value_precise_decimal() {
        let pdec = pdec!("1.23456789");
        let bytes = scrypto_encode(&pdec).unwrap();
        let decoded: PreciseDecimal = scrypto_decode(&bytes).unwrap();
        assert_eq!(decoded, pdec!("1.23456789"));
    }

    #[test]
    fn test_from_str_precise_decimal() {
        let pdec = PreciseDecimal::from_str("5.0").unwrap();
        assert_eq!(pdec.to_string(), "5");
    }

    #[test]
    fn test_from_str_failure_precise_decimal() {
        let pdec = PreciseDecimal::from_str("non_decimal_value");
        assert_eq!(pdec, Err(ParsePreciseDecimalError::InvalidDigit));
    }

    macro_rules! test_from_into_decimal_precise_decimal {
        ($(($from:expr, $expected:expr, $suffix:expr)),*) => {
            paste!{
            $(
                #[test]
                fn [<test_from_into_decimal_precise_decimal_ $suffix>]() {
                    let dec = dec!($from);
                    let pdec = PreciseDecimal::from(dec);
                    assert_eq!(pdec.to_string(), $expected);

                    let pdec: PreciseDecimal = dec.into();
                    assert_eq!(pdec.to_string(), $expected);
                }
            )*
            }
        };
    }

    test_from_into_decimal_precise_decimal! {
        ("12345678.123456789012345678", "12345678.123456789012345678", 1),
        ("0.000000000000000001", "0.000000000000000001", 2),
        ("-0.000000000000000001", "-0.000000000000000001", 3),
        ("5", "5", 4),
        ("12345678.1", "12345678.1", 5)
    }

    macro_rules! test_try_from_integer_overflow {
        ($(($from:expr, $suffix:expr)),*) => {
            paste!{
            $(
                #[test]
                fn [<test_try_from_integer_overflow_ $suffix>]() {
                    let err = PreciseDecimal::try_from($from).unwrap_err();
                    assert_eq!(err, ParsePreciseDecimalError::Overflow)
                }
            )*
            }
        };
    }

    test_try_from_integer_overflow! {
        (I256::MAX, 1),
        (I256::MIN, 2),
        // maximal PreciseDecimal integer part + 1
        (I256::MAX/(I256::from(10).pow(PreciseDecimal::SCALE)) + I256::ONE, 3),
        // minimal PreciseDecimal integer part - 1
        (I256::MIN/(I256::from(10).pow(PreciseDecimal::SCALE)) - I256::ONE, 4),
        (I256::MIN, 5),
        (I256::MAX, 6)
    }

    macro_rules! test_try_from_integer {
        ($(($from:expr, $expected:expr, $suffix:expr)),*) => {
            paste!{
            $(
                #[test]
                fn [<test_try_from_integer_ $suffix>]() {
                    let dec = PreciseDecimal::try_from($from).unwrap();
                    assert_eq!(dec.to_string(), $expected)
                }
            )*
            }
        };
    }

    test_try_from_integer! {
        (I256::ONE, "1", 1),
        (-I256::ONE, "-1", 2),
        // maximal PreciseDecimal integer part
        (I256::MAX/(I256::from(10).pow(PreciseDecimal::SCALE)), "57896044618658097711785492504343953926634", 3),
        // minimal PreciseDecimal integer part
        (I256::MIN/(I256::from(10).pow(PreciseDecimal::SCALE)), "-57896044618658097711785492504343953926634", 4),
        (U256::MIN, "0", 5)
    }

    macro_rules! test_from_integer {
        ($(($from:expr, $expected:expr, $suffix:expr)),*) => {
            paste!{
            $(
                #[test]
                fn [<test_from_integer_ $suffix>]() {
                    let dec = PreciseDecimal::from($from);
                    assert_eq!(dec.to_string(), $expected)
                }
            )*
            }
        };
    }

    test_from_integer! {
        (I192::ONE, "1", 1),
        (-I192::ONE, "-1", 2),
        (U192::MIN, "0", 3),
        (U192::ONE, "1", 4)
    }

    #[test]
    fn test_truncate_precise_decimal() {
        let pdec = pdec!("12345678.123456789012345678901234567890123456");
        assert_eq!(pdec.truncate().to_string(), "12345678.123456789012345678");
    }

    #[test]
    fn test_truncate_1_precise_decimal() {
        let pdec = pdec!(1);
        assert_eq!(pdec.truncate().to_string(), "1");
    }

    #[test]
    fn test_truncate_123_5_precise_decimal() {
        let pdec = pdec!("123.5");
        assert_eq!(pdec.truncate().to_string(), "123.5");
    }

    #[test]
    fn test_sqrt() {
        let sqrt_of_42 = pdec!(42).sqrt();
        let sqrt_of_0 = pdec!(0).sqrt();
        let sqrt_of_negative = pdec!("-1").sqrt();
        assert_eq!(
            sqrt_of_42.unwrap(),
            pdec!("6.480740698407860230965967436087996657")
        );
        assert_eq!(sqrt_of_0.unwrap(), pdec!(0));
        assert_eq!(sqrt_of_negative, None);
    }

    #[test]
    fn test_cbrt() {
        let cbrt_of_42 = pdec!(42).cbrt();
        let cbrt_of_0 = pdec!(0).cbrt();
        let cbrt_of_negative_42 = pdec!("-42").cbrt();
        assert_eq!(cbrt_of_42, pdec!("3.476026644886449786739865219004537434"));
        assert_eq!(cbrt_of_0, pdec!("0"));
        assert_eq!(
            cbrt_of_negative_42,
            pdec!("-3.476026644886449786739865219004537434")
        );
    }

    #[test]
    fn test_nth_root() {
        let root_4_42 = pdec!(42).nth_root(4);
        let root_5_42 = pdec!(42).nth_root(5);
        let root_42_42 = pdec!(42).nth_root(42);
        let root_neg_4_42 = pdec!("-42").nth_root(4);
        let root_neg_5_42 = pdec!("-42").nth_root(5);
        let root_0 = pdec!(42).nth_root(0);
        assert_eq!(
            root_4_42.unwrap(),
            pdec!("2.545729895021830518269788960576288685")
        );
        assert_eq!(
            root_5_42.unwrap(),
            pdec!("2.111785764966753912732567330550233486")
        );
        assert_eq!(
            root_42_42.unwrap(),
            pdec!("1.093072057934823618682784731855625786")
        );
        assert_eq!(root_neg_4_42, None);
        assert_eq!(
            root_neg_5_42.unwrap(),
            pdec!("-2.111785764966753912732567330550233486")
        );
        assert_eq!(root_0, None);
    }

    #[test]
    fn no_panic_with_36_decimal_places() {
        // Arrange
        let string = "1.111111111111111111111111111111111111";

        // Act
        let decimal = PreciseDecimal::from_str(string);

        // Assert
        assert!(decimal.is_ok())
    }

    #[test]
    fn no_panic_with_37_decimal_places() {
        // Arrange
        let string = "1.1111111111111111111111111111111111111";

        // Act
        let decimal = PreciseDecimal::from_str(string);

        // Assert
        assert!(matches!(
            decimal,
            Err(ParsePreciseDecimalError::UnsupportedDecimalPlace)
        ))
    }

    // These tests make sure that any basic arithmetic operation
    // between Decimal and PreciseDecimal produces a PreciseDecimal, no matter the order.
    // Additionally result of such operation shall be equal, if operands are derived from the same
    // value
    // Example:
    //   Decimal(10) * PreciseDecimal(10) -> PreciseDecimal(100)
    //   PreciseDecimal(10) * Decimal(10) -> PreciseDecimal(100)
    #[test]
    fn test_arith_precise_decimal_decimal() {
        let p1 = PreciseDecimal::from(Decimal::MAX);
        let d1 = Decimal::from(2);
        let d2 = Decimal::MAX;
        let p2 = PreciseDecimal::from(2);
        assert_eq!(p1.safe_mul(d1).unwrap(), d2.safe_mul(p2).unwrap());
        assert_eq!(p1.safe_div(d1).unwrap(), d2.safe_div(p2).unwrap());
        assert_eq!(p1.safe_add(d1).unwrap(), d2.safe_add(p2).unwrap());
        assert_eq!(p1.safe_sub(d1).unwrap(), d2.safe_sub(p2).unwrap());

        let p1 = PreciseDecimal::from(Decimal::MIN);
        let d1 = Decimal::from(2);
        let d2 = Decimal::MIN;
        let p2 = PreciseDecimal::from(2);
        assert_eq!(p1.safe_mul(d1).unwrap(), d2.safe_mul(p2).unwrap());
        assert_eq!(p1.safe_div(d1).unwrap(), d2.safe_div(p2).unwrap());
        assert_eq!(p1.safe_add(d1).unwrap(), d2.safe_add(p2).unwrap());
        assert_eq!(p1.safe_sub(d1).unwrap(), d2.safe_sub(p2).unwrap());

        let p1 = pdec!("0.000001");
        let d1 = dec!("0.001");
        let d2 = dec!("0.000001");
        let p2 = pdec!("0.001");
        assert_eq!(p1.safe_mul(d1).unwrap(), d2.safe_mul(p2).unwrap());
        assert_eq!(p1.safe_div(d1).unwrap(), d2.safe_div(p2).unwrap());
        assert_eq!(p1.safe_add(d1).unwrap(), d2.safe_add(p2).unwrap());
        assert_eq!(p1.safe_sub(d1).unwrap(), d2.safe_sub(p2).unwrap());

        let p1 = pdec!("0.000000000000000001");
        let d1 = Decimal::MIN;
        let d2 = dec!("0.000000000000000001");
        let p2 = PreciseDecimal::from(Decimal::MIN);
        assert_eq!(p1.safe_mul(d1).unwrap(), d2.safe_mul(p2).unwrap());
        assert_eq!(p1.safe_div(d1).unwrap(), d2.safe_div(p2).unwrap());
        assert_eq!(p1.safe_add(d1).unwrap(), d2.safe_add(p2).unwrap());
        assert_eq!(p1.safe_sub(d1).unwrap(), d2.safe_sub(p2).unwrap());

        let p1 = PreciseDecimal::ZERO;
        let d1 = Decimal::ONE;
        let d2 = Decimal::ZERO;
        let p2 = PreciseDecimal::ONE;
        assert_eq!(p1.safe_mul(d1).unwrap(), d2.safe_mul(p2).unwrap());
        assert_eq!(p1.safe_div(d1).unwrap(), d2.safe_div(p2).unwrap());
        assert_eq!(p1.safe_add(d1).unwrap(), d2.safe_add(p2).unwrap());
        assert_eq!(p1.safe_sub(d1).unwrap(), d2.safe_sub(p2).unwrap());
    }

    // These tests make sure that any basic arithmetic operation
    // between primitive type and PreciseDecimal produces a PreciseDecimal, no matter the order.
    // Additionally result of such operation shall be equal, if operands are derived from the same
    // value
    // Example:
    //   PreciseDecimal(10) * 10_u32 -> PreciseDecimal(100)
    //   10_u32 * PreciseDecimal(10) -> PreciseDecimal(100)
    macro_rules! test_arith_precise_decimal_primitive {
        ($type:ident) => {
            paste! {
                #[test]
                fn [<test_arith_precise_decimal_$type>]() {
                    let d1 = PreciseDecimal::ONE;
                    let u1 = 2 as $type;
                    let u2 = 1 as $type;
                    let d2 = PreciseDecimal::from(2);
                    assert_eq!(d1.safe_mul(u1).unwrap(), u2.safe_mul(d2).unwrap());
                    assert_eq!(d1.safe_div(u1).unwrap(), u2.safe_div(d2).unwrap());
                    assert_eq!(d1.safe_add(u1).unwrap(), u2.safe_add(d2).unwrap());
                    assert_eq!(d1.safe_sub(u1).unwrap(), u2.safe_sub(d2).unwrap());

                    let d1 = pdec!("2");
                    let u1 = $type::MAX;
                    let u2 = 2 as $type;
                    let d2 = PreciseDecimal::from($type::MAX);
                    assert_eq!(d1.safe_mul(u1).unwrap(), u2.safe_mul(d2).unwrap());
                    assert_eq!(d1.safe_div(u1).unwrap(), u2.safe_div(d2).unwrap());
                    assert_eq!(d1.safe_add(u1).unwrap(), u2.safe_add(d2).unwrap());
                    assert_eq!(d1.safe_sub(u1).unwrap(), u2.safe_sub(d2).unwrap());

                    let d1 = PreciseDecimal::from($type::MIN);
                    let u1 = 2 as $type;
                    let u2 = $type::MIN;
                    let d2 = pdec!("2");
                    assert_eq!(d1.safe_mul(u1).unwrap(), u2.safe_mul(d2).unwrap());
                    assert_eq!(d1.safe_div(u1).unwrap(), u2.safe_div(d2).unwrap());
                    assert_eq!(d1.safe_add(u1).unwrap(), u2.safe_add(d2).unwrap());
                    assert_eq!(d1.safe_sub(u1).unwrap(), u2.safe_sub(d2).unwrap());
                }
            }
        };
    }
    test_arith_precise_decimal_primitive!(u8);
    test_arith_precise_decimal_primitive!(u16);
    test_arith_precise_decimal_primitive!(u32);
    test_arith_precise_decimal_primitive!(u64);
    test_arith_precise_decimal_primitive!(u128);
    test_arith_precise_decimal_primitive!(usize);
    test_arith_precise_decimal_primitive!(i8);
    test_arith_precise_decimal_primitive!(i16);
    test_arith_precise_decimal_primitive!(i32);
    test_arith_precise_decimal_primitive!(i64);
    test_arith_precise_decimal_primitive!(i128);
    test_arith_precise_decimal_primitive!(isize);
}
