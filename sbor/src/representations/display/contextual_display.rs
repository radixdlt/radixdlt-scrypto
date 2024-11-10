use super::*;
use crate::representations::*;
use crate::rust::prelude::*;
use crate::traversal::*;
use crate::*;
use radix_rust::*;

#[derive(Debug, Clone, Copy)]
pub enum PrintMode {
    SingleLine,
    MultiLine {
        indent_size: usize,
        base_indent: usize,
        first_line_indent: usize,
    },
}

/// The display mode chooses how the value is displayed
#[derive(Debug, Clone, Copy)]
pub enum DisplayMode {
    /// RustLike - takes inspiration from Rust and other programming languages.
    ///
    /// With `RustLikeOptions::full()` we get:
    ///   - Struct: `TypeName { field1: X, }`
    ///   - Array: `[value1, value2]`
    ///   - Map: `{ key1 => value1 }`
    ///   - Enum: `Name::Variant`, `Name::Variant(value1)`, `Name::Variant { field1: value1 }`
    ///
    /// With `RustLikeOptions::debug()` we don't get the Enum type names, and values are
    /// formatted more like their Debug representation.
    RustLike(RustLikeOptions),
    /// ==RustLike is recommended over NestedString. This may be deprecated soon==
    /// NestedString - is somewhat like the Manifest format, eg:
    ///   - Struct: `Tuple:TypeName(field1 = X)`
    ///   - Array: `Array<X>(value1, value2)`
    ///   - Map: `Map<KeyKind:TypeName, ValueKind>(key1, value1)`
    ///   - Enum: `Enum:TypeName(0:VariantName, field1, field2)`
    NestedString,
}

#[derive(Debug, Clone, Copy)]
pub enum ValueDisplayParameters<'s, 'a, E: FormattableCustomExtension> {
    Schemaless {
        display_mode: DisplayMode,
        print_mode: PrintMode,
        custom_context: E::CustomDisplayContext<'a>,
        depth_limit: usize,
    },
    Annotated {
        display_mode: DisplayMode,
        print_mode: PrintMode,
        custom_context: E::CustomDisplayContext<'a>,
        schema: &'s Schema<E::CustomSchema>,
        type_id: LocalTypeId,
        depth_limit: usize,
    },
}

enum Context<'s, 'a, E: FormattableCustomExtension> {
    Nested(NestedStringDisplayContext<'s, 'a, E>, LocalTypeId, usize),
    RustLike(RustLikeDisplayContext<'s, 'a, E>, LocalTypeId, usize),
}

impl<'s, 'a, E: FormattableCustomExtension> ValueDisplayParameters<'s, 'a, E> {
    fn get_context_and_type_id(&self) -> Context<'s, 'a, E> {
        match self {
            Self::Schemaless {
                display_mode: DisplayMode::NestedString,
                print_mode,
                custom_context,
                depth_limit,
            } => Context::Nested(
                NestedStringDisplayContext {
                    schema: E::CustomSchema::empty_schema(),
                    print_mode: *print_mode,
                    custom_context: *custom_context,
                },
                LocalTypeId::any(),
                *depth_limit,
            ),
            Self::Annotated {
                display_mode: DisplayMode::NestedString,
                print_mode,
                custom_context,
                schema,
                type_id,
                depth_limit,
            } => Context::Nested(
                NestedStringDisplayContext {
                    schema: *schema,
                    print_mode: *print_mode,
                    custom_context: *custom_context,
                },
                *type_id,
                *depth_limit,
            ),
            Self::Schemaless {
                display_mode: DisplayMode::RustLike(options),
                print_mode,
                custom_context,
                depth_limit,
            } => Context::RustLike(
                RustLikeDisplayContext {
                    schema: E::CustomSchema::empty_schema(),
                    print_mode: *print_mode,
                    custom_context: *custom_context,
                    options: *options,
                },
                LocalTypeId::any(),
                *depth_limit,
            ),
            Self::Annotated {
                display_mode: DisplayMode::RustLike(options),
                print_mode,
                custom_context,
                schema,
                type_id,
                depth_limit,
            } => Context::RustLike(
                RustLikeDisplayContext {
                    schema: *schema,
                    print_mode: *print_mode,
                    custom_context: *custom_context,
                    options: *options,
                },
                *type_id,
                *depth_limit,
            ),
        }
    }
}

#[derive(Debug, Clone)]
pub enum FormattingError {
    Fmt(fmt::Error),
    Sbor(String),
}

impl<'s, 'a, 'b, E: FormattableCustomExtension> ContextualDisplay<ValueDisplayParameters<'s, 'a, E>>
    for RawPayload<'b, E>
{
    type Error = FormattingError;

    fn contextual_format<F: fmt::Write>(
        &self,
        f: &mut F,
        options: &ValueDisplayParameters<'s, 'a, E>,
    ) -> Result<(), Self::Error> {
        let context = options.get_context_and_type_id();
        match context {
            Context::Nested(context, type_id, depth_limit) => format_payload_as_nested_string(
                f,
                &context,
                self.payload_bytes(),
                type_id,
                depth_limit,
            ),
            Context::RustLike(context, type_id, depth_limit) => format_payload_as_rustlike_value(
                f,
                &context,
                self.payload_bytes(),
                type_id,
                depth_limit,
            ),
        }
    }
}

impl<'s, 'a, 'b, E: FormattableCustomExtension> ContextualDisplay<ValueDisplayParameters<'s, 'a, E>>
    for RawValue<'b, E>
{
    type Error = FormattingError;

    fn contextual_format<F: fmt::Write>(
        &self,
        f: &mut F,
        options: &ValueDisplayParameters<'s, 'a, E>,
    ) -> Result<(), Self::Error> {
        let context = options.get_context_and_type_id();
        match context {
            Context::Nested(context, type_id, depth_limit) => {
                format_partial_payload_as_nested_string(
                    f,
                    self.value_body_bytes(),
                    ExpectedStart::ValueBody(self.value_kind()),
                    true,
                    0,
                    &context,
                    type_id,
                    depth_limit,
                )
            }
            Context::RustLike(context, type_id, depth_limit) => {
                format_partial_payload_as_rustlike_value(
                    f,
                    self.value_body_bytes(),
                    ExpectedStart::ValueBody(self.value_kind()),
                    true,
                    0,
                    &context,
                    type_id,
                    depth_limit,
                )
            }
        }
    }
}
