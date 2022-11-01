use crate::constants::*;
use crate::rust::borrow::Cow;
use crate::rust::borrow::ToOwned;
use crate::rust::boxed::Box;
use crate::rust::cell::RefCell;
use crate::rust::collections::*;
use crate::rust::rc::Rc;
use crate::rust::string::String;
use crate::rust::vec::Vec;
use crate::*;

// primitive types
const TYPE_UNIT: u8 = 0x00;
const TYPE_BOOL: u8 = 0x01;
const TYPE_I8: u8 = 0x02;
const TYPE_I16: u8 = 0x03;
const TYPE_I32: u8 = 0x04;
const TYPE_I64: u8 = 0x05;
const TYPE_I128: u8 = 0x06;
const TYPE_U8: u8 = 0x07;
const TYPE_U16: u8 = 0x08;
const TYPE_U32: u8 = 0x09;
const TYPE_U64: u8 = 0x0a;
const TYPE_U128: u8 = 0x0b;
const TYPE_STRING: u8 = 0x0c;
// struct & enum
const TYPE_STRUCT: u8 = 0x10;
const TYPE_ENUM: u8 = 0x11;
// composite types
const TYPE_ARRAY: u8 = 0x20; // [T; N]
const TYPE_TUPLE: u8 = 0x21; // (T1, T2, T3)

/// A SBOR type ID.
pub trait TypeId {
    fn type_id() -> SborTypeId;
}

impl TypeId for () {
    #[inline]
    fn type_id() -> SborTypeId {
        SborTypeId::Unit
    }
}

impl TypeId for bool {
    #[inline]
    fn type_id() -> SborTypeId {
        SborTypeId::Bool
    }
}

impl TypeId for i8 {
    #[inline]
    fn type_id() -> SborTypeId {
        SborTypeId::I8
    }
}
impl TypeId for u8 {
    #[inline]
    fn type_id() -> SborTypeId {
        SborTypeId::U8
    }
}

macro_rules! type_id_int {
    ($type:ty, $type_id:expr) => {
        impl TypeId for $type {
            #[inline]
            fn type_id() -> SborTypeId {
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

impl TypeId for isize {
    #[inline]
    fn type_id() -> SborTypeId {
        SborTypeId::I64
    }
}

impl TypeId for usize {
    #[inline]
    fn type_id() -> SborTypeId {
        SborTypeId::U64
    }
}

impl TypeId for str {
    #[inline]
    fn type_id() -> SborTypeId {
        SborTypeId::String
    }
}

impl TypeId for &str {
    #[inline]
    fn type_id() -> SborTypeId {
        SborTypeId::String
    }
}

impl TypeId for String {
    #[inline]
    fn type_id() -> SborTypeId {
        SborTypeId::String
    }
}

impl<T> TypeId for Option<T> {
    #[inline]
    fn type_id() -> SborTypeId {
        SborTypeId::Enum
    }
}

impl<'a, B: ?Sized + 'a + ToOwned + TypeId> TypeId for Cow<'a, B> {
    #[inline]
    fn type_id() -> SborTypeId {
        B::type_id()
    }
}

impl<T: TypeId> TypeId for Box<T> {
    #[inline]
    fn type_id() -> SborTypeId {
        T::type_id()
    }
}

impl<T: TypeId> TypeId for Rc<T> {
    #[inline]
    fn type_id() -> SborTypeId {
        T::type_id()
    }
}

impl<T: TypeId> TypeId for RefCell<T> {
    #[inline]
    fn type_id() -> SborTypeId {
        T::type_id()
    }
}

impl<T, const N: usize> TypeId for [T; N] {
    #[inline]
    fn type_id() -> SborTypeId {
        SborTypeId::Array
    }
}

macro_rules! type_id_tuple {
    ($n:tt $($idx:tt $name:ident)+) => {
        impl<$($name),+> TypeId for ($($name,)+) {
            #[inline]
            fn type_id() -> SborTypeId {
                SborTypeId::Tuple
            }
        }
    };
}

type_id_tuple! { 2 0 A 1 B }
type_id_tuple! { 3 0 A 1 B 2 C }
type_id_tuple! { 4 0 A 1 B 2 C 3 D }
type_id_tuple! { 5 0 A 1 B 2 C 3 D 4 E }
type_id_tuple! { 6 0 A 1 B 2 C 3 D 4 E 5 F }
type_id_tuple! { 7 0 A 1 B 2 C 3 D 4 E 5 F 6 G }
type_id_tuple! { 8 0 A 1 B 2 C 3 D 4 E 5 F 6 G 7 H }
type_id_tuple! { 9 0 A 1 B 2 C 3 D 4 E 5 F 6 G 7 H 8 I }
type_id_tuple! { 10 0 A 1 B 2 C 3 D 4 E 5 F 6 G 7 H 8 I 9 J }

impl<T, E> TypeId for Result<T, E> {
    #[inline]
    fn type_id() -> SborTypeId {
        SborTypeId::Enum
    }
}

impl<T> TypeId for Vec<T> {
    #[inline]
    fn type_id() -> SborTypeId {
        SborTypeId::Array
    }
}

impl<T> TypeId for [T] {
    #[inline]
    fn type_id() -> SborTypeId {
        SborTypeId::Array
    }
}

impl<T> TypeId for BTreeSet<T> {
    #[inline]
    fn type_id() -> SborTypeId {
        SborTypeId::Array
    }
}

impl<K, V> TypeId for BTreeMap<K, V> {
    #[inline]
    fn type_id() -> SborTypeId {
        SborTypeId::Array
    }
}

impl<T> TypeId for HashSet<T> {
    #[inline]
    fn type_id() -> SborTypeId {
        SborTypeId::Array
    }
}

impl<K, V> TypeId for HashMap<K, V> {
    #[inline]
    fn type_id() -> SborTypeId {
        SborTypeId::Array
    }
}
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type") // For JSON readability, see https://serde.rs/enum-representations.html
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, TypeId, Encode, Decode)]
pub enum SborTypeId {
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
    Struct,
    Enum,
    Array,
    Tuple,
    Custom(u8),
}

impl SborTypeId {
    pub fn id(&self) -> u8 {
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
            SborTypeId::Struct => TYPE_STRUCT,
            SborTypeId::Enum => TYPE_ENUM,
            SborTypeId::Array => TYPE_ARRAY,
            SborTypeId::Tuple => TYPE_TUPLE,
            SborTypeId::Custom(type_id) => *type_id,
        }
    }

    pub fn from_id(id: u8) -> Option<Self> {
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
            TYPE_STRUCT => Some(SborTypeId::Struct),
            TYPE_ENUM => Some(SborTypeId::Enum),
            TYPE_ARRAY => Some(SborTypeId::Array),
            TYPE_TUPLE => Some(SborTypeId::Tuple),
            id if id >= CUSTOM_TYPE_START => Some(SborTypeId::Custom(id)),
            _ => None,
        }
    }
}
