use proc_macro2::{Span, TokenStream};
use quote::format_ident;
use quote::quote;
use quote::ToTokens;
use spanned::Spanned as _;
use syn::*;

use crate::utils::*;

macro_rules! trace {
    ($($arg:expr),*) => {{
        #[cfg(feature = "trace")]
        println!($($arg),*);
    }};
}

pub fn handle_sbor_assert_derive(
    input: TokenStream,
    context_custom_schema: &str,
) -> Result<TokenStream> {
    trace!("handle_sbor_assert_derive() starts");

    let parsed: DeriveInput = parse2(input)?;

    if parsed.generics.params.len() > 0 || parsed.generics.where_clause.is_some() {
        // In future we could support them by allowing concrete type parameters to be provided in attributes,
        // to be used for the purpose of generating the test.
        return Err(Error::new(
            Span::call_site(),
            "Generic types are not presently supported with the SborAssert macros",
        ));
    };

    let assertion_variant = extract_assertion_variant(&parsed.attrs)?;

    let output = match assertion_variant {
        AssertionVariant::Generate => handle_generate(context_custom_schema, parsed),
        AssertionVariant::Fixed(params) => handle_fixed(context_custom_schema, parsed, params),
        AssertionVariant::BackwardsCompatible(params) => {
            handle_backwards_compatible(context_custom_schema, parsed, params)
        }
    }?;

    trace!("handle_sbor_assert_derive() finishes");
    Ok(output)
}

const GENERAL_PARSE_ERROR_MESSAGE: &'static str = "Expected `#[sbor_assert(generate)]` OR `#[sbor_assert(fixed(...))]` or `#[sbor_assert(backwards_compatible(...))]`";
const FIXED_PARSE_ERROR_MESSAGE: &'static str = "Expected `#[sbor_assert(fixed(\"<hex-encoded schema>\"))]` OR  `#[sbor_assert(fixed(CONSTANT))]` where CONSTANT is a string or implements `IntoSchema<SingleTypeSchema>`";
const BACKWARDS_COMPATIBLE_PARSE_ERROR_MESSAGE: &'static str = "Expected `#[sbor_assert(backwards_compatible(version1 = \"...\", version2 = \"...\"))]` where the placeholders are hex-encoded schemas, OR `#[sbor_assert(backwards_compatible(CONSTANT))]` where CONSTANT implements `IntoIterator<Item = (K, V)>, K: AsRef<str>, V: IntoSchema<SingleTypeSchema>`. For example: `const TYPE_X_NAMED_VERSIONS: [(&'static str, &'static str); X] = [(\"version1\", \"...\")]`";

fn extract_assertion_variant(attributes: &[Attribute]) -> Result<AssertionVariant> {
    // When we come to extract fixed named types,
    let inner_attributes = extract_wrapped_root_attributes(attributes, "sbor_assert")?;
    let keyed_inner_attributes =
        extract_wrapped_inner_attributes(&inner_attributes, GENERAL_PARSE_ERROR_MESSAGE)?;
    if keyed_inner_attributes.len() != 1 {
        return Err(Error::new(Span::call_site(), GENERAL_PARSE_ERROR_MESSAGE));
    }
    let (attribute_name, (attribute_name_ident, attribute_value)) =
        keyed_inner_attributes.into_iter().next().unwrap();

    match attribute_name.as_str() {
        "generate" => {
            if attribute_value.is_some() {
                Err(Error::new(
                    attribute_name_ident.span(),
                    GENERAL_PARSE_ERROR_MESSAGE,
                ))
            } else {
                Ok(AssertionVariant::Generate)
            }
        }
        "backwards_compatible" => {
            let schema_parameters = match attribute_value {
                Some(meta_list) => {
                    if let [NestedMeta::Meta(Meta::Path(path))] = meta_list.as_slice() {
                        // Constant
                        BackwardsCompatibleSchemaParameters::FromConstant {
                            constant: path.clone(),
                        }
                    } else {
                        // Key-value based
                        let named_schemas = meta_list
                            .iter()
                            .map(|meta| -> Result<_> {
                                match meta {
                                    NestedMeta::Meta(Meta::NameValue(MetaNameValue {
                                        path,
                                        lit,
                                        ..
                                    })) => {
                                        let Some(ident) = path.get_ident() else {
                                            return Err(Error::new(
                                                path.span(),
                                                BACKWARDS_COMPATIBLE_PARSE_ERROR_MESSAGE,
                                            ));
                                        };
                                        Ok(NamedSchema {
                                            name: ident.clone(),
                                            schema: lit.into_token_stream(),
                                        })
                                    }
                                    _ => {
                                        return Err(Error::new(
                                            meta.span(),
                                            BACKWARDS_COMPATIBLE_PARSE_ERROR_MESSAGE,
                                        ))
                                    }
                                }
                            })
                            .collect::<Result<_>>()?;
                        BackwardsCompatibleSchemaParameters::NamedSchemas { named_schemas }
                    }
                }
                _ => {
                    return Err(Error::new(
                        attribute_name_ident.span(),
                        BACKWARDS_COMPATIBLE_PARSE_ERROR_MESSAGE,
                    ));
                }
            };
            Ok(AssertionVariant::BackwardsCompatible(schema_parameters))
        }
        "fixed" => {
            let fixed_schema_parameters = match attribute_value {
                Some(inner) if inner.len() == 1 => match inner[0] {
                    NestedMeta::Meta(Meta::Path(path)) => FixedSchemaParameters::FromConstant {
                        constant_path: path.clone(),
                    },
                    NestedMeta::Lit(Lit::Str(lit_str)) => FixedSchemaParameters::FixedSchema {
                        fixed_schema: lit_str.into_token_stream(),
                    },
                    _ => {
                        return Err(Error::new(
                            attribute_name_ident.span(),
                            FIXED_PARSE_ERROR_MESSAGE,
                        ));
                    }
                },
                _ => {
                    return Err(Error::new(
                        attribute_name_ident.span(),
                        FIXED_PARSE_ERROR_MESSAGE,
                    ));
                }
            };
            Ok(AssertionVariant::Fixed(fixed_schema_parameters))
        }
        _ => Err(Error::new(
            attribute_name_ident.span(),
            GENERAL_PARSE_ERROR_MESSAGE,
        )),
    }
}

enum AssertionVariant {
    Generate,
    Fixed(FixedSchemaParameters),
    BackwardsCompatible(BackwardsCompatibleSchemaParameters),
}

enum FixedSchemaParameters {
    FromConstant { constant_path: Path },
    FixedSchema { fixed_schema: TokenStream },
}

enum BackwardsCompatibleSchemaParameters {
    FromConstant { constant: Path },
    NamedSchemas { named_schemas: Vec<NamedSchema> },
}

struct NamedSchema {
    name: Ident,
    schema: TokenStream,
}

fn handle_generate(context_custom_schema: &str, parsed: DeriveInput) -> Result<TokenStream> {
    let DeriveInput { ident, .. } = &parsed;

    let custom_schema: Path = parse_str(context_custom_schema)?;
    let test_ident = format_ident!("test_{}_type_is_generated_in_panic_message", ident);

    // NOTE: Generics are explicitly _NOT_ supported for now, because we need a concrete type
    //       to generate the schema from.
    let output = quote! {
        #[cfg(test)]
        #[test]
        #[allow(non_snake_case)]
        pub fn #test_ident() {
            let type_schema = sbor::schema::SingleTypeSchema::<#custom_schema>::for_type::<#ident>();
            panic!(
                "Copy the below encoded type schema and replace `generate` with `fixed` or `backwards_compatible` in the attribute to receive further instructions.\n The current type schema is:\n{}", type_schema.encode_to_hex()
            );
        }
    };

    Ok(output)
}

fn handle_fixed(
    context_custom_schema: &str,
    parsed: DeriveInput,
    params: FixedSchemaParameters,
) -> Result<TokenStream> {
    let DeriveInput { ident, .. } = &parsed;

    let fixed_schema = match params {
        FixedSchemaParameters::FromConstant { constant_path } => {
            quote! { sbor::schema::SingleTypeSchema::from(#constant_path) }
        }
        FixedSchemaParameters::FixedSchema { fixed_schema } => {
            quote! { sbor::schema::SingleTypeSchema::from(#fixed_schema) }
        }
    };

    let custom_schema: Path = parse_str(context_custom_schema)?;
    let test_ident = format_ident!("test_{}_type_is_fixed", ident);

    // NOTE: Generics are explicitly _NOT_ supported for now, because we need a concrete type
    //       to generate the schema from.
    let output = quote! {
        impl sbor::schema::CheckedFixedSchema<#custom_schema> for #ident {}
        impl sbor::schema::CheckedBackwardsCompatibleSchema<#custom_schema> for #ident {}

        #[cfg(test)]
        #[test]
        #[allow(non_snake_case)]
        pub fn #test_ident() {
            let current = sbor::schema::SingleTypeSchema::for_type::<#ident>();
            let fixed = #fixed_schema;
            let result = sbor::schema::compare_single_type_schemas::<
                #custom_schema,
            >(
                &sbor::schema::SchemaComparisonSettings::require_equality(),
                &fixed,
                &current,
            );
            if let Some(error_message) = result.error_message("fixed", "current") {
                use sbor::rust::fmt::Write;
                use sbor::rust::prelude::String;
                let mut error = String::new();
                writeln!(&mut error, "{error_message}").unwrap();
                writeln!(&mut error, "If you are sure the fixed version is incorrect, it can be updated to the current version which is:").unwrap();
                writeln!(&mut error, "{}", current.encode_to_hex()).unwrap();
                panic!("{error}");
            }
        }
    };

    Ok(output)
}

fn handle_backwards_compatible(
    context_custom_schema: &str,
    parsed: DeriveInput,
    params: BackwardsCompatibleSchemaParameters,
) -> Result<TokenStream> {
    let DeriveInput { ident, .. } = &parsed;

    let custom_schema: Path = parse_str(context_custom_schema)?;
    let test_ident = format_ident!("test_{}_type_is_backwards_compatible", ident);

    let test_content = match params {
        BackwardsCompatibleSchemaParameters::FromConstant { constant } => {
            quote! {
                sbor::schema::assert_type_backwards_compatibility::<
                    #custom_schema,
                    #ident,
                >(|v| sbor::schema::NamedSchemaVersions::from(#constant));
            }
        }
        BackwardsCompatibleSchemaParameters::NamedSchemas { named_schemas } => {
            // NOTE: It's okay for these to be empty - the test output will output a correct default schema.
            let (version_names, schemas): (Vec<_>, Vec<_>) = named_schemas
                .into_iter()
                .map(|named_schema| (named_schema.name.to_string(), named_schema.schema))
                .unzip();

            quote! {
                sbor::schema::assert_type_backwards_compatibility::<
                    #custom_schema,
                    #ident,
                >(|v| {
                    v
                    #(
                        .register_version(#version_names, #schemas)
                    )*
                });
            }
        }
    };

    // NOTE: Generics are explicitly _NOT_ supported for now, because we need a concrete type
    //       to generate the schema from.
    let output = quote! {
        impl sbor::schema::CheckedBackwardsCompatibleSchema<#custom_schema> for #ident {}

        #[cfg(test)]
        #[test]
        #[allow(non_snake_case)]
        pub fn #test_ident() {
            #test_content
        }
    };

    Ok(output)
}
