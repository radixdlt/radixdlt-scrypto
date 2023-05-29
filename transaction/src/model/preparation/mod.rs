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
    type Raw: RawTransactionPayload;

    /// Prepares value from a manifest decoder by reading the full SBOR value body (with the value kind)
    fn prepare_for_payload(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError>;

    fn prepare_from_raw(raw: &Self::Raw) -> Result<Self, PrepareError> {
        Self::prepare_from_payload(raw.as_ref())
    }

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

pub trait RawTransactionPayload: AsRef<[u8]> + From<Vec<u8>> + Into<Vec<u8>> {}

#[macro_export]
macro_rules! define_raw_transaction_payload {
    ($name: ident) => {
        #[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Clone, Sbor)]
        #[sbor(transparent)]
        pub struct $name(pub Vec<u8>);

        impl AsRef<[u8]> for $name {
            fn as_ref(&self) -> &[u8] {
                self.0.as_ref()
            }
        }

        impl From<Vec<u8>> for $name {
            fn from(value: Vec<u8>) -> Self {
                Self(value)
            }
        }

        impl From<$name> for Vec<u8> {
            fn from(value: $name) -> Self {
                value.0
            }
        }

        impl RawTransactionPayload for $name {}
    };
}
