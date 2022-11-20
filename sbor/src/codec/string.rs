use crate::rust::string::String;
use crate::type_id::*;
use crate::*;

impl<X: CustomTypeId, E: Encoder<X>> Encode<X, E> for str {
    #[inline]
    fn encode_type_id(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_type_id(Self::type_id())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_size(self.len())?;
        encoder.write_slice(self.as_bytes())?;
        Ok(())
    }
}

impl<X: CustomTypeId, E: Encoder<X>> Encode<X, E> for &str {
    #[inline]
    fn encode_type_id(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_type_id(Self::type_id())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_size(self.len())?;
        encoder.write_slice(self.as_bytes())?;
        Ok(())
    }
}

impl<X: CustomTypeId, E: Encoder<X>> Encode<X, E> for String {
    #[inline]
    fn encode_type_id(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_type_id(Self::type_id())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.encode_body(self.as_str())
    }
}

impl<X: CustomTypeId, D: Decoder<X>> Decode<X, D> for String {
    #[inline]
    fn decode_body_with_type_id(
        decoder: &mut D,
        type_id: SborTypeId<X>,
    ) -> Result<Self, DecodeError> {
        decoder.check_preloaded_type_id(type_id, Self::type_id())?;
        let len = decoder.read_size()?;
        let slice = decoder.read_slice(len)?;
        String::from_utf8(slice.to_vec()).map_err(|_| DecodeError::InvalidUtf8)
    }
}
