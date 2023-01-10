//! Definitions of safe integers and uints.

use num_bigint::{BigInt, Sign};
use num_traits::FromPrimitive;
use num_traits::{One, Pow, Signed, ToPrimitive, Zero};
use paste::paste;
use sbor::rust::cmp::{Ord, Ordering, PartialEq, PartialOrd};
use sbor::rust::convert::{From, TryFrom};
use sbor::rust::fmt;
use sbor::rust::ops::{Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign};
use sbor::rust::ops::{BitXor, BitXorAssign, Div, DivAssign};
use sbor::rust::ops::{Mul, MulAssign, Neg, Not, Rem, RemAssign};
use sbor::rust::ops::{Shl, ShlAssign, Shr, ShrAssign, Sub, SubAssign};
use sbor::rust::str::FromStr;
use sbor::rust::string::*;
use sbor::rust::vec;
use sbor::rust::vec::Vec;
use sbor::value_kind::*;
use sbor::*;

pub mod basic;
pub mod bits;
pub mod convert;
pub mod test;

use crate::abi::*;
use convert::*;

macro_rules! types {

    (self: $self:ident,
     $(
         {
             type: $t:ident,
             self.0: $wrap:ty,
             self.zero(): $tt:ident($zero:expr),
             $ttt:ident::default(): $default:expr,
             format_var: $f:ident,
             format_expr: $fmt:expr,
         }
     ),*) => {
        paste!{
            $(
                /// Provides safe integer arithmetic.
                ///
                /// Operations like `+`, '-', '*', or '/' sometimes produce overflow
                /// which is detected and results in a panic, instead of silently
                /// wrapping around.
                ///
                /// The bit length of output type will be the greater one in the math operation,
                /// and if any of the types was signed, then the resulting type will be signed too,
                /// otherwise the output type is unsigned.
                ///
                /// The underlying value can be retrieved through the `.0` index of the
                #[doc = "`" $t "` tuple."]
                ///
                /// # Layout
                ///
                #[doc = "`" $t "` will have the same methods and traits as"]
                /// the built-in counterpart.
                #[derive(Clone , Copy , Eq , Hash)]
                #[repr(transparent)]
                pub struct $t(pub $wrap);

            impl Default for $t {
                fn default() -> Self {
                    $default
                }
            }

            impl fmt::Debug for $t {
                fn fmt(&$self, $f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    $fmt
                }
            }

            impl fmt::Display for $t {
                fn fmt(&$self, $f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    $fmt
                }
            }

            impl Zero for $t {
                fn zero() -> Self {
                    Self($zero)
                }

                fn is_zero(&self) -> bool {
                    $zero == self.0
                }

                fn set_zero(&mut self) {
                    self.0 = $zero;
                }
            }

            impl One for $t {
                fn one() -> Self {
                    Self::try_from(1u8).unwrap()
                }
            }

            impl Ord for $t {
                fn cmp(&self, other: &Self) -> Ordering {
                   let mut a: Vec<u8> = self.to_le_bytes().into();
                   let mut b: Vec<u8> = other.to_le_bytes().into();
                   a.reverse();
                   b.reverse();
                   if Self::MIN != Zero::zero() {
                       a[0] ^= 0x80;
                       b[0] ^= 0x80;
                   }
                   a.cmp(&b)
                }
            }

            impl PartialOrd for $t {
                fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                    Some(self.cmp(other))
                }
            }

            impl PartialEq for $t {
                fn eq(&self, other: &Self) -> bool {
                    self.0 == other.0
                }
            }

            #[cfg(test)]
            impl $t {
                pub fn type_name(self) -> &'static str {
                    stringify!($t)
                }
            }

            )*
        }
    }
}

types! {
    self: self,
    {
        type: I8,
        self.0: i8,
        self.zero(): I8(0),
        I8::default(): I8(0),
        format_var: f,
        format_expr: (*self).to_i8().unwrap().fmt(f),
    },
    {
        type: I16,
        self.0: i16,
        self.zero(): I16(0),
        I16::default(): I16(0),
        format_var: f,
        format_expr: (*self).to_i16().unwrap().fmt(f),
    },
    {
        type: I32,
        self.0: i32,
        self.zero(): I32(0),
        I32::default(): I32(0),
        format_var: f,
        format_expr: (*self).to_i32().unwrap().fmt(f),
    },
    {
        type: I64,
        self.0: i64,
        self.zero(): I64(0),
        I64::default(): I64(0),
        format_var: f,
        format_expr: (*self).to_i64().unwrap().fmt(f),
    },
    {
        type: I128,
        self.0: i128,
        self.zero(): I128(0),
        I128::default(): I128(0),
        format_var: f,
        format_expr: (*self).to_i128().unwrap().fmt(f),
    },
    {
        type: I256,
        self.0: [u8; 32],
        self.zero(): I256([0u8; 32]),
        I256::default(): I256([0u8; 32]),
        format_var: f,
        format_expr: {
            fmt(*self, f, self.0.len() * 8)
        },
    },
    {
        type: I384,
        self.0: [u8; 48],
        self.zero(): I384([0u8; 48]),
        I384::default(): I384([0u8; 48]),
        format_var: f,
        format_expr: {
            fmt(*self, f, self.0.len() * 8)
        },
    },
    {
        type: I512,
        self.0: [u8; 64],
        self.zero(): I512([0u8; 64]),
        I512::default(): I512([0u8; 64]),
        format_var: f,
        format_expr: {
            fmt(*self, f, self.0.len() * 8)
        },
    },
    {
        type: I768,
        self.0: [u8; 96],
        self.zero(): I768([0u8; 96]),
        U256::default(): I768([0u8; 96]),
        format_var: f,
        format_expr: {
            fmt(*self, f, self.0.len() * 8)
        },
    },
    {
        type: U8,
        self.0: u8,
        self.zero(): U8(0),
        U8::default(): U8(0),
        format_var: f,
        format_expr: (*self).to_u8().unwrap().fmt(f),
    },
    {
        type: U16,
        self.0: u16,
        self.zero(): U16(0),
        U16::default(): U16(0),
        format_var: f,
        format_expr: (*self).to_u16().unwrap().fmt(f),
    },
    {
        type: U32,
        self.0: u32,
        self.zero(): U32(0),
        U32::default(): U32(0),
        format_var: f,
        format_expr: (*self).to_u32().unwrap().fmt(f),
    },
    {
        type: U64,
        self.0: u64,
        self.zero(): U64(0),
        U64::default(): U64(0),
        format_var: f,
        format_expr: (*self).to_u64().unwrap().fmt(f),
    },
    {
        type: U128,
        self.0: u128,
        self.zero(): U128(0),
        U128::default(): U128(0),
        format_var: f,
        format_expr: (*self).to_u128().unwrap().fmt(f),
    },
    {
        type: U256,
        self.0: [u8; 32],
        self.zero(): U256([0u8; 32]),
        U256::default(): U256([0u8; 32]),
        format_var: f,
        format_expr: {
            fmt(*self, f, self.0.len() * 8)
        },
    },
    {
        type: U384,
        self.0: [u8; 48],
        self.zero(): U384([0u8; 48]),
        U384::default(): U384([0u8; 48]),
        format_var: f,
        format_expr: {
            fmt(*self, f, self.0.len() * 8)
        },
    },
    {
        type: U512,
        self.0: [u8; 64],
        self.zero(): U512([0u8; 64]),
        U512::default(): U512([0u8; 64]),
        format_var: f,
        format_expr: {
            fmt(*self, f, self.0.len() * 8)
        },
    },
    {
        type: U768,
        self.0: [u8; 96],
        self.zero(): U768([0u8; 96]),
        U768::default(): U768([0u8; 96]),
        format_var: f,
        format_expr: {
            fmt(*self, f, self.0.len() * 8)
        },
    }
}

macro_rules! sbor_codec {
    ($t:ident, $t_id:expr, $t_model:ident) => {
        impl<X: CustomValueKind> Categorize<X> for $t {
            #[inline]
            fn value_kind() -> ValueKind<X> {
                $t_id
            }
        }

        impl<X: CustomValueKind, E: Encoder<X>> Encode<X, E> for $t {
            #[inline]
            fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
                encoder.write_value_kind(Self::value_kind())
            }

            #[inline]
            fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
                encoder.write_slice(&self.to_le_bytes())
            }
        }

        impl<X: CustomValueKind, D: Decoder<X>> Decode<X, D> for $t {
            fn decode_body_with_value_kind(
                decoder: &mut D,
                value_kind: ValueKind<X>,
            ) -> Result<Self, DecodeError> {
                decoder.check_preloaded_value_kind(value_kind, Self::value_kind())?;
                let slice = decoder.read_slice((Self::BITS / 8) as usize)?;
                let mut bytes = [0u8; (Self::BITS / 8) as usize];
                bytes.copy_from_slice(&slice[..]);
                Ok(Self::from_le_bytes(bytes))
            }
        }

        impl LegacyDescribe for $t {
            fn describe() -> Type {
                Type::$t_model
            }
        }
    };
}
sbor_codec!(I8, ValueKind::I8, I8);
sbor_codec!(I16, ValueKind::I16, I16);
sbor_codec!(I32, ValueKind::I32, I32);
sbor_codec!(I64, ValueKind::I64, I64);
sbor_codec!(I128, ValueKind::I128, I128);
sbor_codec!(U8, ValueKind::U8, U8);
sbor_codec!(U16, ValueKind::U16, U16);
sbor_codec!(U32, ValueKind::U32, U32);
sbor_codec!(U64, ValueKind::U64, U64);
sbor_codec!(U128, ValueKind::U128, U128);

pub trait Min {
    const MIN: Self;
}

fn fmt<
    T: fmt::Display
        + Copy
        + From<u32>
        + Pow<u32, Output = T>
        + Zero
        + One
        + ToPrimitive
        + TryInto<i128>
        + Add<Output = T>
        + Div<Output = T>
        + Rem<Output = T>
        + Sub<Output = T>
        + Eq
        + Ord
        + Min,
>(
    to_fmt: T,
    f: &mut fmt::Formatter<'_>,
    bits: usize,
) -> fmt::Result
where
    <T as TryInto<i128>>::Error: fmt::Debug,
    i128: sbor::rust::convert::TryFrom<T>,
{
    let mut minus = "";
    let mut a = to_fmt;
    let mut ls_digit = String::from("");
    if a < T::zero() {
        minus = "-";
        ls_digit = (a % T::from(10u32)).to_i128().unwrap().neg().to_string();
        a = T::zero() - a / T::from(10u32); // avoid overflow of T::MIN
    }
    let num;
    let divisor = T::from(10u32).pow(38u32);
    if a == T::from(0) {
        num = String::from("0");
    } else {
        num = (0..bits / 128 + 1).fold(String::from(""), |acc, _| {
            let num_part: i128 = (a % divisor).try_into().unwrap();
            a = a / divisor;
            if a == T::zero() {
                if num_part == 0 {
                    acc
                } else {
                    num_part.to_string() + &acc
                }
            } else {
                let padding: String = vec!["0"; 38 - num_part.to_string().len()]
                    .into_iter()
                    .collect();
                padding + &num_part.to_string() + &acc
            }
        });
    }

    if minus == "-" && num == "0" {
        write!(f, "{}{}", minus, ls_digit)
    } else {
        write!(f, "{}{}{}", minus, num, ls_digit)
    }
}

macro_rules! forward_ref_unop {
    (impl $imp:ident, $method:ident for $t:ty) => {
        impl $imp for &$t {
            type Output = <$t as $imp>::Output;

            #[inline]
            fn $method(self) -> <$t as $imp>::Output {
                $imp::$method(*self)
            }
        }
    };
}

macro_rules! forward_ref_binop {
    (impl $imp:ident, $method:ident for $t:ty, $u:ty) => {
        impl<'a> $imp<$u> for &'a $t {
            type Output = <$t as $imp<$u>>::Output;

            #[inline]
            fn $method(self, other: $u) -> <$t as $imp<$u>>::Output {
                $imp::$method(*self, other)
            }
        }

        impl $imp<&$u> for $t {
            type Output = <$t as $imp<$u>>::Output;

            #[inline]
            fn $method(self, other: &$u) -> <$t as $imp<$u>>::Output {
                $imp::$method(self, *other)
            }
        }

        impl $imp<&$u> for &$t {
            type Output = <$t as $imp<$u>>::Output;

            #[inline]
            fn $method(self, other: &$u) -> <$t as $imp<$u>>::Output {
                $imp::$method(*self, *other)
            }
        }
    };
}

macro_rules! forward_ref_op_assign {
    (impl $imp:ident, $method:ident for $t:ty, $u:ty) => {
        impl $imp<&$u> for $t {
            #[inline]
            fn $method(&mut self, other: &$u) {
                $imp::$method(self, *other);
            }
        }
    };
}

macro_rules! op_impl {
        ($($t:ty),*) => {
            paste! {
                $(
                    impl Add for $t {
                        type Output = $t;

                        #[inline]
                        fn add(self, other: $t) -> Self {
                            BigInt::from(self).add(&BigInt::from(other)).try_into().unwrap()
                        }
                    }
                    forward_ref_binop! { impl Add, add for $t, $t }

                    impl AddAssign for $t {
                        #[inline]
                        fn add_assign(&mut self, other: $t) {
                            *self = (*self + other).try_into().unwrap();
                        }
                    }
                    forward_ref_op_assign! { impl AddAssign, add_assign for $t, $t }

                    impl Sub for $t {
                        type Output = $t;

                        #[inline]
                        fn sub(self, other: $t) -> Self {
                            BigInt::from(self).sub(&BigInt::from(other)).try_into().unwrap()
                        }
                    }
                    forward_ref_binop! { impl Sub, sub for $t, $t }

                    impl SubAssign for $t {
                        #[inline]
                        fn sub_assign(&mut self, other: $t) {
                            *self = (*self - other).try_into().unwrap();
                        }
                    }
                    forward_ref_op_assign! { impl SubAssign, sub_assign for $t, $t }

                    impl Mul for $t {
                        type Output = $t;

                        #[inline]
                        fn mul(self, other: $t) -> Self {
                            BigInt::from(self).mul(&BigInt::from(other)).try_into().unwrap()
                        }
                    }
                    forward_ref_binop! { impl Mul, mul for $t, $t }

                    impl MulAssign for $t {
                        #[inline]
                        fn mul_assign(&mut self, other: $t) {
                            *self = (*self * other).try_into().unwrap();
                        }
                    }
                    forward_ref_op_assign! { impl MulAssign, mul_assign for $t, $t }

                    impl Div for $t {
                        type Output = $t;

                        #[inline]
                        fn div(self, other: $t) -> Self {
                            BigInt::from(self).div(&BigInt::from(other)).try_into().unwrap()
                        }
                    }
                    forward_ref_binop! { impl Div, div for $t, $t }

                    impl DivAssign for $t {
                        #[inline]
                        fn div_assign(&mut self, other: $t) {
                            *self = (*self / other).try_into().unwrap();
                        }
                    }
                    forward_ref_op_assign! { impl DivAssign, div_assign for $t, $t }

                    impl Rem for $t {
                        type Output = $t;

                        #[inline]
                        fn rem(self, other: $t) -> Self {
                            BigInt::from(self).rem(&BigInt::from(other)).try_into().unwrap()
                        }
                    }
                    forward_ref_binop! { impl Rem, rem for $t, $t }

                    impl RemAssign for $t {
                        #[inline]
                        fn rem_assign(&mut self, other: $t) {
                            *self = (*self % other).try_into().unwrap();
                        }
                    }
                    forward_ref_op_assign! { impl RemAssign, rem_assign for $t, $t }

                    )*
            }
        };
    }
op_impl! { I8, I16, I32, I64, I128, I256, I384, I512, I768, U8, U16, U32, U64, U128, U256, U384, U512, U768 }

pub trait CheckedAdd {
    fn checked_add(self, other: Self) -> Option<Self>
    where
        Self: Sized;
}

pub trait CheckedSub {
    fn checked_sub(self, other: Self) -> Option<Self>
    where
        Self: Sized;
}

pub trait CheckedMul {
    fn checked_mul(self, other: Self) -> Option<Self>
    where
        Self: Sized;
}

pub trait CheckedDiv {
    fn checked_div(self, other: Self) -> Option<Self>
    where
        Self: Sized;
}

pub trait CheckedRem {
    fn checked_rem(self, other: Self) -> Option<Self>
    where
        Self: Sized;
}

pub trait CheckedNeg {
    fn checked_neg(self) -> Option<Self>
    where
        Self: Sized;
}

pub trait CheckedPow {
    fn checked_pow(self, other: u32) -> Option<Self>
    where
        Self: Sized;
}

macro_rules! checked_impl {
    ($($t:ident),*) => {
        paste!{
            $(
                impl CheckedAdd for $t {
                    #[inline]
                    fn checked_add(self, other: $t) -> Option<$t> {
                        let v: Result<$t, [<Parse $t Error>]> = BigInt::from(self).add(&BigInt::from(other)).try_into();
                        v.ok()
                    }
                }

                impl CheckedSub for $t {
                    #[inline]
                    fn checked_sub(self, other: $t) -> Option<$t> {
                        let v: Result<$t, [<Parse $t Error>]> = BigInt::from(self).sub(&BigInt::from(other)).try_into();
                        v.ok()
                    }
                }

                impl CheckedMul for $t {
                    #[inline]
                    fn checked_mul(self, other: $t) -> Option<$t> {
                        let v: Result<$t, [<Parse $t Error>]> = BigInt::from(self).mul(&BigInt::from(other)).try_into();
                        v.ok()
                    }
                }

                impl CheckedDiv for $t {
                    #[inline]
                    fn checked_div(self, other: $t) -> Option<$t> {
                        let v: Result<$t, [<Parse $t Error>]> = BigInt::from(self).div(&BigInt::from(other)).try_into();
                        v.ok()
                    }
                }

                impl CheckedRem for $t {
                    #[inline]
                    fn checked_rem(self, other: $t) -> Option<$t> {
                        let v: Result<$t, [<Parse $t Error>]> = BigInt::from(self).rem(&BigInt::from(other)).try_into();
                        v.ok()
                    }
                }
                )*
        }
    }
}
checked_impl! { I8, I16, I32, I64, I128, I256, I384, I512, I768, U8, U16, U32, U64, U128, U256, U384, U512, U768 }

macro_rules! pow_impl {
        ($($t:ty),*) => {
            paste! {
                $(
                    impl Pow<u32> for $t
                    {
                        type Output = $t;

                        /// Raises self to the power of `exp`, using exponentiation by squaring.
                        ///
                        #[inline]
                        #[must_use = "this returns the result of the operation, \
                              without modifying the original"]
                        fn pow(self, exp: u32) -> Self {
                            if exp == 0 {
                                return Self::one();
                            }
                            if exp == 1 {
                                return self;
                            }
                            if exp % 2 == 0 {
                                return (self * self).pow(exp / 2);
                            } else {
                                return self * (self * self).pow((exp - 1) / 2);
                            }
                        }
                    }
                    impl CheckedPow for $t
                    {
                        fn checked_pow(self, exp: u32) -> Option<$t> {
                            if exp == 0 {
                                return Some(Self::one());
                            }
                            if exp == 1 {
                                return Some(self);
                            }
                            if exp % 2 == 0 {
                                return self.checked_mul(self).and_then(|x| x.checked_pow(exp / 2));
                            } else {
                                return self.checked_mul(self).and_then(|x| x.checked_pow((exp - 1) / 2)).and_then(|x| x.checked_mul(self));
                            }
                        }
                    }
                )*
            }
        };
}

pow_impl! { I8, I16, I32, I64, I128, I256, I384, I512, I768, U8, U16, U32, U64, U128, U256, U384, U512, U768 }

macro_rules! checked_impl_not_large {
    ($($t:ident),*) => {
        $(
            impl Not for $t {
                type Output = $t;

                #[inline]
                fn not(self) -> Self {
                    self.0.iter().map(|x| x.not()).collect::<Vec<u8>>().try_into().unwrap()
                }
            }
            forward_ref_unop! { impl Not, not for $t }
        )*
    }
}

macro_rules! checked_impl_not_small {
    ($($t:ident),*) => {
        $(
            impl Not for $t {
                type Output = $t;

                #[inline]
                fn not(self) -> Self {
                    $t(!self.0)
                }
            }
            forward_ref_unop! { impl Not, not for $t }
        )*
    }
}

checked_impl_not_large! {I256, I384, I512, I768, U256, U384, U512, U768}
checked_impl_not_small! {I8, I16, I32, I64, I128, U8, U16, U32, U64, U128}

macro_rules! checked_int_impl_signed {
    ($($t:ident, $self:ident, $base:expr),*) => ($(
            paste! {

                impl Neg for $t {
                    type Output = Self;
                    #[inline]
                    fn neg(self) -> Self {
                        Self::zero() - self
                    }
                }
                forward_ref_unop! { impl Neg, neg for $t }

                impl CheckedNeg for $t {
                    fn checked_neg(self) -> Option<Self> {
                        Self::zero().checked_sub(self)
                    }
                }

                impl $t {

                    /// Computes the absolute value of `self`, with overflow causing panic.
                    ///
                    /// The only case where such overflow can occur is when one takes the absolute value of the negative
                    /// minimal value for the type this is a positive value that is too large to represent in the type. In
                    /// such a case, this function panics.
                    ///
                    #[inline]
                    #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
                    pub fn abs($self) -> Self {
                        $base.abs().try_into().unwrap()
                    }

                    /// Returns a number representing sign of `self`.
                    ///
                    ///  - `0` if the number is zero
                    ///  - `1` if the number is positive
                    ///  - `-1` if the number is negative
                    ///
                    #[inline]
                    #[must_use = "this returns the result of the operation, \
                          without modifying the original"]
                    pub fn signum($self) -> Self {
                        $base.signum().try_into().unwrap()
                    }

                    /// Returns `true` if `self` is positive and `false` if the number is zero or
                    /// negative.
                    ///
                    #[must_use]
                    #[inline]
                    pub fn is_positive($self) -> bool {
                        $base.is_positive().try_into().unwrap()
                            // large: self.0.to_vec().into_iter().nth(self.0.len() - 1).unwrap() & 0x80 == 0
                    }

                    /// Returns `true` if `self` is negative and `false` if the number is zero or
                    /// positive.
                    ///
                    #[must_use]
                    #[inline]
                    pub fn is_negative($self) -> bool {
                        $base.is_negative().try_into().unwrap()
                            // large: self.0.to_vec().into_iter().nth(self.0.len() - 1).unwrap() & 0x80 > 0
                    }
                }
            }
    )*)
}

macro_rules! checked_int_impl_signed_all_large {
    ($($t:ident),*) => {$(
        checked_int_impl_signed! {
            $t,
            self,
            BigInt::from(self)
        }
    )*
    }
}

macro_rules! checked_int_impl_signed_all_small {
    ($($t:ident),*) => {$(
        checked_int_impl_signed! {
            $t,
            self,
            self.0
        }
    )*}
}

checked_int_impl_signed_all_large! { I256, I384, I512, I768 }
checked_int_impl_signed_all_small! { I8, I16, I32, I64, I128 }

macro_rules! checked_int_impl_unsigned_large {
    ($($t:ty),*) => ($(
            impl $t {

                /// Returns `true` if and only if `self == 2^k` for some `k`.
                ///
                #[must_use]
                #[inline]
                pub fn is_power_of_two(self) -> bool {
                    if self.0.iter().map(|x| x.count_ones()).sum::<u32>() == 1 {
                        true
                    } else {
                        false
                    }
                }

                /// Returns the smallest power of two greater than or equal to `self`.
                ///
                /// When return value overflows (i.e., `self > (1 << (N-1))` for type
                /// `uN`), it panics. It uses the checked unsigned integer arithmetics.
                ///
                #[inline]
                #[must_use = "this returns the result of the operation, \
                          without modifying the original"]
                pub fn next_power_of_two(self) -> Self {
                    let lz = self.leading_zeros();
                    let co = self.count_ones();
                    if lz == 0 && co > 1 {
                        panic!("overflow");
                    } else {
                        if co == 1 {
                            self
                        } else {
                            Self::from(1u8) << Self::from(Self::BITS - lz)
                        }
                    }
                }
            }
    )*)
}

macro_rules! checked_int_impl_unsigned_small {
    ($($t:ty),*) => ($(
            impl $t {

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
                    Self(self.0.checked_next_power_of_two().unwrap())
                }
            }
    )*)
}

checked_int_impl_unsigned_large! { U256, U384, U512, U768 }
checked_int_impl_unsigned_small! { U8, U16, U32, U64, U128 }

pub trait Sqrt {
    fn sqrt(self) -> Self;
}

pub trait Cbrt {
    fn cbrt(self) -> Self;
}

pub trait NthRoot {
    fn nth_root(self, n: u32) -> Self;
}

macro_rules! roots_op_impl
{
    ($($t:ty),*) => {
            paste! {
                $(
                    impl Sqrt for $t
                    {
                        fn sqrt(self) -> Self
                        {
                            BigInt::from(self).sqrt().try_into().unwrap()
                        }
                    }

                    impl Cbrt for $t
                    {
                        fn cbrt(self) -> Self {
                            BigInt::from(self).cbrt().try_into().unwrap()
                        }
                    }

                    impl NthRoot for $t
                    {
                        fn nth_root(self, n: u32) -> Self
                        {
                            BigInt::from(self).nth_root(n).try_into().unwrap()
                        }
                    }
                )*
            }
        };
}

roots_op_impl! {U8, U16, U32, U64, U128, U256, U384, U512, U768, I8, I16, I32, I64, I128, I256, I384, I512, I768}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::*;

    fn encode_integers(encoder: &mut ScryptoEncoder) -> Result<(), EncodeError> {
        encoder.encode(&I8::by(1i8))?;
        encoder.encode(&I16::by(1i8))?;
        encoder.encode(&I32::by(1i8))?;
        encoder.encode(&I64::by(1i8))?;
        encoder.encode(&I128::by(1i8))?;
        encoder.encode(&U8::by(1u8))?;
        encoder.encode(&U16::by(1u8))?;
        encoder.encode(&U32::by(1u8))?;
        encoder.encode(&U64::by(1u8))?;
        encoder.encode(&U128::by(1u8))?;
        Ok(())
    }

    #[test]
    fn test_integer_encoding() {
        let mut bytes = Vec::with_capacity(512);
        let mut enc = ScryptoEncoder::new(&mut bytes);
        encode_integers(&mut enc).unwrap();

        assert_eq!(
            vec![
                2, 1, // i8
                3, 1, 0, // i16
                4, 1, 0, 0, 0, // i32
                5, 1, 0, 0, 0, 0, 0, 0, 0, // i64
                6, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // i128
                7, 1, // u8
                8, 1, 0, // u16
                9, 1, 0, 0, 0, // u32
                10, 1, 0, 0, 0, 0, 0, 0, 0, // u64
                11, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // u128
            ],
            bytes
        );
    }

    #[test]
    fn test_integer_decoding() {
        let mut bytes = Vec::with_capacity(512);
        let mut enc = ScryptoEncoder::new(&mut bytes);
        encode_integers(&mut enc).unwrap();

        let mut decoder = ScryptoDecoder::new(&bytes);
        assert_eq!(I8::by(1i8), decoder.decode::<I8>().unwrap());
        assert_eq!(I16::by(1i8), decoder.decode::<I16>().unwrap());
        assert_eq!(I32::by(1i8), decoder.decode::<I32>().unwrap());
        assert_eq!(I64::by(1i8), decoder.decode::<I64>().unwrap());
        assert_eq!(I128::by(1i8), decoder.decode::<I128>().unwrap());
        assert_eq!(U8::by(1u8), decoder.decode::<U8>().unwrap());
        assert_eq!(U16::by(1u8), decoder.decode::<U16>().unwrap());
        assert_eq!(U32::by(1u8), decoder.decode::<U32>().unwrap());
        assert_eq!(U64::by(1u8), decoder.decode::<U64>().unwrap());
        assert_eq!(U128::by(1u8), decoder.decode::<U128>().unwrap());
    }
}
