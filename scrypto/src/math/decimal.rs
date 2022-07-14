use core::ops::*;
use num_bigint::BigInt;
use num_traits::{Pow, ToPrimitive, Zero};
use sbor::rust::convert::{TryFrom, TryInto};
use sbor::rust::fmt;
use sbor::rust::fmt::{Display, Formatter};
use sbor::rust::iter;
use sbor::rust::str::FromStr;
use sbor::rust::string::String;
use sbor::rust::string::ToString;
use sbor::rust::vec::Vec;
use sbor::*;

use crate::abi::*;
use crate::math::{I256, I384, U8};
use crate::misc::*;
use paste::paste;

macro_rules! decimals {
    ($(($dec:ident, $wrapped:ident, $scale:literal, $dec_macro:ident)),*) => {
        $(
            paste! {
                /// `$dec` represents a 128 bit representation of a fixed-scale decimal number.
                ///
                /// The finite set of values are of the form `m / 10^18`, where `m` is
                /// an integer such that `-2^127 <= m < 2^127`.
                ///
                /// Unless otherwise specified, all operations will panic if underflow/overflow.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
                pub struct $dec(pub $wrapped);

                impl Default for $dec {
                    fn default() -> Self {
                        Self::zero()
                    }
                }

                impl iter::Sum for $dec {
                    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
                        let mut sum = $dec::zero();
                        iter.for_each(|d| sum += d);
                        sum
                    }
                }

                impl $dec {
                    /// The min value of `$dec`.
                    pub const MIN: Self = Self(<$wrapped>::MIN);

                    /// The max value of `$dec`.
                    pub const MAX: Self = Self(<$wrapped>::MAX);

                    /// The bit length of number storing `$dec`.
                    pub const BITS: usize = <$wrapped>::BITS as usize;

                    /// The fixed scale used by `$dec`.
                    pub const SCALE: u32 = $scale;

                    pub const ZERO: Self = Self(<$wrapped>::zero());

                    pub const ONE: Self = Self(<$wrapped>::from(10u8).pow(Self::SCALE));

                    /// Returns `$dec` of 0.
                    pub fn zero() -> Self {
                        Self::ZERO
                    }

                    /// Returns `$dec` of 1.
                    pub fn one() -> Self {
                        Self::ONE
                    }

                    /// Whether this decimal is zero.
                    pub fn is_zero(&self) -> bool {
                        self.0 == Zero::zero()
                    }

                    /// Whether this decimal is positive.
                    pub fn is_positive(&self) -> bool {
                        self.0 > Zero::zero()
                    }

                    /// Whether this decimal is negative.
                    pub fn is_negative(&self) -> bool {
                        self.0 < Zero::zero()
                    }

                    /// Returns the absolute value.
                    pub fn abs(&self) -> $dec {
                        $dec(self.0.abs())
                    }

                    /// Returns the largest integer that is equal to or less than this number.
                    pub fn floor(&self) -> Self {
                        self.round(0, RoundingMode::TowardsNegativeInfinity)
                    }

                    /// Returns the smallest integer that is equal to or greater than this number.
                    pub fn ceiling(&self) -> Self {
                        self.round(0, RoundingMode::TowardsPositiveInfinity)
                    }

                    pub fn round(&self, decimal_places: u8, mode: RoundingMode) -> Self {
                        assert!(decimal_places <= Self::SCALE.try_into().unwrap());

                        let divisor = <$wrapped>::from(10i8).pow(18 - decimal_places as u32);
                        match mode {
                            RoundingMode::TowardsPositiveInfinity => {
                                if self.0 % divisor == Zero::zero() {
                                    self.clone()
                                } else if self.is_negative() {
                                    Self(self.0 / divisor * divisor)
                                } else {
                                    Self((self.0 / divisor + 1) * divisor)
                                }
                            }
                            RoundingMode::TowardsNegativeInfinity => {
                                if self.0 % divisor == Zero::zero() {
                                    self.clone()
                                } else if self.is_negative() {
                                    Self((self.0 / divisor - 1) * divisor)
                                } else {
                                    Self(self.0 / divisor * divisor)
                                }
                            }
                            RoundingMode::TowardsZero => {
                                if self.0 % divisor == Zero::zero() {
                                    self.clone()
                                } else {
                                    Self(self.0 / divisor * divisor)
                                }
                            }
                            RoundingMode::AwayFromZero => {
                                if self.0 % divisor == Zero::zero() {
                                    self.clone()
                                } else if self.is_negative() {
                                    Self((self.0 / divisor - 1) * divisor)
                                } else {
                                    Self((self.0 / divisor + 1) * divisor)
                                }
                            }
                            RoundingMode::TowardsNearestAndHalfTowardsZero => {
                                if self.0 % divisor == Zero::zero() {
                                    self.clone()
                                } else {
                                    let digit = (self.0 / (divisor / 10) % 10).abs();
                                    if digit > 5 {
                                        if self.is_negative() {
                                            Self((self.0 / divisor - 1) * divisor)
                                        } else {
                                            Self((self.0 / divisor + 1) * divisor)
                                        }
                                    } else {
                                        Self(self.0 / divisor * divisor)
                                    }
                                }
                            }
                            RoundingMode::TowardsNearestAndHalfAwayFromZero => {
                                if self.0 % divisor == Zero::zero() {
                                    self.clone()
                                } else {
                                    let digit = (self.0 / (divisor / 10) % 10).abs();
                                    if digit < 5 {
                                        Self(self.0 / divisor * divisor)
                                    } else {
                                        if self.is_negative() {
                                            Self((self.0 / divisor - 1) * divisor)
                                        } else {
                                            Self((self.0 / divisor + 1) * divisor)
                                        }
                                    }
                                }
                            }
                        }
                    }

                    /// Calculates power using "exponentiation by squaring".
                    pub fn powi(&self, exp: i32) -> Self {
                        let one = BigInt::from(Self::ONE.0);
                        let base = BigInt::from(self.0);
                        let to_dec = |x: BigInt| $dec(<$wrapped>::try_from(x).expect("overflow"));
                        let div = |x: i32, y: i32| x.checked_div(y).expect("overflow");
                        let sub = |x: i32, y: i32| x.checked_sub(y).expect("overflow");
                        let mul = |x: i32, y: i32| x.checked_mul(y).expect("overflow");

                        if exp < 0 {
                            return to_dec(&one * &one / base).powi(mul(exp, -1));
                        }
                        if exp == 0 {
                            return Self::ONE;
                        }
                        if exp % 2 == 0 {
                            return to_dec(&base * &base / &one).powi(div(exp, 2));
                        } else {
                            return to_dec(&base * to_dec(&base * &base / &one).powi(div(sub(exp, 1), 2)).0 / &one);
                        }
                    }
                }

                macro_rules! from_int {
                    ($type:ident) => {
                        impl From<$type> for $dec {
                            fn from(val: $type) -> Self {
                                Self(<$wrapped>::from(val) * Self::ONE.0)
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

                impl From<&str> for $dec {
                    fn from(val: &str) -> Self {
                        Self::from_str(&val).unwrap()
                    }
                }

                impl From<String> for $dec {
                    fn from(val: String) -> Self {
                        Self::from_str(&val).unwrap()
                    }
                }

                impl From<bool> for $dec {
                    fn from(val: bool) -> Self {
                        if val {
                            Self::from(1u8)
                        } else {
                            Self::from(0u8)
                        }
                    }
                }

                /// Creates a `$dec` from literals.
                ///
                /// # Example
                /// ```ignore
                /// use scrypto::prelude::*;
                ///
                /// let a = dec!(1);
                /// let b = dec!("1.1");
                /// ```
                #[macro_export]
                macro_rules! $dec_macro {
                    ($x:literal) => {
                        ::scrypto::math::$dec::from($x)
                    };

                    ($base:literal, $shift:literal) => {
                        // Base can be any type that converts into a $dec, and shift must support
                        // comparison and `-` unary operation, enforced by rustc.
                        {
                            let base = ::scrypto::math::$dec::from($base);
                            if $shift >= 0 {
                                base * <$wrapped>::from(10u8).pow(u32::try_from($shift).expect("Shift overflow"))
                            } else {
                                base / <$wrapped>::from(10u8).pow(u32::try_from(-$shift).expect("Shift overflow"))
                            }
                        }
                    };
                }

                impl<T: TryInto<$dec> + Display> Add<T> for $dec {
                    type Output = $dec;

                    fn add(self, other: T) -> Self::Output {
                        let a = self.0;
                        let b: $wrapped = other.try_into().expect("overflow").0;
                        let c: $wrapped = a + b;
                        Self(c)
                    }
                }

                impl<T: Into<$dec>> Sub<T> for $dec {
                    type Output = $dec;

                    fn sub(self, other: T) -> Self::Output {
                        let a = self.0;
                        let b: $wrapped = other.try_into().expect("overflow");
                        let c: $wrapped = a - b;
                        Self(c)
                    }
                }

                impl<T: Into<$dec>> Mul<T> for $dec {
                    type Output = $dec;

                    fn mul(self, other: T) -> Self::Output {
                        let a = self.0;
                        let b: $wrapped = other.try_into().expect("overflow");
                        let c: $wrapped = a * b;
                        Self(c)
                    }
                }

                impl<T: Into<$dec>> Div<T> for $dec {
                    type Output = $dec;

                    fn div(self, other: T) -> Self::Output {
                        let a = self.0;
                        let b: $wrapped = other.into().0;
                        let c: $wrapped = a / b;
                        Self(c)
                    }
                }

                impl Neg for $dec {
                    type Output = $dec;

                    fn neg(self) -> Self::Output {
                        $dec(-self.0)
                    }
                }

                impl<T: TryInto<$dec>> AddAssign<T> for $dec {
                    fn add_assign(&mut self, other: T) {
                        self.0 += other.try_into().unwrap().0;
                    }
                }

                impl<T: TryInto<$dec>> SubAssign<T> for $dec {
                    fn sub_assign(&mut self, other: T) {
                        self.0 -= other.try_into().unwrap().0;
                    }
                }

                impl<T: TryInto<$dec>> MulAssign<T> for $dec {
                    fn mul_assign(&mut self, other: T) {
                        self.0 *= other.try_into().unwrap().0;
                    }
                }

                impl<T: TryInto<$dec>> DivAssign<T> for $dec {
                    fn div_assign(&mut self, other: T) {
                        self.0 /= other.try_into().unwrap().0;
                    }
                }

                //========
                // binary
                //========

                impl TryFrom<&[u8]> for $dec {
                    type Error = ParseDecimalError;

                    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
                        if slice.len() == 16 {
                            Ok(Self(<$wrapped>::try_from(slice).unwrap()))
                        } else {
                            Err(ParseDecimalError::InvalidLength(slice.len()))
                        }
                    }
                }

                impl $dec {
                    pub fn to_vec(&self) -> Vec<u8> {
                        self.0.to_le_bytes().to_vec()
                    }
                }

                scrypto_type!($dec, ScryptoType::$dec, Vec::new());

                //======
                // text
                //======

                impl FromStr for $dec {
                    type Err = ParseDecimalError;

                    fn from_str(s: &str) -> Result<Self, Self::Err> {
                        let mut sign = <$wrapped>::from(1u8);
                        let mut value = <$wrapped>::from(0u8);

                        let chars: Vec<char> = s.chars().collect();
                        let mut p = 0;

                        // read sign
                        if chars[p] == '-' {
                            sign = <$wrapped>::from(-1i8);
                            p += 1;
                        }

                        // read integral
                        while p < chars.len() && chars[p] != '.' {
                            value = value * 10 + read_digit(chars[p])? * sign;
                            p += 1;
                        }

                        // read radix point
                        if p < chars.len() {
                            read_dot(chars[p])?;
                            p += 1;
                        }

                        // read fraction
                        for _ in 0..Self::SCALE {
                            if p < chars.len() {
                                value = value * 10 + read_digit(chars[p])? * sign;
                                p += 1;
                            } else {
                                value *= 10;
                            }
                        }

                        if p < chars.len() {
                            Err(ParseDecimalError::UnsupportedDecimalPlace)
                        } else {
                            Ok(Self(value))
                        }
                    }
                }

                impl fmt::Display for $dec {
                    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
                        let mut a = self.0;
                        let mut buf = String::new();

                        let mut trailing_zeros = true;
                        for _ in 0..Self::SCALE {
                            let m: $wrapped = a % 10;
                            if m != 0.into() || !trailing_zeros {
                                trailing_zeros = false;
                                buf.push(char::from_digit(m.abs().to_u32().expect("overflow"), 10).unwrap())
                            }
                            a /= 10;
                        }

                        if !buf.is_empty() {
                            buf.push('.');
                        }

                        if a == Zero::zero() {
                            buf.push('0')
                        } else {
                            while a != Zero::zero() {
                                let m: $wrapped = a % 10;
                                buf.push(char::from_digit(m.abs().to_u32().expect("overflow"), 10).unwrap());
                                a /= 10
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

                impl fmt::Debug for $dec {
                    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                        write!(f, "{}", self.to_string())
                    }
                }

            }
        )*
    }
}

decimals! {
    (D256F18, I256, 18, d18),
    (D384F45, I384, 45, d45)
}

pub type Decimal = D256F18;
pub type D18 = Decimal;
pub type D45 = D384F45;
#[macro_export]
macro_rules! dec {
    ($tt:tt) => {
        d18! {$tt}
    };
}

fn read_digit(c: char) -> Result<U8, ParseDecimalError> {
    let n = U8::from(c as u8);
    if n >= 48 && n <= 48 + 9 {
        Ok(n - 48)
    } else {
        Err(ParseDecimalError::InvalidChar(c))
    }
}

fn read_dot(c: char) -> Result<(), ParseDecimalError> {
    if c == '.' {
        Ok(())
    } else {
        Err(ParseDecimalError::InvalidChar(c))
    }
}

/// Defines how rounding should be done.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RoundingMode {
    /// Rounds towards positive infinity, e.g. `3.1 -> 4`, `-3.1 -> -3`.
    TowardsPositiveInfinity,
    /// Rounds towards negative infinity, e.g. `3.1 -> 3`, `-3.1 -> -4`.
    TowardsNegativeInfinity,
    /// Rounds towards zero, e.g. `3.1 -> 3`, `-3.1 -> -3`.
    TowardsZero,
    /// Rounds away from zero, e.g. `3.1 -> 4`, `-3.1 -> -4`.
    AwayFromZero,
    /// Rounds to the nearest and when a number is halfway between two others, it's rounded towards zero, e.g. `3.5 -> 3`, `-3.5 -> -3`.
    TowardsNearestAndHalfTowardsZero,
    /// Rounds to the nearest and when a number is halfway between two others, it's rounded away zero, e.g. `3.5 -> 4`, `-3.5 -> -4`.
    TowardsNearestAndHalfAwayFromZero,
}

//========
// error
//========

/// Represents an error when parsing $dec from hex string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseDecimalError {
    InvalidDecimal(String),
    InvalidChar(char),
    UnsupportedDecimalPlace,
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

/*
 * FIXME: remove commenting above
#[cfg(test)]
mod tests {
    use super::*;
    use sbor::rust::vec;

    #[test]
    fn test_format() {
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
            "170141183460469231731.687303715884105727"
        );
        assert_eq!(
            Decimal::MIN.to_string(),
            "-170141183460469231731.687303715884105728"
        );
    }

    #[test]
    fn test_parse() {
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
            Decimal::from_str("170141183460469231731.687303715884105727").unwrap(),
            Decimal::MAX,
        );
        assert_eq!(
            Decimal::from_str("-170141183460469231731.687303715884105728").unwrap(),
            Decimal::MIN,
        );
    }

    #[test]
    fn test_add() {
        let a = Decimal::from(5u32);
        let b = Decimal::from(7u32);
        assert_eq!((a + b).to_string(), "12");
    }

    #[test]
    #[should_panic(expected = "overflow")]
    fn test_add_oveflow() {
        let _ = Decimal::MAX + 1;
    }

    #[test]
    fn test_sub() {
        let a = Decimal::from(5u32);
        let b = Decimal::from(7u32);
        assert_eq!((a - b).to_string(), "-2");
        assert_eq!((b - a).to_string(), "2");
    }

    #[test]
    #[should_panic(expected = "overflow")]
    fn test_sub_overflow() {
        let _ = Decimal::MIN - 1;
    }

    #[test]
    fn test_mul() {
        let a = Decimal::from(5u32);
        let b = Decimal::from(7u32);
        assert_eq!((a * b).to_string(), "35");
        let a = Decimal::from_str("1000000000").unwrap();
        let b = Decimal::from_str("1000000000").unwrap();
        assert_eq!((a * b).to_string(), "1000000000000000000");
    }

    #[test]
    fn test_mul_no_overflow() {
        // make sure multiplication DOES NOT overflow
        // because of bad implementation
        assert_eq!(Decimal::MAX * 1i8, Decimal::MAX);
    }

    #[test]
    #[should_panic(expected = "overflow")]
    fn test_mul_overflow_by_small() {
        let _ = Decimal::MAX * dec!("1.000000000000000001");
    }

    #[test]
    #[should_panic(expected = "overflow")]
    fn test_mul_overflow_by_a_lot() {
        let _ = Decimal::MAX * dec!("1.1");
    }

    #[test]
    #[should_panic(expected = "overflow")]
    fn test_mul_neg_overflow() {
        let _ = (-Decimal::MAX) * dec!("-1.000000000000000001");
    }

    #[test]
    #[should_panic]
    fn test_div_by_zero() {
        let a = Decimal::from(5u32);
        let b = Decimal::from(0u32);
        assert_eq!((a / b).to_string(), "0");
    }

    #[test]
    #[should_panic]
    fn test_powi_exp_overflow() {
        let a = Decimal::from(5u32);
        let b = i32::MIN;
        assert_eq!(a.powi(b).to_string(), "0");
    }

    #[test]
    fn test_1_powi_max() {
        let a = Decimal::from(1u32);
        let b = i32::MAX;
        assert_eq!(a.powi(b).to_string(), "1");
    }

    #[test]
    fn test_1_powi_min() {
        let a = Decimal::from(1u32);
        let b = i32::MAX - 1;
        assert_eq!(a.powi(b).to_string(), "1");
    }

    #[test]
    fn test_div() {
        let a = Decimal::from(5u32);
        let b = Decimal::from(7u32);
        assert_eq!((a / b).to_string(), "0.714285714285714285");
        assert_eq!((b / a).to_string(), "1.4");
    }

    #[test]
    fn test_div_negative() {
        let a = Decimal::from(-42);
        let b = Decimal::from(2);
        assert_eq!((a / b).to_string(), "-21");
    }

    #[test]
    fn test_0_pow_0() {
        let a = dec!("0");
        assert_eq!((a.powi(0)).to_string(), "1");
    }

    #[test]
    fn test_0_powi_1() {
        let a = dec!("0");
        assert_eq!((a.powi(1)).to_string(), "0");
    }

    #[test]
    fn test_0_powi_10() {
        let a = dec!("0");
        assert_eq!((a.powi(10)).to_string(), "0");
    }

    #[test]
    fn test_1_powi_0() {
        let a = dec!("1");
        assert_eq!((a.powi(0)).to_string(), "1");
    }

    #[test]
    fn test_1_powi_1() {
        let a = dec!("1");
        assert_eq!((a.powi(1)).to_string(), "1");
    }

    #[test]
    fn test_1_powi_10() {
        let a = dec!("1");
        assert_eq!((a.powi(10)).to_string(), "1");
    }

    #[test]
    fn test_2_powi_0() {
        let a = dec!("2");
        assert_eq!((a.powi(0)).to_string(), "1");
    }

    #[test]
    fn test_2_powi_3724() {
        let a = dec!("1.000234891009084238");
        assert_eq!((a.powi(3724)).to_string(), "2.397991232254669619");
    }

    #[test]
    fn test_2_powi_2() {
        let a = dec!("2");
        assert_eq!((a.powi(2)).to_string(), "4");
    }

    #[test]
    fn test_2_powi_3() {
        let a = dec!("2");
        assert_eq!((a.powi(3)).to_string(), "8");
    }

    #[test]
    fn test_10_powi_3() {
        let a = dec!("10");
        assert_eq!((a.powi(3)).to_string(), "1000");
    }

    #[test]
    fn test_5_powi_2() {
        let a = dec!("5");
        assert_eq!((a.powi(2)).to_string(), "25");
    }

    #[test]
    fn test_5_powi_minus2() {
        let a = dec!("5");
        assert_eq!((a.powi(-2)).to_string(), "0.04");
    }

    #[test]
    fn test_10_powi_minus3() {
        let a = dec!("10");
        assert_eq!((a.powi(-3)).to_string(), "0.001");
    }

    #[test]
    fn test_minus10_powi_minus3() {
        let a = dec!("-10");
        assert_eq!((a.powi(-3)).to_string(), "-0.001");
    }

    #[test]
    fn test_minus10_powi_minus2() {
        let a = dec!("-10");
        assert_eq!((a.powi(-2)).to_string(), "0.01");
    }

    #[test]
    fn test_minus05_powi_minus2() {
        let a = dec!("-0.5");
        assert_eq!((a.powi(-2)).to_string(), "4");
    }
    #[test]
    fn test_minus05_powi_minus3() {
        let a = dec!("-0.5");
        assert_eq!((a.powi(-3)).to_string(), "-8");
    }

    #[test]
    fn test_10_powi_15() {
        let a = dec!(10i128);
        assert_eq!(a.powi(15).to_string(), "1000000000000000");
    }

    #[test]
    #[should_panic]
    fn test_10_powi_16() {
        let a = Decimal(10i128);
        assert_eq!(a.powi(16).to_string(), "1000000000000000000000");
    }

    #[test]
    fn test_one_and_zero() {
        assert_eq!(Decimal::one().to_string(), "1");
        assert_eq!(Decimal::zero().to_string(), "0");
    }

    #[test]
    fn test_dec_string_decimal() {
        assert_eq!(
            dec!("1.123456789012345678").to_string(),
            "1.123456789012345678"
        );
        assert_eq!(dec!("-5.6").to_string(), "-5.6");
    }

    #[test]
    fn test_dec_string() {
        assert_eq!(dec!("1").to_string(), "1");
        assert_eq!(dec!("0").to_string(), "0");
    }

    #[test]
    fn test_dec_int() {
        assert_eq!(dec!(1).to_string(), "1");
        assert_eq!(dec!(5).to_string(), "5");
    }

    #[test]
    fn test_dec_bool() {
        assert_eq!((dec!(false)).to_string(), "0");
    }

    #[test]
    fn test_dec_rational() {
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
    fn test_shift_overflow() {
        // u32::MAX + 1
        dec!(1, 4_294_967_296i128); // use explicit type to defer error to runtime
    }

    #[test]
    fn test_floor() {
        assert_eq!(Decimal::MAX.floor().to_string(), "170141183460469231731");
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
    fn test_floor_overflow() {
        Decimal::MIN.floor();
    }

    #[test]
    fn test_ceiling() {
        assert_eq!(dec!("1.2").ceiling().to_string(), "2");
        assert_eq!(dec!("1.0").ceiling().to_string(), "1");
        assert_eq!(dec!("0.9").ceiling().to_string(), "1");
        assert_eq!(dec!("0").ceiling().to_string(), "0");
        assert_eq!(dec!("-0.1").ceiling().to_string(), "0");
        assert_eq!(dec!("-1").ceiling().to_string(), "-1");
        assert_eq!(dec!("-5.2").ceiling().to_string(), "-5");
        assert_eq!(Decimal::MIN.ceiling().to_string(), "-170141183460469231731");
    }

    #[test]
    #[should_panic]
    fn test_ceiling_overflow() {
        Decimal::MAX.ceiling();
    }

    #[test]
    fn test_round_towards_zero() {
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
    fn test_round_away_from_zero() {
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
    fn test_round_towards_nearest_and_half_towards_zero() {
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
    fn test_round_towards_nearest_and_half_away_from_zero() {
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
    fn test_various_decimal_places() {
        let mode = RoundingMode::TowardsNearestAndHalfAwayFromZero;
        let num = dec!("-2.555555555555555555");
        assert_eq!(num.round(0, mode).to_string(), "-3");
        assert_eq!(num.round(1, mode).to_string(), "-2.6");
        assert_eq!(num.round(2, mode).to_string(), "-2.56");
        assert_eq!(num.round(17, mode).to_string(), "-2.55555555555555556");
        assert_eq!(num.round(18, mode).to_string(), "-2.555555555555555555");
    }

    #[test]
    fn test_sum() {
        let decimals = vec![dec!("1"), dec!("2"), dec!("3")];
        // two syntax
        let sum1: Decimal = decimals.iter().copied().sum();
        let sum2: Decimal = decimals.into_iter().sum();
        assert_eq!(sum1, dec!("6"));
        assert_eq!(sum2, dec!("6"));
    }
}
*/
