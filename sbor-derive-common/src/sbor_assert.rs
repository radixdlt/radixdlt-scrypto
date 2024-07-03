use proc_macro2::{Span, TokenStream};
use quote::format_ident;
use quote::quote;
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

    let (assertion_variant, advanced_settings) = extract_settings(&parsed.attrs)?;

    let output = match assertion_variant {
        AssertionMode::Generate(params) => handle_generate(context_custom_schema, parsed, params),
        AssertionMode::Fixed(params) => {
            handle_fixed(context_custom_schema, parsed, params, advanced_settings)
        }
        AssertionMode::BackwardsCompatible(params) => {
            handle_backwards_compatible(context_custom_schema, parsed, params, advanced_settings)
        }
    }?;

    trace!("handle_sbor_assert_derive() finishes");
    Ok(output)
}

const GENERAL_PARSE_ERROR_MESSAGE: &'static str = "Expected `#[sbor_assert(generate(..))]` OR `#[sbor_assert(fixed(..))]` OR `#[sbor_assert(backwards_compatible(..))], with an optional second `settings(..)` parameter to `sbor_assert`.";

fn extract_settings(attributes: &[Attribute]) -> Result<(AssertionMode, AdvancedSettings)> {
    // When we come to extract fixed named types,
    let inner_attributes = extract_wrapped_root_attributes(attributes, "sbor_assert")?;
    let keyed_inner_attributes =
        extract_wrapped_inner_attributes(&inner_attributes, GENERAL_PARSE_ERROR_MESSAGE)?;
    if keyed_inner_attributes.len() == 0 || keyed_inner_attributes.len() > 2 {
        return Err(Error::new(Span::call_site(), GENERAL_PARSE_ERROR_MESSAGE));
    }

    let assertion_mode = {
        let (attribute_name, (attribute_name_ident, attribute_value)) =
            keyed_inner_attributes.get_index(0).unwrap();
        let error_span = attribute_name_ident.span();
        match attribute_name.as_str() {
            "generate" => AssertionMode::Generate(extract_generation_options(
                attribute_value.as_ref(),
                error_span,
            )?),
            "backwards_compatible" => {
                AssertionMode::BackwardsCompatible(extract_backwards_compatible_schema_parameters(
                    attribute_value.as_ref(),
                    attribute_name_ident.span(),
                )?)
            }
            "fixed" => AssertionMode::Fixed(extract_fixed_schema_options(
                attribute_value.as_ref(),
                error_span,
            )?),
            _ => return Err(Error::new(error_span, GENERAL_PARSE_ERROR_MESSAGE)),
        }
    };

    let advanced_settings = if let Some(second_attribute) = keyed_inner_attributes.get_index(1) {
        let (attribute_name, (attribute_name_ident, attribute_value)) = second_attribute;
        let error_span = attribute_name_ident.span();

        match attribute_name.as_str() {
            "settings" => extract_advanced_settings(attribute_value.as_ref(), error_span)?,
            _ => return Err(Error::new(error_span, SETTINGS_PARSE_ERROR_MESSAGE)),
        }
    } else {
        AdvancedSettings {
            settings_resolution: ComparisonSettingsResolution::Default,
        }
    };

    Ok((assertion_mode, advanced_settings))
}

const GENERATE_PARSE_ERROR_MESSAGE: &'static str = "Expected `#[sbor_assert(generate(\"INLINE\"))]` OR `#[sbor_assert(generate(\"FILE:<relative-path.txt>\"))]`";

fn extract_generation_options(
    attribute_value: Option<&Vec<&NestedMeta>>,
    error_span: Span,
) -> Result<GenerationOptions> {
    match attribute_value {
        Some(meta_list) if meta_list.len() == 1 => match meta_list[0] {
            NestedMeta::Lit(Lit::Str(lit_str)) => {
                let content = lit_str.value();
                if content == "INLINE" {
                    return Ok(GenerationOptions::Inline);
                } else if let Some(file_path) = extract_prefixed(lit_str, "FILE:") {
                    return Ok(GenerationOptions::File { file_path });
                }
            }
            _ => {}
        },
        _ => {}
    };
    return Err(Error::new(error_span, GENERATE_PARSE_ERROR_MESSAGE));
}

const FIXED_PARSE_ERROR_MESSAGE: &'static str = "Expected `#[sbor_assert(fixed(X))]` where `X` is one of:\n* `\"INLINE:<hex-encoded schema>\"`\n* `\"FILE:<relative-file-path-to-hex-encoded schema>\"`\n* Either `NAMED_CONSTANT` or `\"CONST:<CONSTANT_NAME>\"` where `<CONSTANT_NAME>` is the name of a defined constant string literal or some other type implementing `IntoSchema<SingleTypeSchema>`";

fn extract_fixed_schema_options(
    attribute_value: Option<&Vec<&NestedMeta>>,
    error_span: Span,
) -> Result<SchemaLocation> {
    match attribute_value {
        Some(meta_list) if meta_list.len() == 1 => match meta_list[0] {
            NestedMeta::Meta(Meta::Path(path)) => {
                return Ok(SchemaLocation::FromConstant {
                    constant_path: path.clone(),
                });
            }
            NestedMeta::Lit(Lit::Str(lit_str)) => {
                return extract_schema_location_from_string(lit_str);
            }
            _ => {}
        },
        _ => {}
    }
    return Err(Error::new(error_span, FIXED_PARSE_ERROR_MESSAGE));
}

const BACKWARDS_COMPATIBLE_PARSE_ERROR_MESSAGE: &'static str = "Expected EITHER `#[sbor_assert(backwards_compatible(version1 = X, version2 = X))]` where `X` is one of:\n* `\"INLINE:<hex-encoded schema>\"`\n* `\"FILE:<relative-file-path-to-hex-encoded schema>\"`\n* `\"CONST:<CONSTANT>\"` where `<CONSTANT_NAME>` is the name of a defined constant string literal or some other type implementing `IntoSchema<SingleTypeSchema>`\n\nOR `#[sbor_assert(backwards_compatible(<CONSTANT_NAME>))]` where `<CONSTANT_NAME>` is the name of a defined constant whose type implements `IntoIterator<Item = (K, V)>, K: AsRef<str>, V: IntoSchema<SingleTypeSchema>`. For example: `const TYPE_X_NAMED_VERSIONS: [(&'static str, &'static str); 1] = [(\"version1\", \"...\")]`";

fn extract_backwards_compatible_schema_parameters(
    attribute_value: Option<&Vec<&NestedMeta>>,
    error_span: Span,
) -> Result<BackwardsCompatibleSchemaParameters> {
    match attribute_value {
        Some(meta_list) => {
            if let [NestedMeta::Meta(Meta::Path(path))] = meta_list.as_slice() {
                return Ok(BackwardsCompatibleSchemaParameters::FromConstant {
                    constant: path.clone(),
                });
            } else {
                // Assume key-value based
                let named_schemas = meta_list
                    .iter()
                    .map(|meta| -> Result<_> {
                        match meta {
                            NestedMeta::Meta(Meta::NameValue(MetaNameValue {
                                path, lit, ..
                            })) => {
                                let Some(ident) = path.get_ident() else {
                                    return Err(Error::new(
                                        path.span(),
                                        BACKWARDS_COMPATIBLE_PARSE_ERROR_MESSAGE,
                                    ));
                                };
                                let Lit::Str(lit_str) = lit else {
                                    return Err(Error::new(
                                        path.span(),
                                        "Only string literals are supported here",
                                    ));
                                };
                                Ok(NamedSchema {
                                    name: ident.clone(),
                                    schema: extract_schema_location_from_string(lit_str)?,
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
                return Ok(BackwardsCompatibleSchemaParameters::NamedSchemas { named_schemas });
            }
        }
        _ => {}
    }

    return Err(Error::new(
        error_span,
        BACKWARDS_COMPATIBLE_PARSE_ERROR_MESSAGE,
    ));
}

fn extract_schema_location_from_string(lit_str: &LitStr) -> Result<SchemaLocation> {
    let schema_definition = if let Some(file_path) = extract_prefixed(lit_str, "FILE:") {
        SchemaLocation::StringFromFile { file_path }
    } else if let Some(constant_path) = extract_prefixed(lit_str, "CONST:") {
        SchemaLocation::FromConstant {
            constant_path: constant_path.parse()?,
        }
    } else if let Some(inline_schema) = extract_prefixed(lit_str, "INLINE:") {
        SchemaLocation::InlineString {
            inline: inline_schema,
        }
    } else {
        return Err(Error::new(
            lit_str.span(),
            "Expected string to be prefixed with FILE:, CONST: or INLINE:",
        ));
    };

    Ok(schema_definition)
}

const SETTINGS_PARSE_ERROR_MESSAGE: &'static str = "Expected `#[sbor_assert(__, settings(allow_name_changes))]` OR `#[sbor_assert(__, settings(<CONSTANT_NAME>))]` `<CONSTANT_NAME>` is the name of a defined constant with type `SchemaComparisonSettings`";

fn extract_advanced_settings(
    attribute_value: Option<&Vec<&NestedMeta>>,
    error_span: Span,
) -> Result<AdvancedSettings> {
    match attribute_value {
        Some(meta_list) if meta_list.len() == 1 => match meta_list[0] {
            NestedMeta::Meta(Meta::Path(path)) => {
                let allow_name_changes = if let Some(ident) = path.get_ident() {
                    ident.to_string() == "allow_name_changes"
                } else {
                    false
                };
                if allow_name_changes {
                    return Ok(AdvancedSettings {
                        settings_resolution:
                            ComparisonSettingsResolution::DefaultAllowingNameChanges,
                    });
                } else {
                    return Ok(AdvancedSettings {
                        settings_resolution: ComparisonSettingsResolution::FromConstant {
                            constant_path: path.clone(),
                        },
                    });
                }
            }
            _ => {}
        },
        _ => {}
    };
    return Err(Error::new(error_span, SETTINGS_PARSE_ERROR_MESSAGE));
}

fn extract_prefixed(lit_str: &LitStr, prefix: &str) -> Option<LitStr> {
    let contents = lit_str.value();
    if contents.starts_with(prefix) {
        let (_prefix, inner_contents) = contents.split_at(prefix.len());
        Some(LitStr::new(inner_contents, lit_str.span()))
    } else {
        None
    }
}

enum AssertionMode {
    Generate(GenerationOptions),
    Fixed(SchemaLocation),
    BackwardsCompatible(BackwardsCompatibleSchemaParameters),
}

enum GenerationOptions {
    Inline,
    File { file_path: LitStr },
}

enum SchemaLocation {
    InlineString { inline: LitStr },
    FromConstant { constant_path: Path },
    StringFromFile { file_path: LitStr },
}

enum BackwardsCompatibleSchemaParameters {
    FromConstant { constant: Path },
    NamedSchemas { named_schemas: Vec<NamedSchema> },
}

struct NamedSchema {
    name: Ident,
    schema: SchemaLocation,
}

struct AdvancedSettings {
    settings_resolution: ComparisonSettingsResolution,
}

enum ComparisonSettingsResolution {
    Default,
    DefaultAllowingNameChanges,
    FromConstant { constant_path: Path },
}

/// Only supposed to be used as a temporary mode, to assist with generating the schema. The generated test always panics.
fn handle_generate(
    context_custom_schema: &str,
    parsed: DeriveInput,
    options: GenerationOptions,
) -> Result<TokenStream> {
    let DeriveInput { ident, .. } = &parsed;

    let custom_schema: Path = parse_str(context_custom_schema)?;
    let test_ident = format_ident!("test_{}_type_schema_is_generated", ident);

    let output_content = match options {
        GenerationOptions::Inline => quote! {
            panic!(
                "Copy the below encoded type schema and replace `generate` with `fixed` or `backwards_compatible` in the attribute to receive further instructions.\n The current type schema is:\n{hex}"
            );
        },
        GenerationOptions::File { file_path } => quote! {
            use std::path::{Path, PathBuf};
            use std::fs::File;
            use std::io::Write;
            use std::convert::AsRef;

            // So `file!()` is only intended for debugging, and is currently a relative path against `CARGO_RUSTC_CURRENT_DIR`.
            // However `CARGO_RUSTC_CURRENT_DIR` is a nightly-only env variable.
            //
            // For single crates, `CARGO_RUSTC_CURRENT_DIR` = `CARGO_MANIFEST_DIR`
            // For workspaces, `CARGO_RUSTC_CURRENT_DIR` is the workspace root, typically an ancestor of `CARGO_MANIFEST_DIR`
            //
            // So we add some resolution logic to resolve things...
            //
            // RELEVANT LINKS:
            // * https://github.com/rust-lang/cargo/issues/3946#issuecomment-412363291 - Absolute use of `file!()`
            // * https://github.com/rust-lang/cargo/issues/3946#issuecomment-1832514876
            // * https://github.com/rust-lang/cargo/pull/13644 - blocked stabilization of `CARGO_RUSTC_CURRENT_DIR`

            let manifest_dir = env!("CARGO_MANIFEST_DIR");
            let relative_source_file_path = file!();

            let mut path_root = PathBuf::from(&manifest_dir);
            let source_file_path = loop {
                let candidate_source_file_path = path_root.as_path().join(relative_source_file_path);
                if candidate_source_file_path.is_file() {
                    break candidate_source_file_path;
                }
                if !path_root.pop() {
                    panic!(
                        "Could not resolve the source file path from CARGO_MANIFEST_DIR ({}) and file!() path ({})",
                        manifest_dir,
                        relative_source_file_path,
                    );
                }
            };

            let source_file_folder = source_file_path
                .parent()
                .unwrap_or_else(|| panic!(
                    "Could not resolve the parent folder of the current source file: {}",
                    source_file_path.display(),
                ));

            if !source_file_folder.is_dir() {
                panic!(
                    "The resolved parent folder of the current source file doesn't appear to exist: {}",
                    source_file_folder.display(),
                );
            }

            // Resolve the provided file path relative to the source file's folder
            let full_file_path = source_file_folder.join(#file_path);

            let mut file = File::create_new(full_file_path.as_path())
                .unwrap_or_else(|err| panic!(
                    "Could not open new file for writing - perhaps it already exists? If you wish to replace it, delete it first: {} - Error: {}",
                    full_file_path.display(),
                    err,
                ));

            file.write_all(hex.as_ref())
                .unwrap_or_else(|err| panic!(
                    "Schema could not be written to {} - Error: {}",
                    full_file_path.display(),
                    err,
                ));

            // We panic because the generate test is always expected to fail - so that someone doesn't leave it in generate mode accidentally.
            panic!("Schema written successfully to {}", full_file_path.display());
        },
    };

    // NOTE: Generics are explicitly _NOT_ supported for now, because we need a concrete type
    //       to generate the schema from.
    let output = quote! {
        #[cfg(test)]
        #[test]
        #[allow(non_snake_case)]
        pub fn #test_ident() {
            let type_schema = sbor::schema::SingleTypeSchema::<#custom_schema>::for_type::<#ident>();
            let hex = type_schema.encode_to_hex();
            #output_content
        }
    };

    Ok(output)
}

fn handle_fixed(
    context_custom_schema: &str,
    parsed: DeriveInput,
    schema_location: SchemaLocation,
    advanced_settings: AdvancedSettings,
) -> Result<TokenStream> {
    let DeriveInput { ident, .. } = &parsed;

    let fixed_schema = schema_location_to_single_type_schema_code(&schema_location);

    let custom_schema: Path = parse_str(context_custom_schema)?;
    let test_ident = format_ident!("test_{}_type_is_fixed", ident);

    let comparison_settings = match advanced_settings.settings_resolution {
        ComparisonSettingsResolution::Default => quote! {
            sbor::schema::SchemaComparisonSettings::require_equality()
        },
        ComparisonSettingsResolution::DefaultAllowingNameChanges => quote! {
            sbor::schema::SchemaComparisonSettings::require_equality()
                .metadata_settings(sbor::schema::SchemaComparisonMetadataSettings::allow_all_changes())
        },
        ComparisonSettingsResolution::FromConstant { constant_path } => quote! {
            #constant_path.clone()
        },
    };

    // NOTE: Generics are explicitly _NOT_ supported for now, because we need a concrete type
    //       to generate the schema from.
    let output = quote! {
        impl sbor::schema::CheckedFixedSchema<#custom_schema> for #ident {}
        impl sbor::schema::CheckedBackwardsCompatibleSchema<#custom_schema> for #ident {}

        #[cfg(test)]
        #[test]
        #[allow(non_snake_case)]
        pub fn #test_ident() {
            let comparison_settings: sbor::schema::SchemaComparisonSettings = #comparison_settings;
            let current = sbor::schema::SingleTypeSchema::for_type::<#ident>();
            let fixed = #fixed_schema;
            let result = sbor::schema::compare_single_type_schemas::<#custom_schema>(&comparison_settings, &fixed, &current);
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

fn schema_location_to_single_type_schema_code(params: &SchemaLocation) -> TokenStream {
    match params {
        SchemaLocation::FromConstant { constant_path } => {
            quote! { sbor::schema::SingleTypeSchema::from(#constant_path) }
        }
        SchemaLocation::InlineString {
            inline: fixed_schema,
        } => {
            quote! { sbor::schema::SingleTypeSchema::from(#fixed_schema) }
        }
        SchemaLocation::StringFromFile { file_path } => {
            quote! { sbor::schema::SingleTypeSchema::from(include_str!(#file_path)) }
        }
    }
}

fn handle_backwards_compatible(
    context_custom_schema: &str,
    parsed: DeriveInput,
    params: BackwardsCompatibleSchemaParameters,
    advanced_settings: AdvancedSettings,
) -> Result<TokenStream> {
    let DeriveInput { ident, .. } = &parsed;

    let custom_schema: Path = parse_str(context_custom_schema)?;
    let test_ident = format_ident!("test_{}_type_is_backwards_compatible", ident);

    let comparison_settings = match advanced_settings.settings_resolution {
        ComparisonSettingsResolution::Default => quote! {
            sbor::schema::SchemaComparisonSettings::allow_extension()
        },
        ComparisonSettingsResolution::DefaultAllowingNameChanges => quote! {
            sbor::schema::SchemaComparisonSettings::allow_extension()
                .metadata_settings(sbor::schema::SchemaComparisonMetadataSettings::allow_all_changes())
        },
        ComparisonSettingsResolution::FromConstant { constant_path } => quote! {
            #constant_path.clone()
        },
    };

    let test_content = match params {
        BackwardsCompatibleSchemaParameters::FromConstant { constant } => {
            quote! {
                let comparison_settings: sbor::schema::SchemaComparisonSettings = #comparison_settings;
                sbor::schema::assert_type_compatibility::<#custom_schema, #ident>(
                    &comparison_settings,
                    |v| sbor::schema::NamedSchemaVersions::from(#constant),
                );
            }
        }
        BackwardsCompatibleSchemaParameters::NamedSchemas { named_schemas } => {
            // NOTE: It's okay for these to be empty - the test output will output a correct default schema.
            let (version_names, schemas): (Vec<_>, Vec<_>) = named_schemas
                .into_iter()
                .map(|named_schema| {
                    (
                        named_schema.name.to_string(),
                        schema_location_to_single_type_schema_code(&named_schema.schema),
                    )
                })
                .unzip();

            quote! {
                let comparison_settings: sbor::schema::SchemaComparisonSettings = #comparison_settings;
                sbor::schema::assert_type_compatibility::<#custom_schema, #ident>(
                    &comparison_settings,
                    |v| {
                        v
                        #(
                            .register_version(#version_names, #schemas)
                        )*
                    }
                );
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
