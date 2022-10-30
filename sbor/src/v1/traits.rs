use super::decoder::{Decoder, DecodeError};
use super::encoder::Encoder;

// Traits that most types will implement
// These traits have a const interpretation

pub trait ConstInterpretation {
    const INTERPRETATION: u8;
}

pub trait Encode: ConstInterpretation {
    /// Encode the value without the interpretation byte
    fn encode_value(&self, encoder: &mut Encoder);
}

pub trait Decode: ConstInterpretation + Sized {
    /// Decode the value without the interpretation byte
    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError>;
}

// Traits permitting dynamic handling of interpretation (for eg use by sbor::Value)
// These are the traits that the encoder/decoder actually require

pub trait Encodable {
    fn interpretation(&self) -> u8;
    fn encode_value_to(&self, encoder: &mut Encoder);
}

impl<T: Encode> Encodable for T {
    #[inline]
    fn interpretation(&self) -> u8 {
        T::INTERPRETATION
    }

    #[inline]
    fn encode_value_to(&self, encoder: &mut Encoder) {
        self.encode_value(encoder)
    }
}

pub trait Decodable: Sized {
    fn check_interpretation(interpretation: u8) -> Result<(), DecodeError>;
    fn decode_value_from(decoder: &mut Decoder) -> Result<Self, DecodeError>;
}

impl<T: Decode> Decodable for T {
    #[inline]
    fn check_interpretation(interpretation: u8) -> Result<(), DecodeError> {
        if T::INTERPRETATION == interpretation {
            Ok(())
        } else {
            Err(DecodeError::InvalidInterpretation { expected: T::INTERPRETATION, actual: interpretation })
        }
    }

    #[inline]
    fn decode_value_from(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        T::decode_value(decoder)
    }
}
