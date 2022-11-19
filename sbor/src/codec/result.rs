use crate::constants::*;
use crate::type_id::*;
use crate::*;

impl<X: CustomTypeId, T: Encode<X>, E: Encode<X>> Encode<X> for Result<T, E> {
    #[inline]
    fn encode_type_id(&self, encoder: &mut Encoder<X>) {
        encoder.write_type_id(Self::type_id());
    }
    #[inline]
    fn encode_body(&self, encoder: &mut Encoder<X>) {
        match self {
            Ok(o) => {
                encoder.write_discriminator(RESULT_VARIANT_OK);
                encoder.write_size(1);
                o.encode(encoder);
            }
            Err(e) => {
                encoder.write_discriminator(RESULT_VARIANT_ERR);
                encoder.write_size(1);
                e.encode(encoder);
            }
        }
    }
}

impl<X: CustomTypeId, D: Decoder<X>, T: Decode<X, D> + TypeId<X>, E: Decode<X, D> + TypeId<X>>
    Decode<X, D> for Result<T, E>
{
    fn decode_body_with_type_id(
        decoder: &mut D,
        type_id: SborTypeId<X>,
    ) -> Result<Self, DecodeError> {
        decoder.check_preloaded_type_id(type_id, Self::type_id())?;
        let discriminator = decoder.read_discriminator()?;
        match discriminator.as_ref() {
            RESULT_VARIANT_OK => {
                decoder.read_and_check_size(1)?;
                Ok(Ok(decoder.decode()?))
            }
            RESULT_VARIANT_ERR => {
                decoder.read_and_check_size(1)?;
                Ok(Err(decoder.decode()?))
            }
            _ => Err(DecodeError::UnknownDiscriminator(discriminator)),
        }
    }
}
