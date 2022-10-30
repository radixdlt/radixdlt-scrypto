use sbor::rust::boxed::Box;
use sbor::rust::collections::*;
use sbor::rust::string::String;
use sbor::rust::vec;
use sbor::rust::vec::Vec;
use sbor::type_id::*;
use sbor::Value;
use sbor::{Decode, Encode, TypeId};

/// Represents a SBOR type.
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type")  // For JSON readability, see https://serde.rs/enum-representations.html
)]
#[derive(Debug, Clone, PartialEq, Eq, TypeId, Decode, Encode)]
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

    Array {
        element_type: Box<Type>,
        length: u16,
    },

    Tuple {
        element_types: Vec<Type>,
    },

    Struct {
        name: String,
        fields: Fields,
    },

    Enum {
        name: String,
        variants: Vec<Variant>, // Order matters as it decides of the variant discriminator
    },

    Option {
        some_type: Box<Type>,
    },

    Result {
        okay_type: Box<Type>,
        err_type: Box<Type>,
    },

    Vec {
        element_type: Box<Type>,
    },

    TreeSet {
        element_type: Box<Type>,
    },

    TreeMap {
        key_type: Box<Type>,
        value_type: Box<Type>,
    },

    HashSet {
        element_type: Box<Type>,
    },

    HashMap {
        key_type: Box<Type>,
        value_type: Box<Type>,
    },

    Custom {
        type_id: u8,
        generics: Vec<Type>,
    },

    // TODO: remove
    // Currently used by `ProofRule` because recursion is not supported
    Any,
}

impl Type {
    pub fn id(&self) -> Option<u8> {
        match self {
            Type::Unit => Some(TYPE_UNIT),
            Type::Bool => Some(TYPE_BOOL),
            Type::I8 => Some(TYPE_I8),
            Type::I16 => Some(TYPE_I16),
            Type::I32 => Some(TYPE_I32),
            Type::I64 => Some(TYPE_I64),
            Type::I128 => Some(TYPE_I128),
            Type::U8 => Some(TYPE_U8),
            Type::U16 => Some(TYPE_U16),
            Type::U32 => Some(TYPE_U32),
            Type::U64 => Some(TYPE_U64),
            Type::U128 => Some(TYPE_U128),
            Type::String => Some(TYPE_STRING),
            Type::Array { .. } => Some(TYPE_ARRAY),
            Type::Tuple { .. } => Some(TYPE_TUPLE),
            Type::Struct { .. } => Some(TYPE_STRUCT),
            Type::Enum { .. } => Some(TYPE_ENUM),
            Type::Option { .. } => Some(TYPE_ENUM),
            Type::Result { .. } => Some(TYPE_ENUM),
            Type::Vec { .. } => Some(TYPE_ARRAY),
            Type::TreeSet { .. } => Some(TYPE_ARRAY),
            Type::TreeMap { .. } => Some(TYPE_ARRAY),
            Type::HashSet { .. } => Some(TYPE_ARRAY),
            Type::HashMap { .. } => Some(TYPE_ARRAY),
            Type::Custom { type_id, .. } => Some(*type_id),
            Type::Any => None,
        }
    }

    pub fn matches(&self, value: &Value) -> bool {
        match self {
            Type::Unit => matches!(value, Value::Unit),
            Type::Bool => matches!(value, Value::Bool { .. }),
            Type::I8 => matches!(value, Value::I8 { .. }),
            Type::I16 => matches!(value, Value::I16 { .. }),
            Type::I32 => matches!(value, Value::I32 { .. }),
            Type::I64 => matches!(value, Value::I64 { .. }),
            Type::I128 => matches!(value, Value::I128 { .. }),
            Type::U8 => matches!(value, Value::U8 { .. }),
            Type::U16 => matches!(value, Value::U16 { .. }),
            Type::U32 => matches!(value, Value::U32 { .. }),
            Type::U64 => matches!(value, Value::U64 { .. }),
            Type::U128 => matches!(value, Value::U128 { .. }),
            Type::String => matches!(value, Value::String { .. }),
            Type::Array {
                element_type,
                length,
            } => {
                if let Value::Array {
                    element_type_id,
                    elements,
                } = value
                {
                    let element_type_matches = match element_type.id() {
                        Some(id) => id == *element_type_id,
                        None => true,
                    };
                    element_type_matches
                        && usize::from(*length) == elements.len()
                        && elements.iter().all(|v| element_type.matches(v))
                } else {
                    false
                }
            }
            Type::Tuple { element_types } => {
                if let Value::Tuple { elements } = value {
                    element_types.len() == elements.len()
                        && element_types
                            .iter()
                            .enumerate()
                            .all(|(i, e)| e.matches(elements.get(i).unwrap()))
                } else {
                    false
                }
            }
            Type::Option { some_type } => {
                if let Value::Enum {
                    discriminator,
                    fields,
                } = value
                {
                    match discriminator.as_str() {
                        OPTION_VARIANT_SOME => fields.len() == 1 && some_type.matches(&fields[0]),
                        OPTION_VARIANT_NONE => fields.len() == 0,
                        _ => false,
                    }
                } else {
                    false
                }
            }
            Type::Result {
                okay_type,
                err_type,
            } => {
                if let Value::Enum {
                    discriminator,
                    fields,
                } = value
                {
                    match discriminator.as_str() {
                        RESULT_VARIANT_OK => fields.len() == 1 && okay_type.matches(&fields[0]),
                        RESULT_VARIANT_ERR => fields.len() == 1 && err_type.matches(&fields[0]),
                        _ => false,
                    }
                } else {
                    false
                }
            }
            Type::Vec { element_type }
            | Type::HashSet { element_type }
            | Type::TreeSet { element_type } => {
                if let Value::Array {
                    element_type_id,
                    elements,
                } = value
                {
                    let element_type_matches = match element_type.id() {
                        Some(id) => id == *element_type_id,
                        None => true,
                    };
                    element_type_matches && elements.iter().all(|v| element_type.matches(v))
                } else {
                    false
                }
            }
            Type::TreeMap {
                key_type,
                value_type,
            }
            | Type::HashMap {
                key_type,
                value_type,
            } => {
                if let Value::Array {
                    element_type_id,
                    elements,
                } = value
                {
                    *element_type_id == TYPE_TUPLE
                        && elements.iter().all(|e| {
                            if let Value::Tuple { elements } = e {
                                elements.len() == 2
                                    && key_type.matches(&elements[0])
                                    && value_type.matches(&elements[1])
                            } else {
                                false
                            }
                        })
                } else {
                    false
                }
            }
            Type::Struct {
                name: _,
                fields: type_fields,
            } => {
                if let Value::Struct { fields } = value {
                    match type_fields {
                        Fields::Unit => fields.is_empty(),
                        Fields::Unnamed { unnamed } => {
                            unnamed.len() == fields.len()
                                && unnamed
                                    .iter()
                                    .enumerate()
                                    .all(|(i, e)| e.matches(fields.get(i).unwrap()))
                        }
                        Fields::Named { named } => {
                            named.len() == fields.len()
                                && named
                                    .iter()
                                    .enumerate()
                                    .all(|(i, (_, e))| e.matches(fields.get(i).unwrap()))
                        }
                    }
                } else {
                    false
                }
            }
            Type::Enum {
                name: _,
                variants: type_variants,
            } => {
                if let Value::Enum {
                    discriminator,
                    fields,
                } = value
                {
                    for variant in type_variants {
                        if variant.name.eq(discriminator) {
                            return match &variant.fields {
                                Fields::Unit => fields.is_empty(),
                                Fields::Unnamed { unnamed } => {
                                    unnamed.len() == fields.len()
                                        && unnamed
                                            .iter()
                                            .enumerate()
                                            .all(|(i, e)| e.matches(fields.get(i).unwrap()))
                                }
                                Fields::Named { named } => {
                                    named.len() == fields.len()
                                        && named
                                            .iter()
                                            .enumerate()
                                            .all(|(i, (_, e))| e.matches(fields.get(i).unwrap()))
                                }
                            };
                        }
                    }
                    false
                } else {
                    false
                }
            }
            Type::Custom {
                type_id: type_type_id,
                generics: _,
            } => {
                if let Value::Custom { type_id, bytes: _ } = value {
                    // TODO: check generics
                    *type_type_id == *type_id
                } else {
                    false
                }
            }
            Type::Any => true,
        }
    }
}

/// Represents the type info of an enum variant.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, TypeId, Decode, Encode)]
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
#[derive(Debug, Clone, PartialEq, Eq, TypeId, Decode, Encode)]
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

describe_basic_type!(isize, Type::I64);
describe_basic_type!(usize, Type::U64);

describe_basic_type!(str, Type::String);
describe_basic_type!(String, Type::String);

impl<T: Describe> Describe for Option<T> {
    fn describe() -> Type {
        let ty = T::describe();
        Type::Option {
            some_type: Box::new(ty),
        }
    }
}

impl<T: Describe, const N: usize> Describe for [T; N] {
    fn describe() -> Type {
        let ty = T::describe();
        Type::Array {
            element_type: Box::new(ty),
            length: N as u16,
        }
    }
}

macro_rules! describe_tuple {
    ($($name:ident)+) => {
        impl<$($name: Describe),+> Describe for ($($name,)+) {
            fn describe() -> Type {
                Type::Tuple { element_types: vec![ $($name::describe(),)* ] }
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

impl<T: Describe, E: Describe> Describe for Result<T, E> {
    fn describe() -> Type {
        let t = T::describe();
        let e = E::describe();
        Type::Result {
            okay_type: Box::new(t),
            err_type: Box::new(e),
        }
    }
}

impl<T: Describe> Describe for Vec<T> {
    fn describe() -> Type {
        let ty = T::describe();
        Type::Vec {
            element_type: Box::new(ty),
        }
    }
}

impl<T: Describe> Describe for BTreeSet<T> {
    fn describe() -> Type {
        let ty = T::describe();
        Type::TreeSet {
            element_type: Box::new(ty),
        }
    }
}

impl<K: Describe, V: Describe> Describe for BTreeMap<K, V> {
    fn describe() -> Type {
        let k = K::describe();
        let v = V::describe();
        Type::TreeMap {
            key_type: Box::new(k),
            value_type: Box::new(v),
        }
    }
}

impl<T: Describe> Describe for HashSet<T> {
    fn describe() -> Type {
        let ty = T::describe();
        Type::HashSet {
            element_type: Box::new(ty),
        }
    }
}

impl<K: Describe, V: Describe> Describe for HashMap<K, V> {
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
