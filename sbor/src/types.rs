extern crate alloc;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

use crate::sbor::{self, Decode, Encode};
#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

// Internally tagged representation for readability
// See: https://serde.rs/enum-representations.html

/// Represents a SBOR data type.
#[cfg_attr(feature = "json", derive(Serialize, Deserialize), serde(tag = "type"))]
#[derive(Debug, PartialEq, Eq, Decode, Encode)]
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
        fields: Fields,
    },

    Enum {
        name: String,
        variants: Vec<Variant>, // Order matters as it decides of the variant index
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

/// Represents the type info of an enum variant.
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq, Eq, Decode, Encode)]
pub struct Variant {
    pub name: String,
    pub fields: Fields,
}

/// Represents the type info of struct fields.
#[cfg_attr(feature = "json", derive(Serialize, Deserialize), serde(tag = "type"))]
#[derive(Debug, PartialEq, Eq, Decode, Encode)]
pub enum Fields {
    Named { named: Vec<(String, Type)> },

    Unnamed { unnamed: Vec<Type> },

    Unit,
}
