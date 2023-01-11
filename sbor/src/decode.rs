use crate::decoder::*;
use crate::value_kind::*;

/// A data structure that can be decoded from a byte array using SBOR.
pub trait Decode<X: CustomValueKind, D: Decoder<X>>: Sized {
    /// Decodes the type from the decoder, which should match a preloaded value kind.
    ///
    /// You may want to call `decoder.decode_deeper_body_with_value_kind` instead of this method. See
    /// the below section for details.
    ///
    /// ## Direct calls and SBOR Depth
    ///
    /// In order to avoid SBOR depth differentials and disagreement about whether a payload
    /// is valid, typed codec implementations should ensure that the SBOR depth as measured
    /// during the encoding/decoding process agrees with the SBOR [`Value`][crate::Value] codec.
    ///
    /// Each layer of the SBOR `Value` counts as one depth.
    ///
    /// If the decoder you're writing is embedding a child type (and is represented as such
    /// in the SBOR `Value` type), then you should call `decoder.decode_body_with_value_kind` to increment
    /// the SBOR depth tracker.
    ///
    /// You should only call `T::decode_body_with_value_kind` directly when the decoding of that type
    /// into an SBOR `Value` doesn't increase the SBOR depth in the decoder, that is:
    /// * When the wrapping type is invisible to the SBOR `Value`, ie:
    ///   * Smart pointers
    ///   * Transparent wrappers
    /// * Where the use of the inner type is invisible to SBOR `Value`, ie:
    ///   * Where the use of `T::decode_body_with_value_kind` is coincidental / code re-use
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<X>,
    ) -> Result<Self, DecodeError>;
}
