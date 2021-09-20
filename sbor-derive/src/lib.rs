mod decode;
mod describe;
mod encode;
mod type_id;
mod utils;

use proc_macro::TokenStream;

/// Derive code that describes this data structure.
#[proc_macro_derive(Describe, attributes(sbor))]
pub fn describe(input: TokenStream) -> TokenStream {
    let output = describe::handle_describe(proc_macro2::TokenStream::from(input));
    TokenStream::from(output)
}

/// Derive code that returns the type ID.
#[proc_macro_derive(TypeId, attributes(sbor))]
pub fn type_id(input: TokenStream) -> TokenStream {
    let output = type_id::handle_type_id(proc_macro2::TokenStream::from(input));
    TokenStream::from(output)
}

/// Derive code that encodes this data structure
#[proc_macro_derive(Encode, attributes(sbor))]
pub fn encode(input: TokenStream) -> TokenStream {
    let output = encode::handle_encode(proc_macro2::TokenStream::from(input));
    TokenStream::from(output)
}

/// Derive code that decodes this data structure from a byte array.
#[proc_macro_derive(Decode, attributes(sbor))]
pub fn decode(input: TokenStream) -> TokenStream {
    let output = decode::handle_decode(proc_macro2::TokenStream::from(input));
    TokenStream::from(output)
}
