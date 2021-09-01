use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::*;

use crate::utils::*;

macro_rules! trace {
    ($($arg:expr),*) => {{
        #[cfg(feature = "trace")]
        println!($($arg),*);
    }};
}

pub fn handle_encode(input: TokenStream) -> TokenStream {
    trace!("handle_encode() starts");

    let DeriveInput { ident, data, .. } = parse2(input).expect("Unable to parse input");
    trace!("Encoding: {}", ident);

    let output = match data {
        Data::Struct(s) => match s.fields {
            syn::Fields::Named(FieldsNamed { named, .. }) => {
                // ns: not skipped
                let ns: Vec<&Field> = named.iter().filter(|f| !is_skipped(f)).collect();
                let ns_n = Index::from(ns.len());
                let ns_ids = ns.iter().map(|f| &f.ident);
                quote! {
                    impl ::sbor::Encode for #ident {
                        fn encode_value(&self, encoder: &mut ::sbor::Encoder) {
                            use ::sbor::{self, Encode};
                            encoder.write_type(::sbor::constants::TYPE_FIELDS_NAMED);
                            encoder.write_len(#ns_n);
                            #(
                                self.#ns_ids.encode(encoder);
                            )*
                        }
                        #[inline]
                        fn type_id() -> u8 {
                            ::sbor::constants::TYPE_STRUCT
                        }
                    }
                }
            }
            syn::Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                let mut ns_idx = Vec::new();
                for (i, f) in unnamed.iter().enumerate() {
                    if !is_skipped(f) {
                        ns_idx.push(Index::from(i));
                    }
                }
                let ns_n = Index::from(ns_idx.len());
                quote! {
                    impl ::sbor::Encode for #ident {
                        fn encode_value(&self, encoder: &mut ::sbor::Encoder) {
                            use ::sbor::{self, Encode};
                            encoder.write_type(::sbor::constants::TYPE_FIELDS_UNNAMED);
                            encoder.write_len(#ns_n);
                            #(self.#ns_idx.encode(encoder);)*
                        }
                        #[inline]
                        fn type_id() -> u8 {
                            ::sbor::constants::TYPE_STRUCT
                        }
                    }
                }
            }
            syn::Fields::Unit => {
                quote! {
                    impl ::sbor::Encode for #ident {
                        fn encode_value(&self, encoder: &mut ::sbor::Encoder) {
                            encoder.write_type(::sbor::constants::TYPE_FIELDS_UNIT);
                        }
                        #[inline]
                        fn type_id() -> u8 {
                            ::sbor::constants::TYPE_STRUCT
                        }
                    }
                }
            }
        },
        Data::Enum(DataEnum { variants, .. }) => {
            let match_arms = variants.iter().enumerate().map(|(i, v)| {
                let v_ith = Index::from(i);
                let v_id = &v.ident;
                match &v.fields {
                    syn::Fields::Named(FieldsNamed { named, .. }) => {
                        let ns: Vec<&Field> = named.iter().filter(|f| !is_skipped(f)).collect();
                        let ns_ids = ns.iter().map(|f| &f.ident);
                        let ns_ids2 = ns.iter().map(|f| &f.ident);
                        let ns_n = Index::from(ns.len());
                        quote! {
                            Self::#v_id {#(#ns_ids,)* ..} => {
                                encoder.write_u8(#v_ith);
                                encoder.write_type(::sbor::constants::TYPE_FIELDS_NAMED);
                                encoder.write_len(#ns_n);
                                #(
                                    #ns_ids2.encode(encoder);
                                )*
                            }
                        }
                    }
                    syn::Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                        let all_args = (0..unnamed.len()).map(|i| format_ident!("a{}", i));
                        let mut ns_args = Vec::<Ident>::new();
                        for (i, f) in unnamed.iter().enumerate() {
                            if !is_skipped(f) {
                                ns_args.push(format_ident!("a{}", i));
                            }
                        }
                        let ns_n = Index::from(ns_args.len());
                        quote! {
                            Self::#v_id (#(#all_args),*) => {
                                encoder.write_u8(#v_ith);
                                encoder.write_type(::sbor::constants::TYPE_FIELDS_UNNAMED);
                                encoder.write_len(#ns_n);
                                #(#ns_args.encode(encoder);)*
                            }
                        }
                    }
                    syn::Fields::Unit => {
                        quote! {
                            Self::#v_id => {
                                encoder.write_u8(#v_ith);
                                encoder.write_type(::sbor::constants::TYPE_FIELDS_UNIT);
                            }
                        }
                    }
                }
            });

            quote! {
                impl ::sbor::Encode for #ident {
                    fn encode_value(&self, encoder: &mut ::sbor::Encoder) {
                        use ::sbor::{self, Encode};

                        match self {
                            #(#match_arms)*
                        }
                    }
                    #[inline]
                    fn type_id() -> u8 {
                        ::sbor::constants::TYPE_ENUM
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
    crate::utils::print_compiled_code("Encode", &output);

    output.into()
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
    fn test_encode_struct() {
        let input = TokenStream::from_str("struct Test {a: u32}").unwrap();
        let output = handle_encode(input);

        assert_code_eq(
            output,
            quote! {
                impl ::sbor::Encode for Test {
                    fn encode_value(&self, encoder: &mut ::sbor::Encoder) {
                        use ::sbor::{self, Encode};
                        encoder.write_type(::sbor::constants::TYPE_FIELDS_NAMED);
                        encoder.write_len(1);
                        self.a.encode(encoder);
                    }
                    #[inline]
                    fn type_id() -> u8 {
                        ::sbor::constants::TYPE_STRUCT
                    }
                }
            },
        );
    }

    #[test]
    fn test_encode_enum() {
        let input = TokenStream::from_str("enum Test {A, B (u32), C {x: u8}}").unwrap();
        let output = handle_encode(input);

        assert_code_eq(
            output,
            quote! {
                impl ::sbor::Encode for Test {
                    fn encode_value(&self, encoder: &mut ::sbor::Encoder) {
                        use ::sbor::{self, Encode};
                        match self {
                            Self::A => {
                                encoder.write_u8(0);
                                encoder.write_type(::sbor::constants::TYPE_FIELDS_UNIT);
                            }
                            Self::B(a0) => {
                                encoder.write_u8(1);
                                encoder.write_type(::sbor::constants::TYPE_FIELDS_UNNAMED);
                                encoder.write_len(1);
                                a0.encode(encoder);
                            }
                            Self::C { x, .. } => {
                                encoder.write_u8(2);
                                encoder.write_type(::sbor::constants::TYPE_FIELDS_NAMED);
                                encoder.write_len(1);
                                x.encode(encoder);
                            }
                        }
                    }
                    #[inline]
                    fn type_id() -> u8 {
                        ::sbor::constants::TYPE_ENUM
                    }
                }
            },
        );
    }
}
