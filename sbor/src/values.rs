use crate::sbor::{self, Decode, Encode};
#[cfg(any(feature = "scrypto_std", feature = "scrypto_alloc"))]
use scrypto_types::{Address, BID, H256, U256};

use crate::collections::*;
use crate::rust::boxed::Box;
use crate::rust::string::String;

/// Represents a SBOR data value.
#[derive(Debug, Clone, PartialEq, Eq, Decode, Encode)]
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

    Struct(String, Fields),

    Enum(String, u8, Variant),

    Vec(Vec<Value>),

    TreeSet(Vec<Value>),

    TreeMap(Vec<(Value, Value)>),

    HashSet(Vec<Value>),

    HashMap(Vec<(Value, Value)>),

    #[cfg(any(feature = "scrypto_std", feature = "scrypto_alloc"))]
    H256(H256),

    #[cfg(any(feature = "scrypto_std", feature = "scrypto_alloc"))]
    U256(U256),

    #[cfg(any(feature = "scrypto_std", feature = "scrypto_alloc"))]
    Address(Address),

    #[cfg(any(feature = "scrypto_std", feature = "scrypto_alloc"))]
    BID(BID),
}

/// Represents a enum variant.
#[derive(Debug, Clone, PartialEq, Eq, Decode, Encode)]
pub struct Variant {
    pub name: String,
    pub fields: Fields,
}

/// Represents struct fields.
#[derive(Debug, Clone, PartialEq, Eq, Decode, Encode)]
pub enum Fields {
    Named { named: Vec<(String, Value)> },

    Unnamed { unnamed: Vec<Value> },

    Unit,
}
