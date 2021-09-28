use core::ops::*;

use sbor::{describe::Type, *};
use uint::construct_uint;

use crate::constants::*;
use crate::rust::borrow::ToOwned;
use crate::rust::convert::TryFrom;
use crate::rust::fmt;
use crate::rust::str::FromStr;
use crate::rust::string::String;
use crate::rust::string::ToString;
use crate::rust::vec::Vec;

construct_uint! {
    struct U256(4);
}

// TODO: Make Amount a big int.

/// Represents a bucket id.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Amount(U256);

/// Represents an error when parsing Amount.
#[derive(Debug, Clone)]
pub enum ParseAmountError {
    InvalidAmount(String),
    InvalidLength(usize),
}

impl Amount {
    pub fn to_vec(&self) -> Vec<u8> {
        let mut bytes = [0u8; 32];
        self.0.to_little_endian(&mut bytes);
        bytes.to_vec()
    }

    pub fn from_little_endian(slice: &[u8]) -> Self {
        Self(U256::from_little_endian(slice))
    }

    pub fn zero() -> Self {
        Self(U256::zero())
    }

    pub fn one() -> Self {
        Self(U256::one())
    }

    pub fn exp10(n: usize) -> Self {
        Self(U256::exp10(n))
    }

    pub fn pow(self, exp: Self) -> Self {
        Self(self.0.pow(exp.0))
    }

    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    pub fn bits(&self) -> usize {
        self.0.bits()
    }

    pub fn as_u32(&self) -> u32 {
        self.0.as_u32()
    }

    pub fn as_u64(&self) -> u64 {
        self.0.as_u64()
    }

    pub fn as_u128(&self) -> u128 {
        self.0.as_u128()
    }

    pub fn as_usize(&self) -> usize {
        self.0.as_usize()
    }
}

macro_rules! from_int {
    ($type:ident) => {
        impl From<$type> for Amount {
            fn from(val: $type) -> Self {
                Self(val.into())
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

impl<T: Into<Amount>> Add<T> for Amount {
    type Output = Amount;

    fn add(self, other: T) -> Self::Output {
        Self(Add::add(self.0, Into::<Amount>::into(other).0))
    }
}

impl<T: Into<Amount>> Sub<T> for Amount {
    type Output = Amount;

    fn sub(self, other: T) -> Self::Output {
        Self(Sub::sub(self.0, Into::<Amount>::into(other).0))
    }
}

impl<T: Into<Amount>> Mul<T> for Amount {
    type Output = Amount;

    fn mul(self, other: T) -> Self::Output {
        Self(Mul::mul(self.0, Into::<Amount>::into(other).0))
    }
}

impl<T: Into<Amount>> Div<T> for Amount {
    type Output = Amount;

    fn div(self, other: T) -> Self::Output {
        Self(Div::div(self.0, Into::<Amount>::into(other).0))
    }
}
impl Shl<usize> for Amount {
    type Output = Amount;

    fn shl(self, shift: usize) -> Self::Output {
        Self(Shl::shl(self.0, shift))
    }
}

impl Shr<usize> for Amount {
    type Output = Amount;

    fn shr(self, shift: usize) -> Self::Output {
        Self(Shr::shr(self.0, shift))
    }
}

impl<T: Into<Amount>> AddAssign<T> for Amount {
    fn add_assign(&mut self, other: T) {
        AddAssign::add_assign(&mut self.0, Into::<Amount>::into(other).0);
    }
}

impl<T: Into<Amount>> SubAssign<T> for Amount {
    fn sub_assign(&mut self, other: T) {
        SubAssign::sub_assign(&mut self.0, Into::<Amount>::into(other).0);
    }
}

impl<T: Into<Amount>> MulAssign<T> for Amount {
    fn mul_assign(&mut self, other: T) {
        MulAssign::mul_assign(&mut self.0, Into::<Amount>::into(other).0);
    }
}

impl<T: Into<Amount>> DivAssign<T> for Amount {
    fn div_assign(&mut self, other: T) {
        DivAssign::div_assign(&mut self.0, Into::<Amount>::into(other).0);
    }
}

impl ShlAssign<usize> for Amount {
    fn shl_assign(&mut self, shift: usize) {
        ShlAssign::shl_assign(&mut self.0, shift);
    }
}

impl ShrAssign<usize> for Amount {
    fn shr_assign(&mut self, shift: usize) {
        ShrAssign::shr_assign(&mut self.0, shift);
    }
}

impl FromStr for Amount {
    type Err = ParseAmountError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(U256::from_dec_str(s).map_err(|_| {
            ParseAmountError::InvalidAmount(s.to_owned())
        })?))
    }
}

impl TryFrom<&[u8]> for Amount {
    type Error = ParseAmountError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        if slice.len() != 32 {
            Err(ParseAmountError::InvalidLength(slice.len()))
        } else {
            Ok(Self::from_little_endian(slice))
        }
    }
}

impl fmt::Debug for Amount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.to_string())
    }
}

impl fmt::Display for Amount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.to_string())
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
        assert_eq!(Amount::from(10), a + 3);
        a += 3;
        assert_eq!(Amount::from(10), a);

        let mut a = Amount::from(7);
        assert_eq!(Amount::from(4), a - 3);
        a -= 3;
        assert_eq!(Amount::from(4), a);

        let mut a = Amount::from(7);
        assert_eq!(Amount::from(21), a * 3);
        a *= 3;
        assert_eq!(Amount::from(21), a);

        let mut a = Amount::from(7);
        assert_eq!(Amount::from(2), a / 3);
        a /= 3;
        assert_eq!(Amount::from(2), a);
    }

    #[test]
    #[should_panic]
    #[allow(unused_must_use)]
    fn test_divide_by_zero() {
        Amount::from(10) / Amount::zero();
    }

    #[test]
    #[should_panic]
    #[allow(unused_must_use)]
    fn test_overflow() {
        Amount::from(1) - Amount::from(2);
    }
}
