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
    use alloc::string::String;

    use serde::Serialize;

    use crate as abi;
    use abi::Describe;

    fn assert_json<T: Serialize>(expected: &str, value: T) {
        let json = serde_json::to_string(&value).unwrap();
        assert_eq!(expected, json);
    }

    #[test]
    pub fn test_basic_types() {
        assert_json("{\"type\":\"Bool\"}", bool::describe());
        assert_json("{\"type\":\"I8\"}", i8::describe());
        assert_json("{\"type\":\"I16\"}", i16::describe());
        assert_json("{\"type\":\"I32\"}", i32::describe());
        assert_json("{\"type\":\"I64\"}", i64::describe());
        assert_json("{\"type\":\"I128\"}", i128::describe());
        assert_json("{\"type\":\"U8\"}", u8::describe());
        assert_json("{\"type\":\"U16\"}", u16::describe());
        assert_json("{\"type\":\"U32\"}", u32::describe());
        assert_json("{\"type\":\"U64\"}", u64::describe());
        assert_json("{\"type\":\"U128\"}", u128::describe());
        assert_json("{\"type\":\"String\"}", str::describe());
        assert_json("{\"type\":\"String\"}", String::describe());
    }

    #[test]
    pub fn test_option() {
        assert_json(
            "{\"type\":\"Option\",\"value\":{\"type\":\"String\"}}",
            Option::<String>::describe(),
        );
    }

    #[test]
    pub fn test_array() {
        assert_json(
            "{\"type\":\"Array\",\"base\":{\"type\":\"U8\"}}",
            <[u8]>::describe(),
        );
    }

    #[test]
    pub fn test_tuple() {
        assert_json(
            "{\"type\":\"Tuple\",\"elements\":[{\"type\":\"U8\"},{\"type\":\"U128\"}]}",
            <(u8, u128)>::describe(),
        );
    }
}
