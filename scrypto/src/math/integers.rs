//! Definitions of safe integers and uints.

use core::ops::{Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign};
use core::ops::{BitXor, BitXorAssign, Div, DivAssign};
use core::ops::{Mul, MulAssign, Neg, Not, Rem, RemAssign};
use core::ops::{Shl, ShlAssign, Shr, ShrAssign, Sub, SubAssign};
use forward_ref::*;
use num_bigint::{BigInt, BigUint, ParseBigIntError, Sign};
use num_traits::{FromPrimitive, Pow, Signed, ToPrimitive, Zero};
use paste::paste;
use sbor::rust::convert::{From, TryFrom};
use sbor::rust::fmt;
use sbor::rust::str::FromStr;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;

macro_rules! types {

    (self: $self:ident,
     $(
         {
             type: $t:ident,
             self.0: $wrap:ty,
             self.zero(): $tt:ident($zero:expr),
             Default::default(): $default:expr,
             self_expr: $self_expr:expr
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
                /// Integer arithmetic can be achieved either through methods like
                #[doc = "/// `checked_add`, or through the " $t "type, which ensures all" ]
                /// standard arithmetic operations on the underlying value to have
                /// checked semantics.
                ///
                /// The underlying value can be retrieved through the `.0` index of the
                #[doc = "/// `" $t "` tuple."]
                ///
                /// # Layout
                ///
                #[doc = "/// `" $t "` will have the same methods and traits as"]
                /// the built-in counterpart.
                #[derive(Clone , Copy , Eq , Hash , Ord , PartialEq , PartialOrd)]
                #[repr(transparent)]
                pub struct $t(pub $wrap);

            impl Default for $t {
                fn default() -> Self {
                    $default
                }
            }

            impl fmt::Debug for $t {
                fn fmt(&$self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    $self_expr.fmt(f)
                }
            }

            impl fmt::Display for $t {
                fn fmt(&$self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    $self_expr.fmt(f)
                }
            }

            impl fmt::Binary for $t {
                fn fmt(&$self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    $self_expr.fmt(f)
                }
            }

            impl fmt::Octal for $t {
                fn fmt(&$self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    $self_expr.fmt(f)
                }
            }

            impl fmt::LowerHex for $t {
                fn fmt(&$self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    $self_expr.fmt(f)
                }
            }

            impl fmt::UpperHex for $t {
                fn fmt(&$self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    $self_expr.fmt(f)
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

            impl FromPrimitive for $t {
                fn from_i64(n: i64) -> Option<Self> {
                    [<bigint_to_$t:lower>] (BigInt::from(n)).ok()
                }
                fn from_i128(n: i128) -> Option<Self> {
                    [<bigint_to_$t:lower>] (BigInt::from(n)).ok()
                }
                fn from_u64(n: u64) -> Option<Self> {
                    [<bigint_to_$t:lower>] (BigInt::from(n)).ok()
                }
                fn from_u128(n: u128) -> Option<Self> {
                    [<bigint_to_$t:lower>] (BigInt::from(n)).ok()
                }
            }

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
        Default::default(): I8(0),
        self_expr: self.0
    },
    {
        type: I16,
        self.0: i16,
        self.zero(): I16(0),
        Default::default(): I16(0),
        self_expr: self.0
    },
    {
        type: I32,
        self.0: i32,
        self.zero(): I32(0),
        Default::default(): I32(0),
        self_expr: self.0
    },
    {
        type: I64,
        self.0: i64,
        self.zero(): I64(0),
        Default::default(): I64(0),
        self_expr: self.0
    },
    {
        type: I128,
        self.0: i128,
        self.zero(): I128(0),
        Default::default(): I128(0),
        self_expr: self.0
    },
    {
        type: I256,
        self.0: [u8; 32],
        self.zero(): I256([0u8; 32]),
        Default::default(): I256([0u8; 32]),
        self_expr: BigInt::from(*self)
    },
    {
        type: I384,
        self.0: [u8; 48],
        self.zero(): I384([0u8; 48]),
        Default::default(): I384([0u8; 48]),
        self_expr: BigInt::from(*self)
    },
    {
        type: I512,
        self.0: [u8; 64],
        self.zero(): I512([0u8; 64]),
        Default::default(): I512([0u8; 64]),
        self_expr: BigInt::from(*self)
    },
    {
        type: U8,
        self.0: u8,
        self.zero(): U8(0),
        Default::default(): U8(0),
        self_expr: self.0
    },
    {
        type: U16,
        self.0: u16,
        self.zero(): U16(0),
        Default::default(): U16(0),
        self_expr: self.0
    },
    {
        type: U32,
        self.0: u32,
        self.zero(): U32(0),
        Default::default(): U32(0),
        self_expr: self.0
    },
    {
        type: U64,
        self.0: u64,
        self.zero(): U64(0),
        Default::default(): U64(0),
        self_expr: self.0
    },
    {
        type: U128,
        self.0: u128,
        self.zero(): U128(0),
        Default::default(): U128(0),
        self_expr: self.0
    },
    {
        type: U256,
        self.0: [u8; 32],
        self.zero(): U256([0u8; 32]),
        Default::default(): U256([0u8; 32]),
        self_expr: BigInt::from(*self)
    },
    {
        type: U384,
        self.0: [u8; 48],
        self.zero(): U384([0u8; 48]),
        Default::default(): U384([0u8; 48]),
        self_expr: BigInt::from(*self)
    },
    {
        type: U512,
        self.0: [u8; 64],
        self.zero(): U512([0u8; 64]),
        Default::default(): U512([0u8; 64]),
        self_expr: BigInt::from(*self)
    }
}

#[derive(Debug)]
pub enum ParseIntError {
    NegativeToUnsigned,
    Overflow,
}

trait PrimIntExt<T> {
    type Output;
    fn rotate_left(self, other: T) -> Self;
    fn rotate_right(self, other: T) -> Self;
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


                    impl BitXor<$o> for $t {
                        type Output = $out;

                        #[inline]
                        fn bitxor(self, other: $o) -> $out {
                            BigInt::from(self).bitxor(&BigInt::from(other)).try_into().unwrap()
                        }
                    }
                    forward_ref_binop! { impl BitXor, bitxor for $t, $o }

                    impl BitXorAssign<$o> for $t {
                        #[inline]
                        fn bitxor_assign(&mut self, other: $o) {
                            *self = (*self ^ other).try_into().unwrap();
                        }
                    }
                    forward_ref_op_assign! { impl BitXorAssign, bitxor_assign for $t, $o }

                    impl BitOr<$o> for $t {
                        type Output = $out;

                        #[inline]
                        fn bitor(self, other: $o) -> $out {
                            BigInt::from(self).bitor(&BigInt::from(other)).try_into().unwrap()
                        }
                    }
                    forward_ref_binop! { impl BitOr, bitor for $t, $o }

                    impl BitOrAssign<$o> for $t {
                        #[inline]
                        fn bitor_assign(&mut self, other: $o) {
                            *self = (*self | other).try_into().unwrap();
                        }
                    }
                    forward_ref_op_assign! { impl BitOrAssign, bitor_assign for $t, $o }

                    impl BitAnd<$o> for $t {
                        type Output = $out;

                        #[inline]
                        fn bitand(self, other: $o) -> $out {
                            BigInt::from(self).bitand(&BigInt::from(other)).try_into().unwrap()
                        }
                    }
                    forward_ref_binop! { impl BitAnd, bitand for $t, $o }

                    impl BitAndAssign<$o> for $t {
                        #[inline]
                        fn bitand_assign(&mut self, other: $o) {
                            *self = (*self & other).try_into().unwrap();
                        }
                    }
                    forward_ref_op_assign! { impl BitAndAssign, bitand_assign for $t, $o }

                    impl Shl<$o> for $t {
                        type Output = $t;

                        #[inline]
                        fn shl(self, other: $o) -> $t {
                            if other > <$t>::BITS.try_into().unwrap() {
                                panic!("overflow");
                            }
                            let to_shift = BigInt::from(self);
                            if <$t>::MIN == <$t>::zero() {
                                let len: usize = to_shift
                                    .to_bytes_le().1
                                    .len()
                                    .min((<$t>::BITS / 8) as usize);
                                let shift = BigInt::from(other).to_i64().unwrap();
                                BigInt::from_bytes_le(Sign::Plus, to_shift.shl(shift).to_bytes_le().1[..len].into())
                                    .try_into()
                                    .unwrap()
                            } else {
                                let len: usize = to_shift
                                    .to_signed_bytes_le()
                                    .len()
                                    .min((<$t>::BITS / 8) as usize);
                                let shift = BigInt::from(other).to_i64().unwrap();
                                BigInt::from_signed_bytes_le(to_shift.shl(shift).to_bytes_le().1[..len].into())
                                    .try_into()
                                    .unwrap()
                            }
                        }
                    }

                    forward_ref_binop! { impl Shl, shl for $t, $o }

                    impl ShlAssign<$o> for $t {
                        #[inline]
                        fn shl_assign(&mut self, other: $o) {
                            *self = *self << other;
                        }
                    }
                    forward_ref_op_assign! { impl ShlAssign, shl_assign for $t, $o }

                    impl Shr<$o> for $t {
                        type Output = $t;

                        #[inline]
                        fn shr(self, other: $o) -> $t {
                            if other > <$t>::BITS.try_into().unwrap() {
                                panic!("overflow");
                            }
                            let to_shift = BigInt::from(self);
                            let shift = BigInt::from(other).to_i64().unwrap();
                            to_shift.shr(shift)
                                .try_into()
                                .unwrap()
                        }
                    }
                    forward_ref_binop! { impl Shr, shr for $t, $o }

                    impl ShrAssign<$o> for $t {
                        #[inline]
                        fn shr_assign(&mut self, other: $o) {
                            *self = *self >> other;
                        }
                    }
            forward_ref_op_assign! { impl ShrAssign, shr_assign for $t, $o }
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

                    impl PrimIntExt<$o> for $t {
                        type Output = $t;
                        /// Shifts the bits to the left by a specified amount, `n`,
                        /// wrapping the truncated bits to the end of the resulting
                        /// integer.
                        ///
                        /// Please note this isn't the same operation as the `<<` shifting
                        /// operator! This method can not overflow as opposed to '<<'.
                        ///
                        /// # Examples
                        ///
                        /// Basic usage:
                        ///
                        /// ```
                        #[doc = "use scrypto::math::" $t ";"]
                        ///
                        /// let n: $t = $t(0x0123456789ABCDEF);
                        /// let m: $t = $t(-0x76543210FEDCBA99);
                        ///
                        /// assert_eq!(n.rotate_left(32), m);
                        /// ```
                        #[inline]
                        #[must_use = "this returns the result of the operation, \
                              without modifying the original"]
                        fn rotate_left(self, other: $o) -> Self {
                            let rot: u32 = (BigInt::from(other) % Self::BITS).to_u32().unwrap();
                            let big: BigInt = BigInt::from(self);
                            let big_rot = big.clone().shl(rot);
                            big_rot.bitor(big.shr(Self::BITS - rot)).try_into().unwrap()
                        }

                        /// Shifts the bits to the right by a specified amount, `n`,
                        /// wrapping the truncated bits to the beginning of the resulting
                        /// integer.
                        ///
                        /// Please note this isn't the same operation as the `>>` shifting
                        /// operator! This method can not overflow as opposed to '>>'.
                        ///
                        /// # Examples
                        ///
                        /// Basic usage:
                        ///
                        /// ```
                        #[doc = "use scrypto::math::" $t ";"]
                        ///
                        /// let n: $t = $t(0x0123456789ABCDEF);
                        /// let m: $t = $t(-0xFEDCBA987654322);
                        ///
                        /// assert_eq!(n.rotate_right(4), m);
                        /// ```
                        #[inline]
                        #[must_use = "this returns the result of the operation, \
                              without modifying the original"]
                        fn rotate_right(self, other: $o) -> Self {
                            let rot: u32 = (BigInt::from(other) % Self::BITS).to_u32().unwrap();
                            let big: BigInt = BigInt::from(self);
                            let big_rot = big.clone().shr(rot);
                            big_rot.bitor(big.shl(Self::BITS - rot)).try_into().unwrap()
                        }
                    }

                    )*
            }
        };
    }
checked_impl! {
//(self, other, output)
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

macro_rules! checked_impl_neg_signed {
    ($($i:ident),*) => {
        $(
            impl Neg for $i {
                type Output = Self;
                #[inline]
                fn neg(self) -> Self {
                    Self::zero() - self
                }
            }
            forward_ref_unop! { impl Neg, neg for $i }
        )*
    }
}

checked_impl_neg_signed! {I8, I16, I32, I64, I128, I256, I384, I512}

macro_rules! checked_int_impl_large {
    (type_id: $t:ident, bytes_len: $bytes_len:literal, MIN: $min: expr, MAX: $max: expr) => {
        paste! {
            impl $t {
                /// Returns the smallest value that can be represented by this integer type.
                ///
                /// # Examples
                ///
                /// Basic usage:
                ///
                /// ```
                #[doc = "use scrypto::math::" $t ";"]
                ///
                #[doc = "assert_eq!(<$t>::MIN, $t(" $bytes_len "::MIN));"]
                /// ```
                pub const MIN: Self = $min;

                /// Returns the largest value that can be represented by this integer type.
                ///
                /// # Examples
                ///
                /// Basic usage:
                ///
                /// ```
                #[doc = "use scrypto::math::" $t ";"]
                ///
                #[doc = "assert_eq!(<$t>::MAX, $t(" $t "::MAX));"]
                /// ```
                pub const MAX: Self = $max;

                /// Returns the size of this integer type in bits.
                ///
                /// # Examples
                ///
                /// Basic usage:
                ///
                /// ```
                #[doc = "use scrypto::math::" $t ";"]
                ///
                #[doc = concat!("assert_eq!(", stringify!($t), "::BITS, ", stringify!(<$t>::BITS.toString()), ");")]
                /// ```
                pub const BITS: u32 = $bytes_len * 8;

                /// Returns the number of ones in the binary representation of `self`.
                ///
                /// # Examples
                ///
                /// Basic usage:
                ///
                /// ```
                #[doc = "use scrypto::math::" $t ";"]
                ///
                #[doc = "let n = $t::from(0b01001100" $t ");"]
                ///
                /// assert_eq!(n.count_ones(), 3);
                /// ```
                #[inline]
                #[doc(alias = "popcount")]
                #[doc(alias = "popcnt")]
                #[must_use = "this returns the result of the operation, \
                          without modifying the original"]
                pub fn count_ones(self) -> u32 {
                    self.0.to_vec().iter().map(|&x| x.count_ones()).sum()
                }

                /// Returns the number of zeros in the binary representation of `self`.
                ///
                /// # Examples
                ///
                /// Basic usage:
                ///
                /// ```
                #[doc = "use scrypto::math::" $t ";"]
                ///
                #[doc = "assert_eq!($t(!0" $t ").count_zeros(), 0);"]
                /// ```
                #[inline]
                #[must_use = "this returns the result of the operation, \
                          without modifying the original"]
                pub fn count_zeros(self) -> u32 {
                    self.0.to_vec().iter().map(|&x| x.count_zeros()).sum()
                }

                /// Returns the number of trailing zeros in the binary representation of `self`.
                ///
                /// # Examples
                ///
                /// Basic usage:
                ///
                /// ```
                #[doc = "use scrypto::math::" $t ";"]
                ///
                #[doc = "let n = $t(0b0101000" $t ");"]
                ///
                /// assert_eq!(n.trailing_zeros(), 3);
                /// ```
                #[inline]
                #[must_use = "this returns the result of the operation, \
                          without modifying the original"]
                pub fn trailing_zeros(self) -> u32 {
                    let mut zeros: u32 = 0;
                    for byte in self.0.to_vec().iter() {
                        let x = byte.trailing_zeros();
                        if x != 8 {
                            return zeros + x;
                        }
                        zeros += 8;
                    }
                    zeros
                }

                /// Reverses the byte order of the integer.
                ///
                /// # Examples
                ///
                /// Basic usage:
                ///
                /// ```
                #[doc = "use scrypto::math::" $t ";"]
                ///
                /// let n: $t = $t(0b0000000_01010101);
                /// assert_eq!(n, $t(85));
                ///
                /// let m = n.swap_bytes();
                ///
                /// assert_eq!(m, $t(0b01010101_00000000));
                /// assert_eq!(m, $t(21760));
                /// ```
                #[inline]
                #[must_use = "this returns the result of the operation, \
                          without modifying the original"]
                pub fn swap_bytes(self) -> Self {
                    $t(self.0.into_iter().rev().collect::<Vec<u8>>().try_into().unwrap())
                }

                /// Reverses the bit pattern of the integer.
                ///
                /// # Examples
                ///
                /// Please note that this example is shared between integer types.
                /// Which explains why `i16` is used here.
                ///
                /// Basic usage:
                ///
                /// ```
                #[doc = "use scrypto::math::" $t ";"]
                ///
                /// let n = $t(0b0000000_01010101i16);
                /// assert_eq!(n, $t(85));
                ///
                /// let m = n.reverse_bits();
                ///
                /// assert_eq!(m.0 as u16, 0b10101010_00000000);
                /// assert_eq!(m, $t(-22016));
                /// ```
                #[must_use = "this returns the result of the operation, \
                          without modifying the original"]
                #[inline]
                pub fn reverse_bits(self) -> Self {
                    $t(self.0.into_iter().rev().map(|x| x.reverse_bits()).collect::<Vec<u8>>().try_into().unwrap())
                }

                /// Converts an integer from big endian to the target's endianness.
                ///
                /// On big endian this is a no-op. On little endian the bytes are
                /// swapped.
                ///
                /// # Examples
                ///
                /// Basic usage:
                ///
                /// ```
                #[doc = "use scrypto::math::" $t ";"]
                ///
                #[doc = "let n = $t(0x1A" $t ");"]
                ///
                /// if cfg!(target_endian = "big") {
                #[doc = "    assert_eq!(<$t>::from_be(n), n)"]
                /// } else {
                #[doc = "    assert_eq!(<$t>::from_be(n), n.swap_bytes())"]
                /// }
                /// ```
                #[inline]
                #[must_use]
                pub fn from_be(x: Self) -> Self {
                    if cfg!(target_endian = "big") {
                        $t(x.0)
                    } else {
                        $t(x.0.into_iter().rev().collect::<Vec<u8>>().try_into().unwrap())
                    }
                }

                /// Converts an integer from little endian to the target's endianness.
                ///
                /// On little endian this is a no-op. On big endian the bytes are
                /// swapped.
                ///
                /// # Examples
                ///
                /// Basic usage:
                ///
                /// ```
                #[doc = "use scrypto::math::" $t ";"]
                ///
                #[doc = "let n = $t(0x1A" $t ");"]
                ///
                /// if cfg!(target_endian = "little") {
                #[doc = "    assert_eq!(<$t>::from_le(n), n)"]
                /// } else {
                #[doc = "    assert_eq!(<$t>::from_le(n), n.swap_bytes())"]
                /// }
                /// ```
                #[inline]
                #[must_use]
                pub fn from_le(x: Self) -> Self {
                    if cfg!(target_endian = "little") {
                        $t(x.0)
                    } else {
                        $t(x.0.into_iter().rev().collect::<Vec<u8>>().try_into().unwrap())
                    }
                }

                /// Converts `self` to big endian from the target's endianness.
                ///
                /// On big endian this is a no-op. On little endian the bytes are
                /// swapped.
                ///
                /// # Examples
                ///
                /// Basic usage:
                ///
                /// ```
                #[doc = "use scrypto::math::" $t ";"]
                ///
                #[doc = "let n = $t(0x1A" $t ");"]
                ///
                /// if cfg!(target_endian = "big") {
                ///     assert_eq!(n.to_be(), n)
                /// } else {
                ///     assert_eq!(n.to_be(), n.swap_bytes())
                /// }
                /// ```
                #[inline]
                #[must_use = "this returns the result of the operation, \
                          without modifying the original"]
                pub fn to_be(self) -> Self {
                    if cfg!(target_endian = "big") {
                        $t(self.0)
                    } else {
                        $t(self.0.into_iter().rev().collect::<Vec<u8>>().try_into().unwrap())
                    }
                }

                /// Converts `self` to little endian from the target's endianness.
                ///
                /// On little endian this is a no-op. On big endian the bytes are
                /// swapped.
                ///
                /// # Examples
                ///
                /// Basic usage:
                ///
                /// ```
                #[doc = "use scrypto::math::" $t ";"]
                ///
                #[doc = "let n = $t(0x1A" $t ");"]
                ///
                /// if cfg!(target_endian = "little") {
                ///     assert_eq!(n.to_le(), n)
                /// } else {
                ///     assert_eq!(n.to_le(), n.swap_bytes())
                /// }
                /// ```
                #[inline]
                #[must_use = "this returns the result of the operation, \
                          without modifying the original"]
                pub fn to_le(self) -> Self {
                    if cfg!(target_endian = "little") {
                        $t(self.0)
                    } else {
                        $t(self.0.into_iter().rev().collect::<Vec<u8>>().try_into().unwrap())
                    }
                }

            }

            impl ToPrimitive for $t {
                fn to_i64(&self) -> Option<i64> {
                    BigInt::from(*self).to_i64()
                }
                fn to_i128(&self) -> Option<i128> {
                    BigInt::from(*self).to_i128()
                }
                fn to_u64(&self) -> Option<u64> {
                    BigInt::from(*self).to_u64()
                }
                fn to_u128(&self) -> Option<u128> {
                    BigInt::from(*self).to_u128()
                }
            }
        }
    }
}

macro_rules! checked_unsigned_large {
    ($($t:ident, $bytes_len:literal),*) => {
        $(
            checked_int_impl_large! {
                type_id: $t,
                bytes_len: $bytes_len,
                MIN: $t([0u8; $bytes_len]),
                MAX: $t([0xffu8; $bytes_len])
            }
        )*
    }
}

macro_rules! checked_signed_large {
    ( $($t:ident, $bytes_len:literal),* ) => {
        $(
            checked_int_impl_large! {
                type_id: $t,
                bytes_len: $bytes_len,
                MIN: {
                    let mut arr = [0u8; $bytes_len];
                    arr[$bytes_len - 1] = 0x80;
                    $t(arr)
                },
                MAX: {
                    let mut arr = [0xff; $bytes_len];
                    arr[$bytes_len - 1] = 0x7f;
                    $t(arr)
                }
            }
        )*
    }
}

checked_signed_large! {
    I256, 32,
    I384, 48,
    I512, 64
}

checked_unsigned_large! {
    U256, 32,
    U384, 48,
    U512, 64
}

macro_rules! checked_int_impl_small {
    ($($t:ident),*) => {$(
        paste! {
            impl $t {
                /// Returns the smallest value that can be represented by this integer type.
                ///
                /// # Examples
                ///
                /// Basic usage:
                ///
                /// ```
                #[doc = "use scrypto::math::" $t ";"]
                ///
                #[doc = concat!("assert_eq!(", stringify!($t), "::MIN, ", stringify!(<$t:lower>::MIN), ");")]
                /// ```
                pub const MIN: Self = Self([<$t:lower>]::MIN);

                /// Returns the largest value that can be represented by this integer type.
                ///
                /// # Examples
                ///
                /// Basic usage:
                ///
                /// ```
                #[doc = "use scrypto::math::" $t ";"]
                ///
                #[doc = concat!("assert_eq!(", stringify!($t), "::MAX, ", stringify!(<$t:lower>::MAX), ");")]
                /// ```
                pub const MAX: Self = Self([<$t:lower>]::MAX);

                /// Returns the size of this integer type in bits.
                ///
                /// # Examples
                ///
                /// Basic usage:
                ///
                /// ```
                #[doc = "use scrypto::math::" $t ";"]
                ///
                #[doc = concat!("assert_eq!(<", stringify!($t), "::BITS, ", stringify!(<$t>::BITS), ");")]
                /// ```
                pub const BITS: u32 = [<$t:lower>]::BITS;

                /// Returns the number of ones in the binary representation of `self`.
                ///
                /// # Examples
                ///
                /// Basic usage:
                ///
                /// ```
                #[doc = "use scrypto::math::" $t ";"]
                ///
                #[doc = "let n = $t(0b01001100" $t ");"]
                ///
                /// assert_eq!(n.count_ones(), 3);
                /// ```
                #[inline]
                #[doc(alias = "popcount")]
                #[doc(alias = "popcnt")]
                #[must_use = "this returns the result of the operation, \
                          without modifying the original"]
                pub const fn count_ones(self) -> u32 {
                    self.0.count_ones()
                }

                /// Returns the number of zeros in the binary representation of `self`.
                ///
                /// # Examples
                ///
                /// Basic usage:
                ///
                /// ```
                #[doc = "use scrypto::math::" $t ";"]
                ///
                #[doc = "assert_eq!($t(!0" $t ").count_zeros(), 0);"]
                /// ```
                #[inline]
                #[must_use = "this returns the result of the operation, \
                          without modifying the original"]
                pub const fn count_zeros(self) -> u32 {
                    self.0.count_zeros()
                }

                /// Returns the number of trailing zeros in the binary representation of `self`.
                ///
                /// # Examples
                ///
                /// Basic usage:
                ///
                /// ```
                #[doc = "use scrypto::math::" $t ";"]
                ///
                #[doc = "let n = $t(0b0101000" $t:lower ");"]
                ///
                /// assert_eq!(n.trailing_zeros(), 3);
                /// ```
                #[inline]
                #[must_use = "this returns the result of the operation, \
                          without modifying the original"]
                pub const fn trailing_zeros(self) -> u32 {
                    self.0.trailing_zeros()
                }
                /// Reverses the byte order of the integer.
                ///
                /// # Examples
                ///
                /// Basic usage:
                ///
                /// ```
                #[doc = "use scrypto::math::" $t ";"]
                ///
                /// let n: $t = $t(0b0000000_01010101);
                /// assert_eq!(n, $t(85));
                ///
                /// let m = n.swap_bytes();
                ///
                /// assert_eq!(m, $t(0b01010101_00000000));
                /// assert_eq!(m, $t(21760));
                /// ```
                #[inline]
                #[must_use = "this returns the result of the operation, \
                          without modifying the original"]
                pub const fn swap_bytes(self) -> Self {
                    $t(self.0.swap_bytes())
                }

                /// Reverses the bit pattern of the integer.
                ///
                /// # Examples
                ///
                /// Please note that this example is shared between integer types.
                /// Which explains why `i16` is used here.
                ///
                /// Basic usage:
                ///
                /// ```
                #[doc = "use scrypto::math::" $t ";"]
                ///
                /// let n = $t(0b0000000_01010101i16);
                /// assert_eq!(n, $t(85));
                ///
                /// let m = n.reverse_bits();
                ///
                /// assert_eq!(m.0 as u16, 0b10101010_00000000);
                /// assert_eq!(m, $t(-22016));
                /// ```
                #[must_use = "this returns the result of the operation, \
                          without modifying the original"]
                #[inline]
                pub const fn reverse_bits(self) -> Self {
                    $t(self.0.reverse_bits())
                }

                /// Converts an integer from big endian to the target's endianness.
                ///
                /// On big endian this is a no-op. On little endian the bytes are
                /// swapped.
                ///
                /// # Examples
                ///
                /// Basic usage:
                ///
                /// ```
                #[doc = "use scrypto::math::" $t ";"]
                ///
                #[doc = "let n = $t(0x1A" $t:lower ");"]
                ///
                /// if cfg!(target_endian = "big") {
                #[doc = "    assert_eq!(<$t>::from_be(n), n)"]
                /// } else {
                #[doc = "    assert_eq!(<$t>::from_be(n), n.swap_bytes())"]
                /// }
                /// ```
                #[inline]
                #[must_use]
                pub const fn from_be(x: Self) -> Self {
                    if cfg!(target_endian = "big") {
                        x
                    } else {
                        $t([<$t:lower>]::from_be(x.0))
                    }
                }

                /// Converts an integer from little endian to the target's endianness.
                ///
                /// On little endian this is a no-op. On big endian the bytes are
                /// swapped.
                ///
                /// # Examples
                ///
                /// Basic usage:
                ///
                /// ```
                #[doc = "use scrypto::math::" $t ";"]
                ///
                #[doc = "let n = $t(0x1A" $t:lower ");"]
                ///
                /// if cfg!(target_endian = "little") {
                #[doc = "    assert_eq!(<$t>::from_le(n), n)"]
                /// } else {
                #[doc = "    assert_eq!(<$t>::from_le(n), n.swap_bytes())"]
                /// }
                /// ```
                #[inline]
                #[must_use]
                pub const fn from_le(x: Self) -> Self {
                    if cfg!(target_endian = "big") {
                        $t([<$t:lower>]::from_le(x.0))
                    } else {
                        x
                    }
                }

                /// Converts `self` to big endian from the target's endianness.
                ///
                /// On big endian this is a no-op. On little endian the bytes are
                /// swapped.
                ///
                /// # Examples
                ///
                /// Basic usage:
                ///
                /// ```
                #[doc = "use scrypto::math::" $t ";"]
                ///
                #[doc = "let n = $t(0x1A" $t:lower ");"]
                ///
                /// if cfg!(target_endian = "big") {
                ///     assert_eq!(n.to_be(), n)
                /// } else {
                ///     assert_eq!(n.to_be(), n.swap_bytes())
                /// }
                /// ```
                #[inline]
                #[must_use = "this returns the result of the operation, \
                          without modifying the original"]
                pub const fn to_be(self) -> Self {
                    if cfg!(target_endian = "big") {
                        self
                    } else {
                        $t(self.0.to_be())
                    }
                }

                /// Converts `self` to little endian from the target's endianness.
                ///
                /// On little endian this is a no-op. On big endian the bytes are
                /// swapped.
                ///
                /// # Examples
                ///
                /// Basic usage:
                ///
                /// ```
                #[doc = "use scrypto::math::" $t ";"]
                ///
                #[doc = "let n = $t(0x1A" $t:lower ");"]
                ///
                /// if cfg!(target_endian = "little") {
                ///     assert_eq!(n.to_le(), n)
                /// } else {
                ///     assert_eq!(n.to_le(), n.swap_bytes())
                /// }
                /// ```
                #[inline]
                #[must_use = "this returns the result of the operation, \
                          without modifying the original"]
                pub const fn to_le(self) -> Self {
                    if cfg!(target_endian = "big") {
                        $t(self.0.to_le())
                    } else {
                        self
                    }
                }
            }

            impl ToPrimitive for $t {
                fn to_i64(&self) -> Option<i64> {
                    i64::try_from(self.0).ok()
                }
                fn to_i128(&self) -> Option<i128> {
                    i128::try_from(self.0).ok()
                }
                fn to_u64(&self) -> Option<u64> {
                    u64::try_from(self.0).ok()
                }
                fn to_u128(&self) -> Option<u128> {
                    u128::try_from(self.0).ok()
                }
            }
        }
        )*}
}

checked_int_impl_small! { I8, I16, I32, I64, I128, U8, U16, U32, U64, U128 }

macro_rules! leading_zeros_large {
    () => {
        /// Returns the number of leading zeros in the binary representation of `self`.
        ///
        #[inline]
        #[must_use = "this returns the result of the operation, \
                          without modifying the original"]
        pub fn leading_zeros(self) -> u32 {
            let mut zeros: u32 = u32::zero();
            for i in self.0.into_iter().rev().enumerate() {
                if i.1 != 0 {
                    return zeros + i.1.leading_zeros();
                }
                zeros += 8;
            }
            zeros
        }
    };
}

macro_rules! leading_zeros_small {
    () => {
        /// Returns the number of leading zeros in the binary representation of `self`.
        ///
        #[inline]
        #[must_use = "this returns the result of the operation, \
                          without modifying the original"]
        pub fn leading_zeros(self) -> u32 {
            self.0.leading_zeros()
        }
    };
}

macro_rules! checked_int_impl_signed {
    ($($t:ident, $self:ident, $leading_zeros:item, $base:expr),*) => ($(
            paste! {
                impl $t {

                    $leading_zeros

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
                    #[doc = "use scrypto::math::" $t ";"]
                    ///
                    #[doc = "assert_eq!($t(10" $t:lower ").signum(), $t(1));"]
                    #[doc = "assert_eq!($t(0" $t:lower ").signum(), $t(0));"]
                    #[doc = "assert_eq!($t(-10" $t:lower ").signum(), $t(-1));"]
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
                    #[doc = "use scrypto::math::" $t ";"]
                    ///
                    #[doc = "assert!($t(10" $t:lower ").is_positive());"]
                    #[doc = "assert!(!$t(-10" $t:lower ").is_positive());"]
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
                    #[doc = "use scrypto::math::" $t ";"]
                    ///
                    #[doc = "assert!($t(-10" $t:lower ").is_negative());"]
                    #[doc = "assert!(!$t(10" $t:lower ").is_negative());"]
                    /// ```
                    #[must_use]
                    #[inline]
                    pub fn is_negative($self) -> bool {
                        $base.is_negative().try_into().unwrap()
                            // large: self.0.to_vec().into_iter().nth(self.0.len() - 1).unwrap() & 0x80 == 1
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
            leading_zeros_large!{},
            BigInt::from(self)
        }
    )*}
}

macro_rules! checked_int_impl_signed_all_small {
    ($($t:ident),*) => {$(
        checked_int_impl_signed! {
            $t,
            self,
            leading_zeros_small!{},
            self.0
        }
    )*}
}

checked_int_impl_signed_all_large! { I256, I384, I512 }
checked_int_impl_signed_all_small! { I8, I16, I32, I64, I128 }

macro_rules! checked_int_impl_unsigned_large {
    ($($t:ty),*) => ($(
            impl $t {
                leading_zeros_large!();

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
                /// `uN`), overflows to `2^N = 0`.
                ///
                #[inline]
                #[must_use = "this returns the result of the operation, \
                          without modifying the original"]
                pub fn next_power_of_two(self) -> Self {
                    (Self::BITS - self.leading_zeros()).into()
                }
            }
    )*)
}

macro_rules! checked_int_impl_unsigned_small {
    ($($t:ty),*) => ($(
            impl $t {
                leading_zeros_small!();

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

macro_rules! try_from_builtin {
    ($t:ident, ($($o:ident),*)) => {
        $(
            paste! {
                impl TryFrom<$o> for $t {
                    type Error = ParseIntError;
                    fn try_from(val: $o) -> Result<Self, Self::Error> {
                        (BigInt::from(val)).try_into().map_err(|_| ParseIntError::Overflow)
                    }
                }
            }
        )*
    };
}

try_from_builtin! {I8, (u8, u16, u32, u64, u128, i16, i32, i64, i128)}
try_from_builtin! {I16, (i32, i64, i128, u16, u32, u64, u128)}
try_from_builtin! {I32, (i64, i128, u32, u64, u128)}
try_from_builtin! {I64, (i128, u64, u128)}
try_from_builtin! {I128, (u128)}
try_from_builtin! {U8, (i8, i16, i32, i64, i128, u16, u32, u64, u128)}
try_from_builtin! {U16, (i8, i16, i32, i64, i128, u32, u64, u128)}
try_from_builtin! {U32, (i8, i16, i32, i64, i128, u64, u128)}
try_from_builtin! {U64, (i8, i16, i32, i64, i128, u128)}
try_from_builtin! {U128, (i8, i16, i32, i64, i128)}
try_from_builtin! {U256, (i8, i16, i32, i64, i128)}
try_from_builtin! {U384, (i8, i16, i32, i64, i128)}
try_from_builtin! {U512, (i8, i16, i32, i64, i128)}

macro_rules! try_from{
    ($t:ident, ($($o:ident),*)) => {
        $(
            paste! {
                impl TryFrom<$o> for $t {
                    type Error = ParseIntError;
                    fn try_from(val: $o) -> Result<$t, ParseIntError> {
                        BigInt::from(val).try_into()
                    }
                }
            }
        )*
    };
}

macro_rules! try_from_large_all {
    ($($t:ident),*) => {
        $(
            try_from! { $t, (I256, I384, I512) }
            try_from! { $t, (U256, U384, U512) }
        )*
    };
}

macro_rules! try_from_small_all {
    ($($t:ident),*) => {
        $(
            try_from! { $t, (I8, I16, I32, I64, I128) }
            try_from! { $t, (U8, U16, U32, U64, U128) }
        )*
    };
}

try_from_large_all! { U8, U16, U32, U64, U128, I8, I16, I32, I64, I128 }
try_from_small_all! { U256, U384, U512, I256, I384, I512 }

try_from! {U256, (I256, I384, I512)}
try_from! {U384, (I256, I384, I512)}
try_from! {U512, (I256, I384, I512)}
try_from! {U256, (U384, U512)}
try_from! {U384, (U256, U512)}
try_from! {U512, (U256, U384)}
try_from! {I256, (U256, U384, U512)}
try_from! {I384, (U256, U384, U512)}
try_from! {I512, (U256, U384, U512)}
try_from! {I256, (I384, I512)}
try_from! {I384, (I256, I512)}
try_from! {I512, (I256, I384)}
try_from! {U8, (I8, I16, I32, I64, I128)}
try_from! {U16, (I8, I16, I32, I64, I128)}
try_from! {U32, (I8, I16, I32, I64, I128)}
try_from! {U64, (I8, I16, I32, I64, I128)}
try_from! {U128, (I8, I16, I32, I64, I128)}
try_from! {I8, (U8, U16, U32, U64, U128)}
try_from! {I16, (U8, U16, U32, U64, U128)}
try_from! {I32, (U8, U16, U32, U64, U128)}
try_from! {I64, (U8, U16, U32, U64, U128)}
try_from! {I128, (U8, U16, U32, U64, U128)}
try_from! {U8, (U16, U32, U64, U128)}
try_from! {U16, (U8, U32, U64, U128)}
try_from! {U32, (U8, U16, U64, U128)}
try_from! {U64, (U8, U16, U32, U128)}
try_from! {U128, (U8, U16, U32, U64)}
try_from! {I8, (I16, I32, I64, I128)}
try_from! {I16, (I8, I32, I64, I128)}
try_from! {I32, (I8, I16, I64, I128)}
try_from! {I64, (I8, I16, I32, I128)}
try_from! {I128, (I8, I16, I32, I64)}

macro_rules! impl_bigint_to_large_unsigned {
    ($($t:ty),*) => {
        $(
            paste! {
                fn [<bigint_to_$t:lower>](b: BigInt) -> Result<$t, ParseIntError> {
                    let (sign, bytes) = b.to_bytes_le();
                    if sign == Sign::Minus {
                        return Err(ParseIntError::NegativeToUnsigned);
                    }
                    const T_BYTES: usize = (<$t>::BITS / 8 ) as usize;
                    if bytes.len() > T_BYTES {
                        return Err(ParseIntError::Overflow);
                    }
                    let mut buf = [0u8; T_BYTES];
                    buf[..bytes.len()].copy_from_slice(&bytes);
                    Ok($t(buf))
                }
            }
        )*
    }
}

macro_rules! impl_bigint_to_large_signed {
    ($($t:ty),*) => {
        $(
            paste! {
                fn [<bigint_to_$t:lower>](b: BigInt) -> Result<$t, ParseIntError> {
                    let bytes = b.to_signed_bytes_le();
                    const T_BYTES: usize = (<$t>::BITS / 8 ) as usize;
                    if bytes.len() > T_BYTES {
                        return Err(ParseIntError::Overflow);
                    }
                    let mut buf = if b.is_negative() {
                        [255u8; T_BYTES]
                    } else {
                        [0u8; T_BYTES]
                    };
                    buf[..bytes.len()].copy_from_slice(&bytes);
                    Ok($t(buf))
                }
            }
        )*
    }
}

macro_rules! impl_bigint_to_small_unsigned {
    ($($t:ty),*) => {
        $(
            paste! {
                fn [<bigint_to_$t:lower>](b: BigInt) -> Result<$t, ParseIntError> {
                    if b.is_negative() {
                        return Err(ParseIntError::NegativeToUnsigned);
                    }
                    Ok($t(b.[<to_$t:lower>]().ok_or_else(|| ParseIntError::Overflow).unwrap()))
                }
            }
        )*
    }
}

macro_rules! impl_bigint_to_small_signed {
    ($($t:ty),*) => {
        $(
            paste! {
                fn [<bigint_to_$t:lower>](b: BigInt) -> Result<$t, ParseIntError> {
                    Ok($t(b.[<to_$t:lower>]().ok_or_else(|| ParseIntError::Overflow).unwrap()))
                }
            }
        )*
    }
}

impl_bigint_to_large_signed! { I256, I384, I512 }
impl_bigint_to_large_unsigned! { U256, U384, U512 }
impl_bigint_to_small_signed! { I8, I16, I32, I64, I128 }
impl_bigint_to_small_unsigned! { U8, U16, U32, U64, U128 }

macro_rules! try_from_big_int_to_signed {
    ($($t:ident),*) => {
        $(
            paste! {
                impl TryFrom<BigInt> for $t {
                    type Error = ParseIntError;
                    fn try_from(val: BigInt) -> Result<$t, ParseIntError> {
                        [<bigint_to_$t:lower>](val)
                    }
                }
            }
        )*
    };
}

macro_rules! try_from_big_int_to_unsigned {
    ($($t:ident),*) => {
        $(
            paste! {
                impl TryFrom<BigInt> for $t {
                    type Error = ParseIntError;

                    fn try_from(val: BigInt) -> Result<Self, Self::Error>  {
                        if val.is_negative() {
                            return Err(ParseIntError::NegativeToUnsigned);
                        }
                        [<bigint_to_$t:lower>](val)
                    }
                }
            }
        )*
    };
}

macro_rules! from_array {
    ($($t:ident),*) => {
        $(
            paste! {
                impl From<[u8; (<$t>::BITS / 8) as usize]> for $t {
                    fn from(val: [u8; (<$t>::BITS / 8) as usize]) -> Self {
                        Self(val)
                    }
                }
            }
        )*
    };
}

#[derive(Debug)]
pub enum ParseSliceError {
    InvalidLength,
}

macro_rules! try_from_vec_and_slice {
    ($($t:ident),*) => {
        $(
            impl TryFrom<&[u8]> for $t {
                type Error = ParseSliceError;
                fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
                    if bytes.len() > (<$t>::BITS / 8) as usize {
                        Err(ParseSliceError::InvalidLength)
                    } else {
                        let mut buf = [0u8; (<$t>::BITS / 8) as usize];
                        buf[..bytes.len()].copy_from_slice(bytes);
                        Ok(Self(buf))
                    }
                }
            }

            impl TryFrom<Vec<u8>> for $t {
                type Error = ParseSliceError;
                fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
                    if bytes.len() > (<$t>::BITS / 8) as usize {
                        Err(ParseSliceError::InvalidLength)
                    } else {
                        let mut buf = [0u8; (<$t>::BITS / 8) as usize];
                        buf[..bytes.len()].copy_from_slice(&bytes);
                        Ok(Self(buf))
                    }
                }
            }
            )*
    };
}

macro_rules! from_int {
    ($(($t:ident, $o:ident)),*) => {
        $(
            paste! {
                impl From<$o> for $t {
                    fn from(val: $o) -> Self {
                        (BigInt::from(val)).try_into().unwrap()
                    }
                }
            }
        )*
    };
}

from_int! {(I8, i8)}

from_int! {(I16, i8), (I16, i16)}
from_int! {(I16, u8)}

from_int! {(I32, i8), (I32, i16), (I32, i32)}
from_int! {(I32, u8), (I32, u16)}

from_int! {(I64, i8), (I64, i16), (I64, i32), (I64, i64)}
from_int! {(I64, u8), (I64, u16), (I64, u32)}

from_int! {(I128, i8), (I128, i16), (I128, i32), (I128, i64), (I128, i128)}
from_int! {(I128, u8), (I128, u16), (I128, u32), (I128, u64)}

from_int! {(I256, i8), (I256, i16), (I256, i32), (I256, i64), (I256, i128)}
from_int! {(I256, u8), (I256, u16), (I256, u32), (I256, u64), (I256, u128)}

from_int! {(I384, i8), (I384, i16), (I384, i32), (I384, i64), (I384, i128)}
from_int! {(I384, u8), (I384, u16), (I384, u32), (I384, u64), (I384, u128)}

from_int! {(I512, i8), (I512, i16), (I512, i32), (I512, i64), (I512, i128)}
from_int! {(I512, u8), (I512, u16), (I512, u32), (I512, u64), (I512, u128)}

from_int! {(U8, u8)}

from_int! {(U16, u8), (U16, u16)}

from_int! {(U32, u8), (U32, u16), (U32, u32)}

from_int! {(U64, u8), (U64, u16), (U64, u32), (U64, u64)}

from_int! {(U128, u8), (U128, u16), (U128, u32), (U128, u64), (U128, u128)}

from_int! {(U256, u8), (U256, u16), (U256, u32), (U256, u64), (U256, u128)}

from_int! {(U384, u8), (U384, u16), (U384, u32), (U384, u64), (U384, u128)}

from_int! {(U512, u8), (U512, u16), (U512, u32), (U512, u64), (U512, u128)}

try_from_big_int_to_signed! { I8, I16, I32, I64, I128, I256, I384, I512 }
try_from_big_int_to_unsigned! { U8, U16, U32, U64, U128, U256, U384, U512 }
try_from_vec_and_slice! { I256, I384, I512, U256, U384, U512 }
from_array! { I256, I384, I512, U256, U384, U512 }
macro_rules! from_string {
    ($($t:ident),*) => {
        $(
            impl FromStr for $t {
                type Err = ParseBigIntError;
                fn from_str(val: &str) -> Result<Self, Self::Err> {
                    match val.parse::<BigInt>() {
                        Ok(big_int) => Ok($t::try_from(big_int).unwrap()),
                        Err(e) => Err(e)
                    }
                }
            }

            impl From<&str> for $t {
                fn from(val: &str) -> Self {
                    Self::from_str(&val).unwrap()
                }
            }

            impl From<String> for $t {
                fn from(val: String) -> Self {
                    Self::from_str(&val).unwrap()
                }
            }
        )*
    };
}

from_string! { I8, I16, I32, I64, I128, I256, I384, I512 }
from_string! { U8, U16, U32, U64, U128, U256, U384, U512 }

macro_rules! big_int_from {
    (U256) => {
        to_big_int_from_large_unsigned!{U256}
    };
    (I256) => {
        to_big_int_from_large_signed!{I256}
    };
    (U384) => {
        to_big_int_from_large_unsigned!{U384}
    };
    (I384) => {
        to_big_int_from_large_signed!{I384}
    };
    (U512) => {
        to_big_int_from_large_unsigned!{U512}
    };
    (I512) => {
        to_big_int_from_large_signed!{I512}
    };
    ($t:ident) => {
        to_big_int_from_small!{$t}
    };
}

macro_rules! to_big_int_from_large_unsigned {
    ($t:ident) => {
            impl From<$t> for BigInt {
                fn from(val: $t) -> BigInt {
                    BigInt::from_bytes_le(Sign::Plus, &val.0)
                }
            }
    };
}

macro_rules! to_big_int_from_large_signed {
    ($t:ident) => {
            impl From<$t> for BigInt {
                fn from(val: $t) -> BigInt {
                    BigInt::from_signed_bytes_le(&val.0)
                }
            }
    };
}

macro_rules! to_big_int_from_small {
    ($t:ident) => {
            impl From<$t> for BigInt {
                fn from(val: $t) -> BigInt{
                    BigInt::from(val.0)
                }
            }
    };
}

big_int_from!{I8}
big_int_from!{I16}
big_int_from!{I32}
big_int_from!{I64}
big_int_from!{I128}
big_int_from!{I256}
big_int_from!{I384}
big_int_from!{I512}
big_int_from!{U8}
big_int_from!{U16}
big_int_from!{U32}
big_int_from!{U64}
big_int_from!{U128}
big_int_from!{U256}
big_int_from!{U384}
big_int_from!{U512}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_from_builtin {
        ($i:ident, ($($t:ident),*)) => {
            paste! {
                $(
                    #[test]
                    fn [<from_builtin_$i:lower _ $t:lower>]() {
                        let b = <$i>::[<from_$t>](127).unwrap();
                        assert_eq!(b.to_string(), "127");
                    }
                )*
            }
        };
    }

    macro_rules! test_impl {
        ($($i:ident),*) => ($(

                paste! {
                    #[test]
                    #[should_panic]
                    fn [<test_add_overflow_$i:lower>]() {
                        let a = <$i>::MAX + <$i>::try_from(1u8).unwrap(); // panics on overflow
                        println!("{}.add({}) == {}", [<$i>]::MAX, 1, a);
                    }

                    #[test]
                    #[should_panic]
                    fn  [<test_sub_overflow_$i:lower>]() {
                        let _ = <$i>::MIN - <$i>::try_from(1u8).unwrap(); // panics on overflow
                    }

                    #[test]
                    #[should_panic]
                    fn  [<test_mul_overflow_$i:lower>]() {
                        let _ = <$i>::MAX * <$i>::try_from(2u8).unwrap(); // panics because of overflow
                    }

                    #[test]
                    #[should_panic]
                    fn  [<test_div_overflow_$i:lower>]() {
                        let _ = <$i>::MIN / <$i>::try_from(0u8).unwrap(); // panics because of division by zero
                    }

                    #[test]
                    #[should_panic]
                    fn  [<test_rem_overflow_$i:lower>]() {
                        let _ = <$i>::MIN % $i::try_from(0u8).unwrap(); // panics because of division by zero
                    }

                    #[test]
                    #[should_panic]
                    fn  [<test_shl_overflow_$i:lower>]() {
                        let _ = <$i>::MAX << (<$i>::BITS + 1);  // panics because of overflow
                    }

                    #[test]
                    #[should_panic]
                    fn  [<test_shr_overflow_$i:lower>]() {
                        let _ = <$i>::MAX >> (<$i>::BITS + 1);  // panics because of overflow
                    }

                    #[test]
                    #[should_panic]
                    fn  [<test_shl_overflow_neg_$i:lower>]() {
                        let _ = <$i>::MIN << (<$i>::BITS + 1);  // panics because of overflow
                    }

                    #[test]
                    #[should_panic]
                    fn  [<test_shr_overflow_neg_$i:lower>]() {
                        let _ = <$i>::MIN >> (<$i>::BITS + 1);  // panics because of overflow
                    }

                    #[test]
                    #[should_panic]
                    fn  [<test_pow_overflow_$i:lower>]() {
                        let a = <$i>::MAX.pow(2u8);             // panics because of overflow
                        println!("{}.pow({}) == {}", [<$i>]::MAX, 2, a);
                    }

                    #[test]
                    fn [<test_binary_$i:lower>]() {
                        let bin = <$i>::try_from(0x0b).unwrap();
                        assert_eq!(format!("{:b}", bin), "1011");
                    }

                    #[test]
                    fn [<test_octal_$i:lower>]() {
                        let oct = <$i>::try_from(0x0b).unwrap();
                        assert_eq!(format!("{:o}", oct), "13");
                    }

                    #[test]
                    fn [<test_hex_lower_$i:lower>]() {
                        let hex_lower = <$i>::try_from(0x0b).unwrap();
                        assert_eq!(format!("{:x}", hex_lower), "b");
                    }

                    #[test]
                    fn [<test_hex_upper_$i:lower>]() {
                        let hex_upper = <$i>::try_from(0x0b).unwrap();
                        assert_eq!(format!("{:X}", hex_upper), "B");
                    }

                    #[test]
                    fn [<test_zero_$i:lower>]() {
                        let zero = <$i>::try_from(0u8).unwrap();
                        assert_eq!(zero, <$i>::zero());
                    }

                    #[test]
                    fn [<test_is_zero_$i:lower>]() {
                        let mut zero = <$i>::try_from(0u8).unwrap();
                        assert_eq!(zero.is_zero(), true);
                        zero = <$i>::try_from(1u8).unwrap();
                        assert_eq!(zero.is_zero(), false);
                    }

                    #[test]
                    fn [<test_set_zero_$i:lower>]() {
                        let mut zero = <$i>::try_from(1u8).unwrap();
                        zero.set_zero();
                        assert_eq!(zero.is_zero(), true);
                    }

                    test_from_builtin!{$i, (i8, i16, i32, i64, i128, u8, u16, u32, u64, u128)}


                }
        )*)
    }
    test_impl! { I8, I16, I32, I64, I128, I256, I384, I512, U8, U16, U32, U64, U128, U256, U384, U512 }

    macro_rules! test_add {
        ($i:literal, $i_bits:literal, ($($t:literal, $t_bits:literal),*)) => {
            paste! {
                $(
                    #[test]
                    fn [<test_add_output_type_ $i $i_bits _ $t $t_bits>]() {
                        let my_bits: usize = $i_bits;
                        let other_bits: usize = $t_bits;
                        let out_bits: usize = my_bits.max(other_bits);
                        let out_type_name = if $i == 'I' || $t == 'I' || $t == 'i' {
                            'I'
                        } else {
                            'U'
                        };
                        let a: [<$i $i_bits>] = [<$i $i_bits>]::from_str("1").unwrap();
                        let b: [<$t $t_bits>] = [<$t $t_bits>]::from_str("2").unwrap();
                        assert_eq!(core::any::type_name_of_val(&(a + b)), format!("scrypto::math::integers::{}{}", out_type_name, out_bits));
                    }
                )*
            }
        };
    }

    macro_rules! test_impl_basic_math {
        ($i:literal, ($($i_bits:literal),*)) => {
            $(

                test_add!{ $i, $i_bits, ('i', 8, 'i', 16, 'i', 32, 'i', 64, 'i', 128, 'I', 8, 'I', 16, 'I', 32, 'I', 64, 'I', 128, 'I', 256, 'I', 384, 'I', 512, 'u', 8, 'u', 16, 'u', 32, 'u', 64, 'u', 128, 'U', 8, 'U', 16, 'U', 32, 'U', 64, 'U', 128, 'U', 256, 'U', 384, 'U', 512) }
            )*
        };
    }
    test_impl_basic_math! { 'I', (8, 16, 32, 64, 128, 256, 384, 512) }
    test_impl_basic_math! { 'U', (8, 16, 32, 64, 128, 256, 384, 512) }
}

// TODO: unify checked_impl_small with checked_impl_large
// TODO: try_from for builtin types
// TODO: test write
// TODO: documentationpart update
// TODO: remove FIXME lines
