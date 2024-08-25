use super::*;
use crate::internal_prelude::*;
use sbor::*;

pub trait TransactionPayload:
    ManifestEncode
    + ManifestDecode
    + ManifestCategorize
    + ManifestSborEnumVariantFor<VersionedTransactionPayload>
where
    Self::OwnedVariant: ManifestDecode,
    for<'a> Self::BorrowedVariant<'a>: ManifestEncode,
{
    type Prepared: TransactionPayloadPreparable<Raw = Self::Raw>;
    type Raw: RawTransactionPayload;

    fn discriminator() -> u8 {
        Self::DISCRIMINATOR
    }

    fn to_raw(&self) -> Result<Self::Raw, EncodeError> {
        Ok(self.to_payload_bytes()?.into())
    }

    fn to_payload_bytes(&self) -> Result<Vec<u8>, EncodeError> {
        manifest_encode(&self.as_encodable_variant())
    }

    fn from_raw(raw: &Self::Raw) -> Result<Self, DecodeError> {
        Self::from_payload_bytes(raw.as_ref())
    }

    fn from_payload_bytes(payload_bytes: &[u8]) -> Result<Self, DecodeError> {
        Ok(Self::from_decoded_variant(manifest_decode(payload_bytes)?))
    }

    fn from_payload_variant(payload_variant: Self::OwnedVariant) -> Self {
        Self::from_decoded_variant(payload_variant)
    }

    fn prepare(&self) -> Result<Self::Prepared, PrepareError> {
        Ok(Self::Prepared::prepare_from_payload(
            &self.to_payload_bytes()?,
        )?)
    }
}

pub trait TransactionPartialEncode: ManifestEncode {
    type Prepared: TransactionPreparableFromValue;

    fn prepare_partial(&self) -> Result<Self::Prepared, PrepareError> {
        Ok(Self::Prepared::prepare_from_payload_slice(
            &manifest_encode(self)?,
        )?)
    }
}

/// Intended for use when the value is encoded without a prefix byte,
/// e.g. when it's under an array.
///
/// Should only decode the value body, NOT read the SBOR value kind.
///
/// NOTE:
/// * The hash should align with the hash from other means.
///   Ideally this means the hash should _not_ include the header byte.
/// * Ideally the summary should not include costing for reading the value kind byte.
pub trait TransactionPreparableFromValueBody: HasSummary + Sized {
    /// Prepares value from a manifest decoder by reading the inner body (without the value kind)
    fn prepare_from_value_body(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError>;

    fn value_kind() -> ManifestValueKind;
}

/// Should read the SBOR value kind, and then the rest of the SBOR value.
///
/// NOTE:
/// * The hash should align with the hash from other means.
///   As of V2, it is suggested that this should _not_ include the header byte.
/// * Ideally the summary should include costing for reading the value kind byte.
pub trait TransactionPreparableFromValue: HasSummary + Sized {
    /// Prepares value from a manifest decoder by reading the full SBOR value
    /// That is - the value kind, and then the value body.
    fn prepare_from_value(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError>;

    /// Only exposed for testing
    fn prepare_from_payload_slice(payload: &[u8]) -> Result<Self, PrepareError> {
        let mut manifest_decoder = ManifestDecoder::new(payload, MANIFEST_SBOR_V1_MAX_DEPTH);
        manifest_decoder.read_and_check_payload_prefix(MANIFEST_SBOR_V1_PAYLOAD_PREFIX)?;
        let mut transaction_decoder = TransactionDecoder::new(manifest_decoder);
        let prepared = Self::prepare_from_value(&mut transaction_decoder)?;
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

pub trait RawTransactionPayload: AsRef<[u8]> + From<Vec<u8>> + Into<Vec<u8>> {
    fn as_slice(&self) -> &[u8] {
        self.as_ref()
    }
}

#[macro_export]
macro_rules! define_raw_transaction_payload {
    ($(#[$docs:meta])* $name: ident) => {
        #[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Clone, Sbor)]
        #[sbor(transparent)]
        $(#[$docs])*
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
