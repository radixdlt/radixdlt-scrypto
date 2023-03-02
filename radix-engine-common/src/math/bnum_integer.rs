//! Definitions of safe integers and uints.

use bnum::{BInt, BUint};
use num_bigint::BigInt;
use num_integer::Roots;
use num_traits::{FromPrimitive, One, Pow, ToPrimitive, Zero};
use paste::paste;
use sbor::rust::cmp::{Ord, Ordering, PartialEq, PartialOrd};
use sbor::rust::convert::{From, TryFrom};
use sbor::rust::fmt;
use sbor::rust::ops::{Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign};
use sbor::rust::ops::{BitXor, BitXorAssign, Div, DivAssign};
use sbor::rust::ops::{Mul, MulAssign, Neg, Not, Rem, RemAssign};
use sbor::rust::ops::{Shl, ShlAssign, Shr, ShrAssign, Sub, SubAssign};
use sbor::rust::str::FromStr;
use sbor::rust::string::*;
use sbor::rust::vec::Vec;

pub mod bits;
pub mod convert;
pub mod test;
pub mod test_macros;

macro_rules! types {
    ($($t:ident, $wrap:ty),*) => {
        paste!{
            $(
                /// Provides safe integer arithmetic.
                ///
                /// Operations like `+`, '-', '*', or '/' sometimes produce overflow
                /// which is detected and results in a panic, in of silently
                /// wrapping around.
                ///
                /// The bit length of output type will be the greater one in the math operation,
                /// and if any of the types was signed, then the resulting type will be signed too,
                /// otherwise the output type is unsigned.
                ///
                /// The underlying value can be retrieved through the `.0` index of the
                #[doc = "`" $t "` tuple."]
                ///
                /// # Layout
                ///
                #[doc = "`" $t "` will have the same methods and traits as"]
                /// the built-in counterpart.
                #[derive(Clone , Copy , Eq , Hash)]
                #[repr(transparent)]
                pub struct $t(pub $wrap);

                impl $t {
                    pub const MIN: Self = Self($wrap::MIN);
                    pub const MAX: Self = Self($wrap::MAX);
                    pub const ZERO: Self = Self($wrap::ZERO);
                    pub const ONE: Self = Self($wrap::ONE);
                    pub const TEN: Self = Self($wrap::TEN);
                    pub const BITS: u32 = $wrap::BITS as u32;
                    pub const BYTES: u32 = $wrap::BYTES as u32;
                    pub const N: usize = ($wrap::BYTES / 8) as usize;
                }

                impl Default for $t {
                    fn default() -> Self {
                        Self::ZERO
                    }
                }

                impl fmt::Debug for $t {
                    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                        self.0.fmt(f)
                    }
                }

                impl fmt::Display for $t {
                    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                        self.0.fmt(f)
                    }
                }

                impl Zero for $t {
                    fn zero() -> Self {
                        Self::ZERO
                    }

                    fn is_zero(&self) -> bool {
                        $wrap::ZERO == self.0
                    }

                    fn set_zero(&mut self) {
                        self.0 = $wrap::ZERO;
                    }
                }

                impl One for $t {
                    fn one() -> Self {
                        Self::ONE
                    }
                }

                impl Ord for $t {
                    fn cmp(&self, other: &Self) -> Ordering {
                        self.0.cmp(&other.0)
                    }
                }

                impl PartialOrd for $t {
                    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                        Some(self.cmp(other))
                    }
                }

                impl PartialEq for $t {
                    fn eq(&self, other: &Self) -> bool {
                        self.0 == other.0
                    }
                }
            )*
        }
    };
}
types! {
    BnumI256, BInt::<4>,
    BnumI384, BInt::<6>,
    BnumI512, BInt::<8>,
    BnumI768, BInt::<12>,
    BnumU256, BUint::<4>,
    BnumU384, BUint::<6>,
    BnumU512, BUint::<8>,
    BnumU768, BUint::<12>
}

pub trait Sqrt {
    fn sqrt(self) -> Self;
}

pub trait Cbrt {
    fn cbrt(self) -> Self;
}

pub trait NthRoot {
    fn nth_root(self, n: u32) -> Self;
}

pub trait CheckedSub {
    fn checked_sub(self, other: Self) -> Option<Self>
    where
        Self: Sized;
}

pub trait CheckedMul {
    fn checked_mul(self, other: Self) -> Option<Self>
    where
        Self: Sized;
}

macro_rules! forward_ref_unop {
    (impl $imp:ident, $method:ident for $t:ty) => {
        impl $imp for &$t {
            type Output = <$t as $imp>::Output;

            #[inline]
            fn $method(self) -> <$t as $imp>::Output {
                $imp::$method(*self)
            }
        }
    };
}

macro_rules! forward_ref_binop {
    (impl $imp:ident, $method:ident for $t:ty, $u:ty) => {
        impl<'a> $imp<$u> for &'a $t {
            type Output = <$t as $imp<$u>>::Output;

            #[inline]
            fn $method(self, other: $u) -> <$t as $imp<$u>>::Output {
                $imp::$method(*self, other)
            }
        }

        impl $imp<&$u> for $t {
            type Output = <$t as $imp<$u>>::Output;

            #[inline]
            fn $method(self, other: &$u) -> <$t as $imp<$u>>::Output {
                $imp::$method(self, *other)
            }
        }

        impl $imp<&$u> for &$t {
            type Output = <$t as $imp<$u>>::Output;

            #[inline]
            fn $method(self, other: &$u) -> <$t as $imp<$u>>::Output {
                $imp::$method(*self, *other)
            }
        }
    };
}

macro_rules! forward_ref_op_assign {
    (impl $imp:ident, $method:ident for $t:ty, $u:ty) => {
        impl $imp<&$u> for $t {
            #[inline]
            fn $method(&mut self, other: &$u) {
                $imp::$method(self, *other);
            }
        }
    };
}

macro_rules! op_impl {
    ($($t:ty),*) => {
        paste! {
            $(
                impl Add for $t {
                    type Output = $t;

                    #[inline]
                    fn add(self, other: $t) -> Self {
                        Self(self.0.checked_add(other.0).expect("Overflow"))
                    }
                }
                forward_ref_binop! { impl Add, add for $t, $t }

                impl AddAssign for $t {
                    #[inline]
                    fn add_assign(&mut self, other: $t) {
                        self.0 = self.0.checked_add(other.0).expect("Overflow");
                    }
                }
                forward_ref_op_assign! { impl AddAssign, add_assign for $t, $t }

                impl Sub for $t {
                    type Output = $t;

                    #[inline]
                    fn sub(self, other: $t) -> Self {
                        Self(self.0.checked_sub(other.0).expect("Overflow"))
                    }
                }
                forward_ref_binop! { impl Sub, sub for $t, $t }

                impl SubAssign for $t {
                    #[inline]
                    fn sub_assign(&mut self, other: $t) {
                        self.0 = self.0.checked_sub(other.0).expect("Overflow");
                    }
                }
                forward_ref_op_assign! { impl SubAssign, sub_assign for $t, $t }

                impl Mul for $t {
                    type Output = $t;

                    #[inline]
                    fn mul(self, other: $t) -> Self {
                        Self(self.0.checked_mul(other.0).expect("Overflow"))
                    }
                }
                forward_ref_binop! { impl Mul, mul for $t, $t }

                impl MulAssign for $t {
                    #[inline]
                    fn mul_assign(&mut self, other: $t) {
                        self.0 = self.0.checked_mul(other.0).expect("Overflow");
                    }
                }
                forward_ref_op_assign! { impl MulAssign, mul_assign for $t, $t }

                impl Div for $t {
                    type Output = $t;

                    #[inline]
                    fn div(self, other: $t) -> Self {
                        Self(self.0.checked_div(other.0).expect("Overflow"))
                    }
                }
                forward_ref_binop! { impl Div, div for $t, $t }

                impl DivAssign for $t {
                    #[inline]
                    fn div_assign(&mut self, other: $t) {
                        self.0 = self.0.checked_div(other.0).expect("Overflow");
                    }
                }
                forward_ref_op_assign! { impl DivAssign, div_assign for $t, $t }

                impl Rem for $t {
                    type Output = $t;

                    #[inline]
                    fn rem(self, other: $t) -> Self {
                        Self(self.0 % other.0)
                    }
                }
                forward_ref_binop! { impl Rem, rem for $t, $t }

                impl RemAssign for $t {
                    #[inline]
                    fn rem_assign(&mut self, other: $t) {
                        self.0 = self.0 % other.0;
                    }
                }
                forward_ref_op_assign! { impl RemAssign, rem_assign for $t, $t }

                impl Not for $t {
                    type Output = $t;

                    #[inline]
                    fn not(self) -> Self {
                        Self(!self.0)
                    }
                }
                forward_ref_unop! { impl Not, not for $t }

                impl Pow<u32> for $t
                {
                    type Output = $t;

                    /// Raises self to the power of `exp`, using exponentiation by squaring.
                    ///
                    #[inline]
                    #[must_use = "this returns the result of the operation, \
                          without modifying the original"]
                    fn pow(self, exp: u32) -> Self {
                        Self(self.0.checked_pow(exp).expect("Overflow"))
                    }
                }

                impl Sqrt for $t
                {
                    fn sqrt(self) -> Self {
                        Self(self.0.sqrt())
                    }
                }

                impl Cbrt for $t
                {
                    fn cbrt(self) -> Self {
                        Self(self.0.cbrt())
                    }
                }

                impl NthRoot for $t
                {
                    fn nth_root(self, n: u32) -> Self {
                        Self(self.0.nth_root(n))
                    }
                }

                impl CheckedSub for $t
                {
                    fn checked_sub(self, other: Self) -> Option<Self> {
                        let opt = self.0.checked_sub(other.0);
                        opt.map(|v| Self(v))
                    }
                }

                impl CheckedMul for $t
                {
                    fn checked_mul(self, other: Self) -> Option<Self> {
                        let opt = self.0.checked_mul(other.0);
                        opt.map(|v| Self(v))
                    }
                }
            )*
        }
    };
}
op_impl! { BnumI256 }
op_impl! { BnumI384 }
op_impl! { BnumI512 }
op_impl! { BnumI768 }
op_impl! { BnumU256 }
op_impl! { BnumU384 }
op_impl! { BnumU512 }
op_impl! { BnumU768 }

macro_rules! op_impl_unsigned {
    ($($t:ty),*) => {
        paste! {
            $(
                impl $t {
                    pub fn is_power_of_two(self) -> bool {
                        self.0.is_power_of_two()
                    }

                    pub fn next_power_of_two(self) -> Self {
                        Self(self.0.next_power_of_two())
                    }
                }
            )*
        }
    };
}
op_impl_unsigned! { BnumU256 }
op_impl_unsigned! { BnumU384 }
op_impl_unsigned! { BnumU512 }
op_impl_unsigned! { BnumU768 }

macro_rules! op_impl_signed {
    ($($t:ty),*) => {
        paste! {
            $(
                impl Neg for $t {
                    type Output = Self;
                    #[inline]
                    fn neg(self) -> Self {
                        Self(self.0.neg())
                    }
                }
                forward_ref_unop! { impl Neg, neg for $t }


                impl $t {

                    /// Computes the absolute value of `self`, with overflow causing panic.
                    ///
                    /// The only case where such overflow can occur is when one takes the absolute value of the negative
                    /// minimal value for the type this is a positive value that is too large to represent in the type. In
                    /// such a case, this function panics.
                    ///
                    #[inline]
                    #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
                    pub fn abs(self) -> Self {
                        Self(self.0.abs())
                    }

                    /// Returns a number representing sign of `self`.
                    ///
                    ///  - `0` if the number is zero
                    ///  - `1` if the number is positive
                    ///  - `-1` if the number is negative
                    ///
                    #[inline]
                    #[must_use = "this returns the result of the operation, \
                          without modifying the original"]
                    pub fn signum(self) -> Self {
                        Self(self.0.signum())
                    }

                    /// Returns `true` if `self` is positive and `false` if the number is zero or
                    /// negative.
                    ///
                    #[must_use]
                    #[inline]
                    pub fn is_positive(self) -> bool {
                        self.0.is_positive()
                    }

                    /// Returns `true` if `self` is negative and `false` if the number is zero or
                    /// positive.
                    ///
                    #[must_use]
                    #[inline]
                    pub fn is_negative(self) -> bool {
                        self.0.is_negative()
                    }
                }
            )*
        }
    }
}

op_impl_signed! { BnumI256 }
op_impl_signed! { BnumI384 }
op_impl_signed! { BnumI512 }
op_impl_signed! { BnumI768 }

macro_rules! error {
    ($($t:ident),*) => {
        paste! {
            $(
                #[derive(Debug, Clone, PartialEq, Eq)]
                pub enum [<Parse $t Error>] {
                    NegativeToUnsigned,
                    Overflow,
                    InvalidLength,
                    InvalidDigit,
                }

                #[cfg(not(feature = "alloc"))]
                impl std::error::Error for [<Parse $t Error>] {}

                #[cfg(not(feature = "alloc"))]
                impl fmt::Display for [<Parse $t Error>] {
                    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                        write!(f, "{:?}", self)
                    }
                }
            )*
        }
    };
}
error! {
    BnumI256,
    BnumI384,
    BnumI512,
    BnumI768,
    BnumU256,
    BnumU384,
    BnumU512,
    BnumU768
}
