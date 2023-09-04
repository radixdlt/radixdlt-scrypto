use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use radix_engine_interface::types::PackageAddress;

/// Objects of this struct are generated as part of the generation process. This struct can then be
/// used inside the quote macro and printed out.
pub struct BlueprintStub {
    pub blueprint_name: String,
    pub invocation_fn_signature_items: Vec<InvocationFn>,
}

impl BlueprintStub {
    pub fn to_token_stream(&self, package_address: PackageAddress) -> TokenStream {
        let package_address_bytes = package_address.to_vec();
        let blueprint_name = self.blueprint_name.clone();
        let owned_blueprint_name = format!("Owned{}", self.blueprint_name);
        let global_blueprint_name = format!("Global{}", self.blueprint_name);

        let blueprint_name_ident = Ident::new(&self.blueprint_name, Span::call_site());
        let blueprint_functions_name_ident = Ident::new(
            format!("{}Functions", &self.blueprint_name).as_str(),
            Span::call_site(),
        );

        let functions = self
            .invocation_fn_signature_items
            .iter()
            .filter(|func| matches!(func.fn_type, FnType::Function))
            .collect::<Vec<_>>();
        let methods = self
            .invocation_fn_signature_items
            .iter()
            .filter(|func| matches!(func.fn_type, FnType::Method { .. }))
            .collect::<Vec<_>>();

        quote! {
            extern_blueprint_internal! {
                PackageAddress::new_or_panic([ #(#package_address_bytes),* ]),
                #blueprint_name_ident,
                #blueprint_name,
                #owned_blueprint_name,
                #global_blueprint_name,
                #blueprint_functions_name_ident
                {
                    #(#functions;)*
                },
                {
                    #(#methods;)*
                }
            }
        }
    }
}

pub struct InvocationFn {
    pub ident: syn::Ident,
    pub inputs: Vec<(syn::Ident, proc_macro2::TokenStream)>,
    pub output: proc_macro2::TokenStream,
    pub fn_type: FnType,
}

pub enum FnType {
    Function,
    Method { is_mutable_receiver: bool },
}

impl ToTokens for InvocationFn {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ident = &self.ident;
        let input_names = &self.inputs.iter().map(|(k, _)| k).collect::<Vec<_>>();
        let input_types = &self.inputs.iter().map(|(_, v)| v).collect::<Vec<_>>();
        let output = &self.output;

        let sep = if self.inputs.is_empty() {
            quote! {}
        } else {
            quote! {,}
        };

        match self.fn_type {
            FnType::Function => quote! {
                fn #ident( #( #input_names: #input_types ),* ) -> #output
            }
            .to_tokens(tokens),
            FnType::Method {
                is_mutable_receiver: true,
            } => quote! {
                fn #ident( &mut self #sep #( #input_names: #input_types ),* ) -> #output
            }
            .to_tokens(tokens),
            FnType::Method {
                is_mutable_receiver: false,
            } => quote! {
                fn #ident( &self #sep #( #input_names: #input_types ),* ) -> #output
            }
            .to_tokens(tokens),
        }
    }
}
