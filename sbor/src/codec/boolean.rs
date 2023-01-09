use crate::type_id::*;
use crate::*;

impl<X: CustomTypeId, E: Encoder<X>> Encode<X, E> for bool {
    #[inline]
    fn encode_type_id(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_type_id(Self::type_id())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_byte(if *self { 1u8 } else { 0u8 })
    }
}

impl<X: CustomTypeId, D: Decoder<X>> Decode<X, D> for bool {
    #[inline]
    fn decode_body_with_type_id(
        decoder: &mut D,
        type_id: SborTypeId<X>,
    ) -> Result<Self, DecodeError> {
        decoder.check_preloaded_type_id(type_id, Self::type_id())?;
        let value = decoder.read_byte()?;
        match value {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(DecodeError::InvalidBool(value)),
        }
    }
}

#[cfg(feature = "schema")]
well_known_basic_type!(bool, BOOL_ID);
