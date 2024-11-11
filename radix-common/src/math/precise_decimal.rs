use crate::internal_prelude::*;
#[cfg(feature = "fuzzing")]
use arbitrary::Arbitrary;
use core::cmp::Ordering;
use core::ops::*;
use num_bigint::BigInt;
use num_traits::{Pow, Zero};

use crate::data::manifest::ManifestCustomValueKind;
use crate::data::scrypto::*;
use crate::math::bnum_integer::*;
use crate::math::decimal::*;
use crate::math::rounding_mode::*;
use crate::math::traits::*;
use crate::well_known_scrypto_custom_type;
use crate::*;

/// `PreciseDecimal` represents a 256 bit representation of a fixed-scale decimal number.
///
/// The finite set of values are of the form `m / 10^36`, where `m` is
/// an integer such that `-2^(256 - 1) <= m < 2^(256 - 1)`.
///
/// ```text
/// Fractional part: ~120 bits / 36 digits
/// Integer part   : 136 bits / 41 digits
/// Max            :  57896044618658097711785492504343953926634.992332820282019728792003956564819967
/// Min            : -57896044618658097711785492504343953926634.992332820282019728792003956564819968
/// ```
///
/// Unless otherwise specified, all operations will panic if there is underflow/overflow.
///
/// To create a PreciseDecimal with a certain number of precise `10^(-36)` subunits,
/// use [`PreciseDecimal::from_precise_subunits`].
#[cfg_attr(feature = "fuzzing", derive(Arbitrary))]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PreciseDecimal(InnerPreciseDecimal);

pub type InnerPreciseDecimal = I256;

impl Default for PreciseDecimal {
    fn default() -> Self {
        Self::zero()
    }
}

// TODO come up with some smarter formatting depending on PreciseDecimal::Scale
macro_rules! fmt_remainder {
    () => {
        "{:036}"
    };
}

impl PreciseDecimal {
    /// The min value of `PreciseDecimal`.
    pub const MIN: Self = Self(I256::MIN);

    /// The max value of `PreciseDecimal`.
    pub const MAX: Self = Self(I256::MAX);

    /// The bit length of number storing `PreciseDecimal`.
    pub const BITS: usize = I256::BITS as usize;

    /// The fixed scale used by `PreciseDecimal`.
    pub const SCALE: u32 = 36;

    pub const ZERO: Self = Self(I256::ZERO);

    pub const ONE_PRECISE_SUBUNIT: Self = Self(I256::ONE);
    // Possibly we should have `ONE_SUBUNIT == ONE_ATTO`, but I don't want to confuse
    // users who may think `ONE_SUBUNIT == ONE_PRECISE_SUBUNIT`.
    pub const ONE_ATTO: Self = Self(I256::from_digits([1000000000000000000, 0, 0, 0]));
    pub const ONE_HUNDREDTH: Self = Self(I256::from_digits([
        4003012203950112768,
        542101086242752,
        0,
        0,
    ]));
    pub const ONE_TENTH: Self = Self(I256::from_digits([
        3136633892082024448,
        5421010862427522,
        0,
        0,
    ]));
    pub const ONE: Self = Self(I256::from_digits([
        12919594847110692864,
        54210108624275221,
        0,
        0,
    ]));
    pub const TEN: Self = Self(I256::from_digits([
        68739955140067328,
        542101086242752217,
        0,
        0,
    ]));
    pub const ONE_HUNDRED: Self = Self(I256::from_digits([
        687399551400673280,
        5421010862427522170,
        0,
        0,
    ]));

    /// Constructs a [`PreciseDecimal`] from its underlying `10^(-36)` subunits.
    pub const fn from_precise_subunits(attos: I256) -> Self {
        Self(attos)
    }

    /// Returns the underlying `10^(-36)` subunits of the [`PreciseDecimal`].
    pub const fn precise_subunits(self) -> I256 {
        self.0
    }

    /// Returns a [`PreciseDecimal`] with value 0.
    pub const fn zero() -> Self {
        Self::ZERO
    }

    /// Returns a [`PreciseDecimal`] with value 1.
    pub const fn one() -> Self {
        Self::ONE
    }

    /// Whether the value is zero.
    pub fn is_zero(&self) -> bool {
        self.0 == I256::zero()
    }

    /// Whether the value is positive.
    pub fn is_positive(&self) -> bool {
        self.0 > I256::zero()
    }

    /// Whether the value is negative.
    pub fn is_negative(&self) -> bool {
        self.0 < I256::zero()
    }

    /// Returns the absolute value.
    pub fn checked_abs(&self) -> Option<Self> {
        if *self != Self::MIN {
            Some(Self(self.0.abs()))
        } else {
            None
        }
    }

    /// Returns the largest integer that is equal to or less than this number.
    pub fn checked_floor(&self) -> Option<Self> {
        self.checked_round(0, RoundingMode::ToNegativeInfinity)
    }

    /// Returns the smallest integer that is equal to or greater than this number.
    pub fn checked_ceiling(&self) -> Option<Self> {
        self.checked_round(0, RoundingMode::ToPositiveInfinity)
    }

    /// Rounds this number to the specified decimal places.
    ///
    /// # Panics
    /// - Panic if the number of decimal places is not within [0..SCALE]
    pub fn checked_round<T: Into<i32>>(
        &self,
        decimal_places: T,
        mode: RoundingMode,
    ) -> Option<Self> {
        let decimal_places = decimal_places.into();
        assert!(decimal_places <= Self::SCALE as i32);
        assert!(decimal_places >= 0);

        let n = Self::SCALE - decimal_places as u32;
        let divisor: I256 = I256::TEN.pow(n);
        let positive_remainder = {
            // % is the "C" style remainder operator, rather than the mathematical modulo operator,
            // So we fix that here https://internals.rust-lang.org/t/mathematical-modulo-operator/5952
            let remainder = self.0 % divisor;
            match remainder.cmp(&I256::ZERO) {
                Ordering::Less => divisor + remainder,
                Ordering::Equal => return Some(*self),
                Ordering::Greater => remainder,
            }
        };

        let resolved_strategy =
            ResolvedRoundingStrategy::from_mode(mode, self.is_positive(), || {
                let midpoint = divisor >> 1; // Half the divisor
                positive_remainder.cmp(&midpoint)
            });

        let rounded_subunits = match resolved_strategy {
            ResolvedRoundingStrategy::RoundUp => {
                let to_add = divisor
                    .checked_sub(positive_remainder)
                    .expect("Always safe");
                self.0.checked_add(to_add)?
            }
            ResolvedRoundingStrategy::RoundDown => self.0.checked_sub(positive_remainder)?,
            ResolvedRoundingStrategy::RoundToEven => {
                let double_divisor = divisor << 1; // Double the divisor
                if self.is_positive() {
                    // If positive, we try rounding down first (to avoid accidental overflow)
                    let rounded_down = self.0.checked_sub(positive_remainder)?;
                    if rounded_down % double_divisor == I256::ZERO {
                        rounded_down
                    } else {
                        rounded_down.checked_add(divisor)?
                    }
                } else {
                    // If negative, we try rounding up first (to avoid accidental overflow)
                    let to_add = divisor
                        .checked_sub(positive_remainder)
                        .expect("Always safe");
                    let rounded_up = self.0.checked_add(to_add)?;
                    if rounded_up % double_divisor == I256::ZERO {
                        rounded_up
                    } else {
                        rounded_up.checked_sub(divisor)?
                    }
                }
            }
        };

        Some(Self(rounded_subunits))
    }

    /// Calculates power using exponentiation by squaring.
    pub fn checked_powi(&self, exp: i64) -> Option<Self> {
        let one_384 = I384::from(Self::ONE.0);
        let base_384 = I384::from(self.0);
        let div = |x: i64, y: i64| x.checked_div(y);
        let sub = |x: i64, y: i64| x.checked_sub(y);
        let mul = |x: i64, y: i64| x.checked_mul(y);

        if exp < 0 {
            let sub_384 = (one_384 * one_384).checked_div(base_384)?;
            let sub_256 = I256::try_from(sub_384).ok()?;
            let exp = mul(exp, -1)?;
            return Self(sub_256).checked_powi(exp);
        }
        if exp == 0 {
            return Some(Self::ONE);
        }
        if exp == 1 {
            return Some(*self);
        }
        if exp % 2 == 0 {
            let sub_384 = base_384.checked_mul(base_384)? / one_384;
            let sub_256 = I256::try_from(sub_384).ok()?;
            let exp = div(exp, 2)?;
            Self(sub_256).checked_powi(exp)
        } else {
            let sub_384 = base_384.checked_mul(base_384)? / one_384;
            let sub_256 = I256::try_from(sub_384).ok()?;
            let sub_pdec = Self(sub_256);
            let exp = div(sub(exp, 1)?, 2)?;
            let b = sub_pdec.checked_powi(exp)?;
            self.checked_mul(b)
        }
    }

    /// Square root of a PreciseDecimal
    pub fn checked_sqrt(&self) -> Option<Self> {
        if self.is_negative() {
            return None;
        }
        if self.is_zero() {
            return Some(Self::ZERO);
        }

        // The I256 i associated to a Decimal d is : i = d*10^36.
        // Therefore, taking sqrt yields sqrt(i) = sqrt(d)*10^32 => We lost precision
        // To get the right precision, we compute : sqrt(i*10^36) = sqrt(d)*10^36
        let self_384 = I384::from(self.0);
        let correct_nb = self_384 * I384::from(Self::ONE.0);
        let sqrt = I256::try_from(correct_nb.sqrt()).ok()?;
        Some(Self(sqrt))
    }

    /// Cubic root of a PreciseDecimal
    pub fn checked_cbrt(&self) -> Option<Self> {
        if self.is_zero() {
            return Some(Self::ZERO);
        }

        // By reasoning in the same way as before, we realise that we need to multiply by 10^36
        let self_bigint = BigInt::from(self.0);
        let correct_nb: BigInt = self_bigint * BigInt::from(Self::ONE.0).pow(2_u32);
        let cbrt = I256::try_from(correct_nb.cbrt()).ok()?;
        Some(Self(cbrt))
    }

    /// Nth root of a PreciseDecimal
    pub fn checked_nth_root(&self, n: u32) -> Option<Self> {
        if (self.is_negative() && n % 2 == 0) || n == 0 {
            None
        } else if n == 1 {
            Some(*self)
        } else {
            if self.is_zero() {
                return Some(Self::ZERO);
            }

            // By induction, we need to multiply by the (n-1)th power of 10^36.
            // To not overflow, we use BigInt
            let self_integer = BigInt::from(self.0);
            let correct_nb = self_integer * BigInt::from(Self::ONE.0).pow(n - 1);
            let nth_root = I256::try_from(correct_nb.nth_root(n)).unwrap();
            Some(Self(nth_root))
        }
    }
}

macro_rules! from_primitive_type {
    ($($type:ident),*) => {
        $(
            impl From<$type> for PreciseDecimal {
                fn from(val: $type) -> Self {
                    Self(I256::from(val) * Self::ONE.0)
                }
            }
        )*
    };
}
macro_rules! to_primitive_type {
    ($($type:ident),*) => {
        $(
            impl TryFrom<PreciseDecimal> for $type {
                type Error = ParsePreciseDecimalError;

                fn try_from(val: PreciseDecimal) -> Result<Self, Self::Error> {
                    let rounded = val.checked_round(0, RoundingMode::ToZero).ok_or(ParsePreciseDecimalError::Overflow)?;
                    let fraction = val.checked_sub(rounded).ok_or(Self::Error::Overflow)?;
                    if !fraction.is_zero() {
                        Err(Self::Error::InvalidDigit)
                    }
                    else {
                        let i_256 = rounded.0 / I256::TEN.pow(PreciseDecimal::SCALE);
                        $type::try_from(i_256)
                            .map_err(|_| Self::Error::Overflow)
                    }
                }
            }

            impl TryFrom<&PreciseDecimal> for $type {
                type Error = ParsePreciseDecimalError;

                fn try_from(val: &PreciseDecimal) -> Result<Self, Self::Error> {
                    $type::try_from(*val)
                }
            }
        )*
    }
}
from_primitive_type!(i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);
to_primitive_type!(i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);

resolvable_with_try_into_impls!(PreciseDecimal);

// from_str() should be enough, but we want to have try_from() to simplify pdec! macro
impl TryFrom<&str> for PreciseDecimal {
    type Error = ParsePreciseDecimalError;

    fn try_from(val: &str) -> Result<Self, Self::Error> {
        Self::from_str(val)
    }
}

impl TryFrom<String> for PreciseDecimal {
    type Error = ParsePreciseDecimalError;

    fn try_from(val: String) -> Result<Self, Self::Error> {
        Self::from_str(&val)
    }
}

impl From<bool> for PreciseDecimal {
    fn from(val: bool) -> Self {
        if val {
            Self::ONE
        } else {
            Self::ZERO
        }
    }
}

impl CheckedNeg<PreciseDecimal> for PreciseDecimal {
    type Output = Self;

    #[inline]
    fn checked_neg(self) -> Option<Self::Output> {
        let c = self.0.checked_neg();
        c.map(Self)
    }
}

impl CheckedAdd<PreciseDecimal> for PreciseDecimal {
    type Output = Self;

    #[inline]
    fn checked_add(self, other: Self) -> Option<Self::Output> {
        let a = self.0;
        let b = other.0;
        let c = a.checked_add(b);
        c.map(Self)
    }
}

impl CheckedSub<PreciseDecimal> for PreciseDecimal {
    type Output = Self;

    #[inline]
    fn checked_sub(self, other: Self) -> Option<Self::Output> {
        let a = self.0;
        let b = other.0;
        let c = a.checked_sub(b);
        c.map(Self)
    }
}

impl CheckedMul<PreciseDecimal> for PreciseDecimal {
    type Output = Self;

    #[inline]
    fn checked_mul(self, other: Self) -> Option<Self> {
        // Use I384 (BInt<6>) to not overflow.
        let a = I384::from(self.0);
        let b = I384::from(other.0);

        let c = a.checked_mul(b)?;
        let c = c.checked_div(I384::from(Self::ONE.0))?;

        let c_256 = I256::try_from(c).ok();
        c_256.map(Self)
    }
}

impl CheckedDiv<PreciseDecimal> for PreciseDecimal {
    type Output = Self;

    #[inline]
    fn checked_div(self, other: Self) -> Option<Self> {
        // Use I384 (BInt<6>) to not overflow.
        let a = I384::from(self.0);
        let b = I384::from(other.0);

        let c = a.checked_mul(I384::from(Self::ONE.0))?;
        let c = c.checked_div(b)?;

        let c_256 = I256::try_from(c).ok();
        c_256.map(Self)
    }
}

impl Neg for PreciseDecimal {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self::Output {
        self.checked_neg().expect("Overflow")
    }
}

impl Add<PreciseDecimal> for PreciseDecimal {
    type Output = Self;

    #[inline]
    fn add(self, other: Self) -> Self::Output {
        self.checked_add(other).expect("Overflow")
    }
}

impl Sub<PreciseDecimal> for PreciseDecimal {
    type Output = Self;

    #[inline]
    fn sub(self, other: Self) -> Self::Output {
        self.checked_sub(other).expect("Overflow")
    }
}

impl Mul<PreciseDecimal> for PreciseDecimal {
    type Output = Self;

    #[inline]
    fn mul(self, other: Self) -> Self::Output {
        self.checked_mul(other).expect("Overflow")
    }
}

impl Div<PreciseDecimal> for PreciseDecimal {
    type Output = Self;

    #[inline]
    fn div(self, other: Self) -> Self::Output {
        self.checked_div(other)
            .expect("Overflow or division by zero")
    }
}

impl AddAssign<PreciseDecimal> for PreciseDecimal {
    #[inline]
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl SubAssign<PreciseDecimal> for PreciseDecimal {
    #[inline]
    fn sub_assign(&mut self, other: Self) {
        *self = *self - other;
    }
}

impl MulAssign<PreciseDecimal> for PreciseDecimal {
    #[inline]
    fn mul_assign(&mut self, other: Self) {
        *self = *self * other;
    }
}

impl DivAssign<PreciseDecimal> for PreciseDecimal {
    #[inline]
    fn div_assign(&mut self, other: Self) {
        *self = *self / other;
    }
}

macro_rules! impl_arith_ops {
    ($type:ident) => {
        impl CheckedAdd<$type> for PreciseDecimal {
            type Output = Self;

            fn checked_add(self, other: $type) -> Option<Self::Output> {
                self.checked_add(Self::try_from(other).ok()?)
            }
        }

        impl CheckedSub<$type> for PreciseDecimal {
            type Output = Self;

            fn checked_sub(self, other: $type) -> Option<Self::Output> {
                self.checked_sub(Self::try_from(other).ok()?)
            }
        }

        impl CheckedMul<$type> for PreciseDecimal {
            type Output = Self;

            fn checked_mul(self, other: $type) -> Option<Self::Output> {
                self.checked_mul(Self::try_from(other).ok()?)
            }
        }

        impl CheckedDiv<$type> for PreciseDecimal {
            type Output = Self;

            fn checked_div(self, other: $type) -> Option<Self::Output> {
                self.checked_div(Self::try_from(other).ok()?)
            }
        }

        impl Add<$type> for PreciseDecimal {
            type Output = Self;

            #[inline]
            fn add(self, other: $type) -> Self::Output {
                self.checked_add(other).expect("Overflow")
            }
        }

        impl Sub<$type> for PreciseDecimal {
            type Output = Self;

            #[inline]
            fn sub(self, other: $type) -> Self::Output {
                self.checked_sub(other).expect("Overflow")
            }
        }

        impl Mul<$type> for PreciseDecimal {
            type Output = Self;

            #[inline]
            fn mul(self, other: $type) -> Self::Output {
                self.checked_mul(other).expect("Overflow")
            }
        }

        impl Div<$type> for PreciseDecimal {
            type Output = Self;

            #[inline]
            fn div(self, other: $type) -> Self::Output {
                self.checked_div(other)
                    .expect("Overflow or division by zero")
            }
        }

        impl Add<PreciseDecimal> for $type {
            type Output = PreciseDecimal;

            #[inline]
            fn add(self, other: PreciseDecimal) -> Self::Output {
                other + self
            }
        }

        impl Sub<PreciseDecimal> for $type {
            type Output = PreciseDecimal;

            #[inline]
            fn sub(self, other: PreciseDecimal) -> Self::Output {
                // Cannot use self.checked_sub directly.
                // It conflicts with already defined checked_sub for primitive types.
                PreciseDecimal::try_from(self)
                    .expect("Overflow")
                    .checked_sub(other)
                    .expect("Overflow")
            }
        }

        impl Mul<PreciseDecimal> for $type {
            type Output = PreciseDecimal;

            #[inline]
            fn mul(self, other: PreciseDecimal) -> Self::Output {
                other * self
            }
        }

        impl Div<PreciseDecimal> for $type {
            type Output = PreciseDecimal;

            #[inline]
            fn div(self, other: PreciseDecimal) -> Self::Output {
                // Cannot use self.checked_div directly.
                // It conflicts with already defined checked_sub for primitive types.
                PreciseDecimal::try_from(self)
                    .expect("Overflow")
                    .checked_div(other)
                    .expect("Overflow or division by zero")
            }
        }

        impl AddAssign<$type> for PreciseDecimal {
            #[inline]
            fn add_assign(&mut self, other: $type) {
                *self = *self + other;
            }
        }

        impl SubAssign<$type> for PreciseDecimal {
            #[inline]
            fn sub_assign(&mut self, other: $type) {
                *self = *self - other;
            }
        }

        impl MulAssign<$type> for PreciseDecimal {
            #[inline]
            fn mul_assign(&mut self, other: $type) {
                *self = *self * other;
            }
        }

        impl DivAssign<$type> for PreciseDecimal {
            #[inline]
            fn div_assign(&mut self, other: $type) {
                *self = *self / other;
            }
        }
    };
}
impl_arith_ops!(u8);
impl_arith_ops!(u16);
impl_arith_ops!(u32);
impl_arith_ops!(u64);
impl_arith_ops!(u128);
impl_arith_ops!(usize);
impl_arith_ops!(i8);
impl_arith_ops!(i16);
impl_arith_ops!(i32);
impl_arith_ops!(i64);
impl_arith_ops!(i128);
impl_arith_ops!(isize);
impl_arith_ops!(Decimal);
impl_arith_ops!(I192);
impl_arith_ops!(I256);
impl_arith_ops!(I320);
impl_arith_ops!(I448);
impl_arith_ops!(I512);
impl_arith_ops!(U192);
impl_arith_ops!(U256);
impl_arith_ops!(U320);
impl_arith_ops!(U448);
impl_arith_ops!(U512);

// Below implements CheckedX traits for given type with PreciseDecimal as an argument.
// It cannot be used for primitive types, since they already implement these traits
// but with different argument type.
macro_rules! impl_arith_ops_non_primitives {
    ($type:ident) => {
        impl CheckedAdd<PreciseDecimal> for $type {
            type Output = PreciseDecimal;

            #[inline]
            fn checked_add(self, other: PreciseDecimal) -> Option<Self::Output> {
                other.checked_add(self)
            }
        }

        impl CheckedSub<PreciseDecimal> for $type {
            type Output = PreciseDecimal;

            fn checked_sub(self, other: PreciseDecimal) -> Option<Self::Output> {
                PreciseDecimal::try_from(self).ok()?.checked_sub(other)
            }
        }

        impl CheckedMul<PreciseDecimal> for $type {
            type Output = PreciseDecimal;

            #[inline]
            fn checked_mul(self, other: PreciseDecimal) -> Option<Self::Output> {
                other.checked_mul(self)
            }
        }

        impl CheckedDiv<PreciseDecimal> for $type {
            type Output = PreciseDecimal;

            fn checked_div(self, other: PreciseDecimal) -> Option<Self::Output> {
                PreciseDecimal::try_from(self).ok()?.checked_div(other)
            }
        }
    };
}
impl_arith_ops_non_primitives!(Decimal);
impl_arith_ops_non_primitives!(I192);
impl_arith_ops_non_primitives!(I256);
impl_arith_ops_non_primitives!(I320);
impl_arith_ops_non_primitives!(I448);
impl_arith_ops_non_primitives!(I512);
impl_arith_ops_non_primitives!(U192);
impl_arith_ops_non_primitives!(U256);
impl_arith_ops_non_primitives!(U320);
impl_arith_ops_non_primitives!(U448);
impl_arith_ops_non_primitives!(U512);

//========
// binary
//========

impl TryFrom<&[u8]> for PreciseDecimal {
    type Error = ParsePreciseDecimalError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        if slice.len() == Self::BITS / 8 {
            let val = I256::try_from(slice).expect("Length should have already been checked.");
            Ok(Self(val))
        } else {
            Err(ParsePreciseDecimalError::InvalidLength(slice.len()))
        }
    }
}

impl PreciseDecimal {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_le_bytes().to_vec()
    }
}

well_known_scrypto_custom_type!(
    PreciseDecimal,
    ScryptoCustomValueKind::PreciseDecimal,
    Type::PreciseDecimal,
    PreciseDecimal::BITS / 8,
    PRECISE_DECIMAL_TYPE,
    precise_decimal_type_data,
);

manifest_type!(
    PreciseDecimal,
    ManifestCustomValueKind::PreciseDecimal,
    PreciseDecimal::BITS / 8,
);

//======
// text
//======

impl FromStr for PreciseDecimal {
    type Err = ParsePreciseDecimalError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let v: Vec<&str> = s.split('.').collect();

        if v.len() > 2 {
            return Err(ParsePreciseDecimalError::MoreThanOneDecimalPoint);
        }

        let integer_part = match I256::from_str(v[0]) {
            Ok(val) => val,
            Err(err) => match err {
                ParseI256Error::NegativeToUnsigned => {
                    unreachable!("NegativeToUnsigned is only for parsing unsigned types, not I256")
                }
                ParseI256Error::Overflow => return Err(ParsePreciseDecimalError::Overflow),
                ParseI256Error::InvalidLength => {
                    unreachable!("InvalidLength is only for parsing &[u8], not &str")
                }
                ParseI256Error::InvalidDigit => return Err(ParsePreciseDecimalError::InvalidDigit),
                // We have decided to be restrictive to force people to type "0.123" instead of ".123"
                // for clarity, and to align with how rust's float literal works
                ParseI256Error::Empty => return Err(ParsePreciseDecimalError::EmptyIntegralPart),
            },
        };

        let mut subunits = integer_part
            .checked_mul(Self::ONE.0)
            .ok_or(ParsePreciseDecimalError::Overflow)?;

        if v.len() == 2 {
            let scale = if let Some(scale) = Self::SCALE.checked_sub(v[1].len() as u32) {
                Ok(scale)
            } else {
                Err(Self::Err::MoreThanThirtySixDecimalPlaces)
            }?;

            let fractional_part = match I256::from_str(v[1]) {
                Ok(val) => val,
                Err(err) => match err {
                    ParseI256Error::NegativeToUnsigned => {
                        unreachable!(
                            "NegativeToUnsigned is only for parsing unsigned types, not I256"
                        )
                    }
                    ParseI256Error::Overflow => return Err(ParsePreciseDecimalError::Overflow),
                    ParseI256Error::InvalidLength => {
                        unreachable!("InvalidLength is only for parsing &[u8], not &str")
                    }
                    ParseI256Error::InvalidDigit => {
                        return Err(ParsePreciseDecimalError::InvalidDigit)
                    }
                    ParseI256Error::Empty => {
                        return Err(ParsePreciseDecimalError::EmptyFractionalPart)
                    }
                },
            };

            // The product of these must be less than Self::SCALE
            let fractional_subunits = fractional_part
                .checked_mul(I256::TEN.pow(scale))
                .expect("No overflow possible");

            // if input is -0. then from_str returns 0 and we loose '-' sign.
            // Therefore check for '-' in input directly
            if integer_part.is_negative() || v[0].starts_with('-') {
                subunits = subunits
                    .checked_sub(fractional_subunits)
                    .ok_or(ParsePreciseDecimalError::Overflow)?;
            } else {
                subunits = subunits
                    .checked_add(fractional_subunits)
                    .ok_or(ParsePreciseDecimalError::Overflow)?;
            }
        }
        Ok(Self(subunits))
    }
}

impl fmt::Display for PreciseDecimal {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        const MULTIPLIER: I256 = PreciseDecimal::ONE.0;
        let quotient = self.0 / MULTIPLIER;
        let remainder = self.0 % MULTIPLIER;

        if !remainder.is_zero() {
            // print remainder with leading zeroes
            let mut sign = "".to_string();

            // take care of sign in case quotient == zere and remainder < 0,
            // eg.
            //  self.0=-100000000000000000 -> -0.1
            if remainder < I256::ZERO && quotient == I256::ZERO {
                sign.push('-');
            }
            let rem_str = format!(fmt_remainder!(), remainder.abs());
            write!(f, "{}{}.{}", sign, quotient, &rem_str.trim_end_matches('0'))
        } else {
            write!(f, "{}", quotient)
        }
    }
}

impl fmt::Debug for PreciseDecimal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

//========
// ParseDecimalError, ParsePreciseDecimalError
//========

/// Represents an error when parsing PreciseDecimal from another type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsePreciseDecimalError {
    InvalidDigit,
    Overflow,
    EmptyIntegralPart,
    EmptyFractionalPart,
    MoreThanThirtySixDecimalPlaces,
    MoreThanOneDecimalPoint,
    InvalidLength(usize),
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParsePreciseDecimalError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParsePreciseDecimalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<Decimal> for PreciseDecimal {
    fn from(val: Decimal) -> Self {
        Self(I256::try_from(val.attos()).unwrap() * I256::TEN.pow(Self::SCALE - Decimal::SCALE))
    }
}

pub trait CheckedTruncate<T> {
    type Output;
    fn checked_truncate(self, mode: RoundingMode) -> Option<Self::Output>;
}

impl CheckedTruncate<Decimal> for PreciseDecimal {
    type Output = Decimal;

    fn checked_truncate(self, mode: RoundingMode) -> Option<Self::Output> {
        let rounded = self.checked_round(Decimal::SCALE as i32, mode)?;

        let a_256 = rounded
            .0
            .checked_div(I256::TEN.pow(Self::SCALE - Decimal::SCALE))?;

        Some(Decimal::from_attos(a_256.try_into().ok()?))
    }
}

macro_rules! try_from_integer {
    ($($t:ident),*) => {
        $(
            impl TryFrom<$t> for PreciseDecimal {
                type Error = ParsePreciseDecimalError;

                fn try_from(val: $t) -> Result<Self, Self::Error> {
                    match I256::try_from(val) {
                        Ok(val) => {
                            match val.checked_mul(Self::ONE.0) {
                                Some(mul) => Ok(Self(mul)),
                                None => Err(ParsePreciseDecimalError::Overflow),
                            }
                        },
                        Err(_) => Err(ParsePreciseDecimalError::Overflow),
                    }
                }
            }
        )*
    };
}

try_from_integer!(I192, I256, I320, I384, I448, I512);
try_from_integer!(U192, U256, U320, U384, U448, U512);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::precise_decimal::RoundingMode;
    use paste::paste;

    macro_rules! test_dec {
        // NOTE: Decimal arithmetic operation safe unwrap.
        // In general, it is assumed that reasonable literals are provided.
        // If not then something is definitely wrong and panic is fine.
        ($x:literal) => {
            $crate::math::Decimal::try_from($x).unwrap()
        };
    }

    macro_rules! test_pdec {
        // NOTE: Decimal arithmetic operation safe unwrap.
        // In general, it is assumed that reasonable literals are provided.
        // If not then something is definitely wrong and panic is fine.
        ($x:literal) => {
            $crate::math::PreciseDecimal::try_from($x).unwrap()
        };
    }

    #[test]
    fn test_format_precise_decimal() {
        assert_eq!(
            PreciseDecimal(1i128.into()).to_string(),
            "0.000000000000000000000000000000000001"
        );
        assert_eq!(
            PreciseDecimal(123456789123456789i128.into()).to_string(),
            "0.000000000000000000123456789123456789"
        );
        assert_eq!(
            PreciseDecimal(I256::from(10).pow(PreciseDecimal::SCALE)).to_string(),
            "1"
        );
        assert_eq!(
            PreciseDecimal(I256::from(10).pow(PreciseDecimal::SCALE) * I256::from(123)).to_string(),
            "123"
        );
        assert_eq!(
            PreciseDecimal(
                I256::from_str("123456789000000000000000000000000000000000000").unwrap()
            )
            .to_string(),
            "123456789"
        );
        assert_eq!(
            PreciseDecimal::MAX.to_string(),
            "57896044618658097711785492504343953926634.992332820282019728792003956564819967"
        );
        assert_eq!(PreciseDecimal::MIN.is_negative(), true);
        assert_eq!(
            PreciseDecimal::MIN.to_string(),
            "-57896044618658097711785492504343953926634.992332820282019728792003956564819968"
        );
    }

    #[test]
    fn test_parse_precise_decimal() {
        assert_eq!(
            PreciseDecimal::from_str("0.000000000000000001").unwrap(),
            PreciseDecimal(I256::from(10).pow(18)),
        );
        assert_eq!(
            PreciseDecimal::from_str("0.0000000000000000000000000000000000001"),
            Err(ParsePreciseDecimalError::MoreThanThirtySixDecimalPlaces),
        );
        assert_eq!(
            PreciseDecimal::from_str("0.123456789123456789").unwrap(),
            PreciseDecimal(I256::from(123456789123456789i128) * I256::from(10i8).pow(18)),
        );
        assert_eq!(
            PreciseDecimal::from_str("1").unwrap(),
            PreciseDecimal(I256::from(10).pow(PreciseDecimal::SCALE)),
        );
        assert_eq!(
            PreciseDecimal::from_str("123456789123456789").unwrap(),
            PreciseDecimal(
                I256::from(123456789123456789i128) * I256::from(10).pow(PreciseDecimal::SCALE)
            ),
        );
        assert_eq!(
            PreciseDecimal::from_str(
                "57896044618658097711785492504343953926634.992332820282019728792003956564819967"
            )
            .unwrap(),
            PreciseDecimal::MAX,
        );
        assert_eq!(
            PreciseDecimal::from_str(
                "57896044618658097711785492504343953926634.992332820282019728792003956564819968"
            ),
            Err(ParsePreciseDecimalError::Overflow),
        );
        assert_eq!(
            PreciseDecimal::from_str("157896044618658097711785492504343953926634"),
            Err(ParsePreciseDecimalError::Overflow),
        );
        assert_eq!(
            PreciseDecimal::from_str(
                "-57896044618658097711785492504343953926634.992332820282019728792003956564819968"
            )
            .unwrap(),
            PreciseDecimal::MIN,
        );
        assert_eq!(
            PreciseDecimal::from_str(
                "-57896044618658097711785492504343953926634.992332820282019728792003956564819969"
            ),
            Err(ParsePreciseDecimalError::Overflow),
        );
        assert_eq!(
            PreciseDecimal::from_str(".000000000000000231"),
            Err(ParsePreciseDecimalError::EmptyIntegralPart),
        );
        assert_eq!(
            PreciseDecimal::from_str("231."),
            Err(ParsePreciseDecimalError::EmptyFractionalPart),
        );

        assert_eq!(test_pdec!("0"), PreciseDecimal::ZERO);
        assert_eq!(test_pdec!("1"), PreciseDecimal::ONE);
        assert_eq!(test_pdec!("0.1"), PreciseDecimal::ONE_TENTH);
        assert_eq!(test_pdec!("10"), PreciseDecimal::TEN);
        assert_eq!(test_pdec!("100"), PreciseDecimal::ONE_HUNDRED);
        assert_eq!(test_pdec!("0.01"), PreciseDecimal::ONE_HUNDREDTH);
        assert_eq!(test_pdec!("0.000000000000000001"), PreciseDecimal::ONE_ATTO);
        assert_eq!(
            test_pdec!("0.000000000000000000000000000000000001"),
            PreciseDecimal::ONE_PRECISE_SUBUNIT
        );

        assert_eq!("0", PreciseDecimal::ZERO.to_string());
        assert_eq!("1", PreciseDecimal::ONE.to_string());
        assert_eq!("0.1", PreciseDecimal::ONE_TENTH.to_string());
        assert_eq!("10", PreciseDecimal::TEN.to_string());
        assert_eq!("100", PreciseDecimal::ONE_HUNDRED.to_string());
        assert_eq!("0.01", PreciseDecimal::ONE_HUNDREDTH.to_string());
        assert_eq!("0.000000000000000001", PreciseDecimal::ONE_ATTO.to_string());
        assert_eq!(
            "0.000000000000000000000000000000000001",
            PreciseDecimal::ONE_PRECISE_SUBUNIT.to_string()
        );
    }

    #[test]
    fn test_add_precise_decimal() {
        let a = PreciseDecimal::from(5u32);
        let b = PreciseDecimal::from(7u32);
        assert_eq!(a.checked_add(b).unwrap().to_string(), "12");
    }

    #[test]
    fn test_add_overflow_precise_decimal() {
        assert!(PreciseDecimal::MAX
            .checked_add(PreciseDecimal::ONE)
            .is_none());
    }

    #[test]
    fn test_sub_precise_decimal() {
        let a = PreciseDecimal::from(5u32);
        let b = PreciseDecimal::from(7u32);
        assert_eq!(a.checked_sub(b).unwrap().to_string(), "-2");
        assert_eq!(b.checked_sub(a).unwrap().to_string(), "2");
    }

    #[test]
    fn test_sub_overflow_precise_decimal() {
        assert!(PreciseDecimal::MIN
            .checked_sub(PreciseDecimal::ONE)
            .is_none());
    }

    #[test]
    fn test_mul_precise_decimal() {
        let a = PreciseDecimal::from(5u32);
        let b = PreciseDecimal::from(7u32);
        assert_eq!(a.checked_mul(b).unwrap().to_string(), "35");
        let a = PreciseDecimal::from_str("1000000000").unwrap();
        let b = PreciseDecimal::from_str("1000000000").unwrap();
        assert_eq!(a.checked_mul(b).unwrap().to_string(), "1000000000000000000");

        let a = PreciseDecimal::MAX.checked_div(test_pdec!(2)).unwrap();
        let b = PreciseDecimal::from(2);
        assert_eq!(
            a.checked_mul(b).unwrap(),
            test_pdec!(
                "57896044618658097711785492504343953926634.992332820282019728792003956564819966"
            )
        );
    }

    #[test]
    fn test_mul_to_max_precise_decimal() {
        let a = PreciseDecimal::MAX.checked_sqrt().unwrap();
        a.checked_mul(a).unwrap();
    }

    #[test]
    fn test_mul_to_minimum_overflow_decimal() {
        let a = PreciseDecimal::MAX.checked_sqrt().unwrap();
        assert!(a.checked_mul(a + PreciseDecimal(I256::ONE)).is_none());
    }

    #[test]
    fn test_mul_overflow_by_small_precise_decimal() {
        assert!(PreciseDecimal::MAX
            .checked_mul(test_pdec!("1.000000000000000000000000000000000001"))
            .is_none());
    }

    #[test]
    fn test_mul_overflow_by_a_lot_precise_decimal() {
        assert!(PreciseDecimal::MAX.checked_mul(test_pdec!("1.1")).is_none());
    }

    #[test]
    fn test_mul_neg_overflow_precise_decimal() {
        assert!(PreciseDecimal::MAX
            .checked_neg()
            .unwrap()
            .checked_mul(test_pdec!("-1.000000000000000000000000000000000001"))
            .is_none());
    }

    #[test]
    fn test_div_by_zero_precise_decimal() {
        let a = PreciseDecimal::from(5u32);
        let b = PreciseDecimal::from(0u32);
        assert!(a.checked_div(b).is_none());
    }

    #[test]
    fn test_powi_exp_overflow_precise_decimal() {
        let a = PreciseDecimal::from(5u32);
        let b = i64::MIN;
        assert!(a.checked_powi(b).is_none());
    }

    #[test]
    fn test_1_powi_max_precise_decimal() {
        let a = PreciseDecimal::from(1u32);
        let b = i64::MAX;
        assert_eq!(a.checked_powi(b).unwrap().to_string(), "1");
    }

    #[test]
    fn test_1_powi_min_precise_decimal() {
        let a = PreciseDecimal::from(1u32);
        let b = i64::MAX - 1;
        assert_eq!(a.checked_powi(b).unwrap().to_string(), "1");
    }

    #[test]
    fn test_powi_max_precise_decimal() {
        let _max = PreciseDecimal::MAX.checked_powi(1).unwrap();
        let _max_sqrt = PreciseDecimal::MAX.checked_sqrt().unwrap();
        let _max_cbrt = PreciseDecimal::MAX.checked_cbrt().unwrap();
        let _max_dec_2 = _max_sqrt.checked_powi(2).unwrap();
        let _max_dec_3 = _max_cbrt.checked_powi(3).unwrap();
    }

    #[test]
    fn test_div_precise_decimal() {
        let a = PreciseDecimal::from(5u32);
        let b = PreciseDecimal::from(7u32);
        assert_eq!(
            a.checked_div(b).unwrap().to_string(),
            "0.714285714285714285714285714285714285"
        );
        assert_eq!(b.checked_div(a).unwrap().to_string(), "1.4");
        let a = PreciseDecimal::MAX;
        let b = PreciseDecimal::from(2);
        assert_eq!(
            a.checked_div(b).unwrap(),
            test_pdec!(
                "28948022309329048855892746252171976963317.496166410141009864396001978282409983"
            )
        );
    }

    #[test]
    fn test_div_negative_precise_decimal() {
        let a = PreciseDecimal::from(-42);
        let b = PreciseDecimal::from(2);
        assert_eq!(a.checked_div(b).unwrap().to_string(), "-21");
    }

    #[test]
    fn test_0_pow_0_precise_decimal() {
        let a = test_pdec!("0");
        assert_eq!(a.checked_powi(0).unwrap().to_string(), "1");
    }

    #[test]
    fn test_0_powi_1_precise_decimal() {
        let a = test_pdec!("0");
        assert_eq!(a.checked_powi(1).unwrap().to_string(), "0");
    }

    #[test]
    fn test_0_powi_10_precise_decimal() {
        let a = test_pdec!("0");
        assert_eq!(a.checked_powi(10).unwrap().to_string(), "0");
    }

    #[test]
    fn test_1_powi_0_precise_decimal() {
        let a = test_pdec!(1);
        assert_eq!(a.checked_powi(0).unwrap().to_string(), "1");
    }

    #[test]
    fn test_1_powi_1_precise_decimal() {
        let a = test_pdec!(1);
        assert_eq!(a.checked_powi(1).unwrap().to_string(), "1");
    }

    #[test]
    fn test_1_powi_10_precise_decimal() {
        let a = test_pdec!(1);
        assert_eq!(a.checked_powi(10).unwrap().to_string(), "1");
    }

    #[test]
    fn test_2_powi_0_precise_decimal() {
        let a = test_pdec!("2");
        assert_eq!(a.checked_powi(0).unwrap().to_string(), "1");
    }

    #[test]
    fn test_2_powi_3724_precise_decimal() {
        let a = test_pdec!("1.000234891009084238");
        assert_eq!(
            a.checked_powi(3724).unwrap().to_string(),
            "2.3979912322546748642222795591580985"
        );
    }

    #[test]
    fn test_2_powi_2_precise_decimal() {
        let a = test_pdec!("2");
        assert_eq!(a.checked_powi(2).unwrap().to_string(), "4");
    }

    #[test]
    fn test_2_powi_3_precise_decimal() {
        let a = test_pdec!("2");
        assert_eq!(a.checked_powi(3).unwrap().to_string(), "8");
    }

    #[test]
    fn test_10_powi_3_precise_decimal() {
        let a = test_pdec!("10");
        assert_eq!(a.checked_powi(3).unwrap().to_string(), "1000");
    }

    #[test]
    fn test_5_powi_2_precise_decimal() {
        let a = test_pdec!("5");
        assert_eq!(a.checked_powi(2).unwrap().to_string(), "25");
    }

    #[test]
    fn test_5_powi_minus2_precise_decimal() {
        let a = test_pdec!("5");
        assert_eq!(a.checked_powi(-2).unwrap().to_string(), "0.04");
    }

    #[test]
    fn test_10_powi_minus3_precise_decimal() {
        let a = test_pdec!("10");
        assert_eq!(a.checked_powi(-3).unwrap().to_string(), "0.001");
    }

    #[test]
    fn test_minus10_powi_minus3_precise_decimal() {
        let a = test_pdec!("-10");
        assert_eq!(a.checked_powi(-3).unwrap().to_string(), "-0.001");
    }

    #[test]
    fn test_minus10_powi_minus2_precise_decimal() {
        let a = test_pdec!("-10");
        assert_eq!(a.checked_powi(-2).unwrap().to_string(), "0.01");
    }

    #[test]
    fn test_minus05_powi_minus2_precise_decimal() {
        let a = test_pdec!("-0.5");
        assert_eq!(a.checked_powi(-2).unwrap().to_string(), "4");
    }
    #[test]
    fn test_minus05_powi_minus3_precise_decimal() {
        let a = test_pdec!("-0.5");
        assert_eq!(a.checked_powi(-3).unwrap().to_string(), "-8");
    }

    #[test]
    fn test_10_powi_15_precise_decimal() {
        let a = test_pdec!(10i128);
        assert_eq!(a.checked_powi(15).unwrap().to_string(), "1000000000000000");
    }

    #[test]
    fn test_10_powi_16_precise_decimal() {
        let a = PreciseDecimal(10i128.into());
        assert_eq!(a.checked_powi(16).unwrap().to_string(), "0");
    }

    #[test]
    fn test_one_and_zero_precise_decimal() {
        assert_eq!(PreciseDecimal::one().to_string(), "1");
        assert_eq!(PreciseDecimal::zero().to_string(), "0");
    }

    #[test]
    fn test_dec_string_decimal_precise_decimal() {
        assert_eq!(
            test_pdec!("1.123456789012345678").to_string(),
            "1.123456789012345678"
        );
        assert_eq!(test_pdec!("-5.6").to_string(), "-5.6");
    }

    #[test]
    fn test_dec_string_precise_decimal() {
        assert_eq!(test_pdec!(1).to_string(), "1");
        assert_eq!(test_pdec!("0").to_string(), "0");
    }

    #[test]
    fn test_dec_int_precise_decimal() {
        assert_eq!(test_pdec!(1).to_string(), "1");
        assert_eq!(test_pdec!(5).to_string(), "5");
    }

    #[test]
    fn test_dec_bool_precise_decimal() {
        assert_eq!((test_pdec!(false)).to_string(), "0");
    }

    #[test]
    fn test_floor_precise_decimal() {
        assert_eq!(
            PreciseDecimal::MAX.checked_floor().unwrap(),
            test_pdec!("57896044618658097711785492504343953926634")
        );
        assert_eq!(test_pdec!("1.2").checked_floor().unwrap(), test_pdec!("1"));
        assert_eq!(test_pdec!("1.0").checked_floor().unwrap(), test_pdec!("1"));
        assert_eq!(test_pdec!("0.9").checked_floor().unwrap(), test_pdec!("0"));
        assert_eq!(test_pdec!("0").checked_floor().unwrap(), test_pdec!("0"));
        assert_eq!(
            test_pdec!("-0.1").checked_floor().unwrap(),
            test_pdec!("-1")
        );
        assert_eq!(test_pdec!("-1").checked_floor().unwrap(), test_pdec!("-1"));
        assert_eq!(
            test_pdec!("-5.2").checked_floor().unwrap(),
            test_pdec!("-6")
        );

        assert_eq!(
            test_pdec!(
                "-57896044618658097711785492504343953926633.992332820282019728792003956564819968"
            ) // PreciseDecimal::MIN+1
            .checked_floor()
            .unwrap(),
            test_pdec!("-57896044618658097711785492504343953926634")
        );
        assert_eq!(
            test_pdec!(
                "-57896044618658097711785492504343953926633.000000000000000000000000000000000001"
            )
            .checked_floor()
            .unwrap(),
            test_pdec!("-57896044618658097711785492504343953926634")
        );
        assert_eq!(
            test_pdec!(
                "-57896044618658097711785492504343953926634.000000000000000000000000000000000000"
            )
            .checked_floor()
            .unwrap(),
            test_pdec!("-57896044618658097711785492504343953926634")
        );

        // below shall return None due to overflow
        assert!(PreciseDecimal::MIN.checked_floor().is_none());

        assert!(test_pdec!(
            "-57896044618658097711785492504343953926634.000000000000000000000000000000000001"
        )
        .checked_floor()
        .is_none());
    }

    #[test]
    fn test_abs_precise_decimal() {
        assert_eq!(test_pdec!(-2).checked_abs().unwrap(), test_pdec!(2));
        assert_eq!(test_pdec!(2).checked_abs().unwrap(), test_pdec!(2));
        assert_eq!(test_pdec!(0).checked_abs().unwrap(), test_pdec!(0));
        assert_eq!(
            PreciseDecimal::MAX.checked_abs().unwrap(),
            PreciseDecimal::MAX
        );

        // below shall return None due to overflow
        assert!(PreciseDecimal::MIN.checked_abs().is_none());
    }

    #[test]
    fn test_ceiling_precise_decimal() {
        assert_eq!(
            test_pdec!("1.2").checked_ceiling().unwrap(),
            test_pdec!("2")
        );
        assert_eq!(
            test_pdec!("1.0").checked_ceiling().unwrap(),
            test_pdec!("1")
        );
        assert_eq!(
            test_pdec!("0.9").checked_ceiling().unwrap(),
            test_pdec!("1")
        );
        assert_eq!(test_pdec!("0").checked_ceiling().unwrap(), test_pdec!("0"));
        assert_eq!(
            test_pdec!("-0.1").checked_ceiling().unwrap(),
            test_pdec!("0")
        );
        assert_eq!(
            test_pdec!("-1").checked_ceiling().unwrap(),
            test_pdec!("-1")
        );
        assert_eq!(
            test_pdec!("-5.2").checked_ceiling().unwrap(),
            test_pdec!("-5")
        );
        assert_eq!(
            PreciseDecimal::MIN.checked_ceiling().unwrap(),
            test_pdec!("-57896044618658097711785492504343953926634")
        );
        assert_eq!(
            test_pdec!(
                "57896044618658097711785492504343953926633.992332820282019728792003956564819967"
            ) // PreciseDecimal::MAX-1
            .checked_ceiling()
            .unwrap(),
            test_pdec!("57896044618658097711785492504343953926634")
        );
        assert_eq!(
            test_pdec!(
                "57896044618658097711785492504343953926633.000000000000000000000000000000000000"
            )
            .checked_ceiling()
            .unwrap(),
            test_pdec!("57896044618658097711785492504343953926633")
        );

        // below shall return None due to overflow
        assert!(PreciseDecimal::MAX.checked_ceiling().is_none());
        assert!(test_pdec!(
            "57896044618658097711785492504343953926634.000000000000000000000000000000000001"
        )
        .checked_ceiling()
        .is_none());
    }

    #[test]
    fn test_rounding_to_zero_precise_decimal() {
        let mode = RoundingMode::ToZero;
        assert_eq!(
            test_pdec!("1.2").checked_round(0, mode).unwrap(),
            test_pdec!("1")
        );
        assert_eq!(
            test_pdec!("1.0").checked_round(0, mode).unwrap(),
            test_pdec!("1")
        );
        assert_eq!(
            test_pdec!("0.9").checked_round(0, mode).unwrap(),
            test_pdec!("0")
        );
        assert_eq!(
            test_pdec!("0").checked_round(0, mode).unwrap(),
            test_pdec!("0")
        );
        assert_eq!(
            test_pdec!("-0.1").checked_round(0, mode).unwrap(),
            test_pdec!("0")
        );
        assert_eq!(
            test_pdec!("-1").checked_round(0, mode).unwrap(),
            test_pdec!("-1")
        );
        assert_eq!(
            test_pdec!("-5.2").checked_round(0, mode).unwrap(),
            test_pdec!("-5")
        );
        assert_eq!(
            PreciseDecimal::MAX.checked_round(0, mode).unwrap(),
            test_pdec!("57896044618658097711785492504343953926634")
        );
        assert_eq!(
            PreciseDecimal::MIN.checked_round(0, mode).unwrap(),
            test_pdec!("-57896044618658097711785492504343953926634")
        );
    }

    #[test]
    fn test_rounding_away_from_zero_precise_decimal() {
        let mode = RoundingMode::AwayFromZero;
        assert_eq!(
            test_pdec!("1.2").checked_round(0, mode).unwrap(),
            test_pdec!("2")
        );
        assert_eq!(
            test_pdec!("1.0").checked_round(0, mode).unwrap(),
            test_pdec!("1")
        );
        assert_eq!(
            test_pdec!("0.9").checked_round(0, mode).unwrap(),
            test_pdec!("1")
        );
        assert_eq!(
            test_pdec!("0").checked_round(0, mode).unwrap(),
            test_pdec!("0")
        );
        assert_eq!(
            test_pdec!("-0.1").checked_round(0, mode).unwrap(),
            test_pdec!("-1")
        );
        assert_eq!(
            test_pdec!("-1").checked_round(0, mode).unwrap(),
            test_pdec!("-1")
        );
        assert_eq!(
            test_pdec!("-5.2").checked_round(0, mode).unwrap(),
            test_pdec!("-6")
        );

        // below shall return None due to overflow
        assert!(PreciseDecimal::MIN.checked_round(0, mode).is_none());
        assert!(test_pdec!("-57896044618658097711785492504343953926634.1")
            .checked_round(0, mode)
            .is_none());
        assert!(PreciseDecimal::MAX.checked_round(0, mode).is_none());
        assert!(test_pdec!("57896044618658097711785492504343953926634.1")
            .checked_round(0, mode)
            .is_none());
    }

    #[test]
    fn test_rounding_midpoint_toward_zero_precise_decimal() {
        let mode = RoundingMode::ToNearestMidpointTowardZero;
        //3.5 -> 3`, `-3.5 -> -3
        assert_eq!(
            test_pdec!("5.5").checked_round(0, mode).unwrap(),
            test_pdec!("5")
        );
        assert_eq!(
            test_pdec!("2.5").checked_round(0, mode).unwrap(),
            test_pdec!("2")
        );
        assert_eq!(
            test_pdec!("1.6").checked_round(0, mode).unwrap(),
            test_pdec!("2")
        );
        assert_eq!(
            test_pdec!("1.1").checked_round(0, mode).unwrap(),
            test_pdec!("1")
        );
        assert_eq!(
            test_pdec!("1.0").checked_round(0, mode).unwrap(),
            test_pdec!("1")
        );
        assert_eq!(
            test_pdec!("-1.0").checked_round(0, mode).unwrap(),
            test_pdec!("-1")
        );
        assert_eq!(
            test_pdec!("-1.1").checked_round(0, mode).unwrap(),
            test_pdec!("-1")
        );
        assert_eq!(
            test_pdec!("-1.6").checked_round(0, mode).unwrap(),
            test_pdec!("-2")
        );
        assert_eq!(
            test_pdec!("-2.5").checked_round(0, mode).unwrap(),
            test_pdec!("-2")
        );
        assert_eq!(
            test_pdec!("-5.5").checked_round(0, mode).unwrap(),
            test_pdec!("-5")
        );

        assert_eq!(
            test_pdec!("-57896044618658097711785492504343953926634.5")
                .checked_round(0, mode)
                .unwrap(),
            test_pdec!("-57896044618658097711785492504343953926634")
        );
        assert_eq!(
            test_pdec!("57896044618658097711785492504343953926634.5")
                .checked_round(0, mode)
                .unwrap(),
            test_pdec!("57896044618658097711785492504343953926634")
        );

        assert!(PreciseDecimal::MIN.checked_round(0, mode).is_none());
        assert!(test_pdec!("-57896044618658097711785492504343953926634.6")
            .checked_round(0, mode)
            .is_none());
        assert!(PreciseDecimal::MAX.checked_round(0, mode).is_none());
        assert!(test_pdec!("57896044618658097711785492504343953926634.6")
            .checked_round(0, mode)
            .is_none());
    }

    #[test]
    fn test_rounding_midpoint_away_from_zero_precise_decimal() {
        let mode = RoundingMode::ToNearestMidpointAwayFromZero;
        assert_eq!(
            test_pdec!("5.5").checked_round(0, mode).unwrap(),
            test_pdec!("6")
        );
        assert_eq!(
            test_pdec!("2.5").checked_round(0, mode).unwrap(),
            test_pdec!("3")
        );
        assert_eq!(
            test_pdec!("1.6").checked_round(0, mode).unwrap(),
            test_pdec!("2")
        );
        assert_eq!(
            test_pdec!("1.1").checked_round(0, mode).unwrap(),
            test_pdec!("1")
        );
        assert_eq!(
            test_pdec!("1.0").checked_round(0, mode).unwrap(),
            test_pdec!("1")
        );
        assert_eq!(
            test_pdec!("-1.0").checked_round(0, mode).unwrap(),
            test_pdec!("-1")
        );
        assert_eq!(
            test_pdec!("-1.1").checked_round(0, mode).unwrap(),
            test_pdec!("-1")
        );
        assert_eq!(
            test_pdec!("-1.6").checked_round(0, mode).unwrap(),
            test_pdec!("-2")
        );
        assert_eq!(
            test_pdec!("-2.5").checked_round(0, mode).unwrap(),
            test_pdec!("-3")
        );
        assert_eq!(
            test_pdec!("-5.5").checked_round(0, mode).unwrap(),
            test_pdec!("-6")
        );

        assert_eq!(
            test_pdec!("-57896044618658097711785492504343953926634.4")
                .checked_round(0, mode)
                .unwrap(),
            test_pdec!("-57896044618658097711785492504343953926634")
        );
        assert_eq!(
            test_pdec!("57896044618658097711785492504343953926634.4")
                .checked_round(0, mode)
                .unwrap(),
            test_pdec!("57896044618658097711785492504343953926634")
        );

        assert!(PreciseDecimal::MIN.checked_round(0, mode).is_none());
        assert!(test_pdec!("-57896044618658097711785492504343953926634.5")
            .checked_round(0, mode)
            .is_none());
        assert!(PreciseDecimal::MAX.checked_round(0, mode).is_none());
        assert!(test_pdec!("57896044618658097711785492504343953926634.5")
            .checked_round(0, mode)
            .is_none());
    }

    #[test]
    fn test_rounding_midpoint_nearest_even_precise_decimal() {
        let mode = RoundingMode::ToNearestMidpointToEven;
        assert_eq!(
            test_pdec!("5.5").checked_round(0, mode).unwrap(),
            test_pdec!("6")
        );
        assert_eq!(
            test_pdec!("2.5").checked_round(0, mode).unwrap(),
            test_pdec!("2")
        );
        assert_eq!(
            test_pdec!("1.6").checked_round(0, mode).unwrap(),
            test_pdec!("2")
        );
        assert_eq!(
            test_pdec!("1.1").checked_round(0, mode).unwrap(),
            test_pdec!("1")
        );
        assert_eq!(
            test_pdec!("1.0").checked_round(0, mode).unwrap(),
            test_pdec!("1")
        );
        assert_eq!(
            test_pdec!("-1.0").checked_round(0, mode).unwrap(),
            test_pdec!("-1")
        );
        assert_eq!(
            test_pdec!("-1.1").checked_round(0, mode).unwrap(),
            test_pdec!("-1")
        );
        assert_eq!(
            test_pdec!("-1.6").checked_round(0, mode).unwrap(),
            test_pdec!("-2")
        );
        assert_eq!(
            test_pdec!("-2.5").checked_round(0, mode).unwrap(),
            test_pdec!("-2")
        );
        assert_eq!(
            test_pdec!("-5.5").checked_round(0, mode).unwrap(),
            test_pdec!("-6")
        );

        assert_eq!(
            test_pdec!("-57896044618658097711785492504343953926634.5")
                .checked_round(0, mode)
                .unwrap(),
            test_pdec!("-57896044618658097711785492504343953926634")
        );
        assert_eq!(
            test_pdec!("57896044618658097711785492504343953926634.5")
                .checked_round(0, mode)
                .unwrap(),
            test_pdec!("57896044618658097711785492504343953926634")
        );
        assert!(PreciseDecimal::MIN.checked_round(0, mode).is_none());
        assert!(test_pdec!("-57896044618658097711785492504343953926634.6")
            .checked_round(0, mode)
            .is_none());
        assert!(PreciseDecimal::MAX.checked_round(0, mode).is_none());
        assert!(test_pdec!("57896044618658097711785492504343953926634.6")
            .checked_round(0, mode)
            .is_none());
    }

    #[test]
    fn test_various_decimal_places_precise_decimal() {
        let num = test_pdec!("2.4595");
        let mode = RoundingMode::AwayFromZero;
        assert_eq!(num.checked_round(0, mode).unwrap(), test_pdec!("3"));
        assert_eq!(num.checked_round(1, mode).unwrap(), test_pdec!("2.5"));
        assert_eq!(num.checked_round(2, mode).unwrap(), test_pdec!("2.46"));
        assert_eq!(num.checked_round(3, mode).unwrap(), test_pdec!("2.46"));

        assert_eq!(
            test_pdec!(
                "57896044618658097711785492504343953926633.992332820282019728792003956564819967"
            )
            .checked_round(1, mode)
            .unwrap(),
            test_pdec!("57896044618658097711785492504343953926634.0")
        );
        assert_eq!(
            test_pdec!(
                "-57896044618658097711785492504343953926633.992332820282019728792003956564819967"
            )
            .checked_round(1, mode)
            .unwrap(),
            test_pdec!("-57896044618658097711785492504343953926634.0")
        );

        let mode = RoundingMode::ToZero;
        assert_eq!(num.checked_round(0, mode).unwrap(), test_pdec!("2"));
        assert_eq!(num.checked_round(1, mode).unwrap(), test_pdec!("2.4"));
        assert_eq!(num.checked_round(2, mode).unwrap(), test_pdec!("2.45"));
        assert_eq!(num.checked_round(3, mode).unwrap(), test_pdec!("2.459"));
        let mode = RoundingMode::ToPositiveInfinity;
        assert_eq!(num.checked_round(0, mode).unwrap(), test_pdec!("3"));
        assert_eq!(num.checked_round(1, mode).unwrap(), test_pdec!("2.5"));
        assert_eq!(num.checked_round(2, mode).unwrap(), test_pdec!("2.46"));
        assert_eq!(num.checked_round(3, mode).unwrap(), test_pdec!("2.46"));
        let mode = RoundingMode::ToNegativeInfinity;
        assert_eq!(num.checked_round(0, mode).unwrap(), test_pdec!("2"));
        assert_eq!(num.checked_round(1, mode).unwrap(), test_pdec!("2.4"));
        assert_eq!(num.checked_round(2, mode).unwrap(), test_pdec!("2.45"));
        assert_eq!(num.checked_round(3, mode).unwrap(), test_pdec!("2.459"));
        let mode = RoundingMode::ToNearestMidpointAwayFromZero;
        assert_eq!(num.checked_round(0, mode).unwrap(), test_pdec!("2"));
        assert_eq!(num.checked_round(1, mode).unwrap(), test_pdec!("2.5"));
        assert_eq!(num.checked_round(2, mode).unwrap(), test_pdec!("2.46"));
        assert_eq!(num.checked_round(3, mode).unwrap(), test_pdec!("2.46"));
        let mode = RoundingMode::ToNearestMidpointTowardZero;
        assert_eq!(num.checked_round(0, mode).unwrap(), test_pdec!("2"));
        assert_eq!(num.checked_round(1, mode).unwrap(), test_pdec!("2.5"));
        assert_eq!(num.checked_round(2, mode).unwrap(), test_pdec!("2.46"));
        assert_eq!(num.checked_round(3, mode).unwrap(), test_pdec!("2.459"));
        let mode = RoundingMode::ToNearestMidpointToEven;
        assert_eq!(num.checked_round(0, mode).unwrap(), test_pdec!("2"));
        assert_eq!(num.checked_round(1, mode).unwrap(), test_pdec!("2.5"));
        assert_eq!(num.checked_round(2, mode).unwrap(), test_pdec!("2.46"));
        assert_eq!(num.checked_round(3, mode).unwrap(), test_pdec!("2.46"));

        let num = test_pdec!("-2.4595");
        let mode = RoundingMode::AwayFromZero;
        assert_eq!(num.checked_round(0, mode).unwrap(), test_pdec!("-3"));
        assert_eq!(num.checked_round(1, mode).unwrap(), test_pdec!("-2.5"));
        assert_eq!(num.checked_round(2, mode).unwrap(), test_pdec!("-2.46"));
        assert_eq!(num.checked_round(3, mode).unwrap(), test_pdec!("-2.46"));
        let mode = RoundingMode::ToZero;
        assert_eq!(num.checked_round(0, mode).unwrap(), test_pdec!("-2"));
        assert_eq!(num.checked_round(1, mode).unwrap(), test_pdec!("-2.4"));
        assert_eq!(num.checked_round(2, mode).unwrap(), test_pdec!("-2.45"));
        assert_eq!(num.checked_round(3, mode).unwrap(), test_pdec!("-2.459"));
        let mode = RoundingMode::ToPositiveInfinity;
        assert_eq!(num.checked_round(0, mode).unwrap(), test_pdec!("-2"));
        assert_eq!(num.checked_round(1, mode).unwrap(), test_pdec!("-2.4"));
        assert_eq!(num.checked_round(2, mode).unwrap(), test_pdec!("-2.45"));
        assert_eq!(num.checked_round(3, mode).unwrap(), test_pdec!("-2.459"));
        let mode = RoundingMode::ToNegativeInfinity;
        assert_eq!(num.checked_round(0, mode).unwrap(), test_pdec!("-3"));
        assert_eq!(num.checked_round(1, mode).unwrap(), test_pdec!("-2.5"));
        assert_eq!(num.checked_round(2, mode).unwrap(), test_pdec!("-2.46"));
        assert_eq!(num.checked_round(3, mode).unwrap(), test_pdec!("-2.46"));
        let mode = RoundingMode::ToNearestMidpointAwayFromZero;
        assert_eq!(num.checked_round(0, mode).unwrap(), test_pdec!("-2"));
        assert_eq!(num.checked_round(1, mode).unwrap(), test_pdec!("-2.5"));
        assert_eq!(num.checked_round(2, mode).unwrap(), test_pdec!("-2.46"));
        assert_eq!(num.checked_round(3, mode).unwrap(), test_pdec!("-2.46"));
        let mode = RoundingMode::ToNearestMidpointTowardZero;
        assert_eq!(num.checked_round(0, mode).unwrap(), test_pdec!("-2"));
        assert_eq!(num.checked_round(1, mode).unwrap(), test_pdec!("-2.5"));
        assert_eq!(num.checked_round(2, mode).unwrap(), test_pdec!("-2.46"));
        assert_eq!(num.checked_round(3, mode).unwrap(), test_pdec!("-2.459"));
        let mode = RoundingMode::ToNearestMidpointToEven;
        assert_eq!(num.checked_round(0, mode).unwrap(), test_pdec!("-2"));
        assert_eq!(num.checked_round(1, mode).unwrap(), test_pdec!("-2.5"));
        assert_eq!(num.checked_round(2, mode).unwrap(), test_pdec!("-2.46"));
        assert_eq!(num.checked_round(3, mode).unwrap(), test_pdec!("-2.46"));
    }

    #[test]
    fn test_encode_decimal_value_precise_decimal() {
        let pdec = test_pdec!("0");
        let bytes = scrypto_encode(&pdec).unwrap();
        assert_eq!(bytes, {
            let mut a = [0; 34];
            a[0] = SCRYPTO_SBOR_V1_PAYLOAD_PREFIX;
            a[1] = ScryptoValueKind::Custom(ScryptoCustomValueKind::PreciseDecimal).as_u8();
            a
        });
    }

    #[test]
    fn test_decode_decimal_value_precise_decimal() {
        let pdec = test_pdec!("1.23456789");
        let bytes = scrypto_encode(&pdec).unwrap();
        let decoded: PreciseDecimal = scrypto_decode(&bytes).unwrap();
        assert_eq!(decoded, test_pdec!("1.23456789"));
    }

    #[test]
    fn test_from_str_precise_decimal() {
        let pdec = PreciseDecimal::from_str("5.0").unwrap();
        assert_eq!(pdec.to_string(), "5");
    }

    #[test]
    fn test_from_str_failure_precise_decimal() {
        let pdec = PreciseDecimal::from_str("non_decimal_value");
        assert_eq!(pdec, Err(ParsePreciseDecimalError::InvalidDigit));
    }

    macro_rules! test_from_into_decimal_precise_decimal {
        ($(($from:expr, $expected:expr, $suffix:expr)),*) => {
            paste!{
            $(
                #[test]
                fn [<test_from_into_decimal_precise_decimal_ $suffix>]() {
                    let dec = test_dec!($from);
                    let pdec = PreciseDecimal::from(dec);
                    assert_eq!(pdec.to_string(), $expected);

                    let pdec: PreciseDecimal = dec.into();
                    assert_eq!(pdec.to_string(), $expected);
                }
            )*
            }
        };
    }

    test_from_into_decimal_precise_decimal! {
        ("12345678.123456789012345678", "12345678.123456789012345678", 1),
        ("0.000000000000000001", "0.000000000000000001", 2),
        ("-0.000000000000000001", "-0.000000000000000001", 3),
        ("5", "5", 4),
        ("12345678.1", "12345678.1", 5)
    }

    macro_rules! test_try_from_integer_overflow {
        ($(($from:expr, $suffix:expr)),*) => {
            paste!{
            $(
                #[test]
                fn [<test_try_from_integer_overflow_ $suffix>]() {
                    let err = PreciseDecimal::try_from($from).unwrap_err();
                    assert_eq!(err, ParsePreciseDecimalError::Overflow)
                }
            )*
            }
        };
    }

    test_try_from_integer_overflow! {
        (I192::MAX, 1),
        (I192::MIN, 2),
        (I256::MAX, 3),
        (I256::MIN, 4),
        (I320::MAX, 5),
        (I320::MIN, 6),
        (I448::MAX, 7),
        (I448::MIN, 8),
        (I512::MAX, 9),
        (I512::MIN, 10),
        // maximal PreciseDecimal integer part + 1
        (I256::MAX/(I256::from(10).pow(PreciseDecimal::SCALE)) + I256::ONE, 11),
        // minimal PreciseDecimal integer part - 1
        (I256::MIN/(I256::from(10).pow(PreciseDecimal::SCALE)) - I256::ONE, 12),
        (U192::MAX, 13),
        (U256::MAX, 14),
        (U320::MAX, 15),
        (U448::MAX, 16),
        (U512::MAX, 17)
    }

    macro_rules! test_try_from_integer {
        ($(($from:expr, $expected:expr, $suffix:expr)),*) => {
            paste!{
            $(
                #[test]
                fn [<test_try_from_integer_ $suffix>]() {
                    let dec = PreciseDecimal::try_from($from).unwrap();
                    assert_eq!(dec.to_string(), $expected)
                }
            )*
            }
        };
    }

    test_try_from_integer! {
        (I192::ONE, "1", 1),
        (-I192::ONE, "-1", 2),
        (I256::ONE, "1", 3),
        (-I256::ONE, "-1", 4),
        (I320::ONE, "1", 5),
        (-I320::ONE, "-1", 6),
        (I448::ONE, "1", 7),
        (-I448::ONE, "-1", 8),
        (I512::ONE, "1", 9),
        (-I512::ONE, "-1", 10),
        // maximal PreciseDecimal integer part
        (I256::MAX/(I256::from(10).pow(PreciseDecimal::SCALE)), "57896044618658097711785492504343953926634", 11),
        // minimal PreciseDecimal integer part
        (I256::MIN/(I256::from(10).pow(PreciseDecimal::SCALE)), "-57896044618658097711785492504343953926634", 12),
        (U192::MIN, "0", 13),
        (U256::MIN, "0", 14),
        (U320::MIN, "0", 15),
        (U448::MIN, "0", 16),
        (U512::MIN, "0", 17)
    }

    #[test]
    fn test_truncate_precise_decimal_towards_zero() {
        for (pdec, dec) in [
            (
                test_pdec!("12345678.123456789012345678901234567890123456"),
                test_dec!("12345678.123456789012345678"),
            ),
            (test_pdec!(1), test_dec!(1)),
            (test_pdec!("123.5"), test_dec!("123.5")),
            (
                test_pdec!("-12345678.123456789012345678901234567890123456"),
                test_dec!("-12345678.123456789012345678"),
            ),
            (
                test_pdec!("-12345678.123456789012345678101234567890123456"),
                test_dec!("-12345678.123456789012345678"),
            ),
        ] {
            assert_eq!(pdec.checked_truncate(RoundingMode::ToZero).unwrap(), dec);
        }
    }

    #[test]
    fn test_truncate_precise_decimal_away_from_zero() {
        for (pdec, dec) in [
            (
                test_pdec!("12345678.123456789012345678901234567890123456"),
                test_dec!("12345678.123456789012345679"),
            ),
            (test_pdec!(1), test_dec!(1)),
            (test_pdec!("123.5"), test_dec!("123.5")),
            (
                test_pdec!("-12345678.123456789012345678901234567890123456"),
                test_dec!("-12345678.123456789012345679"),
            ),
            (
                test_pdec!("-12345678.123456789012345678101234567890123456"),
                test_dec!("-12345678.123456789012345679"),
            ),
        ] {
            assert_eq!(
                pdec.checked_truncate(RoundingMode::AwayFromZero).unwrap(),
                dec
            );
        }
    }

    #[test]
    fn test_sqrt() {
        let sqrt_of_42 = test_pdec!(42).checked_sqrt();
        let sqrt_of_0 = test_pdec!(0).checked_sqrt();
        let sqrt_of_negative = test_pdec!("-1").checked_sqrt();
        assert_eq!(
            sqrt_of_42.unwrap(),
            test_pdec!("6.480740698407860230965967436087996657")
        );
        assert_eq!(sqrt_of_0.unwrap(), test_pdec!(0));
        assert_eq!(sqrt_of_negative, None);
    }

    #[test]
    fn test_cbrt() {
        let cbrt_of_42 = test_pdec!(42).checked_cbrt().unwrap();
        let cbrt_of_0 = test_pdec!(0).checked_cbrt().unwrap();
        let cbrt_of_negative_42 = test_pdec!("-42").checked_cbrt().unwrap();
        assert_eq!(
            cbrt_of_42,
            test_pdec!("3.476026644886449786739865219004537434")
        );
        assert_eq!(cbrt_of_0, test_pdec!("0"));
        assert_eq!(
            cbrt_of_negative_42,
            test_pdec!("-3.476026644886449786739865219004537434")
        );
    }

    #[test]
    fn test_nth_root() {
        let root_4_42 = test_pdec!(42).checked_nth_root(4);
        let root_5_42 = test_pdec!(42).checked_nth_root(5);
        let root_42_42 = test_pdec!(42).checked_nth_root(42);
        let root_neg_4_42 = test_pdec!("-42").checked_nth_root(4);
        let root_neg_5_42 = test_pdec!("-42").checked_nth_root(5);
        let root_0 = test_pdec!(42).checked_nth_root(0);
        assert_eq!(
            root_4_42.unwrap(),
            test_pdec!("2.545729895021830518269788960576288685")
        );
        assert_eq!(
            root_5_42.unwrap(),
            test_pdec!("2.111785764966753912732567330550233486")
        );
        assert_eq!(
            root_42_42.unwrap(),
            test_pdec!("1.093072057934823618682784731855625786")
        );
        assert_eq!(root_neg_4_42, None);
        assert_eq!(
            root_neg_5_42.unwrap(),
            test_pdec!("-2.111785764966753912732567330550233486")
        );
        assert_eq!(root_0, None);
    }

    #[test]
    fn no_panic_with_36_decimal_places() {
        // Arrange
        let string = "1.111111111111111111111111111111111111";

        // Act
        let decimal = PreciseDecimal::from_str(string);

        // Assert
        assert!(decimal.is_ok())
    }

    #[test]
    fn no_panic_with_37_decimal_places() {
        // Arrange
        let string = "1.1111111111111111111111111111111111111";

        // Act
        let decimal = PreciseDecimal::from_str(string);

        // Assert
        assert_matches!(
            decimal,
            Err(ParsePreciseDecimalError::MoreThanThirtySixDecimalPlaces)
        );
    }

    #[test]
    fn test_neg_precise_decimal() {
        let d = PreciseDecimal::ONE;
        assert_eq!(-d, test_pdec!("-1"));
        let d = PreciseDecimal::MAX;
        assert_eq!(-d, PreciseDecimal(I256::MIN + I256::ONE));
    }

    #[test]
    #[should_panic(expected = "Overflow")]
    fn test_neg_precise_decimal_panic() {
        let d = PreciseDecimal::MIN;
        let _ = -d;
    }

    // These tests make sure that any basic arithmetic operation
    // between Decimal and PreciseDecimal produces a PreciseDecimal, no matter the order.
    // Additionally result of such operation shall be equal, if operands are derived from the same
    // value
    // Example:
    //   Decimal(10) * PreciseDecimal(10) -> PreciseDecimal(100)
    //   PreciseDecimal(10) * Decimal(10) -> PreciseDecimal(100)
    #[test]
    fn test_arith_precise_decimal_decimal() {
        let p1 = PreciseDecimal::from(Decimal::MAX);
        let d1 = Decimal::from(2);
        let d2 = Decimal::MAX;
        let p2 = PreciseDecimal::from(2);
        assert_eq!(p1.checked_mul(d1).unwrap(), d2.checked_mul(p2).unwrap());
        assert_eq!(p1.checked_div(d1).unwrap(), d2.checked_div(p2).unwrap());
        assert_eq!(p1.checked_add(d1).unwrap(), d2.checked_add(p2).unwrap());
        assert_eq!(p1.checked_sub(d1).unwrap(), d2.checked_sub(p2).unwrap());

        let p1 = PreciseDecimal::from(Decimal::MIN);
        let d1 = Decimal::from(2);
        let d2 = Decimal::MIN;
        let p2 = PreciseDecimal::from(2);
        assert_eq!(p1.checked_mul(d1).unwrap(), d2.checked_mul(p2).unwrap());
        assert_eq!(p1.checked_div(d1).unwrap(), d2.checked_div(p2).unwrap());
        assert_eq!(p1.checked_add(d1).unwrap(), d2.checked_add(p2).unwrap());
        assert_eq!(p1.checked_sub(d1).unwrap(), d2.checked_sub(p2).unwrap());

        let p1 = test_pdec!("0.000001");
        let d1 = test_dec!("0.001");
        let d2 = test_dec!("0.000001");
        let p2 = test_pdec!("0.001");
        assert_eq!(p1.checked_mul(d1).unwrap(), d2.checked_mul(p2).unwrap());
        assert_eq!(p1.checked_div(d1).unwrap(), d2.checked_div(p2).unwrap());
        assert_eq!(p1.checked_add(d1).unwrap(), d2.checked_add(p2).unwrap());
        assert_eq!(p1.checked_sub(d1).unwrap(), d2.checked_sub(p2).unwrap());

        let p1 = test_pdec!("0.000000000000000001");
        let d1 = Decimal::MIN;
        let d2 = test_dec!("0.000000000000000001");
        let p2 = PreciseDecimal::from(Decimal::MIN);
        assert_eq!(p1.checked_mul(d1).unwrap(), d2.checked_mul(p2).unwrap());
        assert_eq!(p1.checked_div(d1).unwrap(), d2.checked_div(p2).unwrap());
        assert_eq!(p1.checked_add(d1).unwrap(), d2.checked_add(p2).unwrap());
        assert_eq!(p1.checked_sub(d1).unwrap(), d2.checked_sub(p2).unwrap());

        let p1 = PreciseDecimal::ZERO;
        let d1 = Decimal::ONE;
        let d2 = Decimal::ZERO;
        let p2 = PreciseDecimal::ONE;
        assert_eq!(p1.checked_mul(d1).unwrap(), d2.checked_mul(p2).unwrap());
        assert_eq!(p1.checked_div(d1).unwrap(), d2.checked_div(p2).unwrap());
        assert_eq!(p1.checked_add(d1).unwrap(), d2.checked_add(p2).unwrap());
        assert_eq!(p1.checked_sub(d1).unwrap(), d2.checked_sub(p2).unwrap());
    }

    // These tests make sure that any basic arithmetic operation
    // between primitive type and PreciseDecimal produces a PreciseDecimal.
    // Example:
    //   PreciseDecimal(10) * 10_u32 -> PreciseDecimal(100)
    //   10_u32 * PreciseDecimal(10) -> PreciseDecimal(100)
    macro_rules! test_arith_precise_decimal_primitive {
        ($type:ident) => {
            paste! {
                #[test]
                fn [<test_arith_precise_decimal_$type>]() {
                    let p1 = test_pdec!("2");
                    let u1 = 4 as $type;
                    assert_eq!(p1.checked_add(u1).unwrap(), test_pdec!("6"));
                    assert_eq!(p1.checked_sub(u1).unwrap(), test_pdec!("-2"));
                    assert_eq!(p1.checked_mul(u1).unwrap(), test_pdec!("8"));
                    assert_eq!(p1.checked_div(u1).unwrap(), test_pdec!("0.5"));

                    let p1 = test_pdec!("2");
                    let u1 = $type::MAX;
                    let p2 = PreciseDecimal::from($type::MAX);
                    assert_eq!(p1.checked_add(u1).unwrap(), p1.checked_add(p2).unwrap());
                    assert_eq!(p1.checked_sub(u1).unwrap(), p1.checked_sub(p2).unwrap());
                    assert_eq!(p1.checked_mul(u1).unwrap(), p1.checked_mul(p2).unwrap());
                    assert_eq!(p1.checked_div(u1).unwrap(), p1.checked_div(p2).unwrap());

                    let p1 = PreciseDecimal::from($type::MIN);
                    let u1 = 2 as $type;
                    let p2 = test_pdec!("2");
                    assert_eq!(p1.checked_add(u1).unwrap(), p1.checked_add(p2).unwrap());
                    assert_eq!(p1.checked_sub(u1).unwrap(), p1.checked_sub(p2).unwrap());
                    assert_eq!(p1.checked_mul(u1).unwrap(), p1.checked_mul(p2).unwrap());
                    assert_eq!(p1.checked_div(u1).unwrap(), p1.checked_div(p2).unwrap());
                }
            }
        };
    }
    test_arith_precise_decimal_primitive!(u8);
    test_arith_precise_decimal_primitive!(u16);
    test_arith_precise_decimal_primitive!(u32);
    test_arith_precise_decimal_primitive!(u64);
    test_arith_precise_decimal_primitive!(u128);
    test_arith_precise_decimal_primitive!(usize);
    test_arith_precise_decimal_primitive!(i8);
    test_arith_precise_decimal_primitive!(i16);
    test_arith_precise_decimal_primitive!(i32);
    test_arith_precise_decimal_primitive!(i64);
    test_arith_precise_decimal_primitive!(i128);
    test_arith_precise_decimal_primitive!(isize);

    macro_rules! test_arith_precise_decimal_integer {
        ($type:ident) => {
            paste! {
                #[test]
                fn [<test_arith_precise_decimal_$type:lower>]() {
                    let d1 = test_pdec!("2");
                    let u1 = $type::try_from(4).unwrap();
                    let u2 = $type::try_from(2).unwrap();
                    let d2 = test_pdec!("4");
                    assert_eq!(d1.checked_add(u1).unwrap(), u2.checked_add(d2).unwrap());
                    assert_eq!(d1.checked_sub(u1).unwrap(), u2.checked_sub(d2).unwrap());
                    assert_eq!(d1.checked_mul(u1).unwrap(), u2.checked_mul(d2).unwrap());
                    assert_eq!(d1.checked_div(u1).unwrap(), u2.checked_div(d2).unwrap());

                    let d1 = test_pdec!("2");
                    let u1 = $type::MAX;
                    assert!(d1.checked_add(u1).is_none());
                    assert!(d1.checked_sub(u1).is_none());
                    assert!(d1.checked_mul(u1).is_none());
                    assert!(d1.checked_div(u1).is_none());

                    let d1 = PreciseDecimal::MAX;
                    let u1 = $type::try_from(2).unwrap();
                    assert_eq!(d1.checked_add(u1), None);
                    assert_eq!(d1.checked_sub(u1).unwrap(), PreciseDecimal::MAX - test_dec!("2"));
                    assert_eq!(d1.checked_mul(u1), None);
                    assert_eq!(d1.checked_div(u1).unwrap(), PreciseDecimal::MAX / test_dec!("2"));
                }
            }
        };
    }
    test_arith_precise_decimal_integer!(I192);
    test_arith_precise_decimal_integer!(I256);
    test_arith_precise_decimal_integer!(I320);
    test_arith_precise_decimal_integer!(I448);
    test_arith_precise_decimal_integer!(I512);
    test_arith_precise_decimal_integer!(U192);
    test_arith_precise_decimal_integer!(U256);
    test_arith_precise_decimal_integer!(U320);
    test_arith_precise_decimal_integer!(U448);
    test_arith_precise_decimal_integer!(U512);

    macro_rules! test_math_operands_decimal {
        ($type:ident) => {
            paste! {
                #[test]
                fn [<test_math_operands_precise_decimal_$type:lower>]() {
                    let d1 = test_pdec!("2");
                    let u1 = $type::try_from(4).unwrap();
                    assert_eq!(d1 + u1, test_pdec!("6"));
                    assert_eq!(d1 - u1, test_pdec!("-2"));
                    assert_eq!(d1 * u1, test_pdec!("8"));
                    assert_eq!(d1 / u1, test_pdec!("0.5"));

                    let u1 = $type::try_from(2).unwrap();
                    let d1 = test_pdec!("4");
                    assert_eq!(u1 + d1, test_pdec!("6"));
                    assert_eq!(u1 - d1, test_pdec!("-2"));
                    assert_eq!(u1 * d1, test_pdec!("8"));
                    assert_eq!(u1 / d1, test_pdec!("0.5"));

                    let u1 = $type::try_from(4).unwrap();

                    let mut d1 = test_pdec!("2");
                    d1 += u1;
                    assert_eq!(d1, test_pdec!("6"));

                    let mut d1 = test_pdec!("2");
                    d1 -= u1;
                    assert_eq!(d1, test_pdec!("-2"));

                    let mut d1 = test_pdec!("2");
                    d1 *= u1;
                    assert_eq!(d1, test_pdec!("8"));

                    let mut d1 = test_pdec!("2");
                    d1 /= u1;
                    assert_eq!(d1, test_pdec!("0.5"));
                }

                #[test]
                #[should_panic(expected = "Overflow")]
                fn [<test_math_add_precise_decimal_$type:lower _panic>]() {
                    let d1 = PreciseDecimal::MAX;
                    let u1 = $type::try_from(1).unwrap();
                    let _ = d1 + u1;
                }

                #[test]
                #[should_panic(expected = "Overflow")]
                fn [<test_math_add_$type:lower _xprecise_decimal_panic>]() {
                    let d1 = PreciseDecimal::MAX;
                    let u1 = $type::try_from(1).unwrap();
                    let _ = u1 + d1;
                }

                #[test]
                #[should_panic(expected = "Overflow")]
                fn [<test_math_sub_precise_decimal_$type:lower _panic>]() {
                    let d1 = PreciseDecimal::MIN;
                    let u1 = $type::try_from(1).unwrap();
                    let _ = d1 - u1;
                }

                #[test]
                #[should_panic(expected = "Overflow")]
                fn [<test_math_sub_$type:lower _xprecise_precise_decimal_panic>]() {
                    let d1 = PreciseDecimal::MIN;
                    let u1 = $type::try_from(1).unwrap();
                    let _ = u1 - d1;
                }

                #[test]
                #[should_panic(expected = "Overflow")]
                fn [<test_math_mul_precise_decimal_$type:lower _panic>]() {
                    let d1 = PreciseDecimal::MAX;
                    let u1 = $type::try_from(2).unwrap();
                    let _ = d1 * u1;
                }

                #[test]
                #[should_panic(expected = "Overflow")]
                fn [<test_math_mul_$type:lower _xprecise_decimal_panic>]() {
                    let d1 = PreciseDecimal::MAX;
                    let u1 = $type::try_from(2).unwrap();
                    let _ = u1 * d1;
                }

                #[test]
                #[should_panic(expected = "Overflow")]
                fn [<test_math_div_zero_precise_decimal_$type:lower _panic>]() {
                    let d1 = PreciseDecimal::MAX;
                    let u1 = $type::try_from(0).unwrap();
                    let _ = d1 / u1;
                }

                #[test]
                #[should_panic(expected = "Overflow or division by zero")]
                fn [<test_math_div_zero_$type:lower _xdecimal_panic>]() {
                    let d1 = PreciseDecimal::ZERO;
                    let u1 = $type::try_from(1).unwrap();
                    let _ = u1 / d1;
                }

                #[test]
                #[should_panic(expected = "Overflow")]
                fn [<test_math_add_assign_precise_decimal_$type:lower _panic>]() {
                    let mut d1 = PreciseDecimal::MAX;
                    let u1 = $type::try_from(1).unwrap();
                    d1 += u1;
                }

                #[test]
                #[should_panic(expected = "Overflow")]
                fn [<test_math_sub_assign_precise_decimal_$type:lower _panic>]() {
                    let mut d1 = PreciseDecimal::MIN;
                    let u1 = $type::try_from(1).unwrap();
                    d1 -= u1;
                }

                #[test]
                #[should_panic(expected = "Overflow")]
                fn [<test_math_mul_assign_precise_decimal_$type:lower _panic>]() {
                    let mut d1 = PreciseDecimal::MAX;
                    let u1 = $type::try_from(2).unwrap();
                    d1 *= u1;
                }

                #[test]
                #[should_panic(expected = "Overflow or division by zero")]
                fn [<test_math_div_assign_precise_decimal_$type:lower _panic>]() {
                    let mut d1 = PreciseDecimal::MAX;
                    let u1 = $type::try_from(0).unwrap();
                    d1 /= u1;
                }
            }
        };
    }
    test_math_operands_decimal!(PreciseDecimal);
    test_math_operands_decimal!(u8);
    test_math_operands_decimal!(u16);
    test_math_operands_decimal!(u32);
    test_math_operands_decimal!(u64);
    test_math_operands_decimal!(u128);
    test_math_operands_decimal!(usize);
    test_math_operands_decimal!(i8);
    test_math_operands_decimal!(i16);
    test_math_operands_decimal!(i32);
    test_math_operands_decimal!(i64);
    test_math_operands_decimal!(i128);
    test_math_operands_decimal!(isize);
    test_math_operands_decimal!(I192);
    test_math_operands_decimal!(I256);
    test_math_operands_decimal!(I320);
    test_math_operands_decimal!(I448);
    test_math_operands_decimal!(I512);
    test_math_operands_decimal!(U192);
    test_math_operands_decimal!(U256);
    test_math_operands_decimal!(U320);
    test_math_operands_decimal!(U448);
    test_math_operands_decimal!(U512);

    macro_rules! test_from_primitive_type {
        ($type:ident) => {
            paste! {
                #[test]
                fn [<test_precise_decimal_from_primitive_$type>]() {
                    let v = $type::try_from(1).unwrap();
                    assert_eq!(PreciseDecimal::from(v), test_pdec!(1));

                    if $type::MIN != 0 {
                        let v = $type::try_from(-1).unwrap();
                        assert_eq!(PreciseDecimal::from(v), test_pdec!(-1));
                    }

                    let v = $type::MAX;
                    assert_eq!(PreciseDecimal::from(v), PreciseDecimal::from_str(&v.to_string()).unwrap());

                    let v = $type::MIN;
                    assert_eq!(PreciseDecimal::from(v), PreciseDecimal::from_str(&v.to_string()).unwrap());
                }
            }
        };
    }
    test_from_primitive_type!(u8);
    test_from_primitive_type!(u16);
    test_from_primitive_type!(u32);
    test_from_primitive_type!(u64);
    test_from_primitive_type!(u128);
    test_from_primitive_type!(usize);
    test_from_primitive_type!(i8);
    test_from_primitive_type!(i16);
    test_from_primitive_type!(i32);
    test_from_primitive_type!(i64);
    test_from_primitive_type!(i128);
    test_from_primitive_type!(isize);

    macro_rules! test_to_primitive_type {
        ($type:ident) => {
            paste! {
                #[test]
                fn [<test_precise_decimal_to_primitive_$type>]() {
                    let d = test_pdec!(1);
                    let v = $type::try_from(1).unwrap();
                    assert_eq!($type::try_from(d).unwrap(), v);

                    if $type::MIN != 0 {
                        let d = test_pdec!(-1);
                        let v = $type::try_from(-1).unwrap();
                        assert_eq!($type::try_from(d).unwrap(), v);
                    }

                    let v = $type::MAX;
                    let d = PreciseDecimal::from(v);
                    assert_eq!($type::try_from(d).unwrap(), v);

                    let v = $type::MIN;
                    let d = PreciseDecimal::from(v);
                    assert_eq!($type::try_from(d).unwrap(), v);

                    let d = PreciseDecimal::MAX;
                    let err = $type::try_from(d).unwrap_err();
                    assert_eq!(err, ParsePreciseDecimalError::InvalidDigit);

                    let v = $type::MAX;
                    let d = PreciseDecimal::from(v).checked_add(1).unwrap();
                    let err = $type::try_from(d).unwrap_err();
                    assert_eq!(err, ParsePreciseDecimalError::Overflow);

                    let v = $type::MIN;
                    let d = PreciseDecimal::from(v).checked_sub(1).unwrap();
                    let err = $type::try_from(d).unwrap_err();
                    assert_eq!(err, ParsePreciseDecimalError::Overflow);

                    let d = test_pdec!("1.1");
                    let err = $type::try_from(d).unwrap_err();
                    assert_eq!(err, ParsePreciseDecimalError::InvalidDigit);
                }
            }
        };
    }
    test_to_primitive_type!(u8);
    test_to_primitive_type!(u16);
    test_to_primitive_type!(u32);
    test_to_primitive_type!(u64);
    test_to_primitive_type!(u128);
    test_to_primitive_type!(usize);
    test_to_primitive_type!(i8);
    test_to_primitive_type!(i16);
    test_to_primitive_type!(i32);
    test_to_primitive_type!(i64);
    test_to_primitive_type!(i128);
    test_to_primitive_type!(isize);
}
