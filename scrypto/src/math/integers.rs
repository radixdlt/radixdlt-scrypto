//! Definitions of safe integers and uints.


use core::fmt;
use core::ops::{Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign};
use core::ops::{BitXor, BitXorAssign, Div, DivAssign};
use core::ops::{Mul, MulAssign, Neg, Not, Rem, RemAssign};
use core::ops::{Shl, ShlAssign, Shr, ShrAssign, Sub, SubAssign};
use forward_ref::*;
use paste::paste;

/// Provides safe arithmetic on `T`.
///
/// Operations like `+`, '-', '*', or '/' sometimes produce overflow 
/// which is detected and results in a panic, instead of silently
/// wrapping around.
///
/// Integer arithmetic can be achieved either through methods like
/// `checked_add`, or through the `Ixx` and `Uxx` type, which ensures all 
/// standard arithmetic operations on the underlying value to have 
/// checked semantics.
///
/// The underlying value can be retrieved through the `.0` index of the
/// `I128` tuple.
///
/// # Layout
///
/// `I128` is guaranteed to have the same layout and ABI as `T`.

macro_rules! types {
    ($($l:literal)*) => {

        #[derive(Clone , Copy , Default , Eq , Hash , Ord , PartialEq , PartialOrd)]
        #[repr(transparent)]
       paste!{ 
        pub struct I[<$l>](pub i[<$l>]);
        pub struct U[<$l>](pub U[<$l>]);
       }
    }
}

// Generate the types for the given bit widths.
// I8, I16, I32, I64, I128, U8, U16, U32, U64, U128
types! { 8 16 32 64 128 }

macro_rules! impl_i {
    ($($t:ty)*) => {
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
    }
}

impl_i! { I8 I16 I32 I64 I128 U8 U16 U32 U64 U128 }

#[allow(unused_macros)]
macro_rules! sh_impl_signed {
    ($t:ident, $f:ident) => {
        impl Shl<$f> for I128 {
            type Output = I128;

            #[inline]
            fn shl(self, other: $f) -> I128 {
                if other < 0 {
                    Integer(self.0.checked_shr(-other as u32))
                } else {
                    Integer(self.0.checked_shl(other as u32))
                }
            }
        }
        forward_ref_binop! { impl Shl, shl for I128, $f }

        impl ShlAssign<$f> for I128 {
            #[inline]
            fn shl_assign(&mut self, other: $f) {
                *self = *self << other;
            }
        }
        forward_ref_op_assign! { impl ShlAssign, shl_assign for I128, $f }

        impl Shr<$f> for I128 {
            type Output = I128;

            #[inline]
            fn shr(self, other: $f) -> I128 {
                if other < 0 {
                    Integer(self.0.checked_shl(-other as u32))
                } else {
                    Integer(self.0.checked_shr(other as u32))
                }
            }
        }
        forward_ref_binop! { impl Shr, shr for I128, $f }

        impl ShrAssign<$f> for I128 {
            #[inline]
            fn shr_assign(&mut self, other: $f) {
                *self = *self >> other;
            }
        }
        forward_ref_op_assign! { impl ShrAssign, shr_assign for I128, $f }
    };
}

macro_rules! sh_impl_unsigned {
    ($t:ident, $f:ident) => {
        impl Shl<$f> for $t {
            type Output = $t;

            #[inline]
            fn shl(self, other: $f) -> $t {
                Integer(self.0.checked_shl(other as u32).unwrap())
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
                Integer(self.0.checked_shr(other as u32).unwrap())
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
    ($($t:ident)*) => ($(
        sh_impl_unsigned! { $t, u8 }
        sh_impl_unsigned! { $t, u16 }
        sh_impl_unsigned! { $t, u32 }
        sh_impl_unsigned! { $t, u64 }
        sh_impl_unsigned! { $t, u128 }
        sh_impl_unsigned! { $t, usize }
        sh_impl_unsigned! { $t, U8 }
        sh_impl_unsigned! { $t, U16 }
        sh_impl_unsigned! { $t, U32 }
        sh_impl_unsigned! { $t, U64 }
        sh_impl_unsigned! { $t, U128 }

        sh_impl_signed! { $t, i8 }
        sh_impl_signed! { $t, i16 }
        sh_impl_signed! { $t, i32 }
        sh_impl_signed! { $t, i64 }
        sh_impl_signed! { $t, i128 }
        sh_impl_signed! { $t, isize }
        sh_impl_signed! { $t, I8 }
        sh_impl_signed! { $t, I16 }
        sh_impl_signed! { $t, I32 }
        sh_impl_signed! { $t, I64 }
        sh_impl_signed! { $t, I128 }
    )*)
}


sh_impl_all! { I8 I16 I32 I64 I128 U8 U16 U32 U64 U128 }

macro_rules! checked_impl {
    ($($i:ty, $t:ty)*) => ($(
        impl Add for $i {
            type Output = $i;

            #[inline]
            fn add(self, other: $i) -> $i {
                Integer(self.0.checked_add(other.0).unwrap())
            }
        }
        forward_ref_binop! { impl Add, add for $i, $i }

        impl AddAssign for $i {
            #[inline]
            fn add_assign(&mut self, other: $i) {
                *self = *self + other;
            }
        }
        forward_ref_op_assign! { impl AddAssign, add_assign for $i, $i }

        impl AddAssign<$t> for $i {
            #[inline]
            fn add_assign(&mut self, other: $t) {
                *self = *self + Integer(other);
            }
        }
        forward_ref_op_assign! { impl AddAssign, add_assign for $i, $t }

        impl Sub for $i {
            type Output = $i;

            #[inline]
            fn sub(self, other: $i) -> $i {
                Integer(self.0.checked_sub(other.0).unwrap())
            }
        }
        forward_ref_binop! { impl Sub, sub for $i, $i }

        impl SubAssign for $i {
            #[inline]
            fn sub_assign(&mut self, other: $i) {
                *self = *self - other;
            }
        }
        forward_ref_op_assign! { impl SubAssign, sub_assign for $i, $i }

        impl SubAssign<$t> for $i {
            #[inline]
            fn sub_assign(&mut self, other: $t) {
                *self = *self - Integer(other);
            }
        }
        forward_ref_op_assign! { impl SubAssign, sub_assign for $i, $t }

        impl Mul for $i {
            type Output = $i;

            #[inline]
            fn mul(self, other: $i) -> $i {
                Integer(self.0.checked_mul(other.0).unwrap())
            }
        }
        forward_ref_binop! { impl Mul, mul for $i, $i }

        impl MulAssign for $i {
            #[inline]
            fn mul_assign(&mut self, other: $i) {
                *self = *self * other;
            }
        }
        forward_ref_op_assign! { impl MulAssign, mul_assign for $i, $i }

        impl MulAssign<$t> for $i {
            #[inline]
            fn mul_assign(&mut self, other: $t) {
                *self = *self * Integer(other);
            }
        }
        forward_ref_op_assign! { impl MulAssign, mul_assign for $i, $t }

        impl Div for $i {
            type Output = $i;

            #[inline]
            fn div(self, other: $i) -> $i {
                Integer(self.0.checked_div(other.0).unwrap())
            }
        }
        forward_ref_binop! { impl Div, div for $i, $i }

        impl DivAssign for $i {
            #[inline]
            fn div_assign(&mut self, other: $i) {
                *self = *self / other;
            }
        }
        forward_ref_op_assign! { impl DivAssign, div_assign for $i, $i }

        impl DivAssign<$t> for $i {
            #[inline]
            fn div_assign(&mut self, other: $t) {
                *self = *self / Integer(other);
            }
        }
        forward_ref_op_assign! { impl DivAssign, div_assign for $i, $t }

        impl Rem for $i {
            type Output = $i;

            #[inline]
            fn rem(self, other: $i) -> $i {
                Integer(self.0.checked_rem(other.0).unwrap())
            }
        }
        forward_ref_binop! { impl Rem, rem for $i, $i }

        impl RemAssign for $i {
            #[inline]
            fn rem_assign(&mut self, other: $i) {
                *self = *self % other;
            }
        }
        forward_ref_op_assign! { impl RemAssign, rem_assign for $i, $i }

        impl RemAssign<$t> for $i {
            #[inline]
            fn rem_assign(&mut self, other: $t) {
                *self = *self % Integer(other);
            }
        }
        forward_ref_op_assign! { impl RemAssign, rem_assign for $i, $t }

        impl Not for $i {
            type Output = $i;

            #[inline]
            fn not(self) -> $i {
                Integer(!self.0)
            }
        }
        forward_ref_unop! { impl Not, not for $i }

        impl BitXor for $i {
            type Output = $i;

            #[inline]
            fn bitxor(self, other: $i) -> $i {
                Integer(self.0 ^ other.0)
            }
        }
        forward_ref_binop! { impl BitXor, bitxor for $i, $i }

        impl BitXorAssign for $i {
            #[inline]
            fn bitxor_assign(&mut self, other: $i) {
                *self = *self ^ other;
            }
        }
        forward_ref_op_assign! { impl BitXorAssign, bitxor_assign for $i, $i }

        impl BitXorAssign<$t> for $i {
            #[inline]
            fn bitxor_assign(&mut self, other: $t) {
                *self = *self ^ Integer(other);
            }
        }
        forward_ref_op_assign! { impl BitXorAssign, bitxor_assign for $i, $t }

        impl BitOr for $i {
            type Output = $i;

            #[inline]
            fn bitor(self, other: $i) -> $i {
                Integer(self.0 | other.0)
            }
        }
        forward_ref_binop! { impl BitOr, bitor for $i, $i }

        impl BitOrAssign for $i {
            #[inline]
            fn bitor_assign(&mut self, other: $i) {
                *self = *self | other;
            }
        }
        forward_ref_op_assign! { impl BitOrAssign, bitor_assign for $i, $i }

        impl BitOrAssign<$t> for $i {
            #[inline]
            fn bitor_assign(&mut self, other: $t) {
                *self = *self | Integer(other);
            }
        }
        forward_ref_op_assign! { impl BitOrAssign, bitor_assign for $i, $t }

        impl BitAnd for $i {
            type Output = $i;

            #[inline]
            fn bitand(self, other: $i) -> $i {
                Integer(self.0 & other.0)
            }
        }
        forward_ref_binop! { impl BitAnd, bitand for $i, $i }

        impl BitAndAssign for $i {
            #[inline]
            fn bitand_assign(&mut self, other: $i) {
                *self = *self & other;
            }
        }
        forward_ref_op_assign! { impl BitAndAssign, bitand_assign for $i, $i }

        impl BitAndAssign<$t> for $i {
            #[inline]
            fn bitand_assign(&mut self, other: $t) {
                *self = *self & Integer(other);
            }
        }
        forward_ref_op_assign! { impl BitAndAssign, bitand_assign for $i, $t }

        impl Neg for $i {
            type Output = Self;
            #[inline]
            fn neg(self) -> Self {
                Integer(0) - self
            }
        }
        forward_ref_unop! { impl Neg, neg for $i }

    )*)
}

checked_impl! { U8, U16, U32, U64, U128, I8, I16, I32, I64, I128 }

macro_rules! checked_int_impl {
    ($($I:ident, $i:ident, $t:ident)*) => ($(
        paste! {
            impl [<$I$t>] {
                /// Returns the smallest value that can be represented by this integer type.
                ///
                /// # Examples
                ///
                /// Basic usage:
                ///
                /// ```
                /// use scrypto::math::Integer;
                ///
                #[doc = concat!("assert_eq!(<I128>::MIN, Integer(", stringify!($t), "::MIN));")]
                /// ```
                pub const MIN: Self = Self(<[<$i$t>]>::MIN);

                /// Returns the largest value that can be represented by this integer type.
                ///
                /// # Examples
                ///
                /// Basic usage:
                ///
                /// ```
                /// use scrypto::math::Integer;
                ///
                #[doc = concat!("assert_eq!(<I128>::MAX, Integer(", stringify!([<$i$t>]), "::MAX));")]
                /// ```
                pub const MAX: Self = Self(<[<$i$t>]>::MAX);

                /// Returns the size of this integer type in bits.
                ///
                /// # Examples
                ///
                /// Basic usage:
                ///
                /// ```
                /// use scrypto::math::Integer;
                ///
                #[doc = concat!("assert_eq!(<I128>::BITS, ", stringify!([<$i$t>]), "::BITS);")]
                /// ```
                pub const BITS: u32 = <[<$i$t>]>::BITS;

                /// Returns the number of ones in the binary representation of `self`.
                ///
                /// # Examples
                ///
                /// Basic usage:
                ///
                /// ```
                /// use scrypto::math::Integer;
                ///
                #[doc = concat!("let n = Integer(0b01001100", stringify!([<$i$t>]), ");")]
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
                /// use scrypto::math::Integer;
                ///
                #[doc = concat!("assert_eq!(Integer(!0", stringify!([<$i$t>]), ").count_zeros(), 0);")]
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
                /// use scrypto::math::Integer;
                ///
                #[doc = concat!("let n = Integer(0b0101000", stringify!([<$i$t>]), ");")]
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
                /// use scrypto::math::Integer;
                ///
                /// let n: I128 = Integer(0x0123456789ABCDEF);
                /// let m: I128 = Integer(-0x76543210FEDCBA99);
                ///
                /// assert_eq!(n.rotate_left(32), m);
                /// ```
                #[inline]
                #[must_use = "this returns the result of the operation, \
                          without modifying the original"]
                pub const fn rotate_left(self, n: u32) -> Self {
                    Integer(self.0.rotate_left(n))
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
                /// use scrypto::math::Integer;
                ///
                /// let n: I128 = Integer(0x0123456789ABCDEF);
                /// let m: I128 = Integer(-0xFEDCBA987654322);
                ///
                /// assert_eq!(n.rotate_right(4), m);
                /// ```
                #[inline]
                #[must_use = "this returns the result of the operation, \
                          without modifying the original"]
                pub const fn rotate_right(self, n: u32) -> Self {
                    Integer(self.0.rotate_right(n))
                }

                /// Reverses the byte order of the integer.
                ///
                /// # Examples
                ///
                /// Basic usage:
                ///
                /// ```
                /// use scrypto::math::Integer;
                ///
                /// let n: I128 = Integer(0b0000000_01010101);
                /// assert_eq!(n, Integer(85));
                ///
                /// let m = n.swap_bytes();
                ///
                /// assert_eq!(m, Integer(0b01010101_00000000));
                /// assert_eq!(m, Integer(21760));
                /// ```
                #[inline]
                #[must_use = "this returns the result of the operation, \
                          without modifying the original"]
                pub const fn swap_bytes(self) -> Self {
                    Integer(self.0.swap_bytes())
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
                /// use scrypto::math::Integer;
                ///
                /// let n = Integer(0b0000000_01010101i16);
                /// assert_eq!(n, Integer(85));
                ///
                /// let m = n.reverse_bits();
                ///
                /// assert_eq!(m.0 as u16, 0b10101010_00000000);
                /// assert_eq!(m, Integer(-22016));
                /// ```
                #[must_use = "this returns the result of the operation, \
                          without modifying the original"]
                #[inline]
                pub const fn reverse_bits(self) -> Self {
                    Integer(self.0.reverse_bits())
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
                /// use scrypto::math::Integer;
                ///
                #[doc = concat!("let n = Integer(0x1A", stringify!([<$i$t>]), ");")]
                ///
                /// if cfg!(target_endian = "big") {
                #[doc = concat!("    assert_eq!(<I128>::from_be(n), n)")]
                /// } else {
                #[doc = concat!("    assert_eq!(<I128>::from_be(n), n.swap_bytes())")]
                /// }
                /// ```
                #[inline]
                #[must_use]
                pub const fn from_be(x: Self) -> Self {
                    Integer(<[<$i$t>]>::from_be(x.0))
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
                /// use scrypto::math::Integer;
                ///
                #[doc = concat!("let n = Integer(0x1A", stringify!([<$i$t>]), ");")]
                ///
                /// if cfg!(target_endian = "little") {
                #[doc = concat!("    assert_eq!(<I128>::from_le(n), n)")]
                /// } else {
                #[doc = concat!("    assert_eq!(<I128>::from_le(n), n.swap_bytes())")]
                /// }
                /// ```
                #[inline]
                #[must_use]
                pub const fn from_le(x: Self) -> Self {
                    Integer(<[<$i$t>]>::from_le(x.0))
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
                /// use scrypto::math::Integer;
                ///
                #[doc = concat!("let n = Integer(0x1A", stringify!([<$i$t>]), ");")]
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
                    Integer(self.0.to_be())
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
                /// use scrypto::math::Integer;
                ///
                #[doc = concat!("let n = Integer(0x1A", stringify!([<$i$t>]), ");")]
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
                    Integer(self.0.to_le())
                }

                /// Raises self to the power of `exp`, using exponentiation by squaring.
                ///
                #[inline]
                #[must_use = "this returns the result of the operation, \
                          without modifying the original"]
                pub fn pow(self, exp: u32) -> Self {
                    Integer(self.0.checked_pow(exp).unwrap())
                }
            }
        }
    )*)
}

checked_int_impl! { (I, i, 8), (I, i, 16), (I, i, 32), (I, i, 64), (I, i, 128) }
checked_int_impl! { (U, u, 8), (U, u, 16), (U, u, 32), (U, u, 64), (U, u, 128) }
