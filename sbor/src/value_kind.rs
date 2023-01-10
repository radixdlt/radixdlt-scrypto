use crate::constants::*;
use crate::rust::borrow::Cow;
use crate::rust::borrow::ToOwned;
use crate::rust::boxed::Box;
use crate::rust::cell::RefCell;
use crate::rust::collections::*;
use crate::rust::fmt::Debug;
use crate::rust::rc::Rc;
use crate::rust::string::String;
use crate::rust::vec::Vec;

#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type") // See https://serde.rs/enum-representations.html
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueKind<X: CustomValueKind> {
    Unit,
    Bool,
    I8,
    I16,
    I32,
    I64,
    I128,
    U8,
    U16,
    U32,
    U64,
    U128,
    String,
    Enum,
    Array,
    Tuple,
    Custom(X),
}

impl<X: CustomValueKind> ValueKind<X> {
    pub fn as_u8(&self) -> u8 {
        match self {
            ValueKind::Unit => VALUE_KIND_UNIT,
            ValueKind::Bool => VALUE_KIND_BOOL,
            ValueKind::I8 => VALUE_KIND_I8,
            ValueKind::I16 => VALUE_KIND_I16,
            ValueKind::I32 => VALUE_KIND_I32,
            ValueKind::I64 => VALUE_KIND_I64,
            ValueKind::I128 => VALUE_KIND_I128,
            ValueKind::U8 => VALUE_KIND_U8,
            ValueKind::U16 => VALUE_KIND_U16,
            ValueKind::U32 => VALUE_KIND_U32,
            ValueKind::U64 => VALUE_KIND_U64,
            ValueKind::U128 => VALUE_KIND_U128,
            ValueKind::String => VALUE_KIND_STRING,
            ValueKind::Tuple => VALUE_KIND_TUPLE,
            ValueKind::Enum => VALUE_KIND_ENUM,
            ValueKind::Array => VALUE_KIND_ARRAY,
            ValueKind::Custom(custom_value_kind) => custom_value_kind.as_u8(),
        }
    }

    pub fn from_u8(id: u8) -> Option<Self> {
        match id {
            VALUE_KIND_UNIT => Some(ValueKind::Unit),
            VALUE_KIND_BOOL => Some(ValueKind::Bool),
            VALUE_KIND_I8 => Some(ValueKind::I8),
            VALUE_KIND_I16 => Some(ValueKind::I16),
            VALUE_KIND_I32 => Some(ValueKind::I32),
            VALUE_KIND_I64 => Some(ValueKind::I64),
            VALUE_KIND_I128 => Some(ValueKind::I128),
            VALUE_KIND_U8 => Some(ValueKind::U8),
            VALUE_KIND_U16 => Some(ValueKind::U16),
            VALUE_KIND_U32 => Some(ValueKind::U32),
            VALUE_KIND_U64 => Some(ValueKind::U64),
            VALUE_KIND_U128 => Some(ValueKind::U128),
            VALUE_KIND_STRING => Some(ValueKind::String),
            VALUE_KIND_TUPLE => Some(ValueKind::Tuple),
            VALUE_KIND_ENUM => Some(ValueKind::Enum),
            VALUE_KIND_ARRAY => Some(ValueKind::Array),
            custom_value_kind_id if custom_value_kind_id >= CUSTOM_VALUE_KIND_START => {
                X::from_u8(custom_value_kind_id).map(ValueKind::Custom)
            }
            _ => None,
        }
    }
}

// primitive types
pub const VALUE_KIND_UNIT: u8 = 0x00;
pub const VALUE_KIND_BOOL: u8 = 0x01;
pub const VALUE_KIND_I8: u8 = 0x02;
pub const VALUE_KIND_I16: u8 = 0x03;
pub const VALUE_KIND_I32: u8 = 0x04;
pub const VALUE_KIND_I64: u8 = 0x05;
pub const VALUE_KIND_I128: u8 = 0x06;
pub const VALUE_KIND_U8: u8 = 0x07;
pub const VALUE_KIND_U16: u8 = 0x08;
pub const VALUE_KIND_U32: u8 = 0x09;
pub const VALUE_KIND_U64: u8 = 0x0a;
pub const VALUE_KIND_U128: u8 = 0x0b;
pub const VALUE_KIND_STRING: u8 = 0x0c;
// composite types
pub const VALUE_KIND_ARRAY: u8 = 0x20; // [T; N]
pub const VALUE_KIND_TUPLE: u8 = 0x21; // Any "product type" - Tuples and Structs (T1, T2, T3)
pub const VALUE_KIND_ENUM: u8 = 0x22;

/// The `Categorize` trait marks a rust type as having a fixed value kind for SBOR encoding/decoding.
///
/// Most rust types will have a fixed value kind in the SBOR model, and so can implement `Categorize`,
/// but some (such as the SBOR [`Value`][crate::Value]) do not.
///
/// Implementing `Categorize` is required for being able to directly [`Encode`][crate::Encode] / [`Decode`][crate::Decode] any
/// collection containing the rust type - because the value kind is lifted/deduplicated in the encoded payload.
///
/// If a type cannot implement `Categorize`, as a work-around, you can put it into a collection by (eg)
/// wrapping it in a tuple of size 1.
pub trait Categorize<X: CustomValueKind> {
    fn value_kind() -> ValueKind<X>;
}

impl<X: CustomValueKind> Categorize<X> for () {
    #[inline]
    fn value_kind() -> ValueKind<X> {
        ValueKind::Unit
    }
}

impl<X: CustomValueKind> Categorize<X> for bool {
    #[inline]
    fn value_kind() -> ValueKind<X> {
        ValueKind::Bool
    }
}

impl<X: CustomValueKind> Categorize<X> for i8 {
    #[inline]
    fn value_kind() -> ValueKind<X> {
        ValueKind::I8
    }
}
impl<X: CustomValueKind> Categorize<X> for u8 {
    #[inline]
    fn value_kind() -> ValueKind<X> {
        ValueKind::U8
    }
}

macro_rules! value_kind_int {
    ($type:ty, $value_kind:expr) => {
        impl<X: CustomValueKind> Categorize<X> for $type {
            #[inline]
            fn value_kind() -> ValueKind<X> {
                $value_kind
            }
        }
    };
}

value_kind_int!(i16, ValueKind::I16);
value_kind_int!(i32, ValueKind::I32);
value_kind_int!(i64, ValueKind::I64);
value_kind_int!(i128, ValueKind::I128);
value_kind_int!(u16, ValueKind::U16);
value_kind_int!(u32, ValueKind::U32);
value_kind_int!(u64, ValueKind::U64);
value_kind_int!(u128, ValueKind::U128);

impl<X: CustomValueKind> Categorize<X> for isize {
    #[inline]
    fn value_kind() -> ValueKind<X> {
        ValueKind::I64
    }
}

impl<X: CustomValueKind> Categorize<X> for usize {
    #[inline]
    fn value_kind() -> ValueKind<X> {
        ValueKind::U64
    }
}

impl<X: CustomValueKind> Categorize<X> for str {
    #[inline]
    fn value_kind() -> ValueKind<X> {
        ValueKind::String
    }
}

impl<X: CustomValueKind> Categorize<X> for &str {
    #[inline]
    fn value_kind() -> ValueKind<X> {
        ValueKind::String
    }
}

impl<X: CustomValueKind> Categorize<X> for String {
    #[inline]
    fn value_kind() -> ValueKind<X> {
        ValueKind::String
    }
}

impl<X: CustomValueKind, T> Categorize<X> for Option<T> {
    #[inline]
    fn value_kind() -> ValueKind<X> {
        ValueKind::Enum
    }
}

impl<'a, X: CustomValueKind, B: ?Sized + 'a + ToOwned + Categorize<X>> Categorize<X>
    for Cow<'a, B>
{
    #[inline]
    fn value_kind() -> ValueKind<X> {
        B::value_kind()
    }
}

impl<X: CustomValueKind, T: Categorize<X>> Categorize<X> for Box<T> {
    #[inline]
    fn value_kind() -> ValueKind<X> {
        T::value_kind()
    }
}

impl<X: CustomValueKind, T: Categorize<X>> Categorize<X> for Rc<T> {
    #[inline]
    fn value_kind() -> ValueKind<X> {
        T::value_kind()
    }
}

impl<X: CustomValueKind, T: Categorize<X>> Categorize<X> for RefCell<T> {
    #[inline]
    fn value_kind() -> ValueKind<X> {
        T::value_kind()
    }
}

impl<X: CustomValueKind, T, const N: usize> Categorize<X> for [T; N] {
    #[inline]
    fn value_kind() -> ValueKind<X> {
        ValueKind::Array
    }
}

macro_rules! categorize_tuple {
    ($n:tt $($idx:tt $name:ident)+) => {
        impl<X: CustomValueKind, $($name),+> Categorize<X> for ($($name,)+) {
            #[inline]
            fn value_kind() -> ValueKind<X> {
                ValueKind::Tuple
            }
        }
    };
}

categorize_tuple! { 1 0 A }
categorize_tuple! { 2 0 A 1 B }
categorize_tuple! { 3 0 A 1 B 2 C }
categorize_tuple! { 4 0 A 1 B 2 C 3 D }
categorize_tuple! { 5 0 A 1 B 2 C 3 D 4 E }
categorize_tuple! { 6 0 A 1 B 2 C 3 D 4 E 5 F }
categorize_tuple! { 7 0 A 1 B 2 C 3 D 4 E 5 F 6 G }
categorize_tuple! { 8 0 A 1 B 2 C 3 D 4 E 5 F 6 G 7 H }
categorize_tuple! { 9 0 A 1 B 2 C 3 D 4 E 5 F 6 G 7 H 8 I }
categorize_tuple! { 10 0 A 1 B 2 C 3 D 4 E 5 F 6 G 7 H 8 I 9 J }
categorize_tuple! { 11 0 A 1 B 2 C 3 D 4 E 5 F 6 G 7 H 8 I 9 J 10 K  }
categorize_tuple! { 12 0 A 1 B 2 C 3 D 4 E 5 F 6 G 7 H 8 I 9 J 10 K 11 L   }
categorize_tuple! { 13 0 A 1 B 2 C 3 D 4 E 5 F 6 G 7 H 8 I 9 J 10 K 11 L 12 M  }
categorize_tuple! { 14 0 A 1 B 2 C 3 D 4 E 5 F 6 G 7 H 8 I 9 J 10 K 11 L 12 M 13 N  }
categorize_tuple! { 15 0 A 1 B 2 C 3 D 4 E 5 F 6 G 7 H 8 I 9 J 10 K 11 L 12 M 13 N 14 O  }
categorize_tuple! { 16 0 A 1 B 2 C 3 D 4 E 5 F 6 G 7 H 8 I 9 J 10 K 11 L 12 M 13 N 14 O 15 P  }
categorize_tuple! { 17 0 A 1 B 2 C 3 D 4 E 5 F 6 G 7 H 8 I 9 J 10 K 11 L 12 M 13 N 14 O 15 P 16 Q   }
categorize_tuple! { 18 0 A 1 B 2 C 3 D 4 E 5 F 6 G 7 H 8 I 9 J 10 K 11 L 12 M 13 N 14 O 15 P 16 Q 17 R  }
categorize_tuple! { 19 0 A 1 B 2 C 3 D 4 E 5 F 6 G 7 H 8 I 9 J 10 K 11 L 12 M 13 N 14 O 15 P 16 Q 17 R 18 S  }
categorize_tuple! { 20 0 A 1 B 2 C 3 D 4 E 5 F 6 G 7 H 8 I 9 J 10 K 11 L 12 M 13 N 14 O 15 P 16 Q 17 R 18 S 19 T  }

impl<X: CustomValueKind, T, E> Categorize<X> for Result<T, E> {
    #[inline]
    fn value_kind() -> ValueKind<X> {
        ValueKind::Enum
    }
}

impl<X: CustomValueKind, T> Categorize<X> for Vec<T> {
    #[inline]
    fn value_kind() -> ValueKind<X> {
        ValueKind::Array
    }
}

impl<X: CustomValueKind, T> Categorize<X> for [T] {
    #[inline]
    fn value_kind() -> ValueKind<X> {
        ValueKind::Array
    }
}

impl<X: CustomValueKind, T> Categorize<X> for BTreeSet<T> {
    #[inline]
    fn value_kind() -> ValueKind<X> {
        ValueKind::Array
    }
}

impl<X: CustomValueKind, K, V> Categorize<X> for BTreeMap<K, V> {
    #[inline]
    fn value_kind() -> ValueKind<X> {
        ValueKind::Array
    }
}

impl<X: CustomValueKind, T> Categorize<X> for HashSet<T> {
    #[inline]
    fn value_kind() -> ValueKind<X> {
        ValueKind::Array
    }
}

impl<X: CustomValueKind, K, V> Categorize<X> for HashMap<K, V> {
    #[inline]
    fn value_kind() -> ValueKind<X> {
        ValueKind::Array
    }
}

#[cfg(feature = "indexmap")]
impl<X: CustomValueKind, T> Categorize<X> for IndexSet<T> {
    #[inline]
    fn value_kind() -> ValueKind<X> {
        ValueKind::Array
    }
}

#[cfg(feature = "indexmap")]
impl<X: CustomValueKind, K, V> Categorize<X> for IndexMap<K, V> {
    #[inline]
    fn value_kind() -> ValueKind<X> {
        ValueKind::Array
    }
}

pub trait CustomValueKind: Copy + Debug + Clone + PartialEq + Eq {
    fn as_u8(&self) -> u8;

    fn from_u8(id: u8) -> Option<Self>;
}
