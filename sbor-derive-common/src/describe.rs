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
    let is_transparent = is_transparent(&parsed.attrs);

    let output = if is_transparent {
        handle_transparent_describe(parsed, context_custom_type_kind)?
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
            } = process_fields_for_describe(&s.fields);

            if unskipped_field_types.len() != 1 {
                return Err(Error::new(Span::call_site(), "The transparent attribute is only supported for structs with a single unskipped field."));
            }

            let field_type = &unskipped_field_types[0];

            quote! {
                impl #impl_generics ::sbor::Describe <#custom_type_kind_generic> for #ident #ty_generics #where_clause {
                    const TYPE_ID: ::sbor::GlobalTypeId = <#field_type as ::sbor::Describe <#custom_type_kind_generic>>::TYPE_ID;

                    fn type_data() -> Option<::sbor::TypeData<#custom_type_kind_generic, ::sbor::GlobalTypeId>> {
                        <#field_type as ::sbor::Describe <#custom_type_kind_generic>>::type_data()
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
        Data::Struct(s) => match s.fields {
            syn::Fields::Named(FieldsNamed { named, .. }) => {
                let unskipped_fields: Vec<&Field> =
                    named.iter().filter(|f| !is_encoding_skipped(f)).collect();
                let field_types: Vec<_> = unskipped_fields.iter().map(|f| &f.ty).collect();
                let unique_field_types: Vec<_> = get_unique_types(&field_types);
                let field_names: Vec<_> = unskipped_fields
                    .iter()
                    .map(|f| {
                        f.ident
                            .as_ref()
                            .expect("All fields expected to be named")
                            .to_string()
                    })
                    .collect();
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

                        fn type_data() -> Option<::sbor::TypeData<#custom_type_kind_generic, ::sbor::GlobalTypeId>> {
                            Some(::sbor::TypeData::named_fields_tuple(
                                stringify!(#ident),
                                ::sbor::rust::vec![
                                    #((#field_names, <#field_types as ::sbor::Describe<#custom_type_kind_generic>>::TYPE_ID),)*
                                ],
                            ))
                        }

                        fn add_all_dependencies(aggregator: &mut ::sbor::TypeAggregator<#custom_type_kind_generic>) {
                            #(aggregator.add_child_type_and_descendents::<#unique_field_types>();)*
                        }
                    }
                }
            }
            syn::Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                let unskipped_fields: Vec<&Field> =
                    unnamed.iter().filter(|f| !is_encoding_skipped(f)).collect();
                let field_types: Vec<_> = unskipped_fields.iter().map(|f| &f.ty).collect();
                let unique_field_types: Vec<_> = get_unique_types(&field_types);

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

                        fn type_data() -> Option<::sbor::TypeData<#custom_type_kind_generic, ::sbor::GlobalTypeId>> {
                            Some(::sbor::TypeData::named_tuple(
                                stringify!(#ident),
                                ::sbor::rust::vec![
                                    #(<#field_types as ::sbor::Describe<#custom_type_kind_generic>>::TYPE_ID,)*
                                ],
                            ))
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

                        fn type_data() -> Option<::sbor::TypeData<#custom_type_kind_generic, ::sbor::GlobalTypeId>> {
                            Some(::sbor::TypeData::named_unit(stringify!(#ident)))
                        }
                    }
                }
            }
        },
        Data::Enum(DataEnum { variants, .. }) => {
            let n: u8 = variants
                .len()
                .try_into()
                .expect("Too many variants in enum");
            let variant_indices: Vec<u8> = (0..n).into_iter().collect();
            let mut all_field_types = Vec::new();

            let variant_type_data: Vec<_> = {
                variants
                    .iter()
                    .map(|v| {
                        let variant_name = v.ident.to_string();
                        match &v.fields {
                            Fields::Named(FieldsNamed { named, .. }) => {
                                let unskipped_fields: Vec<&Field> =
                                    named.iter().filter(|f| !is_encoding_skipped(f)).collect();
                                let field_types: Vec<_> =
                                    unskipped_fields.iter().map(|f| &f.ty).collect();
                                all_field_types.extend_from_slice(&field_types);
                                let field_names: Vec<_> = unskipped_fields
                                    .iter()
                                    .map(|f| {
                                        f.ident
                                            .as_ref()
                                            .expect("All fields expected to be named")
                                            .to_string()
                                    })
                                    .collect();
                                quote! {
                                    ::sbor::TypeData::named_fields_tuple(
                                        #variant_name,
                                        ::sbor::rust::vec![
                                            #((#field_names, <#field_types as ::sbor::Describe<#custom_type_kind_generic>>::TYPE_ID),)*
                                        ],
                                    )
                                }
                            }
                            Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                                let unskipped_fields: Vec<&Field> =
                                    unnamed.iter().filter(|f| !is_encoding_skipped(f)).collect();
                                let field_types: Vec<_> =
                                    unskipped_fields.iter().map(|f| &f.ty).collect();
                                all_field_types.extend_from_slice(&field_types);
                                quote! {
                                    ::sbor::TypeData::named_tuple(
                                        #variant_name,
                                        ::sbor::rust::vec![
                                            #(<#field_types as ::sbor::Describe<#custom_type_kind_generic>>::TYPE_ID,)*
                                        ],
                                    )
                                }
                            }
                            Fields::Unit => {
                                quote! {
                                    ::sbor::TypeData::named_unit(#variant_name)
                                }
                            }
                        }
                    })
                    .collect()
            };

            let unique_field_types: Vec<_> = get_unique_types(&all_field_types);

            quote! {
                impl #impl_generics ::sbor::Describe <#custom_type_kind_generic> for #ident #ty_generics #where_clause {
                    const TYPE_ID: ::sbor::GlobalTypeId = ::sbor::GlobalTypeId::novel_with_code(
                        stringify!(#ident),
                        &[#(<#child_types>::TYPE_ID,)*],
                        &#code_hash
                    );

                    fn type_data() -> Option<::sbor::TypeData<#custom_type_kind_generic, ::sbor::GlobalTypeId>> {
                        use ::sbor::rust::borrow::ToOwned;
                        Some(::sbor::TypeData::named_enum(
                            stringify!(#ident),
                            ::sbor::rust::collections::btree_map::btreemap![
                                #(#variant_indices => #variant_type_data,)*
                            ],
                        ))
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
        let output = handle_describe(input, None).unwrap();

        assert_code_eq(
            output,
            quote! {
                impl <C: ::sbor::CustomTypeKind<::sbor::GlobalTypeId> > ::sbor::Describe<C> for Test {
                    const TYPE_ID: ::sbor::GlobalTypeId = ::sbor::GlobalTypeId::novel_with_code(
                        stringify!(Test),
                        &[],
                        &[159u8, 209u8, 242u8, 94u8, 4u8, 85u8, 29u8, 88u8, 103u8, 42u8, 6u8, 107u8, 55u8, 82u8, 41u8, 37u8, 138u8, 251u8, 115u8, 188u8]
                    );

                    fn type_data() -> Option<::sbor::TypeData <C, ::sbor::GlobalTypeId>> {
                        Some(::sbor::TypeData::named_fields_tuple(
                            stringify!(Test),
                            ::sbor::rust::vec![
                                ("a", <u32 as ::sbor::Describe<C>>::TYPE_ID),
                                ("b", <Vec<u8> as ::sbor::Describe<C>>::TYPE_ID),
                                ("c", <u32 as ::sbor::Describe<C>>::TYPE_ID),
                            ],
                        ))
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
                        &[
                            159u8, 209u8, 242u8, 94u8, 4u8, 85u8, 29u8, 88u8, 103u8, 42u8, 6u8, 107u8, 55u8, 82u8, 41u8, 37u8, 138u8, 251u8, 115u8, 188u8
                        ]
                    );
                    fn type_data() -> Option<
                        ::sbor::TypeData<
                            radix_engine_interface::data::ScryptoCustomTypeKind<::sbor::GlobalTypeId>,
                            ::sbor::GlobalTypeId>> {
                        Some(::sbor::TypeData::named_fields_tuple(
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
                        ))
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
        let output = handle_describe(input, None).unwrap();

        assert_code_eq(
            output,
            quote! {
                impl <C: ::sbor::CustomTypeKind<::sbor::GlobalTypeId> > ::sbor::Describe<C> for Test {
                    const TYPE_ID: ::sbor::GlobalTypeId = ::sbor::GlobalTypeId::novel_with_code(
                        stringify!(Test),
                        &[],
                        &[34u8, 114u8, 203u8, 221u8, 124u8, 31u8, 95u8, 225u8, 232u8, 133u8, 214u8, 82u8, 163u8, 166u8, 47u8, 94u8, 1u8, 45u8, 24u8, 85u8]
                    );

                    fn type_data() -> Option<::sbor::TypeData <C, ::sbor::GlobalTypeId>> {
                        Some(::sbor::TypeData::named_tuple(
                            stringify!(Test),
                            ::sbor::rust::vec![
                                <u32 as ::sbor::Describe<C>>::TYPE_ID,
                                <Vec<u8> as ::sbor::Describe<C>>::TYPE_ID,
                                <u32 as ::sbor::Describe<C>>::TYPE_ID,
                            ],
                        ))
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
        let output = handle_describe(input, None).unwrap();

        assert_code_eq(
            output,
            quote! {
                impl <C: ::sbor::CustomTypeKind<::sbor::GlobalTypeId> > ::sbor::Describe<C> for Test {
                    const TYPE_ID: ::sbor::GlobalTypeId = ::sbor::GlobalTypeId::novel_with_code(
                        stringify!(Test),
                        &[],
                        &[191u8, 160u8, 163u8, 134u8, 72u8, 226u8, 165u8, 105u8, 145u8, 247u8, 102u8, 49u8, 49u8, 92u8, 177u8, 109u8, 66u8, 111u8, 153u8, 93u8]
                    );

                    fn type_data() -> Option<::sbor::TypeData <C, ::sbor::GlobalTypeId>> {
                        Some(::sbor::TypeData::named_unit(stringify!(Test)))
                    }
                }
            },
        );
    }

    #[test]
    fn test_complex_enum_schema() {
        let input =
            TokenStream::from_str("#[sbor(categorize_types = \"T2\")] enum Test<T: SomeTrait, T2> {A, B (T, Vec<T2>, #[sbor(skip)] i32), C {x: [u8; 5]}}").unwrap();
        let output = handle_describe(input, None).unwrap();

        assert_code_eq(
            output,
            quote! {
                impl <T: SomeTrait, T2, C: ::sbor::CustomTypeKind<::sbor::GlobalTypeId> > ::sbor::Describe<C> for Test<T, T2>
                where
                    T: ::sbor::Describe<C>,
                    T2: ::sbor::Describe<C>,
                    T2: ::sbor::Categorize<C::CustomValueKind>
                {
                    const TYPE_ID: ::sbor::GlobalTypeId = ::sbor::GlobalTypeId::novel_with_code(
                        stringify!(Test),
                        &[<T>::TYPE_ID, <T2>::TYPE_ID,],
                        &[53u8, 102u8, 114u8, 88u8, 176u8, 193u8, 133u8, 184u8, 140u8, 124u8, 151u8, 120u8, 108u8, 184u8, 217u8, 14u8, 89u8, 95u8, 159u8, 189u8]
                    );

                    fn type_data() -> Option<::sbor::TypeData <C, ::sbor::GlobalTypeId>> {
                        use ::sbor::rust::borrow::ToOwned;
                        Some(::sbor::TypeData::named_enum(
                            stringify!(Test),
                            ::sbor::rust::collections::btree_map::btreemap![
                                0u8 => ::sbor::TypeData::named_unit("A"),
                                1u8 => ::sbor::TypeData::named_tuple(
                                    "B",
                                    ::sbor::rust::vec![
                                        <T as ::sbor::Describe<C>>::TYPE_ID,
                                        <Vec<T2> as ::sbor::Describe<C>>::TYPE_ID,
                                    ],
                                ),
                                2u8 => ::sbor::TypeData::named_fields_tuple(
                                    "C",
                                    ::sbor::rust::vec![
                                        ("x", <[u8; 5] as ::sbor::Describe<C>>::TYPE_ID),
                                    ],
                                ),
                            ],
                        ))
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
