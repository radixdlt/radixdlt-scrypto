mod ast;
mod component;
mod describe;
mod import;
mod utils;

use proc_macro::TokenStream;

/// Define a new component.
#[proc_macro]
pub fn component(input: TokenStream) -> TokenStream {
    component::handle_component(input)
}

/// Import a blueprint.
#[proc_macro]
pub fn import(input: TokenStream) -> TokenStream {
    import::handle_import(input)
}

/// Describe a struct or enum
#[proc_macro_derive(Describe)]
pub fn describe(input: TokenStream) -> TokenStream {
    describe::handle_describe(input)
}
