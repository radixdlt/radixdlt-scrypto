use crate::encoder::*;
use crate::type_id::*;

/// A data structure that can be serialized into a byte array using SBOR.
pub trait Encode<X: CustomTypeId, E: Encoder<X>> {
    /// Encodes the SBOR type id of the type to the encoder
    fn encode_type_id(&self, encoder: &mut E) -> Result<(), EncodeError>;

    /// Encodes the SBOR body of the type to the encoder.
    ///
    /// You may want to call `encoder.encode_deeper_body` instead of this method. See
    /// the below section for details.
    ///
    /// ## Direct calls and SBOR Depth
    ///
    /// In order to avoid SBOR depth differentials and disagreement about whether a payload
    /// is valid, typed codec implementations should ensure that the SBOR depth as measured
    /// during the encoding/decoding process agrees with the SborValue codec.
    ///
    /// Each layer of the SborValue counts as one depth.
    ///
    /// If the encoder you're writing is embedding a child type (and is represented as such
    /// in the SborValue type), then you should call `encoder.encode_body` to increment
    /// the SBOR depth tracker.
    ///
    /// You should only call `value.encode_body` directly when the encoding of that type
    /// into an SborValue doesn't increase the SBOR depth in the encoder, that is:
    /// * When the wrapping type is invisible to the SborValue, ie:
    ///   * Smart pointers
    ///   * Transparent wrappers
    /// * Where the use of the inner type is invisible to SborValue, ie:
    ///   * Where the use of `value.encode_body` is coincidental / code re-use
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError>;
}
