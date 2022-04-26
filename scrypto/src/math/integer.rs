//! Definitions of `Integer<T>`.

use core::fmt;
use core::ops::{Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign};
use core::ops::{BitXor, BitXorAssign, Div, DivAssign};
use core::ops::{Mul, MulAssign, Neg, Not, Rem, RemAssign};
use core::ops::{Shl, ShlAssign, Shr, ShrAssign, Sub, SubAssign};
use forward_ref::*;

/// Provides safe arithmetic on `T`.
///
/// Operations like `+`, '-', '*', or '/' sometimes produce overflow 
/// which is detected and results in a panic. This way unexpected 
/// results are not silently produced.
/// 
/// Integer arithmetic can be achieved either through methods like
/// `checked_add`, or through the `Integer<T>` type, which says that
/// all standard arithmetic operations on the underlying value are
/// intended to have checked semantics.
///
/// The underlying value can be retrieved through the `.0` index of the
/// `Integer` tuple.
///
/// # Layout
///
/// `Integer<T>` is guaranteed to have the same layout and ABI as `T`.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default, Hash)]
#[repr(transparent)]
pub struct Integer<T>(pub T);

impl<T: fmt::Debug> fmt::Debug for Integer<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: fmt::Display> fmt::Display for Integer<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: fmt::Binary> fmt::Binary for Integer<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: fmt::Octal> fmt::Octal for Integer<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: fmt::LowerHex> fmt::LowerHex for Integer<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: fmt::UpperHex> fmt::UpperHex for Integer<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[allow(unused_macros)]
macro_rules! sh_impl_signed {
    ($t:ident, $f:ident) => {
        impl Shl<$f> for Integer<$t> {
            type Output = Integer<$t>;

            #[inline]
            fn shl(self, other: $f) -> Integer<$t> {
                if other < 0 {
                    Integer(self.0.checked_shr(-other as u32))
                } else {
                    Integer(self.0.checked_shl(other as u32))
                }
            }
        }
        forward_ref_binop! { impl Shl, shl for Integer<$t>, $f }

        impl ShlAssign<$f> for Integer<$t> {
            #[inline]
            fn shl_assign(&mut self, other: $f) {
                *self = *self << other;
            }
        }
        forward_ref_op_assign! { impl ShlAssign, shl_assign for Integer<$t>, $f }

        impl Shr<$f> for Integer<$t> {
            type Output = Integer<$t>;

            #[inline]
            fn shr(self, other: $f) -> Integer<$t> {
                if other < 0 {
                    Integer(self.0.checked_shl(-other as u32))
                } else {
                    Integer(self.0.checked_shr(other as u32))
                }
            }
        }
        forward_ref_binop! { impl Shr, shr for Integer<$t>, $f }

        impl ShrAssign<$f> for Integer<$t> {
            #[inline]
            fn shr_assign(&mut self, other: $f) {
                *self = *self >> other;
            }
        }
        forward_ref_op_assign! { impl ShrAssign, shr_assign for Integer<$t>, $f }
    };
}

macro_rules! sh_impl_unsigned {
    ($t:ident, $f:ident) => {
        impl Shl<$f> for Integer<$t> {
            type Output = Integer<$t>;

            #[inline]
            fn shl(self, other: $f) -> Integer<$t> {
                Integer(self.0.checked_shl(other as u32).unwrap())
            }
        }
        forward_ref_binop! { impl Shl, shl for Integer<$t>, $f }

        impl ShlAssign<$f> for Integer<$t> {
            #[inline]
            fn shl_assign(&mut self, other: $f) {
                *self = *self << other;
            }
        }
        forward_ref_op_assign! { impl ShlAssign, shl_assign for Integer<$t>, $f }

        impl Shr<$f> for Integer<$t> {
            type Output = Integer<$t>;

            #[inline]
            fn shr(self, other: $f) -> Integer<$t> {
                Integer(self.0.checked_shr(other as u32).unwrap())
            }
        }
        forward_ref_binop! { impl Shr, shr for Integer<$t>, $f }

        impl ShrAssign<$f> for Integer<$t> {
            #[inline]
            fn shr_assign(&mut self, other: $f) {
                *self = *self >> other;
            }
        }
        forward_ref_op_assign! { impl ShrAssign, shr_assign for Integer<$t>, $f }
    };
}

// FIXME (#23545): uncomment the remaining impls
macro_rules! sh_impl_all {
    ($($t:ident)*) => ($(
        //sh_impl_unsigned! { $t, u8 }
        //sh_impl_unsigned! { $t, u16 }
        //sh_impl_unsigned! { $t, u32 }
        //sh_impl_unsigned! { $t, u64 }
        //sh_impl_unsigned! { $t, u128 }
        sh_impl_unsigned! { $t, usize }

        //sh_impl_signed! { $t, i8 }
        //sh_impl_signed! { $t, i16 }
        //sh_impl_signed! { $t, i32 }
        //sh_impl_signed! { $t, i64 }
        //sh_impl_signed! { $t, i128 }
        //sh_impl_signed! { $t, isize }
    )*)
}

sh_impl_all! { u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize }

// FIXME(30524): impl Op<T> for Integer<T>, impl OpAssign<T> for Integer<T>
macro_rules! checked_impl {
    ($($t:ty)*) => ($(
        impl Add for Integer<$t> {
            type Output = Integer<$t>;

            #[inline]
            fn add(self, other: Integer<$t>) -> Integer<$t> {
                Integer(self.0.checked_add(other.0).unwrap())
            }
        }
        forward_ref_binop! { impl Add, add for Integer<$t>, Integer<$t> }

        impl AddAssign for Integer<$t> {
            #[inline]
            fn add_assign(&mut self, other: Integer<$t>) {
                *self = *self + other;
            }
        }
        forward_ref_op_assign! { impl AddAssign, add_assign for Integer<$t>, Integer<$t> }

        impl AddAssign<$t> for Integer<$t> {
            #[inline]
            fn add_assign(&mut self, other: $t) {
                *self = *self + Integer(other);
            }
        }
        forward_ref_op_assign! { impl AddAssign, add_assign for Integer<$t>, $t }

        impl Sub for Integer<$t> {
            type Output = Integer<$t>;

            #[inline]
            fn sub(self, other: Integer<$t>) -> Integer<$t> {
                Integer(self.0.checked_sub(other.0).unwrap())
            }
        }
        forward_ref_binop! { impl Sub, sub for Integer<$t>, Integer<$t> }

        impl SubAssign for Integer<$t> {
            #[inline]
            fn sub_assign(&mut self, other: Integer<$t>) {
                *self = *self - other;
            }
        }
        forward_ref_op_assign! { impl SubAssign, sub_assign for Integer<$t>, Integer<$t> }

        impl SubAssign<$t> for Integer<$t> {
            #[inline]
            fn sub_assign(&mut self, other: $t) {
                *self = *self - Integer(other);
            }
        }
        forward_ref_op_assign! { impl SubAssign, sub_assign for Integer<$t>, $t }

        impl Mul for Integer<$t> {
            type Output = Integer<$t>;

            #[inline]
            fn mul(self, other: Integer<$t>) -> Integer<$t> {
                Integer(self.0.checked_mul(other.0).unwrap())
            }
        }
        forward_ref_binop! { impl Mul, mul for Integer<$t>, Integer<$t> }

        impl MulAssign for Integer<$t> {
            #[inline]
            fn mul_assign(&mut self, other: Integer<$t>) {
                *self = *self * other;
            }
        }
        forward_ref_op_assign! { impl MulAssign, mul_assign for Integer<$t>, Integer<$t> }

        impl MulAssign<$t> for Integer<$t> {
            #[inline]
            fn mul_assign(&mut self, other: $t) {
                *self = *self * Integer(other);
            }
        }
        forward_ref_op_assign! { impl MulAssign, mul_assign for Integer<$t>, $t }

        impl Div for Integer<$t> {
            type Output = Integer<$t>;

            #[inline]
            fn div(self, other: Integer<$t>) -> Integer<$t> {
                Integer(self.0.checked_div(other.0).unwrap())
            }
        }
        forward_ref_binop! { impl Div, div for Integer<$t>, Integer<$t> }

        impl DivAssign for Integer<$t> {
            #[inline]
            fn div_assign(&mut self, other: Integer<$t>) {
                *self = *self / other;
            }
        }
        forward_ref_op_assign! { impl DivAssign, div_assign for Integer<$t>, Integer<$t> }

        impl DivAssign<$t> for Integer<$t> {
            #[inline]
            fn div_assign(&mut self, other: $t) {
                *self = *self / Integer(other);
            }
        }
        forward_ref_op_assign! { impl DivAssign, div_assign for Integer<$t>, $t }

        impl Rem for Integer<$t> {
            type Output = Integer<$t>;

            #[inline]
            fn rem(self, other: Integer<$t>) -> Integer<$t> {
                Integer(self.0.checked_rem(other.0).unwrap())
            }
        }
        forward_ref_binop! { impl Rem, rem for Integer<$t>, Integer<$t> }

        impl RemAssign for Integer<$t> {
            #[inline]
            fn rem_assign(&mut self, other: Integer<$t>) {
                *self = *self % other;
            }
        }
        forward_ref_op_assign! { impl RemAssign, rem_assign for Integer<$t>, Integer<$t> }

        impl RemAssign<$t> for Integer<$t> {
            #[inline]
            fn rem_assign(&mut self, other: $t) {
                *self = *self % Integer(other);
            }
        }
        forward_ref_op_assign! { impl RemAssign, rem_assign for Integer<$t>, $t }

        impl Not for Integer<$t> {
            type Output = Integer<$t>;

            #[inline]
            fn not(self) -> Integer<$t> {
                Integer(!self.0)
            }
        }
        forward_ref_unop! { impl Not, not for Integer<$t> }

        impl BitXor for Integer<$t> {
            type Output = Integer<$t>;

            #[inline]
            fn bitxor(self, other: Integer<$t>) -> Integer<$t> {
                Integer(self.0 ^ other.0)
            }
        }
        forward_ref_binop! { impl BitXor, bitxor for Integer<$t>, Integer<$t> }

        impl BitXorAssign for Integer<$t> {
            #[inline]
            fn bitxor_assign(&mut self, other: Integer<$t>) {
                *self = *self ^ other;
            }
        }
        forward_ref_op_assign! { impl BitXorAssign, bitxor_assign for Integer<$t>, Integer<$t> }

        impl BitXorAssign<$t> for Integer<$t> {
            #[inline]
            fn bitxor_assign(&mut self, other: $t) {
                *self = *self ^ Integer(other);
            }
        }
        forward_ref_op_assign! { impl BitXorAssign, bitxor_assign for Integer<$t>, $t }

        impl BitOr for Integer<$t> {
            type Output = Integer<$t>;

            #[inline]
            fn bitor(self, other: Integer<$t>) -> Integer<$t> {
                Integer(self.0 | other.0)
            }
        }
        forward_ref_binop! { impl BitOr, bitor for Integer<$t>, Integer<$t> }

        impl BitOrAssign for Integer<$t> {
            #[inline]
            fn bitor_assign(&mut self, other: Integer<$t>) {
                *self = *self | other;
            }
        }
        forward_ref_op_assign! { impl BitOrAssign, bitor_assign for Integer<$t>, Integer<$t> }

        impl BitOrAssign<$t> for Integer<$t> {
            #[inline]
            fn bitor_assign(&mut self, other: $t) {
                *self = *self | Integer(other);
            }
        }
        forward_ref_op_assign! { impl BitOrAssign, bitor_assign for Integer<$t>, $t }

        impl BitAnd for Integer<$t> {
            type Output = Integer<$t>;

            #[inline]
            fn bitand(self, other: Integer<$t>) -> Integer<$t> {
                Integer(self.0 & other.0)
            }
        }
        forward_ref_binop! { impl BitAnd, bitand for Integer<$t>, Integer<$t> }

        impl BitAndAssign for Integer<$t> {
            #[inline]
            fn bitand_assign(&mut self, other: Integer<$t>) {
                *self = *self & other;
            }
        }
        forward_ref_op_assign! { impl BitAndAssign, bitand_assign for Integer<$t>, Integer<$t> }

        impl BitAndAssign<$t> for Integer<$t> {
            #[inline]
            fn bitand_assign(&mut self, other: $t) {
                *self = *self & Integer(other);
            }
        }
        forward_ref_op_assign! { impl BitAndAssign, bitand_assign for Integer<$t>, $t }

        impl Neg for Integer<$t> {
            type Output = Self;
            #[inline]
            fn neg(self) -> Self {
                Integer(0) - self
            }
        }
        forward_ref_unop! { impl Neg, neg for Integer<$t> }

    )*)
}

checked_impl! { usize u8 u16 u32 u64 u128 isize i8 i16 i32 i64 i128 }

macro_rules! checked_int_impl {
    ($($t:ty)*) => ($(
        impl Integer<$t> {
            /// Returns the smallest value that can be represented by this integer type.
            ///
            /// # Examples
            ///
            /// Basic usage:
            ///
            /// ```
            /// use scrypto::math::Integer;
            ///
            #[doc = concat!("assert_eq!(<Integer<", stringify!($t), ">>::MIN, Integer(", stringify!($t), "::MIN));")]
            /// ```
            pub const MIN: Self = Self(<$t>::MIN);

            /// Returns the largest value that can be represented by this integer type.
            ///
            /// # Examples
            ///
            /// Basic usage:
            ///
            /// ```
            /// use scrypto::math::Integer;
            ///
            #[doc = concat!("assert_eq!(<Integer<", stringify!($t), ">>::MAX, Integer(", stringify!($t), "::MAX));")]
            /// ```
            pub const MAX: Self = Self(<$t>::MAX);

            /// Returns the size of this integer type in bits.
            ///
            /// # Examples
            ///
            /// Basic usage:
            ///
            /// ```
            /// use scrypto::math::Integer;
            ///
            #[doc = concat!("assert_eq!(<Integer<", stringify!($t), ">>::BITS, ", stringify!($t), "::BITS);")]
            /// ```
            pub const BITS: u32 = <$t>::BITS;

            /// Returns the number of ones in the binary representation of `self`.
            ///
            /// # Examples
            ///
            /// Basic usage:
            ///
            /// ```
            /// use scrypto::math::Integer;
            ///
            #[doc = concat!("let n = Integer(0b01001100", stringify!($t), ");")]
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
            #[doc = concat!("assert_eq!(Integer(!0", stringify!($t), ").count_zeros(), 0);")]
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
            #[doc = concat!("let n = Integer(0b0101000", stringify!($t), ");")]
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
            /// let n: Integer<i64> = Integer(0x0123456789ABCDEF);
            /// let m: Integer<i64> = Integer(-0x76543210FEDCBA99);
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
            /// let n: Integer<i64> = Integer(0x0123456789ABCDEF);
            /// let m: Integer<i64> = Integer(-0xFEDCBA987654322);
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
            /// let n: Integer<i16> = Integer(0b0000000_01010101);
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
            #[doc = concat!("let n = Integer(0x1A", stringify!($t), ");")]
            ///
            /// if cfg!(target_endian = "big") {
            #[doc = concat!("    assert_eq!(<Integer<", stringify!($t), ">>::from_be(n), n)")]
            /// } else {
            #[doc = concat!("    assert_eq!(<Integer<", stringify!($t), ">>::from_be(n), n.swap_bytes())")]
            /// }
            /// ```
            #[inline]
            #[must_use]
            pub const fn from_be(x: Self) -> Self {
                Integer(<$t>::from_be(x.0))
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
            #[doc = concat!("let n = Integer(0x1A", stringify!($t), ");")]
            ///
            /// if cfg!(target_endian = "little") {
            #[doc = concat!("    assert_eq!(<Integer<", stringify!($t), ">>::from_le(n), n)")]
            /// } else {
            #[doc = concat!("    assert_eq!(<Integer<", stringify!($t), ">>::from_le(n), n.swap_bytes())")]
            /// }
            /// ```
            #[inline]
            #[must_use]
            pub const fn from_le(x: Self) -> Self {
                Integer(<$t>::from_le(x.0))
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
            #[doc = concat!("let n = Integer(0x1A", stringify!($t), ");")]
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
            #[doc = concat!("let n = Integer(0x1A", stringify!($t), ");")]
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
    )*)
}

checked_int_impl! { usize u8 u16 u32 u64 u128 isize i8 i16 i32 i64 i128 }

macro_rules! checked_int_impl_signed {
    ($($t:ty)*) => ($(
        impl Integer<$t> {
            /// Returns the number of leading zeros in the binary representation of `self`.
            ///
            /// # Examples
            ///
            /// Basic usage:
            ///
            /// ```
            /// use scrypto::math::Integer;
            ///
            #[doc = concat!("let n = Integer(", stringify!($t), "::MAX) >> 2;")]
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
            pub fn abs(self) -> Integer<$t> {
                Integer(self.0.checked_abs().unwrap())
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
            /// use scrypto::math::Integer;
            ///
            #[doc = concat!("assert_eq!(Integer(10", stringify!($t), ").signum(), Integer(1));")]
            #[doc = concat!("assert_eq!(Integer(0", stringify!($t), ").signum(), Integer(0));")]
            #[doc = concat!("assert_eq!(Integer(-10", stringify!($t), ").signum(), Integer(-1));")]
            /// ```
            #[inline]
            #[must_use = "this returns the result of the operation, \
                          without modifying the original"]
            pub fn signum(self) -> Integer<$t> {
                Integer(self.0.signum())
            }

            /// Returns `true` if `self` is positive and `false` if the number is zero or
            /// negative.
            ///
            /// # Examples
            ///
            /// Basic usage:
            ///
            /// ```
            /// use scrypto::math::Integer;
            ///
            #[doc = concat!("assert!(Integer(10", stringify!($t), ").is_positive());")]
            #[doc = concat!("assert!(!Integer(-10", stringify!($t), ").is_positive());")]
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
            /// use scrypto::math::Integer;
            ///
            #[doc = concat!("assert!(Integer(-10", stringify!($t), ").is_negative());")]
            #[doc = concat!("assert!(!Integer(10", stringify!($t), ").is_negative());")]
            /// ```
            #[must_use]
            #[inline]
            pub const fn is_negative(self) -> bool {
                self.0.is_negative()
            }
        }
    )*)
}

checked_int_impl_signed! { isize i8 i16 i32 i64 i128 }

macro_rules! checked_int_impl_unsigned {
    ($($t:ty)*) => ($(
        impl Integer<$t> {
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
                Integer(self.0.checked_next_power_of_two().unwrap())
            }
        }
    )*)
}

checked_int_impl_unsigned! { usize u8 u16 u32 u64 u128 }

#[cfg(test)]
mod tests {
    use super::*;
    
    macro_rules! test_impl {
        ($($t:tt)*) => ($(


                paste::item! {
                    #[test]
                    #[should_panic]
                    fn [<test_add_overflow$t>]() {
                        let a = Integer(<$t>::MAX) + Integer(1 as $t); // panics on overflow
                        assert_eq!(a , Integer(<$t>::MAX));
                    }

                    #[test]
                    #[should_panic]
                    fn  [<test_sub_overflow$t>]() {
                        let _ = Integer(<$t>::MIN) - Integer(1 as $t); // panics because of overflow
                    }

                    #[test]
                    #[should_panic]
                    fn  [<test_mul_overflow$t>]() {
                        let _ = Integer(<$t>::MAX) * Integer(2 as $t); // panics because of overflow
                    }

                    #[test]
                    #[should_panic]
                    fn  [<test_div_overflow$t>]() {
                        let _ = Integer(<$t>::MIN) / Integer(0 as $t); // panics because of division by zero
                    }

                    #[test]
                    #[should_panic]
                    fn  [<test_rem_overflow$t>]() {
                        let _ = Integer(<$t>::MIN) % Integer(0); // panics because of division by zero
                    }

                    #[test]
                    #[should_panic]
                    fn  [<test_shl_overflow$t>]() {
                        let _ = Integer(<$t>::MAX) << (($t::BITS + 1) as usize);  // panics because of overflow
                    }

                    #[test]
                    #[should_panic]
                    fn  [<test_shr_overflow$t>]() {
                        let _ = Integer(<$t>::MIN) >> (($t::BITS + 1) as usize);  // panics because of overflow
                    }

                    #[test]
                    #[should_panic]
                    fn  [<test_shl_overflow_neg$t>]() {
                        let _ = Integer(<$t>::MIN) << (($t::BITS + 1) as usize);  // panics because of overflow
                    }

                    #[test]
                    #[should_panic]
                    fn  [<test_shr_overflow_neg$t>]() {
                        let _ = Integer(<$t>::MIN) >> (($t::BITS + 1) as usize);  // panics because of overflow
                    }

                    #[test]
                    #[should_panic]
                    fn  [<test_pow_overflow$t>]() {
                        let _ = Integer(<$t>::MAX).pow(2u32);          // panics because of overflow
                    }
                }
                )*)
    }   
    test_impl! { usize u8 u16 u32 u64 u128 isize i8 i16 i32 i64 i128 }
}
