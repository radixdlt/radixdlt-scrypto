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
    let custom_type_schema = custom_type_schema(&attrs);
    let (impl_generics, ty_generics, where_clause, custom_type_schema_generic) =
        build_schema_generics(&generics, custom_type_schema)?;

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
                            &[#(#generic_type_idents::SCHEMA_TYPE_REF,)*],
                            &#code_hash
                        );

                        fn get_local_type_data() -> Option<::sbor::LocalTypeData<C, ::sbor::GlobalTypeRef>> {
                            Some(::sbor::LocalTypeData::named_fields_tuple(
                                stringify!(#ident),
                                vec![
                                    #(<#field_types>::SCHEMA_TYPE_REF,)*
                                ],
                                vec![
                                    #(#field_names.to_owned(),)*
                                ],
                            ))
                        }

                        fn add_all_dependencies(aggregator: &mut ::sbor::SchemaAggregator<C>) {
                            #(aggregator.add_child_type_and_descendents::<#generic_type_idents>();)*
                        }
                    }
                }
            }
            syn::Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                let unskipped_fields: Vec<&Field> =
                    unnamed.iter().filter(|f| !is_encoding_skipped(f)).collect();
                let field_types: Vec<_> = unskipped_fields.iter().map(|f| &f.ty).collect();

                quote! {
                    impl #impl_generics ::sbor::Schema <#custom_type_schema_generic> for #ident #ty_generics #where_clause {
                        const SCHEMA_TYPE_REF: ::sbor::GlobalTypeRef = ::sbor::GlobalTypeRef::complex_with_code(
                            stringify!(#ident),
                            &[#(#generic_type_idents::SCHEMA_TYPE_REF,)*],
                            &#code_hash
                        );

                        fn get_local_type_data() -> Option<::sbor::LocalTypeData<C, ::sbor::GlobalTypeRef>> {
                            Some(::sbor::LocalTypeData::named_tuple(
                                stringify!(#ident),
                                vec![
                                    #(<#field_types>::SCHEMA_TYPE_REF,)*
                                ],
                            ))
                        }

                        fn add_all_dependencies(aggregator: &mut ::sbor::SchemaAggregator<C>) {
                            #(aggregator.add_child_type_and_descendents::<#generic_type_idents>();)*
                        }
                    }
                }
            }
            syn::Fields::Unit => {
                quote! {
                    impl #impl_generics ::sbor::Schema <#custom_type_schema_generic> for #ident #ty_generics #where_clause {
                        const SCHEMA_TYPE_REF: ::sbor::GlobalTypeRef = ::sbor::GlobalTypeRef::complex_with_code(
                            stringify!(#ident),
                            &[],
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

            let variant_type_refs: Vec<_> =
                {
                    variants.iter().map(|v| {
                    let variant_name = v.ident.to_string();
                    quote! {
                        ::sbor::GlobalTypeRef::complex(#variant_name, &[Self::SCHEMA_TYPE_REF])
                    }
                })
                .collect()
                };
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
                                        vec![
                                            #(<#field_types>::SCHEMA_TYPE_REF,)*
                                        ],
                                        vec![
                                            #(#field_names.to_owned(),)*
                                        ],
                                    )
                                }
                            }
                            Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                                let unskipped_fields: Vec<&Field> =
                                    unnamed.iter().filter(|f| !is_encoding_skipped(f)).collect();
                                let field_types: Vec<_> =
                                    unskipped_fields.iter().map(|f| &f.ty).collect();
                                quote! {
                                    ::sbor::LocalTypeData::named_tuple(
                                        #variant_name,
                                        vec![
                                            #(<#field_types>::SCHEMA_TYPE_REF,)*
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

            quote! {
                impl #impl_generics ::sbor::Schema <#custom_type_schema_generic> for #ident #ty_generics #where_clause {
                    const SCHEMA_TYPE_REF: ::sbor::GlobalTypeRef = ::sbor::GlobalTypeRef::complex_with_code(
                        stringify!(#ident),
                        &[#(#generic_type_idents::SCHEMA_TYPE_REF,)*],
                        &#code_hash
                    );

                    fn get_local_type_data() -> Option<::sbor::LocalTypeData<C, ::sbor::GlobalTypeRef>> {
                        Some(::sbor::LocalTypeData::named_enum(
                            stringify!(#ident),
                            ::sbor::rust::collections::btree_map::btreemap![
                                #(#variant_names.to_owned() => #variant_names.to_owned(),)*
                            ],
                            ::sbor::rust::collections::btree_map::btreemap![
                                #(#variant_names.to_owned() => #variant_type_refs,)*
                            ],
                        ))
                    }

                    fn add_all_dependencies(aggregator: &mut ::sbor::SchemaAggregator<C>) {
                        // Add types for the enum variants
                        #(
                            aggregator.add_child_type(
                                #variant_type_refs,
                                || Some(#variant_type_data)
                            );
                        )*
                        // Add the generic type descendents
                        #(aggregator.add_child_type_and_descendents::<#generic_type_idents>();)*
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
    fn test_encode_struct() {
        let input = TokenStream::from_str("struct Test {a: u32}").unwrap();
        let output = handle_schema(input).unwrap();

        assert_code_eq(
            output,
            quote! {
                impl <C: ::sbor::CustomTypeSchema> ::sbor::Schema<C> for Test {
                    const SCHEMA_TYPE_REF: ::sbor::GlobalTypeRef = ::sbor::GlobalTypeRef::complex_with_code(
                        stringify!(Test),
                        &[],
                        &[199u8, 68u8, 83u8, 62u8, 239u8, 118u8, 79u8, 50u8, 26u8, 160u8, 164u8, 229u8, 19u8, 137u8, 68u8, 170u8, 4u8, 70u8, 35u8, 86u8]
                    );

                    fn get_local_type_data() -> Option<::sbor::LocalTypeData <C, ::sbor::GlobalTypeRef>> {
                        Some(::sbor::LocalTypeData::named_tuple_named_fields(
                            stringify!(Test),
                            vec![
                                <u32>::SCHEMA_TYPE_REF,
                            ],
                            &[
                                "a",
                            ],
                        ))
                    }

                    fn add_all_dependencies(aggregator: &mut ::sbor::SchemaAggregator<C>) { }
                }
            },
        );
    }

    #[test]
    fn test_encode_enum() {
        let input =
            TokenStream::from_str("enum Test<T: SomeTrait> {A, B (T), C {x: [u8; 5]}}").unwrap();
        let output = handle_schema(input).unwrap();

        assert_code_eq(
            output,
            quote! {
                impl <T: SomeTrait + ::sbor::Schema<C>, C: ::sbor::CustomTypeSchema> ::sbor::Schema<C> for Test<T> {
                    const SCHEMA_TYPE_REF: ::sbor::GlobalTypeRef = ::sbor::GlobalTypeRef::complex_with_code(
                        stringify!(Test),
                        &[T::SCHEMA_TYPE_REF,],
                        &[39u8, 90u8, 96u8, 95u8, 44u8, 58u8, 217u8, 111u8, 89u8, 23u8, 189u8, 8u8, 81u8, 137u8, 165u8, 47u8, 224u8, 216u8, 203u8, 240u8]
                    );

                    fn get_local_type_data() -> Option<::sbor::LocalTypeData <C, ::sbor::GlobalTypeRef>> {
                        Some(::sbor::LocalTypeData::named_enum(
                            stringify!(Test),
                            ::sbor::rust::collections::btree_map::btreemap![
                                "A".to_owned() => "A".to_owned(),
                                "B".to_owned() => "B".to_owned(),
                                "C".to_owned() => "C".to_owned(),
                            ],
                            ::sbor::rust::collections::btree_map::btreemap![
                                "A".to_owned() => ::sbor::GlobalTypeRef::complex("A", &[Self::SCHEMA_TYPE_REF]),
                                "B".to_owned() => ::sbor::GlobalTypeRef::complex("B", &[Self::SCHEMA_TYPE_REF]),
                                "C".to_owned() => ::sbor::GlobalTypeRef::complex("C", &[Self::SCHEMA_TYPE_REF]),
                            ],
                        ))
                    }

                    fn add_all_dependencies(aggregator: &mut ::sbor::SchemaAggregator<C>) {
                        aggregator.add_child_type(
                            ::sbor::GlobalTypeRef::complex("A", &[Self::SCHEMA_TYPE_REF]),
                            || Some(::sbor::LocalTypeData::named_unit("A"))
                        );
                        aggregator.add_child_type(
                            ::sbor::GlobalTypeRef::complex("B", &[Self::SCHEMA_TYPE_REF]),
                            || Some(::sbor::LocalTypeData::named_tuple(
                                "B",
                                vec![
                                    <T>::SCHEMA_TYPE_REF,
                                ],
                            ))
                        );
                        aggregator.add_child_type(
                            ::sbor::GlobalTypeRef::complex("C", &[Self::SCHEMA_TYPE_REF]),
                            || Some(::sbor::LocalTypeData::named_fields_tuple(
                                "C",
                                vec![
                                    <[u8; 5]>::SCHEMA_TYPE_REF,
                                ],
                                vec![
                                    "x".to_owned(),
                                ],
                            ))
                        );
                        aggregator.add_child_type_and_descendents::<T>();
                    }
                }
            },
        );
    }
}
