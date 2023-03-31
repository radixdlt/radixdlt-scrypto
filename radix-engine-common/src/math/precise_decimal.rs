use num_bigint::BigInt;
use num_traits::{One, Pow, Zero};
use sbor::rust::convert::{TryFrom, TryInto};
use sbor::rust::fmt;
use sbor::rust::format;
use sbor::rust::iter;
use sbor::rust::ops::*;
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
use crate::well_known_scrypto_custom_type;
use crate::*;

/// `PreciseDecimal` represents a 512 bit representation of a fixed-scale decimal number.
///
/// The finite set of values are of the form `m / 10^64`, where `m` is
/// an integer such that `-2^(512 - 1) <= m < 2^(512 - 1)`.
///
/// Unless otherwise specified, all operations will panic if underflow/overflow.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PreciseDecimal(pub BnumI512);

impl Default for PreciseDecimal {
    fn default() -> Self {
        Self::zero()
    }
}

impl iter::Sum for PreciseDecimal {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut sum = PreciseDecimal::zero();
        iter.for_each(|d| sum += d);
        sum
    }
}

// TODO come up with some smarter formatting depending on PreciseDecimal::Scale
macro_rules! fmt_remainder {
    () => {
        "{:064}"
    };
}

impl PreciseDecimal {
    /// The min value of `PreciseDecimal`.
    pub const MIN: Self = Self(BnumI512::MIN);

    /// The max value of `PreciseDecimal`.
    pub const MAX: Self = Self(BnumI512::MAX);

    /// The bit length of number storing `PreciseDecimal`.
    pub const BITS: usize = BnumI512::BITS as usize;

    /// The fixed scale used by `PreciseDecimal`.
    pub const SCALE: u32 = 64;

    pub const ZERO: Self = Self(BnumI512::ZERO);

    pub const ONE: Self = Self(BnumI512::from_digits([
        0,
        7942358959831785217,
        16807427164405733357,
        1593091,
        0,
        0,
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
        self.0 == BnumI512::zero()
    }

    /// Whether this decimal is positive.
    pub fn is_positive(&self) -> bool {
        self.0 > BnumI512::zero()
    }

    /// Whether this decimal is negative.
    pub fn is_negative(&self) -> bool {
        self.0 < BnumI512::zero()
    }

    /// Returns the absolute value.
    pub fn abs(&self) -> PreciseDecimal {
        PreciseDecimal(self.0.abs())
    }

    /// Returns the largest integer that is equal to or less than this number.
    pub fn floor(&self) -> Self {
        self.round(0, RoundingMode::TowardsNegativeInfinity)
    }

    /// Returns the smallest integer that is equal to or greater than this number.
    pub fn ceiling(&self) -> Self {
        self.round(0, RoundingMode::TowardsPositiveInfinity)
    }

    pub fn round(&self, decimal_places: u32, mode: RoundingMode) -> Self {
        assert!(decimal_places <= Self::SCALE);

        let divisor: BnumI512 = BnumI512::from(10i8).pow(Self::SCALE - decimal_places);
        match mode {
            RoundingMode::TowardsPositiveInfinity => {
                if self.0 % divisor == BnumI512::zero() {
                    self.clone()
                } else if self.is_negative() {
                    Self(self.0 / divisor * divisor)
                } else {
                    Self((self.0 / divisor + BnumI512::one()) * divisor)
                }
            }
            RoundingMode::TowardsNegativeInfinity => {
                if self.0 % divisor == BnumI512::zero() {
                    self.clone()
                } else if self.is_negative() {
                    Self((self.0 / divisor - BnumI512::one()) * divisor)
                } else {
                    Self(self.0 / divisor * divisor)
                }
            }
            RoundingMode::TowardsZero => {
                if self.0 % divisor == BnumI512::zero() {
                    self.clone()
                } else {
                    Self(self.0 / divisor * divisor)
                }
            }
            RoundingMode::AwayFromZero => {
                if self.0 % divisor == BnumI512::zero() {
                    self.clone()
                } else if self.is_negative() {
                    Self((self.0 / divisor - BnumI512::one()) * divisor)
                } else {
                    Self((self.0 / divisor + BnumI512::one()) * divisor)
                }
            }
            RoundingMode::TowardsNearestAndHalfTowardsZero => {
                if self.0 % divisor == BnumI512::zero() {
                    self.clone()
                } else {
                    let digit = (self.0 / (divisor / BnumI512::from(10i128))
                        % BnumI512::from(10i128))
                    .abs();
                    if digit > 5.into() {
                        if self.is_negative() {
                            Self((self.0 / divisor - BnumI512::one()) * divisor)
                        } else {
                            Self((self.0 / divisor + BnumI512::one()) * divisor)
                        }
                    } else {
                        Self(self.0 / divisor * divisor)
                    }
                }
            }
            RoundingMode::TowardsNearestAndHalfAwayFromZero => {
                if self.0 % divisor == BnumI512::zero() {
                    self.clone()
                } else {
                    let digit = (self.0 / (divisor / BnumI512::from(10i128))
                        % BnumI512::from(10i128))
                    .abs();
                    if digit < 5.into() {
                        Self(self.0 / divisor * divisor)
                    } else {
                        if self.is_negative() {
                            Self((self.0 / divisor - BnumI512::one()) * divisor)
                        } else {
                            Self((self.0 / divisor + BnumI512::one()) * divisor)
                        }
                    }
                }
            }
        }
    }

    /// Calculates power using exponentiation by squaring.
    pub fn powi(&self, exp: i64) -> Self {
        let one_768 = BnumI768::from(Self::ONE.0);
        let base_768 = BnumI768::from(self.0);
        let div = |x: i64, y: i64| x.checked_div(y).expect("Overflow");
        let sub = |x: i64, y: i64| x.checked_sub(y).expect("Overflow");
        let mul = |x: i64, y: i64| x.checked_mul(y).expect("Overflow");

        if exp < 0 {
            let sub_768 = one_768 * one_768 / base_768;
            let sub_512 = BnumI512::try_from(sub_768).expect("Overflow");
            return PreciseDecimal(sub_512).powi(mul(exp, -1));
        }
        if exp == 0 {
            return Self::ONE;
        }
        if exp == 1 {
            return *self;
        }
        if exp % 2 == 0 {
            let sub_768 = base_768 * base_768 / one_768;
            let sub_512 = BnumI512::try_from(sub_768).expect("Overflow");
            PreciseDecimal(sub_512).powi(div(exp, 2))
        } else {
            let sub_768 = base_768 * base_768 / one_768;
            let sub_512 = BnumI512::try_from(sub_768).expect("Overflow");
            let sub_pdec = PreciseDecimal(sub_512);
            *self * sub_pdec.powi(div(sub(exp, 1), 2))
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

        // The BnumI512 i associated to a Decimal d is : i = d*10^64.
        // Therefore, taking sqrt yields sqrt(i) = sqrt(d)*10^32 => We lost precision
        // To get the right precision, we compute : sqrt(i*10^64) = sqrt(d)*10^64
        let self_768 = BnumI768::from(self.0);
        let correct_nb = self_768 * BnumI768::from(PreciseDecimal::one().0);
        let sqrt = BnumI512::try_from(correct_nb.sqrt()).expect("Overflow");
        Some(PreciseDecimal(sqrt))
    }

    /// Cubic root of a PreciseDecimal
    pub fn cbrt(&self) -> Self {
        if self.is_zero() {
            return Self::ZERO;
        }

        // By reasoning in the same way as before, we realise that we need to multiply by 10^36
        let self_bigint = BigInt::from(self.0);
        let correct_nb: BigInt = self_bigint * BigInt::from(PreciseDecimal::one().0).pow(2_u32);
        let cbrt = BnumI512::try_from(correct_nb.cbrt()).unwrap();
        PreciseDecimal(cbrt)
    }

    /// Nth root of a PreciseDecimal
    pub fn nth_root(&self, n: u32) -> Option<Self> {
        if (self.is_negative() && n % 2 == 0) || n == 0 {
            None
        } else if n == 1 {
            Some(self.clone())
        } else {
            if self.is_zero() {
                return Some(Self::ZERO);
            }

            // By induction, we need to multiply by the (n-1)th power of 10^64.
            // To not overflow, we use BigInt
            let self_integer = BigInt::from(self.0);
            let correct_nb = self_integer * BigInt::from(PreciseDecimal::one().0).pow(n - 1);
            let nth_root = BnumI512::try_from(correct_nb.nth_root(n)).unwrap();
            Some(PreciseDecimal(nth_root))
        }
    }
}

macro_rules! from_int {
    ($type:ident) => {
        impl From<$type> for PreciseDecimal {
            fn from(val: $type) -> Self {
                Self(BnumI512::from(val) * Self::ONE.0)
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

impl<T: TryInto<PreciseDecimal>> Add<T> for PreciseDecimal
where
    <T as TryInto<PreciseDecimal>>::Error: fmt::Debug,
{
    type Output = PreciseDecimal;

    fn add(self, other: T) -> Self::Output {
        let a = self.0;
        let b_dec: PreciseDecimal = other.try_into().expect("Overflow");
        let b: BnumI512 = b_dec.0;
        let c = a + b;
        PreciseDecimal(c)
    }
}

impl<T: TryInto<PreciseDecimal>> Sub<T> for PreciseDecimal
where
    <T as TryInto<PreciseDecimal>>::Error: fmt::Debug,
{
    type Output = PreciseDecimal;

    fn sub(self, other: T) -> Self::Output {
        let a = self.0;
        let b_dec: PreciseDecimal = other.try_into().expect("Overflow");
        let b: BnumI512 = b_dec.0;
        let c: BnumI512 = a - b;
        PreciseDecimal(c)
    }
}

impl<T: TryInto<PreciseDecimal>> Mul<T> for PreciseDecimal
where
    <T as TryInto<PreciseDecimal>>::Error: fmt::Debug,
{
    type Output = PreciseDecimal;

    fn mul(self, other: T) -> Self::Output {
        // Use BnumI768 to not overflow.
        let a = BnumI768::from(self.0);
        let b_dec: PreciseDecimal = other.try_into().expect("Overflow");
        let b = BnumI768::from(b_dec.0);
        let c = a * b / BnumI768::from(Self::ONE.0);
        let c_512 = BnumI512::try_from(c).expect("Overflow");
        PreciseDecimal(c_512)
    }
}

impl<T: TryInto<PreciseDecimal>> Div<T> for PreciseDecimal
where
    <T as TryInto<PreciseDecimal>>::Error: fmt::Debug,
{
    type Output = PreciseDecimal;

    fn div(self, other: T) -> Self::Output {
        // Use BnumI768 to not overflow.
        let a = BnumI768::from(self.0);
        let b_dec: PreciseDecimal = other.try_into().expect("Overflow");
        let b = BnumI768::from(b_dec.0);
        let c = a * BnumI768::from(Self::ONE.0) / b;
        let c_512 = BnumI512::try_from(c).expect("Overflow");
        PreciseDecimal(c_512)
    }
}

impl Neg for PreciseDecimal {
    type Output = PreciseDecimal;

    fn neg(self) -> Self::Output {
        PreciseDecimal(-self.0)
    }
}

impl<T: TryInto<PreciseDecimal>> AddAssign<T> for PreciseDecimal
where
    <T as TryInto<PreciseDecimal>>::Error: fmt::Debug,
{
    fn add_assign(&mut self, other: T) {
        let other: PreciseDecimal = other.try_into().expect("Overflow");
        self.0 += other.0;
    }
}

impl<T: TryInto<PreciseDecimal>> SubAssign<T> for PreciseDecimal
where
    <T as TryInto<PreciseDecimal>>::Error: fmt::Debug,
{
    fn sub_assign(&mut self, other: T) {
        let other: PreciseDecimal = other.try_into().expect("Overflow");
        self.0 -= other.0;
    }
}

impl<T: TryInto<PreciseDecimal>> MulAssign<T> for PreciseDecimal
where
    <T as TryInto<PreciseDecimal>>::Error: fmt::Debug,
{
    fn mul_assign(&mut self, other: T) {
        let other: PreciseDecimal = other.try_into().expect("Overflow");
        self.0 *= other.0;
    }
}

impl<T: TryInto<PreciseDecimal>> DivAssign<T> for PreciseDecimal
where
    <T as TryInto<PreciseDecimal>>::Error: fmt::Debug,
{
    fn div_assign(&mut self, other: T) {
        let other: PreciseDecimal = other.try_into().expect("Overflow");
        self.0 /= other.0;
    }
}

//========
// binary
//========

impl TryFrom<&[u8]> for PreciseDecimal {
    type Error = ParsePreciseDecimalError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        if slice.len() == Self::BITS / 8 {
            match BnumI512::try_from(slice) {
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
    PRECISE_DECIMAL_ID
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
        let tens = BnumI512::from(10);
        let v: Vec<&str> = s.split('.').collect();

        let mut int = match BnumI512::from_str(v[0]) {
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

            let frac = match BnumI512::from_str(v[1]) {
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
        const MULTIPLIER: BnumI512 = PreciseDecimal::ONE.0;
        let quotient = self.0 / MULTIPLIER;
        let remainder = self.0 % MULTIPLIER;

        if !remainder.is_zero() {
            // print remainder with leading zeroes
            let mut sign = "".to_string();

            // take care of sign in case quotient == zere and remainder < 0,
            // eg.
            //  self.0=-100000000000000000 -> -0.1
            if remainder < BnumI512::ZERO && quotient == BnumI512::ZERO {
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
        Self(
            BnumI512::try_from(val.0).unwrap()
                * BnumI512::from(10i8).pow(Self::SCALE - Decimal::SCALE),
        )
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
            (self.0 / BnumI512::from(10i8).pow(PreciseDecimal::SCALE - Decimal::SCALE))
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
                    Self(BnumI512::from(val) * Self::ONE.0)
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
                    match BnumI512::try_from(val) {
                        Ok(val) => {
                            match val.checked_mul(Self::ONE.0) {
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

from_integer!(BnumI256, BnumU256);
try_from_integer!(BnumI512, BnumU512);

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
            "0.0000000000000000000000000000000000000000000000000000000000000001"
        );
        assert_eq!(
            PreciseDecimal(123456789123456789i128.into()).to_string(),
            "0.0000000000000000000000000000000000000000000000123456789123456789"
        );
        assert_eq!(PreciseDecimal(BnumI512::from(10).pow(64)).to_string(), "1");
        assert_eq!(
            PreciseDecimal(BnumI512::from(10).pow(64).mul(BnumI512::from(123))).to_string(),
            "123"
        );
        assert_eq!(
            PreciseDecimal(
                BnumI512::from_str(
                    "1234567890000000000000000000000000000000000000000000000000000000000000000"
                )
                .unwrap()
            )
            .to_string(),
            "123456789"
        );
        assert_eq!(
            PreciseDecimal::MAX.to_string(),
            "670390396497129854978701249910292306373968291029619668886178072186088201503677348840093714.9083451713845015929093243025426876941405973284973216824503042047"
        );
        assert_eq!(PreciseDecimal::MIN.is_negative(), true);
        assert_eq!(
            PreciseDecimal::MIN.to_string(),
            "-670390396497129854978701249910292306373968291029619668886178072186088201503677348840093714.9083451713845015929093243025426876941405973284973216824503042048"
        );
    }

    #[test]
    fn test_parse_precise_decimal() {
        assert_eq!(
            PreciseDecimal::from_str("0.000000000000000001").unwrap(),
            PreciseDecimal(BnumI512::from(10).pow(46)),
        );
        assert_eq!(
            PreciseDecimal::from_str("0.123456789123456789").unwrap(),
            PreciseDecimal(
                BnumI512::from(123456789123456789i128).mul(BnumI512::from(10i8).pow(46))
            ),
        );
        assert_eq!(
            PreciseDecimal::from_str("1").unwrap(),
            PreciseDecimal(BnumI512::from(10).pow(64)),
        );
        assert_eq!(
            PreciseDecimal::from_str("123456789123456789").unwrap(),
            PreciseDecimal(BnumI512::from(123456789123456789i128).mul(BnumI512::from(10).pow(64))),
        );
        assert_eq!(
            PreciseDecimal::from_str(
                "670390396497129854978701249910292306373968291029619668886178072186088201503677348840093714.9083451713845015929093243025426876941405973284973216824503042047"
            )
            .unwrap(),
            PreciseDecimal::MAX,
            );
        assert_eq!(
            PreciseDecimal::from_str(
                "-670390396497129854978701249910292306373968291029619668886178072186088201503677348840093714.9083451713845015929093243025426876941405973284973216824503042048"
            )
            .unwrap(),
            PreciseDecimal::MIN,
            );
    }

    #[test]
    fn test_add_precise_decimal() {
        let a = PreciseDecimal::from(5u32);
        let b = PreciseDecimal::from(7u32);
        assert_eq!((a + b).to_string(), "12");
    }

    #[test]
    #[should_panic(expected = "Overflow")]
    fn test_add_overflow_precise_decimal() {
        let _ = PreciseDecimal::MAX + 1;
    }

    #[test]
    fn test_sub_precise_decimal() {
        let a = PreciseDecimal::from(5u32);
        let b = PreciseDecimal::from(7u32);
        assert_eq!((a - b).to_string(), "-2");
        assert_eq!((b - a).to_string(), "2");
    }

    #[test]
    #[should_panic(expected = "Overflow")]
    fn test_sub_overflow_precise_decimal() {
        let _ = PreciseDecimal::MIN - 1;
    }

    #[test]
    fn test_mul_precise_decimal() {
        let a = PreciseDecimal::from(5u32);
        let b = PreciseDecimal::from(7u32);
        println!("a={} b={} a*b={}", a, b, a * b);
        assert_eq!((a * b).to_string(), "35");
        let a = PreciseDecimal::from_str("1000000000").unwrap();
        let b = PreciseDecimal::from_str("1000000000").unwrap();
        assert_eq!((a * b).to_string(), "1000000000000000000");
    }

    #[test]
    #[should_panic(expected = "Overflow")]
    fn test_mul_overflow_by_small_precise_decimal() {
        let _ = PreciseDecimal::MAX
            * pdec!("1.0000000000000000000000000000000000000000000000000000000000000001");
    }

    #[test]
    #[should_panic(expected = "Overflow")]
    fn test_mul_overflow_by_a_lot_precise_decimal() {
        let _ = PreciseDecimal::MAX * pdec!("1.1");
    }

    #[test]
    #[should_panic(expected = "Overflow")]
    fn test_mul_neg_overflow_precise_decimal() {
        let p = pdec!("-1.0000000000000000000000000000000000000000000000000000000000000001");
        println!("p = {}", p);
        let _ = (-PreciseDecimal::MAX)
            * pdec!("-1.0000000000000000000000000000000000000000000000000000000000000001");
    }

    #[test]
    #[should_panic]
    fn test_div_by_zero_precise_decimal() {
        let a = PreciseDecimal::from(5u32);
        let b = PreciseDecimal::from(0u32);
        assert_eq!((a / b).to_string(), "0");
    }

    #[test]
    #[should_panic]
    fn test_powi_exp_overflow_precise_decimal() {
        let a = PreciseDecimal::from(5u32);
        let b = i64::MIN;
        assert_eq!(a.powi(b).to_string(), "0");
    }

    #[test]
    fn test_1_powi_max_precise_decimal() {
        let a = PreciseDecimal::from(1u32);
        let b = i64::MAX;
        assert_eq!(a.powi(b).to_string(), "1");
    }

    #[test]
    fn test_1_powi_min_precise_decimal() {
        let a = PreciseDecimal::from(1u32);
        let b = i64::MAX - 1;
        assert_eq!(a.powi(b).to_string(), "1");
    }

    #[test]
    fn test_powi_max_precise_decimal() {
        let _max = PreciseDecimal::MAX.powi(1);
        let _max_sqrt = PreciseDecimal::MAX.sqrt().unwrap();
        let _max_cbrt = PreciseDecimal::MAX.cbrt();
        let _max_dec_2 = _max_sqrt.powi(2);
        let _max_dec_3 = _max_cbrt.powi(3);
    }

    #[test]
    fn test_div_precise_decimal() {
        let a = PreciseDecimal::from(5u32);
        let b = PreciseDecimal::from(7u32);
        assert_eq!(
            (a / b).to_string(),
            "0.7142857142857142857142857142857142857142857142857142857142857142"
        );
        assert_eq!((b / a).to_string(), "1.4");
    }

    #[test]
    fn test_div_negative_precise_decimal() {
        let a = PreciseDecimal::from(-42);
        let b = PreciseDecimal::from(2);
        assert_eq!((a / b).to_string(), "-21");
    }

    #[test]
    fn test_0_pow_0_precise_decimal() {
        let a = pdec!("0");
        assert_eq!((a.powi(0)).to_string(), "1");
    }

    #[test]
    fn test_0_powi_1_precise_decimal() {
        let a = pdec!("0");
        assert_eq!((a.powi(1)).to_string(), "0");
    }

    #[test]
    fn test_0_powi_10_precise_decimal() {
        let a = pdec!("0");
        assert_eq!((a.powi(10)).to_string(), "0");
    }

    #[test]
    fn test_1_powi_0_precise_decimal() {
        let a = pdec!("1");
        assert_eq!((a.powi(0)).to_string(), "1");
    }

    #[test]
    fn test_1_powi_1_precise_decimal() {
        let a = pdec!("1");
        assert_eq!((a.powi(1)).to_string(), "1");
    }

    #[test]
    fn test_1_powi_10_precise_decimal() {
        let a = pdec!("1");
        assert_eq!((a.powi(10)).to_string(), "1");
    }

    #[test]
    fn test_2_powi_0_precise_decimal() {
        let a = pdec!("2");
        assert_eq!((a.powi(0)).to_string(), "1");
    }

    #[test]
    fn test_2_powi_3724_precise_decimal() {
        let a = pdec!("1.000234891009084238");
        assert_eq!(
            (a.powi(3724)).to_string(),
            "2.3979912322546748642222795591580998788798125484487121999786797414"
        );
    }

    #[test]
    fn test_2_powi_2_precise_decimal() {
        let a = pdec!("2");
        assert_eq!((a.powi(2)).to_string(), "4");
    }

    #[test]
    fn test_2_powi_3_precise_decimal() {
        let a = pdec!("2");
        assert_eq!((a.powi(3)).to_string(), "8");
    }

    #[test]
    fn test_10_powi_3_precise_decimal() {
        let a = pdec!("10");
        assert_eq!((a.powi(3)).to_string(), "1000");
    }

    #[test]
    fn test_5_powi_2_precise_decimal() {
        let a = pdec!("5");
        assert_eq!((a.powi(2)).to_string(), "25");
    }

    #[test]
    fn test_5_powi_minus2_precise_decimal() {
        let a = pdec!("5");
        assert_eq!((a.powi(-2)).to_string(), "0.04");
    }

    #[test]
    fn test_10_powi_minus3_precise_decimal() {
        let a = pdec!("10");
        assert_eq!((a.powi(-3)).to_string(), "0.001");
    }

    #[test]
    fn test_minus10_powi_minus3_precise_decimal() {
        let a = pdec!("-10");
        assert_eq!((a.powi(-3)).to_string(), "-0.001");
    }

    #[test]
    fn test_minus10_powi_minus2_precise_decimal() {
        let a = pdec!("-10");
        assert_eq!((a.powi(-2)).to_string(), "0.01");
    }

    #[test]
    fn test_minus05_powi_minus2_precise_decimal() {
        let a = pdec!("-0.5");
        assert_eq!((a.powi(-2)).to_string(), "4");
    }
    #[test]
    fn test_minus05_powi_minus3_precise_decimal() {
        let a = pdec!("-0.5");
        assert_eq!((a.powi(-3)).to_string(), "-8");
    }

    #[test]
    fn test_10_powi_15_precise_decimal() {
        let a = pdec!(10i128);
        assert_eq!(a.powi(15).to_string(), "1000000000000000");
    }

    #[test]
    #[should_panic]
    fn test_10_powi_16_precise_decimal() {
        let a = PreciseDecimal(10i128.into());
        assert_eq!(a.powi(16).to_string(), "1000000000000000000000");
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
        assert_eq!(pdec!("1").to_string(), "1");
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
            "670390396497129854978701249910292306373968291029619668886178072186088201503677348840093714"
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
            "-670390396497129854978701249910292306373968291029619668886178072186088201503677348840093714"
        );
    }

    #[test]
    #[should_panic]
    fn test_ceiling_overflow_precise_decimal() {
        PreciseDecimal::MAX.ceiling();
    }

    #[test]
    fn test_round_towards_zero_precise_decimal() {
        let mode = RoundingMode::TowardsZero;
        assert_eq!(pdec!("1.2").round(0, mode).to_string(), "1");
        assert_eq!(pdec!("1.0").round(0, mode).to_string(), "1");
        assert_eq!(pdec!("0.9").round(0, mode).to_string(), "0");
        assert_eq!(pdec!("0").round(0, mode).to_string(), "0");
        assert_eq!(pdec!("-0.1").round(0, mode).to_string(), "0");
        assert_eq!(pdec!("-1").round(0, mode).to_string(), "-1");
        assert_eq!(pdec!("-5.2").round(0, mode).to_string(), "-5");
    }

    #[test]
    fn test_round_away_from_zero_precise_decimal() {
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
    fn test_round_towards_nearest_and_half_towards_zero_precise_decimal() {
        let mode = RoundingMode::TowardsNearestAndHalfTowardsZero;
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
    fn test_round_towards_nearest_and_half_away_from_zero_precise_decimal() {
        let mode = RoundingMode::TowardsNearestAndHalfAwayFromZero;
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
    fn test_various_decimal_places_precise_decimal() {
        let mode = RoundingMode::TowardsNearestAndHalfAwayFromZero;
        let num = pdec!("-2.5555555555555555555555555555555555555555555555555555555555555555");
        assert_eq!(num.round(0, mode).to_string(), "-3");
        assert_eq!(num.round(1, mode).to_string(), "-2.6");
        assert_eq!(num.round(2, mode).to_string(), "-2.56");
        assert_eq!(num.round(17, mode).to_string(), "-2.55555555555555556");
        assert_eq!(num.round(18, mode).to_string(), "-2.555555555555555556");
        assert_eq!(
            num.round(40, mode).to_string(),
            "-2.5555555555555555555555555555555555555556"
        );
        assert_eq!(
            num.round(50, mode).to_string(),
            "-2.55555555555555555555555555555555555555555555555556"
        );
        assert_eq!(
            num.round(63, mode).to_string(),
            "-2.555555555555555555555555555555555555555555555555555555555555556"
        );
    }

    #[test]
    fn test_sum_precise_decimal() {
        let decimals = vec![pdec!("1"), pdec!("2"), pdec!("3")];
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
            let mut a = [0; 66];
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
        (BnumI512::MAX, 1),
        (BnumI512::MIN, 2),
        // maximal PreciseDecimal integer part + 1
        (BnumI512::MAX/(BnumI512::from(10).pow(PreciseDecimal::SCALE)) + BnumI512::ONE, 3),
        // minimal PreciseDecimal integer part - 1
        (BnumI512::MIN/(BnumI512::from(10).pow(PreciseDecimal::SCALE)) - BnumI512::ONE, 4),
        (BnumI512::MIN, 5),
        (BnumU512::MAX, 6)
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
        (BnumI512::ONE, "1", 1),
        (-BnumI512::ONE, "-1", 2),
        // maximal PreciseDecimal integer part
        (BnumI512::MAX/(BnumI512::from(10).pow(PreciseDecimal::SCALE)), "670390396497129854978701249910292306373968291029619668886178072186088201503677348840093714" , 3),
        // minimal PreciseDecimal integer part
        (BnumI512::MIN/(BnumI512::from(10).pow(PreciseDecimal::SCALE)), "-670390396497129854978701249910292306373968291029619668886178072186088201503677348840093714" , 4),
        (BnumU512::MIN, "0", 5)
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
        (BnumI256::MAX, "57896044618658097711785492504343953926634992332820282019728792003956564819967", 1),
        (BnumI256::MIN, "-57896044618658097711785492504343953926634992332820282019728792003956564819968", 2),
        (BnumU256::MIN, "0", 3),
        (BnumU256::MAX, "115792089237316195423570985008687907853269984665640564039457584007913129639935", 4)
    }

    #[test]
    fn test_truncate_precise_decimal() {
        let pdec =
            pdec!("12345678.1234567890123456789012345678901234567890123456789012345678901234");
        assert_eq!(pdec.truncate().to_string(), "12345678.123456789012345678");
    }

    #[test]
    fn test_truncate_1_precise_decimal() {
        let pdec = pdec!("1");
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
            pdec!("6.4807406984078602309659674360879966577052043070583465497113543978")
        );
        assert_eq!(sqrt_of_0.unwrap(), pdec!(0));
        assert_eq!(sqrt_of_negative, None);
    }

    #[test]
    fn test_cbrt() {
        let cbrt_of_42 = pdec!(42).cbrt();
        let cbrt_of_0 = pdec!(0).cbrt();
        let cbrt_of_negative_42 = pdec!("-42").cbrt();
        assert_eq!(
            cbrt_of_42,
            pdec!("3.4760266448864497867398652190045374340048385387869674214742239567")
        );
        assert_eq!(cbrt_of_0, pdec!("0"));
        assert_eq!(
            cbrt_of_negative_42,
            pdec!("-3.4760266448864497867398652190045374340048385387869674214742239567")
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
            pdec!("2.5457298950218305182697889605762886851969608313019270894320607936")
        );
        assert_eq!(
            root_5_42.unwrap(),
            pdec!("2.1117857649667539127325673305502334863032026536306378208090387860")
        );
        assert_eq!(
            root_42_42.unwrap(),
            pdec!("1.0930720579348236186827847318556257862429004287369365621359139742")
        );
        assert_eq!(root_neg_4_42, None);
        assert_eq!(
            root_neg_5_42.unwrap(),
            pdec!("-2.1117857649667539127325673305502334863032026536306378208090387860")
        );
        assert_eq!(root_0, None);
    }

    #[test]
    fn no_panic_with_64_decimal_places() {
        // Arrange
        let string = "1.1111111111111111111111111111111111111111111111111111111111111111";

        // Act
        let decimal = PreciseDecimal::from_str(string);

        // Assert
        assert!(decimal.is_ok())
    }

    #[test]
    fn no_panic_with_65_decimal_places() {
        // Arrange
        let string = "1.11111111111111111111111111111111111111111111111111111111111111111";

        // Act
        let decimal = PreciseDecimal::from_str(string);

        // Assert
        assert!(matches!(
            decimal,
            Err(ParsePreciseDecimalError::UnsupportedDecimalPlace)
        ))
    }
}
