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

    let DeriveInput {
        ident,
        data,
        generics,
        ..
    } = parse2(input)?;

    let mut generics_for_impl = generics.clone();

    // TODO - neaten up
    // This adds the E: ::sbor::v1::Encoder bound to the impl
    {
        let span = Span::call_site();
        generics_for_impl.params.push(GenericParam::Type(TypeParam {
            attrs: vec![],
            ident: Ident::new("E", span),
            colon_token: Some(Token![:](span)),
            bounds: {
                let mut bounds = punctuated::Punctuated::new();
                bounds.push(TypeParamBound::Trait(TraitBound {
                    paren_token: None,
                    modifier: TraitBoundModifier::None,
                    lifetimes: None,
                    path: Path {
                        leading_colon: Some(Token![::](span)),
                        segments: {
                            let mut segments = punctuated::Punctuated::new();
                            segments.push(PathSegment::from(Ident::new("sbor", span)));
                            segments.push(PathSegment::from(Ident::new("v1", span)));
                            segments.push(PathSegment::from(Ident::new("Encoder", span)));
                            segments
                        }
                    }
                }));
                bounds
            },
            eq_token: None,
            default: None,
        }));
    }

    let (_, ty_generics, where_clause) = generics.split_for_impl();
    let (impl_generics, _, _) = generics_for_impl.split_for_impl();

    trace!("Encoding: {}", ident);

    let output = match data {
        Data::Struct(s) => match s.fields {
            syn::Fields::Named(FieldsNamed { named, .. }) => {
                // ns: not skipped
                let ns: Vec<&Field> = named.iter().filter(|f| !is_skipped(f)).collect();
                let ns_ids = ns.iter().map(|f| &f.ident);
                let ns_len = ns_ids.len();
                let encode_length_statement = {
                    if ns_len <= 255 {
                        let ns_len = ns_len as u8;
                        quote! {
                            encoder.write_product_type_header_u8_length(#ns_len)?;
                        }
                    } else if ns_len <= u16::MAX as usize {
                        let ns_len = ns_len as u16;
                        quote! {
                            encoder.write_product_type_header_u16_length(#ns_len)?;
                        }
                    } else {
                        return Err(Error::new(Span::call_site(), format!("More than {} fields not supported!", u16::MAX)));
                    }
                };

                quote! {
                    impl #impl_generics ::sbor::v1::Encode<E> for #ident #ty_generics #where_clause {
                        #[inline]
                        fn encode_value(&self, encoder: &mut E) -> Result<(), ::sbor::v1::EncodeError> {
                            use ::sbor::v1::*;
                            #encode_length_statement
                            #(encoder.encode(&self.#ns_ids)?;)*
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
                    impl #impl_generics ::sbor::Encode for #ident #ty_generics #where_clause {
                        #[inline]
                        fn encode_type_id(encoder: &mut ::sbor::Encoder) {
                            encoder.write_type_id(::sbor::type_id::TYPE_STRUCT);
                        }
                        #[inline]
                        fn encode_value(&self, encoder: &mut ::sbor::Encoder) {
                            use ::sbor::{self, Encode};
                            encoder.write_static_size(#ns_len);
                            #(self.#ns_indices.encode(encoder);)*
                        }
                    }
                }
            }
            syn::Fields::Unit => {
                quote! {
                    impl #impl_generics ::sbor::Encode for #ident #ty_generics #where_clause {
                        #[inline]
                        fn encode_type_id(encoder: &mut ::sbor::Encoder) {
                            encoder.write_type_id(::sbor::type_id::TYPE_STRUCT);
                        }
                        #[inline]
                        fn encode_value(&self, encoder: &mut ::sbor::Encoder) {
                            encoder.write_static_size(0);
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
                                encoder.write_variant_label(#name);
                                encoder.write_static_size(#ns_len);
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
                                encoder.write_variant_label(#name);
                                encoder.write_static_size(#ns_len);
                                #(#ns_args.encode(encoder);)*
                            }
                        }
                    }
                    syn::Fields::Unit => {
                        quote! {
                            Self::#v_id => {
                                encoder.write_variant_label(#name);
                                encoder.write_static_size(0);
                            }
                        }
                    }
                }
            });

            if match_arms.len() == 0 {
                quote! {
                    impl #impl_generics ::sbor::Encode for #ident #ty_generics #where_clause {
                        #[inline]
                        fn encode_type_id(encoder: &mut ::sbor::Encoder) {
                            encoder.write_type_id(::sbor::type_id::TYPE_ENUM);
                        }
                        #[inline]
                        fn encode_value(&self, encoder: &mut ::sbor::Encoder) {
                        }
                    }
                }
            } else {
                quote! {
                    impl #impl_generics ::sbor::Encode for #ident #ty_generics #where_clause {
                        #[inline]
                        fn encode_type_id(encoder: &mut ::sbor::Encoder) {
                            encoder.write_type_id(::sbor::type_id::TYPE_ENUM);
                        }
                        #[inline]
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
                impl <E: ::sbor::v1::Encoder> ::sbor::v1::Encode<E> for Test {
                    #[inline]
                    fn encode_value(&self, encoder: &mut E) -> Result<(), ::sbor::v1::EncodeError> {
                        use ::sbor::v1::*;
                        encoder.write_product_type_header_u8_length(1u8)?;
                        encoder.encode(&self.a)?;
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
                    fn encode_type_id(encoder: &mut ::sbor::Encoder) {
                        encoder.write_type_id(::sbor::type_id::TYPE_ENUM);
                    }
                    #[inline]
                    fn encode_value(&self, encoder: &mut ::sbor::Encoder) {
                        use ::sbor::{self, Encode};
                        match self {
                            Self::A => {
                                encoder.write_variant_label("A");
                                encoder.write_static_size(0);
                            }
                            Self::B(a0) => {
                                encoder.write_variant_label("B");
                                encoder.write_static_size(1);
                                a0.encode(encoder);
                            }
                            Self::C { x, .. } => {
                                encoder.write_variant_label("C");
                                encoder.write_static_size(1);
                                x.encode(encoder);
                            }
                        }
                    }
                }
            },
        );
    }
}
