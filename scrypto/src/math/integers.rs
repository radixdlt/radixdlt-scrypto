//! Definitions of safe integers and uints.


use core::fmt;
use core::ops::{Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign};
use core::ops::{BitXor, BitXorAssign, Div, DivAssign};
use core::ops::{Mul, MulAssign, Neg, Not, Rem, RemAssign};
use core::ops::{Shl, ShlAssign, Shr, ShrAssign, Sub, SubAssign};
use forward_ref::*;
use paste::paste;
use num_bigint::BigInt;
 
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

macro_rules! impl_i_large {
    ($($t:ty),*) => {
        $(
            impl fmt::Debug for $t {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    BigInt::from_signed_bytes_le(&self.0).fmt(f)
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

impl_i_large! { I256, I384, I512, U256, U384, U512 }

macro_rules! impl_bigint_to_i {
    ($($F:ty, $b:literal ),*) => {
        $(
            paste! {
                fn [<bigint_to_$F:lower>](b: BigInt) -> $F {
                    let bytes = b.to_signed_bytes_le();
                    if bytes.len() > $b/8 {
                        panic!("Overflow");
                    } else {
                        let mut buf = if b.is_negative() {
                            [255u8; $b/8]
                        } else {
                            [0u8; $b/8]
                        };
                        buf[..bytes.len()].copy_from_slice(&bytes);
                        $F(buf)
                    }
                }
            }
        )*
    }
}

fn big_int_to_i128(v: BigInt) -> i128 {
    let bytes = v.to_signed_bytes_le();
    if bytes.len() > 16 {
        panic!("Overflow");
    } else {
        let mut buf = if v.is_negative() {
            [255u8; 16]
        } else {
            [0u8; 16]
        };
        buf[..bytes.len()].copy_from_slice(&bytes);
        i128::from_le_bytes(buf)
    }
}


impl_bigint_to_i! { I256, 256, I384, 384, I512, 512 , U256, 256, U384, 384, U512, 512 }

#[allow(unused_macros)]
macro_rules! sh_impl_large {
    ($(to_sh: $T:ty, to_sh_bits: $b:literal, other: $F:ty ),*) => {
        $(
            paste! {
                impl Shl<$F> for $T {
                    type Output = $T;

                    #[inline]
                    fn shl(self, other: $F) -> $T {
                        if(other.abs() > $b) {
                            panic!("overflow");
                        } else {
                            let to_shift = BigInt::from_signed_bytes_le(&self.0);
                            let shift = big_int_to_i128(BigInt::from_signed_bytes_le(&other.0));
                            [<bigint_to_$T:lower>](to_shift.shl(shift))
                        }
                    }
                }
                forward_ref_binop! { impl Shl, shl for $T, $F }

                impl ShlAssign<$F> for $T {
                    #[inline]
                    fn shl_assign(&mut self, other: $F) {
                        *self = *self << other;
                    }
                }
                forward_ref_op_assign! { impl ShlAssign, shl_assign for $T, $F }

                impl Shr<$F> for $T {
                    type Output = $T;

                    #[inline]
                    fn shr(self, other: $F) -> $T {
                        if(other.abs() > $b) {
                            panic!("overflow");
                        } else {
                            let to_shift = BigInt::from_signed_bytes_le(&self.0);
                            let shift = big_int_to_i128(BigInt::from_signed_bytes_le(&other.0));
                            [<bigint_to_$T:lower>](to_shift.shr(shift))
                        }
                    }
                }
                forward_ref_binop! { impl Shr, shr for $T, $F }

                impl ShrAssign<$F> for $T {
                    #[inline]
                    fn shr_assign(&mut self, other: $F) {
                        *self = *self >> other;
                    }
                }
                forward_ref_op_assign! { impl ShrAssign, shr_assign for $T, $F }
            }
        )*
    };
}


#[allow(unused_macros)]
macro_rules! sh_impl_builtin_large {
    (to_sh: $T:ty, to_sh_bits: $b:literal, other: $f:ty) => {
        paste! {
        impl Shl<$f> for $T {
            type Output = $T;

            #[inline]
            fn shl(self, other: $f) -> $T {
                if(other > $b) {
                    panic!("overflow");
                } else {
                    [<bigint_to_$T:lower>](BigInt::from_signed_bytes_le(&self.0).shl(other))
                }
            }
        }
        forward_ref_binop! { impl Shl, shl for $T, $f }

        impl ShlAssign<$f> for $T {
            #[inline]
            fn shl_assign(&mut self, other: $f) {
                *self = *self << other;
            }
        }
        forward_ref_op_assign! { impl ShlAssign, shl_assign for $T, $f }

        impl Shr<$f> for $T {
            type Output = $T;

            #[inline]
            fn shr(self, other: $f) -> $T {
                if(other > $b) {
                    panic!("overflow");
                } else {
                    [<bigint_to_$T:lower>](BigInt::from_signed_bytes_le(&self.0).shr(other))
                }
            }
        }
        forward_ref_binop! { impl Shr, shr for $T, $f }

        impl ShrAssign<$f> for $T {
            #[inline]
            fn shr_assign(&mut self, other: $f) {
                *self = *self >> other;
            }
        }
        forward_ref_op_assign! { impl ShrAssign, shr_assign for $T, $f }
        }
    };
}

macro_rules! sh_impl_all {
    ($($T:ty, $b:literal),*) => {
        $(
            sh_impl_large! { to_sh: $T, to_sh_bits: $b, other: I256}
            sh_impl_large! { to_sh: $T, to_sh_bits: $b, other: I384}
            sh_impl_large! { to_sh: $T, to_sh_bits: $b, other: I512}
            sh_impl_large! { to_sh: $T, to_sh_bits: $b, other: U256}
            sh_impl_large! { to_sh: $T, to_sh_bits: $b, other: U384}
            sh_impl_large! { to_sh: $T, to_sh_bits: $b, other: U512}
            sh_impl_builtin_large! { to_sh: $T, to_sh_bits: $b, other: i8}
            sh_impl_builtin_large! { to_sh: $T, to_sh_bits: $b, other: i16}
            sh_impl_builtin_large! { to_sh: $T, to_sh_bits: $b, other: i32}
            sh_impl_builtin_large! { to_sh: $T, to_sh_bits: $b, other: i64}
            sh_impl_builtin_large! { to_sh: $T, to_sh_bits: $b, other: i128}
            sh_impl_builtin_large! { to_sh: $T, to_sh_bits: $b, other: u8}
            sh_impl_builtin_large! { to_sh: $T, to_sh_bits: $b, other: u16}
            sh_impl_builtin_large! { to_sh: $T, to_sh_bits: $b, other: u32}
            sh_impl_builtin_large! { to_sh: $T, to_sh_bits: $b, other: u64}
            sh_impl_builtin_large! { to_sh: $T, to_sh_bits: $b, other: u128}
        )*
    };
}

sh_impl_all!(I256, 256, I384, 384, I512, 512, U256, 256, U384, 384, U512, 512);

macro_rules! checked_impl {
    ($(($t:ty, $o:ty)),*) => {
        paste! {
            $(
                impl Add<$o> for $t {
                    type Output = $t;

                    #[inline]
                    fn add(self, other: $o) -> $t {
                        [<bigint_to_$t:lower>](BigInt::from_signed_bytes_le(&self.0).add(other))
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
                    type Output = $t;

                    #[inline]
                    fn sub(self, other: $o) -> $t {
                        [<bigint_to_$t:lower>](BigInt::from_signed_bytes_le(&self.0).sub(other))
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
                    type Output = $t;

                    #[inline]
                    fn mul(self, other: $o) -> $t {
                        [<bigint_to_$t:lower>](BigInt::from_signed_bytes_le(&self.0).mul(other))
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
                    type Output = $t;

                    #[inline]
                    fn div(self, other: $o) -> $t {
                        [<bigint_to_$t:lower>](BigInt::from_signed_bytes_le(&self.0).div(other))
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
                    type Output = $t;

                    #[inline]
                    fn rem(self, other: $o) -> $t {
                        [<bigint_to_$t:lower>](BigInt::from_signed_bytes_le(&self.0).rem(other))
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
                    type Output = $t;

                    #[inline]
                    fn bitxor(self, other: $o) -> $t {
                        [<bigint_to_$t:lower>](BigInt::from_signed_bytes_le(&self.0).bitxor(BigInt::from(other)))
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
                    type Output = $t;

                    #[inline]
                    fn bitor(self, other: $o) -> $t {
                        [<bigint_to_$t:lower>](BigInt::from_signed_bytes_le(&self.0).bitor(BigInt::from(other)))
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
                    type Output = $t;

                    #[inline]
                    fn bitand(self, other: $o) -> $t {
                        [<bigint_to_$t:lower>](BigInt::from_signed_bytes_le(&self.0).bitand(BigInt::from(other)))
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
                )*
        }
    };
}
macro_rules! checked_int_ops {
    ($($t:ident),*) => {
        $(
            checked_impl! { 
                ($t, u8),
                ($t, u16),
                ($t, u32),
                ($t, u64),
                ($t, u128),
                ($t, i8),
                ($t, i16),
                ($t, i32),
                ($t, i64),
                ($t, i128)
            }
        )*
    }
}

checked_int_ops! { I256, U256, I384, U384, I512, U512 }

macro_rules! checked_impl_not {
    ($($i:ident),*) => {
        paste! {
            $(
                impl Not for $i {
                    type Output = $i;

                    #[inline]
                    fn not(self) -> $i {
                        [<bigint_to_$i:lower>](BigInt::from_signed_bytes_le(&self.0).not())
                    }
                }
                forward_ref_unop! { impl Not, not for $i }
            )*
        }
    }
}

checked_impl_not! {  I256, U256, I384, U384, I512, U512 }

macro_rules! checked_impl_neg {
    ($($i:ident),*) => {
        paste! {
            $(
                impl Neg for $i {
                    type Output = Self;
                    #[inline]
                    fn neg(self) -> Self {
// FIXME: set .into() syntax, remove paste
                    [<bigint_to_$i:lower>](BigInt::from_signed_bytes_le(&self.0).not())
                    }
                }
                forward_ref_unop! { impl Neg, neg for $i }
            )*
        }
    }
}
checked_impl_neg! { I256, I384, I512 }

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
}


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
