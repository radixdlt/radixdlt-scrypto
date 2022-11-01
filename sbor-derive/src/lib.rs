mod decode;
mod encode;
mod type_id;
mod utils;

use proc_macro::TokenStream;

/// Derive code that returns the type ID.
#[proc_macro_derive(TypeId, attributes(custom_type_id))]
pub fn type_id(input: TokenStream) -> TokenStream {
    type_id::handle_type_id(proc_macro2::TokenStream::from(input))
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Derive code that encodes this data structure
#[proc_macro_derive(Encode, attributes(skip, custom_type_id))]
pub fn encode(input: TokenStream) -> TokenStream {
    encode::handle_encode(proc_macro2::TokenStream::from(input))
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Derive code that decodes this data structure from a byte array.
#[proc_macro_derive(Decode, attributes(skip, custom_type_id))]
pub fn decode(input: TokenStream) -> TokenStream {
    decode::handle_decode(proc_macro2::TokenStream::from(input))
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}
