#[cfg(any(feature = "serde_std", feature = "serde_alloc"))]
use serde::{Deserialize, Serialize};

use crate::sbor::{Decode, Encode};

use crate::rust::boxed::Box;
use crate::rust::collections::*;
use crate::rust::string::String;
use crate::rust::vec;
use crate::rust::vec::Vec;

// For enum, we use internally tagged representation for readability.
// See: https://serde.rs/enum-representations.html

/// Represents a SBOR data type.
#[cfg_attr(
    any(feature = "serde_std", feature = "serde_alloc"),
    derive(Serialize, Deserialize),
    serde(tag = "type")
)]
#[derive(Debug, Clone, PartialEq, Eq, Decode, Encode)]
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

    TreeSet {
        element: Box<Type>,
    },

    TreeMap {
        key: Box<Type>,
        value: Box<Type>,
    },

    HashSet {
        element: Box<Type>,
    },

    HashMap {
        key: Box<Type>,
        value: Box<Type>,
    },

    Custom {
        name: String,
    },
}

/// Represents the type info of an enum variant.
#[cfg_attr(
    any(feature = "serde_std", feature = "serde_alloc"),
    derive(Serialize, Deserialize)
)]
#[derive(Debug, Clone, PartialEq, Eq, Decode, Encode)]
pub struct Variant {
    pub name: String,
    pub fields: Fields,
}

/// Represents the type info of struct fields.
#[cfg_attr(
    any(feature = "serde_std", feature = "serde_alloc"),
    derive(Serialize, Deserialize),
    serde(tag = "type")
)]
#[derive(Debug, Clone, PartialEq, Eq, Decode, Encode)]
pub enum Fields {
    Named { named: Vec<(String, Type)> },

    Unnamed { unnamed: Vec<Type> },

    Unit,
}

/// A data structure that can be described using SBOR types.
pub trait Describe {
    fn describe() -> Type;
}

impl Describe for () {
    fn describe() -> Type {
        Type::Unit
    }
}

macro_rules! describe_basic_type {
    ($type:ident, $type_id:expr) => {
        impl Describe for $type {
            fn describe() -> Type {
                $type_id
            }
        }
    };
}

describe_basic_type!(bool, Type::Bool);
describe_basic_type!(i8, Type::I8);
describe_basic_type!(i16, Type::I16);
describe_basic_type!(i32, Type::I32);
describe_basic_type!(i64, Type::I64);
describe_basic_type!(i128, Type::I128);
describe_basic_type!(u8, Type::U8);
describe_basic_type!(u16, Type::U16);
describe_basic_type!(u32, Type::U32);
describe_basic_type!(u64, Type::U64);
describe_basic_type!(u128, Type::U128);

describe_basic_type!(isize, Type::I32);
describe_basic_type!(usize, Type::U32);

describe_basic_type!(str, Type::String);
describe_basic_type!(String, Type::String);

impl<T: Describe> Describe for Option<T> {
    fn describe() -> Type {
        let ty = T::describe();
        Type::Option {
            value: Box::new(ty),
        }
    }
}

impl<T: Describe> Describe for Box<T> {
    fn describe() -> Type {
        let ty = T::describe();
        Type::Box {
            value: Box::new(ty),
        }
    }
}

impl<T: Describe, const N: usize> Describe for [T; N] {
    fn describe() -> Type {
        let ty = T::describe();
        Type::Array {
            element: Box::new(ty),
            length: N as u16,
        }
    }
}

macro_rules! describe_tuple {
    ($($name:ident)+) => {
        impl<$($name: Describe),+> Describe for ($($name,)+) {
            fn describe() -> Type {
                Type::Tuple { elements: vec![ $($name::describe(),)* ] }
            }
        }
    };
}

describe_tuple! { A B }
describe_tuple! { A B C }
describe_tuple! { A B C D }
describe_tuple! { A B C D E }
describe_tuple! { A B C D E F }
describe_tuple! { A B C D E F G }
describe_tuple! { A B C D E F G H }
describe_tuple! { A B C D E F G H I }
describe_tuple! { A B C D E F G H I J }

impl<T: Describe> Describe for Vec<T> {
    fn describe() -> Type {
        let ty = T::describe();
        Type::Vec {
            element: Box::new(ty),
        }
    }
}

impl<T: Describe> Describe for BTreeSet<T> {
    fn describe() -> Type {
        let ty = T::describe();
        Type::TreeSet {
            element: Box::new(ty),
        }
    }
}

impl<K: Describe, V: Describe> Describe for BTreeMap<K, V> {
    fn describe() -> Type {
        let k = K::describe();
        let v = V::describe();
        Type::TreeMap {
            key: Box::new(k),
            value: Box::new(v),
        }
    }
}

impl<T: Describe> Describe for HashSet<T> {
    fn describe() -> Type {
        let ty = T::describe();
        Type::HashSet {
            element: Box::new(ty),
        }
    }
}

impl<K: Describe, V: Describe> Describe for HashMap<K, V> {
    fn describe() -> Type {
        let k = K::describe();
        let v = V::describe();
        Type::HashMap {
            key: Box::new(k),
            value: Box::new(v),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::describe::*;
    use crate::rust::boxed::Box;
    use crate::rust::string::String;
    use crate::rust::vec;

    #[test]
    pub fn test_basic_types() {
        assert_eq!(Type::Bool, bool::describe());
        assert_eq!(Type::I8, i8::describe());
        assert_eq!(Type::I16, i16::describe());
        assert_eq!(Type::I32, i32::describe());
        assert_eq!(Type::I64, i64::describe());
        assert_eq!(Type::I128, i128::describe());
        assert_eq!(Type::U8, u8::describe());
        assert_eq!(Type::U16, u16::describe());
        assert_eq!(Type::U32, u32::describe());
        assert_eq!(Type::U64, u64::describe());
        assert_eq!(Type::U128, u128::describe());
        assert_eq!(Type::String, String::describe());
    }

    #[test]
    pub fn test_option() {
        assert_eq!(
            Type::Option {
                value: Box::new(Type::String)
            },
            Option::<String>::describe(),
        );
    }

    #[test]
    pub fn test_array() {
        assert_eq!(
            Type::Array {
                element: Box::new(Type::U8),
                length: 3,
            },
            <[u8; 3]>::describe(),
        );
    }

    #[test]
    pub fn test_tuple() {
        assert_eq!(
            Type::Tuple {
                elements: vec![Type::U8, Type::U128]
            },
            <(u8, u128)>::describe(),
        );
    }
}
