use core::ops::*;
use num_traits::{One, Pow, ToPrimitive, Zero};
use paste::paste;
use sbor::rust::convert::{TryFrom, TryInto};
use sbor::rust::fmt;
use sbor::rust::iter;
use sbor::rust::str::FromStr;
use sbor::rust::string::{String, ToString};
use sbor::rust::vec::Vec;
use sbor::*;

use crate::abi::*;
use crate::math::*;

/// `Decimal` represents a 256 bit representation of a fixed-scale decimal number.
///
/// The finite set of values are of the form `m / 10^18`, where `m` is
/// an integer such that `-2^(256 - 1) <= m < 2^(256 - 1)`.
///
/// Unless otherwise specified, all operations will panic if underflow/overflow.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Decimal(pub I256);

impl Default for Decimal {
    fn default() -> Self {
        Self::zero()
    }
}

impl iter::Sum for Decimal {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut sum = Decimal::zero();
        iter.for_each(|d| sum += d);
        sum
    }
}

impl Decimal {
    /// The min value of `Decimal`.
    pub const MIN: Self = Self(I256::MIN);

    /// The max value of `Decimal`.
    pub const MAX: Self = Self(I256::MAX);

    /// The bit length of number storing `Decimal`.
    pub const BITS: usize = I256::BITS as usize;

    /// The fixed scale used by `Decimal`.
    pub const SCALE: u32 = 18;

    pub const ZERO: Self = Self(I256([0; 32]));

    pub const ONE: Self = Self(I256([
        0x00, 0x00, 0x64, 0xA7, 0xB3, 0xB6, 0xE0, 0x0D, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00,
    ]));

    /// Returns `Decimal` of 0.
    pub fn zero() -> Self {
        Self::ZERO
    }

    /// Returns `Decimal` of 1.
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
    pub fn abs(&self) -> Decimal {
        Decimal(self.0.abs())
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

        let divisor: I256 = I256::from(10i8).pow(Self::SCALE - decimal_places);
        match mode {
            RoundingMode::TowardsPositiveInfinity => {
                if self.0 % divisor == I256::zero() {
                    self.clone()
                } else if self.is_negative() {
                    Self(self.0 / divisor * divisor)
                } else {
                    Self((self.0 / divisor + I256::one()) * divisor)
                }
            }
            RoundingMode::TowardsNegativeInfinity => {
                if self.0 % divisor == I256::zero() {
                    self.clone()
                } else if self.is_negative() {
                    Self((self.0 / divisor - I256::one()) * divisor)
                } else {
                    Self(self.0 / divisor * divisor)
                }
            }
            RoundingMode::TowardsZero => {
                if self.0 % divisor == I256::zero() {
                    self.clone()
                } else {
                    Self(self.0 / divisor * divisor)
                }
            }
            RoundingMode::AwayFromZero => {
                if self.0 % divisor == I256::zero() {
                    self.clone()
                } else if self.is_negative() {
                    Self((self.0 / divisor - I256::one()) * divisor)
                } else {
                    Self((self.0 / divisor + I256::one()) * divisor)
                }
            }
            RoundingMode::TowardsNearestAndHalfTowardsZero => {
                if self.0 % divisor == I256::zero() {
                    self.clone()
                } else {
                    let digit =
                        (self.0 / (divisor / I256::from(10i128)) % I256::from(10i128)).abs();
                    if digit > 5.into() {
                        if self.is_negative() {
                            Self((self.0 / divisor - I256::one()) * divisor)
                        } else {
                            Self((self.0 / divisor + I256::one()) * divisor)
                        }
                    } else {
                        Self(self.0 / divisor * divisor)
                    }
                }
            }
            RoundingMode::TowardsNearestAndHalfAwayFromZero => {
                if self.0 % divisor == I256::zero() {
                    self.clone()
                } else {
                    let digit =
                        (self.0 / (divisor / I256::from(10i128)) % I256::from(10i128)).abs();
                    if digit < 5.into() {
                        Self(self.0 / divisor * divisor)
                    } else {
                        if self.is_negative() {
                            Self((self.0 / divisor - I256::one()) * divisor)
                        } else {
                            Self((self.0 / divisor + I256::one()) * divisor)
                        }
                    }
                }
            }
        }
    }

    /// Calculates power usingexponentiation by squaring".
    pub fn powi(&self, exp: i64) -> Self {
        let one = Self::ONE.0;
        let base = self.0;
        let div = |x: i64, y: i64| x.checked_div(y).expect("Overflow");
        let sub = |x: i64, y: i64| x.checked_sub(y).expect("Overflow");
        let mul = |x: i64, y: i64| x.checked_mul(y).expect("Overflow");

        if exp < 0 {
            return Decimal(&one * &one / base).powi(mul(exp, -1));
        }
        if exp == 0 {
            return Self::ONE;
        }
        if exp % 2 == 0 {
            return Decimal(&base * &base / &one).powi(div(exp, 2));
        } else {
            return Decimal(
                &base * Decimal(&base * &base / &one).powi(div(sub(exp, 1), 2)).0 / &one,
            );
        }
    }
}

macro_rules! from_int {
    ($type:ident) => {
        impl From<$type> for Decimal {
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

impl From<&str> for Decimal {
    fn from(val: &str) -> Self {
        Self::from_str(&val).unwrap()
    }
}

impl From<String> for Decimal {
    fn from(val: String) -> Self {
        Self::from_str(&val).unwrap()
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

impl<T: TryInto<Decimal>> Add<T> for Decimal
where
    <T as TryInto<Decimal>>::Error: fmt::Debug,
{
    type Output = Decimal;

    fn add(self, other: T) -> Self::Output {
        let a = self.0;
        let b_dec: Decimal = other.try_into().expect("Overflow");
        let b: I256 = b_dec.0;
        let c = a + b;
        Decimal(c)
    }
}

impl<T: TryInto<Decimal>> Sub<T> for Decimal
where
    <T as TryInto<Decimal>>::Error: fmt::Debug,
{
    type Output = Decimal;

    fn sub(self, other: T) -> Self::Output {
        let a = self.0;
        let b_dec: Decimal = other.try_into().expect("Overflow");
        let b: I256 = b_dec.0;
        let c: I256 = a - b;
        Decimal(c)
    }
}

impl<T: TryInto<Decimal>> Mul<T> for Decimal
where
    <T as TryInto<Decimal>>::Error: fmt::Debug,
{
    type Output = Decimal;

    fn mul(self, other: T) -> Self::Output {
        let a = self.0;
        let b_dec: Decimal = other.try_into().expect("Overflow");
        let b: I256 = b_dec.0;
        let c: I256 = a * b / Self::ONE.0;
        Decimal(c)
    }
}

impl<T: TryInto<Decimal>> Div<T> for Decimal
where
    <T as TryInto<Decimal>>::Error: fmt::Debug,
{
    type Output = Decimal;

    fn div(self, other: T) -> Self::Output {
        let a = self.0;
        let b_dec: Decimal = other.try_into().expect("Overflow");
        let b: I256 = b_dec.0;
        let c: I256 = a * Self::ONE.0 / b;
        Decimal(c)
    }
}

impl Neg for Decimal {
    type Output = Decimal;

    fn neg(self) -> Self::Output {
        Decimal(-self.0)
    }
}

impl<T: TryInto<Decimal>> AddAssign<T> for Decimal
where
    <T as TryInto<Decimal>>::Error: fmt::Debug,
{
    fn add_assign(&mut self, other: T) {
        let other: Decimal = other.try_into().expect("Overflow");
        self.0 += other.0;
    }
}

impl<T: TryInto<Decimal>> SubAssign<T> for Decimal
where
    <T as TryInto<Decimal>>::Error: fmt::Debug,
{
    fn sub_assign(&mut self, other: T) {
        let other: Decimal = other.try_into().expect("Overflow");
        self.0 -= other.0;
    }
}

impl<T: TryInto<Decimal>> MulAssign<T> for Decimal
where
    <T as TryInto<Decimal>>::Error: fmt::Debug,
{
    fn mul_assign(&mut self, other: T) {
        let other: Decimal = other.try_into().expect("Overflow");
        self.0 *= other.0;
    }
}

impl<T: TryInto<Decimal>> DivAssign<T> for Decimal
where
    <T as TryInto<Decimal>>::Error: fmt::Debug,
{
    fn div_assign(&mut self, other: T) {
        let other: Decimal = other.try_into().expect("Overflow");
        self.0 /= other.0;
    }
}

//========
// binary
//========

impl TryFrom<&[u8]> for Decimal {
    type Error = ParseDecimalError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        if slice.len() == Self::BITS / 8 {
            match I256::try_from(slice) {
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

scrypto_type!(Decimal, ScryptoType::Decimal, Vec::new());

//======
// text
//======

impl FromStr for Decimal {
    type Err = ParseDecimalError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut sign = I256::from(1u8);
        let mut value = I256::from(0u8);

        let chars: Vec<char> = s.chars().collect();
        let mut p = 0;

        // read sign
        if chars[p] == '-' {
            sign = I256::from(-1i8);
            p += 1;
        }

        // read integral
        while p < chars.len() && chars[p] != '.' {
            let digit = read_digitdecimal(chars[p]);
            match digit {
                Ok(dig) => value = value * I256::from(10u8) + I256::from(dig) * sign,
                Err(e) => return Err(e),
            }
            p += 1;
        }

        // read radix point
        if p < chars.len() {
            read_dotdecimal(chars[p])?;
            p += 1;
        }

        // read fraction
        for _ in 0..Self::SCALE {
            if p < chars.len() {
                let digit = read_digitdecimal(chars[p]);
                match digit {
                    Ok(dig) => value = value * I256::from(10u8) + I256::from(dig) * sign,
                    Err(e) => return Err(e),
                }
                p += 1;
            } else {
                value *= I256::from(10u8);
            }
        }

        if p < chars.len() {
            Err(ParseDecimalError::UnsupportedDecimalPlace)
        } else {
            Ok(Self(value))
        }
    }
}

impl fmt::Display for Decimal {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let mut a = self.0;
        let mut buf = String::new();

        let mut trailing_zeros = true;
        for _ in 0..Self::SCALE {
            let m: I256 = a % I256::from(10u8);
            if m != 0.into() || !trailing_zeros {
                trailing_zeros = false;
                buf.push(char::from_digit(m.abs().to_u32().expect("Overflow"), 10).unwrap())
            }
            a /= I256::from(10u8);
        }

        if !buf.is_empty() {
            buf.push('.');
        }

        if a == 0.into() {
            buf.push('0')
        } else {
            while a != 0.into() {
                let m: I256 = a % I256::from(10u8);
                buf.push(char::from_digit(m.abs().to_u32().expect("Overflow"), 10).unwrap());
                a /= I256::from(10u8);
            }
        }

        write!(
            f,
            "{}{}",
            if self.is_negative() { "-" } else { "" },
            buf.chars().rev().collect::<String>()
        )
    }
}

impl fmt::Debug for Decimal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}
fn read_digitdecimal(c: char) -> Result<U8, ParseDecimalError> {
    let n = U8::from(c as u8);
    if n >= U8(48u8) && n <= U8(48u8 + 9u8) {
        Ok(n - U8(48u8))
    } else {
        Err(ParseDecimalError::InvalidChar(c))
    }
}

fn read_dotdecimal(c: char) -> Result<(), ParseDecimalError> {
    if c == '.' {
        Ok(())
    } else {
        Err(ParseDecimalError::InvalidChar(c))
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

macro_rules! from_integer {
    ($($t:ident),*) => {
        $(
            impl From<$t> for Decimal {
                fn from(val: $t) -> Self {
                    Self(I256::from(val) * Self::ONE.0)
                }
            }
        )*
    };
}

from_integer!(U8, U16, U32, U64, U128);
from_integer!(I8, I16, I32, I64, I128);

macro_rules! try_from_integer {
    ($($t:ident),*) => {
        paste!{
            $(
                impl TryFrom<$t> for Decimal {
                    type Error = ParseDecimalError;

                    fn try_from(val: $t) -> Result<Self, Self::Error> {
                        Ok(Self(I256::try_from(val).map_err(|_| ParseDecimalError::Overflow).unwrap() * Self::ONE.0))
                    }
                }
            )*
        }
    };
}

try_from_integer!(U256, U384, U512, I256, I384, I512);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dec;
    use sbor::rust::vec;

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
            "57896044618658097711785492504343953926634992332820282019728.792003956564819967"
        );
        assert_eq!(Decimal::MIN.is_negative(), true);
        assert_eq!(
            Decimal::MIN.to_string(),
            "-57896044618658097711785492504343953926634992332820282019728.792003956564819968"
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
            Decimal::from_str(
                "57896044618658097711785492504343953926634992332820282019728.792003956564819967"
            )
            .unwrap(),
            Decimal::MAX,
        );
        assert_eq!(
            Decimal::from_str(
                "-57896044618658097711785492504343953926634992332820282019728.792003956564819968"
            )
            .unwrap(),
            Decimal::MIN,
        );
    }

    #[test]
    fn test_add_decimal() {
        let a = Decimal::from(5u32);
        let b = Decimal::from(7u32);
        assert_eq!((a + b).to_string(), "12");
    }

    #[test]
    #[should_panic(expected = "Overflow")]
    fn test_add_overflow_decimal() {
        let _ = Decimal::MAX + 1;
    }

    #[test]
    fn test_sub_decimal() {
        let a = Decimal::from(5u32);
        let b = Decimal::from(7u32);
        assert_eq!((a - b).to_string(), "-2");
        assert_eq!((b - a).to_string(), "2");
    }

    #[test]
    #[should_panic(expected = "Overflow")]
    fn test_sub_overflow_decimal() {
        let _ = Decimal::MIN - 1;
    }

    #[test]
    fn test_mul_decimal() {
        let a = Decimal::from(5u32);
        let b = Decimal::from(7u32);
        assert_eq!((a * b).to_string(), "35");
        let a = Decimal::from_str("1000000000").unwrap();
        let b = Decimal::from_str("1000000000").unwrap();
        assert_eq!((a * b).to_string(), "1000000000000000000");
    }

    #[test]
    #[should_panic(expected = "Overflow")]
    fn test_mul_overflow_by_small_decimal() {
        let _ = Decimal::MAX * dec!("1.000000000000000001");
    }

    #[test]
    #[should_panic(expected = "Overflow")]
    fn test_mul_overflow_by_a_lot_decimal() {
        let _ = Decimal::MAX * dec!("1.1");
    }

    #[test]
    #[should_panic(expected = "Overflow")]
    fn test_mul_neg_overflow_decimal() {
        let _ = (-Decimal::MAX) * dec!("-1.000000000000000001");
    }

    #[test]
    #[should_panic]
    fn test_div_by_zero_decimal() {
        let a = Decimal::from(5u32);
        let b = Decimal::from(0u32);
        assert_eq!((a / b).to_string(), "0");
    }

    #[test]
    #[should_panic]
    fn test_powi_exp_overflow_decimal() {
        let a = Decimal::from(5u32);
        let b = i64::MIN;
        assert_eq!(a.powi(b).to_string(), "0");
    }

    #[test]
    fn test_1_powi_max_decimal() {
        let a = Decimal::from(1u32);
        let b = i64::MAX;
        assert_eq!(a.powi(b).to_string(), "1");
    }

    #[test]
    fn test_1_powi_min_decimal() {
        let a = Decimal::from(1u32);
        let b = i64::MAX - 1;
        assert_eq!(a.powi(b).to_string(), "1");
    }

    #[test]
    fn test_div_decimal() {
        let a = Decimal::from(5u32);
        let b = Decimal::from(7u32);
        assert_eq!((a / b).to_string(), "0.714285714285714285");
        assert_eq!((b / a).to_string(), "1.4");
    }

    #[test]
    fn test_div_negative_decimal() {
        let a = Decimal::from(-42);
        let b = Decimal::from(2);
        assert_eq!((a / b).to_string(), "-21");
    }

    #[test]
    fn test_0_pow_0_decimal() {
        let a = dec!("0");
        assert_eq!((a.powi(0)).to_string(), "1");
    }

    #[test]
    fn test_0_powi_1_decimal() {
        let a = dec!("0");
        assert_eq!((a.powi(1)).to_string(), "0");
    }

    #[test]
    fn test_0_powi_10_decimal() {
        let a = dec!("0");
        assert_eq!((a.powi(10)).to_string(), "0");
    }

    #[test]
    fn test_1_powi_0_decimal() {
        let a = dec!("1");
        assert_eq!((a.powi(0)).to_string(), "1");
    }

    #[test]
    fn test_1_powi_1_decimal() {
        let a = dec!("1");
        assert_eq!((a.powi(1)).to_string(), "1");
    }

    #[test]
    fn test_1_powi_10_decimal() {
        let a = dec!("1");
        assert_eq!((a.powi(10)).to_string(), "1");
    }

    #[test]
    fn test_2_powi_0_decimal() {
        let a = dec!("2");
        assert_eq!((a.powi(0)).to_string(), "1");
    }

    #[test]
    fn test_2_powi_3724_decimal() {
        let a = dec!("1.000234891009084238");
        assert_eq!((a.powi(3724)).to_string(), "2.397991232254669619");
    }

    #[test]
    fn test_2_powi_2_decimal() {
        let a = dec!("2");
        assert_eq!((a.powi(2)).to_string(), "4");
    }

    #[test]
    fn test_2_powi_3_decimal() {
        let a = dec!("2");
        assert_eq!((a.powi(3)).to_string(), "8");
    }

    #[test]
    fn test_10_powi_3_decimal() {
        let a = dec!("10");
        assert_eq!((a.powi(3)).to_string(), "1000");
    }

    #[test]
    fn test_5_powi_2_decimal() {
        let a = dec!("5");
        assert_eq!((a.powi(2)).to_string(), "25");
    }

    #[test]
    fn test_5_powi_minus2_decimal() {
        let a = dec!("5");
        assert_eq!((a.powi(-2)).to_string(), "0.04");
    }

    #[test]
    fn test_10_powi_minus3_decimal() {
        let a = dec!("10");
        assert_eq!((a.powi(-3)).to_string(), "0.001");
    }

    #[test]
    fn test_minus10_powi_minus3_decimal() {
        let a = dec!("-10");
        assert_eq!((a.powi(-3)).to_string(), "-0.001");
    }

    #[test]
    fn test_minus10_powi_minus2_decimal() {
        let a = dec!("-10");
        assert_eq!((a.powi(-2)).to_string(), "0.01");
    }

    #[test]
    fn test_minus05_powi_minus2_decimal() {
        let a = dec!("-0.5");
        assert_eq!((a.powi(-2)).to_string(), "4");
    }
    #[test]
    fn test_minus05_powi_minus3_decimal() {
        let a = dec!("-0.5");
        assert_eq!((a.powi(-3)).to_string(), "-8");
    }

    #[test]
    fn test_10_powi_15_decimal() {
        let a = dec!(10i128);
        assert_eq!(a.powi(15).to_string(), "1000000000000000");
    }

    #[test]
    #[should_panic]
    fn test_10_powi_16_decimal() {
        let a = Decimal(10i128.into());
        assert_eq!(a.powi(16).to_string(), "1000000000000000000000");
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
        assert_eq!(dec!("1").to_string(), "1");
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
            "57896044618658097711785492504343953926634992332820282019728"
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
            "-57896044618658097711785492504343953926634992332820282019728"
        );
    }

    #[test]
    #[should_panic]
    fn test_ceiling_overflow_decimal() {
        Decimal::MAX.ceiling();
    }

    #[test]
    fn test_round_towards_zero_decimal() {
        let mode = RoundingMode::TowardsZero;
        assert_eq!(dec!("1.2").round(0, mode).to_string(), "1");
        assert_eq!(dec!("1.0").round(0, mode).to_string(), "1");
        assert_eq!(dec!("0.9").round(0, mode).to_string(), "0");
        assert_eq!(dec!("0").round(0, mode).to_string(), "0");
        assert_eq!(dec!("-0.1").round(0, mode).to_string(), "0");
        assert_eq!(dec!("-1").round(0, mode).to_string(), "-1");
        assert_eq!(dec!("-5.2").round(0, mode).to_string(), "-5");
    }

    #[test]
    fn test_round_away_from_zero_decimal() {
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
    fn test_round_towards_nearest_and_half_towards_zero_decimal() {
        let mode = RoundingMode::TowardsNearestAndHalfTowardsZero;
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
    fn test_round_towards_nearest_and_half_away_from_zero_decimal() {
        let mode = RoundingMode::TowardsNearestAndHalfAwayFromZero;
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
    fn test_various_decimal_places_decimal() {
        let mode = RoundingMode::TowardsNearestAndHalfAwayFromZero;
        let num = dec!("-2.555555555555555555");
        assert_eq!(num.round(0, mode).to_string(), "-3");
        assert_eq!(num.round(1, mode).to_string(), "-2.6");
        assert_eq!(num.round(2, mode).to_string(), "-2.56");
        assert_eq!(num.round(17, mode).to_string(), "-2.55555555555555556");
        assert_eq!(num.round(18, mode).to_string(), "-2.555555555555555555");
    }

    #[test]
    fn test_sum_decimal() {
        let decimals = vec![dec!("1"), dec!("2"), dec!("3")];
        // two syntax
        let sum1: Decimal = decimals.iter().copied().sum();
        let sum2: Decimal = decimals.into_iter().sum();
        assert_eq!(sum1, dec!("6"));
        assert_eq!(sum2, dec!("6"));
    }

    #[test]
    fn test_encode_decimal_type_decimal() {
        let mut bytes = Vec::with_capacity(512);
        let mut enc = Encoder::with_static_info(&mut bytes);
        Decimal::encode_type_id(&mut enc);
        assert_eq!(bytes, vec![Decimal::type_id()]);
    }

    #[test]
    fn test_encode_decimal_value_decimal() {
        let dec = dec!("0");
        let mut bytes = Vec::with_capacity(512);
        let mut enc = Encoder::with_static_info(&mut bytes);
        Decimal::encode_type_id(&mut enc);
        dec.encode_value(&mut enc);
        assert_eq!(bytes, {
            let mut a = [0; 37];
            a[0] = Decimal::type_id();
            a[1] = 32;
            a
        });
    }

    #[test]
    fn test_decode_decimal_type_decimal() {
        let mut bytes = Vec::with_capacity(512);
        let mut enc = Encoder::with_static_info(&mut bytes);
        Decimal::encode_type_id(&mut enc);
        let mut decoder = Decoder::new(&bytes, true);
        let typ = decoder.read_type().unwrap();
        assert_eq!(typ, Decimal::type_id());
    }

    #[test]
    fn test_decode_decimal_value_decimal() {
        let dec = dec!("1.23456789");
        let mut bytes = Vec::with_capacity(512);
        let mut enc = Encoder::with_static_info(&mut bytes);
        Decimal::encode_type_id(&mut enc);
        dec.encode_value(&mut enc);
        let mut decoder = Decoder::new(&bytes, true);
        Decimal::check_type_id(&mut decoder).unwrap();
        let val = Decimal::decode_value(&mut decoder).unwrap();
        assert_eq!(val, dec!("1.23456789"));
    }

    #[test]
    fn test_from_str_decimal() {
        let dec = Decimal::from_str("5.0").unwrap();
        assert_eq!(dec.to_string(), "5");
    }

    #[test]
    fn test_from_str_failure_decimal() {
        let dec = Decimal::from_str("non_decimal_value");
        assert_eq!(dec, Err(ParseDecimalError::InvalidChar('n')));
    }
}
