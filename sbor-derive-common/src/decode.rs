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

pub fn handle_decode(
    input: TokenStream,
    context_custom_value_kind: Option<&'static str>,
) -> Result<TokenStream> {
    trace!("handle_decode() starts");

    let DeriveInput {
        attrs,
        ident,
        data,
        generics,
        ..
    } = parse2(input)?;
    let (impl_generics, ty_generics, where_clause, custom_value_kind_generic, decoder_generic) =
        build_decode_generics(&generics, &attrs, context_custom_value_kind)?;

    let output = match data {
        Data::Struct(s) => match s.fields {
            syn::Fields::Named(FieldsNamed { named, .. }) => {
                // ns: not skipped, s: skipped
                let ns: Vec<&Field> = named.iter().filter(|f| !is_decoding_skipped(f)).collect();
                let ns_len = Index::from(ns.len());
                let ns_ids = ns.iter().map(|f| &f.ident);
                let ns_types = ns.iter().map(|f| &f.ty);
                let s: Vec<&Field> = named.iter().filter(|f| is_decoding_skipped(f)).collect();
                let s_ids = s.iter().map(|f| &f.ident);
                let s_types = s.iter().map(|f| &f.ty);
                quote! {
                    impl #impl_generics ::sbor::Decode <#custom_value_kind_generic, #decoder_generic> for #ident #ty_generics #where_clause {
                        #[inline]
                        fn decode_body_with_value_kind(decoder: &mut #decoder_generic, value_kind: ::sbor::ValueKind<#custom_value_kind_generic>) -> Result<Self, ::sbor::DecodeError> {
                            use ::sbor::{self, Decode};
                            decoder.check_preloaded_value_kind(value_kind, ::sbor::ValueKind::Tuple)?;
                            decoder.read_and_check_size(#ns_len)?;
                            Ok(Self {
                                #(#ns_ids: decoder.decode::<#ns_types>()?,)*
                                #(#s_ids: <#s_types>::default()),*
                            })
                        }
                    }
                }
            }
            syn::Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                let mut fields = Vec::<Expr>::new();
                for f in &unnamed {
                    let ty = &f.ty;
                    if is_decoding_skipped(f) {
                        fields.push(parse_quote! {<#ty>::default()})
                    } else {
                        fields.push(parse_quote! {decoder.decode::<#ty>()?})
                    }
                }
                let ns_len =
                    Index::from(unnamed.iter().filter(|f| !is_decoding_skipped(f)).count());
                quote! {
                    impl #impl_generics ::sbor::Decode <#custom_value_kind_generic, #decoder_generic> for #ident #ty_generics #where_clause {
                        #[inline]
                        fn decode_body_with_value_kind(decoder: &mut #decoder_generic, value_kind: ::sbor::ValueKind<#custom_value_kind_generic>) -> Result<Self, ::sbor::DecodeError> {
                            use ::sbor::{self, Decode};
                            decoder.check_preloaded_value_kind(value_kind, ::sbor::ValueKind::Tuple)?;
                            decoder.read_and_check_size(#ns_len)?;
                            Ok(Self (
                                #(#fields,)*
                            ))
                        }
                    }
                }
            }
            syn::Fields::Unit => {
                quote! {
                    impl #impl_generics ::sbor::Decode <#custom_value_kind_generic, #decoder_generic> for #ident #ty_generics #where_clause {
                        #[inline]
                        fn decode_body_with_value_kind(decoder: &mut #decoder_generic, value_kind: ::sbor::ValueKind<#custom_value_kind_generic>) -> Result<Self, ::sbor::DecodeError> {
                            decoder.check_preloaded_value_kind(value_kind, ::sbor::ValueKind::Tuple)?;
                            decoder.read_and_check_size(0)?;
                            Ok(Self {})
                        }
                    }
                }
            }
        },
        Data::Enum(DataEnum { variants, .. }) => {
            let match_arms = variants.iter().enumerate().map(|(i, v)| {
                let v_id = &v.ident;
                let i: u8 = i.try_into().expect("Too many variants found in enum");
                let discriminator: Expr = parse_quote! { #i };

                match &v.fields {
                    syn::Fields::Named(FieldsNamed { named, .. }) => {
                        let ns: Vec<&Field> =
                            named.iter().filter(|f| !is_decoding_skipped(f)).collect();
                        let ns_len = Index::from(ns.len());
                        let ns_ids = ns.iter().map(|f| &f.ident);
                        let ns_types = ns.iter().map(|f| &f.ty);
                        let s: Vec<&Field> =
                            named.iter().filter(|f| is_decoding_skipped(f)).collect();
                        let s_ids = s.iter().map(|f| &f.ident);
                        let s_types = s.iter().map(|f| &f.ty);
                        quote! {
                            #discriminator => {
                                decoder.read_and_check_size(#ns_len)?;
                                Ok(Self::#v_id {
                                    #(#ns_ids: decoder.decode::<#ns_types>()?,)*
                                    #(#s_ids: <#s_types>::default(),)*
                                })
                            }
                        }
                    }
                    syn::Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                        let mut fields = Vec::<Expr>::new();
                        for f in unnamed {
                            let ty = &f.ty;
                            if is_decoding_skipped(f) {
                                fields.push(parse_quote! {<#ty>::default()})
                            } else {
                                fields.push(parse_quote! {decoder.decode::<#ty>()?})
                            }
                        }
                        let ns_len =
                            Index::from(unnamed.iter().filter(|f| !is_decoding_skipped(f)).count());
                        quote! {
                            #discriminator => {
                                decoder.read_and_check_size(#ns_len)?;
                                Ok(Self::#v_id (
                                    #(#fields),*
                                ))
                            }
                        }
                    }
                    syn::Fields::Unit => {
                        quote! {
                            #discriminator => {
                                decoder.read_and_check_size(0)?;
                                Ok(Self::#v_id)
                            }
                        }
                    }
                }
            });

            quote! {
                impl #impl_generics ::sbor::Decode <#custom_value_kind_generic, #decoder_generic> for #ident #ty_generics #where_clause {
                    #[inline]
                    fn decode_body_with_value_kind(decoder: &mut #decoder_generic, value_kind: ::sbor::ValueKind<#custom_value_kind_generic>) -> Result<Self, ::sbor::DecodeError> {
                        use ::sbor::{self, Decode};
                        decoder.check_preloaded_value_kind(value_kind, ::sbor::ValueKind::Enum)?;
                        let discriminator = decoder.read_discriminator()?;
                        match discriminator {
                            #(#match_arms,)*
                            _ => Err(::sbor::DecodeError::UnknownDiscriminator(discriminator))
                        }
                    }
                }
            }
        }
        Data::Union(_) => {
            return Err(Error::new(Span::call_site(), "Union is not supported!"));
        }
    };

    #[cfg(feature = "trace")]
    crate::utils::print_generated_code("Decode", &output);

    trace!("handle_decode() finishes");
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
    fn test_decode_struct() {
        let input = TokenStream::from_str("struct Test {a: u32}").unwrap();
        let output = handle_decode(input, None).unwrap();

        assert_code_eq(
            output,
            quote! {
                impl <D: ::sbor::Decoder<X>, X: ::sbor::CustomValueKind > ::sbor::Decode<X, D> for Test {
                    #[inline]
                    fn decode_body_with_value_kind(decoder: &mut D, value_kind: ::sbor::ValueKind<X>) -> Result<Self, ::sbor::DecodeError> {
                        use ::sbor::{self, Decode};
                        decoder.check_preloaded_value_kind(value_kind, ::sbor::ValueKind::Tuple)?;
                        decoder.read_and_check_size(1)?;
                        Ok(Self {
                            a: decoder.decode::<u32>()?,
                        })
                    }
                }
            },
        );
    }

    #[test]
    fn test_decode_generic() {
        let input = TokenStream::from_str("struct Test<T, D: Clashing> { a: T, b: D }").unwrap();
        let output = handle_decode(input, None).unwrap();

        assert_code_eq(
            output,
            quote! {
                impl <T: ::sbor::Decode<X, D0>, D: Clashing + ::sbor::Decode<X, D0>, D0: ::sbor::Decoder<X>, X: ::sbor::CustomValueKind > ::sbor::Decode<X, D0> for Test<T, D > {
                    #[inline]
                    fn decode_body_with_value_kind(decoder: &mut D0, value_kind: ::sbor::ValueKind<X>) -> Result<Self, ::sbor::DecodeError> {
                        use ::sbor::{self, Decode};
                        decoder.check_preloaded_value_kind(value_kind, ::sbor::ValueKind::Tuple)?;
                        decoder.read_and_check_size(2)?;
                        Ok(Self {
                            a: decoder.decode::<T>()?,
                            b: decoder.decode::<D>()?,
                        })
                    }
                }
            },
        );
    }

    #[test]
    fn test_decode_struct_with_custom_value_kind() {
        let input = TokenStream::from_str(
            "#[sbor(custom_value_kind = \"NoCustomValueKind\")] struct Test {a: u32}",
        )
        .unwrap();
        let output = handle_decode(input, None).unwrap();

        assert_code_eq(
            output,
            quote! {
                impl <D: ::sbor::Decoder<NoCustomValueKind> > ::sbor::Decode<NoCustomValueKind, D> for Test {
                    #[inline]
                    fn decode_body_with_value_kind(decoder: &mut D, value_kind: ::sbor::ValueKind<NoCustomValueKind>) -> Result<Self, ::sbor::DecodeError> {
                        use ::sbor::{self, Decode};
                        decoder.check_preloaded_value_kind(value_kind, ::sbor::ValueKind::Tuple)?;
                        decoder.read_and_check_size(1)?;
                        Ok(Self {
                            a: decoder.decode::<u32>()?,
                        })
                    }
                }
            },
        );
    }

    #[test]
    fn test_decode_struct_with_generic_params() {
        let input = TokenStream::from_str("#[sbor(generic_categorize_bounds = \"T1, T2\")] struct Test<'a, S, T1, T2> {a: &'a u32, b: S, c: Vec<T1>, d: Vec<T2>}").unwrap();
        let output = handle_decode(input, None).unwrap();

        assert_code_eq(
            output,
            quote! {
                impl <'a, S: ::sbor::Decode<X, D>, T1: ::sbor::Decode<X,D> + ::sbor::Categorize<X>, T2: ::sbor::Decode <X, D>, D: ::sbor::Decoder<X>, X: ::sbor::CustomValueKind > ::sbor::Decode<X, D> for Test<'a, S, T1, T2> {
                    #[inline]
                    fn decode_body_with_value_kind(decoder: &mut D, value_kind: ::sbor::ValueKind<X>) -> Result<Self, ::sbor::DecodeError> {
                        use ::sbor::{self, Decode};
                        decoder.check_preloaded_value_kind(value_kind, ::sbor::ValueKind::Tuple)?;
                        decoder.read_and_check_size(4)?;
                        Ok(Self {
                            a: decoder.decode::<&'a u32>()?,
                            b: decoder.decode::<S>()?,
                            c: decoder.decode::<Vec<T1> >()?,
                            d: decoder.decode::<Vec<T2> >()?,
                        })
                    }
                }
            },
        );
    }

    #[test]
    fn test_decode_enum() {
        let input = TokenStream::from_str("enum Test {A, B (u32), C {x: u8}}").unwrap();
        let output = handle_decode(input, None).unwrap();

        assert_code_eq(
            output,
            quote! {
                impl <D: ::sbor::Decoder<X>, X: ::sbor::CustomValueKind > ::sbor::Decode<X, D> for Test {
                    #[inline]
                    fn decode_body_with_value_kind(decoder: &mut D, value_kind: ::sbor::ValueKind<X>) -> Result<Self, ::sbor::DecodeError> {
                        use ::sbor::{self, Decode};
                        decoder.check_preloaded_value_kind(value_kind, ::sbor::ValueKind::Enum)?;
                        let discriminator = decoder.read_discriminator()?;
                        match discriminator {
                            0u8 => {
                                decoder.read_and_check_size(0)?;
                                Ok(Self::A)
                            },
                            1u8 => {
                                decoder.read_and_check_size(1)?;
                                Ok(Self::B(decoder.decode::<u32>()?))
                            },
                            2u8 => {
                                decoder.read_and_check_size(1)?;
                                Ok(Self::C {
                                    x: decoder.decode::<u8>()?,
                                })
                            },
                            _ => Err(::sbor::DecodeError::UnknownDiscriminator(discriminator))
                        }
                    }
                }
            },
        );
    }

    #[test]
    fn test_skip() {
        let input = TokenStream::from_str("struct Test {#[sbor(skip)] a: u32}").unwrap();
        let output = handle_decode(input, None).unwrap();

        assert_code_eq(
            output,
            quote! {
                impl <D: ::sbor::Decoder<X>, X: ::sbor::CustomValueKind > ::sbor::Decode<X, D> for Test {
                    #[inline]
                    fn decode_body_with_value_kind(decoder: &mut D, value_kind: ::sbor::ValueKind<X>) -> Result<Self, ::sbor::DecodeError> {
                        use ::sbor::{self, Decode};
                        decoder.check_preloaded_value_kind(value_kind, ::sbor::ValueKind::Tuple)?;
                        decoder.read_and_check_size(0)?;
                        Ok(Self {
                            a: <u32>::default()
                        })
                    }
                }
            },
        );
    }
}
