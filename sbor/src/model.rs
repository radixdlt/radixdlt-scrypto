extern crate alloc;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec::Vec;

#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

use crate::sbor::{self, Decode, Encode};

// primitives
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
// rust types
pub const TYPE_OPTION: u8 = 0x10;
pub const TYPE_BOX: u8 = 0x11;
pub const TYPE_ARRAY: u8 = 0x12;
pub const TYPE_TUPLE: u8 = 0x13;
pub const TYPE_STRUCT: u8 = 0x14;
pub const TYPE_ENUM: u8 = 0x15;
pub const TYPE_FIELDS_NAMED: u8 = 0x16;
pub const TYPE_FIELDS_UNNAMED: u8 = 0x17;
pub const TYPE_FIELDS_UNIT: u8 = 0x18;
// collections
pub const TYPE_VEC: u8 = 0x20;
pub const TYPE_SET: u8 = 0x21;
pub const TYPE_MAP: u8 = 0x22;

// Internally tagged representation for readability
// See: https://serde.rs/enum-representations.html
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

    Box {
        value: Box<Type>,
    },

    Array {
        element: Box<Type>,
        length: u16,
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

    Vec {
        element: Box<Type>,
    },

    Set {
        element: Box<Type>,
    },

    Map {
        key: Box<Type>,
        value: Box<Type>,
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

    Option(Box<Option<Value>>),

    Box(Box<Value>),

    Array(Vec<Value>),

    Tuple(Vec<Value>),

    Struct(String, FieldValues),

    Enum(String, BTreeMap<String, FieldValues>),

    Vec(Vec<Value>),

    Set(BTreeSet<Value>),

    Map(BTreeMap<String, FieldValues>),
}

#[derive(Debug, PartialEq)]
pub enum FieldValues {
    Named(BTreeMap<String, Value>),

    Unnamed(Vec<Value>),

    Unit,
}
