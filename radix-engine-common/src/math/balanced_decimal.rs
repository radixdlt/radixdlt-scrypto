#[cfg(feature = "radix_engine_fuzzing")]
use arbitrary::Arbitrary;
use num_bigint::BigInt;
use num_traits::{Pow, Zero};
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
use crate::math::decimal::*;
use crate::math::rounding_mode::*;
use crate::math::PreciseDecimal;
use crate::well_known_scrypto_custom_type;
use crate::*;

/// `BalancedDecimal` represents a 256 bit representation of a fixed-scale decimal number.
///
/// The finite set of values are of the form `m / 10^38`, where `m` is
/// an integer such that `-2^(256 - 1) <= m < 2^(256 - 1)`.
///
/// Unless otherwise specified, all operations will panic if underflow/overflow.
#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BalancedDecimal(pub BnumI256);

impl Default for BalancedDecimal {
    fn default() -> Self {
        Self::zero()
    }
}

impl iter::Sum for BalancedDecimal {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut sum = BalancedDecimal::zero();
        iter.for_each(|d| {
            sum += d;
        });
        sum
    }
}

// TODO come up with some smarter formatting depending on Decimal::Scale
macro_rules! fmt_remainder {
    () => {
        "{:038}"
    };
}

impl BalancedDecimal {
    /// The min value of `BalancedDecimal`.
    pub const MIN: Self = Self(BnumI256::MIN);

    /// The max value of `BalancedDecimal`.
    pub const MAX: Self = Self(BnumI256::MAX);

    /// The bit length of number storing `BalancedDecimal`.
    pub const BITS: usize = BnumI256::BITS as usize;

    /// The fixed scale used by `BalancedDecimal`.
    pub const SCALE: u32 = 38;

    pub const ZERO: Self = Self(BnumI256::ZERO);

    pub const ONE: Self = Self(BnumI256::from_digits([
        687399551400673280,
        5421010862427522170,
        0,
        0,
    ]));

    /// Returns `BalancedDecimal` of 0.
    pub fn zero() -> Self {
        Self::ZERO
    }

    /// Returns `BalancedDecimal` of 1.
    pub fn one() -> Self {
        Self::ONE
    }

    /// Whether this decimal is zero.
    pub fn is_zero(&self) -> bool {
        self.0 == BnumI256::ZERO
    }

    /// Whether this decimal is positive.
    pub fn is_positive(&self) -> bool {
        self.0 > BnumI256::ZERO
    }

    /// Whether this decimal is negative.
    pub fn is_negative(&self) -> bool {
        self.0 < BnumI256::ZERO
    }

    /// Returns the absolute value.
    pub fn abs(&self) -> BalancedDecimal {
        BalancedDecimal(self.0.abs())
    }

    /// Returns the largest integer that is equal to or less than this number.
    pub fn floor(&self) -> Self {
        self.round(0, RoundingMode::ToNegativeInfinity)
    }

    /// Returns the smallest integer that is equal to or greater than this number.
    pub fn ceiling(&self) -> Self {
        self.round(0, RoundingMode::ToPositiveInfinity)
    }

    /// Rounds this number to the specified decimal places.
    ///
    /// # Panics
    /// - Panic if the number of decimal places is not within [0..SCALE]
    pub fn round<T: Into<i32>>(&self, decimal_places: T, mode: RoundingMode) -> Self {
        let decimal_places = decimal_places.into();
        assert!(decimal_places <= (Self::SCALE as i32));
        assert!(decimal_places >= 0);

        let n = Self::SCALE - (decimal_places as u32);
        let divisor: BnumI256 = BnumI256::TEN.pow(n);
        match mode {
            RoundingMode::ToPositiveInfinity => {
                if self.0 % divisor == BnumI256::ZERO {
                    self.clone()
                } else if self.is_negative() {
                    Self((self.0 / divisor) * divisor)
                } else {
                    Self((self.0 / divisor + BnumI256::ONE) * divisor)
                }
            }
            RoundingMode::ToNegativeInfinity => {
                if self.0 % divisor == BnumI256::ZERO {
                    self.clone()
                } else if self.is_negative() {
                    Self((self.0 / divisor - BnumI256::ONE) * divisor)
                } else {
                    Self((self.0 / divisor) * divisor)
                }
            }
            RoundingMode::ToZero => {
                if self.0 % divisor == BnumI256::ZERO {
                    self.clone()
                } else {
                    Self((self.0 / divisor) * divisor)
                }
            }
            RoundingMode::AwayFromZero => {
                if self.0 % divisor == BnumI256::ZERO {
                    self.clone()
                } else if self.is_negative() {
                    Self((self.0 / divisor - BnumI256::ONE) * divisor)
                } else {
                    Self((self.0 / divisor + BnumI256::ONE) * divisor)
                }
            }
            RoundingMode::ToNearestMidpointTowardZero => {
                let remainder = (self.0 % divisor).abs();
                if remainder == BnumI256::ZERO {
                    self.clone()
                } else {
                    let mid_point = divisor / BnumI256::from(2);
                    if remainder > mid_point {
                        if self.is_negative() {
                            Self((self.0 / divisor - BnumI256::ONE) * divisor)
                        } else {
                            Self((self.0 / divisor + BnumI256::ONE) * divisor)
                        }
                    } else {
                        Self((self.0 / divisor) * divisor)
                    }
                }
            }
            RoundingMode::ToNearestMidpointAwayFromZero => {
                let remainder = (self.0 % divisor).abs();
                if remainder == BnumI256::ZERO {
                    self.clone()
                } else {
                    let mid_point = divisor / BnumI256::from(2);
                    if remainder >= mid_point {
                        if self.is_negative() {
                            Self((self.0 / divisor - BnumI256::ONE) * divisor)
                        } else {
                            Self((self.0 / divisor + BnumI256::ONE) * divisor)
                        }
                    } else {
                        Self((self.0 / divisor) * divisor)
                    }
                }
            }
            RoundingMode::ToNearestMidpointToEven => {
                let remainder = (self.0 % divisor).abs();
                if remainder == BnumI256::ZERO {
                    self.clone()
                } else {
                    let mid_point = divisor / BnumI256::from(2);
                    if remainder > mid_point {
                        if self.is_negative() {
                            Self((self.0 / divisor - BnumI256::ONE) * divisor)
                        } else {
                            Self((self.0 / divisor + BnumI256::ONE) * divisor)
                        }
                    } else if remainder == mid_point {
                        if (self.0 / divisor) % BnumI256::from(2) == BnumI256::ZERO {
                            Self((self.0 / divisor) * divisor)
                        } else {
                            if self.is_negative() {
                                Self((self.0 / divisor - BnumI256::ONE) * divisor)
                            } else {
                                Self((self.0 / divisor + BnumI256::ONE) * divisor)
                            }
                        }
                    } else {
                        Self((self.0 / divisor) * divisor)
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
            let dec_256 = BnumI256::try_from((one_384 * one_384) / base_384).expect("Overflow");
            return BalancedDecimal(dec_256).powi(mul(exp, -1));
        }
        if exp == 0 {
            return Self::ONE;
        }
        if exp == 1 {
            return *self;
        }
        if exp % 2 == 0 {
            let dec_256 = BnumI256::try_from((base_384 * base_384) / one_384).expect("Overflow");
            BalancedDecimal(dec_256).powi(div(exp, 2))
        } else {
            let dec_256 = BnumI256::try_from((base_384 * base_384) / one_384).expect("Overflow");
            let sub_dec = BalancedDecimal(dec_256);
            *self * sub_dec.powi(div(sub(exp, 1), 2))
        }
    }

    /// Square root of a BalancedDecimal
    pub fn sqrt(&self) -> Option<Self> {
        if self.is_negative() {
            return None;
        }
        if self.is_zero() {
            return Some(Self::ZERO);
        }

        // The BnumI256 i associated to a Decimal d is : i = d*10^38.
        // Therefore, taking sqrt yields sqrt(i) = sqrt(d)*10^19 => We lost precision
        // To get the right precision, we compute : sqrt(i*10^38) = sqrt(d)*10^38
        let self_384: BnumI384 = BnumI384::from(self.0);
        let correct_nb = self_384 * BnumI384::from(BalancedDecimal::one().0);
        let sqrt = BnumI256::try_from(correct_nb.sqrt()).expect("Overflow");
        Some(BalancedDecimal(sqrt))
    }

    /// Cubic root of a BalancedDecimal
    pub fn cbrt(&self) -> Self {
        if self.is_zero() {
            return Self::ZERO;
        }

        // By reasoning in the same way as before, we realise that we need to multiply by 10^72
        let self_512: BnumI512 = BnumI512::from(self.0);
        let correct_nb = self_512 * BnumI512::from(BalancedDecimal::one().0).pow(2);
        let cbrt = BnumI256::try_from(correct_nb.cbrt()).expect("Overflow");
        BalancedDecimal(cbrt)
    }

    /// Nth root of a BalancedDecimal
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
            let correct_nb = self_bigint * BigInt::from(BalancedDecimal::one().0).pow(n - 1);
            let nth_root = BnumI256::try_from(correct_nb.nth_root(n)).unwrap();
            Some(BalancedDecimal(nth_root))
        }
    }
}

macro_rules! from_int {
    ($type:ident) => {
        impl From<$type> for BalancedDecimal {
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

// from_str() should be enough, but we want to have try_from() to simplify bdec! macro
impl TryFrom<&str> for BalancedDecimal {
    type Error = ParseBalancedDecimalError;

    fn try_from(val: &str) -> Result<Self, Self::Error> {
        Self::from_str(val)
    }
}

impl TryFrom<String> for BalancedDecimal {
    type Error = ParseBalancedDecimalError;

    fn try_from(val: String) -> Result<Self, Self::Error> {
        Self::from_str(&val)
    }
}

impl From<bool> for BalancedDecimal {
    fn from(val: bool) -> Self {
        if val {
            Self::from(1u8)
        } else {
            Self::from(0u8)
        }
    }
}

impl<T: TryInto<BalancedDecimal>> Add<T> for BalancedDecimal
where
    <T as TryInto<BalancedDecimal>>::Error: fmt::Debug,
{
    type Output = BalancedDecimal;

    fn add(self, other: T) -> Self::Output {
        let a = self.0;
        let b_dec: BalancedDecimal = other.try_into().expect("Overflow");
        let b: BnumI256 = b_dec.0;
        let c = a + b;
        BalancedDecimal(c)
    }
}

impl<T: TryInto<BalancedDecimal>> Sub<T> for BalancedDecimal
where
    <T as TryInto<BalancedDecimal>>::Error: fmt::Debug,
{
    type Output = BalancedDecimal;

    fn sub(self, other: T) -> Self::Output {
        let a = self.0;
        let b_dec: BalancedDecimal = other.try_into().expect("Overflow");
        let b: BnumI256 = b_dec.0;
        let c: BnumI256 = a - b;
        BalancedDecimal(c)
    }
}

impl<T: TryInto<BalancedDecimal>> Mul<T> for BalancedDecimal
where
    <T as TryInto<BalancedDecimal>>::Error: fmt::Debug,
{
    type Output = BalancedDecimal;

    fn mul(self, other: T) -> Self::Output {
        // Use BnumI384 (BInt<6>) to not overflow.
        let a = BnumI384::from(self.0);
        let b_dec: BalancedDecimal = other.try_into().expect("Overflow");
        let b = BnumI384::from(b_dec.0);
        let c = (a * b) / BnumI384::from(Self::ONE.0);
        let c_256 = BnumI256::try_from(c).expect("Overflow");
        BalancedDecimal(c_256)
    }
}

impl<T: TryInto<BalancedDecimal>> Div<T> for BalancedDecimal
where
    <T as TryInto<BalancedDecimal>>::Error: fmt::Debug,
{
    type Output = BalancedDecimal;

    fn div(self, other: T) -> Self::Output {
        // Use BnumI384 (BInt<6>) to not overflow.
        let a = BnumI384::from(self.0);
        let b_dec: BalancedDecimal = other.try_into().expect("Overflow");
        let b = BnumI384::from(b_dec.0);
        let c = (a * BnumI384::from(Self::ONE.0)) / b;
        let c_256 = BnumI256::try_from(c).expect("Overflow");
        BalancedDecimal(c_256)
    }
}

impl Neg for BalancedDecimal {
    type Output = BalancedDecimal;

    fn neg(self) -> Self::Output {
        BalancedDecimal(-self.0)
    }
}

impl<T: TryInto<BalancedDecimal>> AddAssign<T> for BalancedDecimal
where
    <T as TryInto<BalancedDecimal>>::Error: fmt::Debug,
{
    fn add_assign(&mut self, other: T) {
        let other: BalancedDecimal = other.try_into().expect("Overflow");
        self.0 += other.0;
    }
}

impl<T: TryInto<BalancedDecimal>> SubAssign<T> for BalancedDecimal
where
    <T as TryInto<BalancedDecimal>>::Error: fmt::Debug,
{
    fn sub_assign(&mut self, other: T) {
        let other: BalancedDecimal = other.try_into().expect("Overflow");
        self.0 -= other.0;
    }
}

impl<T: TryInto<BalancedDecimal>> MulAssign<T> for BalancedDecimal
where
    <T as TryInto<BalancedDecimal>>::Error: fmt::Debug,
{
    fn mul_assign(&mut self, other: T) {
        let other: BalancedDecimal = other.try_into().expect("Overflow");
        self.0 *= other.0;
    }
}

impl<T: TryInto<BalancedDecimal>> DivAssign<T> for BalancedDecimal
where
    <T as TryInto<BalancedDecimal>>::Error: fmt::Debug,
{
    fn div_assign(&mut self, other: T) {
        let other: BalancedDecimal = other.try_into().expect("Overflow");
        self.0 /= other.0;
    }
}

//========
// binary
//========

impl TryFrom<&[u8]> for BalancedDecimal {
    type Error = ParseBalancedDecimalError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        if slice.len() == Self::BITS / 8 {
            match BnumI256::try_from(slice) {
                Ok(val) => Ok(Self(val)),
                Err(_) => Err(ParseBalancedDecimalError::Overflow),
            }
        } else {
            Err(ParseBalancedDecimalError::InvalidLength(slice.len()))
        }
    }
}

impl BalancedDecimal {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_le_bytes().to_vec()
    }
}

well_known_scrypto_custom_type!(
    BalancedDecimal,
    ScryptoCustomValueKind::BalancedDecimal,
    Type::BalancedDecimal,
    BalancedDecimal::BITS / 8,
    BALANCED_DECIMAL_ID,
    balanced_decimal_type_data
);

manifest_type!(
    BalancedDecimal,
    ManifestCustomValueKind::BalancedDecimal,
    BalancedDecimal::BITS / 8
);

//======
// text
//======

impl FromStr for BalancedDecimal {
    type Err = ParseBalancedDecimalError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tens = BnumI256::from(10);
        let v: Vec<&str> = s.split('.').collect();

        let mut int = match BnumI256::from_str(v[0]) {
            Ok(val) => val,
            Err(_) => {
                return Err(ParseBalancedDecimalError::InvalidDigit);
            }
        };

        int *= tens.pow(Self::SCALE);

        if v.len() == 2 {
            let scale = (if let Some(scale) = Self::SCALE.checked_sub(v[1].len() as u32) {
                Ok(scale)
            } else {
                Err(Self::Err::UnsupportedDecimalPlace)
            })?;

            let frac = match BnumI256::from_str(v[1]) {
                Ok(val) => val,
                Err(_) => {
                    return Err(ParseBalancedDecimalError::InvalidDigit);
                }
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

impl fmt::Display for BalancedDecimal {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        const MULTIPLIER: BnumI256 = BalancedDecimal::ONE.0;
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

impl fmt::Debug for BalancedDecimal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

/// Represents an error when parsing BalancedDecimal from another type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseBalancedDecimalError {
    InvalidDecimal(String),
    InvalidChar(char),
    InvalidDigit,
    UnsupportedDecimalPlace,
    InvalidLength(usize),
    Overflow,
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseBalancedDecimalError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseBalancedDecimalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl TryFrom<Decimal> for BalancedDecimal {
    type Error = ParseBalancedDecimalError;

    fn try_from(val: Decimal) -> Result<Self, Self::Error> {
        let val_i384 = BnumI384::from(val.0)
            * BnumI384::from(10i8).pow(BalancedDecimal::SCALE - Decimal::SCALE);
        let result = BnumI256::try_from(val_i384);
        match result {
            Ok(val_i256) => Ok(Self(val_i256)),
            Err(_) => Err(ParseBalancedDecimalError::Overflow),
        }
    }
}

impl TryFrom<PreciseDecimal> for BalancedDecimal {
    type Error = ParseBalancedDecimalError;

    fn try_from(val: PreciseDecimal) -> Result<Self, Self::Error> {
        let val_i512 =
            val.0 / BnumI512::from(10i8).pow(PreciseDecimal::SCALE - BalancedDecimal::SCALE);
        let result = BnumI256::try_from(val_i512);
        match result {
            Ok(val_i256) => Ok(Self(val_i256)),
            Err(_) => Err(ParseBalancedDecimalError::Overflow),
        }
    }
}

macro_rules! try_from_integer {
    ($($t:ident),*) => {
        $(
            impl TryFrom<$t> for BalancedDecimal {
                type Error = ParseBalancedDecimalError;

                fn try_from(val: $t) -> Result<Self, Self::Error> {
                    match BnumI256::try_from(val) {
                        Ok(val) => {
                            match val.checked_mul(Self::ONE.0) {
                                Some(mul) => Ok(Self(mul)),
                                None => Err(ParseBalancedDecimalError::Overflow),
                            }
                        },
                        Err(_) => Err(ParseBalancedDecimalError::Overflow),
                    }
                }
            }
        )*
    };
}
try_from_integer!(BnumI256, BnumI512, BnumU256, BnumU512);

pub trait PrecisionRounding {
    fn floor_to_decimal(&self) -> Decimal;
    fn ceil_to_decimal(&self) -> Decimal;
}

impl PrecisionRounding for BalancedDecimal {
    fn floor_to_decimal(&self) -> Decimal {
        Decimal::from(self.round(Decimal::SCALE as i32, RoundingMode::ToNegativeInfinity))
    }

    fn ceil_to_decimal(&self) -> Decimal {
        Decimal::from(self.round(Decimal::SCALE as i32, RoundingMode::ToPositiveInfinity))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bdec;
    use paste::paste;
    use sbor::rust::vec;

    #[test]
    fn test_format_balanced_decimal() {
        assert_eq!(
            BalancedDecimal((1i128).into()).to_string(),
            "0.00000000000000000000000000000000000001"
        );
        assert_eq!(
            BalancedDecimal((123456789123456789i128).into()).to_string(),
            "0.00000000000000000000123456789123456789"
        );
        assert_eq!(
            BalancedDecimal((100000000000000000000000000000000000000i128).into()).to_string(),
            "1"
        );
        assert_eq!(
            BalancedDecimal(
                BnumI256::from(100000000000000000000000000000000000000i128)
                    .mul(BnumI256::from(123))
            )
            .to_string(),
            "123"
        );
        assert_eq!(
            BalancedDecimal(
                BnumI256::from(100000000000000000000000000000000000000i128)
                    .mul(BnumI256::from(123456789123456789i128))
            )
            .to_string(),
            "123456789123456789"
        );
        assert_eq!(
            BalancedDecimal::MAX.to_string(),
            "578960446186580977117854925043439539266.34992332820282019728792003956564819967"
        );
        assert_eq!(BalancedDecimal::MIN.is_negative(), true);
        assert_eq!(
            BalancedDecimal::MIN.to_string(),
            "-578960446186580977117854925043439539266.34992332820282019728792003956564819968"
        );
    }

    #[test]
    fn test_parse_balanced_decimal() {
        assert_eq!(
            BalancedDecimal::from_str("0.00000000000000000000000000000000000001").unwrap(),
            BalancedDecimal((1i128).into())
        );
        assert_eq!(
            BalancedDecimal::from_str("0.123456789123456789").unwrap(),
            BalancedDecimal((12345678912345678900000000000000000000i128).into())
        );
        assert_eq!(
            BalancedDecimal::from_str("1").unwrap(),
            BalancedDecimal((100000000000000000000000000000000000000i128).into())
        );
        assert_eq!(
            BalancedDecimal::from_str("123456789123456789").unwrap(),
            BalancedDecimal(BnumI256::from(123456789123456789i128).mul(BnumI256::from(10).pow(38)))
        );
        assert_eq!(
            BalancedDecimal::from_str(
                "578960446186580977117854925043439539266.34992332820282019728792003956564819967"
            )
            .unwrap(),
            BalancedDecimal::MAX
        );
        assert_eq!(
            BalancedDecimal::from_str(
                "-578960446186580977117854925043439539266.34992332820282019728792003956564819968"
            )
            .unwrap(),
            BalancedDecimal::MIN
        );
    }

    #[test]
    fn test_add_balanced_decimal() {
        let a = BalancedDecimal::from(5u32);
        let b = BalancedDecimal::from(7u32);
        assert_eq!((a + b).to_string(), "12");
    }

    #[test]
    #[should_panic(expected = "Overflow")]
    fn test_add_overflow_balanced_decimal() {
        let _ = BalancedDecimal::MAX + 1;
    }

    #[test]
    fn test_sub_balanced_decimal() {
        let a = BalancedDecimal::from(5u32);
        let b = BalancedDecimal::from(7u32);
        assert_eq!((a - b).to_string(), "-2");
        assert_eq!((b - a).to_string(), "2");
    }

    #[test]
    #[should_panic(expected = "Overflow")]
    fn test_sub_overflow_balanced_decimal() {
        let _ = BalancedDecimal::MIN - 1;
    }

    #[test]
    fn test_mul_balanced_decimal() {
        let a = BalancedDecimal::from(5u32);
        let b = BalancedDecimal::from(7u32);
        assert_eq!((a * b).to_string(), "35");
        let a = BalancedDecimal::from_str("1000000000").unwrap();
        let b = BalancedDecimal::from_str("1000000000").unwrap();
        assert_eq!((a * b).to_string(), "1000000000000000000");
        let _ = BalancedDecimal::MAX * bdec!(1);
    }

    #[test]
    #[should_panic(expected = "Overflow")]
    fn test_mul_overflow_by_small_balanced_decimal() {
        let _ = BalancedDecimal::MAX * bdec!("1.00000000000000000000000000000000000001");
    }

    #[test]
    #[should_panic(expected = "Overflow")]
    fn test_mul_overflow_by_a_lot_balanced_decimal() {
        let _ = BalancedDecimal::MAX * bdec!("1.1");
    }

    #[test]
    #[should_panic(expected = "Overflow")]
    fn test_mul_neg_overflow_balanced_decimal() {
        let _ = -BalancedDecimal::MAX * bdec!("-1.00000000000000000000000000000000000001");
    }

    #[test]
    #[should_panic]
    fn test_div_by_zero_balanced_decimal() {
        let a = BalancedDecimal::from(5u32);
        let b = BalancedDecimal::from(0u32);
        assert_eq!((a / b).to_string(), "0");
    }

    #[test]
    #[should_panic]
    fn test_powi_exp_overflow_balanced_decimal() {
        let a = BalancedDecimal::from(5u32);
        let b = i64::MIN;
        assert_eq!(a.powi(b).to_string(), "0");
    }

    #[test]
    fn test_1_powi_max_balanced_decimal() {
        let a = BalancedDecimal::from(1u32);
        let b = i64::MAX;
        assert_eq!(a.powi(b).to_string(), "1");
    }

    #[test]
    fn test_1_powi_min_balanced_decimal() {
        let a = BalancedDecimal::from(1u32);
        let b = i64::MAX - 1;
        assert_eq!(a.powi(b).to_string(), "1");
    }

    #[test]
    fn test_powi_max_balanced_decimal() {
        let _max = BalancedDecimal::MAX.powi(1);
        let _max_sqrt = BalancedDecimal::MAX.sqrt().unwrap();
        let _max_cbrt = BalancedDecimal::MAX.cbrt();
        let _max_dec_2 = _max_sqrt.powi(2);
        let _max_dec_3 = _max_cbrt.powi(3);
    }

    #[test]
    fn test_div_balanced_decimal() {
        let a = BalancedDecimal::from(5u32);
        let b = BalancedDecimal::from(7u32);
        assert_eq!(
            (a / b).to_string(),
            "0.71428571428571428571428571428571428571"
        );
        assert_eq!((b / a).to_string(), "1.4");
        let _ = BalancedDecimal::MAX / 1;
    }

    #[test]
    fn test_div_negative_balanced_decimal() {
        let a = BalancedDecimal::from(-42);
        let b = BalancedDecimal::from(2);
        assert_eq!((a / b).to_string(), "-21");
    }

    #[test]
    fn test_0_pow_0_balanced_decimal() {
        let a = bdec!("0");
        assert_eq!(a.powi(0).to_string(), "1");
    }

    #[test]
    fn test_0_powi_1_balanced_decimal() {
        let a = bdec!("0");
        assert_eq!(a.powi(1).to_string(), "0");
    }

    #[test]
    fn test_0_powi_10_balanced_decimal() {
        let a = bdec!("0");
        assert_eq!(a.powi(10).to_string(), "0");
    }

    #[test]
    fn test_1_powi_0_balanced_decimal() {
        let a = bdec!(1);
        assert_eq!(a.powi(0).to_string(), "1");
    }

    #[test]
    fn test_1_powi_1_balanced_decimal() {
        let a = bdec!(1);
        assert_eq!(a.powi(1).to_string(), "1");
    }

    #[test]
    fn test_1_powi_10_balanced_decimal() {
        let a = bdec!(1);
        assert_eq!(a.powi(10).to_string(), "1");
    }

    #[test]
    fn test_2_powi_0_balanced_decimal() {
        let a = bdec!("2");
        assert_eq!(a.powi(0).to_string(), "1");
    }

    #[test]
    fn test_2_powi_3724_balanced_decimal() {
        let a = bdec!("1.000234891009084238");
        assert_eq!(
            a.powi(3724).to_string(),
            "2.39799123225467486422227955915809986089"
        );
    }

    #[test]
    fn test_2_powi_2_balanced_decimal() {
        let a = bdec!("2");
        assert_eq!(a.powi(2).to_string(), "4");
    }

    #[test]
    fn test_2_powi_3_balanced_decimal() {
        let a = bdec!("2");
        assert_eq!(a.powi(3).to_string(), "8");
    }

    #[test]
    fn test_10_powi_3_balanced_decimal() {
        let a = bdec!("10");
        assert_eq!(a.powi(3).to_string(), "1000");
    }

    #[test]
    fn test_5_powi_2_balanced_decimal() {
        let a = bdec!("5");
        assert_eq!(a.powi(2).to_string(), "25");
    }

    #[test]
    fn test_5_powi_minus2_balanced_decimal() {
        let a = bdec!("5");
        assert_eq!(a.powi(-2).to_string(), "0.04");
    }

    #[test]
    fn test_10_powi_minus3_balanced_decimal() {
        let a = bdec!("10");
        assert_eq!(a.powi(-3).to_string(), "0.001");
    }

    #[test]
    fn test_minus10_powi_minus3_balanced_decimal() {
        let a = bdec!("-10");
        assert_eq!(a.powi(-3).to_string(), "-0.001");
    }

    #[test]
    fn test_minus10_powi_minus2_balanced_decimal() {
        let a = bdec!("-10");
        assert_eq!(a.powi(-2).to_string(), "0.01");
    }

    #[test]
    fn test_minus05_powi_minus2_balanced_decimal() {
        let a = bdec!("-0.5");
        assert_eq!(a.powi(-2).to_string(), "4");
    }
    #[test]
    fn test_minus05_powi_minus3_balanced_decimal() {
        let a = bdec!("-0.5");
        assert_eq!(a.powi(-3).to_string(), "-8");
    }

    #[test]
    fn test_10_powi_15_balanced_decimal() {
        let a = bdec!(10i128);
        assert_eq!(a.powi(15).to_string(), "1000000000000000");
    }

    #[test]
    #[should_panic]
    fn test_10_powi_16_balanced_decimal() {
        let a = BalancedDecimal((10i128).into());
        assert_eq!(a.powi(16).to_string(), "1000000000000000000000");
    }

    #[test]
    fn test_one_and_zero_balanced_decimal() {
        assert_eq!(BalancedDecimal::one().to_string(), "1");
        assert_eq!(BalancedDecimal::zero().to_string(), "0");
    }

    #[test]
    fn test_dec_string_decimal_balanced_decimal() {
        assert_eq!(
            bdec!("1.123456789012345678").to_string(),
            "1.123456789012345678"
        );
        assert_eq!(bdec!("-5.6").to_string(), "-5.6");
    }

    #[test]
    fn test_dec_string_balanced_decimal() {
        assert_eq!(bdec!(1).to_string(), "1");
        assert_eq!(bdec!("0").to_string(), "0");
    }

    #[test]
    fn test_dec_int_balanced_decimal() {
        assert_eq!(bdec!(1).to_string(), "1");
        assert_eq!(bdec!(5).to_string(), "5");
    }

    #[test]
    fn test_dec_bool_balanced_decimal() {
        assert_eq!(bdec!(false).to_string(), "0");
    }

    #[test]
    fn test_dec_rational_balanced_decimal() {
        assert_eq!(bdec!(11235, 0).to_string(), "11235");
        assert_eq!(bdec!(11235, -2).to_string(), "112.35");
        assert_eq!(bdec!(11235, 2).to_string(), "1123500");

        //    assert_eq!(
        //       bdec!("11200000000000000000000000000000000000001", -38).to_string(),
        //       "112.00000000000000000000000000000000000001"
        //    );
    }

    #[test]
    #[should_panic(expected = "Shift overflow")]
    fn test_shift_overflow_balanced_decimal() {
        // u32::MAX + 1
        bdec!(1, 4_294_967_296i128); // use explicit type to defer error to runtime
    }

    #[test]
    fn test_floor_balanced_decimal() {
        assert_eq!(
            BalancedDecimal::MAX.floor().to_string(),
            "578960446186580977117854925043439539266"
        );
        assert_eq!(bdec!("1.2").floor().to_string(), "1");
        assert_eq!(bdec!("1.0").floor().to_string(), "1");
        assert_eq!(bdec!("0.9").floor().to_string(), "0");
        assert_eq!(bdec!("0").floor().to_string(), "0");
        assert_eq!(bdec!("-0.1").floor().to_string(), "-1");
        assert_eq!(bdec!("-1").floor().to_string(), "-1");
        assert_eq!(bdec!("-5.2").floor().to_string(), "-6");
    }

    #[test]
    #[should_panic]
    fn test_floor_overflow_balanced_decimal() {
        BalancedDecimal::MIN.floor();
    }

    #[test]
    fn test_ceiling_balanced_decimal() {
        assert_eq!(bdec!("1.2").ceiling().to_string(), "2");
        assert_eq!(bdec!("1.0").ceiling().to_string(), "1");
        assert_eq!(bdec!("0.9").ceiling().to_string(), "1");
        assert_eq!(bdec!("0").ceiling().to_string(), "0");
        assert_eq!(bdec!("-0.1").ceiling().to_string(), "0");
        assert_eq!(bdec!("-1").ceiling().to_string(), "-1");
        assert_eq!(bdec!("-5.2").ceiling().to_string(), "-5");
        assert_eq!(
            BalancedDecimal::MIN.ceiling().to_string(),
            "-578960446186580977117854925043439539266"
        );
    }

    #[test]
    #[should_panic]
    fn test_ceiling_overflow_balanced_decimal() {
        BalancedDecimal::MAX.ceiling();
    }

    #[test]
    fn test_floor_to_decimal_balanced_decimal() {
        assert_eq!(
            BalancedDecimal::MAX.floor_to_decimal().to_string(),
            "578960446186580977117854925043439539266.349923328202820197"
        );
        assert_eq!(bdec!("1.0000000000000000012").floor_to_decimal().to_string(), "1.000000000000000001");
        assert_eq!(bdec!("1.0").floor_to_decimal().to_string(), "1");
        assert_eq!(bdec!("0.0000000000000000009").floor_to_decimal().to_string(), "0");
        assert_eq!(bdec!("0").floor_to_decimal().to_string(), "0");
        assert_eq!(bdec!("-0.0000000000000000002").floor_to_decimal().to_string(), "-0.000000000000000001");
        assert_eq!(bdec!("-1").floor_to_decimal().to_string(), "-1");
        assert_eq!(bdec!("-5.0000000000000000057").floor_to_decimal().to_string(), "-5.000000000000000006");
    }

    #[test]
    #[should_panic]
    fn test_floor_to_decimal_overflow_balanced_decimal() {
        BalancedDecimal::MIN.floor_to_decimal();
    }

    #[test]
    fn test_ceil_to_decimal_balanced_decimal() {
        assert_eq!(bdec!("1.0000000000000000012").ceil_to_decimal().to_string(), "1.000000000000000002");
        assert_eq!(bdec!("1.0").ceil_to_decimal().to_string(), "1");
        assert_eq!(bdec!("0.0000000000000000009").ceil_to_decimal().to_string(), "0.000000000000000001");
        assert_eq!(bdec!("0").ceil_to_decimal().to_string(), "0");
        assert_eq!(bdec!("-0.0000000000000000016").ceil_to_decimal().to_string(), "-0.000000000000000001");
        assert_eq!(bdec!("-1").ceil_to_decimal().to_string(), "-1");
        assert_eq!(bdec!("-5.0000000000000000052").ceil_to_decimal().to_string(), "-5.000000000000000005");
        assert_eq!(
            BalancedDecimal::MIN.ceil_to_decimal().to_string(),
            "-578960446186580977117854925043439539266.349923328202820197"
        );
    }

    #[test]
    #[should_panic]
    fn test_ceil_to_decimal_overflow_balanced_decimal() {
        BalancedDecimal::MAX.ceil_to_decimal();
    }

    #[test]
    fn test_rounding_to_zero_balanced_decimal() {
        let mode = RoundingMode::ToZero;
        assert_eq!(bdec!("1.2").round(0, mode).to_string(), "1");
        assert_eq!(bdec!("1.0").round(0, mode).to_string(), "1");
        assert_eq!(bdec!("0.9").round(0, mode).to_string(), "0");
        assert_eq!(bdec!("0").round(0, mode).to_string(), "0");
        assert_eq!(bdec!("-0.1").round(0, mode).to_string(), "0");
        assert_eq!(bdec!("-1").round(0, mode).to_string(), "-1");
        assert_eq!(bdec!("-5.2").round(0, mode).to_string(), "-5");
    }

    #[test]
    fn test_rounding_away_from_zero_balanced_decimal() {
        let mode = RoundingMode::AwayFromZero;
        assert_eq!(bdec!("1.2").round(0, mode).to_string(), "2");
        assert_eq!(bdec!("1.0").round(0, mode).to_string(), "1");
        assert_eq!(bdec!("0.9").round(0, mode).to_string(), "1");
        assert_eq!(bdec!("0").round(0, mode).to_string(), "0");
        assert_eq!(bdec!("-0.1").round(0, mode).to_string(), "-1");
        assert_eq!(bdec!("-1").round(0, mode).to_string(), "-1");
        assert_eq!(bdec!("-5.2").round(0, mode).to_string(), "-6");
    }

    #[test]
    fn test_rounding_midpoint_toward_zero_balanced_decimal() {
        let mode = RoundingMode::ToNearestMidpointTowardZero;
        assert_eq!(bdec!("5.5").round(0, mode).to_string(), "5");
        assert_eq!(bdec!("2.5").round(0, mode).to_string(), "2");
        assert_eq!(bdec!("1.6").round(0, mode).to_string(), "2");
        assert_eq!(bdec!("1.1").round(0, mode).to_string(), "1");
        assert_eq!(bdec!("1.0").round(0, mode).to_string(), "1");
        assert_eq!(bdec!("-1.0").round(0, mode).to_string(), "-1");
        assert_eq!(bdec!("-1.1").round(0, mode).to_string(), "-1");
        assert_eq!(bdec!("-1.6").round(0, mode).to_string(), "-2");
        assert_eq!(bdec!("-2.5").round(0, mode).to_string(), "-2");
        assert_eq!(bdec!("-5.5").round(0, mode).to_string(), "-5");
    }

    #[test]
    fn test_rounding_midpoint_away_from_zero_balanced_decimal() {
        let mode = RoundingMode::ToNearestMidpointAwayFromZero;
        assert_eq!(bdec!("5.5").round(0, mode).to_string(), "6");
        assert_eq!(bdec!("2.5").round(0, mode).to_string(), "3");
        assert_eq!(bdec!("1.6").round(0, mode).to_string(), "2");
        assert_eq!(bdec!("1.1").round(0, mode).to_string(), "1");
        assert_eq!(bdec!("1.0").round(0, mode).to_string(), "1");
        assert_eq!(bdec!("-1.0").round(0, mode).to_string(), "-1");
        assert_eq!(bdec!("-1.1").round(0, mode).to_string(), "-1");
        assert_eq!(bdec!("-1.6").round(0, mode).to_string(), "-2");
        assert_eq!(bdec!("-2.5").round(0, mode).to_string(), "-3");
        assert_eq!(bdec!("-5.5").round(0, mode).to_string(), "-6");
    }

    #[test]
    fn test_rounding_midpoint_nearest_even_zero_balanced_decimal() {
        let mode = RoundingMode::ToNearestMidpointToEven;
        assert_eq!(bdec!("5.5").round(0, mode).to_string(), "6");
        assert_eq!(bdec!("2.5").round(0, mode).to_string(), "2");
        assert_eq!(bdec!("1.6").round(0, mode).to_string(), "2");
        assert_eq!(bdec!("1.1").round(0, mode).to_string(), "1");
        assert_eq!(bdec!("1.0").round(0, mode).to_string(), "1");
        assert_eq!(bdec!("-1.0").round(0, mode).to_string(), "-1");
        assert_eq!(bdec!("-1.1").round(0, mode).to_string(), "-1");
        assert_eq!(bdec!("-1.6").round(0, mode).to_string(), "-2");
        assert_eq!(bdec!("-2.5").round(0, mode).to_string(), "-2");
        assert_eq!(bdec!("-5.5").round(0, mode).to_string(), "-6");
    }

    #[test]
    fn test_various_decimal_places_balanced_decimal() {
        let num = bdec!("2.4595");
        let mode = RoundingMode::AwayFromZero;
        assert_eq!(num.round(0, mode).to_string(), "3");
        assert_eq!(num.round(1, mode).to_string(), "2.5");
        assert_eq!(num.round(2, mode).to_string(), "2.46");
        assert_eq!(num.round(3, mode).to_string(), "2.46");
        let mode = RoundingMode::ToZero;
        assert_eq!(num.round(0, mode).to_string(), "2");
        assert_eq!(num.round(1, mode).to_string(), "2.4");
        assert_eq!(num.round(2, mode).to_string(), "2.45");
        assert_eq!(num.round(3, mode).to_string(), "2.459");
        let mode = RoundingMode::ToPositiveInfinity;
        assert_eq!(num.round(0, mode).to_string(), "3");
        assert_eq!(num.round(1, mode).to_string(), "2.5");
        assert_eq!(num.round(2, mode).to_string(), "2.46");
        assert_eq!(num.round(3, mode).to_string(), "2.46");
        let mode = RoundingMode::ToNegativeInfinity;
        assert_eq!(num.round(0, mode).to_string(), "2");
        assert_eq!(num.round(1, mode).to_string(), "2.4");
        assert_eq!(num.round(2, mode).to_string(), "2.45");
        assert_eq!(num.round(3, mode).to_string(), "2.459");
        let mode = RoundingMode::ToNearestMidpointAwayFromZero;
        assert_eq!(num.round(0, mode).to_string(), "2");
        assert_eq!(num.round(1, mode).to_string(), "2.5");
        assert_eq!(num.round(2, mode).to_string(), "2.46");
        assert_eq!(num.round(3, mode).to_string(), "2.46");
        let mode = RoundingMode::ToNearestMidpointTowardZero;
        assert_eq!(num.round(0, mode).to_string(), "2");
        assert_eq!(num.round(1, mode).to_string(), "2.5");
        assert_eq!(num.round(2, mode).to_string(), "2.46");
        assert_eq!(num.round(3, mode).to_string(), "2.459");
        let mode = RoundingMode::ToNearestMidpointToEven;
        assert_eq!(num.round(0, mode).to_string(), "2");
        assert_eq!(num.round(1, mode).to_string(), "2.5");
        assert_eq!(num.round(2, mode).to_string(), "2.46");
        assert_eq!(num.round(3, mode).to_string(), "2.46");

        let num = bdec!("-2.4595");
        let mode = RoundingMode::AwayFromZero;
        assert_eq!(num.round(0, mode).to_string(), "-3");
        assert_eq!(num.round(1, mode).to_string(), "-2.5");
        assert_eq!(num.round(2, mode).to_string(), "-2.46");
        assert_eq!(num.round(3, mode).to_string(), "-2.46");
        let mode = RoundingMode::ToZero;
        assert_eq!(num.round(0, mode).to_string(), "-2");
        assert_eq!(num.round(1, mode).to_string(), "-2.4");
        assert_eq!(num.round(2, mode).to_string(), "-2.45");
        assert_eq!(num.round(3, mode).to_string(), "-2.459");
        let mode = RoundingMode::ToPositiveInfinity;
        assert_eq!(num.round(0, mode).to_string(), "-2");
        assert_eq!(num.round(1, mode).to_string(), "-2.4");
        assert_eq!(num.round(2, mode).to_string(), "-2.45");
        assert_eq!(num.round(3, mode).to_string(), "-2.459");
        let mode = RoundingMode::ToNegativeInfinity;
        assert_eq!(num.round(0, mode).to_string(), "-3");
        assert_eq!(num.round(1, mode).to_string(), "-2.5");
        assert_eq!(num.round(2, mode).to_string(), "-2.46");
        assert_eq!(num.round(3, mode).to_string(), "-2.46");
        let mode = RoundingMode::ToNearestMidpointAwayFromZero;
        assert_eq!(num.round(0, mode).to_string(), "-2");
        assert_eq!(num.round(1, mode).to_string(), "-2.5");
        assert_eq!(num.round(2, mode).to_string(), "-2.46");
        assert_eq!(num.round(3, mode).to_string(), "-2.46");
        let mode = RoundingMode::ToNearestMidpointTowardZero;
        assert_eq!(num.round(0, mode).to_string(), "-2");
        assert_eq!(num.round(1, mode).to_string(), "-2.5");
        assert_eq!(num.round(2, mode).to_string(), "-2.46");
        assert_eq!(num.round(3, mode).to_string(), "-2.459");
        let mode = RoundingMode::ToNearestMidpointToEven;
        assert_eq!(num.round(0, mode).to_string(), "-2");
        assert_eq!(num.round(1, mode).to_string(), "-2.5");
        assert_eq!(num.round(2, mode).to_string(), "-2.46");
        assert_eq!(num.round(3, mode).to_string(), "-2.46");
    }

    #[test]
    fn test_sum_balanced_decimal() {
        let decimals = vec![bdec!(1), bdec!("2"), bdec!("3")];
        // two syntax
        let sum1: BalancedDecimal = decimals.iter().copied().sum();
        let sum2: BalancedDecimal = decimals.into_iter().sum();
        assert_eq!(sum1, bdec!("6"));
        assert_eq!(sum2, bdec!("6"));
    }

    #[test]
    fn test_encode_decimal_value_balanced_decimal() {
        let dec = bdec!("0");
        let bytes = scrypto_encode(&dec).unwrap();
        assert_eq!(bytes, {
            let mut a = [0; 34];
            a[0] = SCRYPTO_SBOR_V1_PAYLOAD_PREFIX;
            a[1] = ScryptoValueKind::Custom(ScryptoCustomValueKind::BalancedDecimal).as_u8();
            a
        });
    }

    #[test]
    fn test_decode_decimal_value_balanced_decimal() {
        let dec = bdec!("1.23456789");
        let bytes = scrypto_encode(&dec).unwrap();
        let decoded: BalancedDecimal = scrypto_decode(&bytes).unwrap();
        assert_eq!(decoded, bdec!("1.23456789"));
    }

    #[test]
    fn test_from_str_balanced_decimal() {
        let dec = BalancedDecimal::from_str("5.0").unwrap();
        assert_eq!(dec.to_string(), "5");
    }

    #[test]
    fn test_from_str_failure_balanced_decimal() {
        let dec = BalancedDecimal::from_str("non_decimal_value");
        assert_eq!(dec, Err(ParseBalancedDecimalError::InvalidDigit));
    }

    macro_rules! test_from_into_decimal_balanced_decimal {
        ($(($from:expr, $expected:expr, $suffix:expr)),*) => {
            paste!{
            $(
                #[test]
                fn [<test_from_into_decimal_balanced_decimal_ $suffix>]() {
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

    test_from_into_decimal_balanced_decimal!(
        (
            "12345678.123456789012345678",
            "12345678.123456789012345678",
            1
        ),
        ("0.000000000000000001", "0.000000000000000001", 2),
        ("-0.000000000000000001", "-0.000000000000000001", 3),
        ("5", "5", 4),
        ("12345678.1", "12345678.1", 5),
        (
            "578960446186580977117854925043439539266.349923328202820197",
            "578960446186580977117854925043439539266.349923328202820197",
            6
        ),
        (
            "-578960446186580977117854925043439539266.349923328202820197",
            "-578960446186580977117854925043439539266.349923328202820197",
            7
        )
    );

    macro_rules! test_from_decimal_balanced_decimal_overflow {
        ($(($from:expr, $suffix:expr)),*) => {
            paste!{
            $(
                #[test]
                fn [<test_from_balanced_decimal_decimal_overflow_ $suffix>]() {
                    let err = BalancedDecimal::try_from($from).unwrap_err();
                    assert_eq!(err, ParseBalancedDecimalError::Overflow);
                }
            )*
            }
        };
    }

    test_from_decimal_balanced_decimal_overflow!(
        (Decimal::MAX, 1),
        (Decimal::MIN, 2),
        (Decimal::from(BalancedDecimal::MAX) + 1, 3),
        (Decimal::from(BalancedDecimal::MIN) - 1, 4)
    );

    macro_rules! test_from_into_precise_decimal_balanced_decimal {
        ($(($from:expr, $expected:expr, $suffix:expr)),*) => {
            paste!{
            $(
                #[test]
                fn [<test_from_into_precise_decimal_decimal_ $suffix>]() {
                    let pdec = PreciseDecimal::try_from($from).unwrap();
                    println!("{:?}", BalancedDecimal::try_from(pdec));
                    let bdec = BalancedDecimal::try_from(pdec).unwrap();
                    assert_eq!(bdec.to_string(), $expected);

                    let bdec: BalancedDecimal = pdec.try_into().unwrap();
                    assert_eq!(bdec.to_string(), $expected);
                }
            )*
            }
        };
    }

    test_from_into_precise_decimal_balanced_decimal!(
        (
            "12345678.1234567890123456789012345678901234567890123456789012345678901234",
            "12345678.12345678901234567890123456789012345678",
            1
        ),
        (
            "0.0000000000000000000000000000000000000089012345678901234567890123",
            "0",
            2
        ),
        (
            "-0.0000000000000000000000000000000000000089012345678901234567890123",
            "0",
            3
        ),
        ("5", "5", 4),
        ("-12345678.1", "-12345678.1", 5),
        (
            "578960446186580977117854925043439539266.34992332820282019728792003956564819967",
            "578960446186580977117854925043439539266.34992332820282019728792003956564819967",
            6
        ),
        (
            "-578960446186580977117854925043439539266.34992332820282019728792003956564819967",
            "-578960446186580977117854925043439539266.34992332820282019728792003956564819967",
            7
        )
    );

    macro_rules! test_from_precise_decimal_balanced_decimal_overflow {
        ($(($from:expr, $suffix:expr)),*) => {
            paste!{
            $(
                #[test]
                fn [<test_from_precise_decimal_balanced_decimal_overflow_ $suffix>]() {
                    let err = BalancedDecimal::try_from($from).unwrap_err();
                    assert_eq!(err, ParseBalancedDecimalError::Overflow);
                }
            )*
            }
        };
    }

    test_from_precise_decimal_balanced_decimal_overflow!(
        (PreciseDecimal::MAX, 1),
        (PreciseDecimal::MIN, 2),
        (PreciseDecimal::from(BalancedDecimal::MAX) + 1, 3),
        (PreciseDecimal::from(BalancedDecimal::MIN), 4)
    );

    macro_rules! test_try_from_integer_overflow {
        ($(($from:expr, $suffix:expr)),*) => {
            paste!{
            $(
                #[test]
                fn [<test_try_from_integer_overflow_ $suffix>]() {
                    let err = BalancedDecimal::try_from($from).unwrap_err();
                    assert_eq!(err, ParseBalancedDecimalError::Overflow)
                }
            )*
            }
        };
    }

    test_try_from_integer_overflow!(
        (BnumI256::MAX, 1),
        (BnumI256::MIN, 2),
        // maximal BalancedDecimal integer part + 1
        (
            BnumI256::MAX / BnumI256::from(10).pow(BalancedDecimal::SCALE) + BnumI256::ONE,
            3
        ),
        // minimal BalancedDecimal integer part - 1
        (
            BnumI256::MIN / BnumI256::from(10).pow(BalancedDecimal::SCALE) - BnumI256::ONE,
            4
        ),
        (BnumU256::MAX, 5),
        (BnumI512::MAX, 6),
        (BnumI512::MIN, 7),
        (BnumU512::MAX, 8)
    );

    macro_rules! test_try_from_integer {
        ($(($from:expr, $expected:expr, $suffix:expr)),*) => {
            paste!{
            $(
                #[test]
                fn [<test_try_from_integer_ $suffix>]() {
                    let dec = BalancedDecimal::try_from($from).unwrap();
                    assert_eq!(dec.to_string(), $expected)
                }
            )*
            }
        };
    }

    test_try_from_integer!(
        (BnumI256::ONE, "1", 1),
        (-BnumI256::ONE, "-1", 2),
        // maximal BalancedDecimal integer part
        (
            BnumI256::MAX / BnumI256::from(10_u64).pow(BalancedDecimal::SCALE),
            "578960446186580977117854925043439539266",
            3
        ),
        // minimal BalancedDecimal integer part
        (
            BnumI256::MIN / BnumI256::from(10_u64).pow(BalancedDecimal::SCALE),
            "-578960446186580977117854925043439539266",
            4
        ),
        (BnumU256::MIN, "0", 5),
        (BnumU512::MIN, "0", 6),
        (BnumI512::ONE, "1", 7),
        (-BnumI512::ONE, "-1", 8)
    );

    #[test]
    fn test_sqrt() {
        let sqrt_of_42 = bdec!(42).sqrt();
        let sqrt_of_0 = bdec!(0).sqrt();
        let sqrt_of_negative = bdec!("-1").sqrt();
        let sqrt_max = BalancedDecimal::MAX.sqrt();
        assert_eq!(
            sqrt_of_42.unwrap(),
            bdec!("6.48074069840786023096596743608799665770")
        );
        assert_eq!(sqrt_of_0.unwrap(), bdec!(0));
        assert_eq!(sqrt_of_negative, None);
        assert_eq!(
            sqrt_max.unwrap(),
            bdec!("24061596916800451154.5033772477625056927114980741063148377")
        );
    }

    #[test]
    fn test_cbrt() {
        let cbrt_of_42 = bdec!(42).cbrt();
        let cbrt_of_0 = bdec!(0).cbrt();
        let cbrt_of_negative_42 = bdec!("-42").cbrt();
        let cbrt_max = BalancedDecimal::MAX.cbrt();
        assert_eq!(cbrt_of_42, bdec!("3.476026644886449786739865219004537434"));
        assert_eq!(cbrt_of_0, bdec!("0"));
        assert_eq!(
            cbrt_of_negative_42,
            bdec!("-3.476026644886449786739865219004537434")
        );
        assert_eq!(
            cbrt_max,
            bdec!("8334565515049.55065578647965760880872812752814461188")
        );
    }

    #[test]
    fn test_nth_root() {
        let root_4_42 = bdec!(42).nth_root(4);
        let root_5_42 = bdec!(42).nth_root(5);
        let root_42_42 = bdec!(42).nth_root(42);
        let root_neg_4_42 = bdec!("-42").nth_root(4);
        let root_neg_5_42 = bdec!("-42").nth_root(5);
        let root_0 = bdec!(42).nth_root(0);
        assert_eq!(
            root_4_42.unwrap(),
            bdec!("2.54572989502183051826978896057628868519")
        );
        assert_eq!(
            root_5_42.unwrap(),
            bdec!("2.11178576496675391273256733055023348630")
        );
        assert_eq!(
            root_42_42.unwrap(),
            bdec!("1.09307205793482361868278473185562578624")
        );
        assert_eq!(root_neg_4_42, None);
        assert_eq!(
            root_neg_5_42.unwrap(),
            bdec!("-2.11178576496675391273256733055023348630")
        );
        assert_eq!(root_0, None);
    }

    #[test]
    fn no_panic_with_38_decimal_places() {
        // Arrange
        let string = "1.11111111111111111111111111111111111111";

        // Act
        let decimal = BalancedDecimal::from_str(string);

        // Assert
        assert!(decimal.is_ok())
    }

    #[test]
    fn no_panic_with_39_decimal_places() {
        // Arrange
        let string = "1.111111111111111111111111111111111111111";

        // Act
        let decimal = BalancedDecimal::from_str(string);

        // Assert
        assert!(matches!(
            decimal,
            Err(ParseBalancedDecimalError::UnsupportedDecimalPlace)
        ))
    }
}
