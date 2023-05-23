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

pub fn handle_categorize(
    input: TokenStream,
    context_custom_value_kind: Option<&'static str>,
) -> Result<TokenStream> {
    trace!("handle_categorize() starts");

    let parsed: DeriveInput = parse2(input)?;
    let is_transparent = is_transparent(&parsed.attrs)?;

    let output = if is_transparent {
        handle_transparent_categorize(parsed, context_custom_value_kind)?
    } else {
        handle_normal_categorize(parsed, context_custom_value_kind)?
    };

    #[cfg(feature = "trace")]
    crate::utils::print_generated_code("Categorize", &output);

    trace!("handle_categorize() finishes");
    Ok(output)
}

fn handle_normal_categorize(
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
    let (impl_generics, ty_generics, where_clause, sbor_cvk) =
        build_custom_categorize_generic(&generics, &attrs, context_custom_value_kind, false)?;

    let output = match data {
        Data::Struct(s) => {
            let FieldsData {
                unskipped_field_names,
                ..
            } = process_fields_for_categorize(&s.fields)?;
            let field_count = unskipped_field_names.len();
            quote! {
                impl #impl_generics ::sbor::Categorize <#sbor_cvk> for #ident #ty_generics #where_clause {
                    #[inline]
                    fn value_kind() -> ::sbor::ValueKind <#sbor_cvk> {
                        ::sbor::ValueKind::Tuple
                    }
                }

                impl #impl_generics ::sbor::SborTuple <#sbor_cvk> for #ident #ty_generics #where_clause {
                    fn get_length(&self) -> usize {
                        #field_count
                    }
                }
            }
        }
        Data::Enum(DataEnum { variants, .. }) => {
            let discriminator_mapping = get_variant_discriminator_mapping(&attrs, &variants)?;
            let (discriminator_match_arms, field_count_match_arms): (Vec<_>, Vec<_>) = variants
                .iter()
                .enumerate()
                .map(|(i, v)| {
                    let v_id = &v.ident;
                    let discriminator = &discriminator_mapping[&i];

                    let FieldsData {
                        unskipped_field_count,
                        empty_fields_unpacking,
                        ..
                    } = process_fields_for_encode(&v.fields)?;
                    Ok((
                        quote! { Self::#v_id #empty_fields_unpacking => #discriminator, },
                        quote! { Self::#v_id #empty_fields_unpacking => #unskipped_field_count, },
                    ))
                })
                .collect::<Result<Vec<_>>>()?
                .into_iter()
                .unzip();

            let discriminator_match = if discriminator_match_arms.len() > 0 {
                quote! {
                    match self {
                        #(#discriminator_match_arms)*
                    }
                }
            } else {
                quote! { 0 }
            };

            let field_count_match = if field_count_match_arms.len() > 0 {
                quote! {
                    match self {
                        #(#field_count_match_arms)*
                    }
                }
            } else {
                quote! { 0 }
            };

            quote! {
                impl #impl_generics ::sbor::Categorize <#sbor_cvk> for #ident #ty_generics #where_clause {
                    #[inline]
                    fn value_kind() -> ::sbor::ValueKind <#sbor_cvk> {
                        ::sbor::ValueKind::Enum
                    }
                }

                impl #impl_generics ::sbor::SborEnum <#sbor_cvk> for #ident #ty_generics #where_clause {
                    fn get_discriminator(&self) -> u8 {
                        #discriminator_match
                    }

                    fn get_length(&self) -> usize {
                        #field_count_match
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

fn handle_transparent_categorize(
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
    let (impl_generics, ty_generics, where_clause, sbor_cvk) =
        build_custom_categorize_generic(&generics, &attrs, context_custom_value_kind, true)?;
    let output = match data {
        Data::Struct(s) => {
            let FieldsData {
                unskipped_field_names,
                unskipped_field_types,
                ..
            } = process_fields_for_categorize(&s.fields)?;
            if unskipped_field_types.len() != 1 {
                return Err(Error::new(Span::call_site(), "The transparent attribute is only supported for structs with a single unskipped field."));
            }
            let field_type = &unskipped_field_types[0];
            let field_name = &unskipped_field_names[0];

            let categorize_impl = quote! {
                impl #impl_generics ::sbor::Categorize <#sbor_cvk> for #ident #ty_generics #where_clause {
                    #[inline]
                    fn value_kind() -> ::sbor::ValueKind <#sbor_cvk> {
                        <#field_type as ::sbor::Categorize::<#sbor_cvk>>::value_kind()
                    }
                }
            };

            // Dependent SborTuple impl:
            // We'd like to just say "where #field_type: ::sbor::SborTuple" - but this doesn't work because of
            // https://github.com/rust-lang/rust/issues/48214#issuecomment-1374378038
            // Instead we can use that T: SborTuple => &T: SborTuple to apply a constraint on &T: SborTuple instead

            // Rebuild the generic parameters without requiring categorize on generic parameters
            let (impl_generics, ty_generics, where_clause, sbor_cvk) =
                build_custom_categorize_generic(
                    &generics,
                    &attrs,
                    context_custom_value_kind,
                    false,
                )?;

            let tuple_where_clause = add_where_predicate(
                where_clause,
                parse_quote!(for<'b_> &'b_ #field_type: ::sbor::SborTuple <#sbor_cvk>),
            );

            let dependent_sbor_tuple_impl = quote! {
                impl #impl_generics ::sbor::SborTuple <#sbor_cvk> for #ident #ty_generics #tuple_where_clause {
                    fn get_length(&self) -> usize {
                        <&#field_type as ::sbor::SborTuple <#sbor_cvk>>::get_length(&&self.#field_name)
                    }
                }
            };

            let enum_where_clause = add_where_predicate(
                where_clause,
                parse_quote!(for<'b_> &'b_ #field_type: ::sbor::SborEnum <#sbor_cvk>),
            );

            let dependent_sbor_enum_impl = quote! {
                impl #impl_generics ::sbor::SborEnum <#sbor_cvk> for #ident #ty_generics #enum_where_clause {
                    fn get_discriminator(&self) -> u8 {
                        <&#field_type as ::sbor::SborEnum <#sbor_cvk>>::get_discriminator(&&self.#field_name)
                    }

                    fn get_length(&self) -> usize {
                        <&#field_type as ::sbor::SborEnum <#sbor_cvk>>::get_length(&&self.#field_name)
                    }
                }
            };

            quote! {
                #categorize_impl

                #dependent_sbor_tuple_impl

                #dependent_sbor_enum_impl
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

#[cfg(test)]
mod tests {
    use proc_macro2::TokenStream;
    use std::str::FromStr;

    use super::*;

    fn assert_code_eq(a: TokenStream, b: TokenStream) {
        assert_eq!(a.to_string(), b.to_string());
    }

    #[test]
    fn test_categorize_struct() {
        let input = TokenStream::from_str("struct Test {a: u32}").unwrap();
        let output = handle_categorize(input, None).unwrap();

        assert_code_eq(
            output,
            quote! {
                impl <X: ::sbor::CustomValueKind> ::sbor::Categorize<X> for Test {
                    #[inline]
                    fn value_kind() -> ::sbor::ValueKind<X> {
                        ::sbor::ValueKind::Tuple
                    }
                }

                impl<X: ::sbor::CustomValueKind> ::sbor::SborTuple<X> for Test {
                    fn get_length(&self) -> usize {
                        1usize
                    }
                }
            },
        );
    }

    #[test]
    fn test_categorize_transparent_struct() {
        let input =
            TokenStream::from_str("#[sbor(transparent)] struct Test {a: u32, #[sbor(skip)]b: u16}")
                .unwrap();
        let output = handle_categorize(input, None).unwrap();

        assert_code_eq(
            output,
            quote! {
                impl <X: ::sbor::CustomValueKind> ::sbor::Categorize<X> for Test {
                    #[inline]
                    fn value_kind() -> ::sbor::ValueKind<X> {
                        <u32 as ::sbor::Categorize::<X>>::value_kind()
                    }
                }

                impl<X: ::sbor::CustomValueKind> ::sbor::SborTuple<X> for Test
                    where for <'b_> &'b_ u32: ::sbor::SborTuple<X>
                {
                    fn get_length(&self) -> usize {
                        <&u32 as ::sbor::SborTuple<X>>::get_length(&&self.a)
                    }
                }

                impl<X: ::sbor::CustomValueKind> ::sbor::SborEnum<X> for Test
                    where for <'b_> &'b_ u32: ::sbor::SborEnum<X>
                {
                    fn get_discriminator(&self) -> u8 {
                        <&u32 as ::sbor::SborEnum<X>>::get_discriminator(&&self.a)
                    }

                    fn get_length(&self) -> usize {
                        <&u32 as ::sbor::SborEnum<X>>::get_length(&&self.a)
                    }
                }
            },
        );
    }

    #[test]
    fn test_categorize_struct_generics() {
        let input = TokenStream::from_str("struct Test<A> {a: A}").unwrap();
        let output = handle_categorize(input, None).unwrap();

        assert_code_eq(
            output,
            quote! {
                impl <A, X: ::sbor::CustomValueKind> ::sbor::Categorize<X> for Test<A> {
                    #[inline]
                    fn value_kind() -> ::sbor::ValueKind<X> {
                        ::sbor::ValueKind::Tuple
                    }
                }

                impl<A, X: ::sbor::CustomValueKind> ::sbor::SborTuple<X> for Test<A> {
                    fn get_length(&self) -> usize {
                        1usize
                    }
                }
            },
        );
    }

    #[test]
    fn test_categorize_transparent_struct_generics() {
        let input = TokenStream::from_str("#[sbor(transparent)] struct Test<A> {a: A}").unwrap();
        let output = handle_categorize(input, None).unwrap();

        assert_code_eq(
            output,
            quote! {
                impl <A: ::sbor::Categorize<X>, X: ::sbor::CustomValueKind> ::sbor::Categorize<X> for Test<A> {
                    #[inline]
                    fn value_kind() -> ::sbor::ValueKind<X> {
                        <A as ::sbor::Categorize::<X>>::value_kind()
                    }
                }

                impl<A, X: ::sbor::CustomValueKind> ::sbor::SborTuple<X> for Test<A>
                    where for <'b_> &'b_ A: ::sbor::SborTuple<X>
                {
                    fn get_length(&self) -> usize {
                        <&A as ::sbor::SborTuple<X>>::get_length(&&self.a)
                    }
                }

                impl<A, X: ::sbor::CustomValueKind> ::sbor::SborEnum<X> for Test<A>
                    where for <'b_> &'b_ A: ::sbor::SborEnum<X>
                {
                    fn get_discriminator(&self) -> u8 {
                        <&A as ::sbor::SborEnum<X>>::get_discriminator(&&self.a)
                    }

                    fn get_length(&self) -> usize {
                        <&A as ::sbor::SborEnum<X>>::get_length(&&self.a)
                    }
                }
            },
        );
    }

    #[test]
    fn test_categorize_enum() {
        let input = TokenStream::from_str("enum Test {A, B (u32), C {x: u8}}").unwrap();
        let output = handle_categorize(input, None).unwrap();

        assert_code_eq(
            output,
            quote! {
                impl <X: ::sbor::CustomValueKind> ::sbor::Categorize<X> for Test {
                    #[inline]
                    fn value_kind() -> ::sbor::ValueKind<X> {
                        ::sbor::ValueKind::Enum
                    }
                }

                impl <X: ::sbor::CustomValueKind>  ::sbor::SborEnum<X> for Test {
                    fn get_discriminator(&self) -> u8 {
                        match self {
                            Self::A => 0u8,
                            Self::B(_) => 1u8,
                            Self::C { .. } => 2u8,
                        }
                    }

                    fn get_length(&self) -> usize {
                        match self {
                            Self::A => 0,
                            Self::B(_) => 1,
                            Self::C { .. } => 1,
                        }
                    }
                }
            },
        );
    }
}
