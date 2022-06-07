//! Definitions of safe integers and uints.

use core::fmt;
use core::ops::{Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign};
use core::ops::{BitXor, BitXorAssign, Div, DivAssign};
use core::ops::{Mul, MulAssign, Neg, Not, Rem, RemAssign};
use core::ops::{Shl, ShlAssign, Shr, ShrAssign, Sub, SubAssign};
use forward_ref::*;
use num_bigint::BigInt;
use num_traits::{FromPrimitive, Num, Signed, ToPrimitive, Zero, Zero};
use num_traits::{Signed, Zero};
use paste::paste;

macro_rules! types {

    ($(
            {
                type: $t:ident ( $wrap:ty ),
                default: $default:expr,
                self: $self:ident,
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

            impl std::default::Default for $t {
                fn default() -> Self {
                    Self($default)
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

            )*
        }
    }
}

types! {
    {
        type: U8(u8),
        default: 0,
        self: self,
        self_expr: self.0
    },
    {
        type: U16(u16),
        default: 0,
        self: self,
        self_expr: self.0
    },
    {
        type: U32(u32),
        default: 0,
        self: self,
        self_expr: self.0
    },
    {
        type: U64(u64),
        default: 0,
        self: self,
        self_expr: self.0
    },
    {
        type: U128(u128),
        default: 0,
        self: self,
        self_expr: self.0
    },
    {
        type: U256([u8; 32]),
        default: [0u8; 32],
        self: self,
        self_expr: BigInt::from_signed_bytes_le(&self.0)
    },
    {
        type: U384([u8; 48]),
        default: [0u8; 48],
        self: self,
        self_expr: BigInt::from_signed_bytes_le(&self.0)
    },
    {
        type: U512([u8; 64]),
        default: [0u8; 64],
        self: self,
        self_expr: BigInt::from_signed_bytes_le(&self.0)
    },
    {
        type: I8(i8),
        default: 0,
        self: self,
        self_expr: self.0
    },
    {
        type: I16(i16),
        default: 0,
        self: self,
        self_expr: self.0
    },
    {
        type: I32(i32),
        default: 0,
        self: self,
        self_expr: self.0
    },
    {
        type: I64(i64),
        default: 0,
        self: self,
        self_expr: self.0
    },
    {
        type: I128(i128),
        default: 0,
        self: self,
        self_expr: self.0
    },
    {
        type: I256([u8; 32]),
        default: [0u8; 32],
        self: self,
        self_expr: BigInt::from_signed_bytes_le(&self.0)
    },
    {
        type: I384([u8; 48]),
        default: [0u8; 48],
        self: self,
        self_expr: BigInt::from_signed_bytes_le(&self.0)
    },
    {
        type: I512([u8; 64]),
        default: [0u8; 64],
        self: self,
        self_expr: BigInt::from_signed_bytes_le(&self.0)
    }
}

#[derive(Debug)]
pub enum ParseBigIntError {
    NegativeToUnsigned,
    Overflow,
}

macro_rules! impl_bigint_to_large {
    ($($t:ty),*) => {
        $(
            paste! {
                fn [<bigint_to_$t:lower>](b: BigInt) -> Result<$t, ParseBigIntError> {
                    let bytes = b.to_signed_bytes_le();
                    const t_bytes: usize = (<$t>::BITS / 8 ) as usize;
                    if bytes.len() > t_bytes {
                        return Err(ParseBigIntError::Overflow);
                    }
                    let mut buf = if b.is_negative() {
                        [255u8; t_bytes]
                    } else {
                        [0u8; t_bytes]
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
            let shift = BigInt::from_signed_bytes_le(&$other.0).to_i128().unwrap();
            to_shift.shl(shift).try_into().unwrap()
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
        let to_shift = BigInt::from_signed_bytes_le(&$other.0).to_i128().unwrap();
        // FIXME: line below might create a BigInt whose size might become larger than self.
        // Check this!!!
        BigInt::from_signed_bytes_le(&$self.0)
            .shl(to_shift)
            .try_into()
            .unwrap()
    }};
    (large.shr(large_unsigned), $self:tt, $other:tt, $t:tt) => {{
        let to_shift = BigInt::from_signed_bytes_le(&$other.0).to_i128().unwrap();
        // FIXME: line below might create a BigInt whose size might become larger than self.
        // Check this!!!
        BigInt::from_signed_bytes_le(&$self.0)
            .shr(to_shift)
            .try_into()
            .unwrap()
    }};
    (large.shl(builtin), $self:tt, $other:tt, $t:tt) => {
        if $other > <$t>::BITS.try_into().unwrap() {
            panic!("overflow");
        } else {
            // FIXME: line below might create a BigInt whose size might become larger than self.
            // Check this!!!
            BigInt::from_signed_bytes_le(&$self.0)
                .shl($other)
                .try_into()
                .unwrap()
        }
    };
    (large.shr(builtin), $self:tt, $other:tt, $t:tt) => {
        if $other > <$t>::BITS.try_into().unwrap() {
            panic!("overflow");
        } else {
            // FIXME: line below might create a BigInt whose size might become larger than self.
            // Check this!!!
            BigInt::from_signed_bytes_le(&$self.0)
                .shr($other)
                .try_into()
                .unwrap()
        }
    };
    (large.shl(small), $self:tt, $other:tt, $t:tt) => {
        // FIXME: line below might create a BigInt whose size might become larger than self.
        // Check this!!!
        BigInt::from_signed_bytes_le(&$self.0)
            .shl($other.0)
            .try_into()
            .unwrap()
    };
    (large.shr(small), $self:tt, $other:tt, $t:tt) => {
        // FIXME: line below might create a BigInt whose size might become larger than self.
        // Check this!!!
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
macro_rules! shift_impl_all {
    (large: $($t:ty),*) => {
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
    (small: $($t:ty),*) => {
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

shift_impl_all! {large: I256, I384, I512, U256, U384, U512}

shift_impl_all! {small: I8, I16, I32, I64, I128, U8, U16, U32, U64, U128}

macro_rules! checked_impl {
    ($((impl Op<$o:ty> for $t:ty { fn op($self:ident, $other:ident: $oo:ty) -> $ot:ty { $sexpr:expr=>op($oexpr:expr)}}), checked_prefix: $checked:expr),*) => {
        paste! {
            $(
                impl Add<$o> for $t {
                    type Output = $t;

                    #[inline]
                    fn add($self, $other: $o) -> $t {
                        if $checked {
                            $sexpr.checked_add($oexpr).unwrap().try_into().unwrap()
                        } else {
                            $sexpr.add($oexpr).try_into().unwrap()
                        }
                    }
                }
                forward_ref_binop! { impl Add, add for $t, $o }

                impl AddAssign<$o> for $t {
                    #[inline]
                    fn add_assign(&mut $self, $other: $o) {
                        *$self = *$self + $other;
                    }
                }
                forward_ref_op_assign! { impl AddAssign, add_assign for $t, $o }

                impl Sub<$o> for $t {
                    type Output = $t;

                    #[inline]
                    fn sub($self, $other: $o) -> $t {
                        $sexpr.[<$checked:lower sub>]($oexpr).try_into().unwrap()
                    }
                }
                forward_ref_binop! { impl Sub, sub for $t, $o }

                impl SubAssign<$o> for $t {
                    #[inline]
                    fn sub_assign(&mut $self, $other: $o) {
                        *$self = *$self - $other;
                    }
                }
                forward_ref_op_assign! { impl SubAssign, sub_assign for $t, $o }

                impl Mul<$o> for $t {
                    type Output = $t;

                    #[inline]
                    fn mul($self, $other: $o) -> $t {
                        $sexpr.[<$checked:lower mul>]($oexpr).try_into().unwrap()
                    }
                }
                forward_ref_binop! { impl Mul, mul for $t, $o }

                impl MulAssign<$o> for $t {
                    #[inline]
                    fn mul_assign(&mut $self, $other: $o) {
                        *$self = *$self * $other;
                    }
                }
                forward_ref_op_assign! { impl MulAssign, mul_assign for $t, $o }

                impl Div<$o> for $t {
                    type Output = $t;

                    #[inline]
                    fn div($self, $other: $o) -> $t {
                        $sexpr.[<$checked:lower div>]($oexpr).try_into().unwrap()
                    }
                }
                forward_ref_binop! { impl Div, div for $t, $o }

                impl DivAssign<$o> for $t {
                    #[inline]
                    fn div_assign(&mut $self, $other: $o) {
                        *$self = *$self / $other;
                    }
                }
                forward_ref_op_assign! { impl DivAssign, div_assign for $t, $o }

                impl Rem<$o> for $t {
                    type Output = $t;

                    #[inline]
                    fn rem($self, $other: $o) -> $t {
                        $sexpr.rem($oexpr).try_into().unwrap()
                    }
                }
                forward_ref_binop! { impl Rem, rem for $t, $o }

                impl RemAssign<$o> for $t {
                    #[inline]
                    fn rem_assign(&mut $self, $other: $o) {
                        *$self = *$self % $other;
                    }
                }
                forward_ref_op_assign! { impl RemAssign, rem_assign for $t, $o }


                impl BitXor<$o> for $t {
                    type Output = $t;

                    #[inline]
                    fn bitxor($self, $other: $o) -> $t {
                        $sexpr.bitxor($oexpr).try_into().unwrap()
                    }
                }
                forward_ref_binop! { impl BitXor, bitxor for $t, $o }

                impl BitXorAssign<$o> for $t {
                    #[inline]
                    fn bitxor_assign(&mut $self, $other: $o) {
                        *$self = *$self ^ $other;
                    }
                }
                forward_ref_op_assign! { impl BitXorAssign, bitxor_assign for $t, $o }

                impl BitOr<$o> for $t {
                    type Output = $t;

                    #[inline]
                    fn bitor($self, $other: $o) -> $t {
                        $sexpr.bitor($oexpr).try_into().unwrap()
                    }
                }
                forward_ref_binop! { impl BitOr, bitor for $t, $o }

                impl BitOrAssign<$o> for $t {
                    #[inline]
                    fn bitor_assign(&mut $self, $other: $o) {
                        *$self = *$self | $other;
                    }
                }
                forward_ref_op_assign! { impl BitOrAssign, bitor_assign for $t, $o }

                impl BitAnd<$o> for $t {
                    type Output = $t;

                    #[inline]
                    fn bitand($self, $other: $o) -> $t {
                        $sexpr.bitand($oexpr).try_into().unwrap()
                    }
                }
                forward_ref_binop! { impl BitAnd, bitand for $t, $o }

                impl BitAndAssign<$o> for $t {
                    #[inline]
                    fn bitand_assign(&mut $self, $other: $o) {
                        *$self = *$self & $other;
                    }
                }
                forward_ref_op_assign! { impl BitAndAssign, bitand_assign for $t, $o }
                )*
        }
    };
}

// TODO: impl checked_impl for U256, U384, U512, I256, I384, I512

macro_rules! checked_int_ops {
    (large: $($t:ident),*) => {
        $(
            checked_impl! {
                (impl Op<u8> for $t { fn op(self, other: u8) -> $t {BigInt::from_signed_bytes_le(&self.0)=>op(&BigInt::from(other))}}), checked_prefix: false,
                (impl Op<u16> for $t { fn op(self, other: u16) -> $t {BigInt::from_signed_bytes_le(&self.0)=>op(&BigInt::from(other))}}), checked_prefix: false,
                (impl Op<u32> for $t { fn op(self, other: u32) -> $t {BigInt::from_signed_bytes_le(&self.0)=>op(&BigInt::from(other))}}), checked_prefix: false,
                (impl Op<u64> for $t { fn op(self, other: u64) -> $t {BigInt::from_signed_bytes_le(&self.0)=>op(&BigInt::from(other))}}), checked_prefix: false,
                (impl Op<u128> for $t { fn op(self, other: u128) -> $t {BigInt::from_signed_bytes_le(&self.0)=>op(&BigInt::from(other))}}), checked_prefix: false,

                (impl Op<i8> for $t { fn op(self, other: i8) -> $t {BigInt::from_signed_bytes_le(&self.0)=>op(&BigInt::from(other))}}), checked_prefix: false,
                (impl Op<i16> for $t { fn op(self, other: i16) -> $t {BigInt::from_signed_bytes_le(&self.0)=>op(&BigInt::from(other))}}), checked_prefix: false,
                (impl Op<i32> for $t { fn op(self, other: i32) -> $t {BigInt::from_signed_bytes_le(&self.0)=>op(&BigInt::from(other))}}), checked_prefix: false,
                (impl Op<i64> for $t { fn op(self, other: i64) -> $t {BigInt::from_signed_bytes_le(&self.0)=>op(&BigInt::from(other))}}), checked_prefix: false,
                (impl Op<i128> for $t { fn op(self, other: i128) -> $t {BigInt::from_signed_bytes_le(&self.0)=>op(&BigInt::from(other))}}), checked_prefix: false,

                (impl Op<U8> for $t { fn op(self, other: U8) -> $t {BigInt::from_signed_bytes_le(&self.0)=>op(&BigInt::from(other.0))}}), checked_prefix: false,
                (impl Op<U16> for $t { fn op(self, other: U16) -> $t {BigInt::from_signed_bytes_le(&self.0)=>op(&BigInt::from(other.0))}}), checked_prefix: false,
                (impl Op<U32> for $t { fn op(self, other: U32) -> $t {BigInt::from_signed_bytes_le(&self.0)=>op(&BigInt::from(other.0))}}), checked_prefix: false,
                (impl Op<U64> for $t { fn op(self, other: U64) -> $t {BigInt::from_signed_bytes_le(&self.0)=>op(&BigInt::from(other.0))}}), checked_prefix: false,
                (impl Op<U128> for $t { fn op(self, other: U128) -> $t {BigInt::from_signed_bytes_le(&self.0)=>op(&BigInt::from(other.0))}}), checked_prefix: false,
                (impl Op<U256> for $t { fn op(self, other: U256) -> $t {BigInt::from_signed_bytes_le(&self.0)=>op(&BigInt::from_signed_bytes_le(&other.0))}}), checked_prefix: false,
                (impl Op<U384> for $t { fn op(self, other: U384) -> $t {BigInt::from_signed_bytes_le(&self.0)=>op(&BigInt::from_signed_bytes_le(&other.0))}}), checked_prefix: false,
                (impl Op<U512> for $t { fn op(self, other: U512) -> $t {BigInt::from_signed_bytes_le(&self.0)=>op(&BigInt::from_signed_bytes_le(&other.0))}}), checked_prefix: false,

                (impl Op<I8> for $t { fn op(self, other: I8) -> $t {BigInt::from_signed_bytes_le(&self.0)=>op(&BigInt::from(other.0))}}), checked_prefix: false,
                (impl Op<I16> for $t { fn op(self, other: I16) -> $t {BigInt::from_signed_bytes_le(&self.0)=>op(&BigInt::from(other.0))}}), checked_prefix: false,
                (impl Op<I32> for $t { fn op(self, other: I32) -> $t {BigInt::from_signed_bytes_le(&self.0)=>op(&BigInt::from(other.0))}}), checked_prefix: false,
                (impl Op<I64> for $t { fn op(self, other: I64) -> $t {BigInt::from_signed_bytes_le(&self.0)=>op(&BigInt::from(other.0))}}), checked_prefix: false,
                (impl Op<I128> for $t { fn op(self, other: I128) -> $t {BigInt::from_signed_bytes_le(&self.0)=>op(&BigInt::from(other.0))}}), checked_prefix: false,
                (impl Op<I256> for $t { fn op(self, other: I256) -> $t {BigInt::from_signed_bytes_le(&self.0)=>op(&BigInt::from_signed_bytes_le(&other.0))}}), checked_prefix: false,
                (impl Op<I384> for $t { fn op(self, other: I384) -> $t {BigInt::from_signed_bytes_le(&self.0)=>op(&BigInt::from_signed_bytes_le(&other.0))}}), checked_prefix: false,
                (impl Op<I512> for $t { fn op(self, other: I512) -> $t {BigInt::from_signed_bytes_le(&self.0)=>op(&BigInt::from_signed_bytes_le(&other.0))}}), checked_prefix: false
            }
        )*
    };
    (small: $($t:ident),*) => {
        $(
            checked_impl! {
                (impl Op<u8> for $t { fn op(self, other: u8) -> $t {self.0=>op(other.try_into().unwrap())}}), checked_prefix: true,
                (impl Op<u16> for $t { fn op(self, other: u16) -> $t {self.0=>op(other.try_into().unwrap())}}), checked_prefix: true,
                (impl Op<u32> for $t { fn op(self, other: u32) -> $t {self.0=>op(other.try_into().unwrap())}}), checked_prefix: true,
                (impl Op<u64> for $t { fn op(self, other: u64) -> $t {self.0=>op(other.try_into().unwrap())}}), checked_prefix: true,
                (impl Op<u128> for $t { fn op(self, other: u128) -> $t {self.0=>op(other.try_into().unwrap())}}), checked_prefix: true,

                (impl Op<i8> for $t { fn op(self, other: i8) -> $t {self.0=>op(other.try_into().unwrap())}}), checked_prefix: true,
                (impl Op<i16> for $t { fn op(self, other: i16) -> $t {self.0=>op(other.try_into().unwrap())}}), checked_prefix: true,
                (impl Op<i32> for $t { fn op(self, other: i32) -> $t {self.0=>op(other.try_into().unwrap())}}), checked_prefix: true,
                (impl Op<i64> for $t { fn op(self, other: i64) -> $t {self.0=>op(other.try_into().unwrap())}}), checked_prefix: true,
                (impl Op<i128> for $t { fn op(self, other: i128) -> $t {self.0=>op(other.try_into().unwrap())}}), checked_prefix: true,

                (impl Op<U8> for $t { fn op(self, other: U8) -> $t {self.0=>op(other.0.try_into().unwrap())}}), checked_prefix: true,
                (impl Op<U16> for $t { fn op(self, other: U16) -> $t {self.0=>op(other.0.try_into().unwrap())}}), checked_prefix: true,
                (impl Op<U32> for $t { fn op(self, other: U32) -> $t {self.0=>op(other.0.try_into().unwrap())}}), checked_prefix: true,
                (impl Op<U64> for $t { fn op(self, other: U64) -> $t {self.0=>op(other.0.try_into().unwrap())}}), checked_prefix: true,
                (impl Op<U128> for $t { fn op(self, other: U128) -> $t {self.0=>op(other.0.try_into().unwrap())}}), checked_prefix: true,
                (impl Op<U256> for $t { fn op(self, other: U256) -> $t {self.0=>op(BigInt::from_signed_bytes_le(&other.0))}}), checked_prefix: true,
                (impl Op<U384> for $t { fn op(self, other: U384) -> $t {self.0=>op(BigInt::from_signed_bytes_le(&other.0))}}), checked_prefix: true,
                (impl Op<U512> for $t { fn op(self, other: U512) -> $t {self.0=>op(BigInt::from_signed_bytes_le(&other.0))}}), checked_prefix: true,

                (impl Op<I8> for $t { fn op(self, other: I8) -> $t {self.0=>op(other.0.try_into().unwrap())}}), checked_prefix: true,
                (impl Op<I16> for $t { fn op(self, other: I16) -> $t {self.0=>op(other.0.try_into().unwrap())}}), checked_prefix: true,
                (impl Op<I32> for $t { fn op(self, other: I32) -> $t {self.0=>op(other.0.try_into().unwrap())}}), checked_prefix: true,
                (impl Op<I64> for $t { fn op(self, other: I64) -> $t {self.0=>op(other.0.try_into().unwrap())}}), checked_prefix: true,
                (impl Op<I128> for $t { fn op(self, other: I128) -> $t {self.0=>op(other.0.try_into().unwrap())}}), checked_prefix: true,
                (impl Op<I256> for $t { fn op(self, other: I256) -> $t {self.0=>op(BigInt::from_signed_bytes_le(&other.0))}}), checked_prefix: true,
                (impl Op<I384> for $t { fn op(self, other: I384) -> $t {self.0=>op(BigInt::from_signed_bytes_le(&other.0))}}), checked_prefix: true,
                (impl Op<I512> for $t { fn op(self, other: I512) -> $t {self.0=>op(BigInt::from_signed_bytes_le(&other.0))}}), checked_prefix: true
            }
        )*
    }
}

checked_int_ops! {large: I256, U256, I384, U384, I512, U512 }
checked_int_ops! {small: I8, I16, I32, I64, I128, U8, U16, U32, U64, U128}

macro_rules! checked_impl_not {
    ($($i:ident),*) => {
        $(
            impl Not for $i {
                type Output = $i;

                #[inline]
                fn not(self) -> $i {
                    BigInt::from_signed_bytes_le(&self.0).not().try_into().unwrap()
                }
            }
            forward_ref_unop! { impl Not, not for $i }
        )*
    }
}

checked_impl_not! {  I256, U256, I384, U384, I512, U512 }

macro_rules! checked_impl_neg {
    ($($i:ident),*) => {
        $(
            impl Neg for $i {
                type Output = Self;
                #[inline]
                fn neg(self) -> Self {
                    BigInt::from_signed_bytes_le(&self.0).neg().try_into().unwrap()
                }
            }
            forward_ref_unop! { impl Neg, neg for $i }
        )*
    }
}
checked_impl_neg! { I256, I384, I512 }

macro_rules! checked_int_impl {
    (type_id: $i:ident, bytes_len: $bytes_len:literal, MIN: $min: expr, MAX: $max: expr) => {
        paste! {
            impl $i {
                /// Returns the smallest value that can be represented by this integer type.
                ///
                /// # Examples
                ///
                /// Basic usage:
                ///
                /// ```
                #[doc = concat!("use scrypto::math::" ,stringify!($i), ";")]
                ///
                #[doc = concat!("assert_eq!(<$i>::MIN, $i(", stringify!($bytes_len), "::MIN));")]
                /// ```
                pub const MIN: Self = $min;

                /// Returns the largest value that can be represented by this integer type.
                ///
                /// # Examples
                ///
                /// Basic usage:
                ///
                /// ```
                #[doc = concat!("use scrypto::math::" ,stringify!($i), ";")]
                ///
                #[doc = concat!("assert_eq!(<$i>::MAX, $i(", stringify!($i), "::MAX));")]
                /// ```
                pub const MAX: Self = $max;

                /// Returns the size of this integer type in bits.
                ///
                /// # Examples
                ///
                /// Basic usage:
                ///
                /// ```
                #[doc = concat!("use scrypto::math::" ,stringify!($i), ";")]
                ///
                #[doc = concat!("assert_eq!(<$i>::BITS, ", stringify!($i), "::BITS);")]
                /// ```
                pub const BITS: u32 = $bytes_len * 8;

                /// Returns the number of ones in the binary representation of `self`.
                ///
                /// # Examples
                ///
                /// Basic usage:
                ///
                /// ```
                #[doc = concat!("use scrypto::math::" ,stringify!($i), ";")]
                ///
                #[doc = concat!("let n = $i(0b01001100", stringify!($i), ");")]
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
                #[doc = concat!("use scrypto::math::" ,stringify!($i), ";")]
                ///
                #[doc = concat!("assert_eq!($i(!0", stringify!($i), ").count_zeros(), 0);")]
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
                #[doc = concat!("use scrypto::math::" ,stringify!($i), ";")]
                ///
                #[doc = concat!("let n = $i(0b0101000", stringify!($i), ");")]
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
                #[doc = concat!("use scrypto::math::" ,stringify!($i), ";")]
                ///
                /// let n: $i = $i(0x0123456789ABCDEF);
                /// let m: $i = $i(-0x76543210FEDCBA99);
                ///
                /// assert_eq!(n.rotate_left(32), m);
                /// ```
                #[inline]
                #[must_use = "this returns the result of the operation, \
                          without modifying the original"]
                pub fn rotate_left(self, n: u32) -> Self {
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
                #[doc = concat!("use scrypto::math::" ,stringify!($i), ";")]
                ///
                /// let n: $i = $i(0x0123456789ABCDEF);
                /// let m: $i = $i(-0xFEDCBA987654322);
                ///
                /// assert_eq!(n.rotate_right(4), m);
                /// ```
                #[inline]
                #[must_use = "this returns the result of the operation, \
                          without modifying the original"]
                pub fn rotate_right(self, n: u32) -> Self {
                    let rot: u32 = n % Self::BITS;
                    let big: BigInt = BigInt::from_signed_bytes_le(&self.0);
                    let big_rot = big.clone().shr(rot);
                    big_rot.bitor(big.shl(Self::BITS - rot)).try_into().unwrap()
                }

                /// Reverses the byte order of the integer.
                ///
                /// # Examples
                ///
                /// Basic usage:
                ///
                /// ```
                #[doc = concat!("use scrypto::math::" ,stringify!($i), ";")]
                ///
                /// let n: $i = $i(0b0000000_01010101);
                /// assert_eq!(n, $i(85));
                ///
                /// let m = n.swap_bytes();
                ///
                /// assert_eq!(m, $i(0b01010101_00000000));
                /// assert_eq!(m, $i(21760));
                /// ```
                #[inline]
                #[must_use = "this returns the result of the operation, \
                          without modifying the original"]
                pub fn swap_bytes(self) -> Self {
                    $i(self.0.into_iter().rev().collect::<Vec<u8>>().try_into().unwrap())
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
                #[doc = concat!("use scrypto::math::" ,stringify!($i), ";")]
                ///
                /// let n = $i(0b0000000_01010101i16);
                /// assert_eq!(n, $i(85));
                ///
                /// let m = n.reverse_bits();
                ///
                /// assert_eq!(m.0 as u16, 0b10101010_00000000);
                /// assert_eq!(m, $i(-22016));
                /// ```
                #[must_use = "this returns the result of the operation, \
                          without modifying the original"]
                #[inline]
                pub fn reverse_bits(self) -> Self {
                    $i(self.0.into_iter().rev().map(|x| x.reverse_bits()).collect::<Vec<u8>>().try_into().unwrap())
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
                #[doc = concat!("use scrypto::math::" ,stringify!($i), ";")]
                ///
                #[doc = concat!("let n = $i(0x1A", stringify!($i), ");")]
                ///
                /// if cfg!(target_endian = "big") {
                #[doc = concat!("    assert_eq!(<$i>::from_be(n), n)")]
                /// } else {
                #[doc = concat!("    assert_eq!(<$i>::from_be(n), n.swap_bytes())")]
                /// }
                /// ```
                #[inline]
                #[must_use]
                pub fn from_be(x: Self) -> Self {
                    if cfg!(target_endian = "big") {
                        $i(x.0)
                    } else {
                        $i(x.0.into_iter().rev().collect::<Vec<u8>>().try_into().unwrap())
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
                #[doc = concat!("use scrypto::math::" ,stringify!($i), ";")]
                ///
                #[doc = concat!("let n = $i(0x1A", stringify!($i), ");")]
                ///
                /// if cfg!(target_endian = "little") {
                #[doc = concat!("    assert_eq!(<$i>::from_le(n), n)")]
                /// } else {
                #[doc = concat!("    assert_eq!(<$i>::from_le(n), n.swap_bytes())")]
                /// }
                /// ```
                #[inline]
                #[must_use]
                pub fn from_le(x: Self) -> Self {
                    if cfg!(target_endian = "little") {
                        $i(x.0)
                    } else {
                        $i(x.0.into_iter().rev().collect::<Vec<u8>>().try_into().unwrap())
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
                #[doc = concat!("use scrypto::math::" ,stringify!($i), ";")]
                ///
                #[doc = concat!("let n = $i(0x1A", stringify!($i), ");")]
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
                        $i(self.0)
                    } else {
                        $i(self.0.into_iter().rev().collect::<Vec<u8>>().try_into().unwrap())
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
                #[doc = concat!("use scrypto::math::" ,stringify!($i), ";")]
                ///
                #[doc = concat!("let n = $i(0x1A", stringify!($i), ");")]
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
                        $i(self.0)
                    } else {
                        $i(self.0.into_iter().rev().collect::<Vec<u8>>().try_into().unwrap())
                    }
                }

                /// Raises self to the power of `exp`, using exponentiation by squaring.
                ///
                #[inline]
                #[must_use = "this returns the result of the operation, \
                          without modifying the original"]
                pub fn pow(self, exp: u32) -> Self {
                    BigInt::from_signed_bytes_le(&self.0).pow(exp).try_into().unwrap()
                }
            }
        }
    }
}

macro_rules! checked_unsigned {
    ($($t:ident, $bytes_len:literal),*) => {
        $(
            checked_int_impl! {
                type_id: $t,
                bytes_len: $bytes_len,
                MIN: $t([0u8; $bytes_len]),
                MAX: $t([0xffu8; $bytes_len])
            }
        )*
    }
}

macro_rules! checked_signed {
    ( $($t:ident, $bytes_len:literal),* ) => {
        $(
            checked_int_impl! {
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

checked_signed! {
    I256, 32,
    I384, 48,
    I512, 64
}

checked_unsigned! {
    U256, 32,
    U384, 48,
    U512, 64
}

macro_rules! leading_zeros {
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

macro_rules! checked_int_impl_signed {
    ($($t:ident),*) => ($(
        paste! {
            impl $t {

                leading_zeros!();

                /// Computes the absolute value of `self`, with overflow causing panic.
                ///
                /// The only case where such overflow can occur is when one takes the absolute value of the negative
                /// minimal value for the type this is a positive value that is too large to represent in the type. In
                /// such a case, this function panics.
                ///
                #[inline]
                #[must_use = "this returns the result of the operation, \
                          without modifying the original"]
                pub fn abs(self) -> $t {
                    BigInt::from_signed_bytes_le(&self.0).abs().try_into().unwrap()
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
                pub fn signum(self) -> $t {
                    BigInt::from_signed_bytes_le(&self.0).signum().try_into().unwrap()
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
                pub fn is_positive(self) -> bool {
                    BigInt::from_signed_bytes_le(&self.0).is_positive().try_into().unwrap()
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
                pub fn is_negative(self) -> bool {
                   self.0.to_vec().into_iter().nth(self.0.len() - 1).unwrap() & 0x80 == 1
                }
            }
        }
    )*)
}

checked_int_impl_signed! { I256, I384, I512 }

macro_rules! checked_int_impl_unsigned {
    ($($t:ty),*) => ($(
        impl $t {
            leading_zeros!();

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

checked_int_impl_unsigned! { U256, U384, U512 }

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

macro_rules! from_bigint_to_signed {
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

macro_rules! from_bigint_to_unsigned {
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
    ($($t:ident, $bytes_len:literal),*) => {
        $(
            paste! {
                impl From<[u8; $bytes_len]> for $t {
                    fn from(val: [u8; $bytes_len]) -> Self {
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

macro_rules! try_from {
    ($($t:ident, $bytes_len:literal),*) => {
        $(
            impl TryFrom<&[u8]> for $t {
                type Error = ParseSliceError;
                fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
                    if bytes.len() != $bytes_len {
                        Err(ParseSliceError::InvalidLength)
                    } else {
                        Ok(Self(bytes.try_into().unwrap()))
                    }
                }
            }

            impl TryFrom<Vec<u8>> for $t {
                type Error = ParseSliceError;
                fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
                    if bytes.len() != $bytes_len {
                        Err(ParseSliceError::InvalidLength)
                    } else {
                        Ok(Self(bytes.try_into().unwrap()))
                    }
                }
            }
        )*
    };
}
macro_rules! from_int_type {
    ($($t:ident),*) => {
        $(
            from_int!{($t, u8), ($t, u16), ($t, u32), ($t, u64), ($t, u128)}
            from_int!{($t, i8), ($t, i16), ($t, i32), ($t, i64), ($t, i128)}
        )*
    };
}

macro_rules! from_uint_type {
    ($($t:ident),*) => {
        $(
            from_int!{($t, u8), ($t, u16), ($t, u32), ($t, u64), ($t, u128)}
        )*
    };
}

from_int_type! { I256, I384, I512 }
from_uint_type! { U256, U384, U512 }
from_bigint_to_signed! { I256, I384, I512 }
from_bigint_to_unsigned! { U256, U384, U512 }
from_array! { U256, 32, U384, 48, U512, 64, I256, 32, I384, 48, I512, 64 }
try_from! { U256, 32, U384, 48, U512, 64, I256, 32, I384, 48, I512, 64 }

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

// TODO: implement from
// TODO: documentationpart update
