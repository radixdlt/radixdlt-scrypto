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

const GENERAL_PARSE_ERROR_MESSAGE: &'static str = "Expected `#[sbor_assert(fixed(..))]` OR `#[sbor_assert(backwards_compatible(..))]`, with optional additional parameters `generate`, `regenerate`, and `settings(..)`. A command such as `#[sbor_assert(fixed(\"FILE:my_schema.bin\"), generate)]` can be used to generate the schema initially.";

fn extract_settings(attributes: &[Attribute]) -> Result<(AssertionMode, AdvancedSettings)> {
    // When we come to extract fixed named types,
    let inner_attributes = extract_wrapped_root_attributes(attributes, "sbor_assert")?;
    let keyed_inner_attributes =
        extract_wrapped_inner_attributes(&inner_attributes, GENERAL_PARSE_ERROR_MESSAGE)?;
    if keyed_inner_attributes.len() == 0 {
        return Err(Error::new(Span::call_site(), GENERAL_PARSE_ERROR_MESSAGE));
    }

    let assertion_mode = {
        let (attribute_name, (attribute_name_ident, attribute_value)) =
            keyed_inner_attributes.get_index(0).unwrap();
        let error_span = attribute_name_ident.span();
        match attribute_name.as_str() {
            "backwards_compatible" => {
                AssertionMode::BackwardsCompatible(extract_backwards_compatible_schema_parameters(
                    attribute_value.as_ref(),
                    attribute_name_ident.span(),
                )?)
            }
            "fixed" => AssertionMode::Fixed(extract_fixed_schema_parameters(
                attribute_value.as_ref(),
                error_span,
            )?),
            _ => return Err(Error::new(error_span, GENERAL_PARSE_ERROR_MESSAGE)),
        }
    };

    let mut advanced_settings = AdvancedSettings {
        settings_resolution: ComparisonSettingsResolution::Default,
        generate_mode: GenerateMode::NoGenerate,
    };

    for (attribute_name, (attribute_name_ident, attribute_value)) in
        keyed_inner_attributes.iter().skip(1)
    {
        let error_span = attribute_name_ident.span();
        match attribute_name.as_str() {
            "settings" => {
                let SettingsOverrides {
                    settings_resolution,
                } = extract_settings_overrides(attribute_value.as_ref(), error_span)?;
                if let Some(settings_resolution) = settings_resolution {
                    advanced_settings.settings_resolution = settings_resolution;
                }
            }
            "generate" => {
                if attribute_value.is_some() {
                    return Err(Error::new(error_span, GENERATE_PARSE_ERROR_MESSAGE));
                }
                advanced_settings.generate_mode = GenerateMode::Generate {
                    is_regenerate: false,
                };
            }
            "regenerate" => {
                if attribute_value.is_some() {
                    return Err(Error::new(error_span, GENERATE_PARSE_ERROR_MESSAGE));
                }
                advanced_settings.generate_mode = GenerateMode::Generate {
                    is_regenerate: true,
                };
            }
            _ => return Err(Error::new(error_span, GENERAL_PARSE_ERROR_MESSAGE)),
        }
    }

    Ok((assertion_mode, advanced_settings))
}

const GENERATE_PARSE_ERROR_MESSAGE: &'static str = "Expected just `generate` or `regenerate` without any value, for example: `#[sbor_assert(fixed(..), generate)]` OR `#[sbor_assert(backwards_compatible(..), settings(..), regenerate)]`";

const FIXED_PARSE_ERROR_MESSAGE: &'static str = "Expected `#[sbor_assert(fixed(X))]` where `X` is one of:\n* `\"INLINE:<hex-encoded schema>\"`\n* `\"FILE:<relative-file-path-to-encoded schema>\"`\n* Either `NAMED_CONSTANT` or `\"CONST:<CONSTANT_NAME>\"` where `<CONSTANT_NAME>` is the name of a defined constant string literal or some other type implementing `IntoSchema<SingleTypeSchema>`\n* `\"EXPR:Y\"` where `Y` is some expression generating `SingleTypeSchema<C>`, for `C` the custom schema. For example,\n  calling `generate_schema()` where `fn generate_schema() -> SingleTypeSchema<C> { .. }";

fn extract_fixed_schema_parameters(
    attribute_value: Option<&Vec<&NestedMeta>>,
    error_span: Span,
) -> Result<FixedSchemaParameters> {
    match attribute_value {
        Some(meta_list) if meta_list.len() == 1 => match meta_list[0] {
            NestedMeta::Meta(Meta::Path(path)) => {
                return Ok(FixedSchemaParameters::SingleSchema {
                    location: SchemaLocation::FromConstant {
                        constant_path: path.clone(),
                    },
                });
            }
            NestedMeta::Lit(Lit::Str(lit_str)) => {
                return Ok(FixedSchemaParameters::SingleSchema {
                    location: extract_schema_location_from_string(lit_str)?,
                })
            }
            _ => {}
        },
        _ => {}
    }
    return Err(Error::new(error_span, FIXED_PARSE_ERROR_MESSAGE));
}

const BACKWARDS_COMPATIBLE_PARSE_ERROR_MESSAGE: &'static str = "Expected EITHER `#[sbor_assert(backwards_compatible(version1 = X, version2 = X))]` where `X` is one of:\n* `\"INLINE:<hex-encoded schema>\"`\n* `\"FILE:<relative-file-path-to-encoded-schema>\"`\n* `\"CONST:<CONSTANT>\"` where `<CONSTANT_NAME>` is the name of a defined constant string literal or some other type implementing `IntoSchema<SingleTypeSchema>`\n* `\"EXPR:Y\"` where `Y` is some expression generating `SingleTypeSchema<C>`, for `C` the custom schema. For example,\n  calling `generate_schema()` where `fn generate_schema() -> SingleTypeSchema<C> { .. }\nOR `#[sbor_assert(backwards_compatible(\"EXPR:Y\"))]` where `Y` is some expression such as `params_builder()` generating a `SingleTypeSchemaCompatibilityParameters<S>` for `S` the custom schema.";

fn extract_backwards_compatible_schema_parameters(
    attribute_value: Option<&Vec<&NestedMeta>>,
    error_span: Span,
) -> Result<BackwardsCompatibleSchemaParameters> {
    match attribute_value {
        Some(meta_list) => {
            if let [NestedMeta::Lit(Lit::Str(lit_str))] = meta_list.as_slice() {
                if let Some(expression) = extract_prefixed(lit_str, "EXPR:") {
                    return Ok(BackwardsCompatibleSchemaParameters::FromExpression {
                        expression: expression.parse()?,
                    });
                }
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
    } else if let Some(inline_schema) = extract_prefixed(lit_str, "EXPR:") {
        SchemaLocation::FromExpression {
            expression: inline_schema.parse()?,
        }
    } else {
        return Err(Error::new(
            lit_str.span(),
            "Expected string to be prefixed with FILE:, CONST:, INLINE: or EXPR:",
        ));
    };

    Ok(schema_definition)
}

const SETTINGS_PARSE_ERROR_MESSAGE: &'static str = "Expected one of:\n* `settings(allow_name_changes)`\n* `settings(<CONSTANT_NAME>)` or `settings(\"CONST:<CONSTANT_NAME>\")` where `<CONSTANT_NAME>` is the name of a defined constant with type `SchemaComparisonSettings`\n* settings(\"EXPR:<Expression>\") where the expression is `impl FnOnce(SchemaComparisonSettings) -> SchemaComparisonSettings`\n* settings(comparison_between_current_and_latest = \"EXPR:F1\", comparison_between_versions = \"EXPR:F2\") where `F1`, `F2` are expressions which `impl FnOnce(SchemaComparisonSettings) -> SchemaComparisonSettings`";

struct SettingsOverrides {
    settings_resolution: Option<ComparisonSettingsResolution>,
}

fn extract_settings_overrides(
    attribute_value: Option<&Vec<&NestedMeta>>,
    error_span: Span,
) -> Result<SettingsOverrides> {
    match attribute_value {
        Some(meta_list) if meta_list.len() == 1 => match meta_list[0] {
            NestedMeta::Meta(Meta::Path(path)) => {
                let allow_name_changes = if let Some(ident) = path.get_ident() {
                    ident.to_string() == "allow_name_changes"
                } else {
                    false
                };
                if allow_name_changes {
                    return Ok(SettingsOverrides {
                        settings_resolution: Some(
                            ComparisonSettingsResolution::DefaultAllowingNameChanges,
                        ),
                    });
                } else {
                    return Ok(SettingsOverrides {
                        settings_resolution: Some(ComparisonSettingsResolution::FromConstant {
                            constant_path: path.clone(),
                        }),
                    });
                }
            }
            NestedMeta::Lit(Lit::Str(lit_str)) => {
                if let Some(lit_str) = extract_prefixed(lit_str, "CONST:") {
                    return Ok(SettingsOverrides {
                        settings_resolution: Some(ComparisonSettingsResolution::FromConstant {
                            constant_path: lit_str.parse()?,
                        }),
                    });
                } else if let Some(expression) = extract_prefixed(lit_str, "EXPR:") {
                    return Ok(SettingsOverrides {
                        settings_resolution: Some(
                            ComparisonSettingsResolution::FromBuilderExpression {
                                expression: expression.parse()?,
                            },
                        ),
                    });
                }
            }
            _ => {}
        },
        Some(meta_list) if meta_list.len() == 2 => {
            let mut comparison_between_current_and_latest = None;
            let mut comparison_between_versions = None;
            for meta in meta_list.iter() {
                if let NestedMeta::Meta(Meta::NameValue(meta_name_value)) = meta {
                    if let Lit::Str(lit_str) = &meta_name_value.lit {
                        if let Some(expression) = extract_prefixed(lit_str, "EXPR:") {
                            let expression = expression.parse()?;
                            if let Some(ident) = meta_name_value.path.get_ident() {
                                let key = ident.to_string();
                                match key.as_str() {
                                    "comparison_between_current_and_latest" => {
                                        comparison_between_current_and_latest = Some(expression);
                                    }
                                    "comparison_between_versions" => {
                                        comparison_between_versions = Some(expression);
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
            if let Some(comparison_between_current_and_latest) =
                comparison_between_current_and_latest
            {
                if let Some(comparison_between_versions) = comparison_between_versions {
                    return Ok(SettingsOverrides {
                        settings_resolution: Some(
                            ComparisonSettingsResolution::FromNamedBuilderExpressions {
                                comparison_between_current_and_latest,
                                comparison_between_versions,
                            },
                        ),
                    });
                }
            }
        }
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
    Fixed(FixedSchemaParameters),
    BackwardsCompatible(BackwardsCompatibleSchemaParameters),
}

enum GenerationTarget {
    Inline,
    File {
        file_path: LitStr,
        is_regenerate: bool,
    },
}

enum SchemaLocation {
    InlineString { inline: LitStr },
    FromConstant { constant_path: Path },
    FromExpression { expression: Expr },
    StringFromFile { file_path: LitStr },
}

impl SchemaLocation {
    fn generation_target(&self, is_regenerate: bool) -> GenerationTarget {
        match self {
            SchemaLocation::InlineString { .. } => GenerationTarget::Inline,
            SchemaLocation::FromConstant { .. } => GenerationTarget::Inline,
            SchemaLocation::FromExpression { .. } => GenerationTarget::Inline,
            SchemaLocation::StringFromFile { file_path } => GenerationTarget::File {
                file_path: file_path.clone(),
                is_regenerate,
            },
        }
    }
}

enum FixedSchemaParameters {
    SingleSchema { location: SchemaLocation },
}

enum BackwardsCompatibleSchemaParameters {
    FromExpression { expression: Expr },
    NamedSchemas { named_schemas: Vec<NamedSchema> },
}

struct NamedSchema {
    name: Ident,
    schema: SchemaLocation,
}

struct AdvancedSettings {
    settings_resolution: ComparisonSettingsResolution,
    generate_mode: GenerateMode,
}

enum GenerateMode {
    NoGenerate,
    Generate { is_regenerate: bool },
}

#[derive(PartialEq, Eq)]
enum ComparisonSettingsResolution {
    Default,
    DefaultAllowingNameChanges,
    FromConstant {
        constant_path: Path,
    },
    FromBuilderExpression {
        expression: Expr,
    },
    FromNamedBuilderExpressions {
        comparison_between_current_and_latest: Expr,
        comparison_between_versions: Expr,
    },
}

/// Only supposed to be used as a temporary mode, to assist with generating the schema. The generated test always panics.
fn handle_generate(
    ident: &Ident,
    custom_schema: Path,
    test_ident: Ident,
    generation_target: GenerationTarget,
    is_fixed: bool,
) -> Result<TokenStream> {
    let output_content = match generation_target {
        GenerationTarget::Inline => quote! {
            panic!(
                "After copying this, remove the `generate` / `regenerate` from the attribute value. The current type schema is:\n{hex}"
            );
        },
        GenerationTarget::File {
            file_path,
            is_regenerate,
        } => {
            let file_open_code = if is_regenerate {
                quote! {
                    OpenOptions::new()
                        .write(true)
                        .truncate(true)
                        .open(full_file_path.as_path())
                        .unwrap_or_else(|err| panic!(
                            "\nCould not open existing file to regenerate into. If you wish to generate it for the first time, use `generate` rather than `regenerate`.\nPath: {}\nError: {}\n",
                            full_file_path.display(),
                            err,
                        ));
                }
            } else {
                quote! {
                    OpenOptions::new()
                        .write(true)
                        .create_new(true)
                        .open(full_file_path.as_path())
                        .unwrap_or_else(|err| panic!(
                            "\nCould not create new file to generate into. If it already exists, and you intend to replace it, use `regenerate` rather than `generate`.\nPath: {}\nError: {}\n",
                            full_file_path.display(),
                            err,
                        ));
                }
            };

            let contents = match FileFormat::derive(&file_path) {
                FileFormat::Hex => quote! {
                    type_schema.encode_to_hex()
                },
                FileFormat::Bytes => quote! {
                    type_schema.encode_to_bytes()
                },
            };

            quote! {
                let contents = #contents;

                use std::path::{Path, PathBuf};
                use std::fs::OpenOptions;
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

                let mut file = #file_open_code;

                file.write_all(contents.as_ref())
                    .unwrap_or_else(|err| panic!(
                        "Schema could not be written to {} - Error: {}",
                        full_file_path.display(),
                        err,
                    ));

                // We panic because the generate test is always expected to fail - so that someone doesn't leave it in generate mode accidentally.
                panic!("\n\nSchema written successfully to:\n  {}\n\nNow panicking so that you can't accidentally leave this in (re)generate mode and have your tests pass!\n\n", full_file_path.display());
            }
        }
    };

    // We still output the trait impls in generate mode - this is to prevent a chicken-and-egg problem.
    //
    // If a user adds bounds that a type implements these traits, then the compilation of the crate
    // may depend on the type implementing the bounds. If the trait impls get dropped during
    // (re)generation, then this means that the crate can't compile and (re)generation can't run!
    //
    // The test will still fail if it's left in (re)generate mode, so this should be okay.
    let trait_impls = generate_trait_impls(&custom_schema, ident, is_fixed);

    let output = quote! {
        #trait_impls

        #[cfg(test)]
        #[test]
        #[allow(non_snake_case)]
        pub fn #test_ident() {
            let type_schema = sbor::schema::SingleTypeSchema::<#custom_schema>::for_type::<#ident>();
            #output_content
        }
    };

    Ok(output)
}

fn handle_fixed(
    context_custom_schema: &str,
    parsed: DeriveInput,
    params: FixedSchemaParameters,
    advanced_settings: AdvancedSettings,
) -> Result<TokenStream> {
    let DeriveInput { ident, .. } = &parsed;

    let fixed_schema = match &params {
        FixedSchemaParameters::SingleSchema { location } => {
            schema_location_to_single_type_schema_code(location)
        }
    };

    let custom_schema: Path = parse_str(context_custom_schema)?;
    let test_ident = format_ident!("test_{}_type_is_fixed", ident);

    if let GenerateMode::Generate { is_regenerate } = advanced_settings.generate_mode {
        let generation_target = match params {
            FixedSchemaParameters::SingleSchema { location } => {
                location.generation_target(is_regenerate)
            }
        };
        return handle_generate(ident, custom_schema, test_ident, generation_target, true);
    }

    let comparison_settings = match advanced_settings.settings_resolution {
        ComparisonSettingsResolution::Default => quote! {
            sbor::schema::SchemaComparisonSettings::require_equality()
        },
        ComparisonSettingsResolution::DefaultAllowingNameChanges => quote! {
            sbor::schema::SchemaComparisonSettings::require_equality().allow_all_name_changes()
        },
        ComparisonSettingsResolution::FromConstant { constant_path } => quote! {
            #constant_path.clone()
        },
        ComparisonSettingsResolution::FromBuilderExpression { expression } => quote! {
            #expression
        },
        ComparisonSettingsResolution::FromNamedBuilderExpressions {
            comparison_between_versions,
            ..
        } => {
            return Err(Error::new(
                comparison_between_versions.span(),
                "Cannot provide named comparisons to `fixed`",
            ));
        }
    };

    let trait_impls = generate_trait_impls(&custom_schema, ident, true);

    let output = quote! {
        #trait_impls

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
                writeln!(&mut error, "If you are sure the fixed version is incorrect, you can regenerate by changing to `#[sbor_assert(fixed(..), regenerate)]` and re-running the test.").unwrap();
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
        SchemaLocation::StringFromFile { file_path } => match FileFormat::derive(file_path) {
            FileFormat::Hex => quote! {
                sbor::schema::SingleTypeSchema::from(include_str!(#file_path))
            },
            FileFormat::Bytes => quote! {
                sbor::schema::SingleTypeSchema::from(include_bytes!(#file_path))
            },
        },
        SchemaLocation::FromExpression { expression } => {
            quote! { #expression }
        }
    }
}

enum FileFormat {
    Hex,
    Bytes,
}

impl FileFormat {
    fn derive(file_path_suffix: &LitStr) -> Self {
        let is_hex = file_path_suffix
            .value()
            .split(".")
            .last()
            .is_some_and(|extension| extension.to_ascii_lowercase().as_str() == "txt");
        if is_hex {
            Self::Hex
        } else {
            Self::Bytes
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

    if let GenerateMode::Generate { is_regenerate } = advanced_settings.generate_mode {
        let generation_target = match params {
            BackwardsCompatibleSchemaParameters::FromExpression { .. } => GenerationTarget::Inline,
            BackwardsCompatibleSchemaParameters::NamedSchemas { mut named_schemas } => {
                let Some(latest_named_schema) = named_schemas.pop() else {
                    return Err(Error::new(ident.span(), "At least one named schema placeholder needs adding in order for generation to work, e.g. `#[sbor_assert(backwards_compatible(latest = \"FILE:my_schema.bin\"), generate)]"));
                };
                latest_named_schema.schema.generation_target(is_regenerate)
            }
        };
        return handle_generate(ident, custom_schema, test_ident, generation_target, false);
    }

    let (comparison_between_current_and_latest, comparison_between_versions) =
        match &advanced_settings.settings_resolution {
            ComparisonSettingsResolution::Default => (quote! { |b| b }, quote! { |b| b }),
            ComparisonSettingsResolution::DefaultAllowingNameChanges => (
                quote! { |b| b.allow_all_name_changes() },
                quote! { |b| b.allow_all_name_changes() },
            ),
            ComparisonSettingsResolution::FromConstant { constant_path } => (
                quote! { |_| #constant_path.clone() },
                quote! { |_| #constant_path.clone() },
            ),
            ComparisonSettingsResolution::FromBuilderExpression { expression } => {
                (quote! { #expression }, quote! { #expression })
            }
            ComparisonSettingsResolution::FromNamedBuilderExpressions {
                comparison_between_current_and_latest,
                comparison_between_versions,
                ..
            } => (
                quote! { #comparison_between_current_and_latest },
                quote! { #comparison_between_versions },
            ),
        };

    let test_content = match params {
        BackwardsCompatibleSchemaParameters::FromExpression { expression } => {
            if advanced_settings.settings_resolution != ComparisonSettingsResolution::Default {
                return Err(Error::new(expression.span(), "You can't override comparison settings with EXPR mode - instead configure the settings in the SingleTypeSchemaCompatibilityParameters directly."));
            }
            quote! {
                sbor::schema::assert_type_backwards_compatibility::<#custom_schema, #ident>(
                    |_| #expression
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
                sbor::schema::assert_type_backwards_compatibility::<#custom_schema, #ident>(
                    |builder| {
                        builder
                            .with_comparison_between_current_and_latest(#comparison_between_current_and_latest)
                            .with_comparison_between_versions(#comparison_between_versions)
                        #(
                            .register_version(#version_names, #schemas)
                        )*
                    }
                );
            }
        }
    };

    let trait_impls = generate_trait_impls(&custom_schema, ident, false);

    let output = quote! {
        #trait_impls

        #[cfg(test)]
        #[test]
        #[allow(non_snake_case)]
        pub fn #test_ident() {
            #test_content
        }
    };

    Ok(output)
}

fn generate_trait_impls(custom_schema: &Path, ident: &Ident, is_fixed: bool) -> TokenStream {
    // NOTE: Generics are explicitly _NOT_ supported for now, because we need a concrete type
    //       to generate the schema from.
    if is_fixed {
        quote! {
            impl sbor::schema::CheckedFixedSchema<#custom_schema> for #ident {}
            impl sbor::schema::CheckedBackwardsCompatibleSchema<#custom_schema> for #ident {}
        }
    } else {
        quote! {
            impl sbor::schema::CheckedBackwardsCompatibleSchema<#custom_schema> for #ident {}
        }
    }
}
