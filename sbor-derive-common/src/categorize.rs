use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::*;

use crate::{decode::decode_unique_unskipped_field_from_value, utils::*};

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
            let fields_data = process_fields(&s.fields)?;
            let unskipped_field_count = fields_data.unskipped_field_count();
            quote! {
                impl #impl_generics sbor::Categorize <#sbor_cvk> for #ident #ty_generics #where_clause {
                    #[inline]
                    fn value_kind() -> sbor::ValueKind <#sbor_cvk> {
                        sbor::ValueKind::Tuple
                    }
                }

                impl #impl_generics sbor::SborTuple <#sbor_cvk> for #ident #ty_generics #where_clause {
                    fn get_length(&self) -> usize {
                        #unskipped_field_count
                    }
                }
            }
        }
        Data::Enum(DataEnum { variants, .. }) => {
            let EnumVariantsData {
                source_variants, ..
            } = process_enum_variants(&attrs, &variants)?;

            let mut variant_traits = vec![];

            let (discriminator_match_arms, field_count_match_arms): (Vec<_>, Vec<_>) = source_variants
                .iter()
                .map(|source_variant| -> Result<_> {
                    let output = match source_variant {
                        SourceVariantData::Reachable(VariantData {
                            variant_name,
                            discriminator,
                            fields_handling: FieldsHandling::Standard(fields_data),
                            impl_variant_trait,
                            ..
                        }) => {
                            if *impl_variant_trait {
                                variant_traits.push(handle_impl_variant_trait(
                                    &ident,
                                    &impl_generics,
                                    &sbor_cvk,
                                    &ty_generics,
                                    where_clause,
                                    variant_name,
                                    discriminator,
                                    fields_data,
                                    false,
                                )?);
                            }
                            let unskipped_field_count = fields_data.unskipped_field_count();
                            let empty_fields_unpacking = fields_data.empty_fields_unpacking();
                            (
                                quote! { Self::#variant_name #empty_fields_unpacking => #discriminator, },
                                quote! { Self::#variant_name #empty_fields_unpacking => #unskipped_field_count, },
                            )
                        },
                        SourceVariantData::Reachable(VariantData {
                            variant_name,
                            discriminator,
                            fields_handling: FieldsHandling::Flatten {
                                unique_field,
                                fields_data,
                            },
                            impl_variant_trait,
                            ..
                        }) => {
                            if *impl_variant_trait {
                                variant_traits.push(handle_impl_variant_trait(
                                    &ident,
                                    &impl_generics,
                                    &sbor_cvk,
                                    &ty_generics,
                                    where_clause,
                                    variant_name,
                                    discriminator,
                                    fields_data,
                                    true,
                                )?);
                            }
                            let empty_fields_unpacking = fields_data.empty_fields_unpacking();
                            let fields_unpacking = fields_data.fields_unpacking();
                            let unskipped_field_type = unique_field.field_type();
                            let unpacking_variable_name = unique_field.variable_name_from_unpacking();
                            (
                                quote! { Self::#variant_name #empty_fields_unpacking => #discriminator, },
                                quote! { Self::#variant_name #fields_unpacking => <#unskipped_field_type as SborTuple<#sbor_cvk>>::get_length(#unpacking_variable_name), },
                            )
                        },
                        SourceVariantData::Unreachable(UnreachableVariantData { variant_name, fields_data, ..}) => {
                            let empty_fields_unpacking = fields_data.empty_fields_unpacking();
                            let panic_message = format!("Variant {} ignored as unreachable", variant_name.to_string());
                            (
                                quote! { Self::#variant_name #empty_fields_unpacking => panic!(#panic_message), },
                                quote! { Self::#variant_name #empty_fields_unpacking => panic!(#panic_message), },
                            )
                        },
                    };
                    Ok(output)
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

                #(#variant_traits)*
            }
        }
        Data::Union(_) => {
            return Err(Error::new(Span::call_site(), "Union is not supported!"));
        }
    };

    Ok(output)
}

fn handle_impl_variant_trait(
    enum_name: &Ident,
    impl_generics: &Generics,
    sbor_cvk: &Path,
    type_generics: &TypeGenerics,
    where_clause: Option<&WhereClause>,
    variant_name: &Ident,
    discriminator: &Expr,
    fields_data: &FieldsData,
    is_flattened: bool,
) -> Result<TokenStream> {
    let unique_type = fields_data.unique_unskipped_field().ok_or_else(|| {
        Error::new(
            variant_name.span(),
            "impl_variant_trait is active but this variant does not have a single unskipped field",
        )
    })?;
    let variant_type_name = unique_type.field_type();
    let middle = if is_flattened {
        quote! {
            const IS_FLATTENED: bool = true;

            type VariantFields = Self;
            fn from_variant_fields(variant_fields: Self::VariantFields) -> Self {
                variant_fields
            }

            type VariantFieldsRef<'a> = &'a Self;
            fn as_variant_fields_ref(&self) -> Self::VariantFieldsRef<'_> {
                self
            }
        }
    } else {
        quote! {
            const IS_FLATTENED: bool = false;

            type VariantFields = (Self,);
            fn from_variant_fields(variant_fields: Self::VariantFields) -> Self {
                variant_fields.0
            }

            type VariantFieldsRef<'a> = (&'a Self,);
            fn as_variant_fields_ref(&self) -> Self::VariantFieldsRef<'_> {
                (self,)
            }
        }
    };
    let into_enum = decode_unique_unskipped_field_from_value(
        quote! { #enum_name::#variant_name },
        fields_data,
    )?;
    let type_generics_turbofish = type_generics.as_turbofish();
    let output = quote! {
        impl #impl_generics sbor::SborEnumVariantFor<#enum_name #type_generics_turbofish, #sbor_cvk> for #variant_type_name #where_clause {
            const DISCRIMINATOR: u8 = #discriminator;
            #middle
            type OwnedVariant = sbor::SborFixedEnumVariant<{ #discriminator }, Self::VariantFields>;
            type BorrowedVariant<'a> = sbor::SborFixedEnumVariant<{ #discriminator }, Self::VariantFieldsRef<'a>>;

            fn into_enum(self) -> #enum_name {
                let value = self;
                #into_enum
            }
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
            let single_field = process_fields(&s.fields)?
                .unique_unskipped_field()
                .ok_or_else(|| Error::new(
                    Span::call_site(),
                    "The transparent attribute is only supported for structs with a single unskipped field.",
                ))?;

            handle_categorize_as(
                parsed,
                context_custom_value_kind,
                single_field.field_type(),
                &single_field.self_field_reference(),
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
                            Self::A => 0usize,
                            Self::B(_) => 1usize,
                            Self::C { .. } => 1usize,
                        }
                    }
                }
            },
        );
    }
}
