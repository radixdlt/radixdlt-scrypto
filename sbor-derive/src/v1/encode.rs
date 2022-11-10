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

    let body = match data {
        Data::Struct(s) => {
            let fields = match s.fields {
                syn::Fields::Named(FieldsNamed { named, .. }) => named.into_iter().collect(),
                syn::Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => unnamed.into_iter().collect(),
                syn::Fields::Unit => vec![],
            };
            // TODO: Support transparent for length 1
            encode_product_value(
                fields,
                |i, ident| {
                    ident
                        .map(|ident| quote! { &self.#ident })
                        .unwrap_or_else(|| {
                            let index = Index::from(i);
                            quote! { &self.#index }
                        })
                }
            )?
        },
        Data::Enum(DataEnum { variants, .. }) => {
            if variants.len() == 0 {
                quote! {}
            } else {
                let mut match_arms: Vec<TokenStream> = Vec::new();
                for (variant_index, variant) in variants.into_iter().enumerate() {
                    // TODO - implement preference for strings or byte
                    let variant_ident = variant.ident;

                    let discriminator_header = {
                        // TODO - support different discriminator variants
                        if variant_index > 255 {
                            return Err(Error::new(Span::call_site(), format!("More than 255 variants not currently supported!")));
                        }
                        let byte_variant_index = variant_index as u8;
                        quote! {
                            encoder.write_sum_type_u8_discriminator_header(#byte_variant_index)?;
                        }
                    };
    
                    let match_arm = match variant.fields {
                        Fields::Named(FieldsNamed { named, .. }) => {
                            let unskipped: Vec<_> = named.into_iter().filter(|f| !is_skipped(f)).collect();
    
                            let args: Vec<_> = unskipped.clone().into_iter().map(|f| f.ident.unwrap()).collect();
                            let body_value = encode_product_value(
                                unskipped,
                                |_i, field_ident| {
                                    quote! { #field_ident }
                                }
                            )?;
                            // TODO: Support transparent!
                            quote! {
                                Self::#variant_ident {#(#args,)* ..} => {
                                    #discriminator_header
                                    encoder.write_interpretation(DefaultInterpretations::ENUM_VARIANT_STRUCT)?;
                                    #body_value
                                }
                            }
                        },
                        Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                            let mut i: usize = 0;
                            let arg_idents: Vec<_> = unnamed
                                .iter()
                                .map(|f| {
                                    let is_skipped = is_skipped(f);
                                    let arg_ident = if is_skipped {
                                        quote! { _ }
                                    } else {
                                        let ident = format_ident!("a{}", i);
                                        i += 1;
                                        quote! { #ident }
                                    };
                                    arg_ident
                                })
                                .collect();

                            let body_value = encode_product_value(
                                unnamed.into_iter().collect(),
                                |i, _| {
                                    let ident = format_ident!("a{}", i);
                                    quote! { #ident }
                                }
                            )?;
                            quote! {
                                Self::#variant_ident (#(#arg_idents),*) => {
                                    #discriminator_header
                                    encoder.write_interpretation(DefaultInterpretations::ENUM_VARIANT_STRUCT)?;
                                    #body_value
                                }
                            }
                        },
                        Fields::Unit => {
                            let body_value = encode_product_value(
                                vec![],
                                |_, _| { quote! { } }
                            )?;
                            quote! {
                                Self::#variant_ident => {
                                    #discriminator_header
                                    encoder.write_interpretation(DefaultInterpretations::ENUM_VARIANT_UNIT)?;
                                    #body_value
                                }
                            }
                        },
                    };
    
                    match_arms.push(match_arm);
                }
                quote! {
                    match self {
                        #(#match_arms,)*
                    }
                    Ok(())
                }
            }
        }
        Data::Union(_) => {
            return Err(Error::new(Span::call_site(), "Union is not supported!"));
        }
    };

    let output = quote! {
        impl #impl_generics ::sbor::v1::Encode<E> for #ident #ty_generics #where_clause {
            #[inline]
            fn encode_value(&self, encoder: &mut E) -> Result<(), ::sbor::v1::EncodeError> {
                use ::sbor::v1::*;
                #body
            }
        }
    };

    trace!("handle_encode() finishes");

    #[cfg(feature = "trace")]
    crate::utils::print_generated_code("Encode", &output);

    Ok(output)
}

fn encode_product_value(
    unfiltered_fields: Vec<Field>,
    map_ident: impl Fn(usize, Option<&Ident>) -> TokenStream
) -> Result<TokenStream> {
    let filtered_fields: Vec<_> = unfiltered_fields.iter().filter(|f| !is_skipped(f)).collect();
    let field_count = filtered_fields.len();

    let encode_length_statement = {
        if field_count <= 255 {
            let field_count = field_count as u8;
            quote! {
                encoder.write_product_type_header_u8_length(#field_count)?;
            }
        } else if field_count <= u16::MAX as usize {
            let field_count = field_count as u16;
            quote! {
                encoder.write_product_type_header_u16_length(#field_count)?;
            }
        } else {
            return Err(Error::new(Span::call_site(), format!("More than {} fields not supported!", u16::MAX)));
        }
    };

    let mut token_stream = quote! {
        #encode_length_statement
    };

    for (i, field) in filtered_fields.iter().enumerate() {
        let ident_to_use = map_ident(i, field.ident.as_ref());
        token_stream.extend(quote! {
            encoder.encode(#ident_to_use)?;
        });
    }

    Ok(token_stream)
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
                impl <E: ::sbor::v1::Encoder> ::sbor::v1::Encode<E> for Test {
                    #[inline]
                    fn encode_value(&self, encoder: &mut E) -> Result<(), ::sbor::v1::EncodeError> {
                        use ::sbor::v1::*;
                        match self {
                            Self::A => {
                                encoder.write_sum_type_u8_discriminator_header(0u8)?;
                                encoder.write_interpretation(DefaultInterpretations::ENUM_VARIANT_TUPLE)?;
                                encoder.write_product_type_header_u8_length(0u8)?;
                            },
                            Self::B(a0) => {
                                encoder.write_sum_type_u8_discriminator_header(1u8)?;
                                encoder.write_interpretation(DefaultInterpretations::ENUM_VARIANT_TUPLE)?;
                                encoder.write_product_type_header_u8_length(1u8)?;
                                encoder.encode(a0)?;
                            },
                            Self::C { x, .. } => {
                                encoder.write_sum_type_u8_discriminator_header(2u8)?;
                                encoder.write_interpretation(DefaultInterpretations::ENUM_VARIANT_STRUCT)?;
                                encoder.write_product_type_header_u8_length(1u8)?;
                                encoder.encode(x)?;
                            },
                        }
                        Ok(())
                    }
                }
            },
        );
    }
}
