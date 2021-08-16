use proc_macro2::TokenStream;
use quote::quote;
use syn::*;

macro_rules! trace {
    ($($arg:expr),*) => {{
        #[cfg(feature = "trace")]
        println!($($arg),*);
    }};
}

pub fn handle_describe(input: TokenStream) -> TokenStream {
    trace!("handle_describe() starts");

    let DeriveInput { ident, data, .. } = parse2(input).expect("Unable to parse input");
    let ident_str = ident.to_string();
    trace!("Describing: {}", ident);

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
                    impl ::sbor::Describe for #ident {
                        fn describe() -> ::sbor::types::Type {
                            extern crate alloc;
                            use alloc::vec::Vec;
                            use alloc::string::ToString;
                            use ::sbor::{self, Describe};

                            let mut named = Vec::new();
                            #(named.push((#names.to_string(), <#types>::describe()));)*

                            ::sbor::types::Type::Struct {
                                name: #ident_str.to_string(),
                                fields: ::sbor::types::Fields::Named { named },
                            }
                        }
                    }
                }
            }
            syn::Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                let types = unnamed.iter().map(|f| &f.ty);

                quote! {
                    impl ::sbor::Describe for #ident {
                        fn describe() -> ::sbor::types::Type {
                            extern crate alloc;
                            use alloc::string::ToString;
                            use alloc::vec::Vec;
                            use ::sbor::{self, Describe};

                            let mut unnamed = Vec::new();
                            #(unnamed.push(<#types>::describe());)*

                            ::sbor::types::Type::Struct {
                                name: #ident_str.to_string(),
                                fields: ::sbor::types::Fields::Unnamed { unnamed },
                            }
                        }
                    }
                }
            }
            syn::Fields::Unit => {
                quote! {
                    impl ::sbor::Describe for #ident {
                        fn describe() -> ::sbor::types::Type {
                            extern crate alloc;
                            use alloc::string::ToString;

                            ::sbor::types::Type::Struct {
                                name: #ident_str.to_string(),
                                fields: ::sbor::types::Fields::Unit,
                            }
                        }
                    }
                }
            }
        },
        Data::Enum(DataEnum { variants, .. }) => {
            let names = variants.iter().map(|v| v.ident.to_string());
            let fields = variants.iter().map(|v| {
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
                                let mut named = Vec::new();
                                #(named.push((#names.to_string(), <#types>::describe()));)*
                                ::sbor::types::Fields::Named {
                                    named
                                }
                            }
                        }
                    }
                    syn::Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                        let types = unnamed.iter().map(|f| &f.ty);
                        quote! {
                            {
                                let mut unnamed = Vec::new();
                                #(unnamed.push(<#types>::describe());)*
                                ::sbor::types::Fields::Unnamed {
                                    unnamed
                                }
                            }
                        }
                    }
                    syn::Fields::Unit => {
                        quote! {
                            {
                                ::sbor::types::Fields::Unit
                            }
                        }
                    }
                }
            });

            quote! {
                impl ::sbor::Describe for #ident {
                    fn describe() -> ::sbor::types::Type {
                        extern crate alloc;
                        use alloc::string::ToString;
                        use alloc::vec::Vec;
                        use ::sbor::{self, Describe};

                        let mut variants = Vec::new();
                        #(variants.push(::sbor::types::Variant {
                            name: #names.to_string(),
                            fields: #fields
                        });)*

                        ::sbor::types::Type::Enum {
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
    trace!("handle_derive() finishes");

    #[cfg(feature = "trace")]
    crate::utils::print_compiled_code("Describe", &output);

    output.into()
}

#[cfg(test)]
mod tests {
    extern crate alloc;
    use alloc::str::FromStr;

    use super::*;
    use proc_macro2::TokenStream;

    fn assert_code_eq(a: TokenStream, b: TokenStream) {
        assert_eq!(a.to_string(), b.to_string());
    }

    #[test]
    fn test_describe_struct() {
        let input = TokenStream::from_str("struct Test {a: u32}").unwrap();
        let output = handle_describe(input);

        assert_code_eq(
            output,
            quote! {
                impl ::sbor::Describe for Test {
                    fn describe() -> ::sbor::types::Type {
                        extern crate alloc;
                        use alloc::vec::Vec;
                        use alloc::string::ToString;
                        use ::sbor::{self, Describe};
                        let mut named = Vec::new();
                        named.push(("a".to_string(), <u32>::describe()));
                        ::sbor::types::Type::Struct {
                            name: "Test".to_string(),
                            fields: ::sbor::types::Fields::Named { named },
                        }
                    }
                }
            },
        );
    }

    #[test]
    fn test_describe_enum() {
        let input = TokenStream::from_str("enum Test {A, B (u32), C {x: u8}}").unwrap();
        let output = handle_describe(input);

        assert_code_eq(
            output,
            quote! {
                impl ::sbor::Describe for Test {
                    fn describe() -> ::sbor::types::Type {
                        extern crate alloc;
                        use alloc::string::ToString;
                        use alloc::vec::Vec;
                        use ::sbor::{self, Describe};
                        let mut variants = Vec::new();
                        variants.push(::sbor::types::Variant {
                            name: "A".to_string(),
                            fields: { ::sbor::types::Fields::Unit }
                        });
                        variants.push(::sbor::types::Variant {
                            name: "B".to_string(),
                            fields: {
                                let mut unnamed = Vec::new();
                                unnamed.push(<u32>::describe());
                                ::sbor::types::Fields::Unnamed { unnamed }
                            }
                        });
                        variants.push(::sbor::types::Variant {
                            name: "C".to_string(),
                            fields: {
                                let mut named = Vec::new();
                                named.push(("x".to_string(), <u8>::describe()));
                                ::sbor::types::Fields::Named { named }
                            }
                        });
                        ::sbor::types::Type::Enum {
                            name: "Test".to_string(),
                            variants,
                        }
                    }
                }
            },
        );
    }
}
