extern crate alloc;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

use crate::sbor::{self, Decode, Encode};

pub const TYPE_UNIT: u8 = 0;
pub const TYPE_BOOL: u8 = 1;
pub const TYPE_I8: u8 = 2;
pub const TYPE_I16: u8 = 3;
pub const TYPE_I32: u8 = 4;
pub const TYPE_I64: u8 = 5;
pub const TYPE_I128: u8 = 6;
pub const TYPE_U8: u8 = 7;
pub const TYPE_U16: u8 = 8;
pub const TYPE_U32: u8 = 9;
pub const TYPE_U64: u8 = 10;
pub const TYPE_U128: u8 = 11;
pub const TYPE_STRING: u8 = 12;
pub const TYPE_OPTION: u8 = 13;
pub const TYPE_ARRAY: u8 = 14;
pub const TYPE_VEC: u8 = 15;
pub const TYPE_TUPLE: u8 = 16;
pub const TYPE_STRUCT: u8 = 17;
pub const TYPE_ENUM: u8 = 18;
pub const TYPE_FIELDS_NAMED: u8 = 19;
pub const TYPE_FIELDS_UNNAMED: u8 = 20;
pub const TYPE_FIELDS_UNIT: u8 = 21;
pub const TYPE_B_TREE_MAP: u8 = 22;
pub const TYPE_BOX: u8 = 23;

#[cfg_attr(feature = "json", derive(Serialize, Deserialize), serde(tag = "type"))]
#[derive(Debug, PartialEq, Decode, Encode)]
pub enum Type {
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

    Option {
        value: Box<Type>,
    },

    Array {
        base: Box<Type>,
        length: u16,
    },

    Vec {
        base: Box<Type>,
    },

    Tuple {
        elements: Vec<Type>,
    },

    Struct {
        name: String,
        fields: FieldTypes,
    },

    Enum {
        name: String,
        variants: BTreeMap<String, FieldTypes>,
    },
}

#[cfg_attr(feature = "json", derive(Serialize, Deserialize), serde(tag = "type"))]
#[derive(Debug, PartialEq, Decode, Encode)]
pub enum FieldTypes {
    Named { fields: BTreeMap<String, Type> },

    Unnamed { fields: Vec<Type> },

    Unit,
}

#[derive(Debug, PartialEq)]
pub enum Value {
    Unit,

    Bool(bool),

    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),

    String(String),

    Option(Option<Box<Value>>),

    Array(Vec<Value>),

    Vec(Vec<Value>),

    Tuple(Vec<Value>),

    Struct {
        name: String,
        fields: FieldValues,
    },

    Enum {
        name: String,
        variants: BTreeMap<String, FieldValues>,
    },
}

#[derive(Debug, PartialEq)]
pub enum FieldValues {
    Named { fields: BTreeMap<String, Value> },

    Unnamed { fields: Vec<Value> },

    Unit,
}
