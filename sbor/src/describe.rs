use crate::rust::boxed::Box;
use crate::rust::collections::*;
use crate::rust::string::String;
use crate::rust::vec;
use crate::rust::vec::Vec;
use crate::sbor::{Decode, Encode, TypeId};
use sbor::Value;

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

    Option {
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

    Result {
        okay: Box<Type>,
        error: Box<Type>,
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
        type_id: u8,
        generics: Vec<Type>,
    },

    Any,
}

impl Type {
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
            Type::Option { value: type_value } => {
                if let Value::Option { value } = value {
                    match &**value {
                        None => true,
                        Some(value) => type_value.matches(value),
                    }
                } else {
                    false
                }
            }
            Type::Array {
                element: type_element,
                length,
            } => {
                if let Value::Array {
                    element_type_id: _,
                    elements,
                } = value
                {
                    let length = usize::from(*length);
                    length == elements.len() && elements.iter().all(|v| type_element.matches(v))
                } else {
                    false
                }
            }
            Type::Tuple {
                elements: type_elements,
            } => {
                if let Value::Tuple { elements } = value {
                    type_elements.len() == elements.len()
                        && type_elements
                            .iter()
                            .enumerate()
                            .all(|(i, e)| e.matches(elements.get(i).unwrap()))
                } else {
                    false
                }
            }
            Type::Result { okay, error } => {
                if let Value::Result { value } = value {
                    match &**value {
                        Result::Ok(v) => okay.matches(v),
                        Result::Err(e) => error.matches(e),
                    }
                } else {
                    false
                }
            }
            Type::TreeSet {
                element: type_element,
            } => {
                if let Value::Set {
                    element_type_id: _,
                    elements,
                } = value
                {
                    elements.iter().all(|v| type_element.matches(v))
                } else {
                    false
                }
            }
            Type::TreeMap {
                key: type_key,
                value: type_value,
            } => {
                if let Value::Map {
                    key_type_id: _,
                    value_type_id: _,
                    elements,
                } = value
                {
                    elements.iter().enumerate().all(|(i, e)| {
                        if i % 2 == 0 {
                            type_key.matches(e)
                        } else {
                            type_value.matches(e)
                        }
                    })
                } else {
                    false
                }
            }
            Type::Vec {
                element: type_element,
            } => {
                if let Value::List {
                    element_type_id: _,
                    elements,
                } = value
                {
                    elements.iter().all(|v| type_element.matches(v))
                } else {
                    false
                }
            }
            Type::HashSet {
                element: type_element,
            } => {
                if let Value::Set {
                    element_type_id: _,
                    elements,
                } = value
                {
                    elements.iter().all(|v| type_element.matches(v))
                } else {
                    false
                }
            }
            Type::HashMap {
                key: type_key,
                value: type_value,
            } => {
                if let Value::Map {
                    key_type_id: _,
                    value_type_id: _,
                    elements,
                } = value
                {
                    elements.iter().enumerate().all(|(i, e)| {
                        if i % 2 == 0 {
                            type_key.matches(e)
                        } else {
                            type_value.matches(e)
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
                if let Value::Enum { name, fields } = value {
                    for variant in type_variants {
                        if variant.name.eq(name) {
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

impl<T: Describe, E: Describe> Describe for Result<T, E> {
    fn describe() -> Type {
        let t = T::describe();
        let e = E::describe();
        Type::Result {
            okay: Box::new(t),
            error: Box::new(e),
        }
    }
}

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
