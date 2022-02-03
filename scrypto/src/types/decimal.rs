use core::ops::*;

use num_bigint::BigInt;
use num_traits::Signed;
use sbor::{describe::Type, *};

use crate::buffer::*;
use crate::rust::borrow::ToOwned;
use crate::rust::convert::TryFrom;
use crate::rust::fmt;
use crate::rust::str::FromStr;
use crate::rust::string::String;
use crate::rust::vec;
use crate::rust::vec::Vec;
use crate::types::copy_u8_array;

/// The universal precision used by `Decimal`.
pub const PRECISION: i128 = 10i128.pow(18);

/// Represents a **signed**, **bounded** fixed-point decimal, where the precision is 10^-18.
///
/// Panic when there is an overflow.
///
/// FIXME prevent RE from panicking caused by arithmetic overflow.
///
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Decimal(pub i128);

/// Represents an error when parsing Decimal.
#[derive(Debug, Clone)]
pub enum ParseDecimalError {
    InvalidDecimal(String),
    InvalidChar(char),
    UnsupportedDecimalPlace,
    InvalidLength,
}

impl fmt::Display for ParseDecimalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseDecimalError {}

impl Decimal {
    /// The min value of `Decimal`.
    pub const MIN: Self = Self(i128::MIN);

    /// The max value of `Decimal`.
    pub const MAX: Self = Self(i128::MAX);

    /// Returns `Decimal` of 0.
    pub fn zero() -> Self {
        0.into()
    }

    /// Returns `Decimal` of 1.
    pub fn one() -> Self {
        1.into()
    }

    /// Converts into a vector of bytes.
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_le_bytes().to_vec()
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
}

macro_rules! from_int {
    ($type:ident) => {
        impl From<$type> for Decimal {
            fn from(val: $type) -> Self {
                Self((val as i128) * PRECISION)
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

#[macro_export]
macro_rules! dec {
    
    ($x:expr) => {
       Decimal::from($x) 
    };
    
    ($int:expr, $exponent:expr) => {
        if (($exponent) / 1i128) < 0 {
            Decimal(($int) * PRECISION).div(10f64.powf(-($exponent) as f64) as i128)
        } else {
            Decimal(($int) * PRECISION).mul(10f64.powf(($exponent) as f64) as i128)
        }
    };
}

//=====
// ADD
//=====

impl<T: Into<Decimal>> Add<T> for Decimal {
    type Output = Decimal;

    fn add(self, other: T) -> Self::Output {
        Decimal(self.0 + other.into().0)
    }
}

//=====
// Sub
//=====

impl<T: Into<Decimal>> Sub<T> for Decimal {
    type Output = Decimal;

    fn sub(self, other: T) -> Self::Output {
        Decimal(self.0 - other.into().0)
    }
}

//=====
// Mul
//=====

fn big_int_to_decimal(v: BigInt) -> Decimal {
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
        Decimal(i128::from_le_bytes(buf))
    }
}

impl<T: Into<Decimal>> Mul<T> for Decimal {
    type Output = Decimal;

    fn mul(self, other: T) -> Self::Output {
        let a = BigInt::from(self.0);
        let b = BigInt::from(other.into().0);
        let c = a * b / PRECISION;
        big_int_to_decimal(c)
    }
}

//=====
// Div
//=====

impl<T: Into<Decimal>> Div<T> for Decimal {
    type Output = Decimal;

    fn div(self, other: T) -> Self::Output {
        let a = BigInt::from(self.0);
        let b = BigInt::from(other.into().0);
        let c = a * PRECISION / b;
        big_int_to_decimal(c)
    }
}

//=======
// Neg
//=======

impl Neg for Decimal {
    type Output = Decimal;

    fn neg(self) -> Self::Output {
        Decimal(-self.0)
    }
}

//===========
// AddAssign
//===========

impl<T: Into<Decimal>> AddAssign<T> for Decimal {
    fn add_assign(&mut self, other: T) {
        self.0 += other.into().0;
    }
}

//===========
// SubAssign
//===========

impl<T: Into<Decimal>> SubAssign<T> for Decimal {
    fn sub_assign(&mut self, other: T) {
        self.0 -= other.into().0;
    }
}

//===========
// MulAssign
//===========

impl<T: Into<Decimal>> MulAssign<T> for Decimal {
    fn mul_assign(&mut self, other: T) {
        self.0 = (self.clone() * other.into()).0;
    }
}

//===========
// DivAssign
//===========

impl<T: Into<Decimal>> DivAssign<T> for Decimal {
    fn div_assign(&mut self, other: T) {
        self.0 = (self.clone() / other.into()).0;
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

impl TryFrom<&[u8]> for Decimal {
    type Error = ParseDecimalError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        if slice.len() != 16 {
            return Err(ParseDecimalError::InvalidLength);
        };

        Ok(Self(i128::from_le_bytes(copy_u8_array(slice))))
    }
}

impl fmt::Debug for Decimal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

impl fmt::Display for Decimal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl TypeId for Decimal {
    #[inline]
    fn type_id() -> u8 {
        SCRYPTO_TYPE_DECIMAL
    }
}

impl Encode for Decimal {
    fn encode_value(&self, encoder: &mut Encoder) {
        let bytes = self.to_vec();
        encoder.write_len(bytes.len());
        encoder.write_slice(&bytes);
    }
}

impl Decode for Decimal {
    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        let len = decoder.read_len()?;
        let slice = decoder.read_bytes(len)?;
        Self::try_from(slice).map_err(|_| DecodeError::InvalidCustomData(SCRYPTO_TYPE_DECIMAL))
    }
}

impl Describe for Decimal {
    fn describe() -> Type {
        Type::Custom {
            name: SCRYPTO_NAME_DECIMAL.to_owned(),
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
    fn test_sub() {
        let a = Decimal::from(5u32);
        let b = Decimal::from(7u32);
        assert_eq!((a - b).to_string(), "-2");
        assert_eq!((b - a).to_string(), "2");
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
    fn test_one_and_zero() {
        assert_eq!(Decimal::one().to_string(), "1");
        assert_eq!(Decimal::zero().to_string(), "0");
    }

    #[test]
    fn test_dec_string_decimal() {
        assert_eq!(dec!("1.123456789012345678").to_string(), "1.123456789012345678");
        assert_eq!(dec!("-5.6").to_string(), "-5.6");
    }

    #[test]
    fn test_dec_string_decimal_exp() {
        assert_eq!(dec!("1.12345678901234567".to_string() + "8").to_string(), "1.123456789012345678");
        assert_eq!(dec!("-".to_string() + "5.6").to_string(), "-5.6");
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
    fn test_dec_int_expr() {
        assert_eq!(dec!(1 + 2 - 2).to_string(), "1");
        assert_eq!(dec!(5 * 3 / 3).to_string(), "5");
    }

    #[test]
    fn test_dec_bool() {
        assert_eq!((dec!(true)).to_string(), "1");
        assert_eq!((dec!(false)).to_string(), "0");
    }

    #[test]
    fn test_dec_bool_expr() {
        assert_eq!((dec!(if 4 < 5 {true} else {false})).to_string(), "1");
        assert_eq!((dec!(if -2 > 3 {true} else {false})).to_string(), "0");
    }

    #[test]
    fn test_dec_rational() {
        assert_eq!((dec!(11235, 0)).to_string(), "11235");
        assert_eq!((dec!(11235, -2)).to_string(), "112.35");
        assert_eq!((dec!(11235, 2)).to_string(), "1123500");

        assert_eq!((dec!(112000000000000000001, -18)).to_string(),
            "112.000000000000000001");
        
        assert_eq!((dec!(112000000000000000001, -18)).to_string(),
            "112.000000000000000001");
    }

    #[test]
    fn test_dec_rational_expr() {
        let a = 3;
        assert_eq!((dec!(a + 2, 5)).to_string(), "500000");
        assert_eq!((dec!(3 + 2, 5)).to_string(), "500000");
        assert_eq!((dec!(100 + 12, 0 - 2)).to_string(), "1.12");
        assert_eq!((dec!(a, a - 5)).to_string(), "0.03");
    }
}
