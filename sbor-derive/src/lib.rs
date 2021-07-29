mod describe;
mod utils;

use proc_macro::TokenStream;

#[proc_macro_derive(Describe)]
pub fn describe(input: TokenStream) -> TokenStream {
    let output = describe::handle_describe(proc_macro2::TokenStream::from(input));
    TokenStream::from(output)
}
