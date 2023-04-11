use num_bigint::BigInt;
use num_traits::{One, Pow, Zero};
use sbor::rust::convert::{TryFrom, TryInto};
use sbor::rust::fmt;
use sbor::rust::format;
use sbor::rust::iter;
use sbor::rust::ops::*;
use sbor::rust::prelude::*;
use sbor::*;

use crate::data::manifest::ManifestCustomValueKind;
use crate::data::scrypto::*;
use crate::math::bnum_integer::*;
use crate::math::rounding_mode::*;
use crate::math::PreciseDecimal;
use crate::well_known_scrypto_custom_type;
use crate::*;

/// `Decimal` represents a 256 bit representation of a fixed-scale decimal number.
///
/// The finite set of values are of the form `m / 10^18`, where `m` is
/// an integer such that `-2^(256 - 1) <= m < 2^(256 - 1)`.
///
/// Unless otherwise specified, all operations will panic if underflow/overflow.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Decimal(pub BnumI256);

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

// TODO come up with some smarter formatting depending on Decimal::Scale
macro_rules! fmt_remainder {
    () => {
        "{:018}"
    };
}

impl Decimal {
    /// The min value of `Decimal`.
    pub const MIN: Self = Self(BnumI256::MIN);

    /// The max value of `Decimal`.
    pub const MAX: Self = Self(BnumI256::MAX);

    /// The bit length of number storing `Decimal`.
    pub const BITS: usize = BnumI256::BITS as usize;

    /// The fixed scale used by `Decimal`.
    pub const SCALE: u32 = 18;

    pub const ZERO: Self = Self(BnumI256::ZERO);

    pub const ONE: Self = Self(BnumI256::from_digits([10_u64.pow(Decimal::SCALE), 0, 0, 0]));

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
        self.0 == BnumI256::zero()
    }

    /// Whether this decimal is positive.
    pub fn is_positive(&self) -> bool {
        self.0 > BnumI256::zero()
    }

    /// Whether this decimal is negative.
    pub fn is_negative(&self) -> bool {
        self.0 < BnumI256::zero()
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

        let divisor: BnumI256 = BnumI256::from(10i8).pow(Self::SCALE - decimal_places);
        match mode {
            RoundingMode::TowardsPositiveInfinity => {
                if self.0 % divisor == BnumI256::zero() {
                    self.clone()
                } else if self.is_negative() {
                    Self(self.0 / divisor * divisor)
                } else {
                    Self((self.0 / divisor + BnumI256::one()) * divisor)
                }
            }
            RoundingMode::TowardsNegativeInfinity => {
                if self.0 % divisor == BnumI256::zero() {
                    self.clone()
                } else if self.is_negative() {
                    Self((self.0 / divisor - BnumI256::one()) * divisor)
                } else {
                    Self(self.0 / divisor * divisor)
                }
            }
            RoundingMode::TowardsZero => {
                if self.0 % divisor == BnumI256::zero() {
                    self.clone()
                } else {
                    Self(self.0 / divisor * divisor)
                }
            }
            RoundingMode::AwayFromZero => {
                if self.0 % divisor == BnumI256::zero() {
                    self.clone()
                } else if self.is_negative() {
                    Self((self.0 / divisor - BnumI256::one()) * divisor)
                } else {
                    Self((self.0 / divisor + BnumI256::one()) * divisor)
                }
            }
            RoundingMode::TowardsNearestAndHalfTowardsZero => {
                if self.0 % divisor == BnumI256::zero() {
                    self.clone()
                } else {
                    let digit = (self.0 / (divisor / BnumI256::from(10i128))
                        % BnumI256::from(10i128))
                    .abs();
                    if digit > 5.into() {
                        if self.is_negative() {
                            Self((self.0 / divisor - BnumI256::one()) * divisor)
                        } else {
                            Self((self.0 / divisor + BnumI256::one()) * divisor)
                        }
                    } else {
                        Self(self.0 / divisor * divisor)
                    }
                }
            }
            RoundingMode::TowardsNearestAndHalfAwayFromZero => {
                if self.0 % divisor == BnumI256::zero() {
                    self.clone()
                } else {
                    let digit = (self.0 / (divisor / BnumI256::from(10i128))
                        % BnumI256::from(10i128))
                    .abs();
                    if digit < 5.into() {
                        Self(self.0 / divisor * divisor)
                    } else {
                        if self.is_negative() {
                            Self((self.0 / divisor - BnumI256::one()) * divisor)
                        } else {
                            Self((self.0 / divisor + BnumI256::one()) * divisor)
                        }
                    }
                }
            }
        }
    }

    /// Calculates power using exponentiation by squaring".
    pub fn powi(&self, exp: i64) -> Self {
        let one_384 = BnumI384::from(Self::ONE.0);
        let base_384 = BnumI384::from(self.0);
        let div = |x: i64, y: i64| x.checked_div(y).expect("Overflow");
        let sub = |x: i64, y: i64| x.checked_sub(y).expect("Overflow");
        let mul = |x: i64, y: i64| x.checked_mul(y).expect("Overflow");

        if exp < 0 {
            let dec_256 = BnumI256::try_from(one_384 * one_384 / base_384).expect("Overflow");
            return Decimal(dec_256).powi(mul(exp, -1));
        }
        if exp == 0 {
            return Self::ONE;
        }
        if exp == 1 {
            return *self;
        }
        if exp % 2 == 0 {
            let dec_256 = BnumI256::try_from(base_384 * base_384 / one_384).expect("Overflow");
            Decimal(dec_256).powi(div(exp, 2))
        } else {
            let dec_256 = BnumI256::try_from(base_384 * base_384 / one_384).expect("Overflow");
            let sub_dec = Decimal(dec_256);
            *self * sub_dec.powi(div(sub(exp, 1), 2))
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

        // The BnumI256 i associated to a Decimal d is : i = d*10^18.
        // Therefore, taking sqrt yields sqrt(i) = sqrt(d)*10^9 => We lost precision
        // To get the right precision, we compute : sqrt(i*10^18) = sqrt(d)*10^18
        let self_384: BnumI384 = BnumI384::from(self.0);
        let correct_nb = self_384 * BnumI384::from(Decimal::one().0);
        let sqrt = BnumI256::try_from(correct_nb.sqrt()).expect("Overflow");
        Some(Decimal(sqrt))
    }

    /// Cubic root of a Decimal
    pub fn cbrt(&self) -> Self {
        if self.is_zero() {
            return Self::ZERO;
        }

        // By reasoning in the same way as before, we realise that we need to multiply by 10^36
        let self_384: BnumI384 = BnumI384::from(self.0);
        let correct_nb = self_384 * BnumI384::from(Decimal::one().0).pow(2);
        let cbrt = BnumI256::try_from(correct_nb.cbrt()).expect("Overflow");
        Decimal(cbrt)
    }

    /// Nth root of a Decimal
    pub fn nth_root(&self, n: u32) -> Option<Self> {
        if (self.is_negative() && n % 2 == 0) || n == 0 {
            None
        } else if n == 1 {
            Some(self.clone())
        } else {
            if self.is_zero() {
                return Some(Self::ZERO);
            }

            // By induction, we need to multiply by the (n-1)th power of 10^18.
            // To not overflow, we use BigInts
            let self_bigint = BigInt::from(self.0);
            let correct_nb = self_bigint * BigInt::from(Decimal::one().0).pow(n - 1);
            let nth_root = BnumI256::try_from(correct_nb.nth_root(n)).unwrap();
            Some(Decimal(nth_root))
        }
    }
}

macro_rules! from_int {
    ($type:ident) => {
        impl From<$type> for Decimal {
            fn from(val: $type) -> Self {
                Self(BnumI256::from(val) * Self::ONE.0)
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

impl<T: TryInto<Decimal>> Add<T> for Decimal
where
    <T as TryInto<Decimal>>::Error: fmt::Debug,
{
    type Output = Decimal;

    fn add(self, other: T) -> Self::Output {
        let a = self.0;
        let b_dec: Decimal = other.try_into().expect("Overflow");
        let b: BnumI256 = b_dec.0;
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
        let b: BnumI256 = b_dec.0;
        let c: BnumI256 = a - b;
        Decimal(c)
    }
}

impl<T: TryInto<Decimal>> Mul<T> for Decimal
where
    <T as TryInto<Decimal>>::Error: fmt::Debug,
{
    type Output = Decimal;

    fn mul(self, other: T) -> Self::Output {
        // Use BnumI384 (BInt<6>) to not overflow.
        let a = BnumI384::from(self.0);
        let b_dec: Decimal = other.try_into().expect("Overflow");
        let b = BnumI384::from(b_dec.0);
        let c = a * b / BnumI384::from(Self::ONE.0);
        let c_256 = BnumI256::try_from(c).expect("Overflow");
        Decimal(c_256)
    }
}

impl<T: TryInto<Decimal>> Div<T> for Decimal
where
    <T as TryInto<Decimal>>::Error: fmt::Debug,
{
    type Output = Decimal;

    fn div(self, other: T) -> Self::Output {
        // Use BnumI384 (BInt<6>) to not overflow.
        let a = BnumI384::from(self.0);
        let b_dec: Decimal = other.try_into().expect("Overflow");
        let b = BnumI384::from(b_dec.0);
        let c = a * BnumI384::from(Self::ONE.0) / b;
        let c_256 = BnumI256::try_from(c).expect("Overflow");
        Decimal(c_256)
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
            match BnumI256::try_from(slice) {
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
    DECIMAL_ID
);

manifest_type!(Decimal, ManifestCustomValueKind::Decimal, Decimal::BITS / 8);

//======
// text
//======

impl FromStr for Decimal {
    type Err = ParseDecimalError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tens = BnumI256::from(10);
        let v: Vec<&str> = s.split('.').collect();

        let mut int = match BnumI256::from_str(v[0]) {
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

            let frac = match BnumI256::from_str(v[1]) {
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
        const MULTIPLIER: BnumI256 = Decimal::ONE.0;
        let quotient = self.0 / MULTIPLIER;
        let remainder = self.0 % MULTIPLIER;

        if !remainder.is_zero() {
            // print remainder with leading zeroes
            let mut sign = "".to_string();

            // take care of sign in case quotient == zere and remainder < 0,
            // eg.
            //  self.0=-100000000000000000 -> -0.1
            if remainder < BnumI256::ZERO && quotient == BnumI256::ZERO {
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
        write!(f, "{}", self.to_string())
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
        let val_i512 = val.0 / BnumI512::from(10i8).pow(PreciseDecimal::SCALE - Decimal::SCALE);
        let result = BnumI256::try_from(val_i512);
        match result {
            Ok(val_i256) => Ok(Self(val_i256)),
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
                    match BnumI256::try_from(val) {
                        Ok(val) => {
                            match val.checked_mul(Self::ONE.0) {
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
try_from_integer!(BnumI256, BnumI512, BnumU256, BnumU512);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dec;
    use paste::paste;
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
        let _ = Decimal::MAX * dec!(1);
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
    fn test_powi_max_decimal() {
        let _max = Decimal::MAX.powi(1);
        let _max_sqrt = Decimal::MAX.sqrt().unwrap();
        let _max_cbrt = Decimal::MAX.cbrt();
        let _max_dec_2 = _max_sqrt.powi(2);
        let _max_dec_3 = _max_cbrt.powi(3);
    }

    #[test]
    fn test_div_decimal() {
        let a = Decimal::from(5u32);
        let b = Decimal::from(7u32);
        assert_eq!((a / b).to_string(), "0.714285714285714285");
        assert_eq!((b / a).to_string(), "1.4");
        let _ = Decimal::MAX / 1;
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
    fn test_encode_decimal_value_decimal() {
        let dec = dec!("0");
        let bytes = scrypto_encode(&dec).unwrap();
        assert_eq!(bytes, {
            let mut a = [0; 34];
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
        ("12345678.1234567890123456789012345678901234567890123456789012345678901234", "12345678.123456789012345678", 1),
        ("0.0000000000000000000000000008901234567890123456789012345678901234", "0", 2),
        ("-0.0000000000000000000000000008901234567890123456789012345678901234", "0", 3),
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
        (PreciseDecimal::from(Decimal::MAX) + 1, 3),
        (PreciseDecimal::from(Decimal::MIN) - 1, 4)
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
        (BnumI256::MAX, 1),
        (BnumI256::MIN, 2),
        // maximal Decimal integer part + 1
        (BnumI256::MAX/(BnumI256::from(10).pow(Decimal::SCALE)) + BnumI256::ONE, 3),
        // minimal Decimal integer part - 1
        (BnumI256::MIN/(BnumI256::from(10).pow(Decimal::SCALE)) - BnumI256::ONE, 4),
        (BnumU256::MAX, 5),
        (BnumI512::MAX, 6),
        (BnumI512::MIN, 7),
        (BnumU512::MAX, 8)
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
        (BnumI256::ONE, "1", 1),
        (-BnumI256::ONE, "-1", 2),
        // maximal Decimal integer part
        (BnumI256::MAX/(BnumI256::from(10_u64.pow(Decimal::SCALE))), "57896044618658097711785492504343953926634992332820282019728" , 3),
        // minimal Decimal integer part
        (BnumI256::MIN/(BnumI256::from(10_u64.pow(Decimal::SCALE))), "-57896044618658097711785492504343953926634992332820282019728" , 4),
        (BnumU256::MIN, "0", 5),
        (BnumU512::MIN, "0", 6),
        (BnumI512::ONE, "1", 7),
        (-BnumI512::ONE, "-1", 8)
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
            dec!("240615969168004511545033772477.625056927114980741")
        );
    }

    #[test]
    fn test_cbrt() {
        let cbrt_of_42 = dec!(42).cbrt();
        let cbrt_of_0 = dec!(0).cbrt();
        let cbrt_of_negative_42 = dec!("-42").cbrt();
        let cbrt_max = Decimal::MAX.cbrt();
        assert_eq!(cbrt_of_42, dec!("3.476026644886449786"));
        assert_eq!(cbrt_of_0, dec!("0"));
        assert_eq!(cbrt_of_negative_42, dec!("-3.476026644886449786"));
        assert_eq!(cbrt_max, dec!("38685626227668133590.597631999999999999"));
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
}
