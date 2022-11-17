use crate::type_id::*;
use crate::*;

impl<X: CustomTypeId> Encode<X> for () {
    #[inline]
    fn encode_type_id(&self, encoder: &mut Encoder<X>) {
        encoder.write_type_id(Self::type_id());
    }
    #[inline]
    fn encode_body(&self, encoder: &mut Encoder<X>) {
        encoder.write_byte(0);
    }
}

impl<X: CustomTypeId> Decode<X> for () {
    fn decode_with_type_id(
        decoder: &mut Decoder<X>,
        type_id: SborTypeId<X>,
    ) -> Result<Self, DecodeError> {
        decoder.check_preloaded_type_id(type_id, Self::type_id())?;
        let value = decoder.read_byte()?;
        match value {
            0 => Ok(()),
            _ => Err(DecodeError::InvalidUnit(value)),
        }
    }
}
