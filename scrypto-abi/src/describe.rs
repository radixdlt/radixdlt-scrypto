extern crate alloc;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use crate::*;

/// Describe a data format
pub trait Describe {
    fn describe() -> Type;
}

macro_rules! describe_basic_type {
    ($type:ident, $abi_type:expr) => {
        impl Describe for $type {
            fn describe() -> Type {
                $abi_type
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
describe_basic_type!(String, Type::String);

impl<T: Describe> Describe for Option<T> {
    fn describe() -> Type {
        let ty = T::describe();
        Type::Option {
            value: Box::new(ty),
        }
    }
}

impl<T: Describe> Describe for [T] {
    fn describe() -> Type {
        let ty = T::describe();
        Type::Array { base: Box::new(ty) }
    }
}

impl<T: Describe> Describe for Vec<T> {
    fn describe() -> Type {
        let ty = T::describe();
        Type::Array { base: Box::new(ty) }
    }
}

macro_rules! tuple_impl {
    ($($name:ident)+) => {
        impl<$($name: Describe),+> Describe for ($($name,)+) {
            fn describe() -> Type {
                let mut elements = vec!();
                $(elements.push($name::describe());)+
                Type::Tuple { elements }
            }
        }
    };
}

tuple_impl! { A }
tuple_impl! { A B }
tuple_impl! { A B C }
tuple_impl! { A B C D }
tuple_impl! { A B C D E }
tuple_impl! { A B C D E F }
tuple_impl! { A B C D E F G }
tuple_impl! { A B C D E F G H }
tuple_impl! { A B C D E F G H I }
tuple_impl! { A B C D E F G H I J }

#[cfg(test)]
mod tests {
    extern crate alloc;
    use alloc::collections::BTreeMap;
    use alloc::string::String;
    use alloc::string::ToString;
    use alloc::vec;

    use crate as abi;
    use abi::Describe;

    #[allow(dead_code)]
    struct X {
        a: u32,
        b: String,
        c: (u8, u16),
    }

    impl abi::Describe for X {
        fn describe() -> abi::Type {
            let mut fields = BTreeMap::new();
            fields.insert("a".to_string(), u32::describe());
            fields.insert("b".to_string(), String::describe());
            fields.insert("c".to_string(), <(u8, u16)>::describe());

            abi::Type::Struct {
                name: "Y".to_string(),
                fields: abi::Fields::Named { fields },
            }
        }
    }

    impl X {
        #[allow(dead_code)]
        pub fn new() -> Self {
            Self {
                a: 0,
                b: "hello".to_string(),
                c: (1, 2),
            }
        }

        #[allow(dead_code)]
        pub fn add(&self, a: u32, b: u32) -> u32 {
            a + b
        }
    }

    #[allow(non_snake_case)]
    pub fn X_abi() -> abi::Component {
        abi::Component {
            name: "X".to_string(),
            methods: vec![
                abi::Method {
                    name: "new".to_string(),
                    kind: abi::MethodKind::Functional,
                    mutability: abi::Mutability::Immutable,
                    inputs: vec![],
                    output: X::describe(),
                },
                abi::Method {
                    name: "add".to_string(),
                    kind: abi::MethodKind::Stateful,
                    mutability: abi::Mutability::Immutable,
                    inputs: vec![u32::describe(), u32::describe()],
                    output: u32::describe(),
                },
            ],
        }
    }

    #[test]
    pub fn test_abi_describe() {
        #[allow(unused_variables)]
        let json = serde_json::to_string_pretty(&X_abi()).unwrap();

        #[cfg(feature = "std")]
        println!("{}", json);
    }
}
