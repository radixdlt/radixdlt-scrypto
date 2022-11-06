use super::*;
use crate::rust::string::String;

pub struct EnumDiscriminatorString(String);

impl Interpretation for EnumDiscriminatorString {
    const INTERPRETATION: u8 = DefaultInterpretations::UTF8_STRING_DISCRIMINATOR;
}

impl <E: Encoder> Encode<E> for EnumDiscriminatorString {
    fn encode_value(&self, encoder: &mut E) -> Result<(), EncodeError> {
        Encode::encode_value(&self.0, encoder)
    }
}

impl <D: Decoder> Decode<D> for EnumDiscriminatorString {
    fn decode_value(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(EnumDiscriminatorString(Decode::decode_value(decoder)?))
    }
}

pub struct EnumValueUnit;

impl Interpretation for EnumValueUnit {
    const INTERPRETATION: u8 = DefaultInterpretations::ENUM_VARIANT_UNIT;
}

impl <E: Encoder> Encode<E> for EnumValueUnit {
    #[inline]
    fn encode_value(&self, encoder: &mut E) -> Result<(), EncodeError> {
        Encode::encode_value(&(), encoder)
    }
}

impl <D: Decoder> Decode<D> for EnumValueUnit {
    #[inline]
    fn decode_value(decoder: &mut D) -> Result<Self, DecodeError> {
        let () = Decode::decode_value(decoder)?;
        Ok(EnumValueUnit)
    }
}