use super::*;
use crate::rust::string::String;

pub struct EnumDiscriminatorString(String);

impl Interpretation for EnumDiscriminatorString {
    const INTERPRETATION: u8 = DefaultInterpretations::UTF8_STRING_DISCRIMINATOR;
}

impl Encode for EnumDiscriminatorString {
    fn encode_value(&self, encoder: &mut Encoder) {
        Encode::encode_value(&self.0, encoder);
    }
}

impl Decode for EnumDiscriminatorString {
    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        Ok(EnumDiscriminatorString(Decode::decode_value(decoder)?))
    }
}

pub struct EnumValueUnit;

impl Interpretation for EnumValueUnit {
    const INTERPRETATION: u8 = DefaultInterpretations::ENUM_VARIANT_UNIT;
}

impl Encode for EnumValueUnit {
    #[inline]
    fn encode_value(&self, encoder: &mut Encoder) {
        Encode::encode_value(&(), encoder);
    }
}

impl Decode for EnumValueUnit {
    #[inline]
    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        let () = Decode::decode_value(decoder)?;
        Ok(EnumValueUnit)
    }
}