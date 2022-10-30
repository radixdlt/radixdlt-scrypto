use super::*;

pub struct EnumDiscriminatorString(String);

impl ConstInterpretation for EnumDiscriminatorString {
    const INTERPRETATION: u8 = DefaultInterpretations::UTF8_STRING_DISCRIMINATOR;
}

impl Encode for EnumDiscriminatorString {
    fn encode_value(&self, encoder: &mut Encoder) {
        self.0.encode_value(encoder);
    }
}

impl Decode for EnumDiscriminatorString {
    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        Ok(EnumDiscriminatorString(String::decode_value(decoder)?))
    }
}