use crate::value_kind::*;
use crate::*;

categorize_simple!(bool, ValueKind::Bool);

impl<X: CustomValueKind, E: Encoder<X>> Encode<X, E> for bool {
    #[inline]
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(Self::value_kind())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_byte(if *self { 1u8 } else { 0u8 })
    }
}

impl<X: CustomValueKind, D: Decoder<X>> Decode<X, D> for bool {
    #[inline]
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<X>,
    ) -> Result<Self, DecodeError> {
        decoder.check_preloaded_value_kind(value_kind, Self::value_kind())?;
        let value = decoder.read_byte()?;
        match value {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(DecodeError::InvalidBool(value)),
        }
    }
}

describe_basic_well_known_type!(bool, BOOL_ID);
