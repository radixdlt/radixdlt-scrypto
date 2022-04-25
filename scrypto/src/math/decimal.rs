use core::ops::*;
use num_bigint::BigInt;
use num_traits::{Signed, Zero};
use sbor::*;

use crate::misc::*;
use crate::rust::convert::TryFrom;
use crate::rust::fmt;
use crate::rust::str::FromStr;
use crate::rust::string::String;
use crate::rust::string::ToString;
use crate::rust::vec::Vec;
use crate::types::*;

/// `Decimal` represents a 128 bit representation of a fixed-scale decimal number.
///
/// The finite set of values are of the form `m / 10^18`, where `m` is
/// an integer such that `-2^127 <= m < 2^127`.
///
/// Unless otherwise specified, all operations will panic if underflow/overflow.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Decimal(pub i128);

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

impl Default for Decimal {
    fn default() -> Self {
        Self::zero()
    }
}

impl Decimal {
    /// The min value of `Decimal`.
    pub const MIN: Self = Self(i128::MIN);

    /// The max value of `Decimal`.
    pub const MAX: Self = Self(i128::MAX);

    /// The fixed scale used by `Decimal`.
    pub const SCALE: u32 = 18;

    pub const ZERO: Self = Self(0i128);

    pub const ONE: Self = Self(10i128.pow(Self::SCALE));

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
        self.0 == 0
    }

    /// Whether this decimal is positive.
    pub fn is_positive(&self) -> bool {
        self.0 > 0
    }

    /// Whether this decimal is negative.
    pub fn is_negative(&self) -> bool {
        self.0 < 0
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

    pub fn round(&self, decimal_places: u8, mode: RoundingMode) -> Self {
        assert!(decimal_places <= 18);

        let divisor = 10i128.pow(18 - decimal_places as u32);
        match mode {
            RoundingMode::TowardsPositiveInfinity => {
                if self.0 % divisor == 0 {
                    self.clone()
                } else if self.is_negative() {
                    Self(self.0 / divisor * divisor)
                } else {
                    Self((self.0 / divisor + 1) * divisor)
                }
            }
            RoundingMode::TowardsNegativeInfinity => {
                if self.0 % divisor == 0 {
                    self.clone()
                } else if self.is_negative() {
                    Self((self.0 / divisor - 1) * divisor)
                } else {
                    Self(self.0 / divisor * divisor)
                }
            }
            RoundingMode::TowardsZero => {
                if self.0 % divisor == 0 {
                    self.clone()
                } else {
                    Self(self.0 / divisor * divisor)
                }
            }
            RoundingMode::AwayFromZero => {
                if self.0 % divisor == 0 {
                    self.clone()
                } else if self.is_negative() {
                    Self((self.0 / divisor - 1) * divisor)
                } else {
                    Self((self.0 / divisor + 1) * divisor)
                }
            }
            RoundingMode::TowardsNearestAndHalfTowardsZero => {
                if self.0 % divisor == 0 {
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
                if self.0 % divisor == 0 {
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

    pub fn powi(&self, exp: i32) -> Self {
        let mut x: BigInt = self.0.clone().into();
        if x.is_negative() {
            panic!("powi is not supported for negative numbers");
        }
        let s = Self::SCALE;
        let b: BigInt = 10i128.pow(s).into();
        let mut n: BigInt = exp.abs().into();
        let mut z: BigInt;
        if x.clone() == BigInt::from(0i8) {
            if n.clone() == Zero::zero() {
                z = b.clone();
            } else {
                z = Zero::zero();
            }
        } else if x.clone() == b.clone() {
            z = b.clone();
        } else {
            if n.clone() % 2i8 == Zero::zero() {
                z = b.clone();
            } else {
                z = x.clone();
            }
            let half = b.clone() / 2i8;
            n /= 2i8;
            while n.clone() > Zero::zero() {
                let xx = x.clone() * x.clone();
                if xx.clone() / x.clone() != x.clone() {
                    panic!("pow overflow: square of {} too large", x);
                }
                let xx_round = xx.clone() + half.clone();
                if xx_round.clone() < xx.clone() {
                    panic!("pow overflow: sum of {} and {} too large", xx, half);
                }
                x = xx_round.clone() / b.clone();
                if n.clone() % 2i8 != Zero::zero() {
                    let zx = z.clone() * x.clone();
                    if x.clone() != Zero::zero() && zx.clone() / x.clone() != z.clone() {
                        panic!("pow overflow: product of {} and {} too large", z, x);
                    }
                    let zx_round = zx.clone() + half.clone();
                    if zx_round < zx.clone() {
                        panic!("pow overflow: sum of {} and {} too large", zx, half);
                    }
                    z = zx_round.clone() / b.clone();
                }
                n /= 2i8;
            }
        }   

        if exp.is_negative() {
            Self(big_int_to_i128(b.clone() * b / z))
        } else {
            Self(big_int_to_i128(z))
        }
    }
}

macro_rules! from_int {
    ($type:ident) => {
        impl From<$type> for Decimal {
            fn from(val: $type) -> Self {
                Self((val as i128) * Self::ONE.0)
            }
        }
    };
}
from_int!(u8);
from_int!(u16);
from_int!(u32);
from_int!(u64);
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
            Self::from(1)
        } else {
            Self::from(0)
        }
    }
}

/// Creates a `Decimal` from literals.
///
/// # Example
/// ```ignore
/// use scrypto::prelude::*;
///
/// let a = dec!(1);
/// let b = dec!("1.1");
/// ```
#[macro_export]
macro_rules! dec {
    ($x:literal) => {
        ::scrypto::math::Decimal::from($x)
    };

    ($base:literal, $shift:literal) => {
        // Base can be any type that converts into a Decimal, and shift must support
        // comparison and `-` unary operation, enforced by rustc.
        {
            let base = ::scrypto::math::Decimal::from($base);
            if $shift >= 0 {
                base * 10i128.pow(u32::try_from($shift).expect("Shift overflow"))
            } else {
                base / 10i128.pow(u32::try_from(-$shift).expect("Shift overflow"))
            }
        }
    };
}

impl<T: Into<Decimal>> Add<T> for Decimal {
    type Output = Decimal;

    fn add(self, other: T) -> Self::Output {
        let a = BigInt::from(self.0);
        let b = BigInt::from(other.into().0);
        let c = a + b;
        big_int_to_decimal(c)
    }
}

impl<T: Into<Decimal>> Sub<T> for Decimal {
    type Output = Decimal;

    fn sub(self, other: T) -> Self::Output {
        let a = BigInt::from(self.0);
        let b = BigInt::from(other.into().0);
        let c = a - b;
        big_int_to_decimal(c)
    }
}

fn big_int_to_i128(v: BigInt) -> i128 {
    i128::from_le_bytes(calc_le_bytes(v))
}

fn calc_le_bytes (v: BigInt) -> [u8; 16] {
    let bytes = v.to_signed_bytes_le();
    if bytes.len() > 16 {
        panic!("Overflow");
    } else {
        let mut buf = if v.is_negative() {
            [255u8; 16]
        } else {
            [0u8; 16]
        };
        buf[..bytes.len()].copy_from_slice(&bytes);
        buf
    }
}

fn big_int_to_decimal(v: BigInt) -> Decimal {
    Decimal(i128::from_le_bytes(calc_le_bytes(v)))
}

impl<T: Into<Decimal>> Mul<T> for Decimal {
    type Output = Decimal;

    fn mul(self, other: T) -> Self::Output {
        let a = BigInt::from(self.0);
        let b = BigInt::from(other.into().0);
        let c = a * b / Self::ONE.0;
        big_int_to_decimal(c)
    }
}

impl<T: Into<Decimal>> Div<T> for Decimal {
    type Output = Decimal;

    fn div(self, other: T) -> Self::Output {
        let a = BigInt::from(self.0);
        let b = BigInt::from(other.into().0);
        let c = a * Self::ONE.0 / b;
        big_int_to_decimal(c)
    }
}

impl Neg for Decimal {
    type Output = Decimal;

    fn neg(self) -> Self::Output {
        Decimal(-self.0)
    }
}

impl<T: Into<Decimal>> AddAssign<T> for Decimal {
    fn add_assign(&mut self, other: T) {
        self.0 += other.into().0;
    }
}

impl<T: Into<Decimal>> SubAssign<T> for Decimal {
    fn sub_assign(&mut self, other: T) {
        self.0 -= other.into().0;
    }
}

impl<T: Into<Decimal>> MulAssign<T> for Decimal {
    fn mul_assign(&mut self, other: T) {
        self.0 = (self.clone() * other.into()).0;
    }
}

impl<T: Into<Decimal>> DivAssign<T> for Decimal {
    fn div_assign(&mut self, other: T) {
        self.0 = (self.clone() / other.into()).0;
    }
}

//========
// error
//========

/// Represents an error when parsing Decimal from hex string.
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

//========
// binary
//========

impl TryFrom<&[u8]> for Decimal {
    type Error = ParseDecimalError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        if slice.len() == 16 {
            Ok(Self(i128::from_le_bytes(copy_u8_array(slice))))
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
        let mut sign = 1i128;
        let mut value = 0i128;

        let chars: Vec<char> = s.chars().collect();
        let mut p = 0;

        // read sign
        if chars[p] == '-' {
            sign = -1;
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
        for _ in 0..18 {
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

impl fmt::Display for Decimal {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let mut a = self.0;
        let mut buf = String::new();

        let mut trailing_zeros = true;
        for _ in 0..18 {
            let m = a % 10;
            if m != 0 || !trailing_zeros {
                trailing_zeros = false;
                buf.push(char::from_digit(m.abs() as u32, 10).unwrap())
            }
            a /= 10;
        }

        if !buf.is_empty() {
            buf.push('.');
        }

        if a == 0 {
            buf.push('0')
        } else {
            while a != 0 {
                let m = a % 10;
                buf.push(char::from_digit(m.abs() as u32, 10).unwrap());
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

impl fmt::Debug for Decimal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

fn read_digit(c: char) -> Result<i128, ParseDecimalError> {
    let n = c as i128;
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

#[cfg(test)]
mod tests {
    use super::*;

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
    #[should_panic(expected = "Overflow")]
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
    #[should_panic(expected = "Overflow")]
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
    #[should_panic(expected = "Overflow")]
    fn test_mul_overflow_by_small() {
        let _ = Decimal::MAX * dec!("1.000000000000000001");
    }

    #[test]
    #[should_panic(expected = "Overflow")]
    fn test_mul_overflow_by_a_lot() {
        let _ = Decimal::MAX * dec!("1.1");
    }

    #[test]
    #[should_panic(expected = "Overflow")]
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
        assert_eq!((a.pow(0)).to_string(), "1");
    }

    #[test]
    fn test_0_pow_1() {
        let a = dec!("0");
        assert_eq!((a.pow(1)).to_string(), "0");
    }

    #[test]
    fn test_0_pow_10() {
        let a = dec!("0");
        assert_eq!((a.pow(10)).to_string(), "0");
    }

    #[test]
    fn test_1_pow_0() {
        let a = dec!("1");
        assert_eq!((a.pow(0)).to_string(), "1");
    }

    #[test]
    fn test_1_pow_1() {
        let a = dec!("1");
        assert_eq!((a.pow(1)).to_string(), "1");
    }

    #[test]
    fn test_1_pow_10() {
        let a = dec!("1");
        assert_eq!((a.pow(10)).to_string(), "1");
    }
    
    #[test]
    fn test_2_pow_0() {
        let a = dec!("2");
        assert_eq!((a.pow(0)).to_string(), "1");
    }
    
    #[test]
    fn test_2_pow_1() {
        let a = dec!("1.000234891009084238");
        assert_eq!((a.pow(3724)).to_string(), "2.397991232254676688");
    }
    
    #[test]
    fn test_2_pow_2() {
        let a = dec!("2");
        assert_eq!((a.pow(2)).to_string(), "4");
    }
    
    #[test]
    fn test_2_pow_3() {
        let a = dec!("2");
        assert_eq!((a.pow(3)).to_string(), "8");
    }

    #[test]
    fn test_10_pow_3() {
        let a = dec!("10");
        assert_eq!((a.pow(3)).to_string(), "1000");
    }

    #[test]
    fn test_5_pow_2() {
        let a = dec!("5");
        assert_eq!((a.pow(2)).to_string(), "25");
    }

    #[test]
    fn test_5_pow_minus2() {
        let a = dec!("5");
        assert_eq!((a.pow(-2)).to_string(), "0.04");
    }

    #[test]
    fn test_10_pow_minus3() {
        let a = dec!("10");
        assert_eq!((a.pow(-3)).to_string(), "0.001");
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
}
