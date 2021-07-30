mod describe;
mod encode;
mod utils;

use proc_macro::TokenStream;

#[proc_macro_derive(Describe)]
pub fn describe(input: TokenStream) -> TokenStream {
    let output = describe::handle_describe(proc_macro2::TokenStream::from(input));
    TokenStream::from(output)
}

#[proc_macro_derive(Encode)]
pub fn encode(input: TokenStream) -> TokenStream {
    let output = encode::handle_encode(proc_macro2::TokenStream::from(input));
    TokenStream::from(output)
}
