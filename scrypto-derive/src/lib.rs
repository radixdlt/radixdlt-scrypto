mod abi;
mod ast;
mod component;
mod import;

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
