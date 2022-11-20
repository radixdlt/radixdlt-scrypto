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
use sbor::{
    Decode, DecodeError, Decoder, Encode, EncodeError, Encoder, SborTypeId, SborValue, TypeId,
    VecDecoder, VecEncoder,
};
pub use schema_matcher::*;
pub use schema_path::*;
pub use value_formatter::*;

pub const MAX_SCRYPTO_SBOR_DEPTH: u8 = 64;

pub type ScryptoEncoder<'a> = VecEncoder<'a, ScryptoCustomTypeId, MAX_SCRYPTO_SBOR_DEPTH>;
pub type ScryptoDecoder<'a> = VecDecoder<'a, ScryptoCustomTypeId, MAX_SCRYPTO_SBOR_DEPTH>;
pub type ScryptoSborTypeId = SborTypeId<ScryptoCustomTypeId>;
pub type ScryptoValue = SborValue<ScryptoCustomTypeId, ScryptoCustomValue>;

// These trait "aliases" should only be used for parameters, never implementations
// Implementations should implement the underlying traits (TypeId<ScryptoCustomTypeId>/Encode<ScryptoCustomTypeId, E>/Decode<ScryptoCustomTypeId, D>)
pub trait ScryptoTypeId: TypeId<ScryptoCustomTypeId> {}
impl<T: TypeId<ScryptoCustomTypeId> + ?Sized> ScryptoTypeId for T {}

pub trait ScryptoDecode: for<'a> Decode<ScryptoCustomTypeId, ScryptoDecoder<'a>> {}
impl<T: for<'a> Decode<ScryptoCustomTypeId, ScryptoDecoder<'a>>> ScryptoDecode for T {}

pub trait ScryptoEncode: for<'a> Encode<ScryptoCustomTypeId, ScryptoEncoder<'a>> {}
impl<T: for<'a> Encode<ScryptoCustomTypeId, ScryptoEncoder<'a>> + ?Sized> ScryptoEncode for T {}

/// Encodes a data structure into byte array.
pub fn scrypto_encode<T: ScryptoEncode + ?Sized>(value: &T) -> Result<Vec<u8>, EncodeError> {
    let mut buf = Vec::with_capacity(512);
    let encoder = ScryptoEncoder::new(&mut buf);
    encoder.encode_payload(value)?;
    Ok(buf)
}

/// Decodes a data structure from a byte array.
pub fn scrypto_decode<T: ScryptoDecode>(buf: &[u8]) -> Result<T, DecodeError> {
    ScryptoDecoder::new(buf).decode_payload()
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
        use ::sbor::Encoder;
        let mut buf = ::sbor::rust::vec::Vec::new();
        let mut encoder = radix_engine_interface::data::ScryptoEncoder::new(&mut buf);
        encoder.write_type_id(radix_engine_interface::data::ScryptoSborTypeId::Struct).unwrap();
        // Hack: stringify to skip ownership move semantics
        encoder.write_size(radix_engine_interface::count!($(stringify!($args)),*)).unwrap();
        $(
            let arg = $args;
            encoder.encode(&arg).unwrap();
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
            .unwrap()
        )
    }

    #[test]
    fn test_args_with_non_fungible_id() {
        let id = NonFungibleId::from_u32(1);
        let _x = args!(BTreeSet::from([id]));
    }
}
