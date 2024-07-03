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
/// use manifest::prelude::*;
///
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
/// use manifest::prelude::*;
///
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
/// use manifest::prelude::*;
///
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
///
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
/// use scrypto::prelude::*;
///
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
/// use scrypto::prelude::*;
///
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
/// use scrypto::prelude::*;
///
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
/// use scrypto::prelude::*;
///
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
#[proc_macro_derive(ScryptoSbor, attributes(sbor))]
pub fn scrypto_sbor(input: TokenStream) -> TokenStream {
    scrypto_sbor::handle_scrypto_sbor(proc_macro2::TokenStream::from(input))
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// A macro for outputting tests and marker traits to assert that a type has maintained its shape over time.
///
/// There are two types of assertion modes:
/// * "fixed" mode is used to ensure that a type is unchanged.
/// * "backwards_compatible" mode is used when multiple versions of the type are permissible, but
///   newer versions of the type must be compatible with the older version where they align.
///   This mode (A) ensures that the type's current schema is equivalent to the latest version, and
///   (B) ensures that each of the schemas is a strict extension of the previous mode.
///
/// There is also a "generate" mode which can be used to export the current schema. Upon running the generated test,
/// the schema is either written to a file, or output in a panic message.
///
/// ## Initial schema generation
///
/// To output the generated schema to a file path relative to the source file, add an attribute like this -
/// and then run the test which gets generated. If using Rust Analyzer this can be run from the IDE,
/// or it can be run via `cargo test`.
///
/// ```no_run
/// #[derive(ScryptoSbor, ScryptoSborAssertion)]
/// #[sbor_assert(generate("FILE:MyType-schema-v1.txt"))]
/// struct MyType {
///     // ...
/// }
/// ```
///
/// The test should generate the given file and then panic. If you wish to only generate the schema
/// in the panic message, you can with `#[sbor_assert(generate("INLINE"))]`.
///
/// ## Fixed schema verification
///
/// To verify the type's schema is unchanged, do:
/// ```no_run
/// #[derive(ScryptoSbor, ScryptoSborAssertion)]
/// #[sbor_assert(fixed("FILE:MyType-schema-v1.txt"))]
/// struct MyType {
///     // ...
/// }
/// ```
///
/// Other supported options are `fixed("INLINE:<hex>")` and `fixed("CONST:<Constant>")`.
///
/// ## Backwards compatibility verification
///
/// To allow multiple backwards-compatible versions, you can do this:
/// ```no_run
/// #[derive(ScryptoSbor, ScryptoSborAssertion)]
/// #[sbor_assert(backwards_compatible(
///     version1 = "FILE:MyType-schema-v1.txt",
///     version2 = "FILE:MyType-schema-v2.txt",
/// ))]
/// struct MyType {
///     // ...
/// }
/// ```
///
/// Instead of `"FILE:X"`, you can also use `"INLINE:<hex>"` and `"CONST:<Constant>"`.
///
/// ## Custom settings
/// By default, the `fixed` mode will use `SchemaComparisonSettings::require_equality()` and
/// the `backwards_compatible` mode will use `SchemaComparisonSettings::allow_extension()`.
///
/// You may wish to change these:
/// * If you just wish to ignore the equality of metadata such as names, you can use the
///   `allow_name_changes` flag.
/// * If you wish to override any settings, you can provide a constant containing your
///   own SchemaComparisonSettings.
///
/// For example:
/// ```no_run
/// #[derive(ScryptoSbor, ScryptoSborAssertion)]
/// #[sbor_assert(
///     fixed("FILE:MyType-schema-v1.txt"),
///     settings(allow_name_changes),
/// )]
/// struct MyType {
///     // ...
/// }
///
/// const CUSTOM_COMPARISON_SETTINGS: sbor::schema::SchemaComparisonSettings = sbor::schema::SchemaComparisonSettings::require_equality();
///
/// #[derive(ScryptoSbor, ScryptoSborAssertion)]
/// #[sbor_assert(
///     backwards_compatible(
///         version1 = "FILE:MyType-schema-v1.txt",
///         version2 = "FILE:MyType-schema-v2.txt",
///     ),
///     settings(CUSTOM_COMPARISON_SETTINGS),
/// )]
/// struct MyOtherType {
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
/// use scrypto::prelude::*;
///
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
