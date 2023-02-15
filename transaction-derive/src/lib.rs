mod categorize;
mod decode;
mod encode;

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
pub fn encode(input: TokenStream) -> TokenStream {
    encode::handle_encode(proc_macro2::TokenStream::from(input))
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
pub fn decode(input: TokenStream) -> TokenStream {
    decode::handle_decode(proc_macro2::TokenStream::from(input))
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
pub fn categorize(input: TokenStream) -> TokenStream {
    categorize::handle_categorize(proc_macro2::TokenStream::from(input))
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}
