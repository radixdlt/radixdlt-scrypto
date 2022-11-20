use crate::decoder::*;
use crate::type_id::*;

/// A data structure that can be decoded from a byte array using SBOR.
pub trait Decode<X: CustomTypeId, D: Decoder<X>>: Sized {
    /// Decodes the type from the decoder, using a preloaded type id.
    ///
    /// You likely want to call `decoder.decode_body_with_type_id` instead of this method. See
    /// the below section for details.
    ///
    /// ## Direct calls and SBOR Depth
    ///
    /// In order to avoid SBOR depth differentials and disagreement about whether a payload
    /// is valid, typed codec implementations should ensure that the SBOR depth as measured
    /// during the encoding/decoding process agrees with the SborValue codec.
    ///
    /// If the decoder you're writing is embedding a child type (and is represented as such
    /// in the SborValue type), then you should call `decoder.decode_body_with_type_id` to increment
    /// the SBOR depth tracker.
    ///
    /// You should only call `T::decode_body_with_type_id` directly when the decoding of that type
    /// into an SborValue doesn't increase the SBOR depth in the decoder, that is:
    /// * When the wrapping type is invisible to the SborValue, ie:
    ///   * Smart pointers
    ///   * Transparent wrappers
    /// * Where the use of the inner type is invisible to SborValue, ie:
    ///   * Where the use of `T::decode_body_with_type_id` is coincidental / code re-use
    fn decode_body_with_type_id(
        decoder: &mut D,
        type_id: SborTypeId<X>,
    ) -> Result<Self, DecodeError>;
}
