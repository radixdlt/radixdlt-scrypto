use crate::data::{to_decimal, to_non_fungible_local_id, to_precise_decimal};
use radix_engine_interface::data::manifest::{
    model::*, ManifestCustomValue, ManifestCustomValueKind, ManifestValue, ManifestValueKind,
};
use radix_engine_interface::types::ResourceAddress;
use radix_engine_interface::{address::Bech32Encoder, blueprints::resource::NonFungibleGlobalId};
use sbor::rust::collections::NonIterMap;
use sbor::rust::fmt;
use sbor::*;
use utils::ContextualDisplay;

#[derive(Clone, Copy, Debug)]
pub struct ManifestValueDisplayContext<'a> {
    pub bech32_encoder: Option<&'a Bech32Encoder>,
    pub bucket_names: Option<&'a NonIterMap<ManifestBucket, String>>,
    pub proof_names: Option<&'a NonIterMap<ManifestProof, String>>,
}

impl<'a> ManifestValueDisplayContext<'a> {
    pub fn no_context() -> Self {
        Self {
            bech32_encoder: None,
            bucket_names: None,
            proof_names: None,
        }
    }

    pub fn with_optional_bech32(bech32_encoder: Option<&'a Bech32Encoder>) -> Self {
        Self {
            bech32_encoder,
            bucket_names: None,
            proof_names: None,
        }
    }

    pub fn with_bech32_and_names(
        bech32_encoder: Option<&'a Bech32Encoder>,
        bucket_names: &'a NonIterMap<ManifestBucket, String>,
        proof_names: &'a NonIterMap<ManifestProof, String>,
    ) -> Self {
        Self {
            bech32_encoder,
            bucket_names: Some(bucket_names),
            proof_names: Some(proof_names),
        }
    }

    pub fn get_bucket_name(&self, bucket_id: &ManifestBucket) -> Option<&str> {
        self.bucket_names
            .and_then(|names| names.get(bucket_id).map(|s| s.as_str()))
    }

    pub fn get_proof_name(&self, proof_id: &ManifestProof) -> Option<&str> {
        self.proof_names
            .and_then(|names| names.get(proof_id).map(|s| s.as_str()))
    }
}

impl<'a> Into<ManifestValueDisplayContext<'a>> for &'a Bech32Encoder {
    fn into(self) -> ManifestValueDisplayContext<'a> {
        ManifestValueDisplayContext::with_optional_bech32(Some(self))
    }
}

impl<'a> Into<ManifestValueDisplayContext<'a>> for Option<&'a Bech32Encoder> {
    fn into(self) -> ManifestValueDisplayContext<'a> {
        ManifestValueDisplayContext::with_optional_bech32(self)
    }
}

impl<'a> ContextualDisplay<ManifestValueDisplayContext<'a>> for ManifestValue {
    type Error = fmt::Error;

    fn contextual_format<F: fmt::Write>(
        &self,
        f: &mut F,
        context: &ManifestValueDisplayContext<'a>,
    ) -> Result<(), Self::Error> {
        format_manifest_value(f, self, context)
    }
}

pub fn format_manifest_value<F: fmt::Write>(
    f: &mut F,
    value: &ManifestValue,
    context: &ManifestValueDisplayContext,
) -> fmt::Result {
    match value {
        // primitive types
        Value::Bool { value } => write!(f, "{}", value)?,
        Value::I8 { value } => write!(f, "{}i8", value)?,
        Value::I16 { value } => write!(f, "{}i16", value)?,
        Value::I32 { value } => write!(f, "{}i32", value)?,
        Value::I64 { value } => write!(f, "{}i64", value)?,
        Value::I128 { value } => write!(f, "{}i128", value)?,
        Value::U8 { value } => write!(f, "{}u8", value)?,
        Value::U16 { value } => write!(f, "{}u16", value)?,
        Value::U32 { value } => write!(f, "{}u32", value)?,
        Value::U64 { value } => write!(f, "{}u64", value)?,
        Value::U128 { value } => write!(f, "{}u128", value)?,
        Value::String { value } => write!(f, "\"{}\"", value)?,
        Value::Tuple { fields } => {
            if fields.len() == 2 {
                if let (
                    ManifestValue::Custom {
                        value: ManifestCustomValue::Address(address),
                    },
                    ManifestValue::Custom {
                        value: ManifestCustomValue::NonFungibleLocalId(id),
                    },
                ) = (&fields[0], &fields[1])
                {
                    if let Ok(resource_address) = ResourceAddress::try_from(address.0.as_ref()) {
                        let global_id = NonFungibleGlobalId::new(
                            resource_address,
                            to_non_fungible_local_id(id.clone()),
                        );
                        return write!(
                            f,
                            "NonFungibleGlobalId(\"{}\")",
                            global_id.display(context.bech32_encoder)
                        );
                    }
                }
            }

            f.write_str("Tuple(")?;
            format_elements(f, fields, context)?;
            f.write_str(")")?;
        }
        Value::Enum {
            discriminator,
            fields,
        } => {
            f.write_str("Enum(")?;
            f.write_str(discriminator.to_string().as_str())?;
            f.write_str("u8")?;
            if !fields.is_empty() {
                f.write_str(", ")?;
                format_elements(f, fields, context)?;
            }
            f.write_str(")")?;
        }
        Value::Array {
            element_value_kind,
            elements,
        } => match element_value_kind {
            ValueKind::U8 => {
                let vec: Vec<u8> = elements
                    .iter()
                    .map(|e| match e {
                        Value::U8 { value } => Ok(*value),
                        _ => Err(fmt::Error),
                    })
                    .collect::<Result<_, _>>()?;
                f.write_str("Bytes(\"")?;
                f.write_str(&hex::encode(vec))?;
                f.write_str("\")")?;
            }
            _ => {
                f.write_str("Array<")?;
                format_value_kind(f, element_value_kind)?;
                f.write_str(">(")?;
                format_elements(f, elements, context)?;
                f.write_str(")")?;
            }
        },
        Value::Map {
            key_value_kind,
            value_value_kind,
            entries,
        } => {
            f.write_str("Map<")?;
            format_value_kind(f, key_value_kind)?;
            f.write_str(", ")?;
            format_value_kind(f, value_value_kind)?;
            f.write_str(">(")?;
            format_kv_entries(f, entries, context)?;
            f.write_str(")")?;
        }
        // custom types
        Value::Custom { value } => {
            format_custom_value(f, value, context)?;
        }
    };
    Ok(())
}

pub fn format_tuple<F: fmt::Write>(
    f: &mut F,
    name: &'static str,
    fields: &[ManifestValue],
    context: &ManifestValueDisplayContext,
) -> fmt::Result {
    f.write_str(name)?;
    f.write_str("(")?;
    format_elements(f, fields, context)?;
    f.write_str(")")?;
    Ok(())
}

pub fn format_value_kind<F: fmt::Write>(f: &mut F, value_kind: &ManifestValueKind) -> fmt::Result {
    match value_kind {
        ValueKind::Bool => f.write_str("Bool"),
        ValueKind::I8 => f.write_str("I8"),
        ValueKind::I16 => f.write_str("I16"),
        ValueKind::I32 => f.write_str("I32"),
        ValueKind::I64 => f.write_str("I64"),
        ValueKind::I128 => f.write_str("I128"),
        ValueKind::U8 => f.write_str("U8"),
        ValueKind::U16 => f.write_str("U16"),
        ValueKind::U32 => f.write_str("U32"),
        ValueKind::U64 => f.write_str("U64"),
        ValueKind::U128 => f.write_str("U128"),
        ValueKind::String => f.write_str("String"),
        ValueKind::Enum => f.write_str("Enum"),
        ValueKind::Array => f.write_str("Array"),
        ValueKind::Tuple => f.write_str("Tuple"),
        ValueKind::Map => f.write_str("Map"),
        ValueKind::Custom(value_kind) => match value_kind {
            ManifestCustomValueKind::Address => f.write_str("Address"),
            ManifestCustomValueKind::Bucket => f.write_str("Bucket"),
            ManifestCustomValueKind::Proof => f.write_str("Proof"),
            ManifestCustomValueKind::Expression => f.write_str("Expression"),
            ManifestCustomValueKind::Blob => f.write_str("Blob"),
            ManifestCustomValueKind::Decimal => f.write_str("Decimal"),
            ManifestCustomValueKind::PreciseDecimal => f.write_str("PreciseDecimal"),
            ManifestCustomValueKind::NonFungibleLocalId => f.write_str("NonFungibleLocalId"),
        },
    }
}

pub fn display_value_kind(value_kind: &ManifestValueKind) -> DisplayableManifestValueKind {
    DisplayableManifestValueKind(value_kind)
}

pub struct DisplayableManifestValueKind<'a>(&'a ManifestValueKind);

impl<'a> fmt::Display for DisplayableManifestValueKind<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        format_value_kind(f, &self.0)
    }
}

pub fn format_elements<F: fmt::Write>(
    f: &mut F,
    values: &[ManifestValue],
    context: &ManifestValueDisplayContext,
) -> fmt::Result {
    for (i, x) in values.iter().enumerate() {
        if i != 0 {
            f.write_str(", ")?;
        }
        format_manifest_value(f, x, context)?;
    }
    Ok(())
}

pub fn format_kv_entries<F: fmt::Write>(
    f: &mut F,
    entries: &[(ManifestValue, ManifestValue)],
    context: &ManifestValueDisplayContext,
) -> fmt::Result {
    for (i, x) in entries.iter().enumerate() {
        if i != 0 {
            f.write_str(", ")?;
        }
        format_manifest_value(f, &x.0, context)?;
        f.write_str(", ")?;
        format_manifest_value(f, &x.1, context)?;
    }
    Ok(())
}

impl<'a> ContextualDisplay<ManifestValueDisplayContext<'a>> for ManifestCustomValue {
    type Error = fmt::Error;

    fn contextual_format<F: fmt::Write>(
        &self,
        f: &mut F,
        context: &ManifestValueDisplayContext<'a>,
    ) -> Result<(), Self::Error> {
        format_custom_value(f, self, context)
    }
}

pub fn format_custom_value<F: fmt::Write>(
    f: &mut F,
    value: &ManifestCustomValue,
    context: &ManifestValueDisplayContext,
) -> fmt::Result {
    match value {
        ManifestCustomValue::Address(value) => {
            f.write_str("Address(\"")?;
            if let Some(encoder) = context.bech32_encoder {
                if let Ok(bech32) = encoder.encode(value.0.as_ref()) {
                    f.write_str(&bech32)?;
                } else {
                    f.write_str(&hex::encode(value.0.as_ref()))?;
                }
            } else {
                f.write_str(&hex::encode(value.0.as_ref()))?;
            }
            f.write_str("\")")?;
        }
        ManifestCustomValue::Bucket(value) => {
            if let Some(name) = context.get_bucket_name(&value) {
                write!(f, "Bucket(\"{}\")", name)?;
            } else {
                write!(f, "Bucket({}u32)", value.0)?;
            }
        }
        ManifestCustomValue::Proof(value) => {
            if let Some(name) = context.get_proof_name(&value) {
                write!(f, "Proof(\"{}\")", name)?;
            } else {
                write!(f, "Proof({}u32)", value.0)?;
            }
        }
        ManifestCustomValue::Expression(value) => {
            write!(
                f,
                "Expression(\"{}\")",
                match value {
                    ManifestExpression::EntireWorktop => "ENTIRE_WORKTOP",
                    ManifestExpression::EntireAuthZone => "ENTIRE_AUTH_ZONE",
                }
            )?;
        }
        ManifestCustomValue::Blob(value) => {
            write!(f, "Blob(\"{}\")", hex::encode(&value.0))?;
        }
        ManifestCustomValue::Decimal(value) => {
            write!(f, "Decimal(\"{}\")", to_decimal(value.clone()))?;
        }
        ManifestCustomValue::PreciseDecimal(value) => {
            write!(
                f,
                "PreciseDecimal(\"{}\")",
                to_precise_decimal(value.clone())
            )?;
        }
        ManifestCustomValue::NonFungibleLocalId(value) => {
            write!(
                f,
                "NonFungibleLocalId(\"{}\")",
                to_non_fungible_local_id(value.clone())
            )?;
        }
    }
    Ok(())
}
