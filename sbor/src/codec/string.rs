use crate::rust::string::String;
use crate::type_id::*;
use crate::*;

impl<X: CustomTypeId> Encode<X> for str {
    #[inline]
    fn encode_type_id(&self, encoder: &mut Encoder<X>) {
        encoder.write_type_id(Self::type_id());
    }
    #[inline]
    fn encode_body(&self, encoder: &mut Encoder<X>) {
        encoder.write_size(self.len());
        encoder.write_slice(self.as_bytes());
    }
}

impl<X: CustomTypeId> Encode<X> for &str {
    #[inline]
    fn encode_type_id(&self, encoder: &mut Encoder<X>) {
        encoder.write_type_id(Self::type_id());
    }
    #[inline]
    fn encode_body(&self, encoder: &mut Encoder<X>) {
        encoder.write_size(self.len());
        encoder.write_slice(self.as_bytes());
    }
}

impl<X: CustomTypeId> Encode<X> for String {
    #[inline]
    fn encode_type_id(&self, encoder: &mut Encoder<X>) {
        encoder.write_type_id(Self::type_id());
    }
    #[inline]
    fn encode_body(&self, encoder: &mut Encoder<X>) {
        self.as_str().encode_body(encoder);
    }
}

impl<X: CustomTypeId> Decode<X> for String {
    fn decode_with_type_id(
        decoder: &mut Decoder<X>,
        type_id: SborTypeId<X>,
    ) -> Result<Self, DecodeError> {
        decoder.check_preloaded_type_id(type_id, Self::type_id())?;
        let len = decoder.read_size()?;
        let slice = decoder.read_slice(len)?;
        String::from_utf8(slice.to_vec()).map_err(|_| DecodeError::InvalidUtf8)
    }
}
