mod categorize;
mod decode;
mod describe;
mod encode;
mod legacy_describe;
mod non_fungible_data;
mod scrypto_sbor;

use proc_macro::TokenStream;

/// Derive code that describes this data structure.
///
/// Note that this derive doesn't work with recursive type, such as
/// ```ignore
/// struct A {
///     array: Vec<A>
/// }
/// ```
#[proc_macro_derive(LegacyDescribe, attributes(legacy_skip))]
pub fn legacy_describe(input: TokenStream) -> TokenStream {
    legacy_describe::handle_describe(proc_macro2::TokenStream::from(input))
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
pub fn encode(input: TokenStream) -> TokenStream {
    encode::handle_encode(proc_macro2::TokenStream::from(input))
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
pub fn decode(input: TokenStream) -> TokenStream {
    decode::handle_decode(proc_macro2::TokenStream::from(input))
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
pub fn categorize(input: TokenStream) -> TokenStream {
    categorize::handle_categorize(proc_macro2::TokenStream::from(input))
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
pub fn describe(input: TokenStream) -> TokenStream {
    describe::handle_describe(proc_macro2::TokenStream::from(input))
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

/// Derive code that describe a non-fungible data structure.
///
/// # Example
///
/// ```ignore
/// use scrypto::prelude::*;
///
/// #[derive(NonFungibleData)]
/// pub struct MyStruct {
///     pub field_1: u32,
///     #[mutable]
///     pub field_2: String,
/// }
/// ```
#[proc_macro_derive(NonFungibleData, attributes(mutable))]
pub fn non_fungible_data(input: TokenStream) -> TokenStream {
    non_fungible_data::handle_non_fungible_data(proc_macro2::TokenStream::from(input))
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}
