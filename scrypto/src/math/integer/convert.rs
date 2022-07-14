use super::*;
use num_bigint::{BigInt, ParseBigIntError, Sign};
use num_traits::{FromPrimitive, Signed, ToPrimitive};
use paste::paste;
use sbor::rust::convert::{From, TryFrom};
use sbor::rust::str::FromStr;
use sbor::rust::string::String;

#[derive(Debug)]
pub enum ParseIntError {
    NegativeToUnsigned,
    Overflow,
}

/// Trait for short hand notation for try_from().unwrap()
/// As opposed to `try_from(x).unwrap()` this will panic if the conversion fails.
pub trait TFrom<T> {
    type Output;
    fn tfrom(val: T) -> Self::Output;
}

macro_rules! impl_from_primitive {
    ($($t:ident),*) => {
        paste! {
            $(
                impl FromPrimitive for $t {
                    fn from_i64(n: i64) -> Option<Self> {
                       Self::try_from(n).ok()
                    }
                    fn from_i128(n: i128) -> Option<Self> {
                       Self::try_from(n).ok()
                    }
                    fn from_u64(n: u64) -> Option<Self> {
                       Self::try_from(n).ok()
                    }
                    fn from_u128(n: u128) -> Option<Self> {
                       Self::try_from(n).ok()
                    }
                }

                impl ToPrimitive for $t {
                    fn to_i64(&self) -> Option<i64> {
                        i64::try_from(*self).ok()
                    }
                    fn to_i128(&self) -> Option<i128> {
                        i128::try_from(*self).ok()
                    }
                    fn to_u64(&self) -> Option<u64> {
                        u64::try_from(*self).ok()
                    }
                    fn to_u128(&self) -> Option<u128> {
                        u128::try_from(*self).ok()
                    }
            }
            )*
        }
    };
}
impl_from_primitive! { I8, I16, I32, I64, I128, I256, I384, I512, U8, U16, U32, U64, U128, U256, U384, U512 }

macro_rules! try_from{
    ($t:ident, ($($o:ident),*)) => {
        $(
            paste! {
                impl TryFrom<$o> for $t {
                    type Error = ParseIntError;
                    fn try_from(val: $o) -> Result<$t, ParseIntError> {
                        BigInt::from(val).try_into().map_err(|_| ParseIntError::Overflow)
                    }
                }
                impl TFrom<$o> for $t {
                    type Output = $t;
                    fn tfrom(val: $o) -> Self::Output {
                        Self::try_from(val).unwrap()
                    }
                }
            }
        )*
    };
}

macro_rules! try_from_large_all {
    ($($t:ident),*) => {
        $(
            try_from! { $t, (I256, I384, I512) }
            try_from! { $t, (U256, U384, U512) }
        )*
    };
}

try_from_large_all! { U8, U16, U32, U64, U128, I8, I16, I32, I64, I128 }

try_from! {u8, (U16, U32, U64, U128, U256, U384, U512, I8, I16, I32, I64, I128, I256, I384, I512)}
try_from! {u16, (U32, U64, U128, U256, U384, U512, I8, I16, I32, I64, I128, I256, I384, I512)}
try_from! {u32, (U64, U128, U256, U384, U512, I8, I16, I32, I64, I128, I256, I384, I512)}
try_from! {u64, (U128, U256, U384, U512, I8, I16, I32, I64, I128, I256, I384, I512)}
try_from! {u128, (U256, U384, U512, I8, I16, I32, I64, I128, I256, I384, I512)}
try_from! {i8, (U8, U16, U32, U64, U128, U256, U384, U512, I16, I32, I64, I128, I256, I384, I512)}
try_from! {i16, ( U16, U32, U64, U128, U256, U384, U512, I32, I64, I128, I256, I384, I512)}
try_from! {i32, (U32, U64, U128, U256, U384, U512, I64, I128, I256, I384, I512)}
try_from! {i64, (U64, U128, U256, U384, U512, I128, I256, I384, I512)}
try_from! {i128, (U128, U256, U384, U512, I256, I384, I512)}

try_from! {U8, (i8, i16, i32, i64, i128, u16, u32, u64, u128)}
try_from! {U16, (i8, i16, i32, i64, i128, u32, u64, u128)}
try_from! {U32, (i8, i16, i32, i64, i128, u64, u128)}
try_from! {U64, (i8, i16, i32, i64, i128, u128)}
try_from! {U128, (i8, i16, i32, i64, i128)}
try_from! {U256, (i8, i16, i32, i64, i128)}
try_from! {U384, (i8, i16, i32, i64, i128)}
try_from! {U512, (i8, i16, i32, i64, i128)}
try_from! {I8, (u8, u16, u32, u64, u128, i16, i32, i64, i128)}
try_from! {I16, (i32, i64, i128, u16, u32, u64, u128)}
try_from! {I32, (i64, i128, u32, u64, u128)}
try_from! {I64, (i128, u64, u128)}
try_from! {I128, (u128)}

try_from! {U256, (I8, I16, I32, I64, I128, I256, I384, I512)}
try_from! {U384, (I8, I16, I32, I64, I128, I256, I384, I512)}
try_from! {U512, (I8, I16, I32, I64, I128, I256, I384, I512)}
try_from! {U256, (U384, U512)}
try_from! {U384, (U512)}
try_from! {I256, (U256, U384, U512)}
try_from! {I384, (U384, U512)}
try_from! {I512, (U512)}
try_from! {I256, (I384, I512)}
try_from! {I384, (I512)}
try_from! {U8, (I8, I16, I32, I64, I128)}
try_from! {U16, (I8, I16, I32, I64, I128)}
try_from! {U32, (I8, I16, I32, I64, I128)}
try_from! {U64, (I8, I16, I32, I64, I128)}
try_from! {U128, (I8, I16, I32, I64, I128)}
try_from! {I8, (U8, U16, U32, U64, U128)}
try_from! {I16, (U16, U32, U64, U128)}
try_from! {I32, (U32, U64, U128)}
try_from! {I64, (U64, U128)}
try_from! {I128, (U128)}
try_from! {U8, (U16, U32, U64, U128)}
try_from! {U16, (U32, U64, U128)}
try_from! {U32, (U64, U128)}
try_from! {U64, (U128)}
try_from! {I8, (I16, I32, I64, I128)}
try_from! {I16, (I32, I64, I128)}
try_from! {I32, (I64, I128)}
try_from! {I64, (I128)}

macro_rules! impl_others_to_large_unsigned {
    ($($t:ty),*) => {
        $(
            paste! {
                impl TryFrom<BigInt> for $t {
                    type Error = ParseIntError;
                    fn try_from(val: BigInt) -> Result<$t, ParseIntError> {
                        let (sign, bytes) = val.to_bytes_le();
                        if sign == Sign::Minus {
                            return Err(ParseIntError::NegativeToUnsigned);
                        }
                        const T_BYTES: usize = (<$t>::BITS / 8 ) as usize;
                        if bytes.len() > T_BYTES {
                            return Err(ParseIntError::Overflow);
                        }
                        let mut buf = [0u8; T_BYTES];
                        buf[..bytes.len()].copy_from_slice(&bytes);
                        Ok($t(buf))
                    }
                }

                impl TFrom<BigInt> for $t {
                    type Output = $t;
                    fn tfrom(val: BigInt) -> Self::Output {
                        Self::try_from(val).unwrap()
                    }
                }

                impl TryFrom<&[u8]> for $t {
                    type Error = ParseSliceError;
                    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
                        if bytes.len() > (<$t>::BITS / 8) as usize {
                            Err(ParseSliceError::InvalidLength)
                        } else {
                            let mut buf = [0u8; (<$t>::BITS / 8) as usize];
                            buf[..bytes.len()].copy_from_slice(bytes);
                            Ok(Self(buf))
                        }
                    }
                }

                impl TFrom<&[u8]> for $t {
                    type Output = $t;
                    fn tfrom(val: &[u8]) -> Self::Output {
                        Self::try_from(val).unwrap()
                    }
                }


                impl TryFrom<Vec<u8>> for $t {
                    type Error = ParseSliceError;
                    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
                        if bytes.len() > (<$t>::BITS / 8) as usize {
                            Err(ParseSliceError::InvalidLength)
                        } else {
                            let mut buf = [0u8; (<$t>::BITS / 8) as usize];
                            buf[..bytes.len()].copy_from_slice(&bytes);
                            Ok(Self(buf))
                        }
                    }
                }

                impl TFrom<Vec<u8>> for $t {
                    type Output = $t;
                    fn tfrom(val: Vec<u8>) -> Self::Output {
                        Self::try_from(val).unwrap()
                    }
                }

            }
        )*
    }
}

impl_others_to_large_unsigned! { U256, U384, U512 }

macro_rules! impl_others_to_large_signed {
    ($($t:ty),*) => {
        $(
            paste! {
                impl TryFrom<BigInt> for $t {
                    type Error = ParseIntError;
                    fn try_from(val: BigInt) -> Result<$t, ParseIntError> {
                        let bytes = val.to_signed_bytes_le();
                        const T_BYTES: usize = (<$t>::BITS / 8 ) as usize;
                        if bytes.len() > T_BYTES {
                            return Err(ParseIntError::Overflow);
                        }
                        let mut buf = if val.is_negative() {
                            [255u8; T_BYTES]
                        } else {
                            [0u8; T_BYTES]
                        };
                        buf[..bytes.len()].copy_from_slice(&bytes);
                        Ok($t(buf))
                    }
                }

                impl TFrom<BigInt> for $t {
                    type Output = $t;
                    fn tfrom(val: BigInt) -> Self::Output {
                        Self::try_from(val).unwrap()
                    }
                }

                impl TryFrom<&[u8]> for $t {
                    type Error = ParseSliceError;
                    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
                        const T_BYTES: usize = (<$t>::BITS / 8 ) as usize;
                        if bytes.len() > T_BYTES {
                            Err(ParseSliceError::InvalidLength)
                        } else {
                            let mut buf = if bytes.len() != 0 && (*bytes.iter().last().unwrap() as i8) < 0 {
                                [255u8; T_BYTES]
                            } else {
                                [0u8; T_BYTES]
                            };
                            buf[..bytes.len()].copy_from_slice(bytes);
                            Ok(Self(buf))
                        }
                    }
                }

                impl TFrom<&[u8]> for $t {
                    type Output = $t;
                    fn tfrom(val: &[u8]) -> Self::Output {
                        Self::try_from(val).unwrap()
                    }
                }


                impl TryFrom<Vec<u8>> for $t {
                    type Error = ParseSliceError;
                    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
                        const T_BYTES: usize = (<$t>::BITS / 8 ) as usize;
                        if bytes.len() > T_BYTES {
                            Err(ParseSliceError::InvalidLength)
                        } else {
                            let mut buf = if bytes.len() != 0 && (*bytes.last().unwrap() as i8) < 0 {
                                [255u8; T_BYTES]
                            } else {
                                [0u8; T_BYTES]
                            };
                            buf[..bytes.len()].copy_from_slice(&bytes);
                            Ok(Self(buf))
                        }
                    }
                }

                impl TFrom<Vec<u8>> for $t {
                    type Output = $t;
                    fn tfrom(val: Vec<u8>) -> Self::Output {
                        Self::try_from(val).unwrap()
                    }
                }
            }
        )*
    }
}
impl_others_to_large_signed! { I256, I384, I512 }

macro_rules! impl_others_to_small_unsigned {
    ($($t:ty),*) => {
        $(
            paste! {
                impl TryFrom<BigInt> for $t {
                    type Error = ParseIntError;
                    fn try_from(val: BigInt) -> Result<$t, ParseIntError> {
                        if val.is_negative() {
                            return Err(ParseIntError::NegativeToUnsigned);
                        }
                        Ok($t(val.[<to_$t:lower>]().ok_or_else(|| ParseIntError::Overflow).unwrap()))
                    }
                }

                impl TFrom<BigInt> for $t {
                    type Output = $t;
                    fn tfrom(val: BigInt) -> Self::Output {
                        Self::try_from(val).unwrap()
                    }
                }

                impl TryFrom<&[u8]> for $t {
                    type Error = ParseSliceError;
                    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
                        const T_BYTES: usize = (<$t>::BITS / 8 ) as usize;
                        if bytes.len() > T_BYTES {
                            Err(ParseSliceError::InvalidLength)
                        } else {
                            let mut buf = [0u8; T_BYTES];
                            buf[..bytes.len()].copy_from_slice(bytes);
                            let wrapped: [<$t:lower>] = [<$t:lower>]::from_le_bytes(buf);
                            Ok(Self(wrapped))
                        }
                    }
                }

                impl TFrom<&[u8]> for $t {
                    type Output = $t;
                    fn tfrom(val: &[u8]) -> Self::Output {
                        Self::try_from(val).unwrap()
                    }
                }

                impl TryFrom<Vec<u8>> for $t {
                    type Error = ParseSliceError;
                    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
                        const T_BYTES: usize = (<$t>::BITS / 8 ) as usize;
                        if bytes.len() > T_BYTES {
                            Err(ParseSliceError::InvalidLength)
                        } else {
                            let mut buf = [0u8; T_BYTES];
                            buf[..bytes.len()].copy_from_slice(&bytes);
                            let wrapped: [<$t:lower>] = [<$t:lower>]::from_le_bytes(buf);
                            Ok(Self(wrapped))
                        }
                    }
                }

                impl TFrom<Vec<u8>> for $t {
                    type Output = $t;
                    fn tfrom(val: Vec<u8>) -> Self::Output {
                        Self::try_from(val).unwrap()
                    }
                }

            }
        )*
    }
}

impl_others_to_small_unsigned! { U8, U16, U32, U64, U128 }

macro_rules! impl_others_to_small_signed {
    ($($t:ty),*) => {
        $(
            paste! {
                impl TryFrom<BigInt> for $t {
                    type Error = ParseIntError;
                    fn try_from(val: BigInt) -> Result<$t, ParseIntError> {
                        Ok($t(val.[<to_$t:lower>]().ok_or_else(|| ParseIntError::Overflow).unwrap()))
                    }
                }

                impl TFrom<BigInt> for $t {
                    type Output = $t;
                    fn tfrom(val: BigInt) -> Self::Output {
                        Self::try_from(val).unwrap()
                    }
                }

                impl TryFrom<&[u8]> for $t {
                    type Error = ParseSliceError;
                    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
                        const T_BYTES: usize = (<$t>::BITS / 8 ) as usize;
                        if bytes.len() > T_BYTES {
                            Err(ParseSliceError::InvalidLength)
                        } else {
                            let mut buf = if bytes.len() != 0 && (*bytes.last().unwrap() as i8) < 0 {
                                [255u8; T_BYTES]
                            } else {
                                [0u8; T_BYTES]
                            };
                            buf[..bytes.len()].copy_from_slice(bytes);
                            let wrapped: [<$t:lower>] = [<$t:lower>]::from_le_bytes(buf);
                            Ok(Self(wrapped))
                        }
                    }
                }

                impl TFrom<&[u8]> for $t {
                    type Output = $t;
                    fn tfrom(val: &[u8]) -> Self::Output {
                        Self::try_from(val).unwrap()
                    }
                }

                impl TryFrom<Vec<u8>> for $t {
                    type Error = ParseSliceError;
                    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
                        const T_BYTES: usize = (<$t>::BITS / 8 ) as usize;
                        if bytes.len() > T_BYTES {
                            Err(ParseSliceError::InvalidLength)
                        } else {
                            let mut buf = if bytes.len() != 0 && (*bytes.last().unwrap() as i8) < 0 {
                                [255u8; T_BYTES]
                            } else {
                                [0u8; T_BYTES]
                            };
                            buf[..bytes.len()].copy_from_slice(&bytes);
                            let wrapped: [<$t:lower>] = [<$t:lower>]::from_le_bytes(buf);
                            Ok(Self(wrapped))
                        }
                    }
                }

                impl TFrom<Vec<u8>> for $t {
                    type Output = $t;
                    fn tfrom(val: Vec<u8>) -> Self::Output {
                        Self::try_from(val).unwrap()
                    }
                }

            }
        )*
    }
}

impl_others_to_small_signed! { I8, I16, I32, I64, I128 }

macro_rules! from_array_large {
    ($($t:ident),*) => {
        $(
            paste! {
                impl From<[u8; (<$t>::BITS / 8) as usize]> for $t {
                    fn from(val: [u8; (<$t>::BITS / 8) as usize]) -> Self {
                        Self(val)
                    }
                }

                impl TFrom<[u8; (<$t>::BITS / 8) as usize]> for $t {
                    type Output = $t;
                    fn tfrom(val: [u8; (<$t>::BITS / 8) as usize]) -> Self::Output {
                        Self::try_from(val).unwrap()
                    }
                }

                impl $t {
                    pub fn from_le_bytes(val: [u8; (<$t>::BITS / 8) as usize]) -> Self {
                        Self::from(val)
                    }
                }
            }
        )*
    };
}
from_array_large! { I256, I384, I512, U256, U384, U512 }

macro_rules! from_array_small {
    ($($t:ident),*) => {
        $(
            paste! {
                impl From<[u8; (<$t>::BITS / 8) as usize]> for $t {
                    fn from(val: [u8; (<$t>::BITS / 8) as usize]) -> Self {
                        let wrapped: [<$t:lower>] = [<$t:lower>]::from_le_bytes(val);
                        $t(wrapped)
                    }
                }

                impl TFrom<[u8; (<$t>::BITS / 8) as usize]> for $t {
                    type Output = $t;
                    fn tfrom(val: [u8; (<$t>::BITS / 8) as usize]) -> Self::Output {
                        Self::try_from(val).unwrap()
                    }
                }

                impl $t {
                    pub fn from_le_bytes(val: [u8; (<$t>::BITS / 8) as usize]) -> Self {
                        Self::from(val)
                    }
                }

            }
        )*
    };
}

from_array_small! { U8, U16, U32, U64, U128, I8, I16, I32, I64, I128 }

#[derive(Debug)]
pub enum ParseSliceError {
    InvalidLength,
}

macro_rules! from_int {
    ($t:ident, ($($o:ident),*)) => {
        $(
            paste! {
                impl From<$o> for $t {
                    fn from(val: $o) -> Self {
                        (BigInt::from(val)).try_into().unwrap()
                    }
                }

                impl TFrom<$o> for $t {
                    type Output = $t;
                    fn tfrom(val: $o) -> Self::Output {
                        Self::try_from(val).unwrap()
                    }
                }
            }
        )*
    };
}

from_int! {i8, (I8)}

from_int! {i16, (I8, I16)}
from_int! {i16, (U8)}

from_int! {i32, (I8, I16, I32)}
from_int! {i32, (U8, U16)}

from_int! {i64, (I8, I16, I32, I64)}
from_int! {i64, (U8, U16, U32)}

from_int! {i128, (I8, I16, I32, I64, I128)}
from_int! {i128, (U8, U16, U32, U64)}

from_int! {u8, (U8)}

from_int! {u16, (U8, U16)}

from_int! {u32, (U8, U16, U32)}

from_int! {u64, (U8, U16, U32, U64)}

from_int! {u128, (U8, U16, U32, U64, U128)}

from_int! {I8, (i8)}

from_int! {I16, (i8, i16)}
from_int! {I16, (u8)}

from_int! {I32, (i8, i16, i32)}
from_int! {I32, (u8, u16)}

from_int! {I64, (i8, i16, i32, i64)}
from_int! {I64, (u8, u16, u32)}

from_int! {I128, (i8, i16, i32, i64, i128)}
from_int! {I128, (u8, u16, u32, u64)}

from_int! {I256, (i8, i16, i32, i64, i128)}
from_int! {I256, (u8, u16, u32, u64, u128)}

from_int! {I384, (i8, i16, i32, i64, i128)}
from_int! {I384, (u8, u16, u32, u64, u128)}

from_int! {I512, (i8, i16, i32, i64, i128)}
from_int! {I512, (u8, u16, u32, u64, u128)}

from_int! {U8, (u8)}

from_int! {U16, (u8, u16)}

from_int! {U32, (u8, u16, u32)}

from_int! {U64, (u8, u16, u32, u64)}

from_int! {U128, (u8, u16, u32, u64, u128)}

from_int! {U256, (u8, u16, u32, u64, u128)}

from_int! {U384, (u8, u16, u32, u64, u128)}

from_int! {U512, (u8, u16, u32, u64, u128)}

from_int! {I16, (I8)}
from_int! {I16, (U8)}

from_int! {I32, (I8, I16)}
from_int! {I32, (U8, U16)}

from_int! {I64, (I8, I16, I32)}
from_int! {I64, (U8, U16, U32)}

from_int! {I128, (I8, I16, I32, I64)}
from_int! {I128, (U8, U16, U32, U64)}

from_int! {I256, (I8, I16, I32, I64, I128)}
from_int! {I256, (U8, U16, U32, U64, U128)}

from_int! {I384, (I8, I16, I32, I64, I128, I256)}
from_int! {I384, (U8, U16, U32, U64, U128, U256)}

from_int! {I512, (I8, I16, I32, I64, I128, I256, I384)}
from_int! {I512, (U8, U16, U32, U64, U128, U256, U384)}

from_int! {U16, (U8)}

from_int! {U32, (U8, U16)}

from_int! {U64, (U8, U16, U32)}

from_int! {U128, (U8, U16, U32, U64)}

from_int! {U256, (U8, U16, U32, U64, U128)}

from_int! {U384, (U8, U16, U32, U64, U128, U256)}

from_int! {U512, (U8, U16, U32, U64, U128, U256, U384)}

macro_rules! from_string {
    ($($t:ident),*) => {
        $(
            impl FromStr for $t {
                type Err = ParseBigIntError;
                fn from_str(val: &str) -> Result<Self, Self::Err> {
                    match val.parse::<BigInt>() {
                        Ok(big_int) => Ok($t::try_from(big_int).unwrap()),
                        Err(e) => Err(e)
                    }
                }
            }

            impl TFrom<&str> for $t {
                type Output = $t;
                fn tfrom(val: &str) -> Self::Output {
                    Self::from_str(val).unwrap()
                }
            }

            impl From<&str> for $t {
                fn from(val: &str) -> Self {
                    Self::from_str(&val).unwrap()
                }
            }

            impl From<String> for $t {
                fn from(val: String) -> Self {
                    Self::from_str(&val).unwrap()
                }
            }

            impl TFrom<String> for $t {
                type Output = $t;
                fn tfrom(val: String) -> Self::Output {
                    Self::from_str(&val).unwrap()
                }
            }
        )*
    };
}

from_string! { I8, I16, I32, I64, I128, I256, I384, I512 }
from_string! { U8, U16, U32, U64, U128, U256, U384, U512 }

macro_rules! big_int_from {
    (U256) => {
        to_big_int_from_large_unsigned! {U256}
    };
    (I256) => {
        to_big_int_from_large_signed! {I256}
    };
    (U384) => {
        to_big_int_from_large_unsigned! {U384}
    };
    (I384) => {
        to_big_int_from_large_signed! {I384}
    };
    (U512) => {
        to_big_int_from_large_unsigned! {U512}
    };
    (I512) => {
        to_big_int_from_large_signed! {I512}
    };
    ($t:ident) => {
        to_big_int_from_small! {$t}
    };
}

macro_rules! to_big_int_from_large_unsigned {
    ($t:ident) => {
        impl From<$t> for BigInt {
            fn from(val: $t) -> BigInt {
                BigInt::from_bytes_le(Sign::Plus, &val.0)
            }
        }
    };
}

macro_rules! to_big_int_from_large_signed {
    ($t:ident) => {
        impl From<$t> for BigInt {
            fn from(val: $t) -> BigInt {
                BigInt::from_signed_bytes_le(&val.0)
            }
        }
    };
}

macro_rules! to_big_int_from_small {
    ($t:ident) => {
        impl From<$t> for BigInt {
            fn from(val: $t) -> BigInt {
                BigInt::from(val.0)
            }
        }
    };
}

big_int_from! {I8}
big_int_from! {I16}
big_int_from! {I32}
big_int_from! {I64}
big_int_from! {I128}
big_int_from! {I256}
big_int_from! {I384}
big_int_from! {I512}
big_int_from! {U8}
big_int_from! {U16}
big_int_from! {U32}
big_int_from! {U64}
big_int_from! {U128}
big_int_from! {U256}
big_int_from! {U384}
big_int_from! {U512}

macro_rules! array_from_large {
    ($($t:ident),*) => {
        $(
            impl $t {
                pub fn to_le_bytes(&self) -> [u8; (<$t>::BITS / 8) as usize] {
                    self.0
                }
            }
        )*
    };
}

macro_rules! array_from_small {
    ($($t:ident),*) => {
        $(
            impl $t {
                pub fn to_le_bytes(&self) -> [u8; (<$t>::BITS / 8) as usize] {
                    self.0.to_le_bytes()
                }
            }
        )*
    };
}

array_from_large! {I256, I384, I512, U256, U384, U512}
array_from_small! {I8, I16, I32, I64, I128, U8, U16, U32, U64, U128}
