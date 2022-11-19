use crate::decoder::*;
use crate::type_id::*;

/// A data structure that can be decoded from a byte array using SBOR.
pub trait Decode<X: CustomTypeId, D: Decoder<X>>: Sized {
    /// Decodes from the byte array encapsulated by the given decoder, with a preloaded type id.
    fn decode_body_with_type_id(
        decoder: &mut D,
        type_id: SborTypeId<X>,
    ) -> Result<Self, DecodeError>;
}
