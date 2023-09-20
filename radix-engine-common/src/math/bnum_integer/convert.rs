use super::*;
use bnum::cast::CastFrom;

macro_rules! impl_from_primitive {
    ($t:ident, $wrapped:ty, ($($type:ident),*)) => {
        paste! {
            impl FromPrimitive for $t {
                $(
                    fn [< from_$type >](n: [<$type>]) -> Option<Self> {
                        <$wrapped>::try_from(n)
                            .map(|val| Self(val))
                            .ok()
                    }
                )*
            }
        }
    }
}
macro_rules! impl_to_primitive {
    ($t:ident, $wrapped:ty, ($($type:ident),*)) => {
        paste! {
            impl ToPrimitive for $t {
                $(
                    fn [< to_$type >](&self) -> Option<[<$type>]> {
                        [<$type>]::try_from(self.0).ok()
                    }
                )*
            }
        }
    }
}
impl_from_primitive! { I192, BInt::<3>, (u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize) }
impl_from_primitive! { I256, BInt::<4>, (u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize) }
impl_from_primitive! { I320, BInt::<5>, (u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize) }
impl_from_primitive! { I384, BInt::<6>, (u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize) }
impl_from_primitive! { I448, BInt::<7>, (u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize) }
impl_from_primitive! { I512, BInt::<8>, (u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize) }
impl_from_primitive! { I768, BInt::<12>, (u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize) }
impl_from_primitive! { U192, BUint::<3>, (u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize) }
impl_from_primitive! { U256, BUint::<4>, (u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize) }
impl_from_primitive! { U320, BUint::<5>, (u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize) }
impl_from_primitive! { U384, BUint::<6>, (u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize) }
impl_from_primitive! { U448, BUint::<7>, (u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize) }
impl_from_primitive! { U512, BUint::<8>, (u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize) }
impl_from_primitive! { U768, BUint::<12>, (u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize) }
impl_to_primitive! { I192, BInt::<3>, (u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize) }
impl_to_primitive! { I256, BInt::<4>, (u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize) }
impl_to_primitive! { I320, BInt::<5>, (u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize) }
impl_to_primitive! { I384, BInt::<6>, (u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize) }
impl_to_primitive! { I448, BInt::<7>, (u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize) }
impl_to_primitive! { I512, BInt::<8>, (u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize) }
impl_to_primitive! { I768, BInt::<12>, (u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize) }
impl_to_primitive! { U192, BUint::<3>, (u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize) }
impl_to_primitive! { U256, BUint::<4>, (u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize) }
impl_to_primitive! { U320, BUint::<5>, (u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize) }
impl_to_primitive! { U384, BUint::<6>, (u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize) }
impl_to_primitive! { U448, BUint::<7>, (u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize) }
impl_to_primitive! { U512, BUint::<8>, (u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize) }
impl_to_primitive! { U768, BUint::<12>, (u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize) }

macro_rules! impl_from_builtin {
    ($t:ident, $wrapped:ty, ($($o:ident),*)) => {
        paste! {
            $(
                impl From<$o> for $t {
                    fn from(val: $o) -> Self {
                        Self::[<from_$o>](val).unwrap()

                    }
                }
            )*
        }
    };
}

macro_rules! impl_try_from_builtin {
    ($t:ident, $wrapped:ty, ($($o:ident),*)) => {
        paste! {
            $(
                impl TryFrom<$o> for $t {
                    type Error = [<Parse $t Error>];

                    fn try_from(val: $o) -> Result<Self, Self::Error> {
                        match Self::[<from_$o>](val) {
                            Some(val) => Ok(val),
                            None => Err([<Parse $t Error>]::Overflow),
                        }
                    }
                }
            )*
        }
    };
}

macro_rules! impl_to_builtin {
    ($t:ident, $wrapped:ty, ($($o:ident),*)) => {
        paste! {
            $(
                impl TryFrom<$t> for $o {
                    type Error = [<Parse $t Error>];

                    fn try_from(val: $t) -> Result<Self, Self::Error> {
                        $o::try_from(val.0)
                            .map_err(|_| [<Parse $t Error>]::Overflow)
                    }
                }
            )*
        }
    };
}

impl_from_builtin! { I192, BInt::<3>, (i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize)}
impl_from_builtin! { I256, BInt::<4>, (i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize)}
impl_from_builtin! { I320, BInt::<5>, (i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize)}
impl_from_builtin! { I384, BInt::<6>, (i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize)}
impl_from_builtin! { I448, BInt::<7>, (i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize)}
impl_from_builtin! { I512, BInt::<8>, (i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize)}
impl_from_builtin! { I768, BInt::<12>, (i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize)}
impl_from_builtin! { U192, BUint::<3>, (u8, u16, u32, u64, u128, usize)}
impl_from_builtin! { U256, BUint::<4>, (u8, u16, u32, u64, u128, usize)}
impl_from_builtin! { U320, BUint::<5>, (u8, u16, u32, u64, u128, usize)}
impl_from_builtin! { U384, BUint::<6>, (u8, u16, u32, u64, u128, usize)}
impl_from_builtin! { U448, BUint::<7>, (u8, u16, u32, u64, u128, usize)}
impl_from_builtin! { U512, BUint::<8>, (u8, u16, u32, u64, u128, usize)}
impl_from_builtin! { U768, BUint::<12>, (u8, u16, u32, u64, u128, usize)}

impl_try_from_builtin! { U192, BUint::<3>, (i8, i16, i32, i64, i128, isize)}
impl_try_from_builtin! { U256, BUint::<4>, (i8, i16, i32, i64, i128, isize)}
impl_try_from_builtin! { U320, BUint::<5>, (i8, i16, i32, i64, i128, isize)}
impl_try_from_builtin! { U384, BUint::<6>, (i8, i16, i32, i64, i128, isize)}
impl_try_from_builtin! { U448, BUint::<7>, (i8, i16, i32, i64, i128, isize)}
impl_try_from_builtin! { U512, BUint::<8>, (i8, i16, i32, i64, i128, isize)}
impl_try_from_builtin! { U768, BUint::<12>, (i8, i16, i32, i64, i128, isize)}

impl_to_builtin! { I192, BInt::<3>, (i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize)}
impl_to_builtin! { I256, BInt::<4>, (i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize)}
impl_to_builtin! { I320, BInt::<5>, (i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize)}
impl_to_builtin! { I384, BInt::<6>, (i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize)}
impl_to_builtin! { I448, BInt::<7>, (i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize)}
impl_to_builtin! { I512, BInt::<8>, (i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize)}
impl_to_builtin! { I768, BInt::<12>, (i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize)}
impl_to_builtin! { U192, BUint::<3>, (i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize)}
impl_to_builtin! { U256, BUint::<4>, (i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize)}
impl_to_builtin! { U320, BUint::<5>, (i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize)}
impl_to_builtin! { U384, BUint::<6>, (i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize)}
impl_to_builtin! { U448, BUint::<7>, (i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize)}
impl_to_builtin! { U512, BUint::<8>, (i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize)}
impl_to_builtin! { U768, BUint::<12>, (i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize)}

macro_rules! impl_try_from_bigint {
    ($($t:ident, $wrapped:ty),*) => {
        paste! {
            $(
                impl TryFrom<BigInt> for $t {
                    type Error = [<Parse $t Error>];

                    fn try_from(val: BigInt) -> Result<Self, Self::Error> {
                        let bytes = val.to_signed_bytes_le();
                        match <$wrapped>::from_le_slice(&bytes) {
                            Some(val) => Ok(Self(val)),
                            None => Err([<Parse $t Error>]::Overflow),
                        }
                    }
                }
            )*
        }
    };
}

macro_rules! impl_to_bigint {
    ($($t:ident, $wrapped:ty),*) => {
        paste! {
            $(
                impl From<$t> for BigInt {
                    fn from(val: $t) -> BigInt {
                        // TODO: switch from str to bytes
                        BigInt::from_str(&val.to_string()).unwrap()
                    }
                }
            )*
        }
    };
}
impl_try_from_bigint! { I192, BInt::<3>  }
impl_try_from_bigint! { I256, BInt::<4>  }
impl_try_from_bigint! { I320, BInt::<5>  }
impl_try_from_bigint! { I384, BInt::<6>  }
impl_try_from_bigint! { I448, BInt::<7>  }
impl_try_from_bigint! { I512, BInt::<8>  }
impl_try_from_bigint! { I768, BInt::<12> }
impl_try_from_bigint! { U192, BUint::<3> }
impl_try_from_bigint! { U256, BUint::<4> }
impl_try_from_bigint! { U320, BUint::<5> }
impl_try_from_bigint! { U384, BUint::<6> }
impl_try_from_bigint! { U448, BUint::<7> }
impl_try_from_bigint! { U512, BUint::<8> }
impl_try_from_bigint! { U768, BUint::<12> }
impl_to_bigint! { I192, BInt::<3> }
impl_to_bigint! { I256, BInt::<4> }
impl_to_bigint! { I320, BInt::<5> }
impl_to_bigint! { I384, BInt::<6> }
impl_to_bigint! { I448, BInt::<7> }
impl_to_bigint! { I512, BInt::<8> }
impl_to_bigint! { I768, BInt::<12> }
impl_to_bigint! { U192, BUint::<3> }
impl_to_bigint! { U256, BUint::<4> }
impl_to_bigint! { U320, BUint::<5> }
impl_to_bigint! { U384, BUint::<6> }
impl_to_bigint! { U448, BUint::<7> }
impl_to_bigint! { U512, BUint::<8> }
impl_to_bigint! { U768, BUint::<12> }

macro_rules! impl_from_string {
    ($($t:ident, $wrapped:ty),*) => {
        $(
            paste! {
                impl FromStr for $t {
                    type Err = [<Parse $t Error>];
                    fn from_str(val: &str) -> Result<Self, Self::Err> {
                        match <$wrapped>::from_str(val) {
                            Ok(val) => Ok(Self(val)),
                            Err(err) => Err(match err.kind() {
                                core::num::IntErrorKind::Empty => [<Parse $t Error>]::Empty,
                                core::num::IntErrorKind::InvalidDigit => [<Parse $t Error>]::InvalidDigit,
                                core::num::IntErrorKind::PosOverflow => [<Parse $t Error>]::Overflow,
                                core::num::IntErrorKind::NegOverflow => [<Parse $t Error>]::Overflow,
                                core::num::IntErrorKind::Zero => unreachable!("Zero is only issued for non-zero type"),
                                _ => [<Parse $t Error>]::InvalidDigit, // Enum is non-exhaustive, sensible fallback is InvalidDigit
                            })
                        }
                    }
                }
            }
        )*
    };
}

impl_from_string! { I192, BInt::<3> }
impl_from_string! { I256, BInt::<4> }
impl_from_string! { I320, BInt::<5> }
impl_from_string! { I384, BInt::<6> }
impl_from_string! { I448, BInt::<7> }
impl_from_string! { I512, BInt::<8> }
impl_from_string! { I768, BInt::<12> }
impl_from_string! { U192, BUint::<3> }
impl_from_string! { U256, BUint::<4> }
impl_from_string! { U320, BUint::<5> }
impl_from_string! { U384, BUint::<6> }
impl_from_string! { U448, BUint::<7> }
impl_from_string! { U512, BUint::<8> }
impl_from_string! { U768, BUint::<12> }

macro_rules! impl_try_from_bnum {
    ($t:ident, $wrapped:ty, ($($into:ident, $into_wrap:ty),*)) => {
        $(
            paste! {
                impl TryFrom<$t> for $into {
                    type Error = [<Parse $into Error>];

                    fn try_from(val: $t) -> Result<Self, Self::Error> {
                        let mut sign = Self::ONE;
                        let mut other = val;

                        if other < <$t>::ZERO {
                            if Self::MIN == Self::ZERO {
                                return Err(Self::Error::NegativeToUnsigned);
                            } else {
                                // This is basically abs() function (which is not available for
                                // unsigned types).
                                // Do not perform below for MIN value to avoid overflow
                                if other != <$t>::MIN {
                                    other = <$t>::ZERO - other;
                                    sign = Self::ZERO - sign;
                                }
                            }
                        }
                        if (other.leading_zeros() as i32) <= <$t>::BITS as i32 - Self::BITS as i32 {
                            return Err(Self::Error::Overflow);
                        }
                        Ok(
                         Self(<$into_wrap>::cast_from(other.0)) * sign)
                    }
                }

            }
        )*
    };
}
macro_rules! impl_from_bnum {
    ($t:ident, $wrapped:ty, ($($into:ident, $into_wrap:ty),*)) => {
        $(
            paste! {
                impl From<$t> for $into {
                    fn from(val: $t) -> $into {
                        let mut sign = <$into>::ONE;
                        let mut other = val;

                        if other < <$t>::ZERO {
                            if <$into>::MIN == <$into>::ZERO {
                                panic!("NegativeToUnsigned");
                            } else {
                                // This is basically abs() function (which is not available for
                                // unsigned types).
                                // Do not perform below for MIN value to avoid overflow
                                if other != <$t>::MIN {
                                    other = <$t>::ZERO - other;
                                    sign = <$into>::ZERO - sign;
                                }
                            }
                        }
                        if (other.leading_zeros() as i32) <= <$t>::BITS as i32 - <$into>::BITS as i32 {
                            panic!("Overflow");
                        }
                        Self(<$into_wrap>::cast_from(other.0)) * sign
                    }
                }

            }
        )*
    };
}

impl_try_from_bnum! {
    I192, BInt::<3>, (
        U192, BUint::<3>,
        U256, BUint::<4>,
        U320, BUint::<5>,
        U384, BUint::<6>,
        U448, BUint::<7>,
        U512, BUint::<8>,
        U768, BUint::<12>
    )
}
impl_from_bnum! {
    I192, BInt::<3>, (
        I256, BInt::<4>,
        I320, BInt::<5>,
        I384, BInt::<6>,
        I448, BInt::<7>,
        I512, BInt::<8>,
        I768, BInt::<12>
    )
}

impl_try_from_bnum! {
    I256, BInt::<4>, (
        I192, BInt::<3>,
        U192, BUint::<3>,
        U256, BUint::<4>,
        U320, BUint::<5>,
        U384, BUint::<6>,
        U448, BUint::<7>,
        U512, BUint::<8>,
        U768, BUint::<12>
    )
}
impl_from_bnum! {
    I256, BInt::<4>, (
        I320, BInt::<5>,
        I384, BInt::<6>,
        I448, BInt::<7>,
        I512, BInt::<8>,
        I768, BInt::<12>
    )
}

impl_try_from_bnum! {
    I320, BInt::<5>, (
        I192, BInt::<3>,
        I256, BInt::<4>,
        U192, BUint::<3>,
        U256, BUint::<4>,
        U320, BUint::<5>,
        U384, BUint::<6>,
        U448, BUint::<7>,
        U512, BUint::<8>,
        U768, BUint::<12>
    )
}
impl_from_bnum! {
    I320, BInt::<5>, (
        I384, BInt::<6>,
        I448, BInt::<7>,
        I512, BInt::<8>,
        I768, BInt::<12>
    )
}

impl_try_from_bnum! {
    I384, BInt::<6>, (
        I192, BInt::<3>,
        I256, BInt::<4>,
        I320, BInt::<5>,
        U192, BUint::<3>,
        U256, BUint::<4>,
        U320, BUint::<5>,
        U384, BUint::<6>,
        U448, BUint::<7>,
        U512, BUint::<8>,
        U768, BUint::<12>
    )
}
impl_from_bnum! {
    I384, BInt::<6>, (
        I448, BInt::<7>,
        I512, BInt::<8>,
        I768, BInt::<12>
    )
}

impl_try_from_bnum! {
    I448, BInt::<7>, (
        I192, BInt::<3>,
        I256, BInt::<4>,
        I320, BInt::<5>,
        I384, BInt::<6>,
        U192, BUint::<3>,
        U256, BUint::<4>,
        U320, BUint::<5>,
        U384, BUint::<6>,
        U448, BUint::<7>,
        U512, BUint::<8>,
        U768, BUint::<12>
    )
}
impl_from_bnum! {
    I448, BInt::<7>, (
        I512, BInt::<8>,
        I768, BInt::<12>
    )
}

impl_try_from_bnum! {
    I512, BInt::<8>, (
        I192, BInt::<3>,
        I256, BInt::<4>,
        I320, BInt::<5>,
        I384, BInt::<6>,
        I448, BInt::<7>,
        U192, BUint::<3>,
        U256, BUint::<4>,
        U320, BUint::<5>,
        U384, BUint::<6>,
        U448, BUint::<7>,
        U512, BUint::<8>,
        U768, BUint::<12>
    )
}
impl_from_bnum! {
    I512, BInt::<8>, (
        I768, BInt::<12>
    )
}

impl_try_from_bnum! {
    I768, BInt::<12>, (
        I192, BInt::<3>,
        I256, BInt::<4>,
        I320, BInt::<5>,
        I384, BInt::<6>,
        I448, BInt::<7>,
        I512, BInt::<8>,
        U192, BUint::<3>,
        U256, BUint::<4>,
        U320, BUint::<5>,
        U384, BUint::<6>,
        U448, BUint::<7>,
        U512, BUint::<8>,
        U768, BUint::<12>
    )
}

// must fit 0 - MAX
impl_try_from_bnum! {
    U192, BUint::<3>, (
        I192, BInt::<3>
    )
}
impl_from_bnum! {
    U192, BUint::<3>, (
        I256, BInt::<4>,
        I320, BInt::<5>,
        I384, BInt::<6>,
        I448, BInt::<7>,
        I512, BInt::<8>,
        I768, BInt::<12>,
        U256, BUint::<4>,
        U320, BUint::<5>,
        U384, BUint::<6>,
        U448, BUint::<7>,
        U512, BUint::<8>,
        U768, BUint::<12>
    )
}

impl_try_from_bnum! {
    U256, BUint::<4>, (
        I192, BInt::<3>,
        U192, BUint::<3>,
        I256, BInt::<4>
    )
}
impl_from_bnum! {
    U256, BUint::<4>, (
        I320, BInt::<5>,
        I384, BInt::<6>,
        I448, BInt::<7>,
        I512, BInt::<8>,
        I768, BInt::<12>,
        U320, BUint::<5>,
        U384, BUint::<6>,
        U448, BUint::<7>,
        U512, BUint::<8>,
        U768, BUint::<12>
    )
}

impl_try_from_bnum! {
    U320, BUint::<5>, (
        I192, BInt::<3>,
        I256, BInt::<4>,
        I320, BInt::<5>,
        U192, BUint::<3>,
        U256, BUint::<4>
    )
}
impl_from_bnum! {
    U320, BUint::<5>, (
        I384, BInt::<6>,
        I448, BInt::<7>,
        I512, BInt::<8>,
        I768, BInt::<12>,
        U384, BUint::<6>,
        U448, BUint::<7>,
        U512, BUint::<8>,
        U768, BUint::<12>
    )
}

impl_try_from_bnum! {
    U384, BUint::<6>, (
        I192, BInt::<3>,
        I256, BInt::<4>,
        I320, BInt::<5>,
        I384, BInt::<6>,
        U192, BUint::<3>,
        U256, BUint::<4>,
        U320, BUint::<5>
    )
}
impl_from_bnum! {
    U384, BUint::<6>, (
        I448, BInt::<7>,
        I512, BInt::<8>,
        I768, BInt::<12>,
        U448, BUint::<7>,
        U512, BUint::<8>,
        U768, BUint::<12>
    )
}

impl_try_from_bnum! {
    U448, BUint::<7>, (
        I192, BInt::<3>,
        I256, BInt::<4>,
        I320, BInt::<5>,
        I384, BInt::<6>,
        I448, BInt::<7>,
        U192, BUint::<3>,
        U256, BUint::<4>,
        U320, BUint::<5>,
        U384, BUint::<6>
    )
}
impl_from_bnum! {
    U448, BUint::<7>, (
        I512, BInt::<8>,
        I768, BInt::<12>,
        U512, BUint::<8>,
        U768, BUint::<12>
    )
}

impl_try_from_bnum! {
    U512, BUint::<8>, (
        I192, BInt::<3>,
        I256, BInt::<4>,
        I320, BInt::<5>,
        I384, BInt::<6>,
        I448, BInt::<7>,
        I512, BInt::<8>,
        U192, BUint::<3>,
        U256, BUint::<4>,
        U320, BUint::<5>,
        U384, BUint::<6>,
        U448, BUint::<7>
    )
}
impl_from_bnum! {
    U512, BUint::<8>, (
        I768, BInt::<12>,
        U768, BUint::<12>
    )
}

impl_try_from_bnum! {
    U768, BUint::<12>, (
        I192, BInt::<3>,
        I256, BInt::<4>,
        I320, BInt::<5>,
        I384, BInt::<6>,
        I448, BInt::<7>,
        I512, BInt::<8>,
        I768, BInt::<12>,
        U192, BUint::<3>,
        U256, BUint::<4>,
        U320, BUint::<5>,
        U384, BUint::<6>,
        U448, BUint::<7>,
        U512, BUint::<8>
    )
}

macro_rules! impl_from_bytes {
    ($($t:ident, $wrapped:ty),*) => {
        paste! {
            $(
                impl TryFrom<&[u8]> for $t {
                    type Error = [<Parse $t Error>];
                    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
                        if bytes.len() > <$t>::BYTES as usize {
                            Err([<Parse $t Error>]::InvalidLength)
                        } else {
                            Ok(Self(<$wrapped>::from_le_slice(&bytes).unwrap()))
                        }
                    }
                }

                impl TryFrom<Vec<u8>> for $t {
                    type Error = [<Parse $t Error>];
                    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
                        if bytes.len() > <$t>::BYTES as usize {
                            Err([<Parse $t Error>]::InvalidLength)
                        } else {
                            Ok(Self(<$wrapped>::from_le_slice(&bytes).unwrap()))
                        }
                    }
                }

                impl $t {
                    pub fn from_le_bytes(bytes: &[u8]) -> Self {
                        Self::try_from(bytes).unwrap()
                    }
                }
            )*
        }
    };
}

macro_rules! impl_to_bytes {
    ($($t:ident, $wrapped:ty),*) => {
        paste! {
            $(
                impl $t {
                    pub fn to_le_bytes(&self) -> [u8; <$wrapped>::BYTES as usize] {
                        let mut shift = 0;
                        let mut bytes = [0_u8; <$wrapped>::BYTES as usize];

                        let ffs = <$t>::from(0xffff_ffff_ffff_ffff_u64);
                        for i in 0..<$wrapped>::BYTES/8 {
                            let idx: usize = (i * 8) as usize;
                            let mut masked  = self.0 >> shift;
                            masked &= ffs.0;
                            let u: u64 = u64::try_from(masked).unwrap();
                            let _ = &bytes[idx..idx+8].copy_from_slice(&u.to_le_bytes());
                            shift += 64;
                        }
                        bytes
                    }
                }
            )*
        }
    };
}

impl_from_bytes! { I192, BInt::<3> }
impl_from_bytes! { I256, BInt::<4> }
impl_from_bytes! { I320, BInt::<5> }
impl_from_bytes! { I384, BInt::<6> }
impl_from_bytes! { I448, BInt::<7> }
impl_from_bytes! { I512, BInt::<8> }
impl_from_bytes! { I768, BInt::<12> }
impl_from_bytes! { U192, BUint::<3> }
impl_from_bytes! { U256, BUint::<4> }
impl_from_bytes! { U320, BUint::<5> }
impl_from_bytes! { U384, BUint::<6> }
impl_from_bytes! { U448, BUint::<7> }
impl_from_bytes! { U512, BUint::<8> }
impl_from_bytes! { U768, BUint::<12> }
impl_to_bytes! { I192, BInt::<3> }
impl_to_bytes! { I256, BInt::<4> }
impl_to_bytes! { I320, BInt::<5> }
impl_to_bytes! { I384, BInt::<6> }
impl_to_bytes! { I448, BInt::<7> }
impl_to_bytes! { I512, BInt::<8> }
impl_to_bytes! { I768, BInt::<12> }
impl_to_bytes! { U192, BUint::<3> }
impl_to_bytes! { U256, BUint::<4> }
impl_to_bytes! { U320, BUint::<5> }
impl_to_bytes! { U384, BUint::<6> }
impl_to_bytes! { U448, BUint::<7> }
impl_to_bytes! { U512, BUint::<8> }
impl_to_bytes! { U768, BUint::<12> }

macro_rules! from_and_to_u64_arr_signed {
    ($($t:ident, $wrapped:ty),*) => {
        $(
            paste! {
                impl $t {

                    pub const fn from_digits(digits: [u64; <$t>::N]) -> Self {
                        let u = BUint::<{$t::N}>::from_digits(digits);
                        Self(<$wrapped>::from_bits(u))
                    }

                    pub const fn to_digits(&self) -> [u64; <$t>::N] {
                        let u: BUint::<{$t::N}> = self.0.to_bits();
                        *u.digits()
                    }
                }
            }
        )*
    };
}

macro_rules! from_and_to_u64_arr_unsigned {
    ($($t:ident, $wrapped:ty),*) => {
        $(
            paste! {
                impl $t {

                    pub const fn from_digits(digits: [u64; <$t>::N]) -> Self {
                        Self(<$wrapped>::from_digits(digits))
                    }

                    pub const fn to_digits(&self) -> [u64; <$t>::N] {
                        *self.0.digits()
                    }
                }
            }
        )*
    };
}

from_and_to_u64_arr_signed! { I192, BInt::<3> }
from_and_to_u64_arr_signed! { I256, BInt::<4> }
from_and_to_u64_arr_signed! { I320, BInt::<5> }
from_and_to_u64_arr_signed! { I384, BInt::<6> }
from_and_to_u64_arr_signed! { I448, BInt::<7> }
from_and_to_u64_arr_signed! { I512, BInt::<8> }
from_and_to_u64_arr_signed! { I768, BInt::<12> }

from_and_to_u64_arr_unsigned! { U192, BUint::<3> }
from_and_to_u64_arr_unsigned! { U256, BUint::<4> }
from_and_to_u64_arr_unsigned! { U320, BUint::<5> }
from_and_to_u64_arr_unsigned! { U384, BUint::<6> }
from_and_to_u64_arr_unsigned! { U448, BUint::<7> }
from_and_to_u64_arr_unsigned! { U512, BUint::<8> }
from_and_to_u64_arr_unsigned! { U768, BUint::<12> }
