mod ast;
mod component;
mod import;
mod utils;

use proc_macro::TokenStream;

/// Define a new component.
#[proc_macro]
pub fn component(input: TokenStream) -> TokenStream {
    let output = component::handle_component(proc_macro2::TokenStream::from(input), true, false);
    TokenStream::from(output)
}

/// Import a blueprint.
#[proc_macro]
pub fn import(input: TokenStream) -> TokenStream {
    let output = import::handle_import(proc_macro2::TokenStream::from(input));
    TokenStream::from(output)
}
