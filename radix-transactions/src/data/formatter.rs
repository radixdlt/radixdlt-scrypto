use crate::data::{to_decimal, to_non_fungible_local_id, to_precise_decimal};
use crate::internal_prelude::*;
use radix_common::types::NonFungibleGlobalId;
use radix_engine_interface::types::ResourceAddress;
use radix_rust::unicode::{CustomCharEscaper, EscapeBehaviour};
use sbor::rust::fmt;
use sbor::*;

#[derive(Clone, Copy, Debug)]
pub struct MultiLine {
    margin: usize,
    indent: usize,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ManifestDecompilationDisplayContext<'a> {
    pub address_bech32_encoder: Option<&'a AddressBech32Encoder>,
    pub object_names: ManifestObjectNamesRef<'a>,
    pub multi_line: Option<MultiLine>,
}

impl<'a> ManifestDecompilationDisplayContext<'a> {
    pub fn no_context() -> Self {
        Self::default()
    }

    pub fn with_optional_bech32(address_bech32_encoder: Option<&'a AddressBech32Encoder>) -> Self {
        Self {
            address_bech32_encoder,
            ..Default::default()
        }
    }

    pub fn with_bech32_and_names(
        address_bech32_encoder: Option<&'a AddressBech32Encoder>,
        object_names: ManifestObjectNamesRef<'a>,
    ) -> Self {
        Self {
            address_bech32_encoder,
            object_names,
            ..Default::default()
        }
    }

    pub fn with_multi_line(mut self, margin: usize, indent: usize) -> Self {
        self.multi_line = Some(MultiLine { margin, indent });
        self
    }

    pub fn get_object_names(&self) -> ManifestObjectNamesRef {
        self.object_names
    }

    pub fn get_indent(&self, depth: usize) -> String {
        if let Some(MultiLine { margin, indent }) = self.multi_line {
            " ".repeat(margin + indent * depth)
        } else {
            String::new()
        }
    }

    pub fn get_new_line(&self) -> &str {
        if self.multi_line.is_some() {
            "\n"
        } else {
            " "
        }
    }
}

impl<'a> Into<ManifestDecompilationDisplayContext<'a>> for &'a AddressBech32Encoder {
    fn into(self) -> ManifestDecompilationDisplayContext<'a> {
        ManifestDecompilationDisplayContext::with_optional_bech32(Some(self))
    }
}

impl<'a> Into<ManifestDecompilationDisplayContext<'a>> for Option<&'a AddressBech32Encoder> {
    fn into(self) -> ManifestDecompilationDisplayContext<'a> {
        ManifestDecompilationDisplayContext::with_optional_bech32(self)
    }
}

impl<'a> ContextualDisplay<ManifestDecompilationDisplayContext<'a>> for ManifestValue {
    type Error = fmt::Error;

    fn contextual_format<F: fmt::Write>(
        &self,
        f: &mut F,
        context: &ManifestDecompilationDisplayContext<'a>,
    ) -> Result<(), Self::Error> {
        format_manifest_value(f, self, context, false, 0)
    }
}

macro_rules! write_with_indent {
    ($f:expr, $context:expr, $should_indent:expr, $depth:expr, $($args:expr),*) => {
        if $should_indent {
            write!($f,
                "{}{}",
                $context.get_indent($depth),
                format!($($args),*)
            )
        } else {
            write!($f, $($args),*)
        }
    };
}

pub fn format_manifest_value<F: fmt::Write>(
    f: &mut F,
    value: &ManifestValue,
    context: &ManifestDecompilationDisplayContext,
    indent_start: bool,
    depth: usize,
) -> fmt::Result {
    match value {
        // primitive types
        Value::Bool { value } => write_with_indent!(f, context, indent_start, depth, "{}", value)?,
        Value::I8 { value } => write_with_indent!(f, context, indent_start, depth, "{}i8", value)?,
        Value::I16 { value } => {
            write_with_indent!(f, context, indent_start, depth, "{}i16", value)?
        }
        Value::I32 { value } => {
            write_with_indent!(f, context, indent_start, depth, "{}i32", value)?
        }
        Value::I64 { value } => {
            write_with_indent!(f, context, indent_start, depth, "{}i64", value)?
        }
        Value::I128 { value } => {
            write_with_indent!(f, context, indent_start, depth, "{}i128", value)?
        }
        Value::U8 { value } => write_with_indent!(f, context, indent_start, depth, "{}u8", value)?,
        Value::U16 { value } => {
            write_with_indent!(f, context, indent_start, depth, "{}u16", value)?
        }
        Value::U32 { value } => {
            write_with_indent!(f, context, indent_start, depth, "{}u32", value)?
        }
        Value::U64 { value } => {
            write_with_indent!(f, context, indent_start, depth, "{}u64", value)?
        }
        Value::U128 { value } => {
            write_with_indent!(f, context, indent_start, depth, "{}u128", value)?
        }
        Value::String { value } => {
            write_with_indent!(
                f,
                context,
                indent_start,
                depth,
                "{}",
                ManifestCustomCharEscaper::escaped(value.as_str())
            )?;
        }
        Value::Tuple { fields } => {
            if fields.len() == 2 {
                if let (
                    ManifestValue::Custom {
                        value: ManifestCustomValue::Address(ManifestAddress::Static(address)),
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
                        return write_with_indent!(
                            f,
                            context,
                            indent_start,
                            depth,
                            "NonFungibleGlobalId(\"{}\")",
                            global_id.display(context.address_bech32_encoder)
                        );
                    }
                }
            }

            if fields.is_empty() {
                write_with_indent!(f, context, indent_start, depth, "Tuple()")?;
            } else {
                write_with_indent!(
                    f,
                    context,
                    indent_start,
                    depth,
                    "Tuple({}",
                    context.get_new_line()
                )?;
                format_elements(f, fields, context, depth + 1)?;
                write_with_indent!(f, context, true, depth, ")")?;
            }
        }
        Value::Enum {
            discriminator,
            fields,
        } => {
            if fields.is_empty() {
                write_with_indent!(
                    f,
                    context,
                    indent_start,
                    depth,
                    "Enum<{}u8>()",
                    discriminator
                )?;
            } else {
                write_with_indent!(
                    f,
                    context,
                    indent_start,
                    depth,
                    "Enum<{}u8>({}",
                    discriminator,
                    context.get_new_line()
                )?;
                format_elements(f, fields, context, depth + 1)?;
                write_with_indent!(f, context, true, depth, ")")?;
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

                write_with_indent!(
                    f,
                    context,
                    indent_start,
                    depth,
                    "Bytes(\"{}\")",
                    hex::encode(vec)
                )?;
            }
            _ => {
                if elements.is_empty() {
                    write_with_indent!(
                        f,
                        context,
                        indent_start,
                        depth,
                        "Array<{}>()",
                        format_value_kind(element_value_kind)
                    )?;
                } else {
                    write_with_indent!(
                        f,
                        context,
                        indent_start,
                        depth,
                        "Array<{}>({}",
                        format_value_kind(element_value_kind),
                        context.get_new_line()
                    )?;
                    format_elements(f, elements, context, depth + 1)?;
                    write_with_indent!(f, context, true, depth, ")")?;
                }
            }
        },
        Value::Map {
            key_value_kind,
            value_value_kind,
            entries,
        } => {
            if entries.is_empty() {
                write_with_indent!(
                    f,
                    context,
                    indent_start,
                    depth,
                    "Map<{}, {}>()",
                    format_value_kind(key_value_kind),
                    format_value_kind(value_value_kind)
                )?;
            } else {
                write_with_indent!(
                    f,
                    context,
                    indent_start,
                    depth,
                    "Map<{}, {}>({}",
                    format_value_kind(key_value_kind),
                    format_value_kind(value_value_kind),
                    context.get_new_line()
                )?;
                format_kv_entries(f, entries, context, depth + 1)?;
                write_with_indent!(f, context, true, depth, ")")?;
            }
        }
        // custom types
        Value::Custom { value } => {
            format_custom_value(f, value, context, indent_start, depth)?;
        }
    };
    Ok(())
}

pub fn format_elements<F: fmt::Write>(
    f: &mut F,
    values: &[ManifestValue],
    context: &ManifestDecompilationDisplayContext,
    depth: usize,
) -> fmt::Result {
    for (i, x) in values.iter().enumerate() {
        format_manifest_value(f, x, context, true, depth)?;
        if i == values.len() - 1 {
            write!(f, "{}", context.get_new_line())?;
        } else {
            write!(f, ",{}", context.get_new_line())?;
        }
    }
    Ok(())
}

pub fn format_kv_entries<F: fmt::Write>(
    f: &mut F,
    entries: &[(ManifestValue, ManifestValue)],
    context: &ManifestDecompilationDisplayContext,
    depth: usize,
) -> fmt::Result {
    for (i, x) in entries.iter().enumerate() {
        format_manifest_value(f, &x.0, context, true, depth)?;
        write!(f, " => ")?;
        format_manifest_value(f, &x.1, context, false, depth)?;
        if i == entries.len() - 1 {
            write!(f, "{}", context.get_new_line())?;
        } else {
            write!(f, ",{}", context.get_new_line())?;
        }
    }
    Ok(())
}

pub fn format_custom_value<F: fmt::Write>(
    f: &mut F,
    value: &ManifestCustomValue,
    context: &ManifestDecompilationDisplayContext,
    indent_start: bool,
    depth: usize,
) -> fmt::Result {
    match value {
        ManifestCustomValue::Address(value) => match value {
            ManifestAddress::Static(node_id) => {
                write_with_indent!(
                    f,
                    context,
                    indent_start,
                    depth,
                    "Address(\"{}\")",
                    if let Some(encoder) = context.address_bech32_encoder {
                        if let Ok(bech32) = encoder.encode(node_id.as_ref()) {
                            bech32
                        } else {
                            hex::encode(node_id.as_bytes())
                        }
                    } else {
                        hex::encode(node_id.as_bytes())
                    }
                )?;
            }
            ManifestAddress::Named(address_id) => {
                write_with_indent!(
                    f,
                    context,
                    indent_start,
                    depth,
                    "NamedAddress(\"{}\")",
                    context.get_object_names().address_name(*address_id)
                )?;
            }
        },
        ManifestCustomValue::Bucket(value) => {
            write_with_indent!(
                f,
                context,
                indent_start,
                depth,
                "Bucket(\"{}\")",
                context.get_object_names().bucket_name(*value)
            )?;
        }
        ManifestCustomValue::Proof(value) => {
            write_with_indent!(
                f,
                context,
                indent_start,
                depth,
                "Proof(\"{}\")",
                context.get_object_names().proof_name(*value)
            )?;
        }
        ManifestCustomValue::AddressReservation(value) => {
            write_with_indent!(
                f,
                context,
                indent_start,
                depth,
                "AddressReservation(\"{}\")",
                context.get_object_names().address_reservation_name(*value)
            )?;
        }
        ManifestCustomValue::Expression(value) => {
            write_with_indent!(
                f,
                context,
                indent_start,
                depth,
                "Expression(\"{}\")",
                match value {
                    ManifestExpression::EntireWorktop => "ENTIRE_WORKTOP",
                    ManifestExpression::EntireAuthZone => "ENTIRE_AUTH_ZONE",
                }
            )?;
        }
        ManifestCustomValue::Blob(value) => {
            write_with_indent!(
                f,
                context,
                indent_start,
                depth,
                "Blob(\"{}\")",
                hex::encode(&value.0)
            )?;
        }
        ManifestCustomValue::Decimal(value) => {
            write_with_indent!(
                f,
                context,
                indent_start,
                depth,
                "Decimal(\"{}\")",
                to_decimal(value.clone())
            )?;
        }
        ManifestCustomValue::PreciseDecimal(value) => {
            write_with_indent!(
                f,
                context,
                indent_start,
                depth,
                "PreciseDecimal(\"{}\")",
                to_precise_decimal(value.clone())
            )?;
        }
        ManifestCustomValue::NonFungibleLocalId(value) => {
            write_with_indent!(
                f,
                context,
                indent_start,
                depth,
                "NonFungibleLocalId(\"{}\")",
                to_non_fungible_local_id(value.clone())
            )?;
        }
    }
    Ok(())
}

pub fn format_value_kind(value_kind: &ManifestValueKind) -> &str {
    match value_kind {
        ValueKind::Bool => "Bool",
        ValueKind::I8 => "I8",
        ValueKind::I16 => "I16",
        ValueKind::I32 => "I32",
        ValueKind::I64 => "I64",
        ValueKind::I128 => "I128",
        ValueKind::U8 => "U8",
        ValueKind::U16 => "U16",
        ValueKind::U32 => "U32",
        ValueKind::U64 => "U64",
        ValueKind::U128 => "U128",
        ValueKind::String => "String",
        ValueKind::Enum => "Enum",
        ValueKind::Array => "Array",
        ValueKind::Tuple => "Tuple",
        ValueKind::Map => "Map",
        ValueKind::Custom(value_kind) => match value_kind {
            ManifestCustomValueKind::Address => "Address",
            ManifestCustomValueKind::Bucket => "Bucket",
            ManifestCustomValueKind::Proof => "Proof",
            ManifestCustomValueKind::Expression => "Expression",
            ManifestCustomValueKind::Blob => "Blob",
            ManifestCustomValueKind::Decimal => "Decimal",
            ManifestCustomValueKind::PreciseDecimal => "PreciseDecimal",
            ManifestCustomValueKind::NonFungibleLocalId => "NonFungibleLocalId",
            ManifestCustomValueKind::AddressReservation => "AddressReservation",
        },
    }
}

pub fn display_value_kind(value_kind: &ManifestValueKind) -> DisplayableManifestValueKind {
    DisplayableManifestValueKind(value_kind)
}

pub struct DisplayableManifestValueKind<'a>(&'a ManifestValueKind);

impl<'a> fmt::Display for DisplayableManifestValueKind<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", format_value_kind(&self.0))
    }
}

impl<'a> ContextualDisplay<ManifestDecompilationDisplayContext<'a>> for ManifestCustomValue {
    type Error = fmt::Error;

    fn contextual_format<F: fmt::Write>(
        &self,
        f: &mut F,
        context: &ManifestDecompilationDisplayContext<'a>,
    ) -> Result<(), Self::Error> {
        format_custom_value(f, self, context, false, 0)
    }
}

pub struct ManifestCustomCharEscaper;

impl CustomCharEscaper for ManifestCustomCharEscaper {
    fn resolve_escape_behaviour(c: char) -> EscapeBehaviour {
        match c {
            '\\' => EscapeBehaviour::Replace(r#"\\"#),
            '\n' => EscapeBehaviour::Replace(r#"\n"#),
            '\r' => EscapeBehaviour::Replace(r#"\r"#),
            '\t' => EscapeBehaviour::Replace(r#"\t"#),
            '\x08' => EscapeBehaviour::Replace(r#"\b"#),
            '\x0c' => EscapeBehaviour::Replace(r#"\f"#),
            '"' => EscapeBehaviour::Replace(r#"\""#),
            _ if should_escape_unicode_char(c) => EscapeBehaviour::UnicodeEscape,
            _ => EscapeBehaviour::None,
        }
    }

    fn format_unicode_escaped_char(f: &mut impl fmt::Write, c: char) -> fmt::Result {
        radix_rust::unicode::format_json_utf16_escaped_char(f, c)
    }
}

fn should_escape_unicode_char(c: char) -> bool {
    // Per the JSON spec, we need to encode as least control characters.
    //
    // Some JSON encoders default to encoding everything that is non-ASCII.
    // But this is a bit too restrictive, as it's common for people to
    // want to use emoji or non-ASCII in metadata, and it would be nice
    // for the manifest canonical encoding to display this.
    //
    // If we try to be minimal, and just encode control characters
    // (e.g. given by `char.is_control()`), this only covers the
    // `Cc` category which misses things like the RTL override which
    // is in the `Cf` category. Such characters could mess up the display
    // of manifests, so we should exclude them.
    //
    // There are also other characters which may cause confusion in
    // text, such as grapheme extenders which can be used to add arbitrary
    // accents to characters.
    //
    // So when Rust formats debug strings, it also escapes characters
    // such as grapheme extenders, and other characters it views as
    // "non-printable". We view this as a sensible default behaviour,
    // so we follow this whne choosing to display manifest strings.
    radix_rust::unicode::rust_1_81_should_unicode_escape_in_debug_str(c)
}
