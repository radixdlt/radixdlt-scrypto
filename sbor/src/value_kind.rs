use crate::rust::fmt::Debug;
use crate::*;

#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type") // See https://serde.rs/enum-representations.html
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueKind<X: CustomValueKind> {
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
    Map,
    Custom(X),
}

impl<X: CustomValueKind> crate::rust::fmt::Display for ValueKind<X> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Custom(x) => write!(f, "{:?}", x),
            _ => write!(f, "{:?}", self),
        }
    }
}

impl<X: CustomValueKind> ValueKind<X> {
    pub fn as_u8(&self) -> u8 {
        match self {
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
            ValueKind::Map => VALUE_KIND_MAP,
            ValueKind::Custom(custom_value_kind) => custom_value_kind.as_u8(),
        }
    }

    pub fn from_u8(id: u8) -> Option<Self> {
        match id {
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
            VALUE_KIND_MAP => Some(ValueKind::Map),
            custom_value_kind_id if custom_value_kind_id >= CUSTOM_VALUE_KIND_START => {
                X::from_u8(custom_value_kind_id).map(ValueKind::Custom)
            }
            _ => None,
        }
    }
}

pub trait CustomValueKind: Copy + Debug + Clone + PartialEq + Eq {
    fn as_u8(&self) -> u8;

    fn from_u8(id: u8) -> Option<Self>;
}

// primitive types
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
pub const VALUE_KIND_ARRAY: u8 = 0x20; // [T] or [T; N]
pub const VALUE_KIND_TUPLE: u8 = 0x21; // Any "product type" - Units, Tuples and Structs (T1, T2, T3)
pub const VALUE_KIND_ENUM: u8 = 0x22;
pub const VALUE_KIND_MAP: u8 = 0x23;
