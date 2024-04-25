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

    let output = match get_derive_strategy(&parsed.attrs)? {
        DeriveStrategy::Normal => handle_normal_categorize(parsed, context_custom_value_kind)?,
        DeriveStrategy::Transparent => {
            handle_transparent_categorize(parsed, context_custom_value_kind)?
        }
        DeriveStrategy::DeriveAs {
            as_type, as_ref, ..
        } => handle_categorize_as(parsed, context_custom_value_kind, &as_type, &as_ref)?,
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
        build_categorize_generics(&generics, &attrs, context_custom_value_kind)?;

    let output = match data {
        Data::Struct(s) => {
            let FieldsData {
                unskipped_field_names,
                ..
            } = process_fields(&s.fields)?;
            let field_count = unskipped_field_names.len();
            quote! {
                impl #impl_generics sbor::Categorize <#sbor_cvk> for #ident #ty_generics #where_clause {
                    #[inline]
                    fn value_kind() -> sbor::ValueKind <#sbor_cvk> {
                        sbor::ValueKind::Tuple
                    }
                }

                impl #impl_generics sbor::SborTuple <#sbor_cvk> for #ident #ty_generics #where_clause {
                    fn get_length(&self) -> usize {
                        #field_count
                    }
                }
            }
        }
        Data::Enum(DataEnum { variants, .. }) => {
            let EnumVariantsData {
                source_variants, ..
            } = process_enum_variants(&attrs, &variants)?;
            let (discriminator_match_arms, field_count_match_arms): (Vec<_>, Vec<_>) = source_variants
                .iter()
                .map(|source_variant| {
                    match source_variant {
                        SourceVariantData::Reachable(VariantData { source_variant, discriminator, fields_data, .. }) => {
                            let v_id = &source_variant.ident;
                            let FieldsData {
                                unskipped_field_count,
                                empty_fields_unpacking,
                                ..
                            } = &fields_data;
                            (
                                quote! { Self::#v_id #empty_fields_unpacking => #discriminator, },
                                quote! { Self::#v_id #empty_fields_unpacking => #unskipped_field_count, },
                            )
                        },
                        SourceVariantData::Unreachable(UnreachableVariantData { source_variant, fields_data, ..}) => {
                            let v_id = &source_variant.ident;
                            let FieldsData {
                                empty_fields_unpacking,
                                ..
                            } = &fields_data;
                            let panic_message = format!("Variant {} ignored as unreachable", v_id.to_string());
                            (
                                quote! { Self::#v_id #empty_fields_unpacking => panic!(#panic_message), },
                                quote! { Self::#v_id #empty_fields_unpacking => panic!(#panic_message), },
                            )
                        },
                    }
                })
                .collect::<Vec<_>>()
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
                impl #impl_generics sbor::Categorize <#sbor_cvk> for #ident #ty_generics #where_clause {
                    #[inline]
                    fn value_kind() -> sbor::ValueKind <#sbor_cvk> {
                        sbor::ValueKind::Enum
                    }
                }

                impl #impl_generics sbor::SborEnum <#sbor_cvk> for #ident #ty_generics #where_clause {
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
    let DeriveInput { data, .. } = &parsed;
    match data {
        Data::Struct(s) => {
            let FieldsData {
                unskipped_field_names,
                unskipped_field_types,
                ..
            } = process_fields(&s.fields)?;
            if unskipped_field_types.len() != 1 {
                return Err(Error::new(Span::call_site(), "The transparent attribute is only supported for structs with a single unskipped field."));
            }
            let field_type = &unskipped_field_types[0];
            let field_name = &unskipped_field_names[0];

            handle_categorize_as(
                parsed,
                context_custom_value_kind,
                field_type,
                &quote! { &self.#field_name }
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

/// This requires that Categorize is implemented for the "as" type.
/// This ensure that e.g. attempting to derive Categorize on `TransparentStruct(sbor::Value)` fails.
///
/// If we have `<T> TransparentStruct(T)` then the user can use `#[sbor(categorize_as = "T")]`
/// to make the `Categorize` implementation on `TransparentStruct` conditional on `T: Categorize`.
///
/// It also implements SborTuple / SborEnum, but only conditionally - i.e. only if
/// they're implemented for the "as" type.
fn handle_categorize_as(
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
    let (impl_generics, ty_generics, where_clause, sbor_cvk) =
        build_categorize_generics(&generics, &attrs, context_custom_value_kind)?;

    // First - Explict impl of Categorize

    let categorize_bound_type =
        get_type_requiring_categorize_bound_for_categorize_as(as_type, &attrs, &generics)?;

    let categorize_where_clause = if let Some(categorize_bound_type) = categorize_bound_type {
        Some(add_where_predicate(
            where_clause,
            parse_quote!(#categorize_bound_type: sbor::Categorize <#sbor_cvk>),
        ))
    } else {
        where_clause.cloned()
    };

    let categorize_impl = quote! {
        impl #impl_generics sbor::Categorize <#sbor_cvk> for #ident #ty_generics #categorize_where_clause {
            #[inline]
            fn value_kind() -> sbor::ValueKind <#sbor_cvk> {
                <#as_type as sbor::Categorize::<#sbor_cvk>>::value_kind()
            }
        }
    };

    // Dependent implementations of X = SborTuple / SborEnum.
    //
    // We'd like to implement X for the type if and only if it is implemented for `as_type`.
    //
    // We'd like to just say "where #as_type: X" - but this doesn't work because of
    // https://github.com/rust-lang/rust/issues/48214#issuecomment-1374378038
    //
    // Basically - these bounds are either trivially true or false, so the compiler "helpfully" reports a
    // compile error to the user, because such a bound is clearly a mistake (or maybe because the compiler
    // needs some work to actually support them!)
    //
    // Instead we can use that for each of X, if T: X then we have implemented X for &T...
    // And it turns out that the constrant &T: X is not trivial enough to cause the compiler to complain.
    //
    // So we can just cheat and use the implementation from `&T` instead!

    let tuple_where_clause = add_where_predicate(
        where_clause,
        parse_quote!(for<'b_> &'b_ #as_type: sbor::SborTuple <#sbor_cvk>),
    );

    let dependent_sbor_tuple_impl = quote! {
        impl #impl_generics sbor::SborTuple <#sbor_cvk> for #ident #ty_generics #tuple_where_clause {
            fn get_length(&self) -> usize {
                <&#as_type as sbor::SborTuple <#sbor_cvk>>::get_length(&#as_ref_code)
            }
        }
    };

    let enum_where_clause = add_where_predicate(
        where_clause,
        parse_quote!(for<'b_> &'b_ #as_type: sbor::SborEnum <#sbor_cvk>),
    );

    let dependent_sbor_enum_impl = quote! {
        impl #impl_generics sbor::SborEnum <#sbor_cvk> for #ident #ty_generics #enum_where_clause {
            fn get_discriminator(&self) -> u8 {
                <&#as_type as sbor::SborEnum <#sbor_cvk>>::get_discriminator(&#as_ref_code)
            }

            fn get_length(&self) -> usize {
                <&#as_type as sbor::SborEnum <#sbor_cvk>>::get_length(&#as_ref_code)
            }
        }
    };

    Ok(quote! {
        #categorize_impl

        #dependent_sbor_tuple_impl

        #dependent_sbor_enum_impl
    })
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
                impl <X: sbor::CustomValueKind> sbor::Categorize<X> for Test {
                    #[inline]
                    fn value_kind() -> sbor::ValueKind<X> {
                        sbor::ValueKind::Tuple
                    }
                }

                impl<X: sbor::CustomValueKind> sbor::SborTuple<X> for Test {
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
                impl <X: sbor::CustomValueKind> sbor::Categorize<X> for Test
                {
                    #[inline]
                    fn value_kind() -> sbor::ValueKind<X> {
                        <u32 as sbor::Categorize::<X>>::value_kind()
                    }
                }

                impl<X: sbor::CustomValueKind> sbor::SborTuple<X> for Test
                    where for <'b_> &'b_ u32: sbor::SborTuple<X>
                {
                    fn get_length(&self) -> usize {
                        <&u32 as sbor::SborTuple<X>>::get_length(& &self.a)
                    }
                }

                impl<X: sbor::CustomValueKind> sbor::SborEnum<X> for Test
                    where for <'b_> &'b_ u32: sbor::SborEnum<X>
                {
                    fn get_discriminator(&self) -> u8 {
                        <&u32 as sbor::SborEnum<X>>::get_discriminator(& &self.a)
                    }

                    fn get_length(&self) -> usize {
                        <&u32 as sbor::SborEnum<X>>::get_length(& &self.a)
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
                impl <A, X: sbor::CustomValueKind> sbor::Categorize<X> for Test<A> {
                    #[inline]
                    fn value_kind() -> sbor::ValueKind<X> {
                        sbor::ValueKind::Tuple
                    }
                }

                impl<A, X: sbor::CustomValueKind> sbor::SborTuple<X> for Test<A> {
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
                impl <A, X: sbor::CustomValueKind> sbor::Categorize<X> for Test<A>
                    where A: sbor::Categorize<X>
                {
                    #[inline]
                    fn value_kind() -> sbor::ValueKind<X> {
                        <A as sbor::Categorize::<X>>::value_kind()
                    }
                }

                impl<A, X: sbor::CustomValueKind> sbor::SborTuple<X> for Test<A>
                    where for <'b_> &'b_ A: sbor::SborTuple<X>
                {
                    fn get_length(&self) -> usize {
                        <&A as sbor::SborTuple<X>>::get_length(& &self.a)
                    }
                }

                impl<A, X: sbor::CustomValueKind> sbor::SborEnum<X> for Test<A>
                    where for <'b_> &'b_ A: sbor::SborEnum<X>
                {
                    fn get_discriminator(&self) -> u8 {
                        <&A as sbor::SborEnum<X>>::get_discriminator(& &self.a)
                    }

                    fn get_length(&self) -> usize {
                        <&A as sbor::SborEnum<X>>::get_length(& &self.a)
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
                impl <X: sbor::CustomValueKind> sbor::Categorize<X> for Test {
                    #[inline]
                    fn value_kind() -> sbor::ValueKind<X> {
                        sbor::ValueKind::Enum
                    }
                }

                impl <X: sbor::CustomValueKind>  sbor::SborEnum<X> for Test {
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
