//! Definitions of safe integers and uints.

use crate::abi::*;
use core::ops::{Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign};
use core::ops::{BitXor, BitXorAssign, Div, DivAssign};
use core::ops::{Mul, MulAssign, Neg, Not, Rem, RemAssign};
use core::ops::{Shl, ShlAssign, Shr, ShrAssign, Sub, SubAssign};
use num_bigint::{BigInt, BigUint, Sign};
use num_traits::{One, Pow, Signed, ToPrimitive, Zero};
use paste::paste;
use sbor::rust::convert::{From, TryFrom};
use sbor::rust::fmt;
use sbor::rust::vec::Vec;
use sbor::*;
use std::cmp::{Ord, Ordering, PartialEq, PartialOrd};

pub mod basic;
pub mod bits;
pub mod convert;
#[cfg(test)]
mod test;
pub use convert::*;

macro_rules! types {

    (self: $self:ident,
     $(
         {
             type: $t:ident,
             self.0: $wrap:ty,
             self.zero(): $tt:ident($zero:expr),
             $ttt:ident::default(): $default:expr,
         }
     ),*) => {
        paste!{
            $(
                /// Provides safe integer arithmetic.
                ///
                /// Operations like `+`, '-', '*', or '/' sometimes produce overflow
                /// which is detected and results in a panic, instead of silently
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

            impl Default for $t {
                fn default() -> Self {
                    $default
                }
            }

            impl fmt::Debug for $t {
                fn fmt(&$self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    BigInt::from(*$self).fmt(f)
                }
            }

            impl fmt::Display for $t {
                fn fmt(&$self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    BigInt::from(*$self).fmt(f)
                }
            }

            impl fmt::Binary for $t {
                fn fmt(&$self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    BigInt::from(*$self).fmt(f)
                }
            }

            impl fmt::Octal for $t {
                fn fmt(&$self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    BigInt::from(*$self).fmt(f)
                }
            }

            impl fmt::LowerHex for $t {
                fn fmt(&$self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    BigInt::from(*$self).fmt(f)
                }
            }

            impl fmt::UpperHex for $t {
                fn fmt(&$self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    BigInt::from(*$self).fmt(f)
                }
            }

            impl Zero for $t {
                fn zero() -> Self {
                    Self($zero)
                }

                fn is_zero(&self) -> bool {
                    $zero == self.0
                }

                fn set_zero(&mut self) {
                    self.0 = $zero;
                }
            }

            impl One for $t {
                fn one() -> Self {
                    Self::try_from(1u8).unwrap()
                }
            }

            impl Ord for $t {
                fn cmp(&self, other: &Self) -> Ordering {
                   let mut a: Vec<u8> = self.to_le_bytes().into();
                   let mut b: Vec<u8> = other.to_le_bytes().into();
                   a.reverse();
                   b.reverse();
                   a[0] ^= 0x80;
                   b[0] ^= 0x80;
                   a.cmp(&b)
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

            #[cfg(test)]
            impl $t {
                pub fn type_name(self) -> &'static str {
                    stringify!($t)
                }
            }

            scrypto_type!($t, ScryptoType::$t, Vec::new());

            )*
        }
    }
}

types! {
    self: self,
    {
        type: I8,
        self.0: i8,
        self.zero(): I8(0),
        I8::default(): I8(0),
    },
    {
        type: I16,
        self.0: i16,
        self.zero(): I16(0),
        I16::default(): I16(0),
    },
    {
        type: I32,
        self.0: i32,
        self.zero(): I32(0),
        I32::default(): I32(0),
    },
    {
        type: I64,
        self.0: i64,
        self.zero(): I64(0),
        I64::default(): I64(0),
    },
    {
        type: I128,
        self.0: i128,
        self.zero(): I128(0),
        I128::default(): I128(0),
    },
    {
        type: I256,
        self.0: [u8; 32],
        self.zero(): I256([0u8; 32]),
        I256::default(): I256([0u8; 32]),
    },
    {
        type: I384,
        self.0: [u8; 48],
        self.zero(): I384([0u8; 48]),
        I384::default(): I384([0u8; 48]),
    },
    {
        type: I512,
        self.0: [u8; 64],
        self.zero(): I512([0u8; 64]),
        I512::default(): I512([0u8; 64]),
    },
    {
        type: U8,
        self.0: u8,
        self.zero(): U8(0),
        U8::default(): U8(0),
    },
    {
        type: U16,
        self.0: u16,
        self.zero(): U16(0),
        U16::default(): U16(0),
    },
    {
        type: U32,
        self.0: u32,
        self.zero(): U32(0),
        U32::default(): U32(0),
    },
    {
        type: U64,
        self.0: u64,
        self.zero(): U64(0),
        U64::default(): U64(0),
    },
    {
        type: U128,
        self.0: u128,
        self.zero(): U128(0),
        U128::default(): U128(0),
    },
    {
        type: U256,
        self.0: [u8; 32],
        self.zero(): U256([0u8; 32]),
        U256::default(): U256([0u8; 32]),
    },
    {
        type: U384,
        self.0: [u8; 48],
        self.zero(): U384([0u8; 48]),
        U384::default(): U384([0u8; 48]),
    },
    {
        type: U512,
        self.0: [u8; 64],
        self.zero(): U512([0u8; 64]),
        U512::default(): U512([0u8; 64]),
    }
}

#[macro_export]
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

#[macro_export]
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

#[macro_export]
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

macro_rules! checked_impl {
        ($(($t:ty, $o:ty, $out:ty)),*) => {
            paste! {
                $(
                    impl Add<$o> for $t {
                        type Output = $out;

                        #[inline]
                        fn add(self, other: $o) -> $out {
                            BigInt::from(self).add(&BigInt::from(other)).try_into().unwrap()
                        }
                    }
                    forward_ref_binop! { impl Add, add for $t, $o }

                    impl AddAssign<$o> for $t {
                        #[inline]
                        fn add_assign(&mut self, other: $o) {
                            *self = (*self + other).try_into().unwrap();
                        }
                    }
                    forward_ref_op_assign! { impl AddAssign, add_assign for $t, $o }

                    impl Sub<$o> for $t {
                        type Output = $out;

                        #[inline]
                        fn sub(self, other: $o) -> $out {
                            BigInt::from(self).sub(&BigInt::from(other)).try_into().unwrap()
                        }
                    }
                    forward_ref_binop! { impl Sub, sub for $t, $o }

                    impl SubAssign<$o> for $t {
                        #[inline]
                        fn sub_assign(&mut self, other: $o) {
                            *self = (*self - other).try_into().unwrap();
                        }
                    }
                    forward_ref_op_assign! { impl SubAssign, sub_assign for $t, $o }

                    impl Mul<$o> for $t {
                        type Output = $out;

                        #[inline]
                        fn mul(self, other: $o) -> $out {
                            BigInt::from(self).mul(&BigInt::from(other)).try_into().unwrap()
                        }
                    }
                    forward_ref_binop! { impl Mul, mul for $t, $o }

                    impl MulAssign<$o> for $t {
                        #[inline]
                        fn mul_assign(&mut self, other: $o) {
                            *self = (*self * other).try_into().unwrap();
                        }
                    }
                    forward_ref_op_assign! { impl MulAssign, mul_assign for $t, $o }

                    impl Div<$o> for $t {
                        type Output = $out;

                        #[inline]
                        fn div(self, other: $o) -> $out {
                            BigInt::from(self).div(&BigInt::from(other)).try_into().unwrap()
                        }
                    }
                    forward_ref_binop! { impl Div, div for $t, $o }

                    impl DivAssign<$o> for $t {
                        #[inline]
                        fn div_assign(&mut self, other: $o) {
                            *self = (*self / other).try_into().unwrap();
                        }
                    }
                    forward_ref_op_assign! { impl DivAssign, div_assign for $t, $o }

                    impl Rem<$o> for $t {
                        type Output = $out;

                        #[inline]
                        fn rem(self, other: $o) -> $out {
                            BigInt::from(self).rem(&BigInt::from(other)).try_into().unwrap()
                        }
                    }
                    forward_ref_binop! { impl Rem, rem for $t, $o }

                    impl RemAssign<$o> for $t {
                        #[inline]
                        fn rem_assign(&mut self, other: $o) {
                            *self = (*self % other).try_into().unwrap();
                        }
                    }
                    forward_ref_op_assign! { impl RemAssign, rem_assign for $t, $o }
                    impl Pow<$o> for $t {
                        type Output = $t;

                        /// Raises self to the power of `exp`, using exponentiation by squaring.
                        ///
                        #[inline]
                        #[must_use = "this returns the result of the operation, \
                              without modifying the original"]
                        fn pow(self, other: $o) -> $t {
                            BigInt::from(self).pow(BigUint::try_from(BigInt::from(other)).unwrap()).try_into().unwrap()
                        }
                    }

                    forward_ref_binop! { impl Pow, pow for $t, $o }

                    )*
            }
        };
    }
checked_impl! {
//(self, other, output)
(u8, U8, U8), (u8, U16, U16), (u8, U32, U32), (u8, U64, U64), (u8, U128, U128),
(u8, U256, U256), (u8, U384, U384), (u8, U512, U512),
(u8, I8, I8), (u8, I16, I16), (u8, I32, I32), (u8, I64, I64), (u8, I128, I128),
(u8, I256, I256), (u8, I384, I384), (u8, I512, I512),

(u16, U8, U16), (u16, U16, U16), (u16, U32, U32), (u16, U64, U64), (u16, U128, U128),
(u16, U256, U256), (u16, U384, U384), (u16, U512, U512), (u16, I8, I16), (u16, I16, I16),
(u16, I32, I32), (u16, I64, I64), (u16, I128, I128), (u16, I256, I256), (u16, I384, I384),
(u16, I512, I512),

(u32, U8, U32), (u32, U16, U32), (u32, U32, U32), (u32, U64, U64), (u32, U128, U128),
(u32, U256, U256), (u32, U384, U384), (u32, U512, U512), (u32, I8, I32), (u32, I16, I32),
(u32, I32, I32), (u32, I64, I64), (u32, I128, I128), (u32, I256, I256), (u32, I384, I384),
(u32, I512, I512),

(u64, U8, U64), (u64, U16, U64), (u64, U32, U64), (u64, U64, U64), (u64, U128, U128),
(u64, U256, U256), (u64, U384, U384), (u64, U512, U512), (u64, I8, I64), (u64, I16, I64),
(u64, I32, I64), (u64, I64, I64), (u64, I128, I128), (u64, I256, I256), (u64, I384, I384),
(u64, I512, I512),

(u128, U8, U128), (u128, U16, U128), (u128, U32, U128), (u128, U64, U128),
(u128, U128, U128), (u128, U256, U256), (u128, U384, U384), (u128, U512, U512),
(u128, I8, I128), (u128, I16, I128), (u128, I32, I128), (u128, I64, I128),
(u128, I128, I128), (u128, I256, I256), (u128, I384, I384), (u128, I512, I512),

(i8, U8, I8), (i8, U16, I16), (i8, U32, I32), (i8, U64, I64), (i8, U128, I128),
(i8, U256, I256), (i8, U384, I384), (i8, U512, I512), (i8, I8, I8), (i8, I16, I16),
(i8, I32, I32), (i8, I64, I64), (i8, I128, I128), (i8, I256, I256), (i8, I384, I384),
(i8, I512, I512),

(i16, U8, I16), (i16, U16, I16), (i16, U32, I32), (i16, U64, I64), (i16, U128, I128),
(i16, U256, I256), (i16, U384, I384), (i16, U512, I512), (i16, I8, I16), (i16, I16, I16),
(i16, I32, I32), (i16, I64, I64), (i16, I128, I128), (i16, I256, I256), (i16, I384, I384),
(i16, I512, I512),

(i32, U8, I32), (i32, U16, I32), (i32, U32, I32), (i32, U64, I64), (i32, U128, I128),
(i32, U256, I256), (i32, U384, I384), (i32, U512, I512), (i32, I8, I32), (i32, I16, I32),
(i32, I32, I32), (i32, I64, I64), (i32, I128, I128), (i32, I256, I256), (i32, I384, I384),
(i32, I512, I512),

(i64, U8, I64), (i64, U16, I64), (i64, U32, I64), (i64, U64, I64), (i64, U128, I128),
(i64, U256, I256), (i64, U384, I384), (i64, U512, I512), (i64, I8, I64), (i64, I16, I64),
(i64, I32, I64), (i64, I64, I64), (i64, I128, I128), (i64, I256, I256), (i64, I384, I384),
(i64, I512, I512),

(i128, U8, I128), (i128, U16, I128), (i128, U32, I128), (i128, U64, I128),
(i128, U128, I128), (i128, U256, I256), (i128, U384, I384), (i128, U512, I512),
(i128, I8, I128), (i128, I16, I128), (i128, I32, I128), (i128, I64, I128),
(i128, I128, I128), (i128, I256, I256), (i128, I384, I384), (i128, I512, I512),

(I8, u8, I8), (I8, u16, I16), (I8, u32, I32), (I8, u64, I64), (I8, u128, I128),
(I8, i8, I8), (I8, i16, I16), (I8, i32, I32), (I8, i64, I64), (I8, i128, I128),
(I8, U8, I8), (I8, U16, I16), (I8, U32, I32), (I8, U64, I64), (I8, U128, I128),
(I8, U256, I256), (I8, U384, I384), (I8, U512, I512), (I8, I8, I8), (I8, I16, I16),
(I8, I32, I32), (I8, I64, I64), (I8, I128, I128), (I8, I256, I256), (I8, I384, I384),
(I8, I512, I512),

(I16, u8, I16), (I16, u16, I16), (I16, u32, I32), (I16, u64, I64), (I16, u128, I128),
(I16, i8, I16), (I16, i16, I16), (I16, i32, I32), (I16, i64, I64), (I16, i128, I128),
(I16, U8, I16), (I16, U16, I16), (I16, U32, I32), (I16, U64, I64), (I16, U128, I128),
(I16, U256, I256), (I16, U384, I384), (I16, U512, I512), (I16, I8, I16), (I16, I16, I16),
(I16, I32, I32), (I16, I64, I64), (I16, I128, I128), (I16, I256, I256), (I16, I384, I384),
(I16, I512, I512),

(I32, u8, I32), (I32, u16, I32), (I32, u32, I32), (I32, u64, I64), (I32, u128, I128),
(I32, i8, I32), (I32, i16, I32), (I32, i32, I32), (I32, i64, I64), (I32, i128, I128),
(I32, U8, I32), (I32, U16, I32), (I32, U32, I32), (I32, U64, I64), (I32, U128, I128),
(I32, U256, I256), (I32, U384, I384), (I32, U512, I512), (I32, I8, I32), (I32, I16, I32),
(I32, I32, I32), (I32, I64, I64), (I32, I128, I128), (I32, I256, I256), (I32, I384, I384),
(I32, I512, I512),

(I64, u8, I64), (I64, u16, I64), (I64, u32, I64), (I64, u64, I64), (I64, u128, I128),
(I64, i8, I64), (I64, i16, I64), (I64, i32, I64), (I64, i64, I64), (I64, i128, I128),
(I64, U8, I64), (I64, U16, I64), (I64, U32, I64), (I64, U64, I64), (I64, U128, I128),
(I64, U256, I256), (I64, U384, I384), (I64, U512, I512), (I64, I8, I64), (I64, I16, I64),
(I64, I32, I64), (I64, I64, I64), (I64, I128, I128), (I64, I256, I256), (I64, I384, I384),
(I64, I512, I512),

(I128, u8, I128), (I128, u16, I128), (I128, u32, I128), (I128, u64, I128),
(I128, u128, I128), (I128, i8, I128), (I128, i16, I128), (I128, i32, I128),
(I128, i64, I128), (I128, i128, I128),
(I128, U8, I128), (I128, U16, I128), (I128, U32, I128), (I128, U64, I128),
(I128, U128, I128), (I128, U256, I256), (I128, U384, I384), (I128, U512, I512),
(I128, I8, I128), (I128, I16, I128), (I128, I32, I128), (I128, I64, I128),
(I128, I128, I128), (I128, I256, I256), (I128, I384, I384), (I128, I512, I512),

(I256, u8, I256), (I256, u16, I256), (I256, u32, I256), (I256, u64, I256),
(I256, u128, I256), (I256, i8, I256), (I256, i16, I256), (I256, i32, I256),
(I256, i64, I256), (I256, i128, I256),
(I256, U8, I256), (I256, U16, I256), (I256, U32, I256), (I256, U64, I256),
(I256, U128, I256), (I256, U256, I256), (I256, U384, I384), (I256, U512, I512),
(I256, I8, I256), (I256, I16, I256), (I256, I32, I256), (I256, I64, I256),
(I256, I128, I256), (I256, I256, I256), (I256, I384, I384), (I256, I512, I512),

(I384, u8, I384), (I384, u16, I384), (I384, u32, I384), (I384, u64, I384),
(I384, u128, I384), (I384, i8, I384), (I384, i16, I384), (I384, i32, I384),
(I384, i64, I384), (I384, i128, I384),
(I384, U8, I384), (I384, U16, I384), (I384, U32, I384), (I384, U64, I384),
(I384, U128, I384), (I384, U256, I384), (I384, U384, I384), (I384, U512, I512),
(I384, I8, I384), (I384, I16, I384), (I384, I32, I384), (I384, I64, I384),
(I384, I128, I384), (I384, I256, I384), (I384, I384, I384), (I384, I512, I512),

(I512, u8, I512), (I512, u16, I512), (I512, u32, I512), (I512, u64, I512),
(I512, u128, I512), (I512, i8, I512), (I512, i16, I512), (I512, i32, I512),
(I512, i64, I512), (I512, i128, I512),
(I512, U8, I512), (I512, U16, I512), (I512, U32, I512), (I512, U64, I512),
(I512, U128, I512), (I512, U256, I512), (I512, U384, I512), (I512, U512, I512),
(I512, I8, I512), (I512, I16, I512), (I512, I32, I512), (I512, I64, I512),
(I512, I128, I512), (I512, I256, I512), (I512, I384, I512), (I512, I512, I512),

(U8, u8, U8), (U8, u16, U16), (U8, u32, U32), (U8, u64, U64), (U8, u128, U128),
(U8, i8, I8), (U8, i16, I16), (U8, i32, I32), (U8, i64, I64), (U8, i128, I128),
(U8, U8, U8), (U8, U16, U16), (U8, U32, U32), (U8, U64, U64), (U8, U128, U128),
(U8, U256, U256), (U8, U384, U384), (U8, U512, U512),
(U8, I8, I8), (U8, I16, I16), (U8, I32, I32), (U8, I64, I64), (U8, I128, I128),
(U8, I256, I256), (U8, I384, I384), (U8, I512, I512),

(U16, u8, U16), (U16, u16, U16), (U16, u32, U32), (U16, u64, U64), (U16, u128, U128),
(U16, i8, I16), (U16, i16, I16), (U16, i32, I32), (U16, i64, I64), (U16, i128, I128),
(U16, U8, U16), (U16, U16, U16), (U16, U32, U32), (U16, U64, U64), (U16, U128, U128),
(U16, U256, U256), (U16, U384, U384), (U16, U512, U512), (U16, I8, I16), (U16, I16, I16),
(U16, I32, I32), (U16, I64, I64), (U16, I128, I128), (U16, I256, I256), (U16, I384, I384),
(U16, I512, I512),

(U32, u8, U32), (U32, u16, U32), (U32, u32, U32), (U32, u64, U64), (U32, u128, U128),
(U32, i8, I32), (U32, i16, I32), (U32, i32, I32), (U32, i64, I64), (U32, i128, I128),
(U32, U8, U32), (U32, U16, U32), (U32, U32, U32), (U32, U64, U64), (U32, U128, U128),
(U32, U256, U256), (U32, U384, U384), (U32, U512, U512), (U32, I8, I32), (U32, I16, I32),
(U32, I32, I32), (U32, I64, I64), (U32, I128, I128), (U32, I256, I256), (U32, I384, I384),
(U32, I512, I512),

(U64, u8, U64), (U64, u16, U64), (U64, u32, U64), (U64, u64, U64), (U64, u128, U128),
(U64, i8, I64), (U64, i16, I64), (U64, i32, I64), (U64, i64, I64), (U64, i128, I128),
(U64, U8, U64), (U64, U16, U64), (U64, U32, U64), (U64, U64, U64), (U64, U128, U128),
(U64, U256, U256), (U64, U384, U384), (U64, U512, U512), (U64, I8, I64), (U64, I16, I64),
(U64, I32, I64), (U64, I64, I64), (U64, I128, I128), (U64, I256, I256), (U64, I384, I384),
(U64, I512, I512),

(U128, u8, U128), (U128, u16, U128), (U128, u32, U128), (U128, u64, U128),
(U128, u128, U128), (U128, i8, I128), (U128, i16, I128), (U128, i32, I128),
(U128, i64, I128), (U128, i128, I128),
(U128, U8, U128), (U128, U16, U128), (U128, U32, U128), (U128, U64, U128),
(U128, U128, U128), (U128, U256, U256), (U128, U384, U384), (U128, U512, U512),
(U128, I8, I128), (U128, I16, I128), (U128, I32, I128), (U128, I64, I128),
(U128, I128, I128), (U128, I256, I256), (U128, I384, I384), (U128, I512, I512),

(U256, u8, U256), (U256, u16, U256), (U256, u32, U256), (U256, u64, U256),
(U256, u128, U256), (U256, i8, I256), (U256, i16, I256), (U256, i32, I256),
(U256, i64, I256), (U256, i128, I256),
(U256, U8, U256),  (U256, U16, U256), (U256, U32, U256), (U256, U64, U256),
(U256, U128, U256), (U256, U256, U256), (U256, U384, U384), (U256, U512, U512),
(U256, I8, I256), (U256, I16, I256), (U256, I32, I256), (U256, I64, I256),
(U256, I128, I256), (U256, I256, I256), (U256, I384, I384), (U256, I512, I512),

(U384, u8, U384), (U384, u16, U384), (U384, u32, U384), (U384, u64, U384),
(U384, u128, U384), (U384, i8, I384), (U384, i16, I384), (U384, i32, I384),
(U384, i64, I384), (U384, i128, I384),
(U384, U8, U384), (U384, U16, U384), (U384, U32, U384), (U384, U64, U384),
(U384, U128, U384), (U384, U256, U384), (U384, U384, U384), (U384, U512, U512),
(U384, I8, I384), (U384, I16, I384), (U384, I32, I384), (U384, I64, I384),
(U384, I128, I384), (U384, I256, I384), (U384, I384, I384), (U384, I512, I512),

(U512, u8, U512), (U512, u16, U512), (U512, u32, U512), (U512, u64, U512),
(U512, u128, U512), (U512, i8, I512), (U512, i16, I512), (U512, i32, I512),
(U512, i64, I512), (U512, i128, I512),
(U512, U8, U512), (U512, U16, U512), (U512, U32, U512), (U512, U64, U512),
(U512, U128, U512), (U512, U256, U512), (U512, U384, U512), (U512, U512, U512),
(U512, I8, I512), (U512, I16, I512), (U512, I32, I512), (U512, I64, I512),
(U512, I128, I512), (U512, I256, I512), (U512, I384, I512), (U512, I512, I512)
 }

macro_rules! checked_impl_not_large {
    ($($t:ident),*) => {
        $(
            impl Not for $t {
                type Output = $t;

                #[inline]
                fn not(self) -> $t {
                    self.0.iter().map(|x| x.not()).collect::<Vec<u8>>().try_into().unwrap()
                }
            }
            forward_ref_unop! { impl Not, not for $t }
        )*
    }
}

macro_rules! checked_impl_not_small {
    ($($t:ident),*) => {
        $(
            impl Not for $t {
                type Output = $t;

                #[inline]
                fn not(self) -> $t {
                    $t(!self.0)
                }
            }
            forward_ref_unop! { impl Not, not for $t }
        )*
    }
}

checked_impl_not_large! {I256, I384, I512, U256, U384, U512}
checked_impl_not_small! {I8, I16, I32, I64, I128, U8, U16, U32, U64, U128}

macro_rules! checked_int_impl_signed {
    ($($t:ident, $self:ident, $base:expr),*) => ($(
            paste! {

                impl Neg for $t {
                    type Output = Self;
                    #[inline]
                    fn neg(self) -> Self {
                        Self::zero() - self
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
                    pub fn abs($self) -> $t {
                        $base.abs().try_into().unwrap()
                    }

                    /// Returns a number representing sign of `self`.
                    ///
                    ///  - `0` if the number is zero
                    ///  - `1` if the number is positive
                    ///  - `-1` if the number is negative
                    ///
                    /// # Examples
                    ///
                    /// Basic usage:
                    ///
                    /// ```
                    #[doc = "use scrypto::prelude::*;"]
                    ///
                    #[doc = "assert_eq!(" $t "::tfrom(10i8).signum(), " $t "::tfrom(1i8));"]
                    #[doc = "assert_eq!(" $t "::tfrom(0i8).signum(), " $t "::tfrom(0i8));"]
                    #[doc = "assert_eq!(" $t "::tfrom(-10i8).signum(), " $t "::tfrom(-1i8));"]
                    /// ```
                    #[inline]
                    #[must_use = "this returns the result of the operation, \
                          without modifying the original"]
                    pub fn signum($self) -> $t {
                        $base.signum().try_into().unwrap()
                    }

                    /// Returns `true` if `self` is positive and `false` if the number is zero or
                    /// negative.
                    ///
                    /// # Examples
                    ///
                    /// Basic usage:
                    ///
                    /// ```
                    #[doc = "use scrypto::prelude::*;"]
                    ///
                    #[doc = "assert!(" $t "::tfrom(10i8).is_positive());"]
                    #[doc = "assert!(!" $t "::tfrom(-10i8).is_positive());"]
                    /// ```
                    #[must_use]
                    #[inline]
                    pub fn is_positive($self) -> bool {
                        $base.is_positive().try_into().unwrap()
                            // large: self.0.to_vec().into_iter().nth(self.0.len() - 1).unwrap() & 0x80 == 0
                    }

                    /// Returns `true` if `self` is negative and `false` if the number is zero or
                    /// positive.
                    ///
                    /// # Examples
                    ///
                    /// Basic usage:
                    ///
                    /// ```
                    #[doc = "use scrypto::prelude::*;"]
                    ///
                    #[doc = "assert!(" $t "::tfrom(-10i8).is_negative());"]
                    #[doc = "assert!(!" $t "::tfrom(10i8).is_negative());"]
                    /// ```
                    #[must_use]
                    #[inline]
                    pub fn is_negative($self) -> bool {
                        $base.is_negative().try_into().unwrap()
                            // large: self.0.to_vec().into_iter().nth(self.0.len() - 1).unwrap() & 0x80 > 0
                    }
                }
            }
    )*)
}

macro_rules! checked_int_impl_signed_all_large {
    ($($t:ident),*) => {$(
        checked_int_impl_signed! {
            $t,
            self,
            BigInt::from(self)
        }
    )*
    }
}

macro_rules! checked_int_impl_signed_all_small {
    ($($t:ident),*) => {$(
        checked_int_impl_signed! {
            $t,
            self,
            self.0
        }
    )*}
}

checked_int_impl_signed_all_large! { I256, I384, I512 }
checked_int_impl_signed_all_small! { I8, I16, I32, I64, I128 }

macro_rules! checked_int_impl_unsigned_large {
    ($($t:ty),*) => ($(
            impl $t {

                /// Returns `true` if and only if `self == 2^k` for some `k`.
                ///
                #[must_use]
                #[inline]
                pub fn is_power_of_two(self) -> bool {
                    if self.0.iter().map(|x| x.count_ones()).sum::<u32>() == 1 {
                        true
                    } else {
                        false
                    }
                }

                /// Returns the smallest power of two greater than or equal to `self`.
                ///
                /// When return value overflows (i.e., `self > (1 << (N-1))` for type
                /// `uN`), it panics. It uses the checked unsigned integer arithmetics.
                ///
                #[inline]
                #[must_use = "this returns the result of the operation, \
                          without modifying the original"]
                pub fn next_power_of_two(self) -> Self {
                    let lz = self.leading_zeros();
                    let co = self.count_ones();
                    if lz == 0 && co > 1 {
                        panic!("overflow");
                    } else {
                        if co == 1 {
                            self
                        } else {
                            Self::from(1u8) << (Self::BITS - lz)
                        }
                    }
                }
            }
    )*)
}

macro_rules! checked_int_impl_unsigned_small {
    ($($t:ty),*) => ($(
            impl $t {

                /// Returns `true` if and only if `self == 2^k` for some `k`.
                ///
                #[must_use]
                #[inline]
                pub fn is_power_of_two(self) -> bool {
                    self.0.is_power_of_two()
                }

                /// Returns the smallest power of two greater than or equal to `self`.
                ///
                /// When return value overflows (i.e., `self > (1 << (N-1))` for type
                /// `uN`), overflows to `2^N = 0`.
                ///
                #[inline]
                #[must_use = "this returns the result of the operation, \
                          without modifying the original"]
                pub fn next_power_of_two(self) -> Self {
                    Self(self.0.checked_next_power_of_two().unwrap())
                }
            }
    )*)
}

checked_int_impl_unsigned_large! { U256, U384, U512 }
checked_int_impl_unsigned_small! { U8, U16, U32, U64, U128 }
