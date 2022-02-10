use core::ops::*;

use num_bigint::{BigInt, Sign};
use num_traits::{sign::Signed, Zero};
use sbor::{describe::Type, *};

use crate::buffer::*;
use crate::rust::borrow::ToOwned;
use crate::rust::convert::TryFrom;
use crate::rust::fmt;
use crate::rust::str::FromStr;
use crate::rust::string::String;
use crate::rust::vec;
use crate::rust::vec::Vec;

/// The universal precision used by `BigDecimal`.
const PRECISION: i128 = 10i128.pow(18);

/// Represents a **signed**, **unbounded** fixed-point decimal, where the precision is 10^-18.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BigDecimal(pub BigInt);

/// Represents an error when parsing decimal.
#[derive(Debug, Clone)]
pub enum ParseBigDecimalError {
    InvalidBigDecimal(String),
    InvalidSign(u8),
    InvalidChar(char),
    UnsupportedDecimalPlace,
    InvalidLength,
}

impl fmt::Display for ParseBigDecimalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseBigDecimalError {}

impl BigDecimal {
    /// Return a `BigDecimal` of 0.
    pub fn zero() -> Self {
        Self(0.into())
    }

    /// Return a `BigDecimal` of 1.
    pub fn one() -> Self {
        Self(1.into())
    }

    /// Converts into a vector of bytes.
    pub fn to_vec(&self) -> Vec<u8> {
        let mut result = Vec::new();
        let (sign, v) = self.0.to_bytes_le();
        match sign {
            Sign::NoSign => result.push(0u8),
            Sign::Plus => result.push(1u8),
            Sign::Minus => result.push(2u8),
        }
        result.extend(v);
        result
    }

    /// Whether this decimal is zero.
    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    /// Whether this decimal is positive.
    pub fn is_positive(&self) -> bool {
        self.0.is_positive()
    }

    /// Whether this decimal is negative.
    pub fn is_negative(&self) -> bool {
        self.0.is_negative()
    }

    /// Returns the absolute value.
    pub fn abs(&self) -> BigDecimal {
        BigDecimal(self.0.abs())
    }
}

macro_rules! from_int {
    ($type:ident) => {
        impl From<$type> for BigDecimal {
            fn from(val: $type) -> Self {
                Self(BigInt::from(val) * PRECISION)
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

impl From<&str> for BigDecimal {
    fn from(val: &str) -> Self {
        Self::from_str(&val).unwrap()
    }
}

impl From<String> for BigDecimal {
    fn from(val: String) -> Self {
        Self::from_str(&val).unwrap()
    }
}

impl From<bool> for BigDecimal {
    fn from(val: bool) -> Self {
        if val {
            Self::from(1)
        } else {
            Self::from(0)
        } 
    }
}

#[macro_export]
macro_rules! bdec {
    
    ($x:literal) => {
       BigDecimal::from($x) 
    };
    
    ($int:literal, $exponent:literal) => {
        if u32::try_from((($exponent) as i128).abs()).is_err() {
            panic!("Overflow of arg2.");
        } else {
            if (($exponent) as i128) < 0 {
                BigDecimal::from(1i128 * ($int))
                    .div(10i128.pow((-1i128 * ($exponent)) as u32))
            } else {
                BigDecimal::from(1i128 * ($int))
                    .mul(10i128.pow((1i128 * ($exponent)) as u32))
            }
        }
    };
}

//=====
// ADD
//=====

impl<T: Into<BigDecimal>> Add<T> for BigDecimal {
    type Output = BigDecimal;

    fn add(self, other: T) -> Self::Output {
        BigDecimal(self.0 + other.into().0)
    }
}

impl<'a> Add<&'a BigDecimal> for BigDecimal {
    type Output = BigDecimal;

    fn add(self, other: &'a BigDecimal) -> Self::Output {
        BigDecimal(self.0.clone() + other.0.clone())
    }
}

impl<'a, T: Into<BigDecimal>> Add<T> for &'a BigDecimal {
    type Output = BigDecimal;

    fn add(self, other: T) -> Self::Output {
        BigDecimal(self.0.clone() + other.into().0)
    }
}

impl<'a, 'b> Add<&'a BigDecimal> for &'b BigDecimal {
    type Output = BigDecimal;

    fn add(self, other: &'a BigDecimal) -> Self::Output {
        BigDecimal(self.0.clone() + other.0.clone())
    }
}

//=====
// Sub
//=====

impl<T: Into<BigDecimal>> Sub<T> for BigDecimal {
    type Output = BigDecimal;

    fn sub(self, other: T) -> Self::Output {
        BigDecimal(self.0 - other.into().0)
    }
}

impl<'a> Sub<&'a BigDecimal> for BigDecimal {
    type Output = BigDecimal;

    fn sub(self, other: &'a BigDecimal) -> Self::Output {
        BigDecimal(self.0.clone() - other.0.clone())
    }
}

impl<'a, T: Into<BigDecimal>> Sub<T> for &'a BigDecimal {
    type Output = BigDecimal;

    fn sub(self, other: T) -> Self::Output {
        BigDecimal(self.0.clone() - other.into().0)
    }
}

impl<'a, 'b> Sub<&'a BigDecimal> for &'b BigDecimal {
    type Output = BigDecimal;

    fn sub(self, other: &'a BigDecimal) -> Self::Output {
        BigDecimal(self.0.clone() - other.0.clone())
    }
}

//=====
// Mul
//=====

impl<T: Into<BigDecimal>> Mul<T> for BigDecimal {
    type Output = BigDecimal;

    fn mul(self, other: T) -> Self::Output {
        BigDecimal(self.0 * other.into().0 / PRECISION)
    }
}

impl<'a> Mul<&'a BigDecimal> for BigDecimal {
    type Output = BigDecimal;

    fn mul(self, other: &'a BigDecimal) -> Self::Output {
        BigDecimal(self.0.clone() * other.0.clone() / PRECISION)
    }
}

impl<'a, T: Into<BigDecimal>> Mul<T> for &'a BigDecimal {
    type Output = BigDecimal;

    fn mul(self, other: T) -> Self::Output {
        BigDecimal(self.0.clone() * other.into().0 / PRECISION)
    }
}

impl<'a, 'b> Mul<&'a BigDecimal> for &'b BigDecimal {
    type Output = BigDecimal;

    fn mul(self, other: &'a BigDecimal) -> Self::Output {
        BigDecimal(self.0.clone() * other.0.clone() / PRECISION)
    }
}

//=====
// Div
//=====

impl<T: Into<BigDecimal>> Div<T> for BigDecimal {
    type Output = BigDecimal;

    fn div(self, other: T) -> Self::Output {
        BigDecimal(self.0 * PRECISION / other.into().0)
    }
}

impl<'a> Div<&'a BigDecimal> for BigDecimal {
    type Output = BigDecimal;

    fn div(self, other: &'a BigDecimal) -> Self::Output {
        BigDecimal(self.0.clone() * PRECISION / other.0.clone())
    }
}

impl<'a, T: Into<BigDecimal>> Div<T> for &'a BigDecimal {
    type Output = BigDecimal;

    fn div(self, other: T) -> Self::Output {
        BigDecimal(self.0.clone() * PRECISION / other.into().0)
    }
}

impl<'a, 'b> Div<&'a BigDecimal> for &'b BigDecimal {
    type Output = BigDecimal;

    fn div(self, other: &'a BigDecimal) -> Self::Output {
        BigDecimal(self.0.clone() * PRECISION / other.0.clone())
    }
}

//=======
// Neg
//=======

impl Neg for BigDecimal {
    type Output = BigDecimal;

    fn neg(self) -> Self::Output {
        BigDecimal(-self.0)
    }
}

impl<'a> Neg for &'a BigDecimal {
    type Output = BigDecimal;

    fn neg(self) -> Self::Output {
        BigDecimal(-self.0.clone())
    }
}

//===========
// AddAssign
//===========

impl<T: Into<BigDecimal>> AddAssign<T> for BigDecimal {
    fn add_assign(&mut self, other: T) {
        self.0 += other.into().0;
    }
}

impl<'a> AddAssign<&'a BigDecimal> for BigDecimal {
    fn add_assign(&mut self, other: &'a BigDecimal) {
        self.0 += other.0.clone();
    }
}

//===========
// SubAssign
//===========

impl<T: Into<BigDecimal>> SubAssign<T> for BigDecimal {
    fn sub_assign(&mut self, other: T) {
        self.0 -= other.into().0;
    }
}

impl<'a> SubAssign<&'a BigDecimal> for BigDecimal {
    fn sub_assign(&mut self, other: &'a BigDecimal) {
        self.0 -= other.0.clone();
    }
}

//===========
// MulAssign
//===========

impl<T: Into<BigDecimal>> MulAssign<T> for BigDecimal {
    fn mul_assign(&mut self, other: T) {
        self.0 = self.0.clone() * other.into().0 / PRECISION;
    }
}

impl<'a> MulAssign<&'a BigDecimal> for BigDecimal {
    fn mul_assign(&mut self, other: &'a BigDecimal) {
        self.0 = self.0.clone() * other.0.clone() / PRECISION;
    }
}

//===========
// DivAssign
//===========

impl<T: Into<BigDecimal>> DivAssign<T> for BigDecimal {
    fn div_assign(&mut self, other: T) {
        self.0 = self.0.clone() * PRECISION / other.into().0;
    }
}

impl<'a> DivAssign<&'a BigDecimal> for BigDecimal {
    fn div_assign(&mut self, other: &'a BigDecimal) {
        self.0 = self.0.clone() * PRECISION / other.0.clone();
    }
}

fn read_digit(c: char) -> Result<i128, ParseBigDecimalError> {
    let n = c as i128;
    if n >= 48 && n <= 48 + 9 {
        Ok(n - 48)
    } else {
        Err(ParseBigDecimalError::InvalidChar(c))
    }
}

fn read_dot(c: char) -> Result<(), ParseBigDecimalError> {
    if c == '.' {
        Ok(())
    } else {
        Err(ParseBigDecimalError::InvalidChar(c))
    }
}

impl FromStr for BigDecimal {
    type Err = ParseBigDecimalError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut sign = 1i128;
        let mut value = BigInt::zero();

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
            Err(ParseBigDecimalError::UnsupportedDecimalPlace)
        } else {
            Ok(Self(value))
        }
    }
}

impl TryFrom<&[u8]> for BigDecimal {
    type Error = ParseBigDecimalError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        let sign = if let Some(b) = slice.get(0) {
            match b {
                0 => Ok(Sign::NoSign),
                1 => Ok(Sign::Plus),
                2 => Ok(Sign::Minus),
                _ => Err(ParseBigDecimalError::InvalidSign(*b)),
            }
        } else {
            Err(ParseBigDecimalError::InvalidLength)
        };

        Ok(Self(BigInt::from_bytes_le(sign?, &slice[1..])))
    }
}

fn big_int_to_u32_unchecked(v: BigInt) -> u32 {
    let (_, bytes) = v.to_bytes_le();
    bytes[0] as u32
}

impl fmt::Debug for BigDecimal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut a = self.0.clone();
        let mut buf = String::new();

        let mut trailing_zeros = true;
        for _ in 0..18 {
            let m: BigInt = &a % 10;
            if !m.is_zero() || !trailing_zeros {
                trailing_zeros = false;
                buf.push(char::from_digit(big_int_to_u32_unchecked(m.abs()), 10).unwrap())
            }
            a /= 10;
        }

        if !buf.is_empty() {
            buf.push('.');
        }

        if a.is_zero() {
            buf.push('0')
        } else {
            while !a.is_zero() {
                let m: BigInt = &a % 10;
                buf.push(char::from_digit(big_int_to_u32_unchecked(m.abs()), 10).unwrap());
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

impl fmt::Display for BigDecimal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl TypeId for BigDecimal {
    #[inline]
    fn type_id() -> u8 {
        SCRYPTO_TYPE_BIG_DECIMAL
    }
}

impl Encode for BigDecimal {
    fn encode_value(&self, encoder: &mut Encoder) {
        let bytes = self.to_vec();
        encoder.write_len(bytes.len());
        encoder.write_slice(&bytes);
    }
}

impl Decode for BigDecimal {
    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        let len = decoder.read_len()?;
        let slice = decoder.read_bytes(len)?;
        Self::try_from(slice).map_err(|_| DecodeError::InvalidCustomData(SCRYPTO_TYPE_BIG_DECIMAL))
    }
}

impl Describe for BigDecimal {
    fn describe() -> Type {
        Type::Custom {
            name: SCRYPTO_NAME_BIG_DECIMAL.to_owned(),
            generics: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rust::string::ToString;

    #[test]
    fn test_format() {
        assert_eq!(BigDecimal(1i128.into()).to_string(), "0.000000000000000001");
        assert_eq!(
            BigDecimal(123456789123456789i128.into()).to_string(),
            "0.123456789123456789"
        );
        assert_eq!(BigDecimal(1000000000000000000i128.into()).to_string(), "1");
        assert_eq!(
            BigDecimal(123000000000000000000i128.into()).to_string(),
            "123"
        );
        assert_eq!(
            BigDecimal(123456789123456789000000000000000000i128.into()).to_string(),
            "123456789123456789"
        );
        assert_eq!(
            BigDecimal(i128::MAX.into()).to_string(),
            "170141183460469231731.687303715884105727"
        );
        assert_eq!(
            BigDecimal(i128::MIN.into()).to_string(),
            "-170141183460469231731.687303715884105728"
        );
    }

    #[test]
    fn test_parse() {
        assert_eq!(
            BigDecimal::from_str("0.000000000000000001").unwrap(),
            BigDecimal(1i128.into()),
        );
        assert_eq!(
            BigDecimal::from_str("0.123456789123456789").unwrap(),
            BigDecimal(123456789123456789i128.into()),
        );
        assert_eq!(
            BigDecimal::from_str("1").unwrap(),
            BigDecimal(1000000000000000000i128.into()),
        );
        assert_eq!(
            BigDecimal::from_str("123456789123456789").unwrap(),
            BigDecimal(123456789123456789000000000000000000i128.into()),
        );
        assert_eq!(
            BigDecimal::from_str("170141183460469231731.687303715884105727").unwrap(),
            BigDecimal(i128::MAX.into())
        );
        assert_eq!(
            BigDecimal::from_str("-170141183460469231731.687303715884105728").unwrap(),
            BigDecimal(i128::MIN.into())
        );
    }

    #[test]
    fn test_add() {
        let a = BigDecimal::from(5u32);
        let b = BigDecimal::from(7u32);
        assert_eq!((a + b).to_string(), "12");
    }

    #[test]
    fn test_sub() {
        let a = BigDecimal::from(5u32);
        let b = BigDecimal::from(7u32);
        assert_eq!((&a - &b).to_string(), "-2");
        assert_eq!((&b - &a).to_string(), "2");
    }

    #[test]
    fn test_mul() {
        let a = BigDecimal::from(5u32);
        let b = BigDecimal::from(7u32);
        assert_eq!((a * b).to_string(), "35");
    }

    #[test]
    #[should_panic]
    fn test_div_by_zero() {
        let a = BigDecimal::from(5u32);
        let b = BigDecimal::from(0u32);
        assert_eq!((a / b).to_string(), "0");
    }

    #[test]
    fn test_div() {
        let a = BigDecimal::from(5u32);
        let b = BigDecimal::from(7u32);
        assert_eq!((a / b).to_string(), "0.714285714285714285");
    }

    #[test]
    fn test_bdec_string_decimal() {
        assert_eq!(bdec!("1.123456789012345678").to_string(), "1.123456789012345678");
        assert_eq!(bdec!("-5.6").to_string(), "-5.6");
    }

    #[test]
    fn test_bdec_string() {
        assert_eq!(bdec!("1").to_string(), "1");
        assert_eq!(bdec!("0").to_string(), "0");
    }

    #[test]
    fn test_bdec_int() {
        assert_eq!(bdec!(1).to_string(), "1");
        assert_eq!(bdec!(5).to_string(), "5");
    }

    #[test]
    fn test_bdec_bool() {
        assert_eq!((bdec!(true)).to_string(), "1");
        assert_eq!((bdec!(false)).to_string(), "0");
    }

    #[test]
    fn test_bdec_rational() {
        assert_eq!((bdec!(11235, 0)).to_string(), "11235");
        assert_eq!((bdec!(11235, -2)).to_string(), "112.35");
        assert_eq!((bdec!(11235, 2)).to_string(), "1123500");

        assert_eq!((bdec!(112000000000000000001, -18)).to_string(),
            "112.000000000000000001");
        
        assert_eq!((bdec!(112000000000000000001, -18)).to_string(),
            "112.000000000000000001");
    }
    
    #[test]
    #[should_panic(expected = "Overflow of arg2.")]
    fn test_arg1_overflow_arg1() {
        //u32::MAX + 1
        bdec!(1, 4_294_967_296);
    }
}
