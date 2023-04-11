use crate::data::*;
use radix_engine_interface::data::manifest::model::{
    ManifestBlobRef, ManifestBucket, ManifestExpression, ManifestProof,
};
use radix_engine_interface::data::manifest::{
    ManifestCustomValue, ManifestCustomValueKind, ManifestValue, ManifestValueKind,
};
use radix_engine_interface::data::scrypto::model::{Own, Reference};
use radix_engine_interface::data::scrypto::{
    ScryptoCustomValue, ScryptoCustomValueKind, ScryptoValue, ScryptoValueKind,
};
use sbor::rust::vec::Vec;

pub trait TransformHandler<E> {
    fn replace_bucket(&mut self, b: ManifestBucket) -> Result<Own, E>;
    fn replace_proof(&mut self, p: ManifestProof) -> Result<Own, E>;
    fn replace_expression(&mut self, e: ManifestExpression) -> Result<Vec<Own>, E>;
    fn replace_blob(&mut self, b: ManifestBlobRef) -> Result<Vec<u8>, E>;
}

pub fn transform<T: TransformHandler<E>, E>(
    value: ManifestValue,
    handler: &mut T,
) -> Result<ScryptoValue, E> {
    match value {
        sbor::Value::Bool { value } => Ok(ScryptoValue::Bool { value }),
        sbor::Value::I8 { value } => Ok(ScryptoValue::I8 { value }),
        sbor::Value::I16 { value } => Ok(ScryptoValue::I16 { value }),
        sbor::Value::I32 { value } => Ok(ScryptoValue::I32 { value }),
        sbor::Value::I64 { value } => Ok(ScryptoValue::I64 { value }),
        sbor::Value::I128 { value } => Ok(ScryptoValue::I128 { value }),
        sbor::Value::U8 { value } => Ok(ScryptoValue::U8 { value }),
        sbor::Value::U16 { value } => Ok(ScryptoValue::U16 { value }),
        sbor::Value::U32 { value } => Ok(ScryptoValue::U32 { value }),
        sbor::Value::U64 { value } => Ok(ScryptoValue::U64 { value }),
        sbor::Value::U128 { value } => Ok(ScryptoValue::U128 { value }),
        sbor::Value::String { value } => Ok(ScryptoValue::String { value }),
        sbor::Value::Enum {
            discriminator,
            fields,
        } => {
            let mut transformed_fields = Vec::new();
            for field in fields {
                transformed_fields.push(transform(field, handler)?);
            }
            Ok(ScryptoValue::Enum {
                discriminator,
                fields: transformed_fields,
            })
        }
        sbor::Value::Array {
            element_value_kind,
            elements,
        } => {
            let mut transformed_elements = Vec::new();
            for element in elements {
                transformed_elements.push(transform(element, handler)?);
            }
            Ok(ScryptoValue::Array {
                element_value_kind: transform_value_kind(element_value_kind),
                elements: transformed_elements,
            })
        }
        sbor::Value::Tuple { fields } => {
            let mut transformed_fields = Vec::new();
            for field in fields {
                transformed_fields.push(transform(field, handler)?);
            }
            Ok(ScryptoValue::Tuple {
                fields: transformed_fields,
            })
        }
        sbor::Value::Map {
            key_value_kind,
            value_value_kind,
            entries,
        } => {
            let mut transformed_entries = Vec::new();
            for entry in entries {
                transformed_entries
                    .push((transform(entry.0, handler)?, transform(entry.1, handler)?));
            }
            Ok(ScryptoValue::Map {
                key_value_kind: transform_value_kind(key_value_kind),
                value_value_kind: transform_value_kind(value_value_kind),
                entries: transformed_entries,
            })
        }
        sbor::Value::Custom { value } => match value {
            ManifestCustomValue::Address(address) => Ok(ScryptoValue::Custom {
                value: ScryptoCustomValue::Reference(Reference(address.0)),
            }),
            ManifestCustomValue::Bucket(b) => Ok(ScryptoValue::Custom {
                value: ScryptoCustomValue::Own(handler.replace_bucket(b)?),
            }),
            ManifestCustomValue::Proof(p) => Ok(ScryptoValue::Custom {
                value: ScryptoCustomValue::Own(handler.replace_proof(p)?),
            }),
            ManifestCustomValue::Expression(e) => Ok(ScryptoValue::Array {
                element_value_kind: ScryptoValueKind::Custom(ScryptoCustomValueKind::Own),
                elements: handler
                    .replace_expression(e)?
                    .into_iter()
                    .map(|e| ScryptoValue::Custom {
                        value: ScryptoCustomValue::Own(e),
                    })
                    .collect(),
            }),
            ManifestCustomValue::Blob(b) => Ok(ScryptoValue::Array {
                element_value_kind: ScryptoValueKind::Custom(ScryptoCustomValueKind::Own),
                elements: handler
                    .replace_blob(b)?
                    .into_iter()
                    .map(|e| ScryptoValue::U8 { value: e })
                    .collect(),
            }),
            ManifestCustomValue::Decimal(d) => Ok(ScryptoValue::Custom {
                value: ScryptoCustomValue::Decimal(to_decimal(d)),
            }),
            ManifestCustomValue::PreciseDecimal(d) => Ok(ScryptoValue::Custom {
                value: ScryptoCustomValue::PreciseDecimal(to_precise_decimal(d)),
            }),
            ManifestCustomValue::NonFungibleLocalId(id) => Ok(ScryptoValue::Custom {
                value: ScryptoCustomValue::NonFungibleLocalId(to_non_fungible_local_id(id)),
            }),
        },
    }
}

pub fn transform_value_kind(kind: ManifestValueKind) -> ScryptoValueKind {
    match kind {
        sbor::ValueKind::Bool => ScryptoValueKind::Bool,
        sbor::ValueKind::I8 => ScryptoValueKind::I8,
        sbor::ValueKind::I16 => ScryptoValueKind::I16,
        sbor::ValueKind::I32 => ScryptoValueKind::I32,
        sbor::ValueKind::I64 => ScryptoValueKind::I64,
        sbor::ValueKind::I128 => ScryptoValueKind::I128,
        sbor::ValueKind::U8 => ScryptoValueKind::U8,
        sbor::ValueKind::U16 => ScryptoValueKind::U16,
        sbor::ValueKind::U32 => ScryptoValueKind::U32,
        sbor::ValueKind::U64 => ScryptoValueKind::U64,
        sbor::ValueKind::U128 => ScryptoValueKind::U128,
        sbor::ValueKind::String => ScryptoValueKind::String,
        sbor::ValueKind::Enum => ScryptoValueKind::Enum,
        sbor::ValueKind::Array => ScryptoValueKind::Array,
        sbor::ValueKind::Tuple => ScryptoValueKind::Tuple,
        sbor::ValueKind::Map => ScryptoValueKind::Map,
        sbor::ValueKind::Custom(c) => match c {
            ManifestCustomValueKind::Address => {
                ScryptoValueKind::Custom(ScryptoCustomValueKind::Reference)
            }
            ManifestCustomValueKind::Bucket => {
                ScryptoValueKind::Custom(ScryptoCustomValueKind::Own)
            }
            ManifestCustomValueKind::Proof => ScryptoValueKind::Custom(ScryptoCustomValueKind::Own),
            ManifestCustomValueKind::Expression => ScryptoValueKind::Array,
            ManifestCustomValueKind::Blob => ScryptoValueKind::Array,
            ManifestCustomValueKind::Decimal => {
                ScryptoValueKind::Custom(ScryptoCustomValueKind::Decimal)
            }
            ManifestCustomValueKind::PreciseDecimal => {
                ScryptoValueKind::Custom(ScryptoCustomValueKind::PreciseDecimal)
            }
            ManifestCustomValueKind::NonFungibleLocalId => {
                ScryptoValueKind::Custom(ScryptoCustomValueKind::NonFungibleLocalId)
            }
        },
    }
}
