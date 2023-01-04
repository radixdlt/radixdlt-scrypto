use crate::rust::vec::Vec;
use crate::*;

#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type") // See https://serde.rs/enum-representations.html
)]
#[derive(Copy, Debug, Clone, PartialEq, Eq)]
pub enum NoCustomTypeId {}

#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type") // See https://serde.rs/enum-representations.html
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NoCustomValue {}

pub const DEFAULT_BASIC_MAX_DEPTH: u8 = 64;
pub type BasicEncoder<'a> = VecEncoder<'a, NoCustomTypeId, DEFAULT_BASIC_MAX_DEPTH>;
pub type BasicDecoder<'a> = VecDecoder<'a, NoCustomTypeId, DEFAULT_BASIC_MAX_DEPTH>;
pub type BasicSborValue = SborValue<NoCustomTypeId, NoCustomValue>;
pub type BasicSborTypeId = SborTypeId<NoCustomTypeId>;

// 5b for (basic) [5b]or - (90 in decimal)
pub const BASIC_SBOR_V1_PAYLOAD_PREFIX: u8 = 0x5b;

// The following trait "aliases" are to be used in parameters.
//
// They are much nicer to read than the underlying traits, but because they are "new", and are defined
// via blanket impls, they can only be used for parameters, but cannot be used for implementations.
//
// Implementations should instead implement the underlying traits:
// * TypeId<X> (impl over all X: CustomTypeId)
// * Encode<X, E> (impl over all X: CustomTypeId, E: Encoder<X>)
// * Decode<X, D> (impl over all X: CustomTypeId, D: Decoder<X>)
//
// TODO: Change these to be Trait aliases once stable in rust: https://github.com/rust-lang/rust/issues/41517
pub trait BasicTypeId: TypeId<NoCustomTypeId> {}
impl<T: TypeId<NoCustomTypeId> + ?Sized> BasicTypeId for T {}

pub trait BasicDecode: for<'a> Decode<NoCustomTypeId, BasicDecoder<'a>> {}
impl<T: for<'a> Decode<NoCustomTypeId, BasicDecoder<'a>>> BasicDecode for T {}

pub trait BasicEncode: for<'a> Encode<NoCustomTypeId, BasicEncoder<'a>> {}
impl<T: for<'a> Encode<NoCustomTypeId, BasicEncoder<'a>> + ?Sized> BasicEncode for T {}

/// Encode a `T` into byte array.
pub fn basic_encode<T: BasicEncode + ?Sized>(v: &T) -> Result<Vec<u8>, EncodeError> {
    let mut buf = Vec::with_capacity(512);
    let encoder = BasicEncoder::new(&mut buf);
    encoder.encode_payload(v, BASIC_SBOR_V1_PAYLOAD_PREFIX)?;
    Ok(buf)
}

/// Decode an instance of `T` from a slice.
pub fn basic_decode<T: BasicDecode>(buf: &[u8]) -> Result<T, DecodeError> {
    BasicDecoder::new(buf).decode_payload(BASIC_SBOR_V1_PAYLOAD_PREFIX)
}

impl CustomTypeId for NoCustomTypeId {
    fn as_u8(&self) -> u8 {
        panic!("No custom type")
    }

    fn from_u8(_id: u8) -> Option<Self> {
        panic!("No custom type")
    }
}

impl<X: CustomTypeId, E: Encoder<X>> Encode<X, E> for NoCustomValue {
    fn encode_type_id(&self, _encoder: &mut E) -> Result<(), EncodeError> {
        panic!("No custom value")
    }

    fn encode_body(&self, _encoder: &mut E) -> Result<(), EncodeError> {
        panic!("No custom value")
    }
}

impl<X: CustomTypeId, D: Decoder<X>> Decode<X, D> for NoCustomValue {
    fn decode_body_with_type_id(
        _decoder: &mut D,
        _type_id: SborTypeId<X>,
    ) -> Result<Self, DecodeError>
    where
        Self: Sized,
    {
        panic!("No custom value")
    }
}

#[cfg(feature = "schema")]
pub use schema::*;

#[cfg(feature = "schema")]
mod schema {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum NoCustomTypeSchema {}

    impl CustomTypeSchema for NoCustomTypeSchema {
        type CustomTypeId = NoCustomTypeId;
    }

    pub enum NoCustomWellKnownTypes {}

    impl CustomWellKnownType for NoCustomWellKnownTypes {
        type CustomTypeSchema = NoCustomTypeSchema;

        fn from_well_known_index(
            _well_known_index: u8,
        ) -> Option<LocalTypeData<Self::CustomTypeSchema, SchemaLocalTypeRef>> {
            None
        }
    }

    impl CompleteCustomTypeSchema for NoCustomTypeSchema {
        type WellKnownTypes = NoCustomWellKnownTypes;
    }

    impl LinearizableCustomTypeSchema for NoCustomTypeSchema {
        type Linearized = NoCustomTypeSchema;

        fn linearize(self, _schemas: &indexmap::IndexSet<ComplexTypeHash>) -> Self::Linearized {
            self
        }
    }

    pub type BasicTypeSchema<L> = TypeSchema<NoCustomTypeId, NoCustomTypeSchema, L>;
    pub type BasicLinearTypeSchema = BasicTypeSchema<SchemaLocalTypeRef>;
    pub type BasicFullTypeSchema = FullTypeSchema<NoCustomTypeSchema>;
}
