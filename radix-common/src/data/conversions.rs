use crate::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConversionError {
    NoDirectEquivalence,
    DecodeError(DecodeError),
    EncodeError(EncodeError),
}

type Result<T> = core::result::Result<T, ConversionError>;

pub fn scrypto_value_to_manifest_value(scrypto_value: ScryptoValue) -> Result<ManifestValue> {
    match scrypto_value {
        ScryptoValue::Bool { value } => Ok(ManifestValue::Bool { value }),
        ScryptoValue::I8 { value } => Ok(ManifestValue::I8 { value }),
        ScryptoValue::I16 { value } => Ok(ManifestValue::I16 { value }),
        ScryptoValue::I32 { value } => Ok(ManifestValue::I32 { value }),
        ScryptoValue::I64 { value } => Ok(ManifestValue::I64 { value }),
        ScryptoValue::I128 { value } => Ok(ManifestValue::I128 { value }),
        ScryptoValue::U8 { value } => Ok(ManifestValue::U8 { value }),
        ScryptoValue::U16 { value } => Ok(ManifestValue::U16 { value }),
        ScryptoValue::U32 { value } => Ok(ManifestValue::U32 { value }),
        ScryptoValue::U64 { value } => Ok(ManifestValue::U64 { value }),
        ScryptoValue::U128 { value } => Ok(ManifestValue::U128 { value }),
        ScryptoValue::String { value } => Ok(ManifestValue::String { value }),
        ScryptoValue::Enum {
            discriminator,
            fields,
        } => fields
            .into_iter()
            .map(scrypto_value_to_manifest_value)
            .collect::<Result<_>>()
            .map(|fields| ManifestValue::Enum {
                discriminator,
                fields,
            }),
        ScryptoValue::Array {
            element_value_kind,
            elements,
        } => {
            let element_value_kind = scrypto_value_kind_to_manifest_value_kind(element_value_kind)?;
            let elements = elements
                .into_iter()
                .map(scrypto_value_to_manifest_value)
                .collect::<Result<Vec<_>>>()?;
            Ok(ManifestValue::Array {
                element_value_kind,
                elements,
            })
        }
        ScryptoValue::Tuple { fields } => fields
            .into_iter()
            .map(scrypto_value_to_manifest_value)
            .collect::<Result<_>>()
            .map(|fields| ManifestValue::Tuple { fields }),
        ScryptoValue::Map {
            key_value_kind,
            value_value_kind,
            entries,
        } => {
            let key_value_kind = scrypto_value_kind_to_manifest_value_kind(key_value_kind)?;
            let value_value_kind = scrypto_value_kind_to_manifest_value_kind(value_value_kind)?;
            let entries = entries
                .into_iter()
                .map(|(key, value)| -> Result<(ManifestValue, ManifestValue)> {
                    let key = scrypto_value_to_manifest_value(key)?;
                    let value = scrypto_value_to_manifest_value(value)?;
                    Ok((key, value))
                })
                .collect::<Result<Vec<_>>>()?;
            Ok(ManifestValue::Map {
                key_value_kind,
                value_value_kind,
                entries,
            })
        }
        ScryptoValue::Custom {
            value: ScryptoCustomValue::Decimal(value),
        } => Ok(ManifestValue::Custom {
            value: ManifestCustomValue::Decimal(from_decimal(&value)),
        }),
        ScryptoValue::Custom {
            value: ScryptoCustomValue::PreciseDecimal(value),
        } => Ok(ManifestValue::Custom {
            value: ManifestCustomValue::PreciseDecimal(from_precise_decimal(&value)),
        }),
        ScryptoValue::Custom {
            value: ScryptoCustomValue::Reference(value),
        } => Ok(ManifestValue::Custom {
            value: ManifestCustomValue::Address(ManifestAddress::Static(value.0)),
        }),
        ScryptoValue::Custom {
            value: ScryptoCustomValue::NonFungibleLocalId(value),
        } => Ok(ManifestValue::Custom {
            value: ManifestCustomValue::NonFungibleLocalId(from_non_fungible_local_id(value)),
        }),
        ScryptoValue::Custom {
            value: ScryptoCustomValue::Own(..),
        } => Err(ConversionError::NoDirectEquivalence),
    }
}

pub fn scrypto_value_kind_to_manifest_value_kind(
    value_kind: ScryptoValueKind,
) -> Result<ManifestValueKind> {
    match value_kind {
        ScryptoValueKind::Bool => Ok(ManifestValueKind::Bool),
        ScryptoValueKind::I8 => Ok(ManifestValueKind::I8),
        ScryptoValueKind::I16 => Ok(ManifestValueKind::I16),
        ScryptoValueKind::I32 => Ok(ManifestValueKind::I32),
        ScryptoValueKind::I64 => Ok(ManifestValueKind::I64),
        ScryptoValueKind::I128 => Ok(ManifestValueKind::I128),
        ScryptoValueKind::U8 => Ok(ManifestValueKind::U8),
        ScryptoValueKind::U16 => Ok(ManifestValueKind::U16),
        ScryptoValueKind::U32 => Ok(ManifestValueKind::U32),
        ScryptoValueKind::U64 => Ok(ManifestValueKind::U64),
        ScryptoValueKind::U128 => Ok(ManifestValueKind::U128),
        ScryptoValueKind::String => Ok(ManifestValueKind::String),
        ScryptoValueKind::Enum => Ok(ManifestValueKind::Enum),
        ScryptoValueKind::Array => Ok(ManifestValueKind::Array),
        ScryptoValueKind::Tuple => Ok(ManifestValueKind::Tuple),
        ScryptoValueKind::Map => Ok(ManifestValueKind::Map),
        ScryptoValueKind::Custom(ScryptoCustomValueKind::Reference) => {
            Ok(ManifestValueKind::Custom(ManifestCustomValueKind::Address))
        }
        ScryptoValueKind::Custom(ScryptoCustomValueKind::Own) => {
            Err(ConversionError::NoDirectEquivalence)
        }
        ScryptoValueKind::Custom(ScryptoCustomValueKind::Decimal) => {
            Ok(ManifestValueKind::Custom(ManifestCustomValueKind::Decimal))
        }
        ScryptoValueKind::Custom(ScryptoCustomValueKind::PreciseDecimal) => Ok(
            ManifestValueKind::Custom(ManifestCustomValueKind::PreciseDecimal),
        ),
        ScryptoValueKind::Custom(ScryptoCustomValueKind::NonFungibleLocalId) => Ok(
            ManifestValueKind::Custom(ManifestCustomValueKind::NonFungibleLocalId),
        ),
    }
}

pub fn manifest_value_to_scrypto_value(manifest_value: ManifestValue) -> Result<ScryptoValue> {
    match manifest_value {
        ManifestValue::Bool { value } => Ok(ScryptoValue::Bool { value }),
        ManifestValue::I8 { value } => Ok(ScryptoValue::I8 { value }),
        ManifestValue::I16 { value } => Ok(ScryptoValue::I16 { value }),
        ManifestValue::I32 { value } => Ok(ScryptoValue::I32 { value }),
        ManifestValue::I64 { value } => Ok(ScryptoValue::I64 { value }),
        ManifestValue::I128 { value } => Ok(ScryptoValue::I128 { value }),
        ManifestValue::U8 { value } => Ok(ScryptoValue::U8 { value }),
        ManifestValue::U16 { value } => Ok(ScryptoValue::U16 { value }),
        ManifestValue::U32 { value } => Ok(ScryptoValue::U32 { value }),
        ManifestValue::U64 { value } => Ok(ScryptoValue::U64 { value }),
        ManifestValue::U128 { value } => Ok(ScryptoValue::U128 { value }),
        ManifestValue::String { value } => Ok(ScryptoValue::String { value }),
        ManifestValue::Enum {
            discriminator,
            fields,
        } => fields
            .into_iter()
            .map(manifest_value_to_scrypto_value)
            .collect::<Result<_>>()
            .map(|fields| ScryptoValue::Enum {
                discriminator,
                fields,
            }),
        ManifestValue::Array {
            element_value_kind,
            elements,
        } => {
            let element_value_kind = manifest_value_kind_to_scrypto_value_kind(element_value_kind)?;
            let elements = elements
                .into_iter()
                .map(manifest_value_to_scrypto_value)
                .collect::<Result<Vec<_>>>()?;
            Ok(ScryptoValue::Array {
                element_value_kind,
                elements,
            })
        }
        ManifestValue::Tuple { fields } => fields
            .into_iter()
            .map(manifest_value_to_scrypto_value)
            .collect::<Result<_>>()
            .map(|fields| ScryptoValue::Tuple { fields }),
        ManifestValue::Map {
            key_value_kind,
            value_value_kind,
            entries,
        } => {
            let key_value_kind = manifest_value_kind_to_scrypto_value_kind(key_value_kind)?;
            let value_value_kind = manifest_value_kind_to_scrypto_value_kind(value_value_kind)?;
            let entries = entries
                .into_iter()
                .map(|(key, value)| -> Result<(ScryptoValue, ScryptoValue)> {
                    let key = manifest_value_to_scrypto_value(key)?;
                    let value = manifest_value_to_scrypto_value(value)?;
                    Ok((key, value))
                })
                .collect::<Result<Vec<_>>>()?;
            Ok(ScryptoValue::Map {
                key_value_kind,
                value_value_kind,
                entries,
            })
        }
        ManifestValue::Custom {
            value: ManifestCustomValue::Address(ManifestAddress::Static(value)),
        } => Ok(ScryptoValue::Custom {
            value: ScryptoCustomValue::Reference(Reference(value)),
        }),
        ManifestValue::Custom {
            value: ManifestCustomValue::Decimal(value),
        } => Ok(ScryptoValue::Custom {
            value: ScryptoCustomValue::Decimal(to_decimal(&value)),
        }),
        ManifestValue::Custom {
            value: ManifestCustomValue::PreciseDecimal(value),
        } => Ok(ScryptoValue::Custom {
            value: ScryptoCustomValue::PreciseDecimal(to_precise_decimal(&value)),
        }),
        ManifestValue::Custom {
            value: ManifestCustomValue::NonFungibleLocalId(value),
        } => Ok(ScryptoValue::Custom {
            value: ScryptoCustomValue::NonFungibleLocalId(to_non_fungible_local_id(value)),
        }),
        ManifestValue::Custom {
            value:
                ManifestCustomValue::Bucket(..)
                | ManifestCustomValue::Proof(..)
                | ManifestCustomValue::AddressReservation(..)
                | ManifestCustomValue::Expression(..)
                | ManifestCustomValue::Blob(..)
                | ManifestCustomValue::Address(ManifestAddress::Named(..)),
        } => Err(ConversionError::NoDirectEquivalence),
    }
}

pub fn manifest_value_kind_to_scrypto_value_kind(
    value_kind: ManifestValueKind,
) -> Result<ScryptoValueKind> {
    match value_kind {
        ManifestValueKind::Bool => Ok(ScryptoValueKind::Bool),
        ManifestValueKind::I8 => Ok(ScryptoValueKind::I8),
        ManifestValueKind::I16 => Ok(ScryptoValueKind::I16),
        ManifestValueKind::I32 => Ok(ScryptoValueKind::I32),
        ManifestValueKind::I64 => Ok(ScryptoValueKind::I64),
        ManifestValueKind::I128 => Ok(ScryptoValueKind::I128),
        ManifestValueKind::U8 => Ok(ScryptoValueKind::U8),
        ManifestValueKind::U16 => Ok(ScryptoValueKind::U16),
        ManifestValueKind::U32 => Ok(ScryptoValueKind::U32),
        ManifestValueKind::U64 => Ok(ScryptoValueKind::U64),
        ManifestValueKind::U128 => Ok(ScryptoValueKind::U128),
        ManifestValueKind::String => Ok(ScryptoValueKind::String),
        ManifestValueKind::Enum => Ok(ScryptoValueKind::Enum),
        ManifestValueKind::Array => Ok(ScryptoValueKind::Array),
        ManifestValueKind::Tuple => Ok(ScryptoValueKind::Tuple),
        ManifestValueKind::Map => Ok(ScryptoValueKind::Map),
        ManifestValueKind::Custom(ManifestCustomValueKind::Address) => {
            Ok(ScryptoValueKind::Custom(ScryptoCustomValueKind::Reference))
        }
        ManifestValueKind::Custom(ManifestCustomValueKind::Decimal) => {
            Ok(ScryptoValueKind::Custom(ScryptoCustomValueKind::Decimal))
        }
        ManifestValueKind::Custom(ManifestCustomValueKind::PreciseDecimal) => Ok(
            ScryptoValueKind::Custom(ScryptoCustomValueKind::PreciseDecimal),
        ),
        ManifestValueKind::Custom(ManifestCustomValueKind::NonFungibleLocalId) => Ok(
            ScryptoValueKind::Custom(ScryptoCustomValueKind::NonFungibleLocalId),
        ),
        ManifestValueKind::Custom(
            ManifestCustomValueKind::Bucket
            | ManifestCustomValueKind::Proof
            | ManifestCustomValueKind::AddressReservation
            | ManifestCustomValueKind::Expression
            | ManifestCustomValueKind::Blob,
        ) => Err(ConversionError::NoDirectEquivalence),
    }
}
