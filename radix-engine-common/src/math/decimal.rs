#[cfg(feature = "radix_engine_fuzzing")]
use arbitrary::Arbitrary;
use core::cmp::Ordering;
use num_bigint::BigInt;
use num_traits::{Pow, Zero};
use sbor::rust::convert::TryFrom;
use sbor::rust::fmt;
use sbor::rust::format;
use sbor::rust::prelude::*;
use sbor::*;

use crate::data::manifest::ManifestCustomValueKind;
use crate::data::scrypto::*;
use crate::math::bnum_integer::*;
use crate::math::rounding_mode::*;
use crate::math::traits::*;
use crate::math::PreciseDecimal;
use crate::well_known_scrypto_custom_type;
use crate::*;

/// `Decimal` represents a 192 bit representation of a fixed-scale decimal number.
///
/// The finite set of values are of the form `m / 10^18`, where `m` is
/// an integer such that `-2^(192 - 1) <= m < 2^(192 - 1)`.
///
/// Fractional part: ~60 bits/18 digits
/// Integer part   : 132 bits /40 digits
/// Max            :  3138550867693340381917894711603833208051.177722232017256447
/// Min            : -3138550867693340381917894711603833208051.177722232017256448
///
/// Unless otherwise specified, all operations will panic if underflow/overflow.
#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Decimal(pub I192);

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

    pub const ONE: Self = Self(I192::from_digits([10_u64.pow(Decimal::SCALE), 0, 0]));

    /// Returns `Decimal` of 0.
    pub const fn zero() -> Self {
        Self::ZERO
    }

    /// Returns `Decimal` of 1.
    pub const fn one() -> Self {
        Self::ONE
    }

    /// Whether this decimal is zero.
    pub fn is_zero(&self) -> bool {
        self.0 == I192::ZERO
    }

    /// Whether this decimal is positive.
    pub fn is_positive(&self) -> bool {
        self.0 > I192::ZERO
    }

    /// Whether this decimal is negative.
    pub fn is_negative(&self) -> bool {
        self.0 < I192::ZERO
    }

    /// Returns the absolute value.
    pub fn abs(&self) -> Decimal {
        Decimal(self.0.abs())
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
        let divisor: I192 = I192::TEN.pow(n);
        match mode {
            RoundingMode::ToPositiveInfinity => {
                if self.0 % divisor == I192::ZERO {
                    *self
                } else if self.is_negative() {
                    Self(self.0 / divisor * divisor)
                } else {
                    Self((self.0 / divisor + I192::ONE) * divisor)
                }
            }
            RoundingMode::ToNegativeInfinity => {
                if self.0 % divisor == I192::ZERO {
                    *self
                } else if self.is_negative() {
                    Self((self.0 / divisor - I192::ONE) * divisor)
                } else {
                    Self(self.0 / divisor * divisor)
                }
            }
            RoundingMode::ToZero => {
                if self.0 % divisor == I192::ZERO {
                    *self
                } else {
                    Self(self.0 / divisor * divisor)
                }
            }
            RoundingMode::AwayFromZero => {
                if self.0 % divisor == I192::ZERO {
                    *self
                } else if self.is_negative() {
                    Self((self.0 / divisor - I192::ONE) * divisor)
                } else {
                    Self((self.0 / divisor + I192::ONE) * divisor)
                }
            }
            RoundingMode::ToNearestMidpointTowardZero => {
                let remainder = (self.0 % divisor).abs();
                if remainder == I192::ZERO {
                    *self
                } else {
                    let mid_point = divisor / I192::from(2);
                    if remainder > mid_point {
                        if self.is_negative() {
                            Self((self.0 / divisor - I192::ONE) * divisor)
                        } else {
                            Self((self.0 / divisor + I192::ONE) * divisor)
                        }
                    } else {
                        Self(self.0 / divisor * divisor)
                    }
                }
            }
            RoundingMode::ToNearestMidpointAwayFromZero => {
                let remainder = (self.0 % divisor).abs();
                if remainder == I192::ZERO {
                    *self
                } else {
                    let mid_point = divisor / I192::from(2);
                    if remainder >= mid_point {
                        if self.is_negative() {
                            Self((self.0 / divisor - I192::ONE) * divisor)
                        } else {
                            Self((self.0 / divisor + I192::ONE) * divisor)
                        }
                    } else {
                        Self(self.0 / divisor * divisor)
                    }
                }
            }
            RoundingMode::ToNearestMidpointToEven => {
                let remainder = (self.0 % divisor).abs();
                if remainder == I192::ZERO {
                    *self
                } else {
                    let mid_point = divisor / I192::from(2);
                    match remainder.cmp(&mid_point) {
                        Ordering::Greater => {
                            if self.is_negative() {
                                Self((self.0 / divisor - I192::ONE) * divisor)
                            } else {
                                Self((self.0 / divisor + I192::ONE) * divisor)
                            }
                        }
                        Ordering::Equal => {
                            if self.0 / divisor % I192::from(2) == I192::ZERO {
                                Self(self.0 / divisor * divisor)
                            } else if self.is_negative() {
                                Self((self.0 / divisor - I192::ONE) * divisor)
                            } else {
                                Self((self.0 / divisor + I192::ONE) * divisor)
                            }
                        }
                        Ordering::Less => Self(self.0 / divisor * divisor),
                    }
                }
            }
        }
    }

    /// Calculates power using exponentiation by squaring".
    pub fn safe_powi(&self, exp: i64) -> Option<Self> {
        let one_256 = I256::from(Self::ONE.0);
        let base_256 = I256::from(self.0);
        let div = |x: i64, y: i64| x.checked_div(y);
        let sub = |x: i64, y: i64| x.checked_sub(y);
        let mul = |x: i64, y: i64| x.checked_mul(y);

        if exp < 0 {
            let dec_192 = I192::try_from((one_256 * one_256).safe_div(base_256)?).ok()?;
            let exp = mul(exp, -1)?;
            return Self(dec_192).safe_powi(exp);
        }
        if exp == 0 {
            return Some(Self::ONE);
        }
        if exp == 1 {
            return Some(*self);
        }
        if exp % 2 == 0 {
            let dec_192 = I192::try_from(base_256.safe_mul(base_256)? / one_256).ok()?;
            let exp = div(exp, 2)?;
            Self(dec_192).safe_powi(exp)
        } else {
            let dec_192 = I192::try_from(base_256.safe_mul(base_256)? / one_256).ok()?;
            let sub_dec = Self(dec_192);
            let exp = div(sub(exp, 1)?, 2)?;
            let b = sub_dec.safe_powi(exp)?;
            self.safe_mul(b)
        }
    }

    /// Square root of a Decimal
    pub fn sqrt(&self) -> Option<Self> {
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
    pub fn cbrt(&self) -> Option<Self> {
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
    pub fn nth_root(&self, n: u32) -> Option<Self> {
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

macro_rules! from_int {
    ($type:ident) => {
        impl From<$type> for Decimal {
            fn from(val: $type) -> Self {
                Self(I192::from(val) * Self::ONE.0)
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
            Self::from(1u8)
        } else {
            Self::from(0u8)
        }
    }
}

impl SafeNeg<Decimal> for Decimal {
    type Output = Self;

    #[inline]
    fn safe_neg(self) -> Option<Self::Output> {
        let c = self.0.safe_neg();
        c.map(Self)
    }
}

impl SafeAdd<Decimal> for Decimal {
    type Output = Self;

    #[inline]
    fn safe_add(self, other: Self) -> Option<Self::Output> {
        let a = self.0;
        let b = other.0;
        let c = a.safe_add(b);
        c.map(Self)
    }
}

impl SafeSub<Decimal> for Decimal {
    type Output = Self;

    #[inline]
    fn safe_sub(self, other: Self) -> Option<Self::Output> {
        let a = self.0;
        let b = other.0;
        let c = a.safe_sub(b);
        c.map(Self)
    }
}

impl SafeMul<Decimal> for Decimal {
    type Output = Self;

    #[inline]
    fn safe_mul(self, other: Self) -> Option<Self> {
        // Use I256 (BInt<4>) to not overflow.
        let a = I256::from(self.0);
        let b = I256::from(other.0);
        let mut c = a.safe_mul(b)?;
        c = c.safe_div(I256::from(Self::ONE.0))?;

        let c_192 = I192::try_from(c).ok();
        c_192.map(Self)
    }
}

impl SafeDiv<Decimal> for Decimal {
    type Output = Self;

    #[inline]
    fn safe_div(self, other: Self) -> Option<Self> {
        // Use I256 (BInt<4>) to not overflow.
        let a = I256::from(self.0);
        let b = I256::from(other.0);
        let mut c = a.safe_mul(I256::from(Self::ONE.0))?;
        c = c.safe_div(b)?;

        let c_192 = I192::try_from(c).ok();
        c_192.map(Self)
    }
}

macro_rules! impl_arith_ops {
    ($type:ident) => {
        impl SafeAdd<$type> for Decimal {
            type Output = Self;

            fn safe_add(self, other: $type) -> Option<Self::Output> {
                self.safe_add(Self::from(other))
            }
        }

        impl SafeSub<$type> for Decimal {
            type Output = Self;

            fn safe_sub(self, other: $type) -> Option<Self::Output> {
                self.safe_sub(Self::from(other))
            }
        }

        impl SafeMul<$type> for Decimal {
            type Output = Self;

            fn safe_mul(self, other: $type) -> Option<Self::Output> {
                self.safe_mul(Self::from(other))
            }
        }

        impl SafeDiv<$type> for Decimal {
            type Output = Self;

            fn safe_div(self, other: $type) -> Option<Self::Output> {
                self.safe_div(Self::from(other))
            }
        }

        impl SafeAdd<Decimal> for $type {
            type Output = Decimal;

            #[inline]
            fn safe_add(self, other: Decimal) -> Option<Self::Output> {
                other.safe_add(self)
            }
        }

        impl SafeSub<Decimal> for $type {
            type Output = Decimal;

            fn safe_sub(self, other: Decimal) -> Option<Self::Output> {
                Decimal::from(self).safe_sub(other)
            }
        }

        impl SafeMul<Decimal> for $type {
            type Output = Decimal;

            #[inline]
            fn safe_mul(self, other: Decimal) -> Option<Self::Output> {
                other.safe_mul(self)
            }
        }

        impl SafeDiv<Decimal> for $type {
            type Output = Decimal;

            fn safe_div(self, other: Decimal) -> Option<Self::Output> {
                Decimal::from(self).safe_div(other)
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

//========
// binary
//========

impl TryFrom<&[u8]> for Decimal {
    type Error = ParseDecimalError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        if slice.len() == Self::BITS / 8 {
            match I192::try_from(slice) {
                Ok(val) => Ok(Self(val)),
                Err(_) => Err(ParseDecimalError::Overflow),
            }
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
        let tens = I192::from(10);
        let v: Vec<&str> = s.split('.').collect();

        let mut int = match I192::from_str(v[0]) {
            Ok(val) => val,
            Err(_) => return Err(ParseDecimalError::InvalidDigit),
        };

        int *= tens.pow(Self::SCALE);

        if v.len() == 2 {
            let scale = if let Some(scale) = Self::SCALE.checked_sub(v[1].len() as u32) {
                Ok(scale)
            } else {
                Err(Self::Err::UnsupportedDecimalPlace)
            }?;

            let frac = match I192::from_str(v[1]) {
                Ok(val) => val,
                Err(_) => return Err(ParseDecimalError::InvalidDigit),
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
    InvalidDecimal(String),
    InvalidChar(char),
    InvalidDigit,
    UnsupportedDecimalPlace,
    InvalidLength(usize),
    Overflow,
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
        let val_i256 = val.0 / I256::from(10i8).pow(PreciseDecimal::SCALE - Decimal::SCALE);
        let result = I192::try_from(val_i256);
        match result {
            Ok(val_i192) => Ok(Self(val_i192)),
            Err(_) => Err(ParseDecimalError::Overflow),
        }
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
                            match val.safe_mul(Self::ONE.0) {
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
try_from_integer!(I192, I256, I512, U192, U256, U512);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dec;
    use paste::paste;

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
            Decimal::from_str("-3138550867693340381917894711603833208051.177722232017256448")
                .unwrap(),
            Decimal::MIN,
        );
    }

    #[test]
    fn test_add_decimal() {
        let a = Decimal::from(5u32);
        let b = Decimal::from(7u32);
        assert_eq!(a.safe_add(b).unwrap().to_string(), "12");
    }

    #[test]
    #[should_panic(expected = "Overflow")]
    fn test_add_overflow_decimal() {
        let _ = Decimal::MAX.safe_add(Decimal::ONE).expect("Overflow");
    }

    #[test]
    fn test_sub_decimal() {
        let a = Decimal::from(5u32);
        let b = Decimal::from(7u32);
        assert_eq!(a.safe_sub(b).unwrap().to_string(), "-2");
        assert_eq!(b.safe_sub(a).unwrap().to_string(), "2");
    }

    #[test]
    #[should_panic(expected = "Overflow")]
    fn test_sub_overflow_decimal() {
        let _ = Decimal::MIN.safe_sub(Decimal::ONE).expect("Overflow");
    }

    #[test]
    fn test_mul_decimal() {
        let a = Decimal::from(5u32);
        let b = Decimal::from(7u32);
        assert_eq!(a.safe_mul(b).unwrap().to_string(), "35");
        let a = Decimal::from_str("1000000000").unwrap();
        let b = Decimal::from_str("1000000000").unwrap();
        assert_eq!(a.safe_mul(b).unwrap().to_string(), "1000000000000000000");
        let a = Decimal::MAX;
        let b = dec!(1);
        assert_eq!(a.safe_mul(b).unwrap(), Decimal::MAX);
    }

    #[test]
    #[should_panic(expected = "Overflow")]
    fn test_mul_overflow_by_small_decimal() {
        let _ = Decimal::MAX
            .safe_mul(dec!("1.000000000000000001"))
            .expect("Overflow");
    }

    #[test]
    #[should_panic(expected = "Overflow")]
    fn test_mul_overflow_by_a_lot_decimal() {
        let _ = Decimal::MAX.safe_mul(dec!("1.1")).expect("Overflow");
    }

    #[test]
    #[should_panic(expected = "Overflow")]
    fn test_mul_neg_overflow_decimal() {
        let _ = Decimal::MAX
            .safe_neg()
            .unwrap()
            .safe_mul(dec!("-1.000000000000000001"))
            .expect("Overflow");
    }

    #[test]
    #[should_panic]
    fn test_div_by_zero_decimal() {
        let a = Decimal::from(5u32);
        let b = Decimal::from(0u32);
        a.safe_div(b).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_powi_exp_overflow_decimal() {
        let a = Decimal::from(5u32);
        let b = i64::MIN;
        assert_eq!(a.safe_powi(b).unwrap().to_string(), "0");
    }

    #[test]
    fn test_1_powi_max_decimal() {
        let a = Decimal::from(1u32);
        let b = i64::MAX;
        assert_eq!(a.safe_powi(b).unwrap().to_string(), "1");
    }

    #[test]
    fn test_1_powi_min_decimal() {
        let a = Decimal::from(1u32);
        let b = i64::MAX - 1;
        assert_eq!(a.safe_powi(b).unwrap().to_string(), "1");
    }

    #[test]
    fn test_powi_max_decimal() {
        let _max = Decimal::MAX.safe_powi(1);
        let _max_sqrt = Decimal::MAX.sqrt().unwrap();
        let _max_cbrt = Decimal::MAX.cbrt().unwrap();
        let _max_dec_2 = _max_sqrt.safe_powi(2).unwrap();
        let _max_dec_3 = _max_cbrt.safe_powi(3).unwrap();
    }

    #[test]
    fn test_div_decimal() {
        let a = Decimal::from(5u32);
        let b = Decimal::from(7u32);
        assert_eq!(a.safe_div(b).unwrap().to_string(), "0.714285714285714285");
        assert_eq!(b.safe_div(a).unwrap().to_string(), "1.4");
        assert_eq!(Decimal::MAX.safe_div(dec!(1)).unwrap(), Decimal::MAX);
    }

    #[test]
    fn test_div_negative_decimal() {
        let a = Decimal::from(-42);
        let b = Decimal::from(2);
        assert_eq!(a.safe_div(b).unwrap().to_string(), "-21");
    }

    #[test]
    fn test_0_pow_0_decimal() {
        let a = dec!("0");
        assert_eq!((a.safe_powi(0).unwrap()).to_string(), "1");
    }

    #[test]
    fn test_0_powi_1_decimal() {
        let a = dec!("0");
        assert_eq!((a.safe_powi(1).unwrap()).to_string(), "0");
    }

    #[test]
    fn test_0_powi_10_decimal() {
        let a = dec!("0");
        assert_eq!((a.safe_powi(10).unwrap()).to_string(), "0");
    }

    #[test]
    fn test_1_powi_0_decimal() {
        let a = dec!(1);
        assert_eq!((a.safe_powi(0).unwrap()).to_string(), "1");
    }

    #[test]
    fn test_1_powi_1_decimal() {
        let a = dec!(1);
        assert_eq!((a.safe_powi(1).unwrap()).to_string(), "1");
    }

    #[test]
    fn test_1_powi_10_decimal() {
        let a = dec!(1);
        assert_eq!((a.safe_powi(10).unwrap()).to_string(), "1");
    }

    #[test]
    fn test_2_powi_0_decimal() {
        let a = dec!("2");
        assert_eq!(a.safe_powi(0).unwrap().to_string(), "1");
    }

    #[test]
    fn test_2_powi_3724_decimal() {
        let a = dec!("1.000234891009084238");
        assert_eq!(
            a.safe_powi(3724).unwrap().to_string(),
            "2.397991232254669619"
        );
    }

    #[test]
    fn test_2_powi_2_decimal() {
        let a = dec!("2");
        assert_eq!(a.safe_powi(2).unwrap().to_string(), "4");
    }

    #[test]
    fn test_2_powi_3_decimal() {
        let a = dec!("2");
        assert_eq!(a.safe_powi(3).unwrap().to_string(), "8");
    }

    #[test]
    fn test_10_powi_3_decimal() {
        let a = dec!("10");
        assert_eq!(a.safe_powi(3).unwrap().to_string(), "1000");
    }

    #[test]
    fn test_5_powi_2_decimal() {
        let a = dec!("5");
        assert_eq!(a.safe_powi(2).unwrap().to_string(), "25");
    }

    #[test]
    fn test_5_powi_minus2_decimal() {
        let a = dec!("5");
        assert_eq!(a.safe_powi(-2).unwrap().to_string(), "0.04");
    }

    #[test]
    fn test_10_powi_minus3_decimal() {
        let a = dec!("10");
        assert_eq!(a.safe_powi(-3).unwrap().to_string(), "0.001");
    }

    #[test]
    fn test_minus10_powi_minus3_decimal() {
        let a = dec!("-10");
        assert_eq!(a.safe_powi(-3).unwrap().to_string(), "-0.001");
    }

    #[test]
    fn test_minus10_powi_minus2_decimal() {
        let a = dec!("-10");
        assert_eq!(a.safe_powi(-2).unwrap().to_string(), "0.01");
    }

    #[test]
    fn test_minus05_powi_minus2_decimal() {
        let a = dec!("-0.5");
        assert_eq!(a.safe_powi(-2).unwrap().to_string(), "4");
    }
    #[test]
    fn test_minus05_powi_minus3_decimal() {
        let a = dec!("-0.5");
        assert_eq!(a.safe_powi(-3).unwrap().to_string(), "-8");
    }

    #[test]
    fn test_10_powi_15_decimal() {
        let a = dec!(10i128);
        assert_eq!(a.safe_powi(15).unwrap().to_string(), "1000000000000000");
    }

    #[test]
    #[should_panic]
    fn test_10_powi_16_decimal() {
        let a = Decimal(10i128.into());
        assert_eq!(
            a.safe_powi(16).unwrap().to_string(),
            "1000000000000000000000"
        );
    }

    #[test]
    fn test_one_and_zero_decimal() {
        assert_eq!(Decimal::one().to_string(), "1");
        assert_eq!(Decimal::zero().to_string(), "0");
    }

    #[test]
    fn test_dec_string_decimal_decimal() {
        assert_eq!(
            dec!("1.123456789012345678").to_string(),
            "1.123456789012345678"
        );
        assert_eq!(dec!("-5.6").to_string(), "-5.6");
    }

    #[test]
    fn test_dec_string_decimal() {
        assert_eq!(dec!(1).to_string(), "1");
        assert_eq!(dec!("0").to_string(), "0");
    }

    #[test]
    fn test_dec_int_decimal() {
        assert_eq!(dec!(1).to_string(), "1");
        assert_eq!(dec!(5).to_string(), "5");
    }

    #[test]
    fn test_dec_bool_decimal() {
        assert_eq!((dec!(false)).to_string(), "0");
    }

    #[test]
    fn test_dec_rational_decimal() {
        assert_eq!((dec!(11235, 0)).to_string(), "11235");
        assert_eq!((dec!(11235, -2)).to_string(), "112.35");
        assert_eq!((dec!(11235, 2)).to_string(), "1123500");

        assert_eq!(
            (dec!(112000000000000000001i128, -18)).to_string(),
            "112.000000000000000001"
        );

        assert_eq!(
            (dec!(112000000000000000001i128, -18)).to_string(),
            "112.000000000000000001"
        );
    }

    #[test]
    #[should_panic(expected = "Shift overflow")]
    fn test_shift_overflow_decimal() {
        // u32::MAX + 1
        dec!(1, 4_294_967_296i128); // use explicit type to defer error to runtime
    }

    #[test]
    fn test_floor_decimal() {
        assert_eq!(
            Decimal::MAX.floor().to_string(),
            "3138550867693340381917894711603833208051"
        );
        assert_eq!(dec!("1.2").floor().to_string(), "1");
        assert_eq!(dec!("1.0").floor().to_string(), "1");
        assert_eq!(dec!("0.9").floor().to_string(), "0");
        assert_eq!(dec!("0").floor().to_string(), "0");
        assert_eq!(dec!("-0.1").floor().to_string(), "-1");
        assert_eq!(dec!("-1").floor().to_string(), "-1");
        assert_eq!(dec!("-5.2").floor().to_string(), "-6");
    }

    #[test]
    #[should_panic]
    fn test_floor_overflow_decimal() {
        Decimal::MIN.floor();
    }

    #[test]
    fn test_ceiling_decimal() {
        assert_eq!(dec!("1.2").ceiling().to_string(), "2");
        assert_eq!(dec!("1.0").ceiling().to_string(), "1");
        assert_eq!(dec!("0.9").ceiling().to_string(), "1");
        assert_eq!(dec!("0").ceiling().to_string(), "0");
        assert_eq!(dec!("-0.1").ceiling().to_string(), "0");
        assert_eq!(dec!("-1").ceiling().to_string(), "-1");
        assert_eq!(dec!("-5.2").ceiling().to_string(), "-5");
        assert_eq!(
            Decimal::MIN.ceiling().to_string(),
            "-3138550867693340381917894711603833208051"
        );
    }

    #[test]
    #[should_panic]
    fn test_ceiling_overflow_decimal() {
        Decimal::MAX.ceiling();
    }

    #[test]
    fn test_rounding_to_zero_decimal() {
        let mode = RoundingMode::ToZero;
        assert_eq!(dec!("1.2").round(0, mode).to_string(), "1");
        assert_eq!(dec!("1.0").round(0, mode).to_string(), "1");
        assert_eq!(dec!("0.9").round(0, mode).to_string(), "0");
        assert_eq!(dec!("0").round(0, mode).to_string(), "0");
        assert_eq!(dec!("-0.1").round(0, mode).to_string(), "0");
        assert_eq!(dec!("-1").round(0, mode).to_string(), "-1");
        assert_eq!(dec!("-5.2").round(0, mode).to_string(), "-5");
    }

    #[test]
    fn test_rounding_away_from_zero_decimal() {
        let mode = RoundingMode::AwayFromZero;
        assert_eq!(dec!("1.2").round(0, mode).to_string(), "2");
        assert_eq!(dec!("1.0").round(0, mode).to_string(), "1");
        assert_eq!(dec!("0.9").round(0, mode).to_string(), "1");
        assert_eq!(dec!("0").round(0, mode).to_string(), "0");
        assert_eq!(dec!("-0.1").round(0, mode).to_string(), "-1");
        assert_eq!(dec!("-1").round(0, mode).to_string(), "-1");
        assert_eq!(dec!("-5.2").round(0, mode).to_string(), "-6");
    }

    #[test]
    fn test_rounding_midpoint_toward_zero_decimal() {
        let mode = RoundingMode::ToNearestMidpointTowardZero;
        assert_eq!(dec!("5.5").round(0, mode).to_string(), "5");
        assert_eq!(dec!("2.5").round(0, mode).to_string(), "2");
        assert_eq!(dec!("1.6").round(0, mode).to_string(), "2");
        assert_eq!(dec!("1.1").round(0, mode).to_string(), "1");
        assert_eq!(dec!("1.0").round(0, mode).to_string(), "1");
        assert_eq!(dec!("-1.0").round(0, mode).to_string(), "-1");
        assert_eq!(dec!("-1.1").round(0, mode).to_string(), "-1");
        assert_eq!(dec!("-1.6").round(0, mode).to_string(), "-2");
        assert_eq!(dec!("-2.5").round(0, mode).to_string(), "-2");
        assert_eq!(dec!("-5.5").round(0, mode).to_string(), "-5");
    }

    #[test]
    fn test_rounding_midpoint_away_from_zero_decimal() {
        let mode = RoundingMode::ToNearestMidpointAwayFromZero;
        assert_eq!(dec!("5.5").round(0, mode).to_string(), "6");
        assert_eq!(dec!("2.5").round(0, mode).to_string(), "3");
        assert_eq!(dec!("1.6").round(0, mode).to_string(), "2");
        assert_eq!(dec!("1.1").round(0, mode).to_string(), "1");
        assert_eq!(dec!("1.0").round(0, mode).to_string(), "1");
        assert_eq!(dec!("-1.0").round(0, mode).to_string(), "-1");
        assert_eq!(dec!("-1.1").round(0, mode).to_string(), "-1");
        assert_eq!(dec!("-1.6").round(0, mode).to_string(), "-2");
        assert_eq!(dec!("-2.5").round(0, mode).to_string(), "-3");
        assert_eq!(dec!("-5.5").round(0, mode).to_string(), "-6");
    }

    #[test]
    fn test_rounding_midpoint_nearest_even_zero_decimal() {
        let mode = RoundingMode::ToNearestMidpointToEven;
        assert_eq!(dec!("5.5").round(0, mode).to_string(), "6");
        assert_eq!(dec!("2.5").round(0, mode).to_string(), "2");
        assert_eq!(dec!("1.6").round(0, mode).to_string(), "2");
        assert_eq!(dec!("1.1").round(0, mode).to_string(), "1");
        assert_eq!(dec!("1.0").round(0, mode).to_string(), "1");
        assert_eq!(dec!("-1.0").round(0, mode).to_string(), "-1");
        assert_eq!(dec!("-1.1").round(0, mode).to_string(), "-1");
        assert_eq!(dec!("-1.6").round(0, mode).to_string(), "-2");
        assert_eq!(dec!("-2.5").round(0, mode).to_string(), "-2");
        assert_eq!(dec!("-5.5").round(0, mode).to_string(), "-6");
    }

    #[test]
    fn test_various_decimal_places_decimal() {
        let num = dec!("2.4595");
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

        let num = dec!("-2.4595");
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
    fn test_encode_decimal_value_decimal() {
        let dec = dec!("0");
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
        let dec = dec!("1.23456789");
        let bytes = scrypto_encode(&dec).unwrap();
        let decoded: Decimal = scrypto_decode(&bytes).unwrap();
        assert_eq!(decoded, dec!("1.23456789"));
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
        ("0.000000000000000000000000008901234567", "0", 2),
        ("-0.000000000000000000000000008901234567", "0", 3),
        ("5", "5", 4),
        ("12345678.1", "12345678.1", 5)
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
        (PreciseDecimal::from(Decimal::MAX).safe_add(Decimal::ONE).unwrap(), 3),
        (PreciseDecimal::from(Decimal::MIN).safe_sub(Decimal::ONE).unwrap(), 4)
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
        (U256::MAX, 5),
        (I512::MAX, 6),
        (I512::MIN, 7),
        (U512::MAX, 8)
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
        // maximal Decimal integer part
        (I192::MAX/(I192::from(10_u64.pow(Decimal::SCALE))), "3138550867693340381917894711603833208051", 3),
        // minimal Decimal integer part
        (I192::MIN/(I192::from(10_u64.pow(Decimal::SCALE))), "-3138550867693340381917894711603833208051", 4),
        (U256::MIN, "0", 5),
        (U512::MIN, "0", 6),
        (I512::ONE, "1", 7),
        (-I512::ONE, "-1", 8)
    }

    #[test]
    fn test_sqrt() {
        let sqrt_of_42 = dec!(42).sqrt();
        let sqrt_of_0 = dec!(0).sqrt();
        let sqrt_of_negative = dec!("-1").sqrt();
        let sqrt_max = Decimal::MAX.sqrt();
        assert_eq!(sqrt_of_42.unwrap(), dec!("6.48074069840786023"));
        assert_eq!(sqrt_of_0.unwrap(), dec!(0));
        assert_eq!(sqrt_of_negative, None);
        assert_eq!(
            sqrt_max.unwrap(),
            dec!("56022770974786139918.731938227458171762")
        );
    }

    #[test]
    fn test_cbrt() {
        let cbrt_of_42 = dec!(42).cbrt().unwrap();
        let cbrt_of_0 = dec!(0).cbrt().unwrap();
        let cbrt_of_negative_42 = dec!("-42").cbrt().unwrap();
        let cbrt_max = Decimal::MAX.cbrt().unwrap();
        assert_eq!(cbrt_of_42, dec!("3.476026644886449786"));
        assert_eq!(cbrt_of_0, dec!("0"));
        assert_eq!(cbrt_of_negative_42, dec!("-3.476026644886449786"));
        assert_eq!(cbrt_max, dec!("14641190473997.345813510937532903"));
    }

    #[test]
    fn test_nth_root() {
        let root_4_42 = dec!(42).nth_root(4);
        let root_5_42 = dec!(42).nth_root(5);
        let root_42_42 = dec!(42).nth_root(42);
        let root_neg_4_42 = dec!("-42").nth_root(4);
        let root_neg_5_42 = dec!("-42").nth_root(5);
        let root_0 = dec!(42).nth_root(0);
        assert_eq!(root_4_42.unwrap(), dec!("2.545729895021830518"));
        assert_eq!(root_5_42.unwrap(), dec!("2.111785764966753912"));
        assert_eq!(root_42_42.unwrap(), dec!("1.093072057934823618"));
        assert_eq!(root_neg_4_42, None);
        assert_eq!(root_neg_5_42.unwrap(), dec!("-2.111785764966753912"));
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
        assert!(matches!(
            decimal,
            Err(ParseDecimalError::UnsupportedDecimalPlace)
        ))
    }

    // These tests make sure that any basic arithmetic operation
    // between primitive type and Decimal produces a Decimal, no matter the order.
    // Additionally result of such operation shall be equal, if operands are derived from the same
    // value
    // Example:
    //   Decimal(10) * 10_u32 -> Decimal(100)
    //   10_u32 * Decimal(10) -> Decimal(100)
    macro_rules! test_arith_decimal_primitive {
        ($type:ident) => {
            paste! {
                #[test]
                fn [<test_arith_decimal_$type>]() {
                    let d1 = Decimal::ONE;
                    let u1 = 2 as $type;
                    let u2 = 1 as $type;
                    let d2 = Decimal::from(2);
                    assert_eq!(d1.safe_mul(u1).unwrap(), u2.safe_mul(d2).unwrap());
                    assert_eq!(d1.safe_div(u1).unwrap(), u2.safe_div(d2).unwrap());
                    assert_eq!(d1.safe_add(u1).unwrap(), u2.safe_add(d2).unwrap());
                    assert_eq!(d1.safe_sub(u1).unwrap(), u2.safe_sub(d2).unwrap());

                    let d1 = dec!("2");
                    let u1 = $type::MAX;
                    let u2 = 2 as $type;
                    let d2 = Decimal::from($type::MAX);
                    assert_eq!(d1.safe_mul(u1).unwrap(), u2.safe_mul(d2).unwrap());
                    assert_eq!(d1.safe_div(u1).unwrap(), u2.safe_div(d2).unwrap());
                    assert_eq!(d1.safe_add(u1).unwrap(), u2.safe_add(d2).unwrap());
                    assert_eq!(d1.safe_sub(u1).unwrap(), u2.safe_sub(d2).unwrap());

                    let d1 = Decimal::from($type::MIN);
                    let u1 = 2 as $type;
                    let u2 = $type::MIN;
                    let d2 = dec!("2");
                    assert_eq!(d1.safe_mul(u1).unwrap(), u2.safe_mul(d2).unwrap());
                    assert_eq!(d1.safe_div(u1).unwrap(), u2.safe_div(d2).unwrap());
                    assert_eq!(d1.safe_add(u1).unwrap(), u2.safe_add(d2).unwrap());
                    assert_eq!(d1.safe_sub(u1).unwrap(), u2.safe_sub(d2).unwrap());
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
}
