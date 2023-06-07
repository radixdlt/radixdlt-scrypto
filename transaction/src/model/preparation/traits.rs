use super::*;
use crate::internal_prelude::*;
use sbor::*;

pub trait TransactionPayload:
    ManifestEncode + ManifestDecode + ManifestCategorize + ManifestSborTuple
{
    // Note - really we just want to define Self::DISCRIMINATOR and use FixedEnumVariant<{ Self::DISCRIMINATOR }, X>,
    // but that causes an issue because "type parameters may not be used in const expressions"
    // See: https://github.com/rust-lang/rust/issues/76560
    // Instead we use this helper-trait IsFixedEnumVariant which hides the DISCRIMINATOR
    type Versioned: ManifestDecode + IsFixedEnumVariant<ManifestCustomValueKind, Self>;
    type Prepared: TransactionPayloadPreparable<Raw = Self::Raw>;
    type Raw: RawTransactionPayload;

    fn to_raw(&self) -> Result<Self::Raw, EncodeError> {
        Ok(self.to_payload_bytes()?.into())
    }

    fn to_payload_bytes(&self) -> Result<Vec<u8>, EncodeError> {
        manifest_encode(&Self::Versioned::for_encoding(self))
    }

    fn from_raw(raw: &Self::Raw) -> Result<Self, DecodeError> {
        Self::from_payload_bytes(raw.as_ref())
    }

    fn from_payload_bytes(payload_bytes: &[u8]) -> Result<Self, DecodeError> {
        Ok(manifest_decode::<Self::Versioned>(payload_bytes)?.into_fields())
    }

    fn prepare(&self) -> Result<Self::Prepared, PrepareError> {
        Ok(Self::Prepared::prepare_from_payload(
            &self.to_payload_bytes()?,
        )?)
    }
}

pub trait TransactionPartialEncode: ManifestEncode {
    type Prepared: TransactionFullChildPreparable;

    fn prepare_partial(&self) -> Result<Self::Prepared, PrepareError> {
        Ok(Self::Prepared::prepare_as_full_body_child_from_payload(
            &manifest_encode(self)?,
        )?)
    }
}

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
