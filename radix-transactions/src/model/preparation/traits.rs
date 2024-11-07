use super::*;
use crate::internal_prelude::*;
use sbor::*;

pub trait TransactionPayload:
    ManifestEncode
    + ManifestDecode
    + ManifestCategorize
    + for<'a> ManifestSborEnumVariantFor<
        AnyTransaction,
        OwnedVariant: ManifestDecode,
        BorrowedVariant<'a>: ManifestEncode,
    >
{
    type Prepared: PreparedTransaction<Raw = Self::Raw>;
    type Raw: RawTransactionPayload;

    fn discriminator() -> u8 {
        Self::DISCRIMINATOR
    }

    fn to_raw(&self) -> Result<Self::Raw, EncodeError> {
        Ok(manifest_encode(&self.as_encodable_variant())?.into())
    }

    fn from_raw(raw: &Self::Raw) -> Result<Self, DecodeError> {
        Ok(Self::from_decoded_variant(manifest_decode(raw.as_ref())?))
    }

    fn from_payload_variant(payload_variant: Self::OwnedVariant) -> Self {
        Self::from_decoded_variant(payload_variant)
    }

    fn prepare(&self, settings: &PreparationSettings) -> Result<Self::Prepared, PrepareError> {
        Ok(Self::Prepared::prepare(&self.to_raw()?, settings)?)
    }
}

pub trait TransactionPartialPrepare: ManifestEncode {
    type Prepared: TransactionPreparableFromValue;

    fn prepare_partial(
        &self,
        settings: &PreparationSettings,
    ) -> Result<Self::Prepared, PrepareError> {
        let payload = manifest_encode(self).unwrap();
        let mut transaction_decoder = TransactionDecoder::new_partial(&payload, settings)?;
        let prepared = Self::Prepared::prepare_from_value(&mut transaction_decoder)?;
        transaction_decoder.check_complete()?;
        Ok(prepared)
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
    /// Most types when read as a value should have a slightly longer length.
    /// BUT some types (e.g. transaction payloads) must have the same length
    /// regardless, as this length is used for billing the transaction.
    const ADDITIONAL_SUMMARY_LENGTH_AS_VALUE: usize = 1usize;

    /// Prepares the transaction from a transaction decoder by reading the inner body
    /// of the tuple/enum (without the value kind)
    fn prepare_from_value_body(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError>;

    fn value_kind() -> ManifestValueKind;
}

impl<T: TransactionPreparableFromValueBody> TransactionPreparableFromValue for T {
    fn prepare_from_value(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError> {
        decoder.read_and_check_value_kind(Self::value_kind())?;
        let mut prepared = Self::prepare_from_value_body(decoder)?;
        // Add the extra byte to the effective length
        prepared.summary_mut().effective_length = prepared
            .get_summary()
            .effective_length
            .checked_add(Self::ADDITIONAL_SUMMARY_LENGTH_AS_VALUE)
            .ok_or(PrepareError::LengthOverflow)?;
        Ok(prepared)
    }
}

/// Should read the SBOR value kind, and then the rest of the SBOR value.
///
/// NOTE:
/// * In V1, the hash included the value kind byte.
/// * In V2, the hash does _not_ include the value kind byte (which enables the
///   hash to be the same when a Vec child as well as when full values).
///
/// There is a blanket implementation of `TransactionPreparableFromValue` for types
/// which support `TransactionPreparableFromValueBody`, therefore from V2 onwards,
/// this should not be implemented directly.
pub trait TransactionPreparableFromValue: HasSummary + Sized {
    /// Prepares value from a manifest decoder by reading the full SBOR value
    /// That is - the value kind, and then the value body.
    fn prepare_from_value(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError>;
}

/// Transaction payloads are models which are intended to be passed around in their raw form.
///
/// They should have a hash over a payload which starts with the bytes:
/// * `TRANSACTION_HASHABLE_PAYLOAD_PREFIX`
/// * `TransactionDiscriminator::X as u8`
pub trait PreparedTransaction: Sized {
    type Raw: RawTransactionPayload;

    /// Prepares value from a transaction decoder by reading the Enum wrapper
    /// (including its value kind)
    fn prepare_from_transaction_enum(
        decoder: &mut TransactionDecoder,
    ) -> Result<Self, PrepareError>;

    fn prepare(raw: &Self::Raw, settings: &PreparationSettings) -> Result<Self, PrepareError> {
        let payload = raw.as_slice();
        let mut transaction_decoder = TransactionDecoder::new_transaction(
            payload,
            <Self::Raw as RawTransactionPayload>::KIND,
            settings,
        )?;
        let prepared = Self::prepare_from_transaction_enum(&mut transaction_decoder)?;
        transaction_decoder.check_complete()?;
        Ok(prepared)
    }
}

macro_rules! define_transaction_payload {
    (
        $transaction:ident,
        $raw:ty,
        $prepared:ident {
            $($field_name:ident: $field_type:ty,)*
        },
        $discriminator:expr,
    ) => {
        #[derive(Debug, Clone, Eq, PartialEq)]
        pub struct $prepared {
            $(pub $field_name: $field_type,)*
            pub summary: Summary,
        }

        impl TransactionPayload for $transaction {
            type Prepared = $prepared;
            type Raw = $raw;
        }

        impl HasSummary for $prepared {
            fn get_summary(&self) -> &Summary {
                &self.summary
            }

            fn summary_mut(&mut self) -> &mut Summary {
                &mut self.summary
            }
        }

        impl PreparedTransaction for $prepared {
            type Raw = $raw;

            fn prepare_from_transaction_enum(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError> {
                // When embedded as full payload, it's SBOR encoded as an enum
                let (($($field_name,)*), summary) = ConcatenatedDigest::prepare_transaction_payload(
                    decoder,
                    $discriminator,
                    ExpectedHeaderKind::EnumWithValueKind,
                )?;
                Ok(Self {
                    $($field_name,)*
                    summary,
                })
            }
        }

        impl TransactionPreparableFromValueBody for $prepared {
            // Ensure that all manners of preparing the transaction give an
            // equal effective length, so they can all be turned into an executable
            // and have the same billed length.
            const ADDITIONAL_SUMMARY_LENGTH_AS_VALUE: usize = 0;

            fn prepare_from_value_body(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError> {
                // When embedded as an child body, it's SBOR encoded as a struct body (without value kind)
                let (($($field_name,)*), summary) =
                    ConcatenatedDigest::prepare_transaction_payload(
                        decoder,
                        $discriminator,
                        ExpectedHeaderKind::TupleNoValueKind,
                    )?;
                Ok(Self {
                    $($field_name,)*
                    summary,
                })
            }

            fn value_kind() -> ManifestValueKind {
                ManifestValueKind::Tuple
            }
        }
    };
}

pub(crate) use define_transaction_payload;

pub trait RawTransactionPayload: AsRef<[u8]> + From<Vec<u8>> + Into<Vec<u8>> {
    const KIND: TransactionPayloadKind;

    fn as_slice(&self) -> &[u8] {
        self.as_ref()
    }
}

pub trait ValidatedTransactionPayload: IntoExecutable {}

#[derive(Debug, Copy, Clone)]
pub enum TransactionPayloadKind {
    CompleteUserTransaction,
    LedgerTransaction,
    Other,
}

#[macro_export]
macro_rules! define_raw_transaction_payload {
    ($(#[$docs:meta])* $name:ident, $kind:expr) => {
        #[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Clone, Sbor)]
        #[sbor(transparent)]
        $(#[$docs])*
        pub struct $name(Vec<u8>);

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

        impl RawTransactionPayload for $name {
            const KIND: TransactionPayloadKind = $kind;
        }

        impl $name {
            pub fn as_slice(&self) -> &[u8] {
                self.0.as_slice()
            }

            pub fn len(&self) -> usize {
                self.as_slice().len()
            }

            pub fn to_vec(self) -> Vec<u8> {
                self.0
            }

            pub fn from_vec(vec: Vec<u8>) -> Self {
                Self(vec)
            }

            pub fn from_slice(slice: impl AsRef<[u8]>) -> Self {
                Self(slice.as_ref().into())
            }

            pub fn to_hex(&self) -> String {
                hex::encode(self.as_slice())
            }

            pub fn from_hex(hex: impl AsRef<[u8]>) -> Result<Self, hex::FromHexError> {
                Ok(Self(hex::decode(hex)?))
            }
        }
    };
}
