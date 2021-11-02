use core::ops::*;

use num_bigint::BigUint;
use num_traits::{Num, One, ToPrimitive, Zero};
use sbor::{describe::Type, *};

use crate::buffer::*;
use crate::rust::borrow::ToOwned;
use crate::rust::convert::TryFrom;
use crate::rust::fmt;
use crate::rust::str::FromStr;
use crate::rust::string::String;
use crate::rust::vec;
use crate::rust::vec::Vec;
use crate::types::Decimal;
use crate::utils::*;

/// Represents the quantity of some resource. It's always **unsigned**.
///
/// Only a subset of arithmetic operations are allowed:
/// - Adds two `Amount`s;
/// - Subtract an `Amount` by another `Amount`;
/// - Divides an `Amount` by an unsigned number;
/// - Multiplies an `Amount` by an unsigned number.
///
/// If you need more, consider converting into `Decimal` instead.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Amount(BigUint);

/// Represents an error when parsing Amount.
#[derive(Debug, Clone)]
pub enum ParseAmountError {
    InvalidAmount(String),
}

impl Amount {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_bytes_le()
    }

    pub fn to_decimal(&self, decimals: u8) -> Decimal {
        Decimal::new(self.0.clone(), decimals)
    }

    pub fn zero() -> Self {
        Self(BigUint::zero())
    }

    pub fn one() -> Self {
        Self(BigUint::one())
    }

    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }
}

macro_rules! from_uint {
    ($type:ident) => {
        impl From<$type> for Amount {
            fn from(val: $type) -> Self {
                Self(val.into())
            }
        }
    };
}
from_uint!(u8);
from_uint!(u16);
from_uint!(u32);
from_uint!(u64);
from_uint!(u128);
from_uint!(usize);

macro_rules! from_int {
    ($type:ident) => {
        impl From<$type> for Amount {
            fn from(val: $type) -> Self {
                if val < 0 {
                    scrypto_abort("Negative value can't be converted into Amount");
                } else {
                    Self((val as u128).into())
                }
            }
        }
    };
}
from_int!(i8);
from_int!(i16);
from_int!(i32);
from_int!(i64);
from_int!(i128);
from_int!(isize);

impl From<BigUint> for Amount {
    fn from(value: BigUint) -> Self {
        Self(value)
    }
}

impl<T: Into<Amount>> Add<T> for Amount {
    type Output = Amount;

    fn add(self, other: T) -> Self::Output {
        Self(self.0 + other.into().0)
    }
}

impl<T: Into<Amount>> Sub<T> for Amount {
    type Output = Amount;

    fn sub(self, other: T) -> Self::Output {
        Self(self.0 - other.into().0)
    }
}

impl<T: Into<u128>> Mul<T> for Amount {
    type Output = Amount;

    fn mul(self, other: T) -> Self::Output {
        Self(self.0 * other.into())
    }
}

impl<T: Into<u128>> Div<T> for Amount {
    type Output = Amount;

    fn div(self, other: T) -> Self::Output {
        Self(self.0 / other.into())
    }
}

impl<T: Into<Amount>> AddAssign<T> for Amount {
    fn add_assign(&mut self, other: T) {
        self.0 = self.0.clone() + other.into().0;
    }
}

impl<T: Into<Amount>> SubAssign<T> for Amount {
    fn sub_assign(&mut self, other: T) {
        self.0 = self.0.clone() - other.into().0;
    }
}

impl<T: Into<u128>> MulAssign<T> for Amount {
    fn mul_assign(&mut self, other: T) {
        self.0 = self.0.clone() * other.into();
    }
}

impl<T: Into<u128>> DivAssign<T> for Amount {
    fn div_assign(&mut self, other: T) {
        self.0 = self.0.clone() / other.into();
    }
}

impl FromStr for Amount {
    type Err = ParseAmountError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(BigUint::from_str_radix(s, 10).map_err(|_| {
            ParseAmountError::InvalidAmount(s.to_owned())
        })?))
    }
}

impl TryFrom<&[u8]> for Amount {
    type Error = ParseAmountError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        Ok(Self(BigUint::from_bytes_le(slice)))
    }
}

impl fmt::Debug for Amount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // format big int
        let mut v = Vec::new();
        let mut r = self.0.clone();
        loop {
            v.push(char::from_digit((r.clone() % 10u32).to_u32().unwrap(), 10).unwrap());
            r = r / 10u32;
            if r.is_zero() {
                break;
            }
        }
        let raw = v.iter().rev().collect::<String>();

        write!(f, "{}", raw)
    }
}

impl fmt::Display for Amount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl TypeId for Amount {
    #[inline]
    fn type_id() -> u8 {
        SCRYPTO_TYPE_AMOUNT
    }
}

impl Encode for Amount {
    fn encode_value(&self, encoder: &mut Encoder) {
        let bytes = self.to_vec();
        encoder.write_len(bytes.len());
        encoder.write_slice(&bytes);
    }
}

impl Decode for Amount {
    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        let len = decoder.read_len()?;
        let slice = decoder.read_bytes(len)?;
        Self::try_from(slice).map_err(|_| DecodeError::InvalidCustomData(SCRYPTO_TYPE_AMOUNT))
    }
}

impl Describe for Amount {
    fn describe() -> Type {
        Type::Custom {
            name: SCRYPTO_NAME_AMOUNT.to_owned(),
            generics: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rust::string::ToString;

    #[test]
    fn test_from_to_string() {
        let s = "123";
        let a = Amount::from_str(s).unwrap();
        assert_eq!(a.to_string(), s);
    }

    #[test]
    fn test_math() {
        let mut a = Amount::from(7);
        assert_eq!(Amount::from(10), a.clone() + 3);
        a += 3;
        assert_eq!(Amount::from(10), a);

        let mut a = Amount::from(7);
        assert_eq!(Amount::from(4), a.clone() - 3);
        a -= 3;
        assert_eq!(Amount::from(4), a);

        let mut a = Amount::from(7);
        assert_eq!(Amount::from(21), a.clone() * 3u32);
        a *= 3u32;
        assert_eq!(Amount::from(21), a);

        let mut a = Amount::from(7);
        assert_eq!(Amount::from(2), a.clone() / 3u32);
        a /= 3u32;
        assert_eq!(Amount::from(2), a);
    }

    #[test]
    #[should_panic]
    #[allow(unused_must_use)]
    fn test_divide_by_zero() {
        Amount::from(10) / 0u32;
    }

    #[test]
    #[should_panic]
    #[allow(unused_must_use)]
    fn test_overflow() {
        Amount::from(1) - Amount::from(2);
    }
}
