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
    let is_transparent = is_transparent(&parsed.attrs);

    let output = if is_transparent {
        handle_transparent_encode(parsed, context_custom_value_kind)?
    } else {
        handle_normal_encode(parsed, context_custom_value_kind)?
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
            let FieldsData {
                unskipped_self_field_names,
                ..
            } = process_fields_for_encode(&s.fields);
            if unskipped_self_field_names.len() != 1 {
                return Err(Error::new(Span::call_site(), "The transparent attribute is only supported for structs with a single unskipped field."));
            }
            let field_name = &unskipped_self_field_names[0];
            quote! {
                impl #impl_generics ::sbor::Encode <#custom_value_kind_generic, #encoder_generic> for #ident #ty_generics #where_clause {
                    #[inline]
                    fn encode_value_kind(&self, encoder: &mut #encoder_generic) -> Result<(), ::sbor::EncodeError> {
                        use ::sbor::{self, Encode};
                        self.#field_name.encode_value_kind(encoder)
                    }

                    #[inline]
                    fn encode_body(&self, encoder: &mut #encoder_generic) -> Result<(), ::sbor::EncodeError> {
                        use ::sbor::{self, Encode};
                        self.#field_name.encode_body(encoder)
                    }
                }
            }
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
            let FieldsData {
                unskipped_self_field_names,
                unskipped_field_count,
                ..
            } = process_fields_for_encode(&s.fields);
            quote! {
                impl #impl_generics ::sbor::Encode <#custom_value_kind_generic, #encoder_generic> for #ident #ty_generics #where_clause {
                    #[inline]
                    fn encode_value_kind(&self, encoder: &mut #encoder_generic) -> Result<(), ::sbor::EncodeError> {
                        encoder.write_value_kind(::sbor::ValueKind::Tuple)
                    }

                    #[inline]
                    fn encode_body(&self, encoder: &mut #encoder_generic) -> Result<(), ::sbor::EncodeError> {
                        use ::sbor::{self, Encode};
                        encoder.write_size(#unskipped_field_count)?;
                        #(encoder.encode(&self.#unskipped_self_field_names)?;)*
                        Ok(())
                    }
                }
            }
        }
        Data::Enum(DataEnum { variants, .. }) => {
            let match_arms = variants.iter().enumerate().map(|(i, v)| {
                let v_id = &v.ident;
                let i: u8 = i.try_into().expect("Too many variants found in enum");
                let discriminator: Expr = parse_quote! { #i };

                let FieldsData {
                    unskipped_field_count,
                    fields_unpacking,
                    unskipped_unpacked_field_names,
                    ..
                } = process_fields_for_encode(&v.fields);
                quote! {
                    Self::#v_id #fields_unpacking => {
                        encoder.write_discriminator(#discriminator)?;
                        encoder.write_size(#unskipped_field_count)?;
                        #(encoder.encode(#unskipped_unpacked_field_names)?;)*
                    }
                }
            });

            let encode_content = if match_arms.len() == 0 {
                quote! {}
            } else {
                quote! {
                    use ::sbor::{self, Encode};

                    match self {
                        #(#match_arms)*
                    }
                }
            };
            quote! {
                impl #impl_generics ::sbor::Encode <#custom_value_kind_generic, #encoder_generic> for #ident #ty_generics #where_clause {
                    #[inline]
                    fn encode_value_kind(&self, encoder: &mut #encoder_generic) -> Result<(), ::sbor::EncodeError> {
                        encoder.write_value_kind(::sbor::ValueKind::Enum)
                    }

                    #[inline]
                    fn encode_body(&self, encoder: &mut #encoder_generic) -> Result<(), ::sbor::EncodeError> {
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
                impl <E: ::sbor::Encoder<X>, X: ::sbor::CustomValueKind > ::sbor::Encode<X, E> for Test {
                    #[inline]
                    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), ::sbor::EncodeError> {
                        encoder.write_value_kind(::sbor::ValueKind::Tuple)
                    }

                    #[inline]
                    fn encode_body(&self, encoder: &mut E) -> Result<(), ::sbor::EncodeError> {
                        use ::sbor::{self, Encode};
                        encoder.write_size(1)?;
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
                impl <E: ::sbor::Encoder<X>, X: ::sbor::CustomValueKind > ::sbor::Encode<X, E> for Test {
                    #[inline]
                    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), ::sbor::EncodeError> {
                        encoder.write_value_kind(::sbor::ValueKind::Enum)
                    }

                    #[inline]
                    fn encode_body(&self, encoder: &mut E) -> Result<(), ::sbor::EncodeError> {
                        use ::sbor::{self, Encode};
                        match self {
                            Self::A => {
                                encoder.write_discriminator(0u8)?;
                                encoder.write_size(0)?;
                            }
                            Self::B(a0) => {
                                encoder.write_discriminator(1u8)?;
                                encoder.write_size(1)?;
                                encoder.encode(a0)?;
                            }
                            Self::C { x, .. } => {
                                encoder.write_discriminator(2u8)?;
                                encoder.write_size(1)?;
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
                impl <E: ::sbor::Encoder<X>, X: ::sbor::CustomValueKind > ::sbor::Encode<X, E> for Test {
                    #[inline]
                    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), ::sbor::EncodeError> {
                        encoder.write_value_kind(::sbor::ValueKind::Tuple)
                    }

                    #[inline]
                    fn encode_body(&self, encoder: &mut E) -> Result<(), ::sbor::EncodeError> {
                        use ::sbor::{self, Encode};
                        encoder.write_size(0)?;
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
                impl <T, E: Clashing, E0: ::sbor::Encoder<X>, X: ::sbor::CustomValueKind > ::sbor::Encode<X, E0> for Test<T, E >
                where
                    T: ::sbor::Encode<X, E0>,
                    E: ::sbor::Encode<X, E0>,
                    T: ::sbor::Categorize<X>,
                    E: ::sbor::Categorize<X>
                {
                    #[inline]
                    fn encode_value_kind(&self, encoder: &mut E0) -> Result<(), ::sbor::EncodeError> {
                        encoder.write_value_kind(::sbor::ValueKind::Tuple)
                    }

                    #[inline]
                    fn encode_body(&self, encoder: &mut E0) -> Result<(), ::sbor::EncodeError> {
                        use ::sbor::{self, Encode};
                        encoder.write_size(2)?;
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
                impl <E: ::sbor::Encoder<NoCustomValueKind> > ::sbor::Encode<NoCustomValueKind, E> for Test {
                    #[inline]
                    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), ::sbor::EncodeError> {
                        encoder.write_value_kind(::sbor::ValueKind::Tuple)
                    }

                    #[inline]
                    fn encode_body(&self, encoder: &mut E) -> Result<(), ::sbor::EncodeError> {
                        use ::sbor::{self, Encode};
                        encoder.write_size(0)?;
                        Ok(())
                    }
                }
            },
        );
    }

    #[test]
    fn test_custom_value_kind_canonical_path() {
        let input = TokenStream::from_str(
            "#[sbor(custom_value_kind = \"::sbor::basic::NoCustomValueKind\")] struct Test {#[sbor(skip)] a: u32}",
        )
        .unwrap();
        let output = handle_encode(input, None).unwrap();

        assert_code_eq(
            output,
            quote! {
                impl <E: ::sbor::Encoder<::sbor::basic::NoCustomValueKind> > ::sbor::Encode<::sbor::basic::NoCustomValueKind, E> for Test {
                    #[inline]
                    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), ::sbor::EncodeError> {
                        encoder.write_value_kind(::sbor::ValueKind::Tuple)
                    }

                    #[inline]
                    fn encode_body(&self, encoder: &mut E) -> Result<(), ::sbor::EncodeError> {
                        use ::sbor::{self, Encode};
                        encoder.write_size(0)?;
                        Ok(())
                    }
                }
            },
        );
    }
}
