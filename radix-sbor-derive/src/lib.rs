mod manifest_categorize;
mod manifest_decode;
mod manifest_encode;
mod manifest_sbor;
mod scrypto_categorize;
mod scrypto_decode;
mod scrypto_describe;
mod scrypto_encode;
mod scrypto_event;
mod scrypto_sbor;

use proc_macro::TokenStream;

/// Derives code for encoding a struct or enum with Manifest value model.
///
/// # Example
///
/// ```ignore
/// # // Ignored because the generated code references sbor and radix-common which can't be imported
/// # // by the doctest framework, because it doesn't know what those crates are
/// # extern crate radix_sbor_derive;
/// # use radix_sbor_derive::*;
/// #
/// #[derive(ManifestEncode)]
/// pub struct MyStruct {
///     pub field_1: u32,
///     pub field_2: String,
/// }
/// ```
#[proc_macro_derive(ManifestEncode, attributes(sbor))]
pub fn manifest_encode(input: TokenStream) -> TokenStream {
    manifest_encode::handle_manifest_encode(proc_macro2::TokenStream::from(input))
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Derives code for decoding a struct or enum with Manifest value model.
///
/// # Example
///
/// ```ignore
/// # // Ignored because the generated code references sbor and radix-common which can't be imported
/// # // by the doctest framework, because it doesn't know what those crates are
/// # extern crate radix_sbor_derive;
/// # use radix_sbor_derive::*;
/// #
/// #[derive(ManifestDecode)]
/// pub struct MyStruct {
///     pub field_1: u32,
///     pub field_2: String,
/// }
/// ```
#[proc_macro_derive(ManifestDecode, attributes(sbor))]
pub fn manifest_decode(input: TokenStream) -> TokenStream {
    manifest_decode::handle_manifest_decode(proc_macro2::TokenStream::from(input))
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Derives code for categorizing a struct or enum with Manifest value model.
///
/// # Example
///
/// ```ignore
/// # // Ignored because the generated code references sbor and radix-common which can't be imported
/// # // by the doctest framework, because it doesn't know what those crates are
/// # extern crate radix_sbor_derive;
/// # use radix_sbor_derive::*;
/// #
/// #[derive(ManifestCategorize)]
/// pub struct MyStruct {
///     pub field_1: u32,
///     pub field_2: String,
/// }
/// ```
#[proc_macro_derive(ManifestCategorize, attributes(sbor))]
pub fn manifest_categorize(input: TokenStream) -> TokenStream {
    manifest_categorize::handle_manifest_categorize(proc_macro2::TokenStream::from(input))
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// A shortcut for [`ManifestCategorize`], [`ManifestEncode`] and [`ManifestDecode`] derives.
#[proc_macro_derive(ManifestSbor, attributes(sbor))]
pub fn manifest_sbor(input: TokenStream) -> TokenStream {
    manifest_sbor::handle_manifest_sbor(proc_macro2::TokenStream::from(input))
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Derives code for encoding a struct or enum with Scrypto value model.
///
/// # Example
///
/// ```ignore
/// # // Ignored because the generated code references sbor and radix-common which can't be imported
/// # // by the doctest framework, because it doesn't know what those crates are
/// # extern crate radix_sbor_derive;
/// # use radix_sbor_derive::*;
/// #
/// #[derive(ScryptoEncode)]
/// pub struct MyStruct {
///     pub field_1: u32,
///     pub field_2: String,
/// }
/// ```
#[proc_macro_derive(ScryptoEncode, attributes(sbor))]
pub fn scrypto_encode(input: TokenStream) -> TokenStream {
    scrypto_encode::handle_scrypto_encode(proc_macro2::TokenStream::from(input))
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Derives code for decoding a struct or enum with Scrypto value model.
///
/// # Example
///
/// ```ignore
/// # // Ignored because the generated code references sbor and radix-common which can't be imported
/// # // by the doctest framework, because it doesn't know what those crates are
/// # extern crate radix_sbor_derive;
/// # use radix_sbor_derive::*;
/// #
/// #[derive(ScryptoDecode)]
/// pub struct MyStruct {
///     pub field_1: u32,
///     pub field_2: String,
/// }
/// ```
#[proc_macro_derive(ScryptoDecode, attributes(sbor))]
pub fn scrypto_decode(input: TokenStream) -> TokenStream {
    scrypto_decode::handle_scrypto_decode(proc_macro2::TokenStream::from(input))
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Derives code for categorizing a struct or enum with Scrypto value model.
///
/// # Example
///
/// ```ignore
/// # // Ignored because the generated code references sbor and radix-common which can't be imported
/// # // by the doctest framework, because it doesn't know what those crates are
/// # extern crate radix_sbor_derive;
/// # use radix_sbor_derive::*;
/// #
/// #[derive(ScryptoCategorize)]
/// pub struct MyStruct {
///     pub field_1: u32,
///     pub field_2: String,
/// }
/// ```
#[proc_macro_derive(ScryptoCategorize, attributes(sbor))]
pub fn scrypto_categorize(input: TokenStream) -> TokenStream {
    scrypto_categorize::handle_scrypto_categorize(proc_macro2::TokenStream::from(input))
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Derives code for describing a struct or enum with Scrypto schema.
///
/// # Example
///
/// ```ignore
/// # // Ignored because the generated code references sbor and radix-common which can't be imported
/// # // by the doctest framework, because it doesn't know what those crates are
/// # extern crate radix_sbor_derive;
/// # use radix_sbor_derive::*;
/// #
/// #[derive(ScryptoDescribe)]
/// pub struct MyStruct {
///     pub field_1: u32,
///     pub field_2: String,
/// }
/// ```
#[proc_macro_derive(ScryptoDescribe, attributes(sbor))]
pub fn scrypto_describe(input: TokenStream) -> TokenStream {
    scrypto_describe::handle_scrypto_describe(proc_macro2::TokenStream::from(input))
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// A shortcut for [`ScryptoCategorize`], [`ScryptoEncode`], [`ScryptoDecode`], and [`ScryptoDescribe`] derives.
///
/// # Example
///
/// ```ignore
/// # // Ignored because the generated code references sbor and radix-common which can't be imported
/// # // by the doctest framework, because it doesn't know what those crates are
/// # extern crate radix_sbor_derive;
/// # use radix_sbor_derive::*;
/// #
/// #[derive(ScryptoSbor)]
/// pub struct MyStruct {
///     pub field_1: u32,
///     pub field_2: String,
/// }
/// ```
#[proc_macro_derive(ScryptoSbor, attributes(sbor))]
pub fn scrypto_sbor(input: TokenStream) -> TokenStream {
    scrypto_sbor::handle_scrypto_sbor(proc_macro2::TokenStream::from(input))
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
/// # // Ignored because the generated code references sbor and radix-common which can't be imported
/// # // by the doctest framework, because it doesn't know what those crates are
/// # extern crate radix_sbor_derive;
/// # use radix_sbor_derive::*;
/// #
/// #[derive(ScryptoSbor, ScryptoSborAssertion)]
/// #[sbor_assert(fixed("FILE:MyType-schema-v1.bin"), generate)]
/// struct MyType {
///     // ...
/// }
/// ```
///
/// ## Fixed schema verification
///
/// To verify the type's schema is unchanged, do:
/// ```ignore
/// # // Ignored because the generated code references sbor and radix-common which can't be imported
/// # // by the doctest framework, because it doesn't know what those crates are
/// # extern crate radix_sbor_derive;
/// # use radix_sbor_derive::*;
/// #
/// #[derive(ScryptoSbor, ScryptoSborAssertion)]
/// #[sbor_assert(fixed("FILE:MyType-schema-v1.bin"))]
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
/// # // Ignored because the generated code references sbor and radix-common which can't be imported
/// # // by the doctest framework, because it doesn't know what those crates are
/// # extern crate radix_sbor_derive;
/// # use radix_sbor_derive::*;
/// #
/// #[derive(ScryptoSbor, ScryptoSborAssertion)]
/// #[sbor_assert(backwards_compatible(
///     version1 = "FILE:MyType-schema-v1.bin",
///     version2 = "FILE:MyType-schema-v2.bin",
/// ))]
/// struct MyType {
///     // ...
/// }
/// ```
///
/// Instead of `"FILE:X.bin"`, you can also use `"FILE:X.txt"`, `"INLINE:<hex>"`, `"CONST:<Constant>"` or `"EXPR:<Expression>"`
/// where the expression (such as `generate_schema()`) has to generate a `SingleTypeSchema<ScryptoCustomSchema>`.
///
/// If you wish to configure exactly which schemas are used for comparison of the current schema with
/// the latest named schema; and each named schema with its predecessor, you can use:
///
/// ```ignore
/// # // Ignored because the generated code references sbor and radix-common which can't be imported
/// # // by the doctest framework, because it doesn't know what those crates are
/// # extern crate radix_sbor_derive;
/// # use radix_sbor_derive::*;
/// #
/// #[derive(ScryptoSbor, ScryptoSborAssertion)]
/// #[sbor_assert(backwards_compatible("EXPR:<Expression>"))
/// struct MyType {
///     // ...
/// }
/// ```
/// Where the expression (such as `params_builder()`) has to generate a `SingleTypeSchemaCompatibilityParameters<ScryptoCustomSchema>`.
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
/// # // Ignored because the generated code references sbor and radix-common which can't be imported
/// # // by the doctest framework, because it doesn't know what those crates are
/// # extern crate radix_sbor_derive;
/// # use radix_sbor_derive::*;
/// #
/// #[derive(ScryptoSbor, ScryptoSborAssertion)]
/// #[sbor_assert(
///     fixed("FILE:MyType-schema-v1.bin"),
///     settings(allow_name_changes),
/// )]
/// struct MyType {
///     // ...
/// }
///
/// #[derive(ScryptoSbor, ScryptoSborAssertion)]
/// #[sbor_assert(
///     backwards_compatible(
///         v1 = "FILE:MyType-schema-v1.bin",
///         v2 = "FILE:MyType-schema-v2.bin",
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
#[proc_macro_derive(ScryptoSborAssertion, attributes(sbor_assert))]
pub fn scrypto_sbor_assertion(input: TokenStream) -> TokenStream {
    sbor_derive_common::sbor_assert::handle_sbor_assert_derive(
        proc_macro2::TokenStream::from(input),
        "radix_common::data::scrypto::ScryptoCustomSchema",
    )
    .unwrap_or_else(|err| err.to_compile_error())
    .into()
}

/// Derive code for implementing the required logic to mark a type as being an event.
///
/// # Example
///
/// ```ignore
/// # // Ignored because the generated code references sbor and radix-common which can't be imported
/// # // by the doctest framework, because it doesn't know what those crates are
/// # extern crate radix_sbor_derive;
/// # use radix_sbor_derive::*;
/// #
/// #[derive(ScryptoEvent)]
/// pub struct MyStruct {
///     pub field_1: u32,
///     pub field_2: String,
/// }
/// ```
#[proc_macro_derive(ScryptoEvent)]
pub fn scrypto_event(input: TokenStream) -> TokenStream {
    scrypto_event::handle_scrypto_event(proc_macro2::TokenStream::from(input))
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}
