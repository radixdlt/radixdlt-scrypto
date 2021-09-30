use crate::rust::boxed::Box;
use crate::rust::collections::*;
use crate::rust::string::String;
use crate::rust::vec::Vec;

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
// enum and struct
pub const TYPE_STRUCT: u8 = 0x10;
pub const TYPE_ENUM: u8 = 0x11;
pub const TYPE_FIELDS_NAMED: u8 = 0x12;
pub const TYPE_FIELDS_UNNAMED: u8 = 0x13;
pub const TYPE_FIELDS_UNIT: u8 = 0x14;
// composite types
pub const TYPE_OPTION: u8 = 0x20;
pub const TYPE_BOX: u8 = 0x21;
pub const TYPE_ARRAY: u8 = 0x22;
pub const TYPE_TUPLE: u8 = 0x23;
pub const TYPE_RESULT: u8 = 0x24;
// collections
pub const TYPE_VEC: u8 = 0x30;
pub const TYPE_TREE_SET: u8 = 0x31;
pub const TYPE_TREE_MAP: u8 = 0x32;
pub const TYPE_HASH_SET: u8 = 0x33;
pub const TYPE_HASH_MAP: u8 = 0x34;
// custom types start from 0x80 and values are encoded as `len + data`
pub const TYPE_CUSTOM_START: u8 = 0x80;

/// A SBOR type ID.
pub trait TypeId {
    fn type_id() -> u8;
}

impl TypeId for () {
    #[inline]
    fn type_id() -> u8 {
        TYPE_UNIT
    }
}

impl TypeId for bool {
    #[inline]
    fn type_id() -> u8 {
        TYPE_BOOL
    }
}

impl TypeId for i8 {
    #[inline]
    fn type_id() -> u8 {
        TYPE_I8
    }
}
impl TypeId for u8 {
    #[inline]
    fn type_id() -> u8 {
        TYPE_U8
    }
}

macro_rules! type_id_int {
    ($type:ident, $type_id:ident) => {
        impl TypeId for $type {
            #[inline]
            fn type_id() -> u8 {
                $type_id
            }
        }
    };
}

type_id_int!(i16, TYPE_I16);
type_id_int!(i32, TYPE_I32);
type_id_int!(i64, TYPE_I64);
type_id_int!(i128, TYPE_I128);
type_id_int!(u16, TYPE_U16);
type_id_int!(u32, TYPE_U32);
type_id_int!(u64, TYPE_U64);
type_id_int!(u128, TYPE_U128);

impl TypeId for isize {
    #[inline]
    fn type_id() -> u8 {
        i32::type_id()
    }
}

impl TypeId for usize {
    #[inline]
    fn type_id() -> u8 {
        u32::type_id()
    }
}

impl TypeId for str {
    #[inline]
    fn type_id() -> u8 {
        TYPE_STRING
    }
}

impl TypeId for &str {
    #[inline]
    fn type_id() -> u8 {
        TYPE_STRING
    }
}

impl TypeId for String {
    #[inline]
    fn type_id() -> u8 {
        TYPE_STRING
    }
}

impl<T: TypeId> TypeId for Option<T> {
    #[inline]
    fn type_id() -> u8 {
        TYPE_OPTION
    }
}

impl<T: TypeId> TypeId for Box<T> {
    #[inline]
    fn type_id() -> u8 {
        TYPE_BOX
    }
}

impl<T: TypeId, const N: usize> TypeId for [T; N] {
    #[inline]
    fn type_id() -> u8 {
        TYPE_ARRAY
    }
}
macro_rules! type_id_tuple {
    ($n:tt $($idx:tt $name:ident)+) => {
        impl<$($name: TypeId),+> TypeId for ($($name,)+) {
            #[inline]
            fn type_id() -> u8 {
                TYPE_TUPLE
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

impl<T: TypeId, E: TypeId> TypeId for Result<T, E> {
    #[inline]
    fn type_id() -> u8 {
        TYPE_RESULT
    }
}

impl<T: TypeId> TypeId for Vec<T> {
    #[inline]
    fn type_id() -> u8 {
        TYPE_VEC
    }
}

impl<T: TypeId> TypeId for [T] {
    #[inline]
    fn type_id() -> u8 {
        TYPE_VEC
    }
}

impl<T: TypeId> TypeId for BTreeSet<T> {
    #[inline]
    fn type_id() -> u8 {
        TYPE_TREE_SET
    }
}

impl<K: TypeId, V: TypeId> TypeId for BTreeMap<K, V> {
    #[inline]
    fn type_id() -> u8 {
        TYPE_TREE_MAP
    }
}

impl<T: TypeId> TypeId for HashSet<T> {
    #[inline]
    fn type_id() -> u8 {
        TYPE_HASH_SET
    }
}

impl<K: TypeId, V: TypeId> TypeId for HashMap<K, V> {
    #[inline]
    fn type_id() -> u8 {
        TYPE_HASH_MAP
    }
}
