use sbor::rust::boxed::Box;
use sbor::rust::collections::*;
use sbor::rust::string::String;
use sbor::rust::vec;
use sbor::rust::vec::Vec;
use sbor::*;

/// Represents a SBOR type.
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type")  // See https://serde.rs/enum-representations.html
)]
#[derive(Debug, Clone, PartialEq, Eq, Categorize, Decode, Encode)]
pub enum Type {
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

    // Array
    Array {
        element_type: Box<Type>,
        length: u16,
    },
    Vec {
        element_type: Box<Type>,
    },
    HashSet {
        element_type: Box<Type>,
    },
    TreeSet {
        element_type: Box<Type>,
    },
    HashMap {
        key_type: Box<Type>,
        value_type: Box<Type>,
    },
    TreeMap {
        key_type: Box<Type>,
        value_type: Box<Type>,
    },

    // Tuple
    Tuple {
        element_types: Vec<Type>,
    },
    Struct {
        name: String,
        fields: Fields,
    },
    NonFungibleGlobalId,

    // Enum
    Enum {
        name: String,
        variants: Vec<Variant>,
    },
    Option {
        some_type: Box<Type>,
    },
    Result {
        okay_type: Box<Type>,
        err_type: Box<Type>,
    },

    // RE interpreted
    PackageAddress,
    ComponentAddress,
    ResourceAddress,
    Own, /* generic, either bucket, proof, vault, component or kv store. TODO: do we really need this? */
    Bucket,
    Proof,
    Vault,
    Component,
    KeyValueStore {
        key_type: Box<Type>,
        value_type: Box<Type>,
    },

    // Uninterpreted
    Hash,
    EcdsaSecp256k1PublicKey,
    EcdsaSecp256k1Signature,
    EddsaEd25519PublicKey,
    EddsaEd25519Signature,
    Decimal,
    PreciseDecimal,
    NonFungibleLocalId,

    Any,
}

/// Represents the type info of an enum variant.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Categorize, Decode, Encode)]
pub struct Variant {
    pub name: String,
    pub fields: Fields,
}

/// Represents the type info of struct fields.
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type")
)]
#[derive(Debug, Clone, PartialEq, Eq, Categorize, Decode, Encode)]
pub enum Fields {
    Named { named: Vec<(String, Type)> },

    Unnamed { unnamed: Vec<Type> },

    Unit,
}

/// A data structure that can be described using SBOR types.
pub trait LegacyDescribe {
    fn describe() -> Type;
}

macro_rules! describe_basic_type {
    ($type_name:ident, $type:expr) => {
        impl LegacyDescribe for $type_name {
            fn describe() -> Type {
                $type
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

describe_basic_type!(isize, Type::I64);
describe_basic_type!(usize, Type::U64);

describe_basic_type!(str, Type::String);
describe_basic_type!(String, Type::String);

impl<T: LegacyDescribe> LegacyDescribe for Option<T> {
    fn describe() -> Type {
        let ty = T::describe();
        Type::Option {
            some_type: Box::new(ty),
        }
    }
}

impl<T: LegacyDescribe, const N: usize> LegacyDescribe for [T; N] {
    fn describe() -> Type {
        let ty = T::describe();
        Type::Array {
            element_type: Box::new(ty),
            length: N as u16,
        }
    }
}

macro_rules! describe_tuple {
    ($($name:ident)*) => {
        impl<$($name: LegacyDescribe),*> LegacyDescribe for ($($name,)*) {
            fn describe() -> Type {
                Type::Tuple { element_types: vec![ $($name::describe(),)* ] }
            }
        }
    };
}

describe_tuple! {} // Unit
describe_tuple! { A }
describe_tuple! { A B }
describe_tuple! { A B C }
describe_tuple! { A B C D }
describe_tuple! { A B C D E }
describe_tuple! { A B C D E F }
describe_tuple! { A B C D E F G }
describe_tuple! { A B C D E F G H }
describe_tuple! { A B C D E F G H I }
describe_tuple! { A B C D E F G H I J }

impl<T: LegacyDescribe, E: LegacyDescribe> LegacyDescribe for Result<T, E> {
    fn describe() -> Type {
        let t = T::describe();
        let e = E::describe();
        Type::Result {
            okay_type: Box::new(t),
            err_type: Box::new(e),
        }
    }
}

impl<T: LegacyDescribe> LegacyDescribe for Vec<T> {
    fn describe() -> Type {
        let ty = T::describe();
        Type::Vec {
            element_type: Box::new(ty),
        }
    }
}

impl<T: LegacyDescribe> LegacyDescribe for BTreeSet<T> {
    fn describe() -> Type {
        let ty = T::describe();
        Type::TreeSet {
            element_type: Box::new(ty),
        }
    }
}

impl<K: LegacyDescribe, V: LegacyDescribe> LegacyDescribe for BTreeMap<K, V> {
    fn describe() -> Type {
        let k = K::describe();
        let v = V::describe();
        Type::TreeMap {
            key_type: Box::new(k),
            value_type: Box::new(v),
        }
    }
}

impl<T: LegacyDescribe> LegacyDescribe for HashSet<T> {
    fn describe() -> Type {
        let ty = T::describe();
        Type::HashSet {
            element_type: Box::new(ty),
        }
    }
}

impl<K: LegacyDescribe, V: LegacyDescribe> LegacyDescribe for HashMap<K, V> {
    fn describe() -> Type {
        let k = K::describe();
        let v = V::describe();
        Type::HashMap {
            key_type: Box::new(k),
            value_type: Box::new(v),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
                some_type: Box::new(Type::String)
            },
            Option::<String>::describe(),
        );
    }

    #[test]
    pub fn test_array() {
        assert_eq!(
            Type::Array {
                element_type: Box::new(Type::U8),
                length: 3,
            },
            <[u8; 3]>::describe(),
        );
    }

    #[test]
    pub fn test_tuple() {
        assert_eq!(
            Type::Tuple {
                element_types: vec![Type::U8, Type::U128]
            },
            <(u8, u128)>::describe(),
        );
    }
}
