use core::ops::*;

use num_bigint::{BigInt, Sign};
use num_traits::{sign::Signed, ToPrimitive, Zero};
use sbor::{describe::Type, *};

use crate::buffer::*;
use crate::rust::borrow::ToOwned;
use crate::rust::convert::TryFrom;
use crate::rust::fmt;
use crate::rust::format;
use crate::rust::str::FromStr;
use crate::rust::string::String;
use crate::rust::vec;
use crate::rust::vec::Vec;

const PRECISION: u128 = 10u128.pow(18);

/// Represented a **signed** fixed-point decimal, where the precision is 10^-18.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Decimal(pub BigInt);

/// Represents an error when parsing Decimal.
#[derive(Debug, Clone)]
pub enum ParseDecimalError {
    InvalidDecimal(String),
    InvalidSign(u8),
    InvalidLength,
}

impl Decimal {
    pub fn zero() -> Self {
        Self(0.into())
    }

    pub fn one() -> Self {
        Self(1.into())
    }

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

    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    pub fn is_positive(&self) -> bool {
        self.0.is_positive()
    }

    pub fn is_negative(&self) -> bool {
        self.0.is_negative()
    }
}

macro_rules! from_int {
    ($type:ident) => {
        impl From<$type> for Decimal {
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

//=====
// ADD
//=====

impl<T: Into<Decimal>> Add<T> for Decimal {
    type Output = Decimal;

    fn add(self, other: T) -> Self::Output {
        Decimal(self.0 + other.into().0)
    }
}

impl<'a> Add<&'a Decimal> for Decimal {
    type Output = Decimal;

    fn add(self, other: &'a Decimal) -> Self::Output {
        Decimal(self.0.clone() + other.0.clone())
    }
}

impl<'a, T: Into<Decimal>> Add<T> for &'a Decimal {
    type Output = Decimal;

    fn add(self, other: T) -> Self::Output {
        Decimal(self.0.clone() + other.into().0)
    }
}

impl<'a, 'b> Add<&'a Decimal> for &'b Decimal {
    type Output = Decimal;

    fn add(self, other: &'a Decimal) -> Self::Output {
        Decimal(self.0.clone() + other.0.clone())
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

impl<'a> Sub<&'a Decimal> for Decimal {
    type Output = Decimal;

    fn sub(self, other: &'a Decimal) -> Self::Output {
        Decimal(self.0.clone() - other.0.clone())
    }
}

impl<'a, T: Into<Decimal>> Sub<T> for &'a Decimal {
    type Output = Decimal;

    fn sub(self, other: T) -> Self::Output {
        Decimal(self.0.clone() - other.into().0)
    }
}

impl<'a, 'b> Sub<&'a Decimal> for &'b Decimal {
    type Output = Decimal;

    fn sub(self, other: &'a Decimal) -> Self::Output {
        Decimal(self.0.clone() - other.0.clone())
    }
}

//=====
// Mul
//=====

impl<T: Into<Decimal>> Mul<T> for Decimal {
    type Output = Decimal;

    fn mul(self, other: T) -> Self::Output {
        Decimal(self.0 * other.into().0 / PRECISION)
    }
}

impl<'a> Mul<&'a Decimal> for Decimal {
    type Output = Decimal;

    fn mul(self, other: &'a Decimal) -> Self::Output {
        Decimal(self.0.clone() * other.0.clone() / PRECISION)
    }
}

impl<'a, T: Into<Decimal>> Mul<T> for &'a Decimal {
    type Output = Decimal;

    fn mul(self, other: T) -> Self::Output {
        Decimal(self.0.clone() * other.into().0 / PRECISION)
    }
}

impl<'a, 'b> Mul<&'a Decimal> for &'b Decimal {
    type Output = Decimal;

    fn mul(self, other: &'a Decimal) -> Self::Output {
        Decimal(self.0.clone() * other.0.clone() / PRECISION)
    }
}

//=====
// Div
//=====

impl<T: Into<Decimal>> Div<T> for Decimal {
    type Output = Decimal;

    fn div(self, other: T) -> Self::Output {
        Decimal(self.0 * PRECISION / other.into().0)
    }
}

impl<'a> Div<&'a Decimal> for Decimal {
    type Output = Decimal;

    fn div(self, other: &'a Decimal) -> Self::Output {
        Decimal(self.0.clone() * PRECISION / other.0.clone())
    }
}

impl<'a, T: Into<Decimal>> Div<T> for &'a Decimal {
    type Output = Decimal;

    fn div(self, other: T) -> Self::Output {
        Decimal(self.0.clone() * PRECISION / other.into().0)
    }
}

impl<'a, 'b> Div<&'a Decimal> for &'b Decimal {
    type Output = Decimal;

    fn div(self, other: &'a Decimal) -> Self::Output {
        Decimal(self.0.clone() * PRECISION / other.0.clone())
    }
}

//=======
// Shift
//=======

impl Shl<usize> for Decimal {
    type Output = Decimal;

    fn shl(self, shift: usize) -> Self::Output {
        Decimal(self.0.shl(shift))
    }
}

impl Shr<usize> for Decimal {
    type Output = Decimal;

    fn shr(self, shift: usize) -> Self::Output {
        Decimal(self.0.shr(shift))
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

impl<'a> AddAssign<&'a Decimal> for Decimal {
    fn add_assign(&mut self, other: &'a Decimal) {
        self.0 += other.0.clone();
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

impl<'a> SubAssign<&'a Decimal> for Decimal {
    fn sub_assign(&mut self, other: &'a Decimal) {
        self.0 -= other.0.clone();
    }
}

//===========
// MulAssign
//===========

impl<T: Into<Decimal>> MulAssign<T> for Decimal {
    fn mul_assign(&mut self, other: T) {
        self.0 = self.0.clone() * other.into().0 / PRECISION;
    }
}

impl<'a> MulAssign<&'a Decimal> for Decimal {
    fn mul_assign(&mut self, other: &'a Decimal) {
        self.0 = self.0.clone() * other.0.clone() / PRECISION;
    }
}

//===========
// DivAssign
//===========

impl<T: Into<Decimal>> DivAssign<T> for Decimal {
    fn div_assign(&mut self, other: T) {
        self.0 = self.0.clone() * PRECISION / other.into().0;
    }
}

impl<'a> DivAssign<&'a Decimal> for Decimal {
    fn div_assign(&mut self, other: &'a Decimal) {
        self.0 = self.0.clone() * PRECISION / other.0.clone();
    }
}

impl FromStr for Decimal {
    type Err = ParseDecimalError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // TODO: this is for the happy path; need to handle invalid formatting.

        let mut is_negative = false;
        let mut value = BigInt::zero();

        let chars: Vec<char> = s.chars().collect();
        let mut p = 0;

        // read sign
        if chars[p] == '-' {
            is_negative = true;
            p += 1;
        }

        // read integral
        while p < chars.len() && chars[p] != '.' {
            value = value * 10 + chars[p] as u32 - 48;
            p += 1;
        }

        // read radix point
        if p < chars.len() && chars[p] == '.' {
            p += 1;
        }

        // read fraction
        for i in 0..18 {
            if p + i < chars.len() {
                value = value * 10 + chars[p + i] as u32 - 48;
            } else {
                value *= 10;
            }
        }

        if is_negative {
            value *= -1;
        }

        Ok(Self(value))
    }
}

impl TryFrom<&[u8]> for Decimal {
    type Error = ParseDecimalError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        let sign = if let Some(b) = slice.get(0) {
            match b {
                0 => Ok(Sign::NoSign),
                1 => Ok(Sign::Plus),
                2 => Ok(Sign::Minus),
                _ => Err(ParseDecimalError::InvalidSign(*b)),
            }
        } else {
            Err(ParseDecimalError::InvalidLength)
        };

        Ok(Self(BigInt::from_bytes_le(sign?, &slice[1..])))
    }
}

impl fmt::Debug for Decimal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // format big int
        let mut v = Vec::new();
        let mut r = self.0.abs();
        loop {
            v.push(char::from_digit((r.clone() % 10u32).to_u32().unwrap(), 10).unwrap());
            r = r / 10u32;
            if r.is_zero() {
                break;
            }
        }
        let raw = v.iter().rev().collect::<String>();

        // add radix point
        let scaled = if raw.len() <= 18 {
            format!("0.{}{}", "0".repeat(18 - raw.len()), raw)
        } else {
            format!("{}.{}", &raw[..raw.len() - 18], &raw[raw.len() - 18..])
        };

        // strip trailing zeros
        let mut res = scaled.as_str();
        while res.ends_with('0') {
            res = &res[..res.len() - 1];
        }
        if res.ends_with('.') {
            res = &res[..res.len() - 1];
        }

        write!(f, "{}{}", if self.0.is_negative() { "-" } else { "" }, res)
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
        assert_eq!(Decimal(1u128.into()).to_string(), "0.000000000000000001");
        assert_eq!(
            Decimal(123456789123456789u128.into()).to_string(),
            "0.123456789123456789"
        );
        assert_eq!(Decimal(1000000000000000000u128.into()).to_string(), "1");
        assert_eq!(Decimal(123000000000000000000u128.into()).to_string(), "123");
        assert_eq!(
            Decimal(123456789123456789000000000000000000u128.into()).to_string(),
            "123456789123456789"
        );
    }

    #[test]
    fn test_parse() {
        assert_eq!(
            Decimal(1u128.into()),
            "0.000000000000000001".parse().unwrap()
        );
        assert_eq!(
            Decimal(123456789123456789u128.into()),
            "0.123456789123456789".parse().unwrap()
        );
        assert_eq!(
            Decimal(1000000000000000000u128.into()),
            "1".parse().unwrap()
        );
        assert_eq!(
            Decimal(123456789123456789000000000000000000u128.into()),
            "123456789123456789".parse().unwrap()
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
        assert_eq!((&a - &b).to_string(), "-2");
        assert_eq!((&b - &a).to_string(), "2");
    }

    #[test]
    fn test_mul() {
        let a = Decimal::from(5u32);
        let b = Decimal::from(7u32);
        assert_eq!((a * b).to_string(), "35");
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
    }
}
