use super::types::ManifestBucket;
use super::types::ManifestProof;
use crate::address::Bech32Encoder;
use crate::api::types::*;
use crate::data::*;
use sbor::rust::collections::HashMap;
use sbor::rust::fmt;
use utils::ContextualDisplay;

#[derive(Clone, Copy, Debug)]
pub struct ValueFormattingContext<'a> {
    pub bech32_encoder: Option<&'a Bech32Encoder>,
    pub bucket_names: Option<&'a HashMap<ManifestBucket, String>>,
    pub proof_names: Option<&'a HashMap<ManifestProof, String>>,
}

impl<'a> ValueFormattingContext<'a> {
    pub fn no_context() -> Self {
        Self {
            bech32_encoder: None,
            bucket_names: None,
            proof_names: None,
        }
    }

    pub fn no_manifest_context(bech32_encoder: Option<&'a Bech32Encoder>) -> Self {
        Self {
            bech32_encoder,
            bucket_names: None,
            proof_names: None,
        }
    }

    pub fn with_manifest_context(
        bech32_encoder: Option<&'a Bech32Encoder>,
        bucket_names: &'a HashMap<ManifestBucket, String>,
        proof_names: &'a HashMap<ManifestProof, String>,
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

impl<'a> Into<ValueFormattingContext<'a>> for &'a Bech32Encoder {
    fn into(self) -> ValueFormattingContext<'a> {
        ValueFormattingContext::no_manifest_context(Some(self))
    }
}

impl<'a> Into<ValueFormattingContext<'a>> for Option<&'a Bech32Encoder> {
    fn into(self) -> ValueFormattingContext<'a> {
        ValueFormattingContext::no_manifest_context(self)
    }
}

impl<'a> ContextualDisplay<ValueFormattingContext<'a>> for ScryptoValue {
    type Error = fmt::Error;

    fn contextual_format<F: fmt::Write>(
        &self,
        f: &mut F,
        context: &ValueFormattingContext<'a>,
    ) -> Result<(), Self::Error> {
        format_scrypto_value(f, self, context)
    }
}

pub fn format_scrypto_value<F: fmt::Write>(
    f: &mut F,
    value: &ScryptoValue,
    context: &ValueFormattingContext,
) -> fmt::Result {
    match value {
        // primitive types
        Value::Unit => write!(f, "()")?,
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
                    ScryptoValue::Custom {
                        value: ScryptoCustomValue::ResourceAddress(address),
                    },
                    ScryptoValue::Custom {
                        value: ScryptoCustomValue::NonFungibleId(id),
                    },
                ) = (&fields[0], &fields[1])
                {
                    f.write_str("NonFungibleAddress(\"")?;
                    write!(f, "{}", address.display(context.bech32_encoder))?;
                    f.write_str("\", ")?;
                    match id {
                        NonFungibleId::U32(v) => {
                            write!(f, "{}u32", v)?;
                        }
                        NonFungibleId::U64(v) => {
                            write!(f, "{}u64", v)?;
                        }
                        NonFungibleId::UUID(v) => {
                            write!(f, "{}u128", v)?;
                        }
                        NonFungibleId::Bytes(v) => {
                            write!(f, "Bytes(\"{}\")", hex::encode(v))?;
                        }
                        NonFungibleId::String(v) => {
                            write!(f, "\"{}\"", v)?;
                        }
                    }
                    f.write_str(")")?;
                    return Ok(());
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
            match (discriminator.as_str(), fields.len()) {
                // Map aliases
                ("Some", 1) => format_tuple(f, "Some", fields, context)?,
                ("None", 0) => f.write_str("None")?,
                ("Ok", 1) => format_tuple(f, "Ok", fields, context)?,
                ("Err", 1) => format_tuple(f, "Err", fields, context)?,
                // Standard
                (_, _) => {
                    f.write_str("Enum(\"")?;
                    f.write_str(discriminator)?;
                    f.write_str("\"")?;
                    if !fields.is_empty() {
                        f.write_str(", ")?;
                        format_elements(f, fields, context)?;
                    }
                    f.write_str(")")?;
                }
            }
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
    fields: &[ScryptoValue],
    context: &ValueFormattingContext,
) -> fmt::Result {
    f.write_str(name)?;
    f.write_str("(")?;
    format_elements(f, fields, context)?;
    f.write_str(")")?;
    Ok(())
}

pub fn format_value_kind<F: fmt::Write>(f: &mut F, value_kind: &ScryptoValueKind) -> fmt::Result {
    match value_kind {
        ValueKind::Unit => f.write_str("Unit"),
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
        ValueKind::Custom(value_kind) => match value_kind {
            ScryptoCustomValueKind::PackageAddress => f.write_str("PackageAddress"),
            ScryptoCustomValueKind::ComponentAddress => f.write_str("ComponentAddress"),
            ScryptoCustomValueKind::ResourceAddress => f.write_str("ResourceAddress"),
            ScryptoCustomValueKind::SystemAddress => f.write_str("SystemAddress"),
            ScryptoCustomValueKind::Own => f.write_str("Own"),
            ScryptoCustomValueKind::Bucket => f.write_str("Bucket"),
            ScryptoCustomValueKind::Proof => f.write_str("Proof"),
            ScryptoCustomValueKind::Expression => f.write_str("Expression"),
            ScryptoCustomValueKind::Blob => f.write_str("Blob"),
            ScryptoCustomValueKind::Hash => f.write_str("Hash"),
            ScryptoCustomValueKind::EcdsaSecp256k1PublicKey => {
                f.write_str("EcdsaSecp256k1PublicKey")
            }
            ScryptoCustomValueKind::EcdsaSecp256k1Signature => {
                f.write_str("EcdsaSecp256k1Signature")
            }
            ScryptoCustomValueKind::EddsaEd25519PublicKey => f.write_str("EddsaEd25519PublicKey"),
            ScryptoCustomValueKind::EddsaEd25519Signature => f.write_str("EddsaEd25519Signature"),
            ScryptoCustomValueKind::Decimal => f.write_str("Decimal"),
            ScryptoCustomValueKind::PreciseDecimal => f.write_str("PreciseDecimal"),
            ScryptoCustomValueKind::NonFungibleId => f.write_str("NonFungibleId"),
        },
    }
}

pub fn display_value_kind(value_kind: &ScryptoValueKind) -> DisplayableScryptoValueKind {
    DisplayableScryptoValueKind(value_kind)
}

pub struct DisplayableScryptoValueKind<'a>(&'a ScryptoValueKind);

impl<'a> fmt::Display for DisplayableScryptoValueKind<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        format_value_kind(f, &self.0)
    }
}

pub fn format_elements<F: fmt::Write>(
    f: &mut F,
    values: &[ScryptoValue],
    context: &ValueFormattingContext,
) -> fmt::Result {
    for (i, x) in values.iter().enumerate() {
        if i != 0 {
            f.write_str(", ")?;
        }
        format_scrypto_value(f, x, context)?;
    }
    Ok(())
}

impl<'a> ContextualDisplay<ValueFormattingContext<'a>> for ScryptoCustomValue {
    type Error = fmt::Error;

    fn contextual_format<F: fmt::Write>(
        &self,
        f: &mut F,
        context: &ValueFormattingContext<'a>,
    ) -> Result<(), Self::Error> {
        format_custom_value(f, self, context)
    }
}

pub fn format_custom_value<F: fmt::Write>(
    f: &mut F,
    value: &ScryptoCustomValue,
    context: &ValueFormattingContext,
) -> fmt::Result {
    match value {
        // Global address types
        ScryptoCustomValue::PackageAddress(value) => {
            f.write_str("PackageAddress(\"")?;
            value
                .format(f, context.bech32_encoder)
                .expect("Failed to format address");
            f.write_str("\")")?;
        }
        ScryptoCustomValue::ComponentAddress(value) => {
            f.write_str("ComponentAddress(\"")?;
            value
                .format(f, context.bech32_encoder)
                .expect("Failed to format address");
            f.write_str("\")")?;
        }
        ScryptoCustomValue::ResourceAddress(value) => {
            f.write_str("ResourceAddress(\"")?;
            value
                .format(f, context.bech32_encoder)
                .expect("Failed to format address");
            f.write_str("\")")?;
        }
        ScryptoCustomValue::SystemAddress(value) => {
            f.write_str("SystemAddress(\"")?;
            value
                .format(f, context.bech32_encoder)
                .expect("Failed to format address");
            f.write_str("\")")?;
        }
        // RE interpreted
        ScryptoCustomValue::Own(value) => {
            write!(f, "Own(\"{:?}\")", value)?; // TODO: fix syntax
        }
        ScryptoCustomValue::Blob(value) => {
            write!(f, "Blob(\"{}\")", value)?;
        }
        // TX interpreted
        ScryptoCustomValue::Bucket(value) => {
            if let Some(name) = context.get_bucket_name(&value) {
                write!(f, "Bucket(\"{}\")", name)?;
            } else {
                write!(f, "Bucket({}u32)", value.0)?;
            }
        }
        ScryptoCustomValue::Proof(value) => {
            if let Some(name) = context.get_proof_name(&value) {
                write!(f, "Proof(\"{}\")", name)?;
            } else {
                write!(f, "Proof({}u32)", value.0)?;
            }
        }
        ScryptoCustomValue::Expression(value) => {
            write!(
                f,
                "Expression(\"{}\")",
                match value {
                    types::ManifestExpression::EntireWorktop => "ENTIRE_WORKTOP",
                    types::ManifestExpression::EntireAuthZone => "ENTIRE_AUTH_ZONE",
                }
            )?;
        }
        // Uninterpreted
        ScryptoCustomValue::Hash(value) => {
            write!(f, "Hash(\"{}\")", value)?;
        }
        ScryptoCustomValue::EcdsaSecp256k1PublicKey(value) => {
            write!(f, "EcdsaSecp256k1PublicKey(\"{}\")", value)?;
        }
        ScryptoCustomValue::EcdsaSecp256k1Signature(value) => {
            write!(f, "EcdsaSecp256k1Signature(\"{}\")", value)?;
        }
        ScryptoCustomValue::EddsaEd25519PublicKey(value) => {
            write!(f, "EddsaEd25519PublicKey(\"{}\")", value)?;
        }
        ScryptoCustomValue::EddsaEd25519Signature(value) => {
            write!(f, "EddsaEd25519Signature(\"{}\")", value)?;
        }
        ScryptoCustomValue::Decimal(value) => {
            write!(f, "Decimal(\"{}\")", value)?;
        }
        ScryptoCustomValue::PreciseDecimal(value) => {
            write!(f, "PreciseDecimal(\"{}\")", value)?;
        }
        ScryptoCustomValue::NonFungibleId(value) => {
            f.write_str("NonFungibleId(")?;
            format_non_fungible_id_contents(f, value)?;
            write!(f, ")")?;
        }
    }
    Ok(())
}

pub fn format_non_fungible_id_contents<F: fmt::Write>(
    f: &mut F,
    value: &NonFungibleId,
) -> fmt::Result {
    match value {
        NonFungibleId::Bytes(b) => write!(f, "Bytes(\"{}\")", hex::encode(b)),
        NonFungibleId::String(s) => write!(f, "\"{}\"", s),
        NonFungibleId::U32(n) => write!(f, "{}u32", n),
        NonFungibleId::U64(n) => write!(f, "{}u64", n),
        NonFungibleId::UUID(u) => write!(f, "{}u128", u),
    }
}
