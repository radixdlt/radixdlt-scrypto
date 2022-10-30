use super::*;

pub trait PrimIntExt {
    type Output;
    fn rotate_left(self, other: Self) -> Self;
    fn rotate_right(self, other: Self) -> Self;
}

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
                    #[doc = "use scrypto::prelude::*;"]
                    ///
                    #[doc = "let n = " $t "::by(0b01001100u8);"]
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
                    /// Please note that this example is shared between large integer types.
                    /// Which explains why `I8` is used here.
                    ///
                    /// Basic usage:
                    ///
                    /// ```
                    #[doc = "use scrypto::prelude::*;"]
                    ///
                    #[doc = concat!("assert_eq!(", stringify!($t),
                        "::by(0i8).count_zeros(), ", stringify!(<$t>::BITS), ");")]
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
                    /// Please note that this example is shared between large integer types.
                    /// Which explains why `I8` is used here.
                    ///
                    /// Basic usage:
                    ///
                    /// ```
                    #[doc = "use scrypto::prelude::*;"]
                    ///
                    #[doc = "let n = I8::by(0b0101000u8);"]
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
                    /// Please note that this example is shared between large integer types.
                    /// Which explains why `I16` is used here.
                    ///
                    /// Basic usage:
                    ///
                    /// ```
                    #[doc = "use scrypto::prelude::*;"]
                    ///
                    #[doc = "let n: I16  = I16::by(0b0000000_01010101i16);"]
                    #[doc = "assert_eq!(n, I16::by(85));"]
                    ///
                    /// let m = n.swap_bytes();
                    ///
                    #[doc = "assert_eq!(m, I16::by(0b01010101_00000000i16));"]
                    #[doc = "assert_eq!(m, I16::by(21760i16));"]
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
                    /// Please note that this example is shared between large integer types.
                    /// Which explains why `I16` is used here.
                    ///
                    /// Basic usage:
                    ///
                    /// ```
                    #[doc = "use scrypto::prelude::*;"]
                    ///
                    #[doc = "let n: I16  = I16::by(0b0000000_01010101i16);"]
                    #[doc = "assert_eq!(n, I16::by(85i16));"]
                    ///
                    /// let m = n.reverse_bits();
                    ///
                    /// assert_eq!(m.0 as u16, 0b10101010_00000000);
                    #[doc = "assert_eq!(m, I16::by(-22016i16));"]
                    /// ```
                    #[must_use = "this returns the result of the operation, \
                          without modifying the original"]
                    #[inline]
                    pub fn reverse_bits(self) -> Self {
                        $t(self.0.into_iter().rev().map(|x| x.reverse_bits()).collect::<Vec<u8>>().try_into().unwrap())
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
                #[doc = "use scrypto::prelude::*;"]
                ///
                #[doc = "let n = " $t "::by(0b01001100u8);"]
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
                /// Please note that this example is shared between large integer types.
                /// Which explains why `I8` is used here.
                ///
                /// Basic usage:
                ///
                /// ```
                #[doc = "use scrypto::prelude::*;"]
                ///
                #[doc = "assert_eq!(I8::by(0i8).count_zeros(), 8);"]
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
                /// Please note that this example is shared between large integer types.
                /// Which explains why `I8` is used here.
                ///
                /// Basic usage:
                ///
                /// ```
                #[doc = "use scrypto::prelude::*;"]
                ///
                #[doc = "let n = I8::by(0b0101000u8);"]
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
                /// Please note that this example is shared between large integer types.
                /// Which explains why `I16` is used here.
                ///
                /// Basic usage:
                ///
                /// ```
                #[doc = "use scrypto::prelude::*;"]
                ///
                #[doc = "let n: I16  = I16::by(0b0000000_01010101i16);"]
                #[doc = "assert_eq!(n, I16::by(85));"]
                ///
                /// let m = n.swap_bytes();
                ///
                #[doc = "assert_eq!(m, I16::by(0b01010101_00000000i16));"]
                #[doc = "assert_eq!(m, I16::by(21760i16));"]
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
                /// Please note that this example is shared between large integer types.
                /// Which explains why `I16` is used here.
                ///
                /// Basic usage:
                ///
                /// ```
                #[doc = "use scrypto::prelude::*;"]
                ///
                #[doc = "let n: I16  = I16::by(0b0000000_01010101i16);"]
                #[doc = "assert_eq!(n, I16::by(85i16));"]
                ///
                /// let m = n.reverse_bits();
                ///
                /// assert_eq!(m.0 as u16, 0b10101010_00000000);
                #[doc = "assert_eq!(m, I16::by(-22016i16));"]
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
                /// Please note that this example is shared between large integer types.
                /// Which explains why `I8` is used here.
                ///
                /// Basic usage:
                ///
                /// ```
                #[doc = "use scrypto::prelude::*;"]
                ///
                #[doc = "let n = " $t "::by(0x1Ai8);"]
                ///
                /// if cfg!(target_endian = "big") {
                #[doc = "    assert_eq!(" $t "::from_be(n), n)"]
                /// } else {
                #[doc = "    assert_eq!(" $t "::from_be(n), n.swap_bytes())"]
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
                /// Please note that this example is shared between large integer types.
                /// Which explains why `I8` is used here.
                ///
                /// Basic usage:
                ///
                /// ```
                #[doc = "use scrypto::prelude::*;"]
                ///
                #[doc = "let n = " $t "::by(0x1Ai8);"]
                ///
                /// if cfg!(target_endian = "little") {
                #[doc = "    assert_eq!(" $t "::from_le(n), n)"]
                /// } else {
                #[doc = "    assert_eq!(" $t "::from_le(n), n.swap_bytes())"]
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
                /// Please note that this example is shared between large integer types.
                /// Which explains why `I8` is used here.
                ///
                /// Basic usage:
                ///
                /// ```
                #[doc = "use scrypto::prelude::*;"]
                ///
                #[doc = "let n = " $t "::by(0x1Ai8);"]
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
                /// Please note that this example is shared between large integer types.
                /// Which explains why `I8` is used here.
                ///
                /// Basic usage:
                ///
                /// ```
                #[doc = "use scrypto::prelude::*;"]
                ///
                #[doc = "let n = " $t "::by(0x1Ai8);"]
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
        ($($t:ty),*) => {
            paste! {
                $(
                    impl BitXor for $t {
                        type Output = Self;

                        #[inline]
                        fn bitxor(self, other: Self) -> $t {
                            BigInt::from(self).bitxor(&BigInt::from(other)).try_into().unwrap()
                        }
                    }

                    impl BitXorAssign for $t {
                        #[inline]
                        fn bitxor_assign(&mut self, other: Self) {
                            *self = (*self ^ other).try_into().unwrap();
                        }
                    }

                    impl BitOr for $t {
                        type Output = Self;

                        #[inline]
                        fn bitor(self, other: Self) -> $t {
                            BigInt::from(self).bitor(&BigInt::from(other)).try_into().unwrap()
                        }
                    }

                    impl BitOrAssign for $t {
                        #[inline]
                        fn bitor_assign(&mut self, other: Self) {
                            *self = (*self | other).try_into().unwrap();
                        }
                    }

                    impl BitAnd for $t {
                        type Output = Self;

                        #[inline]
                        fn bitand(self, other: Self) -> $t {
                            BigInt::from(self).bitand(&BigInt::from(other)).try_into().unwrap()
                        }
                    }

                    impl BitAndAssign for $t {
                        #[inline]
                        fn bitand_assign(&mut self, other: Self) {
                            *self = (*self & other).try_into().unwrap();
                        }
                    }

                    impl Shl for $t {
                        type Output = Self;

                        #[inline]
                        fn shl(self, other: Self) -> $t {
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


                    impl ShlAssign for $t {
                        #[inline]
                        fn shl_assign(&mut self, other: Self) {
                            *self = *self << other;
                        }
                    }

                    impl Shr for $t {
                        type Output = Self;

                        #[inline]
                        fn shr(self, other: Self) -> $t {
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

                    impl ShrAssign for $t {
                        #[inline]
                        fn shr_assign(&mut self, other: Self) {
                            *self = *self >> other;
                        }
                    }
                    impl PrimIntExt for $t {
                        type Output = Self;
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
                        fn rotate_left(self, other: Self) -> Self {
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
                        #[inline]
                        #[must_use = "this returns the result of the operation, \
                              without modifying the original"]
                        fn rotate_right(self, other: Self) -> Self {
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
            checked_impl! { $t }
        )*
    }
}

checked_impl_all! { U8, U16, U32, U64, U128, U256, U384, U512, I8, I16, I32, I64, I128, I256, I384, I512 }
