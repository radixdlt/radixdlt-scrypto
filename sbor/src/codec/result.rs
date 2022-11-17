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

impl<X: CustomTypeId, T: Decode<X> + TypeId<X>, E: Decode<X> + TypeId<X>> Decode<X>
    for Result<T, E>
{
    fn decode_with_type_id(
        decoder: &mut Decoder<X>,
        type_id: SborTypeId<X>,
    ) -> Result<Self, DecodeError> {
        decoder.check_preloaded_type_id(type_id, Self::type_id())?;
        let discriminator = decoder.read_discriminator()?;
        match discriminator.as_ref() {
            RESULT_VARIANT_OK => {
                decoder.check_size(1)?;
                Ok(Ok(T::decode(decoder)?))
            }
            RESULT_VARIANT_ERR => {
                decoder.check_size(1)?;
                Ok(Err(E::decode(decoder)?))
            }
            _ => Err(DecodeError::UnknownDiscriminator(discriminator)),
        }
    }
}
