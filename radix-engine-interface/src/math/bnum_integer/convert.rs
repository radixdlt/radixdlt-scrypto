use super::*;
use bnum::cast::CastFrom;

/// Trait for short hand notation for try_from().unwrap()
/// As opposed to `try_from(x).unwrap()` this will panic if the conversion fails.
pub trait By<T> {
    type Output;
    fn by(val: T) -> Self::Output;
}

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
impl_from_primitive! { BnumI256, BInt::<4>, (u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize) }
impl_from_primitive! { BnumI384, BInt::<6>, (u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize) }
impl_from_primitive! { BnumI512, BInt::<8>, (u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize) }
impl_from_primitive! { BnumI768, BInt::<12>, (u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize) }
impl_from_primitive! { BnumU256, BUint::<4>, (u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize) }
impl_from_primitive! { BnumU384, BUint::<6>, (u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize) }
impl_from_primitive! { BnumU512, BUint::<8>, (u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize) }
impl_from_primitive! { BnumU768, BUint::<12>, (u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize) }
impl_to_primitive! { BnumI256, BInt::<4>, (u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize) }
impl_to_primitive! { BnumI384, BInt::<6>, (u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize) }
impl_to_primitive! { BnumI512, BInt::<8>, (u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize) }
impl_to_primitive! { BnumI768, BInt::<12>, (u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize) }
impl_to_primitive! { BnumU256, BUint::<4>, (u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize) }
impl_to_primitive! { BnumU384, BUint::<4>, (u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize) }
impl_to_primitive! { BnumU512, BUint::<8>, (u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize) }
impl_to_primitive! { BnumU768, BUint::<12>, (u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize) }

macro_rules! impl_from_builtin{
    ($t:ident, $wrapped:ty, ($($o:ident),*)) => {
        paste! {
            $(
                impl From<$o> for $t {
                    fn from(val: $o) -> Self {
                        Self::[<from_$o>](val).unwrap()

                    }
                }
                impl By<$o> for $t {
                    type Output = $t;
                    fn by(val: $o) -> Self::Output {
                        Self::Output::try_from(val).unwrap()
                    }
                }
            )*
        }
    };
}

macro_rules! impl_try_from_builtin{
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

macro_rules! impl_to_builtin{
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

impl_from_builtin! { BnumI256, BInt::<4>, (i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize)}
impl_from_builtin! { BnumI384, BInt::<6>, (i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize)}
impl_from_builtin! { BnumI512, BInt::<8>, (i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize)}
impl_from_builtin! { BnumI768, BInt::<12>,(i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize)}
impl_from_builtin! { BnumU256, BUint::<4>, (u8, u16, u32, u64, u128, usize)}
impl_from_builtin! { BnumU384, BUint::<6>, (u8, u16, u32, u64, u128, usize)}
impl_from_builtin! { BnumU512, BUint::<8>, (u8, u16, u32, u64, u128, usize)}
impl_from_builtin! { BnumU768, BUint::<8>, (u8, u16, u32, u64, u128, usize)}

impl_try_from_builtin! { BnumU256, BUint::<4>, (i8, i16, i32, i64, i128, isize)}
impl_try_from_builtin! { BnumU384, BUint::<6>, (i8, i16, i32, i64, i128, isize)}
impl_try_from_builtin! { BnumU512, BUint::<8>, (i8, i16, i32, i64, i128, isize)}
impl_try_from_builtin! { BnumU768, BUint::<12>, (i8, i16, i32, i64, i128, isize)}

impl_to_builtin! { BnumI256, BInt::<4>, (i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize)}
impl_to_builtin! { BnumI384, BInt::<6>, (i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize)}
impl_to_builtin! { BnumI512, BInt::<8>, (i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize)}
impl_to_builtin! { BnumI768, BInt::<12>, (i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize)}
impl_to_builtin! { BnumU256, BUint::<4>, (i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize)}
impl_to_builtin! { BnumU384, BUint::<6>, (i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize)}
impl_to_builtin! { BnumU512, BUint::<8>, (i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize)}
impl_to_builtin! { BnumU768, BUint::<12>, (i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize)}

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
impl_try_from_bigint! { BnumI256, BInt::<4> }
impl_try_from_bigint! { BnumI384, BInt::<6> }
impl_try_from_bigint! { BnumI512, BInt::<8> }
impl_try_from_bigint! { BnumI768, BInt::<12> }
impl_try_from_bigint! { BnumU256, BUint::<4> }
impl_try_from_bigint! { BnumU384, BUint::<6> }
impl_try_from_bigint! { BnumU512, BUint::<8> }
impl_try_from_bigint! { BnumU768, BUint::<12> }
impl_to_bigint! { BnumI256, BInt::<4> }
impl_to_bigint! { BnumI384, BInt::<6> }
impl_to_bigint! { BnumI512, BInt::<8> }
impl_to_bigint! { BnumI768, BInt::<12> }
impl_to_bigint! { BnumU256, BUint::<4> }
impl_to_bigint! { BnumU384, BUint::<6> }
impl_to_bigint! { BnumU512, BUint::<8> }
impl_to_bigint! { BnumU768, BUint::<12> }

macro_rules! impl_from_string {
    ($($t:ident, $wrapped:ty),*) => {
        $(
            paste! {
                impl FromStr for $t {
                    type Err = [<Parse $t Error>];
                    fn from_str(val: &str) -> Result<Self, Self::Err> {
                        match <$wrapped>::from_str(val) {
                            Ok(val) => Ok(Self(val)),
                            Err(_) => Err([<Parse $t Error>]::InvalidDigit),
                        }
                    }
                }
            }
        )*
    };
}

impl_from_string! { BnumI256, BInt::<4> }
impl_from_string! { BnumI384, BInt::<6> }
impl_from_string! { BnumI512, BInt::<8> }
impl_from_string! { BnumI768, BInt::<12> }
impl_from_string! { BnumU256, BUint::<4> }
impl_from_string! { BnumU384, BUint::<6> }
impl_from_string! { BnumU512, BUint::<8> }
impl_from_string! { BnumU768, BUint::<12> }

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
    BnumI512, BInt::<8>, (
        BnumI256, BInt::<4>,
        BnumI384, BInt::<6>,
        BnumU256, BUint::<4>,
        BnumU384, BUint::<6>,
        BnumU512, BUint::<8>,
        BnumU768, BUint::<12>
    )
}
impl_from_bnum! {
    BnumI512, BInt::<8>, (
        BnumI768, BInt::<12>
    )
}
impl_try_from_bnum! {
    BnumI256, BInt::<4>, (
        BnumU256, BUint::<4>,
        BnumU384, BUint::<6>,
        BnumU512, BUint::<8>,
        BnumU768, BUint::<12>
    )
}
impl_from_bnum! {
    BnumI256, BInt::<4>, (
        BnumI384, BInt::<6>,
        BnumI512, BInt::<8>,
        BnumI768, BInt::<12>
    )
}
impl_try_from_bnum! {
    BnumI384, BInt::<6>, (
        BnumI256, BInt::<4>,
        BnumU256, BUint::<4>,
        BnumU384, BUint::<6>,
        BnumU512, BUint::<8>,
        BnumU768, BUint::<12>
    )
}
impl_from_bnum! {
    BnumI384, BInt::<6>, (
        BnumI512, BInt::<8>,
        BnumI768, BInt::<12>
    )
}
impl_try_from_bnum! {
    BnumI768, BInt::<12>, (
        BnumI256, BInt::<4>,
        BnumI384, BInt::<6>,
        BnumI512, BInt::<8>,
        BnumU256, BUint::<4>,
        BnumU384, BUint::<6>,
        BnumU512, BUint::<8>,
        BnumU768, BUint::<12>
    )
}

// must fit 0 - MAX
impl_try_from_bnum! {
    BnumU512, BUint::<8>, (
        BnumI256, BInt::<4>,
        BnumI384, BInt::<6>,
        BnumI512, BInt::<8>,
        BnumU256, BUint::<4>,
        BnumU384, BUint::<6>
    )
}
impl_from_bnum! {
    BnumU512, BUint::<8>, (
        BnumI768, BInt::<12>,
        BnumU768, BUint::<12>
    )
}
impl_try_from_bnum! {
    BnumU256, BUint::<4>, (
        BnumI256, BInt::<4>
    )
}
impl_from_bnum! {
    BnumU256, BUint::<4>, (
        BnumI384, BInt::<6>,
        BnumI512, BInt::<8>,
        BnumI768, BInt::<12>,
        BnumU384, BUint::<6>,
        BnumU512, BUint::<8>,
        BnumU768, BUint::<12>
    )
}
impl_try_from_bnum! {
    BnumU384, BUint::<6>, (
        BnumI256, BInt::<4>,
        BnumI384, BInt::<6>,
        BnumU256, BUint::<4>
    )
}
impl_from_bnum! {
    BnumU384, BUint::<6>, (
        BnumI512, BInt::<8>,
        BnumI768, BInt::<12>,
        BnumU512, BUint::<8>,
        BnumU768, BUint::<12>
    )
}
impl_try_from_bnum! {
    BnumU768, BUint::<12>, (
        BnumI256, BInt::<4>,
        BnumI384, BInt::<6>,
        BnumI512, BInt::<8>,
        BnumI768, BInt::<12>,
        BnumU256, BUint::<4>,
        BnumU384, BUint::<6>,
        BnumU512, BUint::<8>
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

                    pub fn to_vec(&self) -> Vec<u8> {
                        self.to_le_bytes().to_vec()
                    }
                }
            )*
        }
    };
}

impl_from_bytes! { BnumI256, BInt::<4> }
impl_from_bytes! { BnumI384, BInt::<6> }
impl_from_bytes! { BnumI512, BInt::<8> }
impl_from_bytes! { BnumI768, BInt::<12> }
impl_from_bytes! { BnumU256, BUint::<4> }
impl_from_bytes! { BnumU384, BUint::<6> }
impl_from_bytes! { BnumU512, BUint::<8> }
impl_from_bytes! { BnumU768, BUint::<12> }
impl_to_bytes! { BnumI256, BInt::<4> }
impl_to_bytes! { BnumI384, BInt::<6> }
impl_to_bytes! { BnumI512, BInt::<8> }
impl_to_bytes! { BnumI768, BInt::<12> }
impl_to_bytes! { BnumU256, BUint::<4> }
impl_to_bytes! { BnumU384, BUint::<6> }
impl_to_bytes! { BnumU512, BUint::<8> }
impl_to_bytes! { BnumU768, BUint::<12> }

macro_rules! impl_from_u64_arr_signed {
    ($($t:ident, $wrapped:ty),*) => {
        $(
            paste! {
                impl $t {

                    pub const fn from_digits(digits: [u64; <$t>::N]) -> Self {
                        let u = BUint::<{$t::N}>::from_digits(digits);
                        Self(<$wrapped>::from_bits(u))
                    }
                }
            }
        )*
    };
}

macro_rules! from_u64_arr_unsigned {
    ($($t:ident, $wrapped:ty),*) => {
        $(
            paste! {
                impl $t {

                    pub const fn from_digits(digits: [u64; <$t>::N]) -> Self {
                        Self(<$wrapped>::from_digits(digits))
                    }
                }
            }
        )*
    };
}

impl_from_u64_arr_signed! { BnumI256, BInt::<4> }
impl_from_u64_arr_signed! { BnumI384, BInt::<6> }
impl_from_u64_arr_signed! { BnumI512, BInt::<8> }
impl_from_u64_arr_signed! { BnumI768, BInt::<12> }

from_u64_arr_unsigned! { BnumU256, BUint::<4> }
from_u64_arr_unsigned! { BnumU384, BUint::<6> }
from_u64_arr_unsigned! { BnumU512, BUint::<8> }
from_u64_arr_unsigned! { BnumU768, BUint::<12> }
