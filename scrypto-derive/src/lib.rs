use proc_macro::TokenStream;
use quote::quote;

/// Defines a new component.
#[proc_macro]
pub fn component(_input: TokenStream) -> TokenStream {
    let output = quote! {};

    output.into()
}

/// Imports the ABI of a component.
#[proc_macro]
pub fn import_abi(_input: TokenStream) -> TokenStream {
    let output = quote! {};

    output.into()
}
