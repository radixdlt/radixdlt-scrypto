use proc_macro::TokenStream;
use std::str::FromStr;

/// Derive code that returns the value kind.
#[proc_macro_derive(Categorize, attributes(sbor))]
pub fn categorize(input: TokenStream) -> TokenStream {
    sbor_derive_common::categorize::handle_categorize(proc_macro2::TokenStream::from(input), None)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Derive code that encodes this data structure
#[proc_macro_derive(Encode, attributes(sbor))]
pub fn encode(input: TokenStream) -> TokenStream {
    sbor_derive_common::encode::handle_encode(proc_macro2::TokenStream::from(input), None)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Derive code that decodes this data structure from a byte array.
#[proc_macro_derive(Decode, attributes(sbor))]
pub fn decode(input: TokenStream) -> TokenStream {
    sbor_derive_common::decode::handle_decode(proc_macro2::TokenStream::from(input), None)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Derive code that describes this type.
#[proc_macro_derive(Describe, attributes(sbor))]
pub fn describe(input: TokenStream) -> TokenStream {
    sbor_derive_common::describe::handle_describe(proc_macro2::TokenStream::from(input), None)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// A shortcut for [`Categorize`], [`Encode`], [`Decode`], and [`Describe`] derives.
///
#[proc_macro_derive(Sbor, attributes(sbor))]
pub fn sbor(input: TokenStream) -> TokenStream {
    sbor_derive_common::sbor::handle_sbor(proc_macro2::TokenStream::from(input), None, None)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// An empty derive which exists solely to allow the helper "sbor" attribute
/// to be used without generating a compile error.
///
/// The intended use-case is as a utility for building other macros,
/// which wish to add sbor attribute annotations to types for when they do
/// use an Sbor derive - but wish to avoid the following error when they don't:
/// ```text
/// error: cannot find attribute `sbor` in this scope
/// ```
///
/// Ideally this would output an empty token stream, but instead we
/// return a simply comment, to avoid the proc macro system thinking
/// the macro build has broken and returning this error:
/// ```text
/// proc macro `PermitSborAttributes` not expanded: internal error
/// ```
#[proc_macro_derive(PermitSborAttributes, attributes(sbor))]
pub fn permit_sbor_attributes(_: TokenStream) -> TokenStream {
    TokenStream::from_str(&"// Empty PermitSborAttributes expansion").unwrap()
}

const BASIC_CUSTOM_VALUE_KIND: &str = "sbor::NoCustomValueKind";
const BASIC_CUSTOM_TYPE_KIND: &str = "sbor::NoCustomTypeKind";
const BASIC_CUSTOM_SCHEMA: &str = "sbor::NoCustomSchema";

/// Derive code that returns the value kind - specifically for Basic SBOR.
#[proc_macro_derive(BasicCategorize, attributes(sbor))]
pub fn basic_categorize(input: TokenStream) -> TokenStream {
    sbor_derive_common::categorize::handle_categorize(
        proc_macro2::TokenStream::from(input),
        Some(BASIC_CUSTOM_VALUE_KIND),
    )
    .unwrap_or_else(|err| err.to_compile_error())
    .into()
}

/// Derive code that encodes this data structure - specifically for Basic SBOR.
#[proc_macro_derive(BasicEncode, attributes(sbor))]
pub fn basic_encode(input: TokenStream) -> TokenStream {
    sbor_derive_common::encode::handle_encode(
        proc_macro2::TokenStream::from(input),
        Some(BASIC_CUSTOM_VALUE_KIND),
    )
    .unwrap_or_else(|err| err.to_compile_error())
    .into()
}

/// Derive code that decodes this data structure from a byte array - specifically for Basic SBOR.
#[proc_macro_derive(BasicDecode, attributes(sbor))]
pub fn basic_decode(input: TokenStream) -> TokenStream {
    sbor_derive_common::decode::handle_decode(
        proc_macro2::TokenStream::from(input),
        Some(BASIC_CUSTOM_VALUE_KIND),
    )
    .unwrap_or_else(|err| err.to_compile_error())
    .into()
}

/// Derive code that describes the type - specifically for Basic SBOR.
#[proc_macro_derive(BasicDescribe, attributes(sbor))]
pub fn basic_describe(input: TokenStream) -> TokenStream {
    sbor_derive_common::describe::handle_describe(
        proc_macro2::TokenStream::from(input),
        Some(BASIC_CUSTOM_TYPE_KIND),
    )
    .unwrap_or_else(|err| err.to_compile_error())
    .into()
}

/// A shortcut for [`BasicCategorize`], [`BasicEncode`], [`BasicDecode`], and [`BasicDescribe`] derives.
///
#[proc_macro_derive(BasicSbor, attributes(sbor))]
pub fn basic_sbor(input: TokenStream) -> TokenStream {
    sbor_derive_common::sbor::handle_sbor(
        proc_macro2::TokenStream::from(input),
        Some(BASIC_CUSTOM_VALUE_KIND),
        Some(BASIC_CUSTOM_TYPE_KIND),
    )
    .unwrap_or_else(|err| err.to_compile_error())
    .into()
}

/// A macro for outputting tests and marker traits to assert that a type has maintained its shape over time.
///
/// There are two types of assertion modes:
/// * `fixed` mode is used to ensure that a type is unchanged.
/// * `backwards_compatible` mode is used when multiple versions of the type are permissible, but
///   newer versions of the type must be compatible with the older version where they align.
///   This mode (A) ensures that the type's current schema is equivalent to the latest version, and
///   (B) ensures that each of the schemas is a strict extension of the previous mode.
///
/// ## Initial schema generation and regeneration
///
/// To output a generated schema, temporarily add a `generate` parameter or a `regenerate` parameter,
/// after the `fixed` or `backwards_compatible` parameter, and then run the created test.
/// If using Rust Analyzer this can be run from the IDE, or it can be run via `cargo test`.
///
/// To protect against accidentally doing the wrong thing, `generate` can only be used for initial generation,
/// whereas `regenerate` can only be used for replacing an existing generation.
///
/// If a "FILE:.." path is specified, it will (re)generate that file, else it will output to the console:
/// * In `fixed` mode, this will (re)generate against the given schema location.
/// * In `backwards_compatible` mode, this will (re)generate against the latest schema location (the last in the list).
///
/// The test will then panic to ensure it fails, and can't be left accidentally in (re)generate state.
///
/// ```ignore
/// # // Ignored because the generated code references sbor which can't be imported
/// # // by the doctest framework, because it doesn't know what those crates are
/// # extern crate sbor_derive;
/// # use sbor_derive::*;
/// #
/// #[derive(BasicSbor, BasicSborAssertion)]
/// #[sbor_assert(fixed("FILE:MyType-schema-v1.txt"), generate)]
/// struct MyType {
///     // ...
/// }
/// ```
///
/// ## Fixed schema verification
///
/// To verify the type's schema is unchanged, do:
/// ```ignore
/// # // Ignored because the generated code references sbor which can't be imported
/// # // by the doctest framework, because it doesn't know what those crates are
/// # extern crate sbor_derive;
/// # use sbor_derive::*;
/// #
/// #[derive(BasicSbor, BasicSborAssertion)]
/// #[sbor_assert(fixed("FILE:MyType-schema-v1.txt"))]
/// struct MyType {
///     // ...
/// }
/// ```
///
/// Instead of `"FILE:X"`, you can also use `"INLINE:<hex>"`, `"CONST:<Constant>"` or `"EXPR:<Expression>"`
/// where the expression (such as `generate_schema()`) has to generate a `SingleTypeSchema<NoCustomSchema>`.
///
/// ## Backwards compatibility verification
///
/// To allow multiple backwards-compatible versions, you can do this:
/// ```ignore
/// # // Ignored because the generated code references sbor which can't be imported
/// # // by the doctest framework, because it doesn't know what those crates are
/// # extern crate sbor_derive;
/// # use sbor_derive::*;
/// #
/// #[derive(BasicSbor, BasicSborAssertion)]
/// #[sbor_assert(backwards_compatible(
///     version1 = "FILE:MyType-schema-v1.txt",
///     version2 = "FILE:MyType-schema-v2.txt",
/// ))]
/// struct MyType {
///     // ...
/// }
/// ```
///
/// Instead of `"FILE:X"`, you can also use `"INLINE:<hex>"`, `"CONST:<Constant>"` or `"EXPR:<Expression>"`
/// where the expression (such as `generate_schema()`) has to generate a `SingleTypeSchema<NoCustomSchema>`.
///
/// If you wish to configure exactly which schemas are used for comparison of the current schema with
/// the latest named schema; and each named schema with its predecessor, you can use:
///
/// ```ignore
/// # // Ignored because the generated code references sbor which can't be imported
/// # // by the doctest framework, because it doesn't know what those crates are
/// # extern crate sbor_derive;
/// # use sbor_derive::*;
/// #
/// #[sbor_assert(backwards_compatible("EXPR:<Expression>"))
/// ```
/// Where the expression (such as `params_builder()`) has to generate a `SingleTypeSchemaCompatibilityParameters<NoCustomSchema>`.
///
/// ## Custom settings
/// By default, the `fixed` mode will use `SchemaComparisonSettings::require_equality()` and
/// the `backwards_compatible` mode will use `SchemaComparisonSettings::require_equality()` for the check
/// of `current` aginst the latest version, and `SchemaComparisonSettings::allow_extension()` for the
/// checks between consecutive versions.
///
/// You may wish to change these:
/// * If you just wish to ignore the equality of metadata such as names, you can use the
///   `allow_name_changes` flag.
/// * If you wish to override all settings, you can provide a constant containing your
///   own SchemaComparisonSettings.
/// * If you wish to specify a builder for settings, you can provide `"EXPR:|builder| builder.<stuff>"`
/// * If for `backwards_compatible`, you wish to provide a separate configuration for the "latest" and
///   "named versions" checks, you can use `settings(comparison_between_versions = \"EXPR:F1\", comparison_between_current_and_latest = \"EXPR:F2\") `
///    
///
/// For example:
/// ```ignore
/// # // Ignored because the generated code references sbor which can't be imported
/// # // by the doctest framework, because it doesn't know what those crates are
/// # extern crate sbor_derive;
/// # use sbor_derive::*;
/// #
/// #[derive(BasicSbor, BasicSborAssertion)]
/// #[sbor_assert(
///     fixed("FILE:MyType-schema-v1.txt"),
///     settings(allow_name_changes),
/// )]
/// struct MyType {
///     // ...
/// }
///
/// #[derive(BasicSbor, BasicSborAssertion)]
/// #[sbor_assert(
///     backwards_compatible(
///         v1 = "FILE:MyType-schema-v1.txt",
///         v2 = "FILE:MyType-schema-v2.txt",
///     ),
///     settings(
///         // We allow name changes between versions, but require the current schema to exactly match
///         // the latest version (v2 in this case).
///         // This could be useful to e.g. ensure that we have a fixed schema with the latest naming available.
///         comparison_between_versions = "EXPR: |s| s.allow_all_name_changes()",
///         comparison_between_current_and_latest = "EXPR: |s| s",
///     ),
/// )]
/// struct MyType {
///     // ...
/// }
/// ```
#[proc_macro_derive(BasicSborAssertion, attributes(sbor_assert))]
pub fn basic_sbor_assertion(input: TokenStream) -> TokenStream {
    sbor_derive_common::sbor_assert::handle_sbor_assert_derive(
        proc_macro2::TokenStream::from(input),
        BASIC_CUSTOM_SCHEMA,
    )
    .unwrap_or_else(|err| err.to_compile_error())
    .into()
}
