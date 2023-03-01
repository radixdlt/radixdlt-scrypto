use super::*;

macro_rules! impl_bits {
    ($($t:ident, $wrapped:ty),*) => {
        $(
            paste! {
                impl $t {
                    /// Returns the number of ones in the binary representation of `self`.
                    ///
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
                    #[inline]
                    #[must_use = "this returns the result of the operation, \
                          without modifying the original"]
                    pub const fn count_zeros(self) -> u32 {
                        self.0.count_zeros()
                    }

                    /// Returns the number of trailing zeros in the binary representation of `self`.
                    ///
                    #[inline]
                    #[must_use = "this returns the result of the operation, \
                          without modifying the original"]
                    pub const fn trailing_zeros(self) -> u32 {
                        self.0.trailing_zeros()
                    }

                    /// Reverses the byte order of the integer.
                    ///
                    #[inline]
                    #[must_use = "this returns the result of the operation, \
                          without modifying the original"]
                    pub const fn swap_bytes(self) -> Self {
                        Self(self.0.swap_bytes())
                    }

                    /// Reverses the bit pattern of the integer.
                    ///
                    #[must_use = "this returns the result of the operation, \
                          without modifying the original"]
                    #[inline]
                    pub const fn reverse_bits(self) -> Self {
                        Self(self.0.reverse_bits())
                    }

                    /// Returns the number of leading zeros in the binary representation of `self`.
                    ///
                    #[inline]
                    #[must_use = "this returns the result of the operation, \
                          without modifying the original"]
                    pub const fn leading_zeros(self) -> u32 {
                        self.0.leading_zeros()
                    }

                    /// Shifts the bits to the left by a specified amount, `n`,
                    /// wrapping the truncated bits to the end of the resulting
                    /// integer.
                    ///
                    /// Please note this isn't the same operation as the `<<` shifting
                    /// operator! This method can not overflow as opposed to '<<'.
                    ///
                    /// Please note that this example is shared between integer types.
                    /// Which explains why `I128` is used here.
                    ///
                    #[inline]
                    #[must_use = "this returns the result of the operation, \
                          without modifying the original"]
                    pub fn rotate_left(self, other: Self) -> Self {
                        Self(self.0.rotate_left(u32::try_from(other).unwrap()))
                    }

                    /// Shifts the bits to the right by a specified amount, `n`,
                    /// wrapping the truncated bits to the beginning of the resulting
                    /// integer.
                    ///
                    /// Please note this isn't the same operation as the `>>` shifting
                    /// operator! This method can not overflow as opposed to '>>'.
                    ///
                    #[inline]
                    #[must_use = "this returns the result of the operation, \
                          without modifying the original"]
                    pub fn rotate_right(self, other: Self) -> Self {
                        Self(self.0.rotate_right(u32::try_from(other).unwrap()))
                    }

                    /// Converts an integer from big endian to the target's endianness.
                    ///
                    /// On big endian this is a no-op. On little endian the bytes are
                    /// swapped.
                    ///
                    #[inline]
                    #[must_use]
                    pub const fn from_be(x: Self) -> Self {
                        if cfg!(target_endian = "big") {
                            x
                        } else {
                            Self(<$wrapped>::from_be(x.0))
                        }
                    }

                    /// Converts an integer from little endian to the target's endianness.
                    ///
                    /// On little endian this is a no-op. On big endian the bytes are
                    /// swapped.
                    ///
                    #[inline]
                    #[must_use]
                    pub const fn from_le(x: Self) -> Self {
                        if cfg!(target_endian = "big") {
                            Self(<$wrapped>::from_be(x.0))
                        } else {
                            x
                        }
                    }

                    /// Converts `self` to big endian from the target's endianness.
                    ///
                    /// On big endian this is a no-op. On little endian the bytes are
                    /// swapped.
                    ///
                    #[inline]
                    #[must_use = "this returns the result of the operation, \
                    without modifying the original"]
                    pub const fn to_be(self) -> Self {
                        if cfg!(target_endian = "big") {
                            self
                        } else {
                            Self(self.0.to_be())
                        }
                    }

                    /// Converts `self` to little endian from the target's endianness.
                    ///
                    /// On little endian this is a no-op. On big endian the bytes are
                    /// swapped.
                    ///
                    #[inline]
                    #[must_use = "this returns the result of the operation, \
                    without modifying the original"]
                    pub const fn to_le(self) -> Self {
                        if cfg!(target_endian = "big") {
                            Self(self.0.to_le())
                        } else {
                            self
                        }
                    }
                }

                impl BitXor for $t {
                    type Output = Self;

                    #[inline]
                    fn bitxor(self, other: Self) -> Self {
                        Self(self.0 ^ other.0)
                    }
                }

                impl BitXorAssign for $t {
                    #[inline]
                    fn bitxor_assign(&mut self, other: Self) {
                        self.0 ^= other.0
                    }
                }

                impl BitOr for $t {
                    type Output = Self;

                    #[inline]
                    fn bitor(self, other: Self) -> Self {
                        Self(self.0 | other.0)
                    }
                }

                impl BitOrAssign for $t {
                    #[inline]
                    fn bitor_assign(&mut self, other: Self) {
                        self.0 |= other.0
                    }
                }

                impl BitAnd for $t {
                    type Output = Self;

                    #[inline]
                    fn bitand(self, other: Self) -> Self {
                        Self(self.0 & other.0)
                    }
                }

                impl BitAndAssign for $t {
                    #[inline]
                    fn bitand_assign(&mut self, other: Self) {
                        self.0 &= other.0
                    }
                }

                impl Shl for $t {
                    type Output = Self;

                    #[inline]
                    fn shl(self, other: Self) -> Self {
                        Self(self.0 << other.0)
                    }
                }


                impl ShlAssign for $t {
                    #[inline]
                    fn shl_assign(&mut self, other: Self) {
                        self.0 <<= other.0;
                    }
                }

                impl Shr for $t {
                    type Output = Self;

                    #[inline]
                    fn shr(self, other: Self) -> $t {
                        Self(self.0 >> other.0)
                    }
                }

                impl ShrAssign for $t {
                    #[inline]
                    fn shr_assign(&mut self, other: Self) {
                        self.0 >>= other.0;
                    }
                }
            }
        )*
    }
}
impl_bits! { BnumI256, BInt::<4> }
impl_bits! { BnumI384, BInt::<6> }
impl_bits! { BnumI512, BInt::<8> }
impl_bits! { BnumI768, BInt::<12> }
impl_bits! { BnumU256, BUint::<4> }
impl_bits! { BnumU384, BUint::<6> }
impl_bits! { BnumU512, BUint::<8> }
impl_bits! { BnumU768, BUint::<12> }
