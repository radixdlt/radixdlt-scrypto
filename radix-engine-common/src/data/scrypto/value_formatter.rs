use crate::address::Bech32Encoder;
use crate::data::scrypto::*;
use sbor::rust::fmt;
use sbor::rust::prelude::*;
use utils::ContextualDisplay;

#[derive(Clone, Copy, Debug, Default)]
pub struct ScryptoValueDisplayContext<'a> {
    pub bech32_encoder: Option<&'a Bech32Encoder>,
}

impl<'a> ScryptoValueDisplayContext<'a> {
    pub fn no_context() -> Self {
        Self {
            bech32_encoder: None,
        }
    }

    pub fn with_optional_bench32(bech32_encoder: Option<&'a Bech32Encoder>) -> Self {
        Self { bech32_encoder }
    }
}

impl<'a> Into<ScryptoValueDisplayContext<'a>> for &'a Bech32Encoder {
    fn into(self) -> ScryptoValueDisplayContext<'a> {
        ScryptoValueDisplayContext::with_optional_bench32(Some(self))
    }
}

impl<'a> Into<ScryptoValueDisplayContext<'a>> for Option<&'a Bech32Encoder> {
    fn into(self) -> ScryptoValueDisplayContext<'a> {
        ScryptoValueDisplayContext::with_optional_bench32(self)
    }
}

impl<'a> ContextualDisplay<ScryptoValueDisplayContext<'a>> for ScryptoValue {
    type Error = fmt::Error;

    fn contextual_format<F: fmt::Write>(
        &self,
        f: &mut F,
        context: &ScryptoValueDisplayContext<'a>,
    ) -> Result<(), Self::Error> {
        format_scrypto_value(f, self, context)
    }
}

pub fn format_scrypto_value<F: fmt::Write>(
    f: &mut F,
    value: &ScryptoValue,
    context: &ScryptoValueDisplayContext,
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
    fields: &[ScryptoValue],
    context: &ScryptoValueDisplayContext,
) -> fmt::Result {
    f.write_str(name)?;
    f.write_str("(")?;
    format_elements(f, fields, context)?;
    f.write_str(")")?;
    Ok(())
}

pub fn format_value_kind<F: fmt::Write>(f: &mut F, value_kind: &ScryptoValueKind) -> fmt::Result {
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
            ScryptoCustomValueKind::Address => f.write_str("Address"),
            ScryptoCustomValueKind::Own => f.write_str("Own"),
            ScryptoCustomValueKind::Decimal => f.write_str("Decimal"),
            ScryptoCustomValueKind::PreciseDecimal => f.write_str("PreciseDecimal"),
            ScryptoCustomValueKind::NonFungibleLocalId => f.write_str("NonFungibleLocalId"),
            ScryptoCustomValueKind::Reference => f.write_str("Reference"),
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
    context: &ScryptoValueDisplayContext,
) -> fmt::Result {
    for (i, x) in values.iter().enumerate() {
        if i != 0 {
            f.write_str(", ")?;
        }
        format_scrypto_value(f, x, context)?;
    }
    Ok(())
}

pub fn format_kv_entries<F: fmt::Write>(
    f: &mut F,
    entries: &[(ScryptoValue, ScryptoValue)],
    context: &ScryptoValueDisplayContext,
) -> fmt::Result {
    for (i, x) in entries.iter().enumerate() {
        if i != 0 {
            f.write_str(", ")?;
        }
        format_scrypto_value(f, &x.0, context)?;
        f.write_str(", ")?;
        format_scrypto_value(f, &x.1, context)?;
    }
    Ok(())
}

impl<'a> ContextualDisplay<ScryptoValueDisplayContext<'a>> for ScryptoCustomValue {
    type Error = fmt::Error;

    fn contextual_format<F: fmt::Write>(
        &self,
        f: &mut F,
        context: &ScryptoValueDisplayContext<'a>,
    ) -> Result<(), Self::Error> {
        format_custom_value(f, self, context)
    }
}

pub fn format_custom_value<F: fmt::Write>(
    f: &mut F,
    value: &ScryptoCustomValue,
    _context: &ScryptoValueDisplayContext,
) -> fmt::Result {
    match value {
        ScryptoCustomValue::Address(value) => {
            write!(f, "Address(\"{}\")", hex::encode(value.to_vec()))?;
        }
        ScryptoCustomValue::InternalRef(value) => {
            write!(f, "Address(\"{}\")", hex::encode(value.to_vec()))?;
        }
        ScryptoCustomValue::Own(value) => {
            write!(f, "Own(\"{}\")", hex::encode(value.to_vec()))?;
        }
        ScryptoCustomValue::Decimal(value) => {
            write!(f, "Decimal(\"{}\")", value)?;
        }
        ScryptoCustomValue::PreciseDecimal(value) => {
            write!(f, "PreciseDecimal(\"{}\")", value)?;
        }
        ScryptoCustomValue::NonFungibleLocalId(value) => {
            write!(f, "NonFungibleLocalId(\"{}\")", value)?;
        }
    }
    Ok(())
}
