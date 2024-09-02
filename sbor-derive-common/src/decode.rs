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

    let parsed: DeriveInput = parse2(input)?;

    let output = match get_derive_strategy(&parsed.attrs)? {
        DeriveStrategy::Normal => handle_normal_decode(parsed, context_custom_value_kind)?,
        DeriveStrategy::Transparent => {
            handle_transparent_decode(parsed, context_custom_value_kind)?
        }
        DeriveStrategy::DeriveAs {
            as_type,
            from_value,
            ..
        } => handle_decode_as(parsed, context_custom_value_kind, &as_type, &from_value)?,
    };

    #[cfg(feature = "trace")]
    crate::utils::print_generated_code("Decode", &output);

    trace!("handle_decode() finishes");
    Ok(output)
}

pub fn handle_transparent_decode(
    parsed: DeriveInput,
    context_custom_value_kind: Option<&'static str>,
) -> Result<TokenStream> {
    let DeriveInput { data, .. } = &parsed;

    match data {
        Data::Struct(s) => {
            let fields_data = process_fields(&s.fields)?;
            let single_field = fields_data
                .unique_unskipped_field()
                .ok_or_else(|| Error::new(
                    Span::call_site(),
                    "The transparent attribute is only supported for structs with a single unskipped field.",
                ))?;

            let decode_content = decode_unique_unskipped_field_from_value(
                quote!{ Self },
                &fields_data,
            )?;

            handle_decode_as(
                parsed,
                context_custom_value_kind,
                single_field.field_type(),
                &decode_content,
            )
        }
        Data::Enum(_) => {
            Err(Error::new(Span::call_site(), "The transparent attribute is only supported for structs with a single unskipped field."))
        }
        Data::Union(_) => {
            Err(Error::new(Span::call_site(), "Union is not supported!"))
        }
    }
}

fn handle_decode_as(
    parsed: DeriveInput,
    context_custom_value_kind: Option<&'static str>,
    as_type: &Type,
    from_value: &TokenStream,
) -> Result<TokenStream> {
    let DeriveInput {
        attrs,
        ident,
        generics,
        ..
    } = parsed;
    let (impl_generics, ty_generics, where_clause, custom_value_kind_generic, decoder_generic) =
        build_decode_generics(&generics, &attrs, context_custom_value_kind)?;

    let output = quote! {
        impl #impl_generics sbor::Decode <#custom_value_kind_generic, #decoder_generic> for #ident #ty_generics #where_clause {
            #[inline]
            fn decode_body_with_value_kind(decoder: &mut #decoder_generic, value_kind: sbor::ValueKind<#custom_value_kind_generic>) -> Result<Self, sbor::DecodeError> {
                use sbor::{self, Decode};
                let value = <#as_type as sbor::Decode<#custom_value_kind_generic, #decoder_generic>>::decode_body_with_value_kind(decoder, value_kind)?;
                Ok(#from_value)
            }
        }
    };

    Ok(output)
}

pub fn handle_normal_decode(
    parsed: DeriveInput,
    context_custom_value_kind: Option<&'static str>,
) -> Result<TokenStream> {
    let DeriveInput {
        attrs,
        ident,
        data,
        generics,
        ..
    } = parsed;
    let (impl_generics, ty_generics, where_clause, custom_value_kind_generic, decoder_generic) =
        build_decode_generics(&generics, &attrs, context_custom_value_kind)?;

    let output = match data {
        Data::Struct(s) => {
            let fields_data = process_fields(&s.fields)?;
            let decode_fields_content = decode_fields_content(quote! { Self }, &fields_data)?;

            quote! {
                impl #impl_generics sbor::Decode <#custom_value_kind_generic, #decoder_generic> for #ident #ty_generics #where_clause {
                    #[inline]
                    fn decode_body_with_value_kind(decoder: &mut #decoder_generic, value_kind: sbor::ValueKind<#custom_value_kind_generic>) -> Result<Self, sbor::DecodeError> {
                        use sbor::{self, Decode};
                        decoder.check_preloaded_value_kind(value_kind, sbor::ValueKind::Tuple)?;
                        #decode_fields_content
                    }
                }
            }
        }
        Data::Enum(DataEnum { variants, .. }) => {
            let EnumVariantsData { sbor_variants, .. } = process_enum_variants(&attrs, &variants)?;
            let match_arms = sbor_variants
                .iter()
                .map(|VariantData {
                        variant_name,
                        discriminator_pattern,
                        fields_handling,
                        ..
                    }| -> Result<_> {
                        let content = match fields_handling {
                            FieldsHandling::Standard(fields_data) => {
                                decode_fields_content(
                                    quote! { Self::#variant_name },
                                    fields_data,
                                )?
                            },
                            FieldsHandling::Flatten { unique_field, fields_data } => {
                                let field_type = unique_field.field_type();
                                let tuple_assertion = output_flatten_type_is_sbor_tuple_assertion(
                                    &custom_value_kind_generic,
                                    field_type,
                                );
                                let construct_variant = decode_unique_unskipped_field_from_value(
                                    quote! { Self::#variant_name },
                                    fields_data,
                                )?;
                                quote! {
                                    #tuple_assertion
                                    let value = <#field_type as sbor::Decode<#custom_value_kind_generic, #decoder_generic>>::decode_body_with_value_kind(decoder, ValueKind::Tuple)?;
                                    Ok(#construct_variant)
                                }
                            },
                        };
                        Ok(quote! {
                            #discriminator_pattern => {
                                #content
                            }
                        })
                    },
                )
                .collect::<Result<Vec<_>>>()?;

            // Note: We use #[deny(unreachable_patterns)] to protect against users
            // defining overlapping consts in their custom #[sbor(discriminator(X))] definitions
            quote! {
                impl #impl_generics sbor::Decode <#custom_value_kind_generic, #decoder_generic> for #ident #ty_generics #where_clause {
                    #[inline]
                    fn decode_body_with_value_kind(decoder: &mut #decoder_generic, value_kind: sbor::ValueKind<#custom_value_kind_generic>) -> Result<Self, sbor::DecodeError> {
                        use sbor::{self, Decode};
                        decoder.check_preloaded_value_kind(value_kind, sbor::ValueKind::Enum)?;
                        let discriminator = decoder.read_discriminator()?;
                        #[deny(unreachable_patterns)]
                        match discriminator {
                            #(#match_arms,)*
                            _ => Err(sbor::DecodeError::UnknownDiscriminator(discriminator))
                        }
                    }
                }
            }
        }
        Data::Union(_) => {
            return Err(Error::new(Span::call_site(), "Union is not supported!"));
        }
    };

    Ok(output)
}

fn decode_fields_content(
    self_constructor: TokenStream,
    fields_data: &FieldsData,
) -> Result<TokenStream> {
    let unskipped_field_count = fields_data.unskipped_field_count();

    Ok(match fields_data {
        FieldsData::Named(fields) => {
            let assignments = fields.iter().map(
                |NamedField {
                     name,
                     field_type,
                     is_skipped,
                     ..
                 }| {
                    if *is_skipped {
                        quote! { #name: <#field_type>::default() }
                    } else {
                        quote! { #name: decoder.decode::<#field_type>()? }
                    }
                },
            );
            quote! {
                decoder.read_and_check_size(#unskipped_field_count)?;
                Ok(#self_constructor {
                    #(#assignments,)*
                })
            }
        }
        FieldsData::Unnamed(fields) => {
            let values = fields.iter().map(
                |UnnamedField {
                     field_type,
                     is_skipped,
                     ..
                 }| {
                    if *is_skipped {
                        quote! { <#field_type>::default() }
                    } else {
                        quote! { decoder.decode::<#field_type>()? }
                    }
                },
            );
            quote! {
                decoder.read_and_check_size(#unskipped_field_count)?;
                Ok(#self_constructor
                (
                    #(#values,)*
                ))
            }
        }
        FieldsData::Unit => {
            quote! {
                decoder.read_and_check_size(#unskipped_field_count)?;
                Ok(#self_constructor)
            }
        }
    })
}

pub(crate) fn decode_unique_unskipped_field_from_value(
    self_constructor: TokenStream,
    fields_data: &FieldsData,
) -> Result<TokenStream> {
    if fields_data.unique_unskipped_field().is_none() {
        panic!("Should already have checked that there is only one unique unskipped field before calling this method");
    }

    let output = match &fields_data {
        FieldsData::Named(fields) => {
            let assignments = fields.iter().map(
                |NamedField {
                     name,
                     field_type,
                     is_skipped,
                     ..
                 }| {
                    if *is_skipped {
                        quote! { #name: <#field_type>::default() }
                    } else {
                        // Have already checked there's only one of these
                        quote! { #name: value }
                    }
                },
            );
            quote! {
                #self_constructor {
                    #(#assignments,)*
                }
            }
        }
        FieldsData::Unnamed(fields) => {
            let field_values = fields.iter().map(
                |UnnamedField {
                     field_type,
                     is_skipped,
                     ..
                 }| {
                    if *is_skipped {
                        quote! { <#field_type>::default() }
                    } else {
                        // Have already checked there's only one of these
                        quote! { value }
                    }
                },
            );
            quote! {
                #self_constructor(
                    #(#field_values,)*
                )
            }
        }
        FieldsData::Unit => {
            quote! {
                #self_constructor
            }
        }
    };

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
                impl <D: sbor::Decoder<X>, X: sbor::CustomValueKind > sbor::Decode<X, D> for Test {
                    #[inline]
                    fn decode_body_with_value_kind(decoder: &mut D, value_kind: sbor::ValueKind<X>) -> Result<Self, sbor::DecodeError> {
                        use sbor::{self, Decode};
                        decoder.check_preloaded_value_kind(value_kind, sbor::ValueKind::Tuple)?;
                        decoder.read_and_check_size(1usize)?;
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
                impl <T, D: Clashing, D0: sbor::Decoder<X>, X: sbor::CustomValueKind> sbor::Decode<X, D0> for Test<T, D>
                    where
                        T : sbor::Decode<X, D0>,
                        D : sbor::Decode<X, D0>
                {
                    #[inline]
                    fn decode_body_with_value_kind(decoder: &mut D0, value_kind: sbor::ValueKind<X>) -> Result<Self, sbor::DecodeError> {
                        use sbor::{self, Decode};
                        decoder.check_preloaded_value_kind(value_kind, sbor::ValueKind::Tuple)?;
                        decoder.read_and_check_size(2usize)?;
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
                impl <D: sbor::Decoder<NoCustomValueKind> > sbor::Decode<NoCustomValueKind, D> for Test {
                    #[inline]
                    fn decode_body_with_value_kind(decoder: &mut D, value_kind: sbor::ValueKind<NoCustomValueKind>) -> Result<Self, sbor::DecodeError> {
                        use sbor::{self, Decode};
                        decoder.check_preloaded_value_kind(value_kind, sbor::ValueKind::Tuple)?;
                        decoder.read_and_check_size(1usize)?;
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
        let input = TokenStream::from_str("#[sbor(categorize_types = \"T1, T2\")] struct Test<'a, S, T1, T2> {a: &'a u32, b: S, c: Vec<T1>, d: Vec<T2>}").unwrap();
        let output = handle_decode(input, None).unwrap();

        assert_code_eq(
            output,
            quote! {
                impl <'a, S, T1, T2, D: sbor::Decoder<X>, X: sbor::CustomValueKind > sbor::Decode<X, D> for Test<'a, S, T1, T2>
                where
                    S: sbor::Decode<X, D>,
                    T1: sbor::Decode<X, D>,
                    T2: sbor::Decode<X, D>,
                    T1: sbor::Categorize<X>,
                    T2: sbor::Categorize<X>
                {
                    #[inline]
                    fn decode_body_with_value_kind(decoder: &mut D, value_kind: sbor::ValueKind<X>) -> Result<Self, sbor::DecodeError> {
                        use sbor::{self, Decode};
                        decoder.check_preloaded_value_kind(value_kind, sbor::ValueKind::Tuple)?;
                        decoder.read_and_check_size(4usize)?;
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
                impl <D: sbor::Decoder<X>, X: sbor::CustomValueKind > sbor::Decode<X, D> for Test {
                    #[inline]
                    fn decode_body_with_value_kind(decoder: &mut D, value_kind: sbor::ValueKind<X>) -> Result<Self, sbor::DecodeError> {
                        use sbor::{self, Decode};
                        decoder.check_preloaded_value_kind(value_kind, sbor::ValueKind::Enum)?;
                        let discriminator = decoder.read_discriminator()?;
                        #[deny(unreachable_patterns)]
                        match discriminator {
                            0u8 => {
                                decoder.read_and_check_size(0usize)?;
                                Ok(Self::A)
                            },
                            1u8 => {
                                decoder.read_and_check_size(1usize)?;
                                Ok(Self::B(decoder.decode::<u32>()?,))
                            },
                            2u8 => {
                                decoder.read_and_check_size(1usize)?;
                                Ok(Self::C {
                                    x: decoder.decode::<u8>()?,
                                })
                            },
                            _ => Err(sbor::DecodeError::UnknownDiscriminator(discriminator))
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
                impl <D: sbor::Decoder<X>, X: sbor::CustomValueKind > sbor::Decode<X, D> for Test {
                    #[inline]
                    fn decode_body_with_value_kind(decoder: &mut D, value_kind: sbor::ValueKind<X>) -> Result<Self, sbor::DecodeError> {
                        use sbor::{self, Decode};
                        decoder.check_preloaded_value_kind(value_kind, sbor::ValueKind::Tuple)?;
                        decoder.read_and_check_size(0usize)?;
                        Ok(Self {
                            a: <u32>::default(),
                        })
                    }
                }
            },
        );
    }
}
