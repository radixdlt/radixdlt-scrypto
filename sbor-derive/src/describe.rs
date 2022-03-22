use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::*;

use crate::utils::*;

macro_rules! trace {
    ($($arg:expr),*) => {{
        #[cfg(feature = "trace")]
        println!($($arg),*);
    }};
}

pub fn handle_describe(input: TokenStream) -> Result<TokenStream> {
    trace!("handle_describe() starts");

    let DeriveInput { ident, data, .. } = parse2(input)?;
    let ident_str = ident.to_string();
    trace!("Describing: {}", ident);

    let output = match data {
        Data::Struct(s) => match s.fields {
            syn::Fields::Named(FieldsNamed { named, .. }) => {
                // ns: not skipped
                let ns: Vec<&Field> = named.iter().filter(|f| !is_skipped(f)).collect();
                let names = ns.iter().map(|f| {
                    f.ident
                        .clone()
                        .expect("All fields must be named")
                        .to_string()
                });
                let types1 = ns.iter().map(|f| &f.ty);
                let types2 = ns.iter().map(|f| &f.ty);
                quote! {
                    impl ::sbor::Describe for #ident {
                        fn describe() -> ::sbor::describe::Type {
                            use ::sbor::rust::borrow::ToOwned;
                            use ::sbor::rust::vec;
                            use ::sbor::Describe;

                            #(::sbor::describe::require_no_indirection::<#types1>();)*
                            ::sbor::describe::Type::Struct {
                                name: #ident_str.to_owned(),
                                fields: ::sbor::describe::Fields::Named {
                                    named: vec![#((#names.to_owned(), <#types2>::describe())),*]
                                },
                            }
                        }
                    }
                    impl ::sbor::NoIndirection for #ident {
                    }
                }
            }
            syn::Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                let ns: Vec<&Field> = unnamed.iter().filter(|f| !is_skipped(f)).collect();
                let types1 = ns.iter().map(|f| &f.ty);
                let types2 = ns.iter().map(|f| &f.ty);
                quote! {
                    impl ::sbor::Describe for #ident {
                        fn describe() -> ::sbor::describe::Type {
                            use ::sbor::rust::borrow::ToOwned;
                            use ::sbor::rust::vec;
                            use ::sbor::Describe;

                            #(::sbor::describe::require_no_indirection::<#types1>();)*
                            ::sbor::describe::Type::Struct {
                                name: #ident_str.to_owned(),
                                fields: ::sbor::describe::Fields::Unnamed {
                                    unnamed: vec![#(<#types2>::describe()),*]
                                },
                            }
                        }
                    }
                    impl ::sbor::NoIndirection for #ident {
                    }
                }
            }
            syn::Fields::Unit => {
                quote! {
                    impl ::sbor::Describe for #ident {
                        fn describe() -> ::sbor::describe::Type {
                            use ::sbor::rust::borrow::ToOwned;

                            ::sbor::describe::Type::Struct {
                                name: #ident_str.to_owned(),
                                fields: ::sbor::describe::Fields::Unit,
                            }
                        }
                    }
                    impl ::sbor::NoIndirection for #ident {
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
                        let ns: Vec<&Field> = named.iter().filter(|f| !is_skipped(f)).collect();
                        let names = ns.iter().map(|f| {
                            f.ident
                                .clone()
                                .expect("All fields must be named")
                                .to_string()
                        });
                        let types1 = ns.iter().map(|f| &f.ty);
                        let types2 = ns.iter().map(|f| &f.ty);
                        quote! {
                            {
                                #(::sbor::describe::require_no_indirection::<#types1>();)*
                                ::sbor::describe::Fields::Named {
                                    named: vec![#((#names.to_owned(), <#types2>::describe())),*]
                                }
                            }
                        }
                    }
                    syn::Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                        let ns: Vec<&Field> = unnamed.iter().filter(|f| !is_skipped(f)).collect();
                        let types1 = ns.iter().map(|f| &f.ty);
                        let types2 = ns.iter().map(|f| &f.ty);
                        quote! {
                            {
                                #(::sbor::describe::require_no_indirection::<#types1>();)*
                                ::sbor::describe::Fields::Unnamed {
                                    unnamed: vec![#(<#types2>::describe()),*]
                                }
                            }
                        }
                    }
                    syn::Fields::Unit => {
                        quote! {
                            {
                                ::sbor::describe::Fields::Unit
                            }
                        }
                    }
                }
            });

            quote! {
                impl ::sbor::Describe for #ident {
                    fn describe() -> ::sbor::describe::Type {
                        use ::sbor::rust::borrow::ToOwned;
                        use ::sbor::rust::vec;
                        use ::sbor::Describe;

                        ::sbor::describe::Type::Enum {
                            name: #ident_str.to_owned(),
                            variants: vec![
                                #(::sbor::describe::Variant {
                                    name: #names.to_owned(),
                                    fields: #fields
                                }),*
                            ]
                        }
                    }
                }
                impl ::sbor::NoIndirection for #ident {
                }
            }
        }
        Data::Union(_) => {
            return Err(Error::new(Span::call_site(), "Union is not supported!"));
        }
    };
    trace!("handle_describe() finishes");

    #[cfg(feature = "trace")]
    crate::utils::print_generated_code("Describe", &output);

    Ok(output)
}

#[cfg(test)]
mod tests {
    use proc_macro2::TokenStream;
    use std::str::FromStr;

    use super::*;

    fn assert_code_eq(a: TokenStream, b: TokenStream) {
        assert_eq!(a.to_string(), b.to_string());
    }

    #[test]
    fn test_describe_struct() {
        let input = TokenStream::from_str("struct Test {a: u32}").unwrap();
        let output = handle_describe(input).unwrap();

        assert_code_eq(
            output,
            quote! {
                impl ::sbor::Describe for Test {
                    fn describe() -> ::sbor::describe::Type {
                        use ::sbor::rust::borrow::ToOwned;
                        use ::sbor::rust::vec;
                        use ::sbor::Describe;

                        ::sbor::describe::require_no_indirection::<u32>();
                        ::sbor::describe::Type::Struct {
                            name: "Test".to_owned(),
                            fields: ::sbor::describe::Fields::Named {
                                named: vec![("a".to_owned(), <u32>::describe())]
                            },
                        }
                    }
                }
                impl ::sbor::NoIndirection for Test {
                }
            },
        );
    }

    #[test]
    fn test_describe_enum() {
        let input = TokenStream::from_str("enum Test {A, B (u32), C {x: u8}}").unwrap();
        let output = handle_describe(input).unwrap();

        assert_code_eq(
            output,
            quote! {
                impl ::sbor::Describe for Test {
                    fn describe() -> ::sbor::describe::Type {
                        use ::sbor::rust::borrow::ToOwned;
                        use ::sbor::rust::vec;
                        use ::sbor::Describe;

                        ::sbor::describe::Type::Enum {
                            name: "Test".to_owned(),
                            variants: vec![
                                ::sbor::describe::Variant {
                                    name: "A".to_owned(),
                                    fields: { ::sbor::describe::Fields::Unit }
                                },
                                ::sbor::describe::Variant {
                                    name: "B".to_owned(),
                                    fields: {
                                        ::sbor::describe::require_no_indirection::<u32>();
                                        ::sbor::describe::Fields::Unnamed { unnamed: vec![<u32>::describe()] }
                                    }
                                },
                                ::sbor::describe::Variant {
                                    name: "C".to_owned(),
                                    fields: {
                                        ::sbor::describe::require_no_indirection::<u8>();
                                        ::sbor::describe::Fields::Named { named: vec![("x".to_owned(), <u8>::describe())] }
                                    }
                                }
                            ]
                        }
                    }
                }
                impl ::sbor::NoIndirection for Test {
                }
            },
        );
    }

    #[test]
    fn test_skip_field_1() {
        let input = TokenStream::from_str("struct Test {#[sbor(skip)] a: u32}").unwrap();
        let output = handle_describe(input).unwrap();

        assert_code_eq(
            output,
            quote! {
                impl ::sbor::Describe for Test {
                    fn describe() -> ::sbor::describe::Type {
                        use ::sbor::rust::borrow::ToOwned;
                        use ::sbor::rust::vec;
                        use ::sbor::Describe;

                        ::sbor::describe::Type::Struct {
                            name: "Test".to_owned(),
                            fields: ::sbor::describe::Fields::Named { named: vec![] },
                        }
                    }
                }
                impl ::sbor::NoIndirection for Test {
                }
            },
        );
    }

    #[test]
    fn test_skip_field_2() {
        let input =
            TokenStream::from_str("enum Test {A, B (#[sbor(skip)] u32), C {#[sbor(skip)] x: u8}}")
                .unwrap();
        let output = handle_describe(input).unwrap();

        assert_code_eq(
            output,
            quote! {
                impl ::sbor::Describe for Test {
                    fn describe() -> ::sbor::describe::Type {
                        use ::sbor::rust::borrow::ToOwned;
                        use ::sbor::rust::vec;
                        use ::sbor::Describe;

                        ::sbor::describe::Type::Enum {
                            name: "Test".to_owned(),
                            variants: vec![
                                ::sbor::describe::Variant {
                                    name: "A".to_owned(),
                                    fields: { ::sbor::describe::Fields::Unit }
                                },
                                ::sbor::describe::Variant {
                                    name: "B".to_owned(),
                                    fields: {
                                        ::sbor::describe::Fields::Unnamed { unnamed: vec![] }
                                    }
                                },
                                ::sbor::describe::Variant {
                                    name: "C".to_owned(),
                                    fields: {
                                        ::sbor::describe::Fields::Named { named: vec![] }
                                    }
                                }
                            ]
                        }
                    }
                }
                impl ::sbor::NoIndirection for Test {
                }
            },
        );
    }
}
