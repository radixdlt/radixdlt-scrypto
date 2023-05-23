use super::*;
use crate::internal_prelude::*;
use sbor::*;

//=================================================================================
// See REP-82 for justification behind this preparation strategy.
//
// Roughly:
// * Preparation: decoding + hash calculation
// * Validation: further checks + signature verification
//=================================================================================

mod decoder;
mod references;
mod summarized_composite;
mod summarized_raw;
mod summary;
pub use decoder::*;
pub use references::*;
pub use summarized_composite::*;
pub use summarized_raw::*;
pub use summary::*;

pub trait TransactionChildBodyPreparable: HasSummary + Sized {
    /// Prepares value from a manifest decoder by reading the inner body (without the value kind)
    fn prepare_as_inner_body_child(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError>;

    fn value_kind() -> ManifestValueKind;
}

pub trait TransactionFullChildPreparable: HasSummary + Sized {
    /// Prepares value from a manifest decoder by reading the full SBOR value body (with the value kind)
    fn prepare_as_full_body_child(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError>;

    /// Only exposed for testing
    fn prepare_as_full_body_child_from_payload(payload: &[u8]) -> Result<Self, PrepareError> {
        let mut manifest_decoder = ManifestDecoder::new(payload, MANIFEST_SBOR_V1_MAX_DEPTH);
        manifest_decoder.read_and_check_payload_prefix(MANIFEST_SBOR_V1_PAYLOAD_PREFIX)?;
        let mut transaction_decoder = TransactionDecoder::new(manifest_decoder);
        let prepared = Self::prepare_as_full_body_child(&mut transaction_decoder)?;
        transaction_decoder.destructure().check_end()?;
        Ok(prepared)
    }
}

pub trait TransactionPayloadPreparable: HasSummary + Sized {
    /// Prepares value from a manifest decoder by reading the full SBOR value body (with the value kind)
    fn prepare_for_payload(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError>;

    /// Prepares from a full payload
    fn prepare_from_payload(payload: &[u8]) -> Result<Self, PrepareError> {
        let mut manifest_decoder = ManifestDecoder::new(payload, MANIFEST_SBOR_V1_MAX_DEPTH);
        manifest_decoder.read_and_check_payload_prefix(MANIFEST_SBOR_V1_PAYLOAD_PREFIX)?;
        let mut transaction_decoder = TransactionDecoder::new(manifest_decoder);
        let prepared = Self::prepare_for_payload(&mut transaction_decoder)?;
        transaction_decoder.destructure().check_end()?;
        Ok(prepared)
    }
}
