mod decode;
mod describe;
mod encode;
mod utils;

use proc_macro::TokenStream;

/// Derive code that describes this data structure.
#[proc_macro_derive(Describe)]
pub fn describe(input: TokenStream) -> TokenStream {
    let output = describe::handle_describe(proc_macro2::TokenStream::from(input));
    TokenStream::from(output)
}

/// Derive code responsible for encoding this data structure.
#[proc_macro_derive(Encode)]
pub fn encode(input: TokenStream) -> TokenStream {
    let output = encode::handle_encode(proc_macro2::TokenStream::from(input));
    TokenStream::from(output)
}

/// Derive code responsible for decoding this data structure from bytes.
#[proc_macro_derive(Decode)]
pub fn decode(input: TokenStream) -> TokenStream {
    let output = decode::handle_decode(proc_macro2::TokenStream::from(input));
    TokenStream::from(output)
}
