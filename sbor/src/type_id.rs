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
pub enum SborTypeId<X: CustomTypeId> {
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

impl<X: CustomTypeId> SborTypeId<X> {
    pub fn as_u8(&self) -> u8 {
        match self {
            SborTypeId::Unit => TYPE_UNIT,
            SborTypeId::Bool => TYPE_BOOL,
            SborTypeId::I8 => TYPE_I8,
            SborTypeId::I16 => TYPE_I16,
            SborTypeId::I32 => TYPE_I32,
            SborTypeId::I64 => TYPE_I64,
            SborTypeId::I128 => TYPE_I128,
            SborTypeId::U8 => TYPE_U8,
            SborTypeId::U16 => TYPE_U16,
            SborTypeId::U32 => TYPE_U32,
            SborTypeId::U64 => TYPE_U64,
            SborTypeId::U128 => TYPE_U128,
            SborTypeId::String => TYPE_STRING,
            SborTypeId::Tuple => TYPE_TUPLE,
            SborTypeId::Enum => TYPE_ENUM,
            SborTypeId::Array => TYPE_ARRAY,
            SborTypeId::Custom(type_id) => type_id.as_u8(),
        }
    }

    pub fn from_u8(id: u8) -> Option<Self> {
        match id {
            TYPE_UNIT => Some(SborTypeId::Unit),
            TYPE_BOOL => Some(SborTypeId::Bool),
            TYPE_I8 => Some(SborTypeId::I8),
            TYPE_I16 => Some(SborTypeId::I16),
            TYPE_I32 => Some(SborTypeId::I32),
            TYPE_I64 => Some(SborTypeId::I64),
            TYPE_I128 => Some(SborTypeId::I128),
            TYPE_U8 => Some(SborTypeId::U8),
            TYPE_U16 => Some(SborTypeId::U16),
            TYPE_U32 => Some(SborTypeId::U32),
            TYPE_U64 => Some(SborTypeId::U64),
            TYPE_U128 => Some(SborTypeId::U128),
            TYPE_STRING => Some(SborTypeId::String),
            TYPE_TUPLE => Some(SborTypeId::Tuple),
            TYPE_ENUM => Some(SborTypeId::Enum),
            TYPE_ARRAY => Some(SborTypeId::Array),
            type_id if type_id >= CUSTOM_TYPE_START => X::from_u8(type_id).map(SborTypeId::Custom),
            _ => None,
        }
    }
}

// primitive types
pub const TYPE_UNIT: u8 = 0x00;
pub const TYPE_BOOL: u8 = 0x01;
pub const TYPE_I8: u8 = 0x02;
pub const TYPE_I16: u8 = 0x03;
pub const TYPE_I32: u8 = 0x04;
pub const TYPE_I64: u8 = 0x05;
pub const TYPE_I128: u8 = 0x06;
pub const TYPE_U8: u8 = 0x07;
pub const TYPE_U16: u8 = 0x08;
pub const TYPE_U32: u8 = 0x09;
pub const TYPE_U64: u8 = 0x0a;
pub const TYPE_U128: u8 = 0x0b;
pub const TYPE_STRING: u8 = 0x0c;
// composite types
pub const TYPE_TUPLE: u8 = 0x21; // Any "product type" - Tuples and Structs (T1, T2, T3)
pub const TYPE_ENUM: u8 = 0x11;
pub const TYPE_ARRAY: u8 = 0x20; // [T; N]

/// A SBOR type ID.
pub trait TypeId<X: CustomTypeId> {
    fn type_id() -> SborTypeId<X>;
}

impl<X: CustomTypeId> TypeId<X> for () {
    #[inline]
    fn type_id() -> SborTypeId<X> {
        SborTypeId::Unit
    }
}

impl<X: CustomTypeId> TypeId<X> for bool {
    #[inline]
    fn type_id() -> SborTypeId<X> {
        SborTypeId::Bool
    }
}

impl<X: CustomTypeId> TypeId<X> for i8 {
    #[inline]
    fn type_id() -> SborTypeId<X> {
        SborTypeId::I8
    }
}
impl<X: CustomTypeId> TypeId<X> for u8 {
    #[inline]
    fn type_id() -> SborTypeId<X> {
        SborTypeId::U8
    }
}

macro_rules! type_id_int {
    ($type:ty, $type_id:expr) => {
        impl<X: CustomTypeId> TypeId<X> for $type {
            #[inline]
            fn type_id() -> SborTypeId<X> {
                $type_id
            }
        }
    };
}

type_id_int!(i16, SborTypeId::I16);
type_id_int!(i32, SborTypeId::I32);
type_id_int!(i64, SborTypeId::I64);
type_id_int!(i128, SborTypeId::I128);
type_id_int!(u16, SborTypeId::U16);
type_id_int!(u32, SborTypeId::U32);
type_id_int!(u64, SborTypeId::U64);
type_id_int!(u128, SborTypeId::U128);

impl<X: CustomTypeId> TypeId<X> for isize {
    #[inline]
    fn type_id() -> SborTypeId<X> {
        SborTypeId::I64
    }
}

impl<X: CustomTypeId> TypeId<X> for usize {
    #[inline]
    fn type_id() -> SborTypeId<X> {
        SborTypeId::U64
    }
}

impl<X: CustomTypeId> TypeId<X> for str {
    #[inline]
    fn type_id() -> SborTypeId<X> {
        SborTypeId::String
    }
}

impl<X: CustomTypeId> TypeId<X> for &str {
    #[inline]
    fn type_id() -> SborTypeId<X> {
        SborTypeId::String
    }
}

impl<X: CustomTypeId> TypeId<X> for String {
    #[inline]
    fn type_id() -> SborTypeId<X> {
        SborTypeId::String
    }
}

impl<X: CustomTypeId, T> TypeId<X> for Option<T> {
    #[inline]
    fn type_id() -> SborTypeId<X> {
        SborTypeId::Enum
    }
}

impl<'a, X: CustomTypeId, B: ?Sized + 'a + ToOwned + TypeId<X>> TypeId<X> for Cow<'a, B> {
    #[inline]
    fn type_id() -> SborTypeId<X> {
        B::type_id()
    }
}

impl<X: CustomTypeId, T: TypeId<X>> TypeId<X> for Box<T> {
    #[inline]
    fn type_id() -> SborTypeId<X> {
        T::type_id()
    }
}

impl<X: CustomTypeId, T: TypeId<X>> TypeId<X> for Rc<T> {
    #[inline]
    fn type_id() -> SborTypeId<X> {
        T::type_id()
    }
}

impl<X: CustomTypeId, T: TypeId<X>> TypeId<X> for RefCell<T> {
    #[inline]
    fn type_id() -> SborTypeId<X> {
        T::type_id()
    }
}

impl<X: CustomTypeId, T, const N: usize> TypeId<X> for [T; N] {
    #[inline]
    fn type_id() -> SborTypeId<X> {
        SborTypeId::Array
    }
}

macro_rules! type_id_tuple {
    ($n:tt $($idx:tt $name:ident)+) => {
        impl<X: CustomTypeId, $($name),+> TypeId<X> for ($($name,)+) {
            #[inline]
            fn type_id() -> SborTypeId<X> {
                SborTypeId::Tuple
            }
        }
    };
}

type_id_tuple! { 1 0 A }
type_id_tuple! { 2 0 A 1 B }
type_id_tuple! { 3 0 A 1 B 2 C }
type_id_tuple! { 4 0 A 1 B 2 C 3 D }
type_id_tuple! { 5 0 A 1 B 2 C 3 D 4 E }
type_id_tuple! { 6 0 A 1 B 2 C 3 D 4 E 5 F }
type_id_tuple! { 7 0 A 1 B 2 C 3 D 4 E 5 F 6 G }
type_id_tuple! { 8 0 A 1 B 2 C 3 D 4 E 5 F 6 G 7 H }
type_id_tuple! { 9 0 A 1 B 2 C 3 D 4 E 5 F 6 G 7 H 8 I }
type_id_tuple! { 10 0 A 1 B 2 C 3 D 4 E 5 F 6 G 7 H 8 I 9 J }

impl<X: CustomTypeId, T, E> TypeId<X> for Result<T, E> {
    #[inline]
    fn type_id() -> SborTypeId<X> {
        SborTypeId::Enum
    }
}

impl<X: CustomTypeId, T> TypeId<X> for Vec<T> {
    #[inline]
    fn type_id() -> SborTypeId<X> {
        SborTypeId::Array
    }
}

impl<X: CustomTypeId, T> TypeId<X> for [T] {
    #[inline]
    fn type_id() -> SborTypeId<X> {
        SborTypeId::Array
    }
}

impl<X: CustomTypeId, T> TypeId<X> for BTreeSet<T> {
    #[inline]
    fn type_id() -> SborTypeId<X> {
        SborTypeId::Array
    }
}

impl<X: CustomTypeId, K, V> TypeId<X> for BTreeMap<K, V> {
    #[inline]
    fn type_id() -> SborTypeId<X> {
        SborTypeId::Array
    }
}

impl<X: CustomTypeId, T> TypeId<X> for HashSet<T> {
    #[inline]
    fn type_id() -> SborTypeId<X> {
        SborTypeId::Array
    }
}

impl<X: CustomTypeId, K, V> TypeId<X> for HashMap<K, V> {
    #[inline]
    fn type_id() -> SborTypeId<X> {
        SborTypeId::Array
    }
}

#[cfg(feature = "indexmap")]
impl<X: CustomTypeId, K, V> TypeId<X> for indexmap::IndexMap<K, V> {
    #[inline]
    fn type_id() -> SborTypeId<X> {
        SborTypeId::Array
    }
}

pub trait CustomTypeId: Copy + Debug + Clone + PartialEq + Eq {
    fn as_u8(&self) -> u8;

    fn from_u8(id: u8) -> Option<Self>;
}
