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

/// Constructs argument list for Scrypto function/method invocation.
#[macro_export]
macro_rules! args {
    ($($args: expr),*) => {{
        let mut fields = Vec::new();
        $(
            let encoded = scrypto_encode(&$args);
            fields.push(decode_any::<ScryptoCustomTypeId, ScryptoCustomValue>(&encoded).unwrap());
        )*
        let input_struct = ::sbor::SborValue::Struct {
            fields,
        };
        encode_any::<ScryptoCustomTypeId, ScryptoCustomValue>(&input_struct)
    }};
}
