extern crate alloc;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq)]
#[cfg(feature = "json")]
#[derive(Serialize, Deserialize)]
#[cfg(feature = "json")]
#[serde(tag = "type")]
pub enum Type {
    /* unit */
    Unit,

    /* boolean */
    Bool,

    /* integers */
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

    /* String, &str */
    String,

    /* Option<T> */
    Option {
        value: Box<Type>,
    },

    /* [T] */
    Array {
        base: Box<Type>,
    },

    /* (A, B, C) */
    Tuple {
        elements: Vec<Type>,
    },

    /* struct */
    Struct {
        name: String,
        fields: Fields,
    },

    /* enum */
    Enum {
        name: String,
        variants: BTreeMap<String, Fields>,
    },
}

#[derive(Debug, PartialEq)]
#[cfg(feature = "json")]
#[derive(Serialize, Deserialize)]
#[cfg(feature = "json")]
#[serde(tag = "type")]
pub enum Fields {
    Named { fields: BTreeMap<String, Type> },

    Unnamed { fields: Vec<Type> },

    Unit,
}
