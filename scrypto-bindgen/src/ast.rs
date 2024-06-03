use proc_macro2::*;
use quote::*;
use radix_common::prelude::*;

use crate::token_stream_from_str;

pub struct PackageStub {
    pub blueprints: Vec<BlueprintStub>,
    pub auxiliary_types: Vec<AuxiliaryType>,
}

impl ToTokens for PackageStub {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let blueprints = &self.blueprints;
        let auxiliary_types = &self.auxiliary_types;
        quote! {
            #(#blueprints)*
            #(#auxiliary_types)*
        }
        .to_tokens(tokens)
    }
}

/// Objects of this struct are generated as part of the generation process. This struct can then be
/// used inside the quote macro and printed out.
pub struct BlueprintStub {
    pub blueprint_name: String,
    pub fn_signatures: Vec<FnSignature>,
    pub package_address: PackageAddress,
}

impl ToTokens for BlueprintStub {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let package_address_bytes = self.package_address.to_vec();
        let blueprint_name = self.blueprint_name.clone();
        let owned_blueprint_name = format!("Owned{}", self.blueprint_name);
        let global_blueprint_name = format!("Global{}", self.blueprint_name);

        let blueprint_name_ident = Ident::new(&self.blueprint_name, Span::call_site());
        let blueprint_functions_name_ident = Ident::new(
            format!("{}Functions", &self.blueprint_name).as_str(),
            Span::call_site(),
        );

        let functions = self
            .fn_signatures
            .iter()
            .filter(|func| matches!(func.fn_type, FnType::Function))
            .collect::<Vec<_>>();
        let methods = self
            .fn_signatures
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
        .to_tokens(tokens)
    }
}

pub struct FnSignature {
    pub ident: syn::Ident,
    pub inputs: Vec<(syn::Ident, TokenStream)>,
    pub output: TokenStream,
    pub fn_type: FnType,
}

pub enum FnType {
    Function,
    Method { is_mutable_receiver: bool },
}

impl ToTokens for FnSignature {
    fn to_tokens(&self, tokens: &mut TokenStream) {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuxiliaryType {
    TupleStruct {
        struct_name: String,
        field_types: Vec<String>,
    },
    NamedFieldsStruct {
        struct_name: String,
        fields: IndexMap<String, String>,
    },
    Enum {
        enum_name: String,
        variants: Vec<EnumVariant>,
    },
}

impl ToTokens for AuxiliaryType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::TupleStruct {
                struct_name,
                field_types,
            } => {
                let struct_name = token_stream_from_str!(struct_name);
                let field_types = field_types
                    .iter()
                    .map(|string| token_stream_from_str!(string));

                quote! {
                    #[derive(::scrypto::prelude::ScryptoSbor)]
                    pub struct #struct_name(
                        #(
                            #field_types
                        ),*
                    );
                }
                .to_tokens(tokens)
            }
            Self::NamedFieldsStruct {
                struct_name,
                fields,
            } => {
                let struct_name = token_stream_from_str!(struct_name);
                let field_names = fields.keys().map(|string| token_stream_from_str!(string));
                let field_types = fields.values().map(|string| token_stream_from_str!(string));

                quote! {
                    #[derive(::scrypto::prelude::ScryptoSbor)]
                    pub struct #struct_name {
                        #(
                            #field_names: #field_types
                        ),*
                    }
                }
                .to_tokens(tokens)
            }
            Self::Enum {
                enum_name,
                variants,
            } => {
                let enum_name = token_stream_from_str!(enum_name);

                quote! {
                    #[derive(::scrypto::prelude::ScryptoSbor)]
                    pub enum #enum_name {
                        #(
                            #variants
                        ),*
                    }
                }
                .to_tokens(tokens)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnumVariant {
    Unit {
        variant_name: String,
        variant_index: u8,
    },
    Tuple {
        variant_name: String,
        variant_index: u8,
        field_types: Vec<String>,
    },
    NamedFields {
        variant_name: String,
        variant_index: u8,
        fields: IndexMap<String, String>,
    },
}

impl ToTokens for EnumVariant {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Unit {
                variant_name,
                variant_index,
            } => {
                let variant_name = token_stream_from_str!(variant_name);
                quote! {
                    #[sbor(discriminator( #variant_index ))]
                    #variant_name
                }
                .to_tokens(tokens)
            }
            Self::Tuple {
                variant_name,
                variant_index,
                field_types,
            } => {
                let variant_name = token_stream_from_str!(variant_name);
                let field_types = field_types
                    .iter()
                    .map(|string| token_stream_from_str!(string));
                quote! {
                    #[sbor(discriminator( #variant_index ))]
                    #variant_name (
                        #(
                            #field_types
                        ),*
                    )
                }
                .to_tokens(tokens)
            }
            Self::NamedFields {
                variant_name,
                variant_index,
                fields,
            } => {
                let variant_name = token_stream_from_str!(variant_name);
                let field_names = fields.keys().map(|string| token_stream_from_str!(string));
                let field_types = fields.values().map(|string| token_stream_from_str!(string));

                quote! {
                    #[sbor(discriminator( #variant_index ))]
                    #variant_name {
                        #(
                            #field_names: #field_types
                        ),*
                    }
                }
                .to_tokens(tokens)
            }
        }
    }
}
