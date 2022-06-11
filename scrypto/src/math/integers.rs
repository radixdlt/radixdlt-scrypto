//! Definitions of safe integers and uints.

use core::fmt;
use core::ops::{Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign};
use core::ops::{BitXor, BitXorAssign, Div, DivAssign};
use core::ops::{Mul, MulAssign, Neg, Not, Rem, RemAssign};
use core::ops::{Shl, ShlAssign, Shr, ShrAssign, Sub, SubAssign};
use forward_ref::*;
use num_bigint::BigInt;
use num_traits::{FromPrimitive, Num, Signed, ToPrimitive, Zero};
use paste::paste;
use std::cmp::min;
use std::default::Default;

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
                #[doc = concat!("/// `checked_add`, or through the ", stringify!($t) , "type, which ensures all") ]
                /// standard arithmetic operations on the underlying value to have
                /// checked semantics.
                ///
                /// The underlying value can be retrieved through the `.0` index of the
                #[doc = concat!("/// `", stringify!($t), "` tuple.")]
                ///
                /// # Layout
                ///
                #[doc = concat!("/// `", stringify!($t), "` will have the same methods and traits as")]
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

            )*
        }
    }
}

types! {
    self: self,
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
        self_expr: BigInt::from_signed_bytes_le(&self.0)
    },
    {
        type: U384,
        self.0: [u8; 48],
        self.zero(): U384([0u8; 48]),
        Default::default(): U384([0u8; 48]),
        self_expr: BigInt::from_signed_bytes_le(&self.0)
    },
    {
        type: U512,
        self.0: [u8; 64],
        self.zero(): U512([0u8; 64]),
        Default::default(): U512([0u8; 64]),
        self_expr: BigInt::from_signed_bytes_le(&self.0)
    },
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
        self_expr: BigInt::from_signed_bytes_le(&self.0)
    },
    {
        type: I384,
        self.0: [u8; 48],
        self.zero(): I384([0u8; 48]),
        Default::default(): I384([0u8; 48]),
        self_expr: BigInt::from_signed_bytes_le(&self.0)
    },
    {
        type: I512,
        self.0: [u8; 64],
        self.zero(): I512([0u8; 64]),
        Default::default(): I512([0u8; 64]),
        self_expr: BigInt::from_signed_bytes_le(&self.0)
    }
}

#[derive(Debug)]
pub enum ParseBigIntError {
    NegativeToUnsigned,
    Overflow,
}

macro_rules! sh_impl {
    (to_sh: $t:ty, other: $o:ty, other_var: $other:ident, self_var: $self:ident, shl_expr: $shl_expr:expr, shr_expr: $shr_expr:expr ) => {
        paste! {
            impl Shl<$o> for $t {
                type Output = $t;

                #[inline]
                fn shl($self, $other: $o) -> $t {
                    $shl_expr
                }
            }
            forward_ref_binop! { impl Shl, shl for $t, $o }

            impl ShlAssign<$o> for $t {
                #[inline]
                fn shl_assign(&mut self, $other: $o) {
                    *self = *self << $other;
                }
            }
            forward_ref_op_assign! { impl ShlAssign, shl_assign for $t, $o }

            impl Shr<$o> for $t {
                type Output = $t;

                #[inline]
                fn shr($self, $other: $o) -> $t {
                    $shr_expr
                }
            }
            forward_ref_binop! { impl Shr, shr for $t, $o }

            impl ShrAssign<$o> for $t {
                #[inline]
                fn shr_assign(&mut self, $other: $o) {
                    *self = *self >> $other;
                }
            }
            forward_ref_op_assign! { impl ShrAssign, shr_assign for $t, $o }
        }
    };
}

// large: U256, U384, U512, I256, I384, I512
// small: U8, U16, U32, U64, U128, I8, I16, I32, I64, I128
// builtin: u8, u16, u32, u64, u128, i8, i16, i32, i64, i128
macro_rules! sh {
    (large.shl(large_signed), $self:tt, $other:tt, $t:tt) => {
        if BigInt::from_signed_bytes_le(&$other.0).abs() > BigInt::from(<$t>::BITS) {
            panic!("overflow");
        } else {
            let to_shift = BigInt::from_signed_bytes_le(&$self.0);
            let len = min(
                (<$t>::BITS / 8) as usize,
                to_shift.to_signed_bytes_le().len(),
                );
            let shift = BigInt::from_signed_bytes_le(&$other.0).to_i128().unwrap();
            to_shift.shl(shift).to_signed_bytes_le()[..len]
                .try_into()
                .unwrap()
        }
    };
    (large.shr(large_signed), $self:tt, $other:tt, $t:tt) => {
        if BigInt::from_signed_bytes_le(&$other.0).abs() > BigInt::from(<$t>::BITS) {
            panic!("overflow");
        } else {
            let to_shift = BigInt::from_signed_bytes_le(&$self.0);
            let shift = BigInt::from_signed_bytes_le(&$other.0).to_i128().unwrap();
            to_shift.shr(shift).try_into().unwrap()
        }
    };
    (large.shl(large_unsigned), $self:tt, $other:tt, $t:tt) => {{
        let to_shift = BigInt::from_signed_bytes_le(&$self.0);
        let len = min(
            (<$t>::BITS / 8) as usize,
            to_shift.to_signed_bytes_le().len(),
            );
        let shift = BigInt::from_signed_bytes_le(&$other.0).to_i128().unwrap();
        to_shift.shl(shift).to_signed_bytes_le()[..len]
            .try_into()
            .unwrap()
    }};
    (large.shr(large_unsigned), $self:tt, $other:tt, $t:tt) => {{
        let shift = BigInt::from_signed_bytes_le(&$other.0).to_i128().unwrap();
        BigInt::from_signed_bytes_le(&$self.0)
            .shr(shift)
            .try_into()
            .unwrap()
    }};
    (large.shl(builtin), $self:tt, $other:tt, $t:tt) => {
        if $other > <$t>::BITS.try_into().unwrap() {
            panic!("overflow");
        } else {
            let to_shift = BigInt::from_signed_bytes_le(&$self.0);
            let len = min(
                (<$t>::BITS / 8) as usize,
                to_shift.to_signed_bytes_le().len(),
                );
            let shift = $other;
            to_shift.shl(shift).to_signed_bytes_le()[..len]
                .try_into()
                .unwrap()
        }
    };
    (large.shr(builtin), $self:tt, $other:tt, $t:tt) => {
        if $other > <$t>::BITS.try_into().unwrap() {
            panic!("overflow");
        } else {
            BigInt::from_signed_bytes_le(&$self.0)
                .shr($other)
                .try_into()
                .unwrap()
        }
    };
    (large.shl(small), $self:tt, $other:tt, $t:tt) => {
        let to_shift = BigInt::from_signed_bytes_le(&$self.0);
        let len = min(
            (<$t>::BITS / 8) as usize,
            to_shift.to_signed_bytes_le().len(),
            );
        let shift = $other.0;
        to_shift.shl(shift).to_signed_bytes_le()[..len]
            .try_into()
            .unwrap()
    };
    (large.shr(small), $self:tt, $other:tt, $t:tt) => {
        BigInt::from_signed_bytes_le(&$self.0)
            .shr($other.0)
            .try_into()
            .unwrap()
    };
    (small.shl(small_signed), $self:tt, $other:tt, $t:tt) => {
        if $other.0 < 0 {
            $t($self.0.checked_shr(-$other.0 as u32).unwrap())
        } else {
            $t($self.0.checked_shl($other.0 as u32).unwrap())
        }
    };
    (small.shr(small_signed), $self:tt, $other:tt, $t:tt) => {
        if $other.0 < 0 {
            $t($self.0.checked_shl(-$other.0 as u32).unwrap())
        } else {
            $t($self.0.checked_shr($other.0 as u32).unwrap())
        }
    };
    (small.shl(small_unsigned), $self:tt, $other:tt, $t:tt) => {
        $t($self
            .0
            .checked_shl(u32::try_from($other.0).unwrap())
            .unwrap())
    };
    (small.shr(small_unsigned), $self:tt, $other:tt, $t:tt) => {
        $t($self
            .0
            .checked_shr(u32::try_from($other.0).unwrap())
            .unwrap())
    };
    (small.shl(builtin_signed), $self:tt, $other:tt, $t:tt) => {
        if $other < 0 {
            $t($self.0.checked_shr(-$other as u32).unwrap())
        } else {
            $t($self.0.checked_shl($other as u32).unwrap())
        }
    };
    (small.shr(builtin_signed), $self:tt, $other:tt, $t:tt) => {
        if $other < 0 {
            $t($self.0.checked_shl(-$other as u32).unwrap())
        } else {
            $t($self.0.checked_shr($other as u32).unwrap())
        }
    };
    (small.shl(builtin_unsigned), $self:tt, $other:tt, $t:tt) => {
        $t($self.0.checked_shl(u32::try_from($other).unwrap()).unwrap())
    };
    (small.shr(builtin_unsigned), $self:tt, $other:tt, $t:tt) => {
        $t($self.0.checked_shr(u32::try_from($other).unwrap()).unwrap())
    };
    (small.shl(large_unsigned), $self:tt, $other:tt, $t:tt) => {
        $t($self
            .0
            .checked_shl({
                if $other > u32::MAX.into() {
                    panic!("overflow")
                } else {
                    let other_le_bytes = $other.0[0..4].try_into().unwrap();
                    u32::from_le_bytes(other_le_bytes)
                }
            })
            .unwrap())
    };
    (small.shr(large_unsigned), $self:tt, $other:tt, $t:tt) => {
        $t($self
            .0
            .checked_shr({
                if $other > u32::MAX.into() {
                    panic!("overflow")
                } else {
                    let other_le_bytes = $other.0[0..4].try_into().unwrap();
                    u32::from_le_bytes(other_le_bytes)
                }
            })
            .unwrap())
    };
    (small.shl(large_signed), $self:tt, $other:tt, $t:tt) => {
        if $other > 0.into() {
            $t($self
                .0
                .checked_shl({
                    if $other > u32::MAX.into() {
                        panic!("overflow")
                    } else {
                        let other_le_bytes = $other.0[0..4].try_into().unwrap();
                        u32::from_le_bytes(other_le_bytes)
                    }
                })
                .unwrap())
        } else {
            $t($self
                .0
                .checked_shr({
                    if $other.abs() > u32::MAX.into() {
                        panic!("overflow")
                    } else {
                        let other_le_bytes = $other.abs().0[0..4].try_into().unwrap();
                        u32::from_le_bytes(other_le_bytes)
                    }
                })
                .unwrap())
        }
    };
    (small.shr(large_signed), $self:tt, $other:tt, $t:tt) => {
        if $other > 0.into() {
            $t($self
                .0
                .checked_shr({
                    if $other > u32::MAX.into() {
                        panic!("overflow")
                    } else {
                        let other_le_bytes = $other.0[0..4].try_into().unwrap();
                        u32::from_le_bytes(other_le_bytes)
                    }
                })
                .unwrap())
        } else {
            $t($self
                .0
                .checked_shl({
                    if $other.abs() > u32::MAX.into() {
                        panic!("overflow")
                    } else {
                        let other_le_bytes = $other.abs().0[0..4].try_into().unwrap();
                        u32::from_le_bytes(other_le_bytes)
                    }
                })
                .unwrap())
        }
    };
}

// large: U256, U384, U512, I256, I384, I512
// small: U8, U16, U32, U64, U128, I8, I16, I32, I64, I128
// builtin: u8, u16, u32, u64, u128, i8, i16, i32, i64, i128
macro_rules! shift_impl_all_large {
    ($($t:ty),*) => {
        $(
            sh_impl!{
                to_sh: $t,
                other: I256,
                other_var: other,
                self_var: self,
                shl_expr: sh!{large.shl(large_signed), self, other, $t},
                shr_expr: sh!{large.shr(large_signed), self, other, $t}
            }
            sh_impl!{
                to_sh: $t,
                other: I384,
                other_var: other,
                self_var: self,
                shl_expr: sh!{large.shl(large_signed), self, other, $t},
                shr_expr: sh!{large.shr(large_signed), self, other, $t}
            }
            sh_impl!{
                to_sh: $t,
                other: I512,
                other_var: other,
                self_var: self,
                shl_expr: sh!{large.shl(large_signed), self, other, $t},
                shr_expr: sh!{large.shr(large_signed), self, other, $t}
            }
            sh_impl!{
                to_sh: $t,
                other: U256,
                other_var: other,
                self_var: self,
                shl_expr: sh!{large.shl(large_unsigned), self, other, $t},
                shr_expr: sh!{large.shr(large_unsigned), self, other, $t}
            }
            sh_impl!{
                to_sh: $t,
                other: U384,
                other_var: other,
                self_var: self,
                shl_expr: sh!{large.shl(large_unsigned), self, other, $t},
                shr_expr: sh!{large.shr(large_unsigned), self, other, $t}
            }
            sh_impl!{
                to_sh: $t,
                other: U512,
                other_var: other,
                self_var: self,
                shl_expr: sh!{large.shl(large_unsigned), self, other, $t},
                shr_expr: sh!{large.shr(large_unsigned), self, other, $t}
            }
            sh_impl!{
                to_sh: $t,
                other: i16,
                other_var: other,
                self_var: self,
                shl_expr: sh!{large.shl(builtin), self, other, $t},
                shr_expr: sh!{large.shr(builtin), self, other, $t}
            }
            sh_impl!{
                to_sh: $t,
                other: i32,
                other_var: other,
                self_var: self,
                shl_expr: sh!{large.shl(builtin), self, other, $t},
                shr_expr: sh!{large.shr(builtin), self, other, $t}
            }
            sh_impl!{
                to_sh: $t,
                other: i64,
                other_var: other,
                self_var: self,
                shl_expr: sh!{large.shl(builtin), self, other, $t},
                shr_expr: sh!{large.shr(builtin), self, other, $t}
            }
            sh_impl!{
                to_sh: $t,
                other: i128,
                other_var: other,
                self_var: self,
                shl_expr: sh!{large.shl(builtin), self, other, $t},
                shr_expr: sh!{large.shr(builtin), self, other, $t}
            }
            sh_impl!{
                to_sh: $t,
                other: u16,
                other_var: other,
                self_var: self,
                shl_expr: sh!{large.shl(builtin), self, other, $t},
                shr_expr: sh!{large.shr(builtin), self, other, $t}
            }
            sh_impl!{
                to_sh: $t,
                other: u32,
                other_var: other,
                self_var: self,
                shl_expr: sh!{large.shl(builtin), self, other, $t},
                shr_expr: sh!{large.shr(builtin), self, other, $t}
            }
            sh_impl!{
                to_sh: $t,
                other: u64,
                other_var: other,
                self_var: self,
                shl_expr: sh!{large.shl(builtin), self, other, $t},
                shr_expr: sh!{large.shr(builtin), self, other, $t}
            }
            sh_impl!{
                to_sh: $t,
                other: u128,
                other_var: other,
                self_var: self,
                shl_expr: sh!{large.shl(builtin), self, other, $t},
                shr_expr: sh!{large.shr(builtin), self, other, $t}
            }
            sh_impl!{
                to_sh: $t,
                other: I16,
                other_var: other,
                self_var: self,
                shl_expr: sh!{large.shl(small), self, other, $t},
                shr_expr: sh!{large.shr(small), self, other, $t}
            }
            sh_impl!{
                to_sh: $t,
                other: I32,
                other_var: other,
                self_var: self,
                shl_expr: sh!{large.shl(small), self, other, $t},
                shr_expr: sh!{large.shr(small), self, other, $t}
            }
            sh_impl!{
                to_sh: $t,
                other: I64,
                other_var: other,
                self_var: self,
                shl_expr: sh!{large.shl(small), self, other, $t},
                shr_expr: sh!{large.shr(small), self, other, $t}
            }
            sh_impl!{
                to_sh: $t,
                other: I128,
                other_var: other,
                self_var: self,
                shl_expr: sh!{large.shl(small), self, other, $t},
                shr_expr: sh!{large.shr(small), self, other, $t}
            }
            sh_impl!{
                to_sh: $t,
                other: U16,
                other_var: other,
                self_var: self,
                shl_expr: sh!{large.shl(small), self, other, $t},
                shr_expr: sh!{large.shr(small), self, other, $t}
            }
            sh_impl!{
                to_sh: $t,
                other: U32,
                other_var: other,
                self_var: self,
                shl_expr: sh!{large.shl(small), self, other, $t},
                shr_expr: sh!{large.shr(small), self, other, $t}
            }
            sh_impl!{
                to_sh: $t,
                other: U64,
                other_var: other,
                self_var: self,
                shl_expr: sh!{large.shl(small), self, other, $t},
                shr_expr: sh!{large.shr(small), self, other, $t}
            }
            sh_impl!{
                to_sh: $t,
                other: U128,
                other_var: other,
                self_var: self,
                shl_expr: sh!{large.shl(small), self, other, $t},
                shr_expr: sh!{large.shr(small), self, other, $t}
            }
            )*
    };
}

macro_rules! shift_impl_all_small {
    ($($t:ty),*) => {
        $(
            sh_impl!{
                to_sh: $t,
                other: I8,
                other_var: other,
                self_var: self,
                shl_expr: sh!{small.shl(small_signed), self, other, $t},
                shr_expr: sh!{small.shr(small_signed), self, other, $t}
            }
            sh_impl!{
                to_sh: $t,
                other: I16,
                other_var: other,
                self_var: self,
                shl_expr: sh!{small.shl(small_signed), self, other, $t},
                shr_expr: sh!{small.shr(small_signed), self, other, $t}
            }
            sh_impl!{
                to_sh: $t,
                other: I32,
                other_var: other,
                self_var: self,
                shl_expr: sh!{small.shl(small_signed), self, other, $t},
                shr_expr: sh!{small.shr(small_signed), self, other, $t}
            }
            sh_impl!{
                to_sh: $t,
                other: I64,
                other_var: other,
                self_var: self,
                shl_expr: sh!{small.shl(small_signed), self, other, $t},
                shr_expr: sh!{small.shr(small_signed), self, other, $t}
            }
            sh_impl!{
                to_sh: $t,
                other: I128,
                other_var: other,
                self_var: self,
                shl_expr: sh!{small.shl(small_signed), self, other, $t},
                shr_expr: sh!{small.shr(small_signed), self, other, $t}
            }
            sh_impl!{
                to_sh: $t,
                other: U8,
                other_var: other,
                self_var: self,
                shl_expr: sh!{small.shl(small_unsigned), self, other, $t},
                shr_expr: sh!{small.shr(small_unsigned), self, other, $t}
            }
            sh_impl!{
                to_sh: $t,
                other: U16,
                other_var: other,
                self_var: self,
                shl_expr: sh!{small.shl(small_unsigned), self, other, $t},
                shr_expr: sh!{small.shr(small_unsigned), self, other, $t}
            }
            sh_impl!{
                to_sh: $t,
                other: U32,
                other_var: other,
                self_var: self,
                shl_expr: sh!{small.shl(small_unsigned), self, other, $t},
                shr_expr: sh!{small.shr(small_unsigned), self, other, $t}
            }
            sh_impl!{
                to_sh: $t,
                other: U64,
                other_var: other,
                self_var: self,
                shl_expr: sh!{small.shl(small_unsigned), self, other, $t},
                shr_expr: sh!{small.shr(small_unsigned), self, other, $t}
            }
            sh_impl!{
                to_sh: $t,
                other: U128,
                other_var: other,
                self_var: self,
                shl_expr: sh!{small.shl(small_unsigned), self, other, $t},
                shr_expr: sh!{small.shr(small_unsigned), self, other, $t}
            }
            sh_impl!{
                to_sh: $t,
                other: i8,
                other_var: other,
                self_var: self,
                shl_expr: sh!{small.shl(builtin_signed), self, other, $t},
                shr_expr: sh!{small.shr(builtin_signed), self, other, $t}
            }
            sh_impl!{
                to_sh: $t,
                other: i16,
                other_var: other,
                self_var: self,
                shl_expr: sh!{small.shl(builtin_signed), self, other, $t},
                shr_expr: sh!{small.shr(builtin_signed), self, other, $t}
            }
            sh_impl!{
                to_sh: $t,
                other: i32,
                other_var: other,
                self_var: self,
                shl_expr: sh!{small.shl(builtin_signed), self, other, $t},
                shr_expr: sh!{small.shr(builtin_signed), self, other, $t}
            }
            sh_impl!{
                to_sh: $t,
                other: i64,
                other_var: other,
                self_var: self,
                shl_expr: sh!{small.shl(builtin_signed), self, other, $t},
                shr_expr: sh!{small.shr(builtin_signed), self, other, $t}
            }
            sh_impl!{
                to_sh: $t,
                other: i128,
                other_var: other,
                self_var: self,
                shl_expr: sh!{small.shl(builtin_signed), self, other, $t},
                shr_expr: sh!{small.shr(builtin_signed), self, other, $t}
            }
            sh_impl!{
                to_sh: $t,
                other: u8,
                other_var: other,
                self_var: self,
                shl_expr: sh!{small.shl(builtin_unsigned), self, other, $t},
                shr_expr: sh!{small.shr(builtin_unsigned), self, other, $t}
            }
            sh_impl!{
                to_sh: $t,
                other: u16,
                other_var: other,
                self_var: self,
                shl_expr: sh!{small.shl(builtin_unsigned), self, other, $t},
                shr_expr: sh!{small.shr(builtin_unsigned), self, other, $t}
            }
            sh_impl!{
                to_sh: $t,
                other: u32,
                other_var: other,
                self_var: self,
                shl_expr: sh!{small.shl(builtin_unsigned), self, other, $t},
                shr_expr: sh!{small.shr(builtin_unsigned), self, other, $t}
            }
            sh_impl!{
                to_sh: $t,
                other: u64,
                other_var: other,
                self_var: self,
                shl_expr: sh!{small.shl(builtin_unsigned), self, other, $t},
                shr_expr: sh!{small.shr(builtin_unsigned), self, other, $t}
            }
            sh_impl!{
                to_sh: $t,
                other: u128,
                other_var: other,
                self_var: self,
                shl_expr: sh!{small.shl(builtin_unsigned), self, other, $t},
                shr_expr: sh!{small.shr(builtin_unsigned), self, other, $t}
            }
            sh_impl!{
                to_sh: $t,
                other: I256,
                other_var: other,
                self_var: self,
                shl_expr: sh!{small.shl(large_signed), self, other, $t},
                shr_expr: sh!{small.shr(large_signed), self, other, $t}
            }
            sh_impl!{
                to_sh: $t,
                other: I384,
                other_var: other,
                self_var: self,
                shl_expr: sh!{small.shl(large_signed), self, other, $t},
                shr_expr: sh!{small.shr(large_signed), self, other, $t}
            }
            sh_impl!{
                to_sh: $t,
                other: I512,
                other_var: other,
                self_var: self,
                shl_expr: sh!{small.shl(large_signed), self, other, $t},
                shr_expr: sh!{small.shr(large_signed), self, other, $t}
            }
            sh_impl!{
                to_sh: $t,
                other: U256,
                other_var: other,
                self_var: self,
                shl_expr: sh!{small.shl(large_unsigned), self, other, $t},
                shr_expr: sh!{small.shr(large_unsigned), self, other, $t}
            }
            sh_impl!{
                to_sh: $t,
                other: U384,
                other_var: other,
                self_var: self,
                shl_expr: sh!{small.shl(large_unsigned), self, other, $t},
                shr_expr: sh!{small.shr(large_unsigned), self, other, $t}
            }
            sh_impl!{
                to_sh: $t,
                other: U512,
                other_var: other,
                self_var: self,
                shl_expr: sh!{small.shl(large_unsigned), self, other, $t},
                shr_expr: sh!{small.shr(large_unsigned), self, other, $t}
            }
            )*
    };
}

shift_impl_all_large! {I256, I384, I512, U256, U384, U512}

shift_impl_all_small! {I8, I16, I32, I64, I128, U8, U16, U32, U64, U128}

macro_rules! other_expr {
    ($t:ty, $o:ty, $other:ident) => {
        paste! {
            if <$t>::BITS >= 256 {
                if <$o>::BITS >= 256 {
                    &BigInt::from_signed_bytes_le(&$other.0)
                } else {
                    &BigInt::from($other.0)
                }
            } else {
                if <$o>::BITS >= 256 {
                    BigInt::from_signed_bytes_le(&$other.0).try_into().unwrap()
                } else {
                    $other.0.try_into().unwrap()
                }
            }
        }
    }
}

macro_rules! self_expr {
    ($t:ty, $self:ident) => {
        if <$t>::BITS >= 256 {
            BigInt::from_signed_bytes_le(&$self.0)
        } else {
            $self.0
        }
    }
}

macro_rules! checked_impl_large {
    ($(($t:ty, $o:ty, $out:ty)),*) => {
        paste! {
            $(
                impl Add<$o> for $t {
                    type Output = $out;

                    #[inline]
                    fn add(self, other: $o) -> $out {
                        self_expr!($t, self).add(other_expr!($t, $o, other)).try_into().unwrap()
                    }
                }
                forward_ref_binop! { impl Add, add for $t, $o }

                impl AddAssign<$o> for $t {
                    #[inline]
                    fn add_assign(&mut self, other: $o) {
                        *self = *self + other;
                    }
                }
                forward_ref_op_assign! { impl AddAssign, add_assign for $t, $o }

                impl Sub<$o> for $t {
                    type Output = $out;

                    #[inline]
                    fn sub(self, other: $o) -> $out {
                        self_expr!($t, self).sub(other_expr!($t, $o, other)).try_into().unwrap()
                    }
                }
                forward_ref_binop! { impl Sub, sub for $t, $o }

                impl SubAssign<$o> for $t {
                    #[inline]
                    fn sub_assign(&mut self, other: $o) {
                        *self = *self - other;
                    }
                }
                forward_ref_op_assign! { impl SubAssign, sub_assign for $t, $o }

                impl Mul<$o> for $t {
                    type Output = $out;

                    #[inline]
                    fn mul(self, other: $o) -> $out {
                        self_expr!($t, self).mul(other_expr!($t, $o, other)).try_into().unwrap()
                    }
                }
                forward_ref_binop! { impl Mul, mul for $t, $o }

                impl MulAssign<$o> for $t {
                    #[inline]
                    fn mul_assign(&mut self, other: $o) {
                        *self = *self * other;
                    }
                }
                forward_ref_op_assign! { impl MulAssign, mul_assign for $t, $o }

                impl Div<$o> for $t {
                    type Output = $out;

                    #[inline]
                    fn div(self, other: $o) -> $out {
                        self_expr!($t, self).div(other_expr!($t, $o, other)).try_into().unwrap()
                    }
                }
                forward_ref_binop! { impl Div, div for $t, $o }

                impl DivAssign<$o> for $t {
                    #[inline]
                    fn div_assign(&mut self, other: $o) {
                        *self = *self / other;
                    }
                }
                forward_ref_op_assign! { impl DivAssign, div_assign for $t, $o }

                impl Rem<$o> for $t {
                    type Output = $out;

                    #[inline]
                    fn rem(self, other: $o) -> $out {
                        self_expr!($t, self).rem(other_expr!($t, $o, other)).try_into().unwrap()
                    }
                }
                forward_ref_binop! { impl Rem, rem for $t, $o }

                impl RemAssign<$o> for $t {
                    #[inline]
                    fn rem_assign(&mut self, other: $o) {
                        *self = *self % other;
                    }
                }
                forward_ref_op_assign! { impl RemAssign, rem_assign for $t, $o }


                impl BitXor<$o> for $t {
                    type Output = $out;

                    #[inline]
                    fn bitxor(self, other: $o) -> $out {
                        self_expr!($t, self).bitxor(other_expr!($t, $o, other)).try_into().unwrap()
                    }
                }
                forward_ref_binop! { impl BitXor, bitxor for $t, $o }

                impl BitXorAssign<$o> for $t {
                    #[inline]
                    fn bitxor_assign(&mut self, other: $o) {
                        *self = *self ^ other;
                    }
                }
                forward_ref_op_assign! { impl BitXorAssign, bitxor_assign for $t, $o }

                impl BitOr<$o> for $t {
                    type Output = $out;

                    #[inline]
                    fn bitor(self, other: $o) -> $out {
                        self_expr!($t, self).bitor(other_expr!($t, $o, other)).try_into().unwrap()
                    }
                }
                forward_ref_binop! { impl BitOr, bitor for $t, $o }

                impl BitOrAssign<$o> for $t {
                    #[inline]
                    fn bitor_assign(&mut self, other: $o) {
                        *self = *self | other;
                    }
                }
                forward_ref_op_assign! { impl BitOrAssign, bitor_assign for $t, $o }

                impl BitAnd<$o> for $t {
                    type Output = $out;

                    #[inline]
                    fn bitand(self, other: $o) -> $out {
                        self_expr!($t, self).bitand(other_expr!($t, $o, other)).try_into().unwrap()
                    }
                }
                forward_ref_binop! { impl BitAnd, bitand for $t, $o }

                impl BitAndAssign<$o> for $t {
                    #[inline]
                    fn bitand_assign(&mut self, other: $o) {
                        *self = *self & other;
                    }
                }
                forward_ref_op_assign! { impl BitAndAssign, bitand_assign for $t, $o }
                
                impl $t {
                    /// Raises self to the power of `exp`, using exponentiation by squaring.
                    ///
                    #[inline]
                    #[must_use = "this returns the result of the operation, \
                          without modifying the original"]
                    pub fn pow(self, exp: $o) -> $t {
                        BigInt::from_signed_bytes_le(&self.0).pow(exp).try_into().unwrap()
                    }

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
                    #[doc = concat!("use scrypto::math::" ,stringify!($t), ";")]
                    ///
                    /// let n: $t = $t(0x0123456789ABCDEF);
                    /// let m: $t = $t(-0x76543210FEDCBA99);
                    ///
                    /// assert_eq!(n.rotate_left(32), m);
                    /// ```
                    #[inline]
                    #[must_use = "this returns the result of the operation, \
                          without modifying the original"]
                    pub fn rotate_left(self, n: $o) -> $t {
                        let rot: u32 = n % Self::BITS;
                        let big: BigInt = BigInt::from_signed_bytes_le(&self.0);
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
                    #[doc = concat!("use scrypto::math::" ,stringify!($t), ";")]
                    ///
                    /// let n: $t = $t(0x0123456789ABCDEF);
                    /// let m: $t = $t(-0xFEDCBA987654322);
                    ///
                    /// assert_eq!(n.rotate_right(4), m);
                    /// ```
                    #[inline]
                    #[must_use = "this returns the result of the operation, \
                          without modifying the original"]
                    pub fn rotate_right(self, n: $o) -> $t {
                        let rot: u32 = n % Self::BITS;
                        let big: BigInt = BigInt::from_signed_bytes_le(&self.0);
                        let big_rot = big.clone().shr(rot);
                        big_rot.bitor(big.shl(Self::BITS - rot)).try_into().unwrap()
                    }
                }
                )*
        }
    };
}

macro_rules! checked_impl_small {
    ($(($t:ty, $o:ty, $out:ty)),*) => {
        paste! {
            $(
                impl Add<$o> for $t {
                    type Output = $out;

                    #[inline]
                    fn add(self, other: $o) -> $out {
                        self_expr!($t, self).checked_add(other_expr!($t, $o, other)).unwrap().try_into().unwrap()
                    }
                }
                forward_ref_binop! { impl Add, add for $t, $o }

                impl AddAssign<$o> for $t {
                    #[inline]
                    fn add_assign(&mut self, other: $o) {
                        *self = *self + other;
                    }
                }
                forward_ref_op_assign! { impl AddAssign, add_assign for $t, $o }

                impl Sub<$o> for $t {
                    type Output = $out;

                    #[inline]
                    fn sub(self, other: $o) -> $out {
                        self_expr!($t, self).checked_sub(other_expr!($t, $o, other)).unwrap().try_into().unwrap()
                    }
                }
                forward_ref_binop! { impl Sub, sub for $t, $o }

                impl SubAssign<$o> for $t {
                    #[inline]
                    fn sub_assign(&mut self, other: $o) {
                        *self = *self - other;
                    }
                }
                forward_ref_op_assign! { impl SubAssign, sub_assign for $t, $o }

                impl Mul<$o> for $t {
                    type Output = $out;

                    #[inline]
                    fn mul(self, other: $o) -> $out {
                        self_expr!($t, self).checked_mul(other_expr!($t, $o, other)).unwrap().try_into().unwrap()
                    }
                }
                forward_ref_binop! { impl Mul, mul for $t, $o }

                impl MulAssign<$o> for $t {
                    #[inline]
                    fn mul_assign(&mut self, other: $o) {
                        *self = *self * other;
                    }
                }
                forward_ref_op_assign! { impl MulAssign, mul_assign for $t, $o }

                impl Div<$o> for $t {
                    type Output = $out;

                    #[inline]
                    fn div(self, other: $o) -> $out {
                        self_expr!($t, self).checked_div(other_expr!($t, $o, other)).unwrap().try_into().unwrap()
                    }
                }
                forward_ref_binop! { impl Div, div for $t, $o }

                impl DivAssign<$o> for $t {
                    #[inline]
                    fn div_assign(&mut self, other: $o) {
                        *self = *self / other;
                    }
                }
                forward_ref_op_assign! { impl DivAssign, div_assign for $t, $o }

                impl Rem<$o> for $t {
                    type Output = $out;

                    #[inline]
                    fn rem(self, other: $o) -> $out {
                        self_expr!($t, self).rem(other_expr!($t, $o, other)).try_into().unwrap()
                    }
                }
                forward_ref_binop! { impl Rem, rem for $t, $o }

                impl RemAssign<$o> for $t {
                    #[inline]
                    fn rem_assign(&mut self, other: $o) {
                        *self = *self % other;
                    }
                }
                forward_ref_op_assign! { impl RemAssign, rem_assign for $t, $o }


                impl BitXor<$o> for $t {
                    type Output = $out;

                    #[inline]
                    fn bitxor(self, other: $o) -> $out {
                        self_expr!($t, self).bitxor(other_expr!($t, $o, other)).try_into().unwrap()
                    }
                }
                forward_ref_binop! { impl BitXor, bitxor for $t, $o }

                impl BitXorAssign<$o> for $t {
                    #[inline]
                    fn bitxor_assign(&mut self, other: $o) {
                        *self = *self ^ other;
                    }
                }
                forward_ref_op_assign! { impl BitXorAssign, bitxor_assign for $t, $o }

                impl BitOr<$o> for $t {
                    type Output = $out;

                    #[inline]
                    fn bitor(self, other: $o) -> $out {
                        self_expr!($t, self).bitor(other_expr!($t, $o, other)).try_into().unwrap()
                    }
                }
                forward_ref_binop! { impl BitOr, bitor for $t, $o }

                impl BitOrAssign<$o> for $t {
                    #[inline]
                    fn bitor_assign(&mut self, other: $o) {
                        *self = *self | other;
                    }
                }
                forward_ref_op_assign! { impl BitOrAssign, bitor_assign for $t, $o }

                impl BitAnd<$o> for $t {
                    type Output = $out;

                    #[inline]
                    fn bitand(self, other: $o) -> $out {
                        self_expr!($t, self).bitand(other_expr!($t, $o, other)).try_into().unwrap()
                    }
                }
                forward_ref_binop! { impl BitAnd, bitand for $t, $o }

                impl BitAndAssign<$o> for $t {
                    #[inline]
                    fn bitand_assign(&mut self, other: $o) {
                        *self = *self & other;
                    }
                }
                forward_ref_op_assign! { impl BitAndAssign, bitand_assign for $t, $o }

                impl $t {
                    /// Raises self to the power of `exp`, using exponentiation by squaring.
                    ///
                    #[inline]
                    #[must_use = "this returns the result of the operation, \
                          without modifying the original"]
                    pub fn pow(self, exp: $o) -> $t {
                        $t(self.0.checked_pow(exp.try_into().unwrap()).unwrap())
                    }

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
                    #[doc = concat!("use scrypto::math::" ,stringify!($t), ";")]
                    ///
                    /// let n: $t = $t(0x0123456789ABCDEF);
                    /// let m: $t = $t(-0x76543210FEDCBA99);
                    ///
                    /// assert_eq!(n.rotate_left(32), m);
                    /// ```
                    #[inline]
                    #[must_use = "this returns the result of the operation, \
                          without modifying the original"]
                    pub const fn rotate_left(self, n: $o) -> $t {
                        $t(self.0.rotate_left(n))
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
                    #[doc = concat!("use scrypto::math::" ,stringify!($t), ";")]
                    ///
                    /// let n: $t = $t(0x0123456789ABCDEF);
                    /// let m: $t = $t(-0xFEDCBA987654322);
                    ///
                    /// assert_eq!(n.rotate_right(4), m);
                    /// ```
                    #[inline]
                    #[must_use = "this returns the result of the operation, \
                          without modifying the original"]
                    pub const fn rotate_right(self, n: u32) -> $t {
                        $t(self.0.rotate_right(n))
                    }
                }

                )*
        }
    };
}

checked_impl_large! {
    //(self, other, output)
    (U256, u8, U256), (U256, u16, U256), (U256, u32, U256), (U256, u64, U256), 
(U256, u128, U256), (U256, i8, I256), (U256, i16, I256), (U256, i32, I256), 
(U256, i64, I256), (U256, i128, I256),

(U384, u8, U384), (U384, u16, U384), (U384, u32, U384), (U384, u64, U384), 
(U384, u128, U384), (U384, i8, I384), (U384, i16, I384), (U384, i32, I384), 
(U384, i64, I384), (U384, i128, I384),

(U512, u8, U512), (U512, u16, U512), (U512, u32, U512), (U512, u64, U512), 
(U512, u128, U512), (U512, i8, I512), (U512, i16, I512), (U512, i32, I512), 
(U512, i64, I512), (U512, i128, I512),

(U256, U8, U256), (U256, U16, U256), (U256, U32, U256), (U256, U64, U256), 
(U256, U128, U256), (U256, U256, U256), (U256, U384, U384), (U256, U512, U512), 
(U256, I8, I256), (U256, I16, I256), (U256, I32, I256), (U256, I64, I256), 
(U256, I128, I256), (U256, I256, I256), (U256, I384, I384),

(U384, U8, U384), (U384, U16, U384), (U384, U32, U384), (U384, U64, U384), 
(U384, U128, U384), (U384, U256, U256), (U384, U384, U384), (U384, U512, U512), 
(U384, I8, I384), (U384, I16, I384), (U384, I32, I384), (U384, I64, I384), 
(U384, I128, I384), (U384, I256, I256), (U384, I384, I384), (U384, I512, I512),

(U512, U8, U512), (U512, U16, U512), (U512, U32, U512), (U512, U64, U512), 
(U512, U128, U512), (U512, U256, U256), (U512, U384, U384), (U512, U512, U512), 
(U512, I8, I512), (U512, I16, I512), (U512, I32, I512), (U512, I64, I512), 
(U512, I128, I512), (U512, I256, I256), (U512, I384, I384), (U512, I512, I512),

(I256, u8, I256), (I256, u16, I256), (I256, u32, I256), (I256, u64, I256), 
(I256, u128, I256), (I256, i8, I256), (I256, i16, I256), (I256, i32, I256), 
(I256, i64, I256), (I256, i128, I256),

(I384, u8, I384), (I384, u16, I384), (I384, u32, I384), (I384, u64, I384), 
(I384, u128, I384), (I384, i8, I384), (I384, i16, I384), (I384, i32, I384), 
(I384, i64, I384), (I384, i128, I384),

(I512, u8, I512), (I512, u16, I512), (I512, u32, I512), (I512, u64, I512), 
(I512, u128, I512), (I512, i8, I512), (I512, i16, I512), (I512, i32, I512), 
(I512, i64, I512), (I512, i128, I512),

    (I256, U8, I256), (I256, U16, I256), (I256, U32, I256), (I256, U64, I256), 
(I256, U128, I256), (I256, U256, I256), (I256, U384, I384), (I256, U512, I512), 
(I256, I8, I256), (I256, I16, I256), (I256, I32, I256), (I256, I64, I256), 
(I256, I128, I256), (I256, I256, I256), (I256, I384, I384),

    (I384, U8, I384), (I384, U16, I384), (I384, U32, I384), (I384, U64, I384), 
(I384, U128, I384), (I384, U256, I256), (I384, U384, I384), (I384, U512, I512), 
(I384, I8, I384), (I384, I16, I384), (I384, I32, I384), (I384, I64, I384), 
(I384, I128, I384), (I384, I256, I256), (I384, I384, I384), (I384, I512, I512),

    (I512, U8, I512), (I512, U16, I512), (I512, U32, I512), (I512, U64, I512), 
(I512, U128, I512), (I512, U256, I256), (I512, U384, I384), (I512, U512, I512), 
(I512, I8, I512), (I512, I16, I512), (I512, I32, I512), (I512, I64, I512), 
(I512, I128, I512), (I512, I256, I256), (I512, I384, I384), (I512, I512, I512) }

checked_impl_small! { 
    //(self, other, output)
    (U8, u8, U8), (U8, u16, U16), (U8, u32, U32), (U8, u64, U64), (U8, u128, U128), 
(U8, i8, I8), (U8, i16, I16), (U8, i32, I32), (U8, i64, I64), (U8, i128, I128),

    (U16, u8, U16), (U16, u16, U16), (U16, u32, U32), (U16, u64, U64), (U16, u128, U128), 
(U16, i8, I16), (U16, i16, I16), (U16, i32, I32), (U16, i64, I64), (U16, i128, I128),

    (U32, u8, U32), (U32, u16, U32), (U32, u32, U32), (U32, u64, U64), (U32, u128, U128), 
(U32, i8, I32), (U32, i16, I32), (U32, i32, I32), (U32, i64, I64), (U32, i128, I128),

    (U64, u8, U64), (U64, u16, U64), (U64, u32, U64), (U64, u64, U64), (U64, u128, U128), 
(U64, i8, I64), (U64, i16, I64), (U64, i32, I64), (U64, i64, I64), (U64, i128, I128),

    (U128, u8, U128), (U128, u16, U128), (U128, u32, U128), (U128, u64, U128), 
(U128, u128, U128), (U128, i8, I128), (U128, i16, I128), (U128, i32, I128), 
(U128, i64, I128), (U128, i128, I128),

    (U16, U8, U16), (U16, U16, U16), (U16, U32, U32), (U16, U64, U64), (U16, U128, U128), 
(U16, U256, U256), (U16, U384, U384), (U16, U512, U512), (U16, I8, I16), (U16, I16, I16), 
(U16, I32, I32), (U16, I64, I64), (U16, I128, I128), (U16, I256, I256), (U16, I384, I384), 
(U16, I512, I512),

    (U32, U8, U32), (U32, U16, U32), (U32, U32, U32), (U32, U64, U64), (U32, U128, U128), 
(U32, U256, U256), (U32, U384, U384), (U32, U512, U512), (U32, I8, I32), (U32, I16, I32), 
(U32, I32, I32), (U32, I64, I64), (U32, I128, I128), (U32, I256, I256), (U32, I384, I384), 
(U32, I512, I512),

    (U64, U8, U64), (U64, U16, U64), (U64, U32, U64), (U64, U64, U64), (U64, U128, U128), 
(U64, U256, U256), (U64, U384, U384), (U64, U512, U512), (U64, I8, I64), (U64, I16, I64), 
(U64, I32, I64), (U64, I64, I64), (U64, I128, I128), (U64, I256, I256), (U64, I384, I384), 
(U64, I512, I512),

    (U128, U8, U128), (U128, U16, U128), (U128, U32, U128), (U128, U64, U128), 
(U128, U128, U128), (U128, U256, U256), (U128, U384, U384), (U128, U512, U512), 
(U128, I8, I128), (U128, I16, I128), (U128, I32, I128), (U128, I64, I128), 
(U128, I128, I128), (U128, I256, I256), (U128, I384, I384), (U128, I512, I512),

    (I8, u8, I8), (I8, u16, I16), (I8, u32, I32), (I8, u64, I64), (I8, u128, I128), 
(I8, i8, I8), (I8, i16, I16), (I8, i32, I32), (I8, i64, I64), (I8, i128, I128),

    (I16, u8, I16), (I16, u16, I16), (I16, u32, I32), (I16, u64, I64), (I16, u128, I128), 
(I16, i8, I16), (I16, i16, I16), (I16, i32, I32), (I16, i64, I64), (I16, i128, I128),

    (I32, u8, I32), (I32, u16, I32), (I32, u32, I32), (I32, u64, I64), (I32, u128, I128), 
(I32, i8, I32), (I32, i16, I32), (I32, i32, I32), (I32, i64, I64), (I32, i128, I128),

    (I64, u8, I64), (I64, u16, I64), (I64, u32, I64), (I64, u64, I64), (I64, u128, I128), 
(I64, i8, I64), (I64, i16, I64), (I64, i32, I64), (I64, i64, I64), (I64, i128, I128),

    (I128, u8, I128), (I128, u16, I128), (I128, u32, I128), (I128, u64, I128), 
(I128, u128, I128), (I128, i8, I128), (I128, i16, I128), (I128, i32, I128), 
(I128, i64, I128), (I128, i128, I128),

    (I8, U8, I8), (I8, U16, I16), (I8, U32, I32), (I8, U64, I64), (I8, U128, I128), 
(I8, U256, I256), (I8, U384, I384), (I8, U512, I512), (I8, I8, I8), (I8, I16, I16), 
(I8, I32, I32), (I8, I64, I64), (I8, I128, I128), (I8, I256, I256), (I8, I384, I384), 
(I8, I512, I512),

    (I16, U8, I16), (I16, U16, I16), (I16, U32, I32), (I16, U64, I64), (I16, U128, I128), 
(I16, U256, I256), (I16, U384, I384), (I16, U512, I512), (I16, I8, I16), (I16, I16, I16), 
(I16, I32, I32), (I16, I64, I64), (I16, I128, I128), (I16, I256, I256), (I16, I384, I384), 
(I16, I512, I512),

    (I32, U8, I32), (I32, U16, I32), (I32, U32, I32), (I32, U64, I64), (I32, U128, I128), 
(I32, U256, I256), (I32, U384, I384), (I32, U512, I512), (I32, I8, I32), (I32, I16, I32), 
(I32, I32, I32), (I32, I64, I64), (I32, I128, I128), (I32, I256, I256), (I32, I384, I384), 
(I32, I512, I512),

    (I64, U8, I64), (I64, U16, I64), (I64, U32, I64), (I64, U64, I64), (I64, U128, I128), 
(I64, U256, I256), (I64, U384, I384), (I64, U512, I512), (I64, I8, I64), (I64, I16, I64), 
(I64, I32, I64), (I64, I64, I64), (I64, I128, I128), (I64, I256, I256), (I64, I384, I384), 
(I64, I512, I512),

    (I128, U8, I128), (I128, U16, I128), (I128, U32, I128), (I128, U64, I128), 
(I128, U128, I128), (I128, U256, I256), (I128, U384, I384), (I128, U512, I512), 
(I128, I8, I128), (I128, I16, I128), (I128, I32, I128), (I128, I64, I128), 
(I128, I128, I128), (I128, I256, I256), (I128, I384, I384), (I128, I512, I512) }

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
    (type_id: $t:ident, bytes_len: $bytes_len:literal, MIN: $min: expr, MAX: $max: expr, fn pow(self, exp: $o:ident)) => {
        paste! {
            impl $t {
                /// Returns the smallest value that can be represented by this integer type.
                ///
                /// # Examples
                ///
                /// Basic usage:
                ///
                /// ```
                #[doc = concat!("use scrypto::math::" ,stringify!($t), ";")]
                ///
                #[doc = concat!("assert_eq!(<$t>::MIN, $t(", stringify!($bytes_len), "::MIN));")]
                /// ```
                pub const MIN: Self = $min;

                /// Returns the largest value that can be represented by this integer type.
                ///
                /// # Examples
                ///
                /// Basic usage:
                ///
                /// ```
                #[doc = concat!("use scrypto::math::" ,stringify!($t), ";")]
                ///
                #[doc = concat!("assert_eq!(<$t>::MAX, $t(", stringify!($t), "::MAX));")]
                /// ```
                pub const MAX: Self = $max;

                /// Returns the size of this integer type in bits.
                ///
                /// # Examples
                ///
                /// Basic usage:
                ///
                /// ```
                #[doc = concat!("use scrypto::math::" ,stringify!($t), ";")]
                ///
                #[doc = concat!("assert_eq!(<$t>::BITS, ", stringify!($t), "::BITS);")]
                /// ```
                pub const BITS: u32 = $bytes_len * 8;

                /// Returns the number of ones in the binary representation of `self`.
                ///
                /// # Examples
                ///
                /// Basic usage:
                ///
                /// ```
                #[doc = concat!("use scrypto::math::" ,stringify!($t), ";")]
                ///
                #[doc = concat!("let n = $t(0b01001100", stringify!($t), ");")]
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
                #[doc = concat!("use scrypto::math::" ,stringify!($t), ";")]
                ///
                #[doc = concat!("assert_eq!($t(!0", stringify!($t), ").count_zeros(), 0);")]
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
                #[doc = concat!("use scrypto::math::" ,stringify!($t), ";")]
                ///
                #[doc = concat!("let n = $t(0b0101000", stringify!($t), ");")]
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
                #[doc = concat!("use scrypto::math::" ,stringify!($t), ";")]
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
                #[doc = concat!("use scrypto::math::" ,stringify!($t), ";")]
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
                #[doc = concat!("use scrypto::math::" ,stringify!($t), ";")]
                ///
                #[doc = concat!("let n = $t(0x1A", stringify!($t), ");")]
                ///
                /// if cfg!(target_endian = "big") {
                #[doc = concat!("    assert_eq!(<$t>::from_be(n), n)")]
                /// } else {
                #[doc = concat!("    assert_eq!(<$t>::from_be(n), n.swap_bytes())")]
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
                #[doc = concat!("use scrypto::math::" ,stringify!($t), ";")]
                ///
                #[doc = concat!("let n = $t(0x1A", stringify!($t), ");")]
                ///
                /// if cfg!(target_endian = "little") {
                #[doc = concat!("    assert_eq!(<$t>::from_le(n), n)")]
                /// } else {
                #[doc = concat!("    assert_eq!(<$t>::from_le(n), n.swap_bytes())")]
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
                #[doc = concat!("use scrypto::math::" ,stringify!($t), ";")]
                ///
                #[doc = concat!("let n = $t(0x1A", stringify!($t), ");")]
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
                #[doc = concat!("use scrypto::math::" ,stringify!($t), ";")]
                ///
                #[doc = concat!("let n = $t(0x1A", stringify!($t), ");")]
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
        }
    }
}

macro_rules! checked_unsigned_large {
    ($($t:ident, $o:ident, $bytes_len:literal),*) => {
        $(
            checked_int_impl_large! {
                type_id: $t,
                bytes_len: $bytes_len,
                MIN: $t([0u8; $bytes_len]),
                MAX: $t([0xffu8; $bytes_len]),
                fn pow(self, exp: $o)
            }
        )*
    }
}

macro_rules! checked_signed_large {
    ( $($t:ident, $o:ident, $bytes_len:literal),* ) => {
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
                },
                fn pow(self, exp: $o)
            }
        )*
    }
}

macro_rules! checked_all {
    ($($t:ident),*) => {
        $(

            checked_signed_large! {
                I256, $t, 32,
                I384, $t, 48,
                I512, $t, 64
            }

            checked_unsigned_large! {
                U256, $t, 32,
                U384, $t, 48,
                U512, $t, 64
            }
        )*
    };
}
checked_all! { i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, I8, I16, I32, I64, I128, I256, I384, I512, U8, U16, U32, U64, U128, U256, U384, U512 }

macro_rules! checked_int_impl_small {
    ($(($t:ident, $o:ident)),*) => {$(
        paste! {
            impl $t {
                /// Returns the smallest value that can be represented by this integer type.
                ///
                /// # Examples
                ///
                /// Basic usage:
                ///
                /// ```
                #[doc = concat!("use scrypto::math::" ,stringify!($t), ";")]
                ///
                #[doc = concat!("assert_eq!(<$t>::MIN, $t(", stringify!($n), "::MIN));")]
                /// ```
                pub const MIN: Self = Self([<$t:lower>]::MIN);

                /// Returns the largest value that can be represented by this integer type.
                ///
                /// # Examples
                ///
                /// Basic usage:
                ///
                /// ```
                #[doc = concat!("use scrypto::math::" ,stringify!($t), ";")]
                ///
                #[doc = concat!("assert_eq!(<$t>::MAX, $t(", stringify!($i), "::MAX));")]
                /// ```
                pub const MAX: Self = Self([<$t:lower>]::MAX);

                /// Returns the size of this integer type in bits.
                ///
                /// # Examples
                ///
                /// Basic usage:
                ///
                /// ```
                #[doc = concat!("use scrypto::math::" ,stringify!($t), ";")]
                ///
                #[doc = concat!("assert_eq!(<$t>::BITS, ", stringify!($i), "::BITS);")]
                /// ```
                pub const BITS: u32 = [<$t:lower>]::BITS;

                /// Returns the number of ones in the binary representation of `self`.
                ///
                /// # Examples
                ///
                /// Basic usage:
                ///
                /// ```
                #[doc = concat!("use scrypto::math::" ,stringify!($t), ";")]
                ///
                #[doc = concat!("let n = $t(0b01001100", stringify!($i), ");")]
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
                #[doc = concat!("use scrypto::math::" ,stringify!($t), ";")]
                ///
                #[doc = concat!("assert_eq!($t(!0", stringify!($i), ").count_zeros(), 0);")]
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
                #[doc = concat!("use scrypto::math::" ,stringify!($t), ";")]
                ///
                #[doc = concat!("let n = $t(0b0101000", stringify!($i), ");")]
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
                #[doc = concat!("use scrypto::math::" ,stringify!($t), ";")]
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
                #[doc = concat!("use scrypto::math::" ,stringify!($t), ";")]
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
                #[doc = concat!("use scrypto::math::" ,stringify!($t), ";")]
                ///
                #[doc = concat!("let n = $t(0x1A", stringify!($i), ");")]
                ///
                /// if cfg!(target_endian = "big") {
                #[doc = concat!("    assert_eq!(<$t>::from_be(n), n)")]
                /// } else {
                #[doc = concat!("    assert_eq!(<$t>::from_be(n), n.swap_bytes())")]
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
                #[doc = concat!("use scrypto::math::" ,stringify!($t), ";")]
                ///
                #[doc = concat!("let n = $t(0x1A", stringify!($i), ");")]
                ///
                /// if cfg!(target_endian = "little") {
                #[doc = concat!("    assert_eq!(<$t>::from_le(n), n)")]
                /// } else {
                #[doc = concat!("    assert_eq!(<$t>::from_le(n), n.swap_bytes())")]
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
                #[doc = concat!("use scrypto::math::" ,stringify!($t), ";")]
                ///
                #[doc = concat!("let n = $t(0x1A", stringify!($i), ");")]
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
                #[doc = concat!("use scrypto::math::" ,stringify!($t), ";")]
                ///
                #[doc = concat!("let n = $t(0x1A", stringify!($i), ");")]
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
        }
    )*}
}

macro_rules! checked_impl_all {
    ($($t:ident),*) => {$(
        checked_int_impl_small! { (I8, $t), (I16, $t), (I32, $t), (I64, $t), (I128, $t) }
        checked_int_impl_small! { (U8, $t), (U16, $t), (U32, $t), (U64, $t), (U128, $t) }
    )*}
}

checked_impl_all! { i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, I8, I16, I32, I64, I128, I256, I384, I512, U8, U16, U32, U64, U128, U256, U384, U512 }

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
                    #[doc = concat!("use scrypto::math::" ,stringify!($t), ";")]
                    ///
                    #[doc = concat!("assert_eq!($t(10", stringify!($t:lower), ").signum(), $t(1));")]
                    #[doc = concat!("assert_eq!($t(0", stringify!($t:lower), ").signum(), $t(0));")]
                    #[doc = concat!("assert_eq!($t(-10", stringify!($t:lower), ").signum(), $t(-1));")]
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
                    #[doc = concat!("use scrypto::math::" ,stringify!($t), ";")]
                    ///
                    #[doc = concat!("assert!($t(10", stringify!($t:lower), ").is_positive());")]
                    #[doc = concat!("assert!(!$t(-10", stringify!($t:lower), ").is_positive());")]
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
                    #[doc = concat!("use scrypto::math::" ,stringify!($t), ";")]
                    ///
                    #[doc = concat!("assert!($t(-10", stringify!($t:lower), ").is_negative());")]
                    #[doc = concat!("assert!(!$t(10", stringify!($t:lower), ").is_negative());")]
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
            BigInt::from_signed_bytes_le(&self.0)
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

macro_rules! impl_bigint_to_large {
    ($($t:ty),*) => {
        $(
            paste! {
                fn [<bigint_to_$t:lower>](b: BigInt) -> Result<$t, ParseBigIntError> {
                    let bytes = b.to_signed_bytes_le();
                    const T_BYTES: usize = (<$t>::BITS / 8 ) as usize;
                    if bytes.len() > T_BYTES {
                        return Err(ParseBigIntError::Overflow);
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

macro_rules! impl_bigint_to_small {
    ($($t:ty),*) => {
        $(
            paste! {
                fn [<bigint_to_$t:lower>](b: BigInt) -> Result<$t, ParseBigIntError> {
                    match b.[<to_$t:lower>]() {
                        Some(v) => Ok($t(v)),
                        None => Err(ParseBigIntError::Overflow),
                    }
                }
            }
        )*
    }
}

impl_bigint_to_large! { I256, I384, I512, U256, U384, U512 }
impl_bigint_to_small! { I8, I16, I32, I64, I128, U8, U16, U32, U64, U128 }

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

macro_rules! try_from_big_int_to_signed {
    ($($t:ident),*) => {
        $(
            paste! {
                impl TryFrom<BigInt> for $t {
                    type Error = ParseBigIntError;
                    fn try_from(val: BigInt) -> Result<$t, ParseBigIntError> {
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
                    type Error = ParseBigIntError;

                    fn try_from(val: BigInt) -> Result<Self, Self::Error>  {
                        if val.is_negative() {
                            return Err(ParseBigIntError::NegativeToUnsigned);
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

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_impl {
        ($(($I:ident, $i: ident)),*) => ($(

                paste::item! {
                    #[test]
                    #[should_panic]
                    fn [<test_add_overflow$i>]() {
                        let a = $I(<$i>::MAX) + $I(1 as $i); // panics on overflow
                        assert_eq!(a , $I(<$i>::MAX));
                    }

                    #[test]
                    #[should_panic]
                    fn  [<test_sub_overflow$i>]() {
                        let _ = $I(<$i>::MIN) - $I(1 as $i); // panics because of overflow
                    }

                    #[test]
                    #[should_panic]
                    fn  [<test_mul_overflow$i>]() {
                        let _ = $I(<$i>::MAX) * $I(2 as $i); // panics because of overflow
                    }

                    #[test]
                    #[should_panic]
                    fn  [<test_div_overflow$i>]() {
                        let _ = $I(<$i>::MIN) / $I(0 as $i); // panics because of division by zero
                    }

                    #[test]
                    #[should_panic]
                    fn  [<test_rem_overflow$i>]() {
                        let _ = $I(<$i>::MIN) % $I(0); // panics because of division by zero
                    }

                    #[test]
                    #[should_panic]
                    fn  [<test_shl_overflow$i>]() {
                        let _ = $I(<$i>::MAX) << ((<$i>::BITS + 1) as $i);  // panics because of overflow
                    }

                    #[test]
                    #[should_panic]
                    fn  [<test_shr_overflow$i>]() {
                        let _ = $I(<$i>::MIN) >> (($i::BITS + 1) as $i);  // panics because of overflow
                    }

                    #[test]
                    #[should_panic]
                    fn  [<test_shl_overflow_neg$i>]() {
                        let _ = $I(<$i>::MIN) << (($i::BITS + 1) as $i);  // panics because of overflow
                    }

                    #[test]
                    #[should_panic]
                    fn  [<test_shr_overflow_neg$i>]() {
                        let _ = $I(<$i>::MIN) >> (($i::BITS + 1) as $i);  // panics because of overflow
                    }

                    #[test]
                    #[should_panic]
                    fn  [<test_pow_overflow$i>]() {
                        let _ = $I(<$i>::MAX).pow(2u32);          // panics because of overflow
                    }
                }
        )*)
    }
    test_impl! { (I8, i8), (I16, i16), (I32, i32), (I64, i64), (I128, i128), (U8, u8), (U16, u16), (U32, u32), (U64, u64), (U128, u128) }
}

// TODO: from string
// TODO: test write
// TODO: documentationpart update
// TODO: sbor integration
