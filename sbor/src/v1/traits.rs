use sbor::v1::DefaultInterpretations;
use super::decoder::{Decoder, DecodeError};
use super::encoder::Encoder;

// Traits permitting dynamic handling of interpretation (for eg use by sbor::Value)
// These are the traits that the encoder/decoder actually require

pub trait Interpretation {
    const INTERPRETATION: u8;
    const IS_BYTE: bool = false;

    #[inline]
    fn get_interpretation(&self) -> u8 {
        if Self::INTERPRETATION == DefaultInterpretations::NOT_FIXED {
            todo!("The get_interpretation method must be overridden if the interpretation is not fixed!")
        }
        Self::INTERPRETATION
    }

    #[inline]
    fn check_interpretation(actual: u8) -> Result<(), DecodeError> {
        if Self::INTERPRETATION == DefaultInterpretations::NOT_FIXED {
            todo!("The check_interpretation method must be overridden if the interpretation is not fixed!")
        }
        let expected = Self::INTERPRETATION;
        if expected == actual {
            Ok(())
        } else {
            Err(DecodeError::InvalidInterpretation { expected, actual })
        }
    }
}

pub trait Encode: Interpretation {
    fn encode_value(&self, encoder: &mut Encoder);
}

pub trait Decode: Interpretation + Sized {
    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError>;
}
