use crate::rust::string::String;
use crate::value_kind::*;
use crate::*;

categorize_simple!(str, ValueKind::String);
categorize_simple!(String, ValueKind::String);

impl<X: CustomValueKind, E: Encoder<X>> Encode<X, E> for str {
    #[inline]
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(Self::value_kind())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_size(self.len())?;
        encoder.write_slice(self.as_bytes())?;
        Ok(())
    }
}

impl<X: CustomValueKind, E: Encoder<X>> Encode<X, E> for String {
    #[inline]
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(Self::value_kind())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.as_str().encode_body(encoder)
    }
}

impl<X: CustomValueKind, D: Decoder<X>> Decode<X, D> for String {
    #[inline]
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<X>,
    ) -> Result<Self, DecodeError> {
        decoder.check_preloaded_value_kind(value_kind, Self::value_kind())?;
        let len = decoder.read_size()?;
        let slice = decoder.read_slice(len)?;
        String::from_utf8(slice.to_vec()).map_err(|_| DecodeError::InvalidUtf8)
    }
}

pub use schema::*;

mod schema {
    use super::*;

    describe_basic_well_known_type!(String, STRING_ID);
    describe_basic_well_known_type!(str, STRING_ID);
}
