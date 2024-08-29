use itertools::Itertools as _;
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

    let output = match get_derive_strategy(&parsed.attrs)? {
        DeriveStrategy::Normal => {
            handle_normal_describe(parsed, code_hash, context_custom_type_kind)?
        }
        DeriveStrategy::Transparent => {
            handle_transparent_describe(parsed, code_hash, context_custom_type_kind)?
        }
        DeriveStrategy::DeriveAs { as_type, .. } => {
            handle_describe_as(parsed, context_custom_type_kind, &as_type, code_hash)?
        }
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
    let DeriveInput { data, .. } = &parsed;
    match &data {
        Data::Struct(s) => {
            let single_field = process_fields(&s.fields)?
                .unique_unskipped_field()
                .ok_or_else(|| Error::new(
                    Span::call_site(),
                    "The transparent attribute is only supported for structs with a single unskipped field.",
                ))?;

            handle_describe_as(
                parsed,
                context_custom_type_kind,
                single_field.field_type(),
                code_hash,
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

fn handle_describe_as(
    parsed: DeriveInput,
    context_custom_type_kind: Option<&'static str>,
    as_type: &Type,
    code_hash: TokenStream,
) -> Result<TokenStream> {
    let DeriveInput {
        attrs,
        ident,
        generics,
        ..
    } = &parsed;
    let (impl_generics, ty_generics, where_clause, _, custom_type_kind_generic) =
        build_describe_generics(&generics, &attrs, context_custom_type_kind)?;

    // Prepare base conditions before any overrides
    let mut is_fully_transparent = true;
    let mut type_data_content = quote! {
        <#as_type as sbor::Describe <#custom_type_kind_generic>>::type_data()
    };

    // Perform each override (currently there's just one, this could be expanded in future)
    let override_type_name = if get_sbor_attribute_bool_value(attrs, "transparent_name")?.value() {
        None
    } else {
        Some(resolve_type_name(ident, attrs)?)
    };
    if let Some(new_type_name) = &override_type_name {
        validate_type_name(new_type_name)?;
        is_fully_transparent = false;
        type_data_content = quote! {
            #type_data_content
                .with_name(Some(Cow::Borrowed(#new_type_name)))
        }
    }

    // Calculate the type id to use
    let transparent_type_id = quote! {
        <#as_type as sbor::Describe <#custom_type_kind_generic>>::TYPE_ID
    };

    let type_id = if is_fully_transparent {
        transparent_type_id
    } else {
        let novel_type_name =
            override_type_name.unwrap_or(LitStr::new(&ident.to_string(), ident.span()));
        quote! {
            sbor::RustTypeId::novel_with_code(
                #novel_type_name,
                &[#transparent_type_id],
                &#code_hash
            )
        }
    };

    let output = quote! {
        impl #impl_generics sbor::Describe <#custom_type_kind_generic> for #ident #ty_generics #where_clause {
            const TYPE_ID: sbor::RustTypeId = #type_id;

            fn type_data() -> sbor::TypeData<#custom_type_kind_generic, sbor::RustTypeId> {
                use sbor::rust::prelude::*;
                #type_data_content
            }

            fn add_all_dependencies(aggregator: &mut sbor::TypeAggregator<#custom_type_kind_generic>) {
                <#as_type as sbor::Describe <#custom_type_kind_generic>>::add_all_dependencies(aggregator)
            }
        }
    };

    Ok(output)
}

#[derive(PartialEq, Eq, Hash)]
enum TypeDependency {
    Child(Type),
    DescendentsOnly(Type),
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
    } = &parsed;
    let (impl_generics, ty_generics, where_clause, child_types, custom_type_kind_generic) =
        build_describe_generics(generics, attrs, context_custom_type_kind)?;

    let type_name = resolve_type_name(ident, attrs)?;

    let type_id = quote! {
        sbor::RustTypeId::novel_with_code(
            #type_name,
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
            &[#(<#child_types as sbor::Describe<#custom_type_kind_generic>>::TYPE_ID,)*],
            &#code_hash
        )
    };

    let mut type_dependencies = vec![];

    let type_data = match data {
        Data::Struct(s) => {
            let fields_data = process_fields(&s.fields)?;

            for field_type in fields_data.unskipped_field_types() {
                type_dependencies.push(TypeDependency::Child(field_type));
            }

            fields_data_to_type_data(&custom_type_kind_generic, &type_name, &fields_data)?
        }
        Data::Enum(DataEnum { variants, .. }) => {
            let EnumVariantsData { sbor_variants, .. } = process_enum_variants(&attrs, &variants)?;

            let match_arms = sbor_variants
                .iter()
                .map(|VariantData { variant_name, fields_handling, discriminator, .. }| {
                    let variant_name_str = ident_to_lit_str(variant_name);

                    let variant_type_data = match fields_handling {
                        FieldsHandling::Standard(fields_data) => {
                            for field_type in fields_data.unskipped_field_types() {
                                type_dependencies.push(TypeDependency::Child(field_type));
                            }
                            fields_data_to_type_data(
                                &custom_type_kind_generic,
                                &variant_name_str,
                                &fields_data,
                            )?
                        }
                        FieldsHandling::Flatten { unique_field, .. } => {
                            let flattened_type = unique_field.field_type();

                            // We need to include the flattened type's descendents,
                            // but not the flatenned type itself (we're taking its type details)
                            // and mutating them effectively
                            type_dependencies.push(TypeDependency::DescendentsOnly(flattened_type.clone()));

                            quote! {
                                let mut flattened_type_data = <#flattened_type as sbor::Describe<#custom_type_kind_generic>>::type_data();
                                // Flatten is only valid if the child type is a Tuple, so we must
                                // double-check that constraint.
                                // We can't do an SborTuple<X> assertion here because we don't have a
                                // specific X = custom value kind to check on (e.g. SborSchema is shared
                                // by both ManifestSbor and ScryptoSbor). This assertion will almost certainly
                                // be checked by an Encode/Decode/Categorize implementation, but on the off-chance
                                // if isn't, let's check the type kind is correct in the type itself at
                                // Describe run time.
                                let sbor::schema::TypeKind::Tuple { .. } = &flattened_type_data.kind else {
                                    panic!("The flatten attribute cannot be used with a non-tuple child");
                                };
                                // We rename the tuple type to be the enum variant name,
                                // and this becomes the enum variant type data
                                flattened_type_data.with_name(Some(sbor::rust::prelude::Cow::Borrowed(#variant_name_str)))
                            }
                        }
                    };

                    Ok(quote! {
                        #discriminator => { #variant_type_data },
                    })
                })
                .collect::<Result<Vec<_>>>()?;

            quote! {
                use sbor::rust::borrow::ToOwned;
                sbor::TypeData::enum_variants(
                    #type_name,
                    sbor::rust::prelude::indexmap![
                        #(#match_arms)*
                    ],
                )
            }
        }
        Data::Union(_) => {
            return Err(Error::new(Span::call_site(), "Union is not supported!"));
        }
    };

    let dependencies =
        type_dependencies
            .iter()
            .unique()
            .map(|type_dependency| match type_dependency {
                TypeDependency::Child(child_type) => quote! {
                    aggregator.add_child_type_and_descendents::<#child_type>();
                },
                TypeDependency::DescendentsOnly(descendent_only_type) => quote! {
                    aggregator.add_schema_descendents::<#descendent_only_type>();
                },
            });

    Ok(quote! {
        impl #impl_generics sbor::Describe <#custom_type_kind_generic> for #ident #ty_generics #where_clause {
            const TYPE_ID: sbor::RustTypeId = #type_id;

            fn type_data() -> sbor::TypeData<#custom_type_kind_generic, sbor::RustTypeId> {
                #type_data
            }

            fn add_all_dependencies(aggregator: &mut sbor::TypeAggregator<#custom_type_kind_generic>) {
                #(#dependencies)*
            }
        }
    })
}

fn fields_data_to_type_data(
    custom_type_kind_generic: &Path,
    type_name: &LitStr,
    fields_data: &FieldsData,
) -> Result<TokenStream> {
    let unskipped_field_types = fields_data.unskipped_field_types();

    let type_data = match fields_data {
        FieldsData::Named(fields) => {
            let unskipped_field_name_strings = fields.unskipped_field_name_strings();
            quote! {
                sbor::TypeData::struct_with_named_fields(
                    #type_name,
                    sbor::rust::vec![
                        #((#unskipped_field_name_strings, <#unskipped_field_types as sbor::Describe<#custom_type_kind_generic>>::TYPE_ID),)*
                    ],
                )
            }
        }
        FieldsData::Unnamed(_) => {
            quote! {
                sbor::TypeData::struct_with_unnamed_fields(
                    #type_name,
                    sbor::rust::vec![
                        #(<#unskipped_field_types as sbor::Describe<#custom_type_kind_generic>>::TYPE_ID,)*
                    ],
                )
            }
        }
        FieldsData::Unit => {
            quote! {
                sbor::TypeData::struct_with_unit_fields(#type_name)
            }
        }
    };
    Ok(type_data)
}

pub fn validate_type_name(type_name: &LitStr) -> Result<()> {
    validate_schema_ident("Sbor type names", &type_name.value())
        .map_err(|error_message| Error::new(type_name.span(), error_message))
}

// IMPORTANT:
// For crate dependency regions, this is duplicated from `sbor`
// If you change it here, please change it there as well
fn validate_schema_ident(
    ident_category_name: &str,
    name: &str,
) -> core::result::Result<(), String> {
    if name.len() == 0 {
        return Err(format!("{ident_category_name} cannot be empty"));
    }

    if name.len() > 100 {
        return Err(format!(
            "{ident_category_name} cannot be more than 100 characters"
        ));
    }

    let first_char = name.chars().next().unwrap();
    if !matches!(first_char, 'A'..='Z' | 'a'..='z') {
        return Err(format!(
            "{ident_category_name} must match [A-Za-z][0-9A-Za-z_]{{0,99}}"
        ));
    }

    for char in name.chars() {
        if !matches!(char, '0'..='9' | 'A'..='Z' | 'a'..='z' | '_') {
            return Err(format!(
                "{ident_category_name} must match [A-Za-z][0-9A-Za-z_]{{0,99}}"
            ));
        }
    }
    Ok(())
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
                impl <C: sbor::CustomTypeKind<sbor::RustTypeId> > sbor::Describe<C> for Test {
                    const TYPE_ID: sbor::RustTypeId = sbor::RustTypeId::novel_with_code(
                        "Test",
                        &[],
                        &#code_hash
                    );

                    fn type_data() -> sbor::TypeData <C, sbor::RustTypeId> {
                        sbor::TypeData::struct_with_named_fields(
                            "Test",
                            sbor::rust::vec![
                                ("a", <u32 as sbor::Describe<C>>::TYPE_ID),
                                ("b", <Vec<u8> as sbor::Describe<C>>::TYPE_ID),
                                ("c", <u32 as sbor::Describe<C>>::TYPE_ID),
                            ],
                        )
                    }

                    fn add_all_dependencies(aggregator: &mut sbor::TypeAggregator<C>) {
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
            Some("radix_common::data::ScryptoCustomTypeKind<sbor::RustTypeId>"),
        )
        .unwrap();

        assert_code_eq(
            output,
            quote! {
                impl sbor::Describe<radix_common::data::ScryptoCustomTypeKind<sbor::RustTypeId> >
                    for Test
                {
                    const TYPE_ID: sbor::RustTypeId = sbor::RustTypeId::novel_with_code(
                        "Test",
                        &[],
                        &#code_hash
                    );
                    fn type_data() ->
                        sbor::TypeData<
                            radix_common::data::ScryptoCustomTypeKind<sbor::RustTypeId>,
                            sbor::RustTypeId> {
                        sbor::TypeData::struct_with_named_fields(
                            "Test",
                            sbor::rust::vec![
                                (
                                    "a",
                                    <u32 as sbor::Describe<
                                        radix_common::data::ScryptoCustomTypeKind<sbor::RustTypeId>
                                    >>::TYPE_ID
                                ),
                                (
                                    "b",
                                    <Vec<u8> as sbor::Describe<
                                        radix_common::data::ScryptoCustomTypeKind<sbor::RustTypeId>
                                    >>::TYPE_ID
                                ),
                                (
                                    "c",
                                    <u32 as sbor::Describe<
                                        radix_common::data::ScryptoCustomTypeKind<sbor::RustTypeId>
                                    >>::TYPE_ID
                                ),
                            ],
                        )
                    }
                    fn add_all_dependencies(
                        aggregator: &mut sbor::TypeAggregator<
                            radix_common::data::ScryptoCustomTypeKind<sbor::RustTypeId>
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
                impl <C: sbor::CustomTypeKind<sbor::RustTypeId> > sbor::Describe<C> for Test {
                    const TYPE_ID: sbor::RustTypeId = sbor::RustTypeId::novel_with_code(
                        "Test",
                        &[],
                        &#code_hash
                    );

                    fn type_data() -> sbor::TypeData <C, sbor::RustTypeId> {
                        sbor::TypeData::struct_with_unnamed_fields(
                            "Test",
                            sbor::rust::vec![
                                <u32 as sbor::Describe<C>>::TYPE_ID,
                                <Vec<u8> as sbor::Describe<C>>::TYPE_ID,
                                <u32 as sbor::Describe<C>>::TYPE_ID,
                            ],
                        )
                    }

                    fn add_all_dependencies(aggregator: &mut sbor::TypeAggregator<C>) {
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
                impl <C: sbor::CustomTypeKind<sbor::RustTypeId> > sbor::Describe<C> for Test {
                    const TYPE_ID: sbor::RustTypeId = sbor::RustTypeId::novel_with_code(
                        "Test",
                        &[],
                        &#code_hash
                    );

                    fn type_data() -> sbor::TypeData <C, sbor::RustTypeId> {
                        sbor::TypeData::struct_with_unit_fields("Test")
                    }

                    fn add_all_dependencies(aggregator: &mut sbor::TypeAggregator<C>) { }
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
                impl <T: SomeTrait, T2, C: sbor::CustomTypeKind<sbor::RustTypeId> > sbor::Describe<C> for Test<T, T2>
                where
                    T: sbor::Describe<C>,
                    T2: sbor::Describe<C>
                {
                    const TYPE_ID: sbor::RustTypeId = sbor::RustTypeId::novel_with_code(
                        "Test",
                        &[<T as sbor::Describe<C>>::TYPE_ID, <T2 as sbor::Describe<C>>::TYPE_ID,],
                        &#code_hash
                    );

                    fn type_data() -> sbor::TypeData <C, sbor::RustTypeId> {
                        use sbor::rust::borrow::ToOwned;
                        sbor::TypeData::enum_variants(
                            "Test",
                            sbor::rust::prelude::indexmap![
                                0u8 => { sbor::TypeData::struct_with_unit_fields("A") },
                                1u8 => { sbor::TypeData::struct_with_unnamed_fields(
                                    "B",
                                    sbor::rust::vec![
                                        <T as sbor::Describe<C>>::TYPE_ID,
                                        <Vec<T2> as sbor::Describe<C>>::TYPE_ID,
                                    ],
                                ) },
                                2u8 => { sbor::TypeData::struct_with_named_fields(
                                    "C",
                                    sbor::rust::vec![
                                        ("x", <[u8; 5] as sbor::Describe<C>>::TYPE_ID),
                                    ],
                                ) },
                            ],
                        )
                    }

                    fn add_all_dependencies(aggregator: &mut sbor::TypeAggregator<C>) {
                        aggregator.add_child_type_and_descendents::<T>();
                        aggregator.add_child_type_and_descendents::<Vec<T2> >();
                        aggregator.add_child_type_and_descendents::<[u8; 5]>();
                    }
                }
            },
        );
    }
}
