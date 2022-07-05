use super::*;
use paste::paste;

macro_rules! checked_int_impl_large {
    ($($t:ident),*) => {
        paste! {
            $(
                impl $t {
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
                }
            )*
        }
    }
}

checked_int_impl_large! { U256, U384, U512, I256, I384, I512 }

macro_rules! checked_int_impl_small {
    ($($t:ident),*) => {$(
        paste! {
            impl $t {

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

                /// Returns the number of leading zeros in the binary representation of `self`.
                ///
                #[inline]
                #[must_use = "this returns the result of the operation, \
                          without modifying the original"]
                pub fn leading_zeros(self) -> u32 {
                    self.0.leading_zeros()
                }
            }
        }
        )*
    }
}

checked_int_impl_small! { I8, I16, I32, I64, I128, U8, U16, U32, U64, U128 }

macro_rules! checked_impl {
        ($t:ty, ($($o:ty),*)) => {
            paste! {
                $(
                    impl BitXor<$o> for $t {
                        type Output = $t;

                        #[inline]
                        fn bitxor(self, other: $o) -> $t {
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
                        type Output = $t;

                        #[inline]
                        fn bitor(self, other: $o) -> $t {
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
                        type Output = $t;

                        #[inline]
                        fn bitand(self, other: $o) -> $t {
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
                            let shift = BigInt::from(other).to_i64().unwrap();
                            if <$t>::MIN == <$t>::zero() {
                                let len: usize = to_shift
                                    .clone()
                                    .shl(shift)
                                    .to_bytes_le()
                                    .1
                                    .len()
                                    .min((<$t>::BITS / 8) as usize);
                                BigInt::from_bytes_le(
                                    Sign::Plus,
                                    to_shift.shl(shift)
                                    .to_bytes_le().1[..len]
                                    .into()
                                )
                                    .try_into()
                                    .unwrap()
                            } else {
                                let len: usize = to_shift
                                    .clone()
                                    .shl(shift)
                                    .to_signed_bytes_le()
                                    .len()
                                    .min((<$t>::BITS / 8) as usize);
                                BigInt::from_signed_bytes_le(
                                    to_shift
                                    .shl(shift)
                                    .to_bytes_le()
                                    .1[..len]
                                    .into()
                                )
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

macro_rules! checked_impl_all {
    ($($t:ident),*) => {
        $(
            checked_impl! { $t, (u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, U8, U16, U32, U64, U128, U256, U384, U512, I8, I16, I32, I64, I128, I256, I384, I512)}
        )*
    }
}

checked_impl_all! { U8, U16, U32, U64, U128, U256, U384, U512, I8, I16, I32, I64, I128, I256, I384, I512 }
