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

                impl Shl<u32> for $t {
                    type Output = Self;

                    #[inline]
                    fn shl(self, other: u32) -> Self {
                        Self(self.0.checked_shl(other).expect("Overflow"))
                    }
                }


                impl ShlAssign<u32> for $t {
                    #[inline]
                    fn shl_assign(&mut self, other: u32) {
                        self.0 = self.0.checked_shl(other).expect("Overflow");
                    }
                }

                impl Shr<u32> for $t {
                    type Output = Self;

                    #[inline]
                    fn shr(self, other: u32) -> $t {
                        Self(self.0.checked_shr(other).expect("Overflow"))
                    }
                }

                impl ShrAssign<u32> for $t {
                    #[inline]
                    fn shr_assign(&mut self, other: u32) {
                        self.0 = self.0.checked_shr(other).expect("Overflow");
                    }
                }
            }
        )*
    }
}
impl_bits! { I192, BInt::<3> }
impl_bits! { I256, BInt::<4> }
impl_bits! { I320, BInt::<5> }
impl_bits! { I384, BInt::<6> }
impl_bits! { I448, BInt::<7> }
impl_bits! { I512, BInt::<8> }
impl_bits! { I768, BInt::<12> }
impl_bits! { U192, BUint::<3> }
impl_bits! { U256, BUint::<4> }
impl_bits! { U320, BUint::<5> }
impl_bits! { U384, BUint::<6> }
impl_bits! { U448, BUint::<7> }
impl_bits! { U512, BUint::<8> }
impl_bits! { U768, BUint::<12> }
