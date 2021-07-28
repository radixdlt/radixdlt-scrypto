use proc_macro::{self, TokenStream};
use quote::quote;
use syn::*;

use crate::utils::*;

macro_rules! trace {
    ($($arg:expr),*) => {{
        #[cfg(feature = "trace")]
        println!($($arg),*);
    }};
}

pub fn handle_describe(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, data, .. } = parse_macro_input!(input);
    let ident_str = ident.to_string();
    trace!("Describing {:?}", &ident);

    let output = match data {
        Data::Struct(s) => match s.fields {
            syn::Fields::Named(FieldsNamed { named, .. }) => {
                let names = named.iter().map(|f| {
                    f.ident
                        .clone()
                        .expect("All fields must be named")
                        .to_string()
                });
                let types = named.iter().map(|f| &f.ty);

                quote! {
                    impl scrypto::abi::Describe for #ident {
                        fn describe() -> scrypto::abi::Type {
                            extern crate alloc;
                            use alloc::collections::BTreeMap;
                            use alloc::string::ToString;
                            use scrypto::abi::{self, Describe};

                            let mut fields = BTreeMap::new();
                            #(fields.insert(#names.to_string(), #types::describe());)*

                            abi::Type::Struct {
                                name: #ident_str.to_string(),
                                fields: abi::Fields::Named { fields },
                            }
                        }
                    }
                }
            }
            syn::Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                let types = unnamed.iter().map(|f| &f.ty);

                quote! {
                    impl scrypto::abi::Describe for #ident {
                        fn describe() -> scrypto::abi::Type {
                            extern crate alloc;
                            use alloc::string::ToString;
                            use alloc::vec::Vec;
                            use scrypto::abi::{self, Describe};

                            let mut fields = Vec::new();
                            #(fields.push(#types::describe());)*

                            abi::Type::Struct {
                                name: #ident_str.to_string(),
                                fields: abi::Fields::Unnamed { fields },
                            }
                        }
                    }
                }
            }
            syn::Fields::Unit => {
                quote! {
                    impl scrypto::abi::Describe for #ident {
                        fn describe() -> scrypto::abi::Type {
                            scrypto::abi::Type::Struct {
                                name: #ident_str.to_string(),
                                fields: scrypto::abi::Fields::Unit,
                            }
                        }
                    }
                }
            }
        },
        Data::Enum(DataEnum { variants, .. }) => {
            let names = variants.iter().map(|v| v.ident.to_string());
            let types = variants.iter().map(|v| {
                let f = &v.fields;

                match f {
                    syn::Fields::Named(FieldsNamed { named, .. }) => {
                        let names = named.iter().map(|f| {
                            f.ident
                                .clone()
                                .expect("All fields must be named")
                                .to_string()
                        });
                        let types = named.iter().map(|f| &f.ty);
                        quote! {
                            {
                                let mut fields = BTreeMap::new();
                                #(fields.insert(#names.to_string(), #types::describe());)*
                                abi::Fields::Named {
                                    fields,
                                }
                            }
                        }
                    }
                    syn::Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                        let types = unnamed.iter().map(|f| &f.ty);
                        quote! {
                            {
                                let mut fields = Vec::new();
                                #(fields.push(#types::describe());)*
                                abi::Fields::Unnamed {
                                    fields,
                                }
                            }
                        }
                    }
                    syn::Fields::Unit => {
                        quote! {
                            {
                                abi::Fields::Unit
                            }
                        }
                    }
                }
            });

            quote! {
                impl scrypto::abi::Describe for #ident {
                    fn describe() -> scrypto::abi::Type {
                        extern crate alloc;
                        use alloc::collections::BTreeMap;
                        use alloc::string::ToString;
                        use alloc::vec::Vec;
                        use scrypto::abi::{self, Describe};

                        let mut variants = BTreeMap::new();
                        #(variants.insert(#names.to_string(), #types);)*

                        abi::Type::Enum {
                            name: #ident_str.to_string(),
                            variants,
                        }
                    }
                }
            }
        }
        Data::Union(_) => {
            panic!("Union is not supported!")
        }
    };

    print_compiled_code("#[derive(Describe)]", &output);

    output.into()
}
