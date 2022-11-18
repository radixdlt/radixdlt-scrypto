/// Defines the custom type ID scrypto uses.
mod custom_type_id;
/// Defines the model of Scrypto custom values.
mod custom_value;
/// Indexed Scrypto value.
mod indexed_value;
/// Matches a Scrypto schema type with a Scrypto value.
mod schema_matcher;
/// Defines a way to uniquely identify an element within a Scrypto schema type.
mod schema_path;
/// Format any Scrypto value using the Manifest syntax.
mod value_formatter;

pub use crate::args;
pub use custom_type_id::*;
pub use custom_value::*;
pub use indexed_value::*;
use sbor::rust::vec::Vec;
use sbor::{decode, encode, Decode, DecodeError, Encode};
pub use schema_matcher::*;
pub use schema_path::*;
pub use value_formatter::*;

// TODO: add trait alias for `Encode` and `Decode` as well, once it becomes stable.

pub type ScryptoEncoder<'a> = sbor::Encoder<'a, ScryptoCustomTypeId>;
pub type ScryptoDecoder<'a> = sbor::Decoder<'a, ScryptoCustomTypeId>;
pub type ScryptoTypeId = sbor::SborTypeId<ScryptoCustomTypeId>;
pub type ScryptoValue = sbor::SborValue<ScryptoCustomTypeId, ScryptoCustomValue>;

/// Encodes a data structure into byte array.
pub fn scrypto_encode<T: Encode<ScryptoCustomTypeId> + ?Sized>(v: &T) -> Vec<u8> {
    encode(v)
}

pub fn scrypto_decode<T: Decode<ScryptoCustomTypeId>>(buf: &[u8]) -> Result<T, DecodeError> {
    decode(buf)
}

#[macro_export]
macro_rules! count {
    () => {0usize};
    ($a:expr) => {1usize};
    ($a:expr, $($rest:expr),*) => {1usize + radix_engine_interface::count!($($rest),*)};
}

/// Constructs argument list for Scrypto function/method invocation.
#[macro_export]
macro_rules! args {
    ($($args: expr),*) => {{
        use ::sbor::Encode;
        let mut buf = ::sbor::rust::vec::Vec::new();
        let mut encoder = radix_engine_interface::data::ScryptoEncoder::new(&mut buf);
        encoder.write_type_id(radix_engine_interface::data::ScryptoTypeId::Struct);
        // Hack: stringify to skip ownership move semantics
        encoder.write_size(radix_engine_interface::count!($(stringify!($args)),*));
        $(
            let arg = $args;
            arg.encode_type_id(&mut encoder);
            arg.encode_body(&mut encoder);
        )*
        buf
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::*;
    use crate::scrypto;
    use sbor::rust::borrow::ToOwned;
    use sbor::rust::collections::BTreeSet;
    use sbor::rust::string::String;

    #[test]
    fn test_args() {
        #[scrypto(Encode, Decode, TypeId)]
        struct A {
            a: u32,
            b: String,
        }

        assert_eq!(
            args!(1u32, "abc"),
            scrypto_encode(&A {
                a: 1,
                b: "abc".to_owned(),
            })
        )
    }

    #[test]
    fn test_args_with_non_fungible_id() {
        let id = NonFungibleId::from_u32(1);
        let _x = args!(BTreeSet::from([id]));
    }
}
