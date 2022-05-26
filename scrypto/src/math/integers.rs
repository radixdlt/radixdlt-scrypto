//! Definitions of safe integers and uints.


use core::fmt;
use core::ops::{Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign};
use core::ops::{BitXor, BitXorAssign, Div, DivAssign};
use core::ops::{Mul, MulAssign, Neg, Not, Rem, RemAssign};
use core::ops::{Shl, ShlAssign, Shr, ShrAssign, Sub, SubAssign};
use forward_ref::*;
use paste::paste;
use num_bigint::BigInt;
use num_traits::Signed;
 
macro_rules! types_large {

    ($((type_ident: $t:ident, bytes_length: $bytes_length: literal)),*) => {
        
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
            pub struct $t(pub [u8; $bytes_length]);

            impl std::default::Default for $t {
                fn default() -> Self {
                    Self {0: [0; $bytes_length]}
                }
            }
        )*
    }
}

types_large! { 
(type_ident: I256, bytes_length: 32), 
(type_ident: I384, bytes_length: 48), 
(type_ident: I512, bytes_length: 64), 
(type_ident: U256, bytes_length: 32), 
(type_ident: U384, bytes_length: 48), 
(type_ident: U512, bytes_length: 64) 
}

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
    ($($t:ty, $bytes_len:literal ),*) => {
        $(
            paste! {
                fn [<bigint_to_$t:lower>](b: BigInt) -> $t {
                    let bytes = b.to_signed_bytes_le();
                    if bytes.len() > $bytes_len {
                        panic!("Overflow"); 
                    } else {
                        let mut buf = if b.is_negative() {
                            [255u8; $bytes_len]
                        } else {
                            [0u8; $bytes_len]
                        };
                        buf[..bytes.len()].copy_from_slice(&bytes);
                        $t(buf)
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


impl_bigint_to_i! { I256, 32, I384, 48, I512, 64 , U256, 32, U384, 48, U512, 64 }

#[allow(unused_macros)]
macro_rules! sh_impl_large_signed {
    (to_sh: $t:ty, to_sh_bits: $b:literal, other: $o:ty ) => {
        paste! {
            impl Shl<$o> for $t {
                type Output = $t;

                #[inline]
                fn shl(self, other: $o) -> $t {
                    if(other.abs() > $b) {
                        panic!("overflow");
                    } else {
                        let to_shift = BigInt::from_signed_bytes_le(&self.0);
                        let shift = big_int_to_i128(BigInt::from_signed_bytes_le(&other.0));
                        [<bigint_to_$t:lower>](to_shift.shl(shift))
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
                    if(other.abs() > $b) {
                        panic!("overflow");
                    } else {
                        let to_shift = BigInt::from_signed_bytes_le(&self.0);
                        let shift = big_int_to_i128(BigInt::from_signed_bytes_le(&other.0));
                        [<bigint_to_$t:lower>](to_shift.shr(shift))
                    }
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
        }
    };
}

#[allow(unused_macros)]
macro_rules! sh_impl_large_unsigned {
    (to_sh: $t:ty, to_sh_bits: $b:literal, other: $o:ty) => {
        paste! {
            impl Shl<$o> for $t {
                type Output = $t;

                #[inline]
                fn shl(self, other: $o) -> $t {
                    let to_shift = BigInt::from_signed_bytes_le(&self.0);
                    let shift = big_int_to_i128(BigInt::from_signed_bytes_le(&other.0));
                    [<bigint_to_$t:lower>](to_shift.shl(shift))
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
                    let to_shift = BigInt::from_signed_bytes_le(&self.0);
                    let shift = big_int_to_i128(BigInt::from_signed_bytes_le(&other.0));
                    [<bigint_to_$t:lower>](to_shift.shr(shift))
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
        }
    };
}


#[allow(unused_macros)]
macro_rules! sh_impl_builtin_large {
    (to_sh: $t:ty, to_sh_bits: $b:literal, other: $f:ty) => {
        paste! {
        impl Shl<$f> for $t {
            type Output = $t;

            #[inline]
            fn shl(self, other: $f) -> $t {
                if(other > $b) {
                    panic!("overflow");
                } else {
                    [<bigint_to_$t:lower>](BigInt::from_signed_bytes_le(&self.0).shl(other))
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
                if(other > $b) {
                    panic!("overflow");
                } else {
                    [<bigint_to_$t:lower>](BigInt::from_signed_bytes_le(&self.0).shr(other))
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

macro_rules! sh_impl_all {
    ($($t:ty, $b:literal),*) => {
        $(
            sh_impl_large_signed! { to_sh: $t, to_sh_bits: $b, other: I256}
//            sh_impl_large_signed! { to_sh: $t, to_sh_bits: $b, other: I384}
//            sh_impl_large_signed! { to_sh: $t, to_sh_bits: $b, other: I512}
//            sh_impl_large_unsigned! { to_sh: $t, to_sh_bits: $b, other: U256}
//            sh_impl_large_unsigned! { to_sh: $t, to_sh_bits: $b, other: U384}
//            sh_impl_large_unsigned! { to_sh: $t, to_sh_bits: $b, other: U512}
            sh_impl_builtin_large! { to_sh: $t, to_sh_bits: $b, other: i8}
            sh_impl_builtin_large! { to_sh: $t, to_sh_bits: $b, other: i16}
            sh_impl_builtin_large! { to_sh: $t, to_sh_bits: $b, other: i32}
            sh_impl_builtin_large! { to_sh: $t, to_sh_bits: $b, other: i64}
            sh_impl_builtin_large! { to_sh: $t, to_sh_bits: $b, other: i128}
            sh_impl_builtin_large! { to_sh: $t, to_sh_bits: $b, other: u8}
            sh_impl_builtin_large! { to_sh: $t, to_sh_bits: $b, other: u16}
            sh_impl_builtin_large! { to_sh: $t, to_sh_bits: $b, other: u32}
            sh_impl_builtin_large! { to_sh: $t, to_sh_bits: $b, other: u64}
            sh_impl_builtin_large! { to_sh: $t, to_sh_bits: $b, other: u128}
        )*
    };
}

sh_impl_all!{ I256, 256, I384, 384, I512, 512, U256, 256, U384, 384, U512, 512}

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
    ($(($i:ident, $n:ident)),*) => {$(
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
                #[doc = concat!("assert_eq!(<$i>::MIN, $i(", stringify!($n), "::MIN));")]
                /// ```
                pub const MIN: Self = Self(0: [255; $n]);

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
                pub const MAX: Self = Self(<$i>::MAX);

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
                pub const BITS: u32 = <$i>::BITS;

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
                #[doc = concat!("use scrypto::math::" ,stringify!($i), ";")]
                ///
                #[doc = concat!("assert_eq!($i(!0", stringify!($i), ").count_zeros(), 0);")]
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
                #[doc = concat!("use scrypto::math::" ,stringify!($i), ";")]
                ///
                #[doc = concat!("let n = $i(0b0101000", stringify!($i), ");")]
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
                pub const fn rotate_left(self, n: u32) -> Self {
                    $i(self.0.rotate_left(n))
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
                pub const fn rotate_right(self, n: u32) -> Self {
                    $i(self.0.rotate_right(n))
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
                pub const fn swap_bytes(self) -> Self {
                    $i(self.0.swap_bytes())
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
                pub const fn reverse_bits(self) -> Self {
                    $i(self.0.reverse_bits())
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
                pub const fn from_be(x: Self) -> Self {
                    $i(<$i>::from_be(x.0))
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
                pub const fn from_le(x: Self) -> Self {
                    $i(<$i>::from_le(x.0))
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
                pub const fn to_be(self) -> Self {
                    $i(self.0.to_be())
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
                pub const fn to_le(self) -> Self {
                    $i(self.0.to_le())
                }

                /// Raises self to the power of `exp`, using exponentiation by squaring.
                ///
                #[inline]
                #[must_use = "this returns the result of the operation, \
                          without modifying the original"]
                pub fn pow(self, exp: u32) -> Self {
                    $i(self.0.checked_pow(exp).unwrap())
                }
            }
        }
    )*}
}

checked_int_impl! { (I8, i8), (I16, i16), (I32, i32), (I64, i64), (I128, i128) }
checked_int_impl! { (U8, u8), (U16, u16), (U32, u32), (U64, u64), (U128, u128) }

macro_rules! checked_int_impl_signed {
    ($($t:ident),*) => ($(
        paste! {
            impl $t {
                /// Returns the number of leading zeros in the binary representation of `self`.
                ///
                /// # Examples
                ///
                /// Basic usage:
                ///
                /// ```
                #[doc = concat!("use scrypto::math::" ,stringify!($t), ";")]
                ///
                #[doc = concat!("let n = $t(", stringify!($t:lower), "::MAX) >> 2;")]
                ///
                /// assert_eq!(n.leading_zeros(), 3);
                /// ```
                #[inline]
                #[must_use = "this returns the result of the operation, \
                          without modifying the original"]
                pub const fn leading_zeros(self) -> u32 {
                    for i in self.0 {
                        if i != 0 {
                            return i.leading_zeros();
                        }
                    }
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
                pub fn abs(self) -> $t {
                    $t(self.0.checked_abs().unwrap())
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
                    $t(self.0.signum())
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
                #[doc = concat!("use scrypto::math::" ,stringify!($t), ";")]
                ///
                #[doc = concat!("assert!($t(-10", stringify!($t:lower), ").is_negative());")]
                #[doc = concat!("assert!(!$t(10", stringify!($t:lower), ").is_negative());")]
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

checked_int_impl_signed! { I256, I384, I512 }

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
    ($(($t:ident, $o:ident)),*) => {
        $(
        paste! {
            impl From<$o> for $t {
                fn from(val: $o) -> Self {
                    [<bigint_to_$t:lower>](BigInt::from(val))
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
                impl From<BigInt> for $t {
                    fn from(val: BigInt) -> Self {
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
                impl From<BigInt> for $t {
                    fn from(val: BigInt) -> Self {
                        if val.is_negative() {
                            panic!("cannot convert negative BigInt to unsigned type");
                        }
                        [<bigint_to_$t:lower>](val)
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
