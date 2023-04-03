/// Defines the custom Scrypto schema types.
mod custom_schema;
/// Defines custom serialization of the types.
mod custom_serde;
/// Defines how to traverse scrypto custom types.
mod custom_traversal;
/// Defines the model of Scrypto custom values.
mod custom_value;
/// Defines the custom value kind model that scrypto uses.
mod custom_value_kind;
/// Defines the scrypto custom well known types.
mod custom_well_known_types;
/// Defines a way to uniquely identify an element within a Scrypto schema type.
mod schema_path;

pub mod model;

pub use custom_schema::*;
pub use custom_serde::*;
pub use custom_traversal::*;
pub use custom_value::*;
pub use custom_value_kind::*;
pub use custom_well_known_types::*;
pub use schema_path::*;

use sbor::rust::vec::Vec;
use sbor::traversal::VecTraverser;
use sbor::*;

// 0x5c for [5c]rypto - (91 in decimal)
pub const SCRYPTO_SBOR_V1_PAYLOAD_PREFIX: u8 = 0x5c;
pub const SCRYPTO_SBOR_V1_MAX_DEPTH: usize = 64;

pub type ScryptoEncoder<'a> = VecEncoder<'a, ScryptoCustomValueKind>;
pub type ScryptoDecoder<'a> = VecDecoder<'a, ScryptoCustomValueKind>;
pub type ScryptoTraverser<'a> = VecTraverser<'a, ScryptoCustomTraversal>;
pub type ScryptoValueKind = ValueKind<ScryptoCustomValueKind>;
pub type ScryptoValue = Value<ScryptoCustomValueKind, ScryptoCustomValue>;

// The following trait "aliases" are to be used in parameters.
//
// They are much nicer to read than the underlying traits, but because they are "new", and are defined
// via blanket impls, they can only be used for parameters, but cannot be used for implementations.
//
// Implementations should instead implement the underlying traits:
// * Categorize<ScryptoCustomValueKind>
// * Encode<ScryptoCustomValueKind, E> (impl over all E: Encoder<ScryptoCustomValueKind>)
// * Decode<ScryptoCustomValueKind, D> (impl over all D: Decoder<ScryptoCustomValueKind>)
//
// TODO: Change these to be Trait aliases once stable in rust: https://github.com/rust-lang/rust/issues/41517
pub trait ScryptoCategorize: Categorize<ScryptoCustomValueKind> {}
impl<T: Categorize<ScryptoCustomValueKind> + ?Sized> ScryptoCategorize for T {}

pub trait ScryptoDecode: for<'a> Decode<ScryptoCustomValueKind, ScryptoDecoder<'a>> {}
impl<T: for<'a> Decode<ScryptoCustomValueKind, ScryptoDecoder<'a>>> ScryptoDecode for T {}

pub trait ScryptoEncode: for<'a> Encode<ScryptoCustomValueKind, ScryptoEncoder<'a>> {}
impl<T: for<'a> Encode<ScryptoCustomValueKind, ScryptoEncoder<'a>> + ?Sized> ScryptoEncode for T {}

pub trait ScryptoDescribe: Describe<ScryptoCustomTypeKind> {}
impl<T: Describe<ScryptoCustomTypeKind>> ScryptoDescribe for T {}

pub trait ScryptoSbor: ScryptoCategorize + ScryptoDecode + ScryptoEncode + ScryptoDescribe {}
impl<T: ScryptoCategorize + ScryptoDecode + ScryptoEncode + ScryptoDescribe> ScryptoSbor for T {}

/// Encodes a data structure into byte array.
pub fn scrypto_encode<T: ScryptoEncode + ?Sized>(value: &T) -> Result<Vec<u8>, EncodeError> {
    let mut buf = Vec::with_capacity(512);
    let encoder = ScryptoEncoder::new(&mut buf, SCRYPTO_SBOR_V1_MAX_DEPTH);
    encoder.encode_payload(value, SCRYPTO_SBOR_V1_PAYLOAD_PREFIX)?;
    Ok(buf)
}

/// Decodes a data structure from a byte array.
pub fn scrypto_decode<T: ScryptoDecode>(buf: &[u8]) -> Result<T, DecodeError> {
    ScryptoDecoder::new(buf, SCRYPTO_SBOR_V1_MAX_DEPTH)
        .decode_payload(SCRYPTO_SBOR_V1_PAYLOAD_PREFIX)
}
