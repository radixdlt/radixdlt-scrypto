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

    /// In producion, you should use a TransactionValidator instead, which has
    /// access to [`PreparationSettings`].
    fn prepare_partial_with_latest_settings(&self) -> Result<Self::Prepared, PrepareError> {
        self.prepare_partial(PreparationSettings::latest_ref())
    }

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
    /// Prepares value from a manifest decoder by reading the inner body (without the value kind)
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
            .checked_add(1)
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
pub trait TransactionPayloadPreparable: Sized {
    type Raw: RawTransactionPayload;

    /// Prepares value from a manifest decoder by reading the full SBOR value body (with the value kind)
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

macro_rules! transaction_payload_v2 {
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

        impl TransactionPayloadPreparable for $prepared {
            type Raw = $raw;

            fn prepare_from_transaction_enum(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError> {
                // When embedded as full payload, it's SBOR encoded as an enum
                let (($($field_name,)*), summary) = ConcatenatedDigest::prepare_from_transaction_payload_enum(
                    decoder,
                    $discriminator,
                )?;
                Ok(Self {
                    $($field_name,)*
                    summary,
                })
            }
        }

        impl TransactionPreparableFromValueBody for $prepared {
            fn prepare_from_value_body(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError> {
                // When embedded as an child body, it's SBOR encoded as a struct body (without value kind)
                let (($($field_name,)*), summary) =
                    ConcatenatedDigest::prepare_from_transaction_child_struct_body(
                        decoder,
                        $discriminator,
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

pub(crate) use transaction_payload_v2;

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

        impl RawTransactionPayload for $name {
            const KIND: TransactionPayloadKind = $kind;
        }
    };
}
