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

pub fn handle_encode(
    input: TokenStream,
    context_custom_value_kind: Option<&'static str>,
) -> Result<TokenStream> {
    trace!("handle_encode() starts");

    let parsed: DeriveInput = parse2(input)?;

    let output = match get_derive_strategy(&parsed.attrs)? {
        DeriveStrategy::Normal => handle_normal_encode(parsed, context_custom_value_kind)?,
        DeriveStrategy::Transparent => {
            handle_transparent_encode(parsed, context_custom_value_kind)?
        }
        DeriveStrategy::DeriveAs {
            as_type, as_ref, ..
        } => handle_encode_as(parsed, context_custom_value_kind, &as_type, &as_ref)?,
    };

    #[cfg(feature = "trace")]
    crate::utils::print_generated_code("Encode", &output);

    trace!("handle_encode() finishes");
    Ok(output)
}

pub fn handle_transparent_encode(
    parsed: DeriveInput,
    context_custom_value_kind: Option<&'static str>,
) -> Result<TokenStream> {
    let output = match &parsed.data {
        Data::Struct(s) => {
            let single_field = process_fields(&s.fields)?
                .unique_unskipped_field()
                .ok_or_else(|| Error::new(
                    Span::call_site(),
                    "The transparent attribute is only supported for structs with a single unskipped field.",
                ))?;
            handle_encode_as(
                parsed,
                context_custom_value_kind,
                single_field.field_type(),
                &single_field.self_field_reference(),
            )?
        }
        Data::Enum(_) => {
            return Err(Error::new(Span::call_site(), "The transparent attribute is only supported for structs with a single unskipped field."));
        }
        Data::Union(_) => {
            return Err(Error::new(Span::call_site(), "Union is not supported!"));
        }
    };

    Ok(output)
}

pub fn handle_encode_as(
    parsed: DeriveInput,
    context_custom_value_kind: Option<&'static str>,
    as_type: &Type,
    as_ref_code: &TokenStream,
) -> Result<TokenStream> {
    let DeriveInput {
        attrs,
        ident,
        generics,
        ..
    } = parsed;
    let (impl_generics, ty_generics, where_clause, custom_value_kind_generic, encoder_generic) =
        build_encode_generics(&generics, &attrs, context_custom_value_kind)?;

    // NOTE: The `: &#as_type` is not strictly needed for the code to compile,
    // but it is useful to sanity check that the user has provided the correct implementation.
    // If they have not, they should get a nice and clear error message.
    let output = quote! {
        impl #impl_generics sbor::Encode <#custom_value_kind_generic, #encoder_generic> for #ident #ty_generics #where_clause {
            #[inline]
            fn encode_value_kind(&self, encoder: &mut #encoder_generic) -> Result<(), sbor::EncodeError> {
                use sbor::{self, Encode};
                let as_ref: &#as_type = #as_ref_code;
                as_ref.encode_value_kind(encoder)
            }

            #[inline]
            fn encode_body(&self, encoder: &mut #encoder_generic) -> Result<(), sbor::EncodeError> {
                use sbor::{self, Encode};
                let as_ref: &#as_type = #as_ref_code;
                as_ref.encode_body(encoder)
            }
        }
    };

    Ok(output)
}

pub fn handle_normal_encode(
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
    let (impl_generics, ty_generics, where_clause, custom_value_kind_generic, encoder_generic) =
        build_encode_generics(&generics, &attrs, context_custom_value_kind)?;

    let output = match data {
        Data::Struct(s) => {
            let fields_data = process_fields(&s.fields)?;
            let unskipped_field_count = fields_data.unskipped_field_count();
            let unskipped_self_field_references = fields_data.unskipped_self_field_references();
            quote! {
                impl #impl_generics sbor::Encode <#custom_value_kind_generic, #encoder_generic> for #ident #ty_generics #where_clause {
                    #[inline]
                    fn encode_value_kind(&self, encoder: &mut #encoder_generic) -> Result<(), sbor::EncodeError> {
                        encoder.write_value_kind(sbor::ValueKind::Tuple)
                    }

                    #[inline]
                    fn encode_body(&self, encoder: &mut #encoder_generic) -> Result<(), sbor::EncodeError> {
                        use sbor::{self, Encode};
                        encoder.write_size(#unskipped_field_count)?;
                        #(encoder.encode(#unskipped_self_field_references)?;)*
                        Ok(())
                    }
                }
            }
        }
        Data::Enum(DataEnum { variants, .. }) => {
            let EnumVariantsData {
                source_variants, ..
            } = process_enum_variants(&attrs, &variants)?;
            let match_arms = source_variants
                .iter()
                .map(|source_variant| {
                    Ok(match source_variant {
                        SourceVariantData::Reachable(VariantData {
                            variant_name,
                            discriminator,
                            fields_handling: FieldsHandling::Standard(fields_data),
                            ..
                        }) => {
                            let unskipped_field_count = fields_data.unskipped_field_count();
                            let fields_unpacking = fields_data.fields_unpacking();
                            let unskipped_unpacking_variable_names = fields_data.unskipped_unpacking_variable_names();
                            quote! {
                                Self::#variant_name #fields_unpacking => {
                                    encoder.write_discriminator(#discriminator)?;
                                    encoder.write_size(#unskipped_field_count)?;
                                    #(encoder.encode(#unskipped_unpacking_variable_names)?;)*
                                }
                            }
                        }
                        SourceVariantData::Reachable(VariantData {
                            variant_name,
                            discriminator,
                            fields_handling: FieldsHandling::Flatten { unique_field, fields_data, },
                            ..
                        }) => {
                            let fields_unpacking = fields_data.fields_unpacking();
                            let field_type = unique_field.field_type();
                            let unpacking_field_name = unique_field.variable_name_from_unpacking();
                            let tuple_assertion = output_flatten_type_is_sbor_tuple_assertion(
                                &custom_value_kind_generic,
                                field_type,
                            );
                            quote! {
                                Self::#variant_name #fields_unpacking => {
                                    // Flatten is only valid if the single child type is an SBOR tuple, so do a
                                    // zero-cost assertion on this so the user gets a good error message if they
                                    // misuse this.
                                    #tuple_assertion
                                    // We make use of the fact that an enum body encodes as (discriminator, fields_count, ..fields)
                                    // And a tuple body encodes as (fields_count, ..fields)
                                    // So we can flatten by encoding the discriminator and then running `encode_body` on the child tuple
                                    encoder.write_discriminator(#discriminator)?;
                                    <#field_type as sbor::Encode <#custom_value_kind_generic, #encoder_generic>>::encode_body(
                                        #unpacking_field_name,
                                        encoder
                                    )?;
                                }
                            }
                        }
                        SourceVariantData::Unreachable(UnreachableVariantData {
                            variant_name,
                            fields_data,
                            ..
                        }) => {
                            let empty_fields_unpacking = fields_data.empty_fields_unpacking();
                            let panic_message =
                                format!("Variant {} ignored as unreachable", variant_name.to_string());
                            quote! {
                                Self::#variant_name #empty_fields_unpacking => panic!(#panic_message),
                            }
                        }
                    })
                })
                .collect::<Result<Vec<_>>>()?;

            let encode_content = if match_arms.len() == 0 {
                quote! {}
            } else {
                quote! {
                    use sbor::{self, Encode};

                    match self {
                        #(#match_arms)*
                    }
                }
            };
            quote! {
                impl #impl_generics sbor::Encode <#custom_value_kind_generic, #encoder_generic> for #ident #ty_generics #where_clause {
                    #[inline]
                    fn encode_value_kind(&self, encoder: &mut #encoder_generic) -> Result<(), sbor::EncodeError> {
                        encoder.write_value_kind(sbor::ValueKind::Enum)
                    }

                    #[inline]
                    fn encode_body(&self, encoder: &mut #encoder_generic) -> Result<(), sbor::EncodeError> {
                        #encode_content
                        Ok(())
                    }
                }
            }
        }
        Data::Union(_) => {
            return Err(Error::new(Span::call_site(), "Union is not supported!"));
        }
    };

    #[cfg(feature = "trace")]
    crate::utils::print_generated_code("Encode", &output);

    trace!("handle_encode() finishes");
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
        let output = handle_encode(input, None).unwrap();

        assert_code_eq(
            output,
            quote! {
                impl <E: sbor::Encoder<X>, X: sbor::CustomValueKind > sbor::Encode<X, E> for Test {
                    #[inline]
                    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), sbor::EncodeError> {
                        encoder.write_value_kind(sbor::ValueKind::Tuple)
                    }

                    #[inline]
                    fn encode_body(&self, encoder: &mut E) -> Result<(), sbor::EncodeError> {
                        use sbor::{self, Encode};
                        encoder.write_size(1usize)?;
                        encoder.encode(&self.a)?;
                        Ok(())
                    }
                }
            },
        );
    }

    #[test]
    fn test_encode_enum() {
        let input = TokenStream::from_str("enum Test {A, B (u32), C {x: u8}}").unwrap();
        let output = handle_encode(input, None).unwrap();

        assert_code_eq(
            output,
            quote! {
                impl <E: sbor::Encoder<X>, X: sbor::CustomValueKind > sbor::Encode<X, E> for Test {
                    #[inline]
                    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), sbor::EncodeError> {
                        encoder.write_value_kind(sbor::ValueKind::Enum)
                    }

                    #[inline]
                    fn encode_body(&self, encoder: &mut E) -> Result<(), sbor::EncodeError> {
                        use sbor::{self, Encode};
                        match self {
                            Self::A => {
                                encoder.write_discriminator(0u8)?;
                                encoder.write_size(0usize)?;
                            }
                            Self::B(a0) => {
                                encoder.write_discriminator(1u8)?;
                                encoder.write_size(1usize)?;
                                encoder.encode(a0)?;
                            }
                            Self::C { x, .. } => {
                                encoder.write_discriminator(2u8)?;
                                encoder.write_size(1usize)?;
                                encoder.encode(x)?;
                            }
                        }
                        Ok(())
                    }
                }
            },
        );
    }

    #[test]
    fn test_skip() {
        let input = TokenStream::from_str("struct Test {#[sbor(skip)] a: u32}").unwrap();
        let output = handle_encode(input, None).unwrap();

        assert_code_eq(
            output,
            quote! {
                impl <E: sbor::Encoder<X>, X: sbor::CustomValueKind > sbor::Encode<X, E> for Test {
                    #[inline]
                    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), sbor::EncodeError> {
                        encoder.write_value_kind(sbor::ValueKind::Tuple)
                    }

                    #[inline]
                    fn encode_body(&self, encoder: &mut E) -> Result<(), sbor::EncodeError> {
                        use sbor::{self, Encode};
                        encoder.write_size(0usize)?;
                        Ok(())
                    }
                }
            },
        );
    }

    #[test]
    fn test_encode_generic() {
        let input = TokenStream::from_str("struct Test<T, E: Clashing> { a: T, b: E, }").unwrap();
        let output = handle_encode(input, None).unwrap();

        assert_code_eq(
            output,
            quote! {
                impl <T, E: Clashing, E0: sbor::Encoder<X>, X: sbor::CustomValueKind > sbor::Encode<X, E0> for Test<T, E >
                where
                    T: sbor::Encode<X, E0>,
                    E: sbor::Encode<X, E0>
                {
                    #[inline]
                    fn encode_value_kind(&self, encoder: &mut E0) -> Result<(), sbor::EncodeError> {
                        encoder.write_value_kind(sbor::ValueKind::Tuple)
                    }

                    #[inline]
                    fn encode_body(&self, encoder: &mut E0) -> Result<(), sbor::EncodeError> {
                        use sbor::{self, Encode};
                        encoder.write_size(2usize)?;
                        encoder.encode(&self.a)?;
                        encoder.encode(&self.b)?;
                        Ok(())
                    }
                }
            },
        );
    }

    #[test]
    fn test_encode_struct_with_custom_value_kind() {
        let input = TokenStream::from_str(
            "#[sbor(custom_value_kind = \"NoCustomValueKind\")] struct Test {#[sbor(skip)] a: u32}",
        )
        .unwrap();
        let output = handle_encode(input, None).unwrap();

        assert_code_eq(
            output,
            quote! {
                impl <E: sbor::Encoder<NoCustomValueKind> > sbor::Encode<NoCustomValueKind, E> for Test {
                    #[inline]
                    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), sbor::EncodeError> {
                        encoder.write_value_kind(sbor::ValueKind::Tuple)
                    }

                    #[inline]
                    fn encode_body(&self, encoder: &mut E) -> Result<(), sbor::EncodeError> {
                        use sbor::{self, Encode};
                        encoder.write_size(0usize)?;
                        Ok(())
                    }
                }
            },
        );
    }

    #[test]
    fn test_custom_value_kind_canonical_path() {
        let input = TokenStream::from_str(
            "#[sbor(custom_value_kind = \"sbor::basic::NoCustomValueKind\")] struct Test {#[sbor(skip)] a: u32}",
        )
        .unwrap();
        let output = handle_encode(input, None).unwrap();

        assert_code_eq(
            output,
            quote! {
                impl <E: sbor::Encoder<sbor::basic::NoCustomValueKind> > sbor::Encode<sbor::basic::NoCustomValueKind, E> for Test {
                    #[inline]
                    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), sbor::EncodeError> {
                        encoder.write_value_kind(sbor::ValueKind::Tuple)
                    }

                    #[inline]
                    fn encode_body(&self, encoder: &mut E) -> Result<(), sbor::EncodeError> {
                        use sbor::{self, Encode};
                        encoder.write_size(0usize)?;
                        Ok(())
                    }
                }
            },
        );
    }
}
