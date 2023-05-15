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

pub fn handle_describe(
    input: TokenStream,
    context_custom_type_kind: Option<&'static str>,
) -> Result<TokenStream> {
    trace!("handle_describe() starts");

    let code_hash = get_code_hash_const_array_token_stream(&input);

    let parsed: DeriveInput = parse2(input)?;
    let is_transparent = is_transparent(&parsed.attrs)?;

    let output = if is_transparent {
        handle_transparent_describe(parsed, code_hash, context_custom_type_kind)?
    } else {
        handle_normal_describe(parsed, code_hash, context_custom_type_kind)?
    };

    #[cfg(feature = "trace")]
    crate::utils::print_generated_code("Describe", &output);

    trace!("handle_describe() finishes");
    Ok(output)
}

fn handle_transparent_describe(
    parsed: DeriveInput,
    code_hash: TokenStream,
    context_custom_type_kind: Option<&'static str>,
) -> Result<TokenStream> {
    let DeriveInput {
        attrs,
        ident,
        data,
        generics,
        ..
    } = parsed;
    let (impl_generics, ty_generics, where_clause, _, custom_type_kind_generic) =
        build_describe_generics(&generics, &attrs, context_custom_type_kind)?;

    let output = match data {
        Data::Struct(s) => {
            let FieldsData {
                unskipped_field_types,
                ..
            } = process_fields_for_describe(&s.fields)?;

            if unskipped_field_types.len() != 1 {
                return Err(Error::new(Span::call_site(), "The transparent attribute is only supported for structs with a single unskipped field."));
            }

            let field_type = &unskipped_field_types[0];

            let mut type_data_content = quote! {
                <#field_type as ::sbor::Describe <#custom_type_kind_generic>>::type_data()
            };
            let mut type_id = quote! {
                <#field_type as ::sbor::Describe <#custom_type_kind_generic>>::TYPE_ID
            };

            // Replace the type name, unless opted out using the "transparent_name" tag
            if !get_sbor_bool_value(&attrs, "transparent_name")? {
                type_data_content = quote! {
                    use ::sbor::rust::prelude::*;
                    #type_data_content
                        .with_name(Some(Cow::Borrowed(stringify!(#ident))))
                };
                type_id = quote! {
                    ::sbor::GlobalTypeId::novel_with_code(
                        stringify!(#ident),
                        &[#type_id],
                        &#code_hash
                    )
                };
            };

            quote! {
                impl #impl_generics ::sbor::Describe <#custom_type_kind_generic> for #ident #ty_generics #where_clause {
                    const TYPE_ID: ::sbor::GlobalTypeId = #type_id;

                    fn type_data() -> ::sbor::TypeData<#custom_type_kind_generic, ::sbor::GlobalTypeId> {
                        #type_data_content
                    }

                    fn add_all_dependencies(aggregator: &mut ::sbor::TypeAggregator<#custom_type_kind_generic>) {
                        <#field_type as ::sbor::Describe <#custom_type_kind_generic>>::add_all_dependencies(aggregator)
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

fn handle_normal_describe(
    parsed: DeriveInput,
    code_hash: TokenStream,
    context_custom_type_kind: Option<&'static str>,
) -> Result<TokenStream> {
    let DeriveInput {
        attrs,
        ident,
        data,
        generics,
        ..
    } = parsed;
    let (impl_generics, ty_generics, where_clause, child_types, custom_type_kind_generic) =
        build_describe_generics(&generics, &attrs, context_custom_type_kind)?;

    let output = match data {
        Data::Struct(s) => match &s.fields {
            syn::Fields::Named(FieldsNamed { .. }) => {
                let FieldsData {
                    unskipped_field_types,
                    unskipped_field_name_strings,
                    ..
                } = process_fields_for_describe(&s.fields)?;
                let unique_field_types: Vec<_> = get_unique_types(&unskipped_field_types);
                quote! {
                    impl #impl_generics ::sbor::Describe <#custom_type_kind_generic> for #ident #ty_generics #where_clause {
                        const TYPE_ID: ::sbor::GlobalTypeId = ::sbor::GlobalTypeId::novel_with_code(
                            stringify!(#ident),
                            // Here we really want to cause distinct types to have distinct hashes, whilst still supporting (most) recursive types.
                            // The code hash itself is pretty good for this, but if you allow generic types, it's not enough, as the same code can create
                            // different types depending on the generic types providing. Adding in the generic types' TYPE_IDs solves that issue.
                            //
                            // It's still technically possible to get a collision (by abusing type namespacing to have two types with identical code
                            // reference other types) but it's good enough - you're only shooting yourself in the food at that point.
                            //
                            // Note that it might seem possible to still hit issues with infinite recursion, if you pass a type as its own generic type parameter.
                            // EG (via a type alias B = A<B>), but these types won't come up in practice because they require an infinite generic depth
                            // which the compiler will throw out for other reasons.
                            &[#(<#child_types>::TYPE_ID,)*],
                            &#code_hash
                        );

                        fn type_data() -> ::sbor::TypeData<#custom_type_kind_generic, ::sbor::GlobalTypeId> {
                            ::sbor::TypeData::struct_with_named_fields(
                                stringify!(#ident),
                                ::sbor::rust::vec![
                                    #((#unskipped_field_name_strings, <#unskipped_field_types as ::sbor::Describe<#custom_type_kind_generic>>::TYPE_ID),)*
                                ],
                            )
                        }

                        fn add_all_dependencies(aggregator: &mut ::sbor::TypeAggregator<#custom_type_kind_generic>) {
                            #(aggregator.add_child_type_and_descendents::<#unique_field_types>();)*
                        }
                    }
                }
            }
            syn::Fields::Unnamed(FieldsUnnamed { .. }) => {
                let FieldsData {
                    unskipped_field_types,
                    ..
                } = process_fields_for_describe(&s.fields)?;
                let unique_field_types: Vec<_> = get_unique_types(&unskipped_field_types);

                quote! {
                    impl #impl_generics ::sbor::Describe <#custom_type_kind_generic> for #ident #ty_generics #where_clause {
                        const TYPE_ID: ::sbor::GlobalTypeId = ::sbor::GlobalTypeId::novel_with_code(
                            stringify!(#ident),
                            // Here we really want to cause distinct types to have distinct hashes, whilst still supporting (most) recursive types.
                            // The code hash itself is pretty good for this, but if you allow generic types, it's not enough, as the same code can create
                            // different types depending on the generic types providing. Adding in the generic types' TYPE_IDs solves that issue.
                            //
                            // It's still technically possible to get a collision (by abusing type namespacing to have two types with identical code
                            // reference other types) but it's good enough - you're only shooting yourself in the food at that point.
                            //
                            // Note that it might seem possible to still hit issues with infinite recursion, if you pass a type as its own generic type parameter.
                            // EG (via a type alias B = A<B>), but these types won't come up in practice because they require an infinite generic depth
                            // which the compiler will throw out for other reasons.
                            &[#(<#child_types>::TYPE_ID,)*],
                            &#code_hash
                        );

                        fn type_data() -> ::sbor::TypeData<#custom_type_kind_generic, ::sbor::GlobalTypeId> {
                            ::sbor::TypeData::struct_with_unnamed_fields(
                                stringify!(#ident),
                                ::sbor::rust::vec![
                                    #(<#unskipped_field_types as ::sbor::Describe<#custom_type_kind_generic>>::TYPE_ID,)*
                                ],
                            )
                        }

                        fn add_all_dependencies(aggregator: &mut ::sbor::TypeAggregator<#custom_type_kind_generic>) {
                            #(aggregator.add_child_type_and_descendents::<#unique_field_types>();)*
                        }
                    }
                }
            }
            syn::Fields::Unit => {
                quote! {
                    impl #impl_generics ::sbor::Describe <#custom_type_kind_generic> for #ident #ty_generics #where_clause {
                        const TYPE_ID: ::sbor::GlobalTypeId = ::sbor::GlobalTypeId::novel_with_code(
                            stringify!(#ident),
                            &[#(<#child_types>::TYPE_ID,)*],
                            &#code_hash
                        );

                        fn type_data() -> ::sbor::TypeData<#custom_type_kind_generic, ::sbor::GlobalTypeId> {
                            ::sbor::TypeData::struct_with_unit_fields(stringify!(#ident))
                        }
                    }
                }
            }
        },
        Data::Enum(DataEnum { variants, .. }) => {
            let discriminator_mapping = get_variant_discriminator_mapping(&attrs, &variants)?;
            let variant_discriminators = (0..variants.len())
                .into_iter()
                .map(|i| &discriminator_mapping[&i])
                .collect::<Vec<_>>();
            let mut all_field_types = Vec::new();

            let variant_type_data: Vec<_> = {
                variants
                    .iter()
                    .map(|v| {
                        let variant_name = v.ident.to_string();
                        let FieldsData {
                            unskipped_field_types,
                            unskipped_field_name_strings,
                            ..
                        } = process_fields_for_describe(&v.fields)?;
                        all_field_types.extend_from_slice(&unskipped_field_types);
                        Ok(match &v.fields {
                            Fields::Named(FieldsNamed { .. }) => {
                                quote! {
                                    ::sbor::TypeData::struct_with_named_fields(
                                        #variant_name,
                                        ::sbor::rust::vec![
                                            #((#unskipped_field_name_strings, <#unskipped_field_types as ::sbor::Describe<#custom_type_kind_generic>>::TYPE_ID),)*
                                        ],
                                    )
                                }
                            }
                            Fields::Unnamed(FieldsUnnamed { .. }) => {
                                quote! {
                                    ::sbor::TypeData::struct_with_unnamed_fields(
                                        #variant_name,
                                        ::sbor::rust::vec![
                                            #(<#unskipped_field_types as ::sbor::Describe<#custom_type_kind_generic>>::TYPE_ID,)*
                                        ],
                                    )
                                }
                            }
                            Fields::Unit => {
                                quote! {
                                    ::sbor::TypeData::struct_with_unit_fields(#variant_name)
                                }
                            }
                        })
                    })
                    .collect::<Result<_>>()?
            };

            let unique_field_types = get_unique_types(&all_field_types);

            quote! {
                impl #impl_generics ::sbor::Describe <#custom_type_kind_generic> for #ident #ty_generics #where_clause {
                    const TYPE_ID: ::sbor::GlobalTypeId = ::sbor::GlobalTypeId::novel_with_code(
                        stringify!(#ident),
                        &[#(<#child_types>::TYPE_ID,)*],
                        &#code_hash
                    );

                    fn type_data() -> ::sbor::TypeData<#custom_type_kind_generic, ::sbor::GlobalTypeId> {
                        use ::sbor::rust::borrow::ToOwned;
                        ::sbor::TypeData::enum_variants(
                            stringify!(#ident),
                            ::sbor::rust::collections::btree_map::btreemap![
                                #(#variant_discriminators => #variant_type_data,)*
                            ],
                        )
                    }

                    fn add_all_dependencies(aggregator: &mut ::sbor::TypeAggregator<#custom_type_kind_generic>) {
                        #(aggregator.add_child_type_and_descendents::<#unique_field_types>();)*
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

#[cfg(test)]
mod tests {
    use proc_macro2::TokenStream;
    use std::str::FromStr;

    use super::*;

    fn assert_code_eq(a: TokenStream, b: TokenStream) {
        assert_eq!(a.to_string(), b.to_string());
    }

    #[test]
    fn test_named_field_struct_schema() {
        let input = TokenStream::from_str("struct Test {a: u32, b: Vec<u8>, c: u32}").unwrap();
        let code_hash = get_code_hash_const_array_token_stream(&input);
        let output = handle_describe(input, None).unwrap();

        assert_code_eq(
            output,
            quote! {
                impl <C: ::sbor::CustomTypeKind<::sbor::GlobalTypeId> > ::sbor::Describe<C> for Test {
                    const TYPE_ID: ::sbor::GlobalTypeId = ::sbor::GlobalTypeId::novel_with_code(
                        stringify!(Test),
                        &[],
                        &#code_hash
                    );

                    fn type_data() -> ::sbor::TypeData <C, ::sbor::GlobalTypeId> {
                        ::sbor::TypeData::struct_with_named_fields(
                            stringify!(Test),
                            ::sbor::rust::vec![
                                ("a", <u32 as ::sbor::Describe<C>>::TYPE_ID),
                                ("b", <Vec<u8> as ::sbor::Describe<C>>::TYPE_ID),
                                ("c", <u32 as ::sbor::Describe<C>>::TYPE_ID),
                            ],
                        )
                    }

                    fn add_all_dependencies(aggregator: &mut ::sbor::TypeAggregator<C>) {
                        aggregator.add_child_type_and_descendents::<u32>();
                        aggregator.add_child_type_and_descendents::<Vec<u8> >();
                    }
                }
            },
        );
    }

    #[test]
    fn test_named_field_struct_schema_custom() {
        let input = TokenStream::from_str("struct Test {a: u32, b: Vec<u8>, c: u32}").unwrap();
        let code_hash = get_code_hash_const_array_token_stream(&input);
        let output = handle_describe(
            input,
            Some("radix_engine_interface::data::ScryptoCustomTypeKind<::sbor::GlobalTypeId>"),
        )
        .unwrap();

        assert_code_eq(
            output,
            quote! {
                impl ::sbor::Describe<radix_engine_interface::data::ScryptoCustomTypeKind<::sbor::GlobalTypeId> >
                    for Test
                {
                    const TYPE_ID: ::sbor::GlobalTypeId = ::sbor::GlobalTypeId::novel_with_code(
                        stringify!(Test),
                        &[],
                        &#code_hash
                    );
                    fn type_data() ->
                        ::sbor::TypeData<
                            radix_engine_interface::data::ScryptoCustomTypeKind<::sbor::GlobalTypeId>,
                            ::sbor::GlobalTypeId> {
                        ::sbor::TypeData::struct_with_named_fields(
                            stringify!(Test),
                            ::sbor::rust::vec![
                                (
                                    "a",
                                    <u32 as ::sbor::Describe<
                                        radix_engine_interface::data::ScryptoCustomTypeKind<::sbor::GlobalTypeId>
                                    >>::TYPE_ID
                                ),
                                (
                                    "b",
                                    <Vec<u8> as ::sbor::Describe<
                                        radix_engine_interface::data::ScryptoCustomTypeKind<::sbor::GlobalTypeId>
                                    >>::TYPE_ID
                                ),
                                (
                                    "c",
                                    <u32 as ::sbor::Describe<
                                        radix_engine_interface::data::ScryptoCustomTypeKind<::sbor::GlobalTypeId>
                                    >>::TYPE_ID
                                ),
                            ],
                        )
                    }
                    fn add_all_dependencies(
                        aggregator: &mut ::sbor::TypeAggregator<
                            radix_engine_interface::data::ScryptoCustomTypeKind<::sbor::GlobalTypeId>
                        >
                    ) {
                        aggregator.add_child_type_and_descendents::<u32>();
                        aggregator.add_child_type_and_descendents::<Vec<u8> >();
                    }
                }
            },
        );
    }

    #[test]
    fn test_unnamed_field_struct_schema() {
        let input = TokenStream::from_str("struct Test(u32, Vec<u8>, u32);").unwrap();
        let code_hash = get_code_hash_const_array_token_stream(&input);
        let output = handle_describe(input, None).unwrap();

        assert_code_eq(
            output,
            quote! {
                impl <C: ::sbor::CustomTypeKind<::sbor::GlobalTypeId> > ::sbor::Describe<C> for Test {
                    const TYPE_ID: ::sbor::GlobalTypeId = ::sbor::GlobalTypeId::novel_with_code(
                        stringify!(Test),
                        &[],
                        &#code_hash
                    );

                    fn type_data() -> ::sbor::TypeData <C, ::sbor::GlobalTypeId> {
                        ::sbor::TypeData::struct_with_unnamed_fields(
                            stringify!(Test),
                            ::sbor::rust::vec![
                                <u32 as ::sbor::Describe<C>>::TYPE_ID,
                                <Vec<u8> as ::sbor::Describe<C>>::TYPE_ID,
                                <u32 as ::sbor::Describe<C>>::TYPE_ID,
                            ],
                        )
                    }

                    fn add_all_dependencies(aggregator: &mut ::sbor::TypeAggregator<C>) {
                        aggregator.add_child_type_and_descendents::<u32>();
                        aggregator.add_child_type_and_descendents::<Vec<u8> >();
                    }
                }
            },
        );
    }

    #[test]
    fn test_unit_struct_schema() {
        let input = TokenStream::from_str("struct Test;").unwrap();
        let code_hash = get_code_hash_const_array_token_stream(&input);
        let output = handle_describe(input, None).unwrap();

        assert_code_eq(
            output,
            quote! {
                impl <C: ::sbor::CustomTypeKind<::sbor::GlobalTypeId> > ::sbor::Describe<C> for Test {
                    const TYPE_ID: ::sbor::GlobalTypeId = ::sbor::GlobalTypeId::novel_with_code(
                        stringify!(Test),
                        &[],
                        &#code_hash
                    );

                    fn type_data() -> ::sbor::TypeData <C, ::sbor::GlobalTypeId> {
                        ::sbor::TypeData::struct_with_unit_fields(stringify!(Test))
                    }
                }
            },
        );
    }

    #[test]
    fn test_complex_enum_schema() {
        let input =
            TokenStream::from_str("#[sbor(categorize_types = \"T2\")] enum Test<T: SomeTrait, T2> {A, B (T, Vec<T2>, #[sbor(skip)] i32), C {x: [u8; 5]}}").unwrap();
        let code_hash = get_code_hash_const_array_token_stream(&input);
        let output = handle_describe(input, None).unwrap();

        assert_code_eq(
            output,
            quote! {
                impl <T: SomeTrait, T2, C: ::sbor::CustomTypeKind<::sbor::GlobalTypeId> > ::sbor::Describe<C> for Test<T, T2>
                where
                    T: ::sbor::Describe<C>,
                    T2: ::sbor::Describe<C>
                {
                    const TYPE_ID: ::sbor::GlobalTypeId = ::sbor::GlobalTypeId::novel_with_code(
                        stringify!(Test),
                        &[<T>::TYPE_ID, <T2>::TYPE_ID,],
                        &#code_hash
                    );

                    fn type_data() -> ::sbor::TypeData <C, ::sbor::GlobalTypeId> {
                        use ::sbor::rust::borrow::ToOwned;
                        ::sbor::TypeData::enum_variants(
                            stringify!(Test),
                            ::sbor::rust::collections::btree_map::btreemap![
                                0u8 => ::sbor::TypeData::struct_with_unit_fields("A"),
                                1u8 => ::sbor::TypeData::struct_with_unnamed_fields(
                                    "B",
                                    ::sbor::rust::vec![
                                        <T as ::sbor::Describe<C>>::TYPE_ID,
                                        <Vec<T2> as ::sbor::Describe<C>>::TYPE_ID,
                                    ],
                                ),
                                2u8 => ::sbor::TypeData::struct_with_named_fields(
                                    "C",
                                    ::sbor::rust::vec![
                                        ("x", <[u8; 5] as ::sbor::Describe<C>>::TYPE_ID),
                                    ],
                                ),
                            ],
                        )
                    }

                    fn add_all_dependencies(aggregator: &mut ::sbor::TypeAggregator<C>) {
                        aggregator.add_child_type_and_descendents::<T>();
                        aggregator.add_child_type_and_descendents::<Vec<T2> >();
                        aggregator.add_child_type_and_descendents::<[u8; 5]>();
                    }
                }
            },
        );
    }
}
