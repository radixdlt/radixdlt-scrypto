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

/// Derive code that implements `ManifestCategorize`, `ManifestEncode` and `ManifestDecode` traits for this struct or enum.
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

/// Derive code that implements `ScryptoCategorize`, `ScryptoEncode`, `ScryptoDecode`, and `ScryptoDescribe` traits for this struct or enum.
///
#[proc_macro_derive(ScryptoSbor, attributes(sbor))]
pub fn scrypto_sbor(input: TokenStream) -> TokenStream {
    scrypto_sbor::handle_scrypto_sbor(proc_macro2::TokenStream::from(input))
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
