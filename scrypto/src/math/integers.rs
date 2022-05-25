//! Definitions of safe integers and uints.


use core::fmt;
use core::ops::{Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign};
use core::ops::{BitXor, BitXorAssign, Div, DivAssign};
use core::ops::{Mul, MulAssign, Neg, Not, Rem, RemAssign};
use core::ops::{Shl, ShlAssign, Shr, ShrAssign, Sub, SubAssign};
use forward_ref::*;
use paste::paste;
use num_bigint::BigInt;


macro_rules! types {
    ($(($I:ident, $i: ty)),*) => {

        $(
            /// Provides safe integer arithmetic.
            ///
            /// Operations like `+`, '-', '*', or '/' sometimes produce overflow 
            /// which is detected and results in a panic, instead of silently
            /// wrapping around.
            ///
            /// Integer arithmetic can be achieved either through methods like
            #[doc = concat!("/// `checked_add`, or through the ", stringify!($I) , "type, which ensures all") ]
            /// standard arithmetic operations on the underlying value to have 
            /// checked semantics.
            ///
            /// The underlying value can be retrieved through the `.0` index of the
            #[doc = concat!("/// `", stringify!($I), "` tuple.")]
            ///
            /// # Layout
            ///
            #[doc = concat!("/// `", stringify!($I), "` will have the same methods and traits as")]
            /// the built-in counterpart.
            #[derive(Clone , Copy , Default , Eq , Hash , Ord , PartialEq , PartialOrd)]
            #[repr(transparent)]
            pub struct $I(pub $i);
        )*
    }
}

types! { (I8, i8), (I16, i16), (I32, i32), (I64, i64), (I128, i128), (U8, u8), (U16, u16), (U32, u32), (U64, u64), (U128, u128) }
 
macro_rules! types_large {

    ($(($I:ident, $b: literal)),*) => {
        
        $(
            /// Provides safe integer arithmetic.
            ///
            /// Operations like `+`, '-', '*', or '/' sometimes produce overflow 
            /// which is detected and results in a panic, instead of silently
            /// wrapping around.
            ///
            /// Integer arithmetic can be achieved either through methods like
            #[doc = concat!("/// `checked_add`, or through the ", stringify!($I) , "type, which ensures all") ]
            /// standard arithmetic operations on the underlying value to have 
            /// checked semantics.
            ///
            /// The underlying value can be retrieved through the `.0` index of the
            #[doc = concat!("/// `", stringify!($I), "` tuple.")]
            ///
            /// # Layout
            ///
            #[doc = concat!("/// `", stringify!($I), "` will have the same methods and traits as")]
            /// the built-in counterpart.
            #[derive(Clone , Copy , Eq , Hash , Ord , PartialEq , PartialOrd)]
            #[repr(transparent)]
            pub struct $I(pub [u8; $b/8]);

            impl std::default::Default for $I {
                fn default() -> Self {
                    Self {0: [0; $b/8]}
                }
            }
        )*
    }
}

types_large! { (I256, 256), (I384, 384), (I512, 512), (U256, 256), (U384, 384), (U512, 512) }

macro_rules! impl_i {
    ($($t:ty)*) => {
        $(
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

            impl fmt::Binary for $t {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    self.0.fmt(f)
                }
            }

            impl fmt::Octal for $t {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    self.0.fmt(f)
                }
            }

            impl fmt::LowerHex for $t {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    self.0.fmt(f)
                }
            }

            impl fmt::UpperHex for $t {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    self.0.fmt(f)
                }
            }
        )*
    }
}

impl_i! { I8 I16 I32 I64 I128 U8 U16 U32 U64 U128 }

macro_rules! impl_i_large {
    ($($t:ty)*) => {
        $(
            impl fmt::Debug for $t {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    self.0.fmt(f)
                }
            }

            impl fmt::Display for $t {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    BigInt::from_signed_bytes_le(&self.0).fmt(f)
                }
            }

            impl fmt::Binary for $t {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    BigInt::from_signed_bytes_le(&self.0).fmt(f)
                }
            }

            impl fmt::Octal for $t {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    BigInt::from_signed_bytes_le(&self.0).fmt(f)
                }
            }

            impl fmt::LowerHex for $t {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    BigInt::from_signed_bytes_le(&self.0).fmt(f)
                }
            }

            impl fmt::UpperHex for $t {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    BigInt::from_signed_bytes_le(&self.0).fmt(f)
                }
            }
        )*
    }
}

impl_i_large! { I256 I384 I512 U256 U384 U512 }

#[allow(unused_macros)]
macro_rules! sh_impl_signed {
    ($t:ident, $f:ident) => {
        impl Shl<$f> for $t {
            type Output = $t;

            #[inline]
            fn shl(self, other: $f) -> $t {
                if other.0 < 0 {
                    $t(self.0.checked_shr(-other.0 as u32).unwrap())
                } else {
                    $t(self.0.checked_shl(other.0 as u32).unwrap())
                }
            }
        }
        forward_ref_binop! { impl Shl, shl for $t, $f }

        impl ShlAssign<$f> for $t {
            #[inline]
            fn shl_assign(&mut self, other: $f) {
                *self = *self << other;
            }
        }
        forward_ref_op_assign! { impl ShlAssign, shl_assign for $t, $f }

        impl Shr<$f> for $t {
            type Output = $t;

            #[inline]
            fn shr(self, other: $f) -> $t {
                if other.0 < 0 {
                    $t(self.0.checked_shl(-other.0 as u32).unwrap())
                } else {
                    $t(self.0.checked_shr(other.0 as u32).unwrap())
                }
            }
        }
        forward_ref_binop! { impl Shr, shr for $t, $f }

        impl ShrAssign<$f> for $t {
            #[inline]
            fn shr_assign(&mut self, other: $f) {
                *self = *self >> other;
            }
        }
        forward_ref_op_assign! { impl ShrAssign, shr_assign for $t, $f }
    };
}

macro_rules! sh_impl_unsigned {
    ($t:ident, $f:ident) => {
        impl Shl<$f> for $t {
            type Output = $t;

            #[inline]
            fn shl(self, other: $f) -> $t {
                $t(self.0.checked_shl(other.0 as u32).unwrap())
            }
        }
        forward_ref_binop! { impl Shl, shl for $t, $f }

        impl ShlAssign<$f> for $t {
            #[inline]
            fn shl_assign(&mut self, other: $f) {
                *self = *self << other;
            }
        }
        forward_ref_op_assign! { impl ShlAssign, shl_assign for $t, $f }

        impl Shr<$f> for $t {
            type Output = $t;

            #[inline]
            fn shr(self, other: $f) -> $t {
                $t(self.0.checked_shr(other.0 as u32).unwrap())
            }
        }
        forward_ref_binop! { impl Shr, shr for $t, $f }

        impl ShrAssign<$f> for $t {
            #[inline]
            fn shr_assign(&mut self, other: $f) {
                *self = *self >> other;
            }
        }
        forward_ref_op_assign! { impl ShrAssign, shr_assign for $t, $f }
    };
}

#[allow(unused_macros)]
macro_rules! sh_impl_signed_builtin {
    ($t:ident, $f:ident) => {
        impl Shl<$f> for $t {
            type Output = $t;

            #[inline]
            fn shl(self, other: $f) -> $t {
                if other < 0 {
                    $t(self.0.checked_shr(-other as u32).unwrap())
                } else {
                    $t(self.0.checked_shl(other as u32).unwrap())
                }
            }
        }
        forward_ref_binop! { impl Shl, shl for $t, $f }

        impl ShlAssign<$f> for $t {
            #[inline]
            fn shl_assign(&mut self, other: $f) {
                *self = *self << other;
            }
        }
        forward_ref_op_assign! { impl ShlAssign, shl_assign for $t, $f }

        impl Shr<$f> for $t {
            type Output = $t;

            #[inline]
            fn shr(self, other: $f) -> $t {
                if other < 0 {
                    $t(self.0.checked_shl(-other as u32).unwrap())
                } else {
                    $t(self.0.checked_shr(other as u32).unwrap())
                }
            }
        }
        forward_ref_binop! { impl Shr, shr for $t, $f }

        impl ShrAssign<$f> for $t {
            #[inline]
            fn shr_assign(&mut self, other: $f) {
                *self = *self >> other;
            }
        }
        forward_ref_op_assign! { impl ShrAssign, shr_assign for $t, $f }
    };
}

macro_rules! sh_impl_unsigned_builtin {
    ($t:ident, $f:ident) => {
        impl Shl<$f> for $t {
            type Output = $t;

            #[inline]
            fn shl(self, other: $f) -> $t {
                $t(self.0.checked_shl(other as u32).unwrap())
            }
        }
        forward_ref_binop! { impl Shl, shl for $t, $f }

        impl ShlAssign<$f> for $t {
            #[inline]
            fn shl_assign(&mut self, other: $f) {
                *self = *self << other;
            }
        }
        forward_ref_op_assign! { impl ShlAssign, shl_assign for $t, $f }

        impl Shr<$f> for $t {
            type Output = $t;

            #[inline]
            fn shr(self, other: $f) -> $t {
                $t(self.0.checked_shr(other as u32).unwrap())
            }
        }
        forward_ref_binop! { impl Shr, shr for $t, $f }

        impl ShrAssign<$f> for $t {
            #[inline]
            fn shr_assign(&mut self, other: $f) {
                *self = *self >> other;
            }
        }
        forward_ref_op_assign! { impl ShrAssign, shr_assign for $t, $f }
    };
}

macro_rules! sh_impl_all {
    ($($t:ident),*) => ($(
        sh_impl_unsigned! { $t, U8 }
        sh_impl_unsigned! { $t, U16 }
        sh_impl_unsigned! { $t, U32 }
        sh_impl_unsigned! { $t, U64 }
        sh_impl_unsigned! { $t, U128 }

        sh_impl_signed! { $t, I8 }
        sh_impl_signed! { $t, I16 }
        sh_impl_signed! { $t, I32 }
        sh_impl_signed! { $t, I64 }
        sh_impl_signed! { $t, I128 }
        sh_impl_unsigned_builtin! { $t, u8 }
        sh_impl_unsigned_builtin! { $t, u16 }
        sh_impl_unsigned_builtin! { $t, u32 }
        sh_impl_unsigned_builtin! { $t, u64 }
        sh_impl_unsigned_builtin! { $t, u128 }

        sh_impl_signed_builtin! { $t, i8 }
        sh_impl_signed_builtin! { $t, i16 }
        sh_impl_signed_builtin! { $t, i32 }
        sh_impl_signed_builtin! { $t, i64 }
        sh_impl_signed_builtin! { $t, i128 }
    )*)
}


sh_impl_all! { I8, I16, I32, I64, I128, U8, U16, U32, U64, U128 }

#[allow(unused_macros)]
macro_rules! sh_impl_large {
    ($t:ident, $f:ident, $fb:literal) => {
        paste! {
            fn [<bigint_to_$f>](b: BigInt) -> $f {
                let bytes = v.to_signed_bytes_le();
                if bytes.len() > $fb/8 {
                    panic!("Overflow");
                } else {
                    let mut buf = if v.is_negative() {
                        [255u8; $fb/8]
                    } else {
                        [0u8; $fb/8]
                    };
                    buf[..bytes.len()].copy_from_slice(&bytes);
                    <$f>::from_le_bytes(buf)
                }
            }
        impl Shl<$f> for $t {
            type Output = $t;

            #[inline]
            fn shl(self, other: $f) -> $t {
                if(other.abs() > $fb$f) {
                    panic!("overflow");
                } else {
                    if $fb > 128i128 {
                        $t(BigInt::from_signed_bytes_le(&self.0).shl([<bigint_to_$f>](BigInt::from_signed_bytes_le(&other.0))))
                    } else {
                        $t(BigInt::from_signed_bytes_le(&self.0).shl(other.0))
                    }
                }
            }
        }
        forward_ref_binop! { impl Shl, shl for $t, $f }

        impl ShlAssign<$f> for $t {
            #[inline]
            fn shl_assign(&mut self, other: $f) {
                *self = *self << other;
            }
        }
        forward_ref_op_assign! { impl ShlAssign, shl_assign for $t, $f }

        impl Shr<$f> for $t {
            type Output = $t;

            #[inline]
            fn shr(self, other: $f) -> $t {
                if(other.abs() > $fb$f) {
                    panic!("overflow");
                } else {
                    if $bi128 > 128i128 {
                        $t(BigInt::from_signed_bytes_le(&self.0).shr([<bigint_to_$f>](BigInt::from_signed_bytes_le(&other.0))))
                    } else {
                        $t(BigInt::from_signed_bytes_le(&self.0).shr(other.0))
                    }
                }
            }
        }
        forward_ref_binop! { impl Shr, shr for $t, $f }

        impl ShrAssign<$f> for $t {
            #[inline]
            fn shr_assign(&mut self, other: $f) {
                *self = *self >> other;
            }
        }
        forward_ref_op_assign! { impl ShrAssign, shr_assign for $t, $f }
        }
    };
}


#[allow(unused_macros)]
macro_rules! sh_impl_builtin_large {
    ($t:ident, $tb: literal, $f:ident) => {
        paste! {
        impl Shl<$f> for $t {
            type Output = $t;

            #[inline]
            fn shl(self, other: $f) -> $t {
                if(other > $tb$f) {
                    panic!("overflow");
                } else {
                    $t(BigInt::from_le_bytes(self.0).shl(other))
                }
            }
        }
        forward_ref_binop! { impl Shl, shl for $t, $f }

        impl ShlAssign<$f> for $t {
            #[inline]
            fn shl_assign(&mut self, other: $f) {
                *self = *self << other;
            }
        }
        forward_ref_op_assign! { impl ShlAssign, shl_assign for $t, $f }

        impl Shr<$f> for $t {
            type Output = $t;

            #[inline]
            fn shr(self, other: $f) -> $t {
                if(other > $tb$f) {
                    panic!("overflow");
                } else {
                    $t(BigInt::from_le_bytes(self.0).shr(other))
                }
            }
        }
        forward_ref_binop! { impl Shr, shr for $t, $f }

        impl ShrAssign<$f> for $t {
            #[inline]
            fn shr_assign(&mut self, other: $f) {
                *self = *self >> other;
            }
        }
        forward_ref_op_assign! { impl ShrAssign, shr_assign for $t, $f }
        }
    };
}

macro_rules! sh_impl_all_large {
    ($($t:ident),*) => {
        $(
        sh_impl_large! { $t, U8, 8 }
        sh_impl_large! { $t, U16, 16 }
        sh_impl_large! { $t, U32, 32 }
        sh_impl_large! { $t, U64, 64 }
        sh_impl_large! { $t, U128, 128 }
        sh_impl_large! { $t, U128, 128 }

        sh_impl_builtin_large! { $t, u8 }
        sh_impl_builtin_large! { $t, u16 }
        sh_impl_builtin_large! { $t, u32 }
        sh_impl_builtin_large! { $t, u64 }
        sh_impl_builtin_large! { $t, u128 }

    )*
    };
}


sh_impl_all_large! { I256, I384, I512, U256, U384, U512 }

macro_rules! checked_impl {
    ($(($I:ident, $t:ident, $u:ty)),*) => ($(
        impl Add<$t> for $I {
            type Output = $I;

            #[inline]
            fn add(self, other: $t) -> $I {
                $I(self.0.checked_add(other.0.try_into().unwrap()).unwrap())
            }
        }
        forward_ref_binop! { impl Add, add for $I, $t }

        impl AddAssign<$t> for $I {
            #[inline]
            fn add_assign(&mut self, other: $t) {
                *self = *self + other;
            }
        }
        forward_ref_op_assign! { impl AddAssign, add_assign for $I, $t }

        impl Sub<$t> for $I {
            type Output = $I;

            #[inline]
            fn sub(self, other: $t) -> $I {
                $I(self.0.checked_sub(other.0.try_into().unwrap()).unwrap())
            }
        }
        forward_ref_binop! { impl Sub, sub for $I, $t }

        impl SubAssign<$t> for $I {
            #[inline]
            fn sub_assign(&mut self, other: $t) {
                *self = *self - other;
            }
        }
        forward_ref_op_assign! { impl SubAssign, sub_assign for $I, $t }

        impl Mul<$t> for $I {
            type Output = $I;

            #[inline]
            fn mul(self, other: $t) -> $I {
                $I(self.0.checked_mul(other.0.try_into().unwrap()).unwrap())
            }
        }
        forward_ref_binop! { impl Mul, mul for $I, $t }

        impl MulAssign<$t> for $I {
            #[inline]
            fn mul_assign(&mut self, other: $t) {
                *self = *self * other;
            }
        }
        forward_ref_op_assign! { impl MulAssign, mul_assign for $I, $t }

        impl Div<$t> for $I {
            type Output = $I;

            #[inline]
            fn div(self, other: $t) -> $I {
                $I(self.0.checked_div(other.0.try_into().unwrap()).unwrap())
            }
        }
        forward_ref_binop! { impl Div, div for $I, $t }

        impl DivAssign<$t> for $I {
            #[inline]
            fn div_assign(&mut self, other: $t) {
                *self = *self / other;
            }
        }
        forward_ref_op_assign! { impl DivAssign, div_assign for $I, $t }

        impl Rem<$t> for $I {
            type Output = $I;

            #[inline]
            fn rem(self, other: $t) -> $I {
                $I(self.0.checked_rem(other.0.try_into().unwrap()).unwrap())
            }
        }
        forward_ref_binop! { impl Rem, rem for $I, $t }

        impl RemAssign<$t> for $I {
            #[inline]
            fn rem_assign(&mut self, other: $t) {
                *self = *self % other;
            }
        }
        forward_ref_op_assign! { impl RemAssign, rem_assign for $I, $t }


        impl BitXor<$t> for $I {
            type Output = $I;

            #[inline]
            fn bitxor(self, other: $t) -> $I {
                $I((<$u>::try_from(self.0).unwrap() ^ <$u>::try_from(other.0).unwrap()).try_into().unwrap())
            }
        }
        forward_ref_binop! { impl BitXor, bitxor for $I, $t }

        impl BitXorAssign<$t> for $I {
            #[inline]
            fn bitxor_assign(&mut self, other: $t) {
                *self = *self ^ other;
            }
        }
        forward_ref_op_assign! { impl BitXorAssign, bitxor_assign for $I, $t }

        impl BitOr<$t> for $I {
            type Output = $I;

            #[inline]
            fn bitor(self, other: $t) -> $I {
                $I((<$u>::try_from(self.0).unwrap() | <$u>::try_from(other.0).unwrap()).try_into().unwrap())
            }
        }
        forward_ref_binop! { impl BitOr, bitor for $I, $t }

        impl BitOrAssign<$t> for $I {
            #[inline]
            fn bitor_assign(&mut self, other: $t) {
                *self = *self | other;
            }
        }
        forward_ref_op_assign! { impl BitOrAssign, bitor_assign for $I, $t }

        impl BitAnd<$t> for $I {
            type Output = $I;

            #[inline]
            fn bitand(self, other: $t) -> $I {
                $I((<$u>::try_from(self.0).unwrap() & <$u>::try_from(other.0).unwrap()).try_into().unwrap())
            }
        }
        forward_ref_binop! { impl BitAnd, bitand for $I, $t }

        impl BitAndAssign<$t> for $I {
            #[inline]
            fn bitand_assign(&mut self, other: $t) {
                *self = *self & other;
            }
        }
        forward_ref_op_assign! { impl BitAndAssign, bitand_assign for $I, $t }
    )*)
}
macro_rules! checked_int_ops {
    ($(($I:ident, $U:ident)),*) => {
        $(
            checked_impl! { 
                ($I, I8, u8),
                ($I, U8, u8),
                ($U, U8, u8),
                ($U, I8, u8),
                ($I, I16, u16),
                ($I, U16, u16),
                ($U, U16, u16),
                ($U, I16, u16),
                ($I, I32, u32),
                ($I, U32, u32),
                ($U, U32, u32),
                ($U, I32, u32),
                ($I, I64, u64),
                ($I, U64, u64),
                ($U, U64, u64),
                ($U, I64, u64),
                ($I, I128, u128),
                ($I, U128, u128),
                ($U, U128, u128),
                ($U, I128, u128)
            }
        )*
    }
}

checked_int_ops! { (I8, U8), (I16, U16), (I32, U32), (I64, U64), (I128, U128) }

macro_rules! checked_impl_not {
    ($($i:ident),*) => {
        $(
        impl Not for $i {
            type Output = $i;

            #[inline]
            fn not(self) -> $i {
                $i(!self.0)
            }
        }
        forward_ref_unop! { impl Not, not for $i }
        )*
    }
}

checked_impl_not! { I8, I16, I32, I64, I128, U8, U16, U32, U64, U128 } 

macro_rules! checked_impl_neg {
    ($($i:ident),*) => {
        $(
        impl Neg for $i {
            type Output = Self;
            #[inline]
            fn neg(self) -> Self {
                $i(0) - self
            }
        }
        forward_ref_unop! { impl Neg, neg for $i }
        )*
    }
}
checked_impl_neg! { I8, I16, I32, I64, I128 }

macro_rules! checked_int_impl {
    ($(($I:ident, $i:ident)),*) => {$(
        paste! {
            impl $I {
                /// Returns the smallest value that can be represented by this integer type.
                ///
                /// # Examples
                ///
                /// Basic usage:
                ///
                /// ```
                #[doc = concat!("use scrypto::math::" ,stringify!($I), ";")]
                ///
                #[doc = concat!("assert_eq!(<$I>::MIN, $I(", stringify!($n), "::MIN));")]
                /// ```
                pub const MIN: Self = Self(<$i>::MIN);

                /// Returns the largest value that can be represented by this integer type.
                ///
                /// # Examples
                ///
                /// Basic usage:
                ///
                /// ```
                #[doc = concat!("use scrypto::math::" ,stringify!($I), ";")]
                ///
                #[doc = concat!("assert_eq!(<$I>::MAX, $I(", stringify!($i), "::MAX));")]
                /// ```
                pub const MAX: Self = Self(<$i>::MAX);

                /// Returns the size of this integer type in bits.
                ///
                /// # Examples
                ///
                /// Basic usage:
                ///
                /// ```
                #[doc = concat!("use scrypto::math::" ,stringify!($I), ";")]
                ///
                #[doc = concat!("assert_eq!(<$I>::BITS, ", stringify!($i), "::BITS);")]
                /// ```
                pub const BITS: u32 = <$i>::BITS;

                /// Returns the number of ones in the binary representation of `self`.
                ///
                /// # Examples
                ///
                /// Basic usage:
                ///
                /// ```
                #[doc = concat!("use scrypto::math::" ,stringify!($I), ";")]
                ///
                #[doc = concat!("let n = $I(0b01001100", stringify!($i), ");")]
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
                #[doc = concat!("use scrypto::math::" ,stringify!($I), ";")]
                ///
                #[doc = concat!("assert_eq!($I(!0", stringify!($i), ").count_zeros(), 0);")]
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
                #[doc = concat!("use scrypto::math::" ,stringify!($I), ";")]
                ///
                #[doc = concat!("let n = $I(0b0101000", stringify!($i), ");")]
                ///
                /// assert_eq!(n.trailing_zeros(), 3);
                /// ```
                #[inline]
                #[must_use = "this returns the result of the operation, \
                          without modifying the original"]
                pub const fn trailing_zeros(self) -> u32 {
                    self.0.trailing_zeros()
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
                #[doc = concat!("use scrypto::math::" ,stringify!($I), ";")]
                ///
                /// let n: $I = $I(0x0123456789ABCDEF);
                /// let m: $I = $I(-0x76543210FEDCBA99);
                ///
                /// assert_eq!(n.rotate_left(32), m);
                /// ```
                #[inline]
                #[must_use = "this returns the result of the operation, \
                          without modifying the original"]
                pub const fn rotate_left(self, n: u32) -> Self {
                    $I(self.0.rotate_left(n))
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
                #[doc = concat!("use scrypto::math::" ,stringify!($I), ";")]
                ///
                /// let n: $I = $I(0x0123456789ABCDEF);
                /// let m: $I = $I(-0xFEDCBA987654322);
                ///
                /// assert_eq!(n.rotate_right(4), m);
                /// ```
                #[inline]
                #[must_use = "this returns the result of the operation, \
                          without modifying the original"]
                pub const fn rotate_right(self, n: u32) -> Self {
                    $I(self.0.rotate_right(n))
                }

                /// Reverses the byte order of the integer.
                ///
                /// # Examples
                ///
                /// Basic usage:
                ///
                /// ```
                #[doc = concat!("use scrypto::math::" ,stringify!($I), ";")]
                ///
                /// let n: $I = $I(0b0000000_01010101);
                /// assert_eq!(n, $I(85));
                ///
                /// let m = n.swap_bytes();
                ///
                /// assert_eq!(m, $I(0b01010101_00000000));
                /// assert_eq!(m, $I(21760));
                /// ```
                #[inline]
                #[must_use = "this returns the result of the operation, \
                          without modifying the original"]
                pub const fn swap_bytes(self) -> Self {
                    $I(self.0.swap_bytes())
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
                #[doc = concat!("use scrypto::math::" ,stringify!($I), ";")]
                ///
                /// let n = $I(0b0000000_01010101i16);
                /// assert_eq!(n, $I(85));
                ///
                /// let m = n.reverse_bits();
                ///
                /// assert_eq!(m.0 as u16, 0b10101010_00000000);
                /// assert_eq!(m, $I(-22016));
                /// ```
                #[must_use = "this returns the result of the operation, \
                          without modifying the original"]
                #[inline]
                pub const fn reverse_bits(self) -> Self {
                    $I(self.0.reverse_bits())
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
                #[doc = concat!("use scrypto::math::" ,stringify!($I), ";")]
                ///
                #[doc = concat!("let n = $I(0x1A", stringify!($i), ");")]
                ///
                /// if cfg!(target_endian = "big") {
                #[doc = concat!("    assert_eq!(<$I>::from_be(n), n)")]
                /// } else {
                #[doc = concat!("    assert_eq!(<$I>::from_be(n), n.swap_bytes())")]
                /// }
                /// ```
                #[inline]
                #[must_use]
                pub const fn from_be(x: Self) -> Self {
                    $I(<$i>::from_be(x.0))
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
                #[doc = concat!("use scrypto::math::" ,stringify!($I), ";")]
                ///
                #[doc = concat!("let n = $I(0x1A", stringify!($i), ");")]
                ///
                /// if cfg!(target_endian = "little") {
                #[doc = concat!("    assert_eq!(<$I>::from_le(n), n)")]
                /// } else {
                #[doc = concat!("    assert_eq!(<$I>::from_le(n), n.swap_bytes())")]
                /// }
                /// ```
                #[inline]
                #[must_use]
                pub const fn from_le(x: Self) -> Self {
                    $I(<$i>::from_le(x.0))
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
                #[doc = concat!("use scrypto::math::" ,stringify!($I), ";")]
                ///
                #[doc = concat!("let n = $I(0x1A", stringify!($i), ");")]
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
                    $I(self.0.to_be())
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
                #[doc = concat!("use scrypto::math::" ,stringify!($I), ";")]
                ///
                #[doc = concat!("let n = $I(0x1A", stringify!($i), ");")]
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
                    $I(self.0.to_le())
                }

                /// Raises self to the power of `exp`, using exponentiation by squaring.
                ///
                #[inline]
                #[must_use = "this returns the result of the operation, \
                          without modifying the original"]
                pub fn pow(self, exp: u32) -> Self {
                    $I(self.0.checked_pow(exp).unwrap())
                }
            }
        }
    )*}
}

checked_int_impl! { (I8, i8), (I16, i16), (I32, i32), (I64, i64), (I128, i128) }
checked_int_impl! { (U8, u8), (U16, u16), (U32, u32), (U64, u64), (U128, u128) }

macro_rules! checked_int_impl_signed {
    ($(($I:ident, $i: ident)),*) => ($(
        paste! {
            impl $I {
                /// Returns the number of leading zeros in the binary representation of `self`.
                ///
                /// # Examples
                ///
                /// Basic usage:
                ///
                /// ```
                #[doc = concat!("use scrypto::math::" ,stringify!($I), ";")]
                ///
                #[doc = concat!("let n = $I(", stringify!($i), "::MAX) >> 2;")]
                ///
                /// assert_eq!(n.leading_zeros(), 3);
                /// ```
                #[inline]
                #[must_use = "this returns the result of the operation, \
                          without modifying the original"]
                pub const fn leading_zeros(self) -> u32 {
                    self.0.leading_zeros()
                }

                /// Computes the absolute value of `self`, with overflow causing panic.
                ///
                /// The only case where such overflow can occur is when one takes the absolute value of the negative
                /// minimal value for the type this is a positive value that is too large to represent in the type. In
                /// such a case, this function panics.
                ///
                #[inline]
                #[must_use = "this returns the result of the operation, \
                          without modifying the original"]
                pub fn abs(self) -> $I {
                    $I(self.0.checked_abs().unwrap())
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
                #[doc = concat!("use scrypto::math::" ,stringify!($I), ";")]
                ///
                #[doc = concat!("assert_eq!($I(10", stringify!($i), ").signum(), $I(1));")]
                #[doc = concat!("assert_eq!($I(0", stringify!($i), ").signum(), $I(0));")]
                #[doc = concat!("assert_eq!($I(-10", stringify!($i), ").signum(), $I(-1));")]
                /// ```
                #[inline]
                #[must_use = "this returns the result of the operation, \
                          without modifying the original"]
                pub fn signum(self) -> $I {
                    $I(self.0.signum())
                }

                /// Returns `true` if `self` is positive and `false` if the number is zero or
                /// negative.
                ///
                /// # Examples
                ///
                /// Basic usage:
                ///
                /// ```
                #[doc = concat!("use scrypto::math::" ,stringify!($I), ";")]
                ///
                #[doc = concat!("assert!($I(10", stringify!($i), ").is_positive());")]
                #[doc = concat!("assert!(!$I(-10", stringify!($i), ").is_positive());")]
                /// ```
                #[must_use]
                #[inline]
                pub const fn is_positive(self) -> bool {
                    self.0.is_positive()
                }

                /// Returns `true` if `self` is negative and `false` if the number is zero or
                /// positive.
                ///
                /// # Examples
                ///
                /// Basic usage:
                ///
                /// ```
                #[doc = concat!("use scrypto::math::" ,stringify!($I), ";")]
                ///
                #[doc = concat!("assert!($I(-10", stringify!($i), ").is_negative());")]
                #[doc = concat!("assert!(!$I(10", stringify!($i), ").is_negative());")]
                /// ```
                #[must_use]
                #[inline]
                pub const fn is_negative(self) -> bool {
                    self.0.is_negative()
                }
            }
        }
    )*)
}

checked_int_impl_signed! { (I8, i8), (I16, i16), (I32, i32), (I64, i64), (I128, i128) }

macro_rules! checked_int_impl_unsigned {
    ($($t:ty),*) => ($(
        impl $t {
            /// Returns the number of leading zeros in the binary representation of `self`.
            ///
            #[inline]
            #[must_use = "this returns the result of the operation, \
                          without modifying the original"]
            pub const fn leading_zeros(self) -> u32 {
                self.0.leading_zeros()
            }

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

checked_int_impl_unsigned! { U8, U16, U32, U64, U128 }


macro_rules! from_int {
    ($(($I:ident, $t:ident)),*) => {
        impl From<$t> for $I {
            fn from(val: $t) -> Self {
                Self((val as i128) * Self::ONE.0)
            }
        }
    };
}

macro_rules! from_int_type {
    ($($I:ident),*) => {
        $(
            from_int!(u8);
            from_int!(u16);
            from_int!(u32);
            from_int!(u64);
            from_int!(usize);
            from_int!(i8);
            from_int!(i16);
            from_int!(i32);
            from_int!(i64);
            from_int!(i128);
            from_int!(isize);
        )
    };


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
