use super::*;
use num_bigint::{BigInt, ParseBigIntError, Sign};
use num_traits::{Signed, ToPrimitive};
use paste::paste;
use sbor::rust::convert::{From, TryFrom};
use sbor::rust::str::FromStr;
use sbor::rust::string::String;

pub mod conv_primitive;
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
                impl conv_primitive::FromPrimitive for $t {
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

                impl conv_primitive::ToPrimitive for $t {
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

macro_rules! error {
    ($($t:ident),*) => {
        paste! {
            $(
                #[derive(Debug, Clone, PartialEq, Eq)]
                pub enum [<Parse $t Error>] {
                    NegativeToUnsigned,
                    Overflow,
                    InvalidLength,
                }

                #[cfg(not(feature = "alloc"))]
                impl std::error::Error for [<Parse $t Error>] {}

                #[cfg(not(feature = "alloc"))]
                impl fmt::Display for [<Parse $t Error>] {
                    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                        write!(f, "{:?}", self)
                    }
                }
            )*
        }
    };
}

error! { i8, i16, i32, i64, isize, i128, u8, u16, u32, u64, usize, u128, I8, I16, I32, I64, I128, I256, I384, I512, U8, U16, U32, U64, U128, U256, U384, U512 }

macro_rules! try_from_large_into_large{
    ($t:ident, $wrapped:ty, ($($o:ident),*)) => {
        $(
            paste! {
                impl TryFrom<$o> for $t {
                    type Error = [<Parse $t Error>];
                    fn try_from(val: $o) -> Result<$t, [<Parse $t Error>]> {
                        let mut sign = <$t>::one();
                        let mut other = val;
                        if other < <$o>::zero() {
                            if <$t>::MIN == <$t>::zero() {
                                return Err([<Parse $t Error>]::NegativeToUnsigned);
                            } else {
                                sign = <$t>::zero() - sign;
                                other = <$o>::zero() - other;
                            }
                        }
                        if (other.leading_zeros() as i32) < <$o>::BITS as i32 - <$t>::BITS as i32 {
                            return Err([<Parse $t Error>]::Overflow);
                        }
                        let mut other_vec = other.0.to_vec();
                        other_vec.resize((<$t>::BITS / 8) as usize, 0);
                        Ok($t(other_vec.try_into().unwrap()) * sign)
                    }
                }

                impl TFrom<$o> for $t {
                    type Output = $t;
                    fn tfrom(val: $o) -> Self::Output {
                        Self::Output::try_from(val).unwrap()
                    }
                }
            }
        )*
    };
}
try_from_large_into_large! { I256, [u8; 32], (I384, I512, U256, U384, U512) }
try_from_large_into_large! { I384, [u8; 48], (I512, U384, U512) }
try_from_large_into_large! { I512, [u8; 64], (U512) }
try_from_large_into_large! { U256, [u8; 32], (I256, I384, I512, U384, U512) }
try_from_large_into_large! { U384, [u8; 48], (I256, I384, I512, U512) }
try_from_large_into_large! { U512, [u8; 64], (I256, I384, I512) }

macro_rules! try_from_small_into_large{
    ($t:ident, $wrapped:ty, ($($o:ident),*)) => {
        $(
            paste! {
                impl TryFrom<$o> for $t {
                    type Error = [<Parse $t Error>];
                    fn try_from(val: $o) -> Result<$t, [<Parse $t Error>]> {
                        let mut sign = <$t>::one();
                        let mut other = val;
                        if val < <$o>::zero() {
                            if <$t>::MIN == <$t>::zero() {
                                return Err([<Parse $t Error>]::NegativeToUnsigned);
                            } else {
                                sign = <$t>::zero() - <$t>::one();
                                other = <$o>::zero() - other;
                            }
                        }
                        let mut other_vec = other.0.to_le_bytes().to_vec();
                        other_vec.resize((<$t>::BITS / 8) as usize, 0);
                        Ok($t(other_vec.try_into().unwrap()) * sign)
                    }
                }

                impl TFrom<$o> for $t {
                    type Output = $t;
                    fn tfrom(val: $o) -> Self::Output {
                        Self::Output::try_from(val).unwrap()
                    }
                }
            }
        )*
    };
}

try_from_small_into_large! { U256, [u8; 32], (I8, I16, I32, I64, I128)}
try_from_small_into_large! { U384, [u8; 48], (I8, I16, I32, I64, I128)}
try_from_small_into_large! { U512, [u8; 64], (I8, I16, I32, I64, I128)}

macro_rules! try_from_large_into_small{
    ($t:ident, $wrapped:ident, ($($o:ident),*)) => {
        $(
            paste! {
                impl TryFrom<$o> for $t {
                    type Error = [<Parse $t Error>];
                    fn try_from(val: $o) -> Result<$t, [<Parse $t Error>]> {
                        let mut sign: $wrapped = <$wrapped>::try_from(1).unwrap();
                        let mut other = val;
                        if val < <$o>::zero() {
                            if <$t>::MIN == <$t>::zero() {
                                return Err([<Parse $t Error>]::NegativeToUnsigned);
                            } else {
                                sign = <$wrapped>::zero() - sign;
                                other = <$o>::zero() - other;
                            }
                        }
                        if (other.leading_zeros() as i32) < <$o>::BITS as i32 - <$t>::BITS as i32 {
                            return Err([<Parse $t Error>]::Overflow);
                        }
                        let mut other_vec = other.0.to_vec();
                        other_vec.resize((<$t>::BITS / 8) as usize, 0);
                        let res_array: [u8; (<$t>::BITS / 8) as usize] = other_vec.try_into().unwrap();
                        Ok($t($wrapped::from_le_bytes(res_array) * sign))
                    }
                }

                impl TFrom<$o> for $t {
                    type Output = $t;
                    fn tfrom(val: $o) -> Self::Output {
                        Self::Output::try_from(val).unwrap()
                    }
                }
            }
        )*
    };
}
try_from_large_into_small! {I8, i8, (I256, I384, I512, U256, U384, U512)}
try_from_large_into_small! {I16, i16, (I256, I384, I512, U256, U384, U512)}
try_from_large_into_small! {I32, i32, (I256, I384, I512, U256, U384, U512)}
try_from_large_into_small! {I64, i64, (I256, I384, I512, U256, U384, U512)}
try_from_large_into_small! {I128, i128, (I256, I384, I512, U256, U384, U512)}
try_from_large_into_small! {U8, u8, (I256, I384, I512, U256, U384, U512)}
try_from_large_into_small! {U16, u16, (I256, I384, I512, U256, U384, U512)}
try_from_large_into_small! {U32, u32, (I256, I384, I512, U256, U384, U512)}
try_from_large_into_small! {U64, u64, (I256, I384, I512, U256, U384, U512)}
try_from_large_into_small! {U128, u128, (I256, I384, I512, U256, U384, U512)}

macro_rules! try_from_small_into_small{
    ($t:ident, $wrapped:ident, ($($o:ident),*)) => {
        $(
            paste! {
                impl TryFrom<$o> for $t {
                    type Error = [<Parse $t Error>];
                    fn try_from(val: $o) -> Result<$t, [<Parse $t Error>]> {
                        let res = val.0.try_into();
                        match res {
                            Ok(val) => Ok($t(val)),
                            Err(_) => Err([<Parse $t Error>]::Overflow)
                        }
                    }
                }
                impl TFrom<$o> for $t {
                    type Output = $t;
                    fn tfrom(val: $o) -> Self::Output {
                        Self::Output::try_from(val).unwrap()
                    }
                }
            }
        )*
    };
}
try_from_small_into_small! { I8, i8, (U8, U16, U32, U64, U128, I16, I32, I64, I128) }
try_from_small_into_small! { I16, i16, (U16, U32, U64, U128, I32, I64, I128) }
try_from_small_into_small! { I32, i32, (U32, U64, U128, I64, I128) }
try_from_small_into_small! { I64, i64, (U64, U128, I128) }
try_from_small_into_small! { I128, i128, (U128) }
try_from_small_into_small! { U8, u8, (U16, U32, U64, U128, I8, I16, I32, I64, I128) }
try_from_small_into_small! { U16, u16, (U32, U64, U128, I8, I16, I32, I64, I128) }
try_from_small_into_small! { U32, u32, (U64, U128, I8, I16, I32, I64, I128) }
try_from_small_into_small! { U64, u64, (U128, I8, I16, I32, I64, I128) }
try_from_small_into_small! { U128, u128, (I8, I16, I32, I64, I128) }

macro_rules! try_from_builtin_into_small{
    ($t:ident, $wrapped:ident, ($($o:ident),*)) => {
        $(
            paste! {
                impl TryFrom<$o> for $t {
                    type Error = [<Parse $t Error>];
                    fn try_from(val: $o) -> Result<$t, [<Parse $t Error>]> {
                        let res = val.try_into();
                        match res {
                            Ok(val) => Ok($t(val)),
                            Err(_) => Err([<Parse $t Error>]::Overflow)
                        }
                    }
                }
                impl TFrom<$o> for $t {
                    type Output = $t;
                    fn tfrom(val: $o) -> Self::Output {
                        Self::Output::try_from(val).unwrap()
                    }
                }
            }
        )*
    };
}
try_from_builtin_into_small! { I8, i8, (u8, u16, u32, u64, usize, u128, i16, i32, i64, isize, i128) }
try_from_builtin_into_small! { I16, i16, (u16, u32, u64, usize, u128, i32, i64, isize, i128) }
try_from_builtin_into_small! { I32, i32, (u32, u64, usize, u128, i64, isize, i128) }
try_from_builtin_into_small! { I64, i64, (u64, usize, u128, i128) }
try_from_builtin_into_small! { I128, i128, (u128) }
try_from_builtin_into_small! { U8, u8, (u16, u32, u64, usize, u128, i8, i16, i32, i64, isize, i128) }
try_from_builtin_into_small! { U16, u16, (u32, u64, usize, u128, i8, i16, i32, i64, isize, i128) }
try_from_builtin_into_small! { U32, u32, (u64, usize, u128, i8, i16, i32, i64, isize, i128) }
try_from_builtin_into_small! { U64, u64, (u128, i8, i16, i32, i64, isize, i128) }
try_from_builtin_into_small! { U128, u128, (i8, i16, i32, i64, isize, i128) }

macro_rules! try_from_builtin_into_large{
    ($t:ident, $wrapped:ty, ($($o:ident),*)) => {
        $(
            paste! {
                impl TryFrom<$o> for $t {
                    type Error = [<Parse $t Error>];
                    fn try_from(val: $o) -> Result<$t, [<Parse $t Error>]> {
                        let mut sign = <$t>::one();
                        let mut other = val;
                        if val < <$o>::zero() {
                            if <$t>::MIN == <$t>::zero() {
                                return Err([<Parse $t Error>]::NegativeToUnsigned);
                            } else {
                                sign = <$t>::zero() - <$t>::one();
                                other = <$o>::zero() - other;
                            }
                        }
                        let mut other_vec = other.to_le_bytes().to_vec();
                        other_vec.resize((<$t>::BITS / 8) as usize, 0);
                        Ok($t(other_vec.try_into().unwrap()) * sign)
                    }
                }

                impl TFrom<$o> for $t {
                    type Output = $t;
                    fn tfrom(val: $o) -> Self::Output {
                        Self::Output::try_from(val).unwrap()
                    }
                }
            }
        )*
    };
}

try_from_builtin_into_large! { U256, [u8; 32], (i8, i16, i32, i64, isize, i128)}
try_from_builtin_into_large! { U384, [u8; 48], (i8, i16, i32, i64, isize, i128)}
try_from_builtin_into_large! { U512, [u8; 64], (i8, i16, i32, i64, isize, i128)}

macro_rules! try_from_small_into_builtin{
    ($t:ident, ($($o:ident),*)) => {
        $(
            paste! {
                impl TryFrom<$o> for $t {
                    type Error = [<Parse $t Error>];
                    fn try_from(val: $o) -> Result<$t, [<Parse $t Error>]> {
                        let res = val.0.try_into();
                        match res {
                            Ok(val) => Ok(val),
                            Err(_) => Err([<Parse $t Error>]::Overflow)
                        }
                    }
                }
                impl TFrom<$o> for $t {
                    type Output = $t;
                    fn tfrom(val: $o) -> Self::Output {
                        Self::Output::try_from(val).unwrap()
                    }
                }
            }
        )*
    };
}
try_from_small_into_builtin! { i8, (U8, U16, U32, U64, U128, I16, I32, I64, I128) }
try_from_small_into_builtin! { i16, (U16, U32, U64, U128, I32, I64, I128) }
try_from_small_into_builtin! { i32, (U32, U64, U128, I64, I128) }
try_from_small_into_builtin! { i64, (U64, U128, I128) }
try_from_small_into_builtin! { i128, (U128) }
try_from_small_into_builtin! { u8, (U16, U32, U64, U128, I8, I16, I32, I64, I128) }
try_from_small_into_builtin! { u16, (U32, U64, U128, I8, I16, I32, I64, I128) }
try_from_small_into_builtin! { u32, (U64, U128, I8, I16, I32, I64, I128) }
try_from_small_into_builtin! { u64, (U128, I8, I16, I32, I64, I128) }
try_from_small_into_builtin! { u128, (I8, I16, I32, I64, I128) }

macro_rules! try_from_large_into_builtin{
    ($t:ident, ($($o:ident),*)) => {
        $(
            paste! {
                impl TryFrom<$o> for $t {
                    type Error = [<Parse $t Error>];
                    fn try_from(val: $o) -> Result<$t, [<Parse $t Error>]> {
                        let mut sign = <$t>::try_from(1).unwrap();
                        let mut other = val;
                        if val < <$o>::zero() {
                            if <$t>::MIN == <$t>::zero() {
                                return Err([<Parse $t Error>]::NegativeToUnsigned);
                            } else {
                                sign = <$t>::zero() - sign;
                                other = <$o>::zero() - other;
                            }
                        }
                        if (other.leading_zeros() as i32) < <$o>::BITS as i32 - <$t>::BITS as i32 {
                            return Err([<Parse $t Error>]::Overflow);
                        }
                        let mut other_vec = other.0.to_vec();
                        other_vec.resize((<$t>::BITS / 8) as usize, 0);
                        let res_array: [u8; (<$t>::BITS / 8) as usize] = other_vec.try_into().unwrap();
                        Ok($t::from_le_bytes(res_array) * sign)
                    }
                }

                impl TFrom<$o> for $t {
                    type Output = $t;
                    fn tfrom(val: $o) -> Self::Output {
                        Self::Output::try_from(val).unwrap()
                    }
                }
            }
        )*
    };
}
try_from_large_into_builtin! {i8, (I256, I384, I512, U256, U384, U512)}
try_from_large_into_builtin! {i16, (I256, I384, I512, U256, U384, U512)}
try_from_large_into_builtin! {i32, (I256, I384, I512, U256, U384, U512)}
try_from_large_into_builtin! {i64, (I256, I384, I512, U256, U384, U512)}
try_from_large_into_builtin! {i128, (I256, I384, I512, U256, U384, U512)}
try_from_large_into_builtin! {u8, (I256, I384, I512, U256, U384, U512)}
try_from_large_into_builtin! {u16, (I256, I384, I512, U256, U384, U512)}
try_from_large_into_builtin! {u32, (I256, I384, I512, U256, U384, U512)}
try_from_large_into_builtin! {u64, (I256, I384, I512, U256, U384, U512)}
try_from_large_into_builtin! {u128, (I256, I384, I512, U256, U384, U512)}

// macro_rules! try_from{
//     ($t:ident, ($($o:ident),*)) => {
//         $(
//             paste! {
//                 impl TryFrom<$o> for $t {
//                     type Error = [<Parse $t Error>];
//                     fn try_from(val: $o) -> Result<$t, [<Parse $t Error>]> {
//                         BigInt::from(val).try_into().map_err(|_| [<Parse $t Error>]::Overflow)
//                     }
//                 }
//                 impl TFrom<$o> for $t {
//                     type Output = $t;
//                     fn tfrom(val: $o) -> Self::Output {
//                         Self::Output::try_from(val).unwrap()
//                     }
//                 }
//             }
//         )*
//     };
// }
//
// macro_rules! try_from_large_all {
//     ($($t:ident),*) => {
//         $(
//             try_from! { $t, (I256, I384, I512) }
//             try_from! { $t, (U256, U384, U512) }
//         )*
//     };
// }
//
// try_from_large_all! { U8, U16, U32, U64, U128, I8, I16, I32, I64, I128 }
//
// try_from! {u8, (U16, U32, U64, U128, U256, U384, U512, I8, I16, I32, I64, I128, I256, I384, I512)}
// try_from! {u16, (U32, U64, U128, U256, U384, U512, I8, I16, I32, I64, I128, I256, I384, I512)}
// try_from! {u32, (U64, U128, U256, U384, U512, I8, I16, I32, I64, I128, I256, I384, I512)}
// try_from! {u64, (U128, U256, U384, U512, I8, I16, I32, I64, I128, I256, I384, I512)}
// try_from! {usize, (U128, U256, U384, U512, I8, I16, I32, I64, I128, I256, I384, I512)}
// try_from! {u128, (U256, U384, U512, I8, I16, I32, I64, I128, I256, I384, I512)}
// try_from! {i8, (U8, U16, U32, U64, U128, U256, U384, U512, I16, I32, I64, I128, I256, I384, I512)}
// try_from! {i16, ( U16, U32, U64, U128, U256, U384, U512, I32, I64, I128, I256, I384, I512)}
// try_from! {i32, (U32, U64, U128, U256, U384, U512, I64, I128, I256, I384, I512)}
// try_from! {i64, (U64, U128, U256, U384, U512, I128, I256, I384, I512)}
// try_from! {isize, (U64, U128, U256, U384, U512, I128, I256, I384, I512)}
// try_from! {i128, (U128, U256, U384, U512, I256, I384, I512)}
//
// try_from! {U8, (I8, I16, I32, I64, I128)}
// try_from! {U8, (U16, U32, U64, U128)}
// try_from! {U8, (i8, i16, i32, i64, isize, i128, u16, u32, u64, usize, u128)}
// try_from! {U16, (I8, I16, I32, I64, I128)}
// try_from! {U16, (U32, U64, U128)}
// try_from! {U16, (i8, i16, i32, i64, isize, i128, u32, u64, usize, u128)}
// try_from! {U32, (I8, I16, I32, I64, I128)}
// try_from! {U32, (U64, U128)}
// try_from! {U32, (i8, i16, i32, i64, isize, i128, u64, usize, u128)}
// try_from! {U64, (I8, I16, I32, I64, I128)}
// try_from! {U64, (U128)}
// try_from! {U64, (i8, i16, i32, i64, isize, i128, u128)}
// try_from! {U128, (I8, I16, I32, I64, I128)}
// try_from! {U128, (i8, i16, i32, i64, isize, i128)}
// try_from! {U256, (I8, I16, I32, I64, I128, I256, I384, I512)}
// try_from! {U256, (U384, U512)}
// try_from! {U256, (i8, i16, i32, i64, isize, i128)}
// try_from! {U384, (I8, I16, I32, I64, I128, I256, I384, I512)}
// try_from! {U384, (U512)}
// try_from! {U384, (i8, i16, i32, i64, isize, i128)}
// try_from! {U512, (I8, I16, I32, I64, I128, I256, I384, I512)}
// try_from! {U512, (i8, i16, i32, i64, isize, i128)}
//
// try_from! {I8, (I16, I32, I64, I128)}
// try_from! {I8, (U8, U16, U32, U64, U128)}
// try_from! {I8, (u8, u16, u32, u64, usize, u128, i16, i32, i64, isize, i128)}
// try_from! {I16, (I32, I64, I128)}
// try_from! {I16, (U16, U32, U64, U128)}
// try_from! {I16, (i32, i64, isize, i128, u16, u32, u64, usize, u128)}
// try_from! {I32, (I64, I128)}
// try_from! {I32, (U32, U64, U128)}
// try_from! {I32, (i64, isize, i128, u32, u64, usize, u128)}
// try_from! {I64, (I128)}
// try_from! {I64, (U64, U128)}
// try_from! {I64, (i128, u64, usize, u128)}
// try_from! {I128, (U128)}
// try_from! {I128, (u128)}
// try_from! {I256, (I384, I512)}
// try_from! {I256, (U256, U384, U512)}
// try_from! {I384, (I512)}
// try_from! {I384, (U384, U512)}
// try_from! {I512, (U512)}

macro_rules! impl_others_to_large_unsigned {
    ($($t:ty),*) => {
        $(
            paste! {
                impl TryFrom<BigInt> for $t {
                    type Error = [<Parse $t Error>];
                    fn try_from(val: BigInt) -> Result<$t, [<Parse $t Error>]> {
                        let (sign, bytes) = val.to_bytes_le();
                        if sign == Sign::Minus {
                            return Err([<Parse $t Error>]::NegativeToUnsigned);
                        }
                        const T_BYTES: usize = (<$t>::BITS / 8 ) as usize;
                        if bytes.len() > T_BYTES {
                            return Err([<Parse $t Error>]::Overflow);
                        }
                        let mut buf = [0u8; T_BYTES];
                        buf[..bytes.len()].copy_from_slice(&bytes);
                        Ok($t(buf))
                    }
                }

                impl TFrom<BigInt> for $t {
                    type Output = $t;
                    fn tfrom(val: BigInt) -> Self::Output {
                        Self::Output::try_from(val).unwrap()
                    }
                }

                impl TryFrom<&[u8]> for $t {
                    type Error = [<Parse $t Error>];
                    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
                        if bytes.len() > (<$t>::BITS / 8) as usize {
                            Err([<Parse $t Error>]::InvalidLength)
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
                        Self::Output::try_from(val).unwrap()
                    }
                }


                impl TryFrom<Vec<u8>> for $t {
                    type Error = [<Parse $t Error>];
                    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
                        if bytes.len() > (<$t>::BITS / 8) as usize {
                            Err([<Parse $t Error>]::InvalidLength)
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
                        Self::Output::try_from(val).unwrap()
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
                    type Error = [<Parse $t Error>];
                    fn try_from(val: BigInt) -> Result<$t, [<Parse $t Error>]> {
                        let bytes = val.to_signed_bytes_le();
                        const T_BYTES: usize = (<$t>::BITS / 8 ) as usize;
                        if bytes.len() > T_BYTES {
                            return Err([<Parse $t Error>]::Overflow);
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
                        Self::Output::try_from(val).unwrap()
                    }
                }

                impl TryFrom<&[u8]> for $t {
                    type Error = [<Parse $t Error>];
                    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
                        const T_BYTES: usize = (<$t>::BITS / 8 ) as usize;
                        if bytes.len() > T_BYTES {
                            Err([<Parse $t Error>]::InvalidLength)
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
                        Self::Output::try_from(val).unwrap()
                    }
                }


                impl TryFrom<Vec<u8>> for $t {
                    type Error = [<Parse $t Error>];
                    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
                        const T_BYTES: usize = (<$t>::BITS / 8 ) as usize;
                        if bytes.len() > T_BYTES {
                            Err([<Parse $t Error>]::InvalidLength)
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
                        Self::Output::try_from(val).unwrap()
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
                    type Error = [<Parse $t Error>];
                    fn try_from(val: BigInt) -> Result<$t, [<Parse $t Error>]> {
                        if val.is_negative() {
                            return Err([<Parse $t Error>]::NegativeToUnsigned);
                        }
                        Ok($t(val.[<to_$t:lower>]().ok_or_else(|| [<Parse $t Error>]::Overflow).unwrap()))
                    }
                }

                impl TFrom<BigInt> for $t {
                    type Output = $t;
                    fn tfrom(val: BigInt) -> Self::Output {
                        Self::Output::try_from(val).unwrap()
                    }
                }

                impl TryFrom<&[u8]> for $t {
                    type Error = [<Parse $t Error>];
                    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
                        const T_BYTES: usize = (<$t>::BITS / 8 ) as usize;
                        if bytes.len() > T_BYTES {
                            Err([<Parse $t Error>]::InvalidLength)
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
                        Self::Output::try_from(val).unwrap()
                    }
                }

                impl TryFrom<Vec<u8>> for $t {
                    type Error = [<Parse $t Error>];
                    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
                        const T_BYTES: usize = (<$t>::BITS / 8 ) as usize;
                        if bytes.len() > T_BYTES {
                            Err([<Parse $t Error>]::InvalidLength)
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
                        Self::Output::try_from(val).unwrap()
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
                    type Error = [<Parse $t Error>];
                    fn try_from(val: BigInt) -> Result<$t, [<Parse $t Error>]> {
                        Ok($t(val.[<to_$t:lower>]().ok_or_else(|| [<Parse $t Error>]::Overflow).unwrap()))
                    }
                }

                impl TFrom<BigInt> for $t {
                    type Output = $t;
                    fn tfrom(val: BigInt) -> Self::Output {
                        Self::Output::try_from(val).unwrap()
                    }
                }

                impl TryFrom<&[u8]> for $t {
                    type Error = [<Parse $t Error>];
                    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
                        const T_BYTES: usize = (<$t>::BITS / 8 ) as usize;
                        if bytes.len() > T_BYTES {
                            Err([<Parse $t Error>]::InvalidLength)
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
                        Self::Output::try_from(val).unwrap()
                    }
                }

                impl TryFrom<Vec<u8>> for $t {
                    type Error = [<Parse $t Error>];
                    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
                        const T_BYTES: usize = (<$t>::BITS / 8 ) as usize;
                        if bytes.len() > T_BYTES {
                            Err([<Parse $t Error>]::InvalidLength)
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
                        Self::Output::try_from(val).unwrap()
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
                        Self::Output::try_from(val).unwrap()
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
                        Self::Output::try_from(val).unwrap()
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

macro_rules! from_large_into_large {
    ($t:ident, ($($o:ident),*)) => {
        $(
            paste! {
                impl From<$o> for $t {
                    fn from(val: $o) -> Self {
                        let mut sign = <$t>::one();
                        let mut other = val;
                        if other < <$o>::zero() {
                                sign = <$t>::zero() - sign;
                                other = <$o>::zero() - other;
                        }
                        let mut other_vec = other.0.to_vec();
                        other_vec.resize((<$t>::BITS / 8) as usize, 0);
                        Self(other_vec.try_into().unwrap()) * sign
                    }
                }

                impl TFrom<$o> for $t {
                    type Output = $t;
                    fn tfrom(val: $o) -> Self::Output {
                        Self::Output::try_from(val).unwrap()
                    }
                }
            }
        )*
    };
}
from_large_into_large! { I384, (I256, U256) }
from_large_into_large! { I512, (I256, I384, U256, U384) }
from_large_into_large! { U384, (U256) }
from_large_into_large! { U512, (U256, U384) }

macro_rules! from_small_into_large{
    ($t:ident, ($($o:ident),*)) => {
        $(
            paste! {
                impl From<$o> for $t {
                    fn from(val: $o) -> Self {
                        let mut sign = <$t>::one();
                        let mut other = val;
                        if val < <$o>::zero() {
                            sign = <$t>::zero() - <$t>::one();
                            other = <$o>::zero() - other;
                        }
                        let mut other_vec = other.0.to_le_bytes().to_vec();
                        other_vec.resize((<$t>::BITS / 8) as usize, 0);
                        Self(other_vec.try_into().unwrap()) * sign
                    }
                }

                impl TFrom<$o> for $t {
                    type Output = $t;
                    fn tfrom(val: $o) -> Self::Output {
                        Self::Output::try_from(val).unwrap()
                    }
                }
            }
        )*
    };
}

from_small_into_large! { I256, (I8, I16, I32, I64, I128, U8, U16, U32, U64, U128) }
from_small_into_large! { I384, (I8, I16, I32, I64, I128, U8, U16, U32, U64, U128) }
from_small_into_large! { I512, (I8, I16, I32, I64, I128, U8, U16, U32, U64, U128) }
from_small_into_large! { U256, (U8, U16, U32, U64, U128) }
from_small_into_large! { U384, (U8, U16, U32, U64, U128) }
from_small_into_large! { U512, (U8, U16, U32, U64, U128) }

macro_rules! from_small_into_small{
    ($t:ident, ($($o:ident),*)) => {
        $(
            paste! {
                impl From<$o> for $t {
                    fn from(val: $o) -> Self {
                        Self(val.0.into())
                    }
                }

                impl TFrom<$o> for $t {
                    type Output = $t;
                    fn tfrom(val: $o) -> Self::Output {
                        Self::Output::try_from(val).unwrap()
                    }
                }
            }
        )*
    };
}

from_small_into_small! { I16, (I8, U8) }
from_small_into_small! { I32, (I8, I16, U8, U16) }
from_small_into_small! { I64, (I8, I16, I32, U8, U16, U32) }
from_small_into_small! { I128, (I8, I16, I32, I64, U8, U16, U32, U64) }
from_small_into_small! { U16, (U8) }
from_small_into_small! { U32, (U8, U16) }
from_small_into_small! { U64, (U8, U16, U32) }
from_small_into_small! { U128, (U8, U16, U32, U64) }

macro_rules! from_builtin_into_small{
    ($t:ident, ($($o:ident),*)) => {
        $(
            paste! {
                impl From<$o> for $t {
                    fn from(val: $o) -> Self {
                        Self(val.into())
                    }
                }
                impl TFrom<$o> for $t {
                    type Output = $t;
                    fn tfrom(val: $o) -> Self::Output {
                        Self::Output::try_from(val).unwrap()
                    }
                }
            }
        )*
    };
}
from_builtin_into_small! { I8, (i8) }
from_builtin_into_small! { I16, (i8, i16, u8) }
from_builtin_into_small! { I32, (i8, i16, i32, u8, u16) }
from_builtin_into_small! { I64, (i8, i16, i32, i64, u8, u16, u32) }
from_builtin_into_small! { I128, (i8, i16, i32, i64, i128, u8, u16, u32, u64) }
from_builtin_into_small! { U8, (u8) }
from_builtin_into_small! { U16, (u8, u16) }
from_builtin_into_small! { U32, (u8, u16, u32) }
from_builtin_into_small! { U64, (u8, u16, u32, u64) }
from_builtin_into_small! { U128, (u8, u16, u32, u64, u128) }

macro_rules! from_builtin_into_large {
    ($t:ident, ($($o:ident),*)) => {
        $(
            paste! {
                impl From<$o> for $t {
                    fn from(val: $o) -> Self {
                        let mut sign = <$t>::one();
                        let mut other = val;
                        if val < <$o>::zero() {
                                sign = <$t>::zero() - <$t>::one();
                                other = <$o>::zero() - other;
                        }
                        let mut other_vec = other.to_le_bytes().to_vec();
                        other_vec.resize((<$t>::BITS / 8) as usize, 0);
                        Self(other_vec.try_into().unwrap()) * sign
                    }
                }

                impl TFrom<$o> for $t {
                    type Output = $t;
                    fn tfrom(val: $o) -> Self::Output {
                        Self::Output::try_from(val).unwrap()
                    }
                }
            }
        )*
    };
}
from_builtin_into_large! { I256, (i8, i16, i32, i64, isize, i128, u8, u16, u32, u64, usize, u128) }
from_builtin_into_large! { I384, (i8, i16, i32, i64, isize, i128, u8, u16, u32, u64, usize, u128) }
from_builtin_into_large! { I512, (i8, i16, i32, i64, isize, i128, u8, u16, u32, u64, usize, u128) }
from_builtin_into_large! { U256, (u8, u16, u32, u64, usize, u128) }
from_builtin_into_large! { U384, (u8, u16, u32, u64, usize, u128) }
from_builtin_into_large! { U512, (u8, u16, u32, u64, usize, u128) }

macro_rules! try_from_small_into_builtin{
    ($t:ident, ($($o:ident),*)) => {
        $(
            paste! {
                impl From<$o> for $t {
                    fn from(val: $o) -> Self {
                        val.0.into()
                    }
                }
                impl TFrom<$o> for $t {
                    type Output = $t;
                    fn tfrom(val: $o) -> Self::Output {
                        Self::Output::try_from(val).unwrap()
                    }
                }
            }
        )*
    };
}
try_from_small_into_builtin! { i8, (I8) }
try_from_small_into_builtin! { i16, (I8, I16, U8) }
try_from_small_into_builtin! { i32, (I8, I16, I32, U8, U16) }
try_from_small_into_builtin! { i64, (I8, I16, I32, I64, U8, U16, U32) }
try_from_small_into_builtin! { i128, (I8, I16, I32, I64, I128, U8, U16, U32, U64) }
try_from_small_into_builtin! { u8, (U8) }
try_from_small_into_builtin! { u16, (U8, U16) }
try_from_small_into_builtin! { u32, (U8, U16, U32) }
try_from_small_into_builtin! { u64, (U8, U16, U32, U64) }
try_from_small_into_builtin! { u128, (U8, U16, U32, U64, U128) }

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
                    Self::Output::try_from(val).unwrap()
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
                    Self::Output::try_from(val).unwrap()
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

macro_rules! seq_from_large {
    ($($t:ident),*) => {
        $(
            impl $t {
                pub fn to_le_bytes(&self) -> [u8; (<$t>::BITS / 8) as usize] {
                    self.0
                }

                pub fn to_vec(&self) -> Vec<u8> {
                    self.0.to_vec()
                }
            }
        )*
    };
}

macro_rules! seq_from_small {
    ($($t:ident),*) => {
        $(
            impl $t {
                pub fn to_le_bytes(&self) -> [u8; (<$t>::BITS / 8) as usize] {
                    self.0.to_le_bytes()
                }

                pub fn to_vec(&self) -> Vec<u8> {
                    self.0.to_le_bytes().to_vec()
                }
            }
        )*
    };
}

seq_from_large! {I256, I384, I512, U256, U384, U512}
seq_from_small! {I8, I16, I32, I64, I128, U8, U16, U32, U64, U128}
