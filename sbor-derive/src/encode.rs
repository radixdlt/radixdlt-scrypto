use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::*;

use crate::utils::*;

macro_rules! trace {
    ($($arg:expr),*) => {{
        #[cfg(feature = "trace")]
        println!($($arg),*);
    }};
}

pub fn handle_encode(input: TokenStream) -> Result<TokenStream> {
    trace!("handle_encode() starts");

    let DeriveInput { ident, data, .. } = parse2(input)?;
    trace!("Encoding: {}", ident);

    let output = match data {
        Data::Struct(s) => match s.fields {
            syn::Fields::Named(FieldsNamed { named, .. }) => {
                // ns: not skipped
                let ns: Vec<&Field> = named.iter().filter(|f| !is_skipped(f)).collect();
                let ns_ids = ns.iter().map(|f| &f.ident);
                let ns_len = Index::from(ns_ids.len());
                quote! {
                    impl ::sbor::Encode for #ident {
                        #[inline]
                        fn encode_type(&self, encoder: &mut ::sbor::Encoder) {
                            use ::sbor::TypeId;
                            encoder.write_type(Self::type_id());
                        }
                        fn encode_value(&self, encoder: &mut ::sbor::Encoder) {
                            use ::sbor::{self, Encode};
                            encoder.write_len(#ns_len);
                            #(self.#ns_ids.encode(encoder);)*
                        }
                    }
                }
            }
            syn::Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                let mut ns_indices = Vec::new();
                for (i, f) in unnamed.iter().enumerate() {
                    if !is_skipped(f) {
                        ns_indices.push(Index::from(i));
                    }
                }
                let ns_len = Index::from(ns_indices.len());
                quote! {
                    impl ::sbor::Encode for #ident {
                        #[inline]
                        fn encode_type(&self, encoder: &mut ::sbor::Encoder) {
                            use ::sbor::TypeId;
                            encoder.write_type(Self::type_id());
                        }
                        fn encode_value(&self, encoder: &mut ::sbor::Encoder) {
                            use ::sbor::{self, Encode};
                            encoder.write_len(#ns_len);
                            #(self.#ns_indices.encode(encoder);)*
                        }
                    }
                }
            }
            syn::Fields::Unit => {
                quote! {
                    impl ::sbor::Encode for #ident {
                        #[inline]
                        fn encode_type(&self, encoder: &mut ::sbor::Encoder) {
                            use ::sbor::TypeId;
                            encoder.write_type(Self::type_id());
                        }
                        fn encode_value(&self, encoder: &mut ::sbor::Encoder) {
                            encoder.write_len(0);
                        }
                    }
                }
            }
        },
        Data::Enum(DataEnum { variants, .. }) => {
            let match_arms = variants.iter().map(|v| {
                let v_id = &v.ident;
                let name_string = v_id.to_string();
                let name: Expr = parse_quote! { #name_string };

                match &v.fields {
                    syn::Fields::Named(FieldsNamed { named, .. }) => {
                        let ns: Vec<&Field> = named.iter().filter(|f| !is_skipped(f)).collect();
                        let ns_ids = ns.iter().map(|f| &f.ident);
                        let ns_ids2 = ns.iter().map(|f| &f.ident);
                        let ns_len = Index::from(ns.len());
                        quote! {
                            Self::#v_id {#(#ns_ids,)* ..} => {
                                #name.to_string().encode_value(encoder);
                                encoder.write_len(#ns_len);
                                #(#ns_ids2.encode(encoder);)*
                            }
                        }
                    }
                    syn::Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                        let args = (0..unnamed.len()).map(|i| format_ident!("a{}", i));
                        let mut ns_args = Vec::<Ident>::new();
                        for (i, f) in unnamed.iter().enumerate() {
                            if !is_skipped(f) {
                                ns_args.push(format_ident!("a{}", i));
                            }
                        }
                        let ns_len = Index::from(ns_args.len());
                        quote! {
                            Self::#v_id (#(#args),*) => {
                                #name.to_string().encode_value(encoder);
                                encoder.write_len(#ns_len);
                                #(#ns_args.encode(encoder);)*
                            }
                        }
                    }
                    syn::Fields::Unit => {
                        quote! {
                            Self::#v_id => {
                                #name.to_string().encode_value(encoder);
                                encoder.write_len(0);
                            }
                        }
                    }
                }
            });

            if match_arms.len() == 0 {
                quote! {
                    impl ::sbor::Encode for #ident {
                        #[inline]
                        fn encode_type(&self, encoder: &mut ::sbor::Encoder) {
                            use ::sbor::TypeId;
                            encoder.write_type(Self::type_id());
                        }
                        fn encode_value(&self, encoder: &mut ::sbor::Encoder) {
                        }
                    }
                }
            } else {
                quote! {
                    impl ::sbor::Encode for #ident {
                        #[inline]
                        fn encode_type(&self, encoder: &mut ::sbor::Encoder) {
                            use ::sbor::TypeId;
                            encoder.write_type(Self::type_id());
                        }
                        fn encode_value(&self, encoder: &mut ::sbor::Encoder) {
                            use ::sbor::{self, Encode};

                            match self {
                                #(#match_arms)*
                            }
                        }
                    }
                }
            }
        }
        Data::Union(_) => {
            return Err(Error::new(Span::call_site(), "Union is not supported!"));
        }
    };
    trace!("handle_encode() finishes");

    #[cfg(feature = "trace")]
    crate::utils::print_generated_code("Encode", &output);

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
    fn test_encode_struct() {
        let input = TokenStream::from_str("struct Test {a: u32}").unwrap();
        let output = handle_encode(input).unwrap();

        assert_code_eq(
            output,
            quote! {
                impl ::sbor::Encode for Test {
                    #[inline]
                    fn encode_type(&self, encoder: &mut ::sbor::Encoder) {
                        use ::sbor::TypeId;
                        encoder.write_type(Self::type_id());
                    }
                    fn encode_value(&self, encoder: &mut ::sbor::Encoder) {
                        use ::sbor::{self, Encode};
                        encoder.write_len(1);
                        self.a.encode(encoder);
                    }
                }
            },
        );
    }

    #[test]
    fn test_encode_enum() {
        let input = TokenStream::from_str("enum Test {A, B (u32), C {x: u8}}").unwrap();
        let output = handle_encode(input).unwrap();

        assert_code_eq(
            output,
            quote! {
                impl ::sbor::Encode for Test {
                    #[inline]
                    fn encode_type(&self, encoder: &mut ::sbor::Encoder) {
                        use ::sbor::TypeId;
                        encoder.write_type(Self::type_id());
                    }
                    fn encode_value(&self, encoder: &mut ::sbor::Encoder) {
                        use ::sbor::{self, Encode};
                        match self {
                            Self::A => {
                                "A".to_string().encode_value(encoder);
                                encoder.write_len(0);
                            }
                            Self::B(a0) => {
                                "B".to_string().encode_value(encoder);
                                encoder.write_len(1);
                                a0.encode(encoder);
                            }
                            Self::C { x, .. } => {
                                "C".to_string().encode_value(encoder);
                                encoder.write_len(1);
                                x.encode(encoder);
                            }
                        }
                    }
                }
            },
        );
    }
}
