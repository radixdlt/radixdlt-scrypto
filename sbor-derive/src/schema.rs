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

pub fn handle_schema(input: TokenStream) -> Result<TokenStream> {
    trace!("handle_schema() starts");

    let code_hash = get_code_hash_const_array_token_stream(&input);

    let DeriveInput {
        attrs,
        ident,
        data,
        generics,
        ..
    } = parse2(input)?;
    let (impl_generics, ty_generics, where_clause, custom_type_schema_generic) =
        build_schema_generics(&generics, &attrs)?;

    let generic_type_idents = ty_generics
        .type_params()
        .map(|t| &t.ident)
        .collect::<Vec<_>>();

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
                    impl #impl_generics ::sbor::Schema <#custom_type_schema_generic> for #ident #ty_generics #where_clause {
                        const SCHEMA_TYPE_REF: ::sbor::GlobalTypeRef = ::sbor::GlobalTypeRef::complex_with_code(
                            stringify!(#ident),
                            // Here we really want to cause distinct types to have distinct hashes, whilst still supporting (most) recursive types.
                            // The code hash itself is pretty good for this, but if you allow generic types, it's not enough, as the same code can create
                            // different types depending on the generic types providing. Adding in the generic types' SCHEMA_TYPE_REFs solves that issue.
                            //
                            // It's still technically possible to get a collision (by abusing type namespacing to have two types with identical code
                            // reference other types) but it's good enough - you're only shooting yourself in the food at that point.
                            //
                            // Note that it might seem possible to still hit issues with infinite recursion, if you pass a type as its own generic type parameter.
                            // EG (via a type alias B = A<B>), but these types won't come up in practice because they require an infinite generic depth
                            // which the compiler will throw out for other reasons.
                            &[#(<#generic_type_idents>::SCHEMA_TYPE_REF,)*],
                            &#code_hash
                        );

                        fn get_local_type_data() -> Option<::sbor::LocalTypeData<C, ::sbor::GlobalTypeRef>> {
                            Some(::sbor::LocalTypeData::named_fields_tuple(
                                stringify!(#ident),
                                ::sbor::rust::vec![
                                    #((#field_names, <#field_types as ::sbor::Schema<C>>::SCHEMA_TYPE_REF),)*
                                ],
                            ))
                        }

                        fn add_all_dependencies(aggregator: &mut ::sbor::SchemaAggregator<C>) {
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
                    impl #impl_generics ::sbor::Schema <#custom_type_schema_generic> for #ident #ty_generics #where_clause {
                        const SCHEMA_TYPE_REF: ::sbor::GlobalTypeRef = ::sbor::GlobalTypeRef::complex_with_code(
                            stringify!(#ident),
                            // Here we really want to cause distinct types to have distinct hashes, whilst still supporting (most) recursive types.
                            // The code hash itself is pretty good for this, but if you allow generic types, it's not enough, as the same code can create
                            // different types depending on the generic types providing. Adding in the generic types' SCHEMA_TYPE_REFs solves that issue.
                            //
                            // It's still technically possible to get a collision (by abusing type namespacing to have two types with identical code
                            // reference other types) but it's good enough - you're only shooting yourself in the food at that point.
                            //
                            // Note that it might seem possible to still hit issues with infinite recursion, if you pass a type as its own generic type parameter.
                            // EG (via a type alias B = A<B>), but these types won't come up in practice because they require an infinite generic depth
                            // which the compiler will throw out for other reasons.
                            &[#(#generic_type_idents::SCHEMA_TYPE_REF,)*],
                            &#code_hash
                        );

                        fn get_local_type_data() -> Option<::sbor::LocalTypeData<C, ::sbor::GlobalTypeRef>> {
                            Some(::sbor::LocalTypeData::named_tuple(
                                stringify!(#ident),
                                ::sbor::rust::vec![
                                    #(<#field_types as ::sbor::Schema<C>>::SCHEMA_TYPE_REF,)*
                                ],
                            ))
                        }

                        fn add_all_dependencies(aggregator: &mut ::sbor::SchemaAggregator<C>) {
                            #(aggregator.add_child_type_and_descendents::<#unique_field_types>();)*
                        }
                    }
                }
            }
            syn::Fields::Unit => {
                quote! {
                    impl #impl_generics ::sbor::Schema <#custom_type_schema_generic> for #ident #ty_generics #where_clause {
                        const SCHEMA_TYPE_REF: ::sbor::GlobalTypeRef = ::sbor::GlobalTypeRef::complex_with_code(
                            stringify!(#ident),
                            &[#(#generic_type_idents::SCHEMA_TYPE_REF,)*],
                            &#code_hash
                        );

                        fn get_local_type_data() -> Option<::sbor::LocalTypeData<C, ::sbor::GlobalTypeRef>> {
                            Some(::sbor::LocalTypeData::named_unit(stringify!(#ident)))
                        }
                    }
                }
            }
        },
        Data::Enum(DataEnum { variants, .. }) => {
            let variant_names: Vec<_> = variants.iter().map(|v| v.ident.to_string()).collect();
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
                                    ::sbor::LocalTypeData::named_fields_tuple(
                                        #variant_name,
                                        ::sbor::rust::vec![
                                            #((#field_names, <#field_types as ::sbor::Schema<C>>::SCHEMA_TYPE_REF),)*
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
                                    ::sbor::LocalTypeData::named_tuple(
                                        #variant_name,
                                        ::sbor::rust::vec![
                                            #(<#field_types as ::sbor::Schema<C>>::SCHEMA_TYPE_REF,)*
                                        ],
                                    )
                                }
                            }
                            Fields::Unit => {
                                quote! {
                                    ::sbor::LocalTypeData::named_unit(#variant_name)
                                }
                            }
                        }
                    })
                    .collect()
            };

            let unique_field_types: Vec<_> = get_unique_types(&all_field_types);

            quote! {
                impl #impl_generics ::sbor::Schema <#custom_type_schema_generic> for #ident #ty_generics #where_clause {
                    const SCHEMA_TYPE_REF: ::sbor::GlobalTypeRef = ::sbor::GlobalTypeRef::complex_with_code(
                        stringify!(#ident),
                        &[#(#generic_type_idents::SCHEMA_TYPE_REF,)*],
                        &#code_hash
                    );

                    fn get_local_type_data() -> Option<::sbor::LocalTypeData<C, ::sbor::GlobalTypeRef>> {
                        use ::sbor::rust::borrow::ToOwned;
                        Some(::sbor::LocalTypeData::named_enum(
                            stringify!(#ident),
                            ::sbor::rust::collections::btree_map::btreemap![
                                #(#variant_names.to_owned() => #variant_type_data,)*
                            ],
                        ))
                    }

                    fn add_all_dependencies(aggregator: &mut ::sbor::SchemaAggregator<C>) {
                        #(aggregator.add_child_type_and_descendents::<#unique_field_types>();)*
                    }
                }
            }
        }
        Data::Union(_) => {
            return Err(Error::new(Span::call_site(), "Union is not supported!"));
        }
    };

    #[cfg(feature = "trace")]
    crate::utils::print_generated_code("Schema", &output);

    trace!("handle_schema() finishes");
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
        let output = handle_schema(input).unwrap();

        assert_code_eq(
            output,
            quote! {
                impl <C: ::sbor::CustomTypeSchema> ::sbor::Schema<C> for Test {
                    const SCHEMA_TYPE_REF: ::sbor::GlobalTypeRef = ::sbor::GlobalTypeRef::complex_with_code(
                        stringify!(Test),
                        &[],
                        &[63u8, 255u8, 173u8, 220u8, 251u8, 214u8, 95u8, 139u8, 106u8, 20u8, 23u8, 4u8, 15u8, 10u8, 124u8, 49u8, 219u8, 44u8, 235u8, 215u8]
                    );

                    fn get_local_type_data() -> Option<::sbor::LocalTypeData <C, ::sbor::GlobalTypeRef>> {
                        Some(::sbor::LocalTypeData::named_fields_tuple(
                            stringify!(Test),
                            ::sbor::rust::vec![
                                ("a", <u32 as ::sbor::Schema<C>>::SCHEMA_TYPE_REF),
                                ("b", <Vec<u8> as ::sbor::Schema<C>>::SCHEMA_TYPE_REF),
                                ("c", <u32 as ::sbor::Schema<C>>::SCHEMA_TYPE_REF),
                            ],
                        ))
                    }

                    fn add_all_dependencies(aggregator: &mut ::sbor::SchemaAggregator<C>) {
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
        let output = handle_schema(input).unwrap();

        assert_code_eq(
            output,
            quote! {
                impl <C: ::sbor::CustomTypeSchema> ::sbor::Schema<C> for Test {
                    const SCHEMA_TYPE_REF: ::sbor::GlobalTypeRef = ::sbor::GlobalTypeRef::complex_with_code(
                        stringify!(Test),
                        &[],
                        &[85u8, 53u8, 15u8, 85u8, 176u8, 230u8, 4u8, 110u8, 15u8, 96u8, 35u8, 64u8, 192u8, 210u8, 254u8, 146u8, 192u8, 7u8, 246u8, 5u8]
                    );

                    fn get_local_type_data() -> Option<::sbor::LocalTypeData <C, ::sbor::GlobalTypeRef>> {
                        Some(::sbor::LocalTypeData::named_tuple(
                            stringify!(Test),
                            ::sbor::rust::vec![
                                <u32 as ::sbor::Schema<C>>::SCHEMA_TYPE_REF,
                                <Vec<u8> as ::sbor::Schema<C>>::SCHEMA_TYPE_REF,
                                <u32 as ::sbor::Schema<C>>::SCHEMA_TYPE_REF,
                            ],
                        ))
                    }

                    fn add_all_dependencies(aggregator: &mut ::sbor::SchemaAggregator<C>) {
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
        let output = handle_schema(input).unwrap();

        assert_code_eq(
            output,
            quote! {
                impl <C: ::sbor::CustomTypeSchema> ::sbor::Schema<C> for Test {
                    const SCHEMA_TYPE_REF: ::sbor::GlobalTypeRef = ::sbor::GlobalTypeRef::complex_with_code(
                        stringify!(Test),
                        &[],
                        &[167u8, 108u8, 181u8, 130u8, 168u8, 229u8, 85u8, 237u8, 66u8, 69u8, 34u8, 138u8, 113u8, 220u8, 225u8, 107u8, 0u8, 247u8, 189u8, 58u8]
                    );

                    fn get_local_type_data() -> Option<::sbor::LocalTypeData <C, ::sbor::GlobalTypeRef>> {
                        Some(::sbor::LocalTypeData::named_unit(stringify!(Test)))
                    }
                }
            },
        );
    }

    #[test]
    fn test_complex_enum_schema() {
        let input =
            TokenStream::from_str("#[sbor(generic_type_id_bounds = \"T2\")] enum Test<T: SomeTrait, T2> {A, B (T, Vec<T2>, #[sbor(skip)] i32), C {x: [u8; 5]}}").unwrap();
        let output = handle_schema(input).unwrap();

        assert_code_eq(
            output,
            quote! {
                impl <T: SomeTrait + ::sbor::Schema<C>, T2: ::sbor::Schema<C> + ::sbor::TypeId<C::CustomTypeId>, C: ::sbor::CustomTypeSchema> ::sbor::Schema<C> for Test<T, T2> {
                    const SCHEMA_TYPE_REF: ::sbor::GlobalTypeRef = ::sbor::GlobalTypeRef::complex_with_code(
                        stringify!(Test),
                        &[T::SCHEMA_TYPE_REF, T2::SCHEMA_TYPE_REF,],
                        &[211u8, 164u8, 57u8, 227u8, 220u8, 74u8, 90u8, 141u8, 72u8, 27u8, 35u8, 85u8, 171u8, 13u8, 176u8, 124u8, 122u8, 28u8, 53u8, 105u8]
                    );

                    fn get_local_type_data() -> Option<::sbor::LocalTypeData <C, ::sbor::GlobalTypeRef>> {
                        use ::sbor::rust::borrow::ToOwned;
                        Some(::sbor::LocalTypeData::named_enum(
                            stringify!(Test),
                            ::sbor::rust::collections::btree_map::btreemap![
                                "A".to_owned() => ::sbor::LocalTypeData::named_unit("A"),
                                "B".to_owned() => ::sbor::LocalTypeData::named_tuple(
                                    "B",
                                    ::sbor::rust::vec![
                                        <T as ::sbor::Schema<C>>::SCHEMA_TYPE_REF,
                                        <Vec<T2> as ::sbor::Schema<C>>::SCHEMA_TYPE_REF,
                                    ],
                                ),
                                "C".to_owned() => ::sbor::LocalTypeData::named_fields_tuple(
                                    "C",
                                    ::sbor::rust::vec![
                                        ("x", <[u8; 5] as ::sbor::Schema<C>>::SCHEMA_TYPE_REF),
                                    ],
                                ),
                            ],
                        ))
                    }

                    fn add_all_dependencies(aggregator: &mut ::sbor::SchemaAggregator<C>) {
                        aggregator.add_child_type_and_descendents::<T>();
                        aggregator.add_child_type_and_descendents::<Vec<T2> >();
                        aggregator.add_child_type_and_descendents::<[u8; 5]>();
                    }
                }
            },
        );
    }
}
