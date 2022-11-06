use sbor::v1::DefaultInterpretations;
use super::EncodeError;
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
        check_matching_interpretation(Self::INTERPRETATION, actual)
    }
}

pub fn check_matching_interpretation(expected: u8, actual: u8) -> Result<(), DecodeError> {
    if expected == actual {
        Ok(())
    } else {
        Err(DecodeError::InvalidInterpretation { expected, actual })
    }
}

/// The trait representing that the value can be encoded with SBOR.
/// 
/// If implementing Encode, you should also implement Interpretation.
///
/// If using Encode as a type constraint, you have two options:
/// * If the type constraint is to implement Encode, use Encode + Interpretation (to match your Intepretation bound)
/// * If the type constraint is for a method, choose Encode + ?Sized - this enables you to take trait objects, slices etc
pub trait Encode<E: Encoder>: XXInternalHasInterpretation {
    /// Encodes the value (should not encode the interpretation)
    fn encode_value(&self, encoder: &mut E) -> Result<(), EncodeError>;
}

/// The trait representing a decode-target for an SBOR payload
pub trait Decode: Interpretation + Sized {
    /// Decodes the value (the interpretation has already been decoded/checked)
    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError>;
}

/// This trait is not intended to be implemented directly - instead, implement the
/// Encode and Decode traits.
pub trait Codec<E: Encoder>: Encode<E> + Decode {}
impl<T: Encode<E> + Decode, E: Encoder> Codec<E> for T {}

/// Important: This trait is never intended to be implemented directly - instead, implement
/// the `Interpretation` trait.
/// 
/// The HasInterpretation trait creates some slight-redirection, so that Encode does not
/// rely explicitly on the Interpretation trait. This ensures that Encode has no direct
/// associated types (such as the various constants and methods on Intepretation which
/// don't take &self), and so allows for it to be boxed in a trait object.
/// 
/// This means traits doing a blanket impl on T: Encode likely need a T: Encode + Implementation
/// bound to match their T: Implementation bound of their impl of their Implementation trait.
/// 
/// NOTE: It might be compelling to create a ChecksInterpretation trait, and make
/// Decode: ChecksInterpretation -- and having blanket impls only implement these traits and
/// not the Interpretation trait. This doesn't work though - because the blanket impls potentially
/// clash with downstream crates impls for fundamental types such as Box<T>.
pub trait XXInternalHasInterpretation {
    fn get_interpretation(&self) -> u8;
}

impl<T: Interpretation + ?Sized> XXInternalHasInterpretation for T {
    #[inline]
    fn get_interpretation(&self) -> u8 {
        T::get_interpretation(&self)
    }
}
