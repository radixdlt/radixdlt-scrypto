use sbor::v1::DefaultInterpretations;
use super::decoder::{Decoder, DecodeError};
use super::encoder::Encoder;

/// Provides the interpretation of the payload.
///
/// Most types/impls will have a fixed interpretation, and can just set the associated const INTERPRETATION.
///
/// Some types/impls will have a dynamic interpration, or can support decoding from multiple interpretations,
/// and can override the get_interpretation / check_interpretation methods.
pub trait Interpretation {
    /// The const INTERPRETATION of the type/impl, or can be set to 0 = DefaultInterpretations::NOT_FIXED
    /// which denotes that the interepretation of the type can be multiple values.
    const INTERPRETATION: u8;

    /// This should be false for all types T except those where their Vec<T> should be turned
    /// into RawBytes, via unsafe direct pointer access. This is only valid for u8/i8 types.
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
    /// Encodes the value (should not encode the interpretation)
    fn encode_value(&self, encoder: &mut Encoder);
}

pub trait Decode: Interpretation + Sized {
    /// Decodes the value (the interpretation has already been decoded/checked)
    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError>;
}
