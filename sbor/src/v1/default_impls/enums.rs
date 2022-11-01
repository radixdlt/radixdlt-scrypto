use super::super::*;
use crate::rust::string::String;

impl<T> Interpretation for Option<T> {
    const INTERPRETATION: u8 = DefaultInterpretations::OPTION;
}

pub const OPTION_VARIANT_SOME: u8 = 0x00;
pub const OPTION_VARIANT_NONE: u8 = 0x01;

impl<T: Encode> Encode for Option<T> {
    fn encode_value(&self, encoder: &mut Encoder) {
        match self {
            Some(value) => encoder.write_sum_type_u8_discriminator(OPTION_VARIANT_SOME, value),
            None => encoder.write_sum_type_u8_discriminator(OPTION_VARIANT_NONE, &()),
        }
    }
}

impl<T: Decode> Decode for Option<T> {
    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        let discriminator_type = decoder.read_sum_type_discriminator_header()?;
        Ok(match discriminator_type {
            SumTypeDiscriminator::U8 => {
                let discriminator = decoder.read_sum_type_u8_discriminator()?;
                match discriminator {
                    OPTION_VARIANT_SOME => Some(decoder.decode()?),
                    OPTION_VARIANT_NONE => {
                        decoder.decode::<()>()?;
                        None
                    },
                    _ => Err(DecodeError::InvalidU8Discriminator(discriminator))?
                }
            },
            SumTypeDiscriminator::Any => {
                let discriminator: String = decoder.read_sum_type_any_discriminator()?;
                match discriminator.as_str() {
                    "Some" => Some(decoder.decode()?),
                    "None" => {
                        decoder.decode::<()>()?;
                        None
                    },
                    _ => Err(DecodeError::InvalidStringDiscriminator(discriminator))?
                }
            }
            _ => Err(DecodeError::InvalidDiscriminatorType(discriminator_type))?
        })
    }
}

impl<T, E> Interpretation for Result<T, E> {
    const INTERPRETATION: u8 = DefaultInterpretations::RESULT;
}

pub const RESULT_VARIANT_OK: u8 = 0x00;
pub const RESULT_VARIANT_ERR: u8 = 0x01;

impl<T: Encode, E: Encode> Encode for Result<T, E> {
    fn encode_value(&self, encoder: &mut Encoder) {
        match self {
            Ok(value) => encoder.write_sum_type_u8_discriminator(0, value),
            Err(err) => encoder.write_sum_type_u8_discriminator(1, err),
        }
    }
}

impl<T: Decode, E: Decode> Decode for Result<T, E> {
    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        let discriminator_type = decoder.read_sum_type_discriminator_header()?;
        Ok(match discriminator_type {
            SumTypeDiscriminator::U8 => {
                let discriminator = decoder.read_sum_type_u8_discriminator()?;
                match discriminator {
                    RESULT_VARIANT_OK => Ok(decoder.decode()?),
                    RESULT_VARIANT_ERR => Err(decoder.decode()?),
                    _ => Err(DecodeError::InvalidU8Discriminator(discriminator))?
                }
            },
            SumTypeDiscriminator::Any => {
                let discriminator: String = decoder.read_sum_type_any_discriminator()?;
                match discriminator.as_str() {
                    "Ok" => Ok(decoder.decode()?),
                    "Err" => Err(decoder.decode()?),
                    _ => Err(DecodeError::InvalidStringDiscriminator(discriminator))?
                }
            }
            _ => Err(DecodeError::InvalidDiscriminatorType(discriminator_type))?
        })
    }
}