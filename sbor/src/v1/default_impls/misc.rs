use super::super::*;
use crate::rust::string::String;

impl Interpretation for () {
    const INTERPRETATION: u8 = DefaultInterpretations::UNIT;
}

impl Encode for () {
    fn encode_value(&self, encoder: &mut Encoder) {
        encoder.write_product_type_header_u8_length(0);
    }
}

impl Decode for () {
    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        decoder.read_product_type_header_u8_length(0)
    }
}

impl Interpretation for bool {
    const INTERPRETATION: u8 = DefaultInterpretations::BOOLEAN;
}

impl Encode for bool {
    fn encode_value(&self, encoder: &mut Encoder) {
        encoder.write_raw_bytes(
            if *self {
                &[1]
            } else {
                &[0]
            }
        )
    }
}

impl Decode for bool {
    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        let bytes = decoder.read_raw_bytes_fixed_length_array::<1>()?;
        match bytes {
            [0] => Ok(false),
            [1] => Ok(true),
            [other] => Err(DecodeError::InvalidBool(other)),
        }
    }
}

impl Interpretation for String {
    const INTERPRETATION: u8 = DefaultInterpretations::UTF8_STRING;
}

impl Encode for String {
    fn encode_value(&self, encoder: &mut Encoder) {
        encoder.write_raw_bytes(self.as_bytes());
    }
}

impl Decode for String {
    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        let slice = decoder.read_raw_bytes()?;
        String::from_utf8(slice.to_vec())
            .map_err(|_| DecodeError::InvalidUtf8)
    }
}

impl Interpretation for str {
    const INTERPRETATION: u8 = DefaultInterpretations::UTF8_STRING;
}

impl Encode for str {
    fn encode_value(&self, encoder: &mut Encoder) {
        encoder.write_raw_bytes(self.as_bytes());
    }
}
