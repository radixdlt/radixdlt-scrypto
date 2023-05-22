use itertools::Itertools;
use std::collections::BTreeMap;
use std::io::Write;
use std::process::Command;
use std::process::Stdio;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Comma;

use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;
use syn::*;

#[allow(dead_code)]
pub fn print_generated_code<S: ToString>(kind: &str, code: S) {
    if let Ok(mut proc) = Command::new("rustfmt")
        .arg("--emit=stdout")
        .arg("--edition=2021")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
    {
        {
            let stdin = proc.stdin.as_mut().unwrap();
            stdin.write_all(code.to_string().as_bytes()).unwrap();
        }
        if let Ok(output) = proc.wait_with_output() {
            if output.status.success() {
                println!(
                    "{}\n{}\n{}\n{}\n",
                    "-".repeat(kind.len()),
                    kind,
                    "-".repeat(kind.len()),
                    String::from_utf8(output.stdout).unwrap()
                );
            }
        }
    }
}

pub enum AttributeValue {
    None(Span),
    Path(Path),
    Lit(Lit),
}

impl AttributeValue {
    fn as_string(&self) -> Option<String> {
        match self {
            AttributeValue::Lit(Lit::Str(str)) => Some(str.value()),
            _ => None,
        }
    }

    fn as_bool(&self) -> Option<bool> {
        match self {
            AttributeValue::None(_) => Some(true),
            AttributeValue::Lit(Lit::Str(str)) => match str.value().as_str() {
                "true" => Some(true),
                "false" => Some(false),
                _ => None,
            },
            AttributeValue::Lit(Lit::Bool(bool)) => Some(bool.value()),
            _ => None,
        }
    }

    fn span(&self) -> Span {
        match self {
            AttributeValue::None(span) => *span,
            AttributeValue::Path(path) => path.span(),
            AttributeValue::Lit(lit) => lit.span(),
        }
    }
}

trait AttributeMap {
    fn get_bool_value(&self, name: &str) -> Result<bool>;
    fn get_string_value(&self, name: &str) -> Result<Option<String>>;
}

impl AttributeMap for BTreeMap<String, AttributeValue> {
    fn get_bool_value(&self, name: &str) -> Result<bool> {
        let Some(value) = self.get(name) else {
            return Ok(false);
        };
        value
            .as_bool()
            .ok_or_else(|| Error::new(value.span(), format!("Expected bool attribute")))
    }

    fn get_string_value(&self, name: &str) -> Result<Option<String>> {
        let Some(value) = self.get(name) else {
            return Ok(None);
        };
        Ok(Some(value.as_string().ok_or_else(|| {
            Error::new(value.span(), format!("Expected string attribute value"))
        })?))
    }
}

/// Permits attribute of the form #[sbor(opt1, opt2 = X, opt3(Y))] for some literal X or some path or literal Y.
pub fn extract_sbor_typed_attributes(
    attributes: &[Attribute],
) -> Result<BTreeMap<String, AttributeValue>> {
    extract_typed_attributes(attributes, "sbor")
}

/// Permits attribute of the form #[{name}(opt1, opt2 = X, opt3(Y))] for some literal X or some path or literal Y.
pub fn extract_typed_attributes(
    attributes: &[Attribute],
    name: &str,
) -> Result<BTreeMap<String, AttributeValue>> {
    let mut fields = BTreeMap::new();
    for attribute in attributes {
        if !attribute.path.is_ident(name) {
            continue;
        }
        let Ok(meta) = attribute.parse_meta() else {
            return Err(Error::new(
                attribute.span(),
                format!("Attribute content is not valid"),
            ));
        };
        let Meta::List(MetaList { nested: options, .. }) = meta else {
            return Err(Error::new(
                attribute.span(),
                format!("Expected list-based attribute as #[{name}(..)]"),
            ));
        };
        let error_message = format!("Expected attribute of the form #[{name}(opt1, opt2 = X, opt3(Y))] for some literal X or some path or literal Y.");
        for option in options.into_iter() {
            match option {
                NestedMeta::Meta(m) => match m {
                    Meta::Path(path) => {
                        if let Some(ident) = path.get_ident() {
                            fields.insert(ident.to_string(), AttributeValue::None(path.span()));
                        } else {
                            return Err(Error::new(path.span(), error_message));
                        }
                    }
                    Meta::NameValue(name_value) => {
                        if let Some(ident) = name_value.path.get_ident() {
                            fields.insert(ident.to_string(), AttributeValue::Lit(name_value.lit));
                        } else {
                            return Err(Error::new(name_value.path.span(), error_message));
                        }
                    }
                    Meta::List(MetaList { nested, path, .. }) => {
                        if let Some(ident) = path.get_ident() {
                            if nested.len() == 1 {
                                match nested.into_iter().next().unwrap() {
                                    NestedMeta::Meta(inner_meta) => match inner_meta {
                                        Meta::Path(path) => {
                                            fields.insert(
                                                ident.to_string(),
                                                AttributeValue::Path(path.clone()),
                                            );
                                        }
                                        _ => {
                                            return Err(Error::new(
                                                inner_meta.span(),
                                                error_message,
                                            ));
                                        }
                                    },
                                    NestedMeta::Lit(lit) => {
                                        fields.insert(
                                            ident.to_string(),
                                            AttributeValue::Lit(lit.clone()),
                                        );
                                    }
                                }
                            } else {
                                return Err(Error::new(nested.span(), error_message));
                            }
                        } else {
                            return Err(Error::new(path.span(), error_message));
                        }
                    }
                },
                _ => {
                    return Err(Error::new(option.span(), error_message));
                }
            }
        }
        return Ok(fields);
    }

    Ok(fields)
}

enum VariantValue {
    Byte(LitByte),
    Path(Path), // EG a constant
}

pub fn get_variant_discriminator_mapping(
    enum_attributes: &[Attribute],
    variants: &Punctuated<Variant, Comma>,
) -> Result<BTreeMap<usize, Expr>> {
    if variants.len() > 255 {
        return Err(Error::new(
            Span::call_site(),
            format!("SBOR can only support enums of size <= 255"),
        ));
    }

    let use_repr_discriminators =
        get_sbor_attribute_boolean_value(enum_attributes, "use_repr_discriminators")?;
    let mut variant_ids: BTreeMap<usize, VariantValue> = BTreeMap::new();

    for (i, variant) in variants.iter().enumerate() {
        let mut variant_attributes = extract_typed_attributes(&variant.attrs, "sbor")?;
        if let Some(attribute) = variant_attributes.remove("discriminator") {
            let id = match attribute {
                AttributeValue::None(span) => {
                    return Err(Error::new(span, format!("No discriminator was provided")));
                }
                AttributeValue::Path(path) => VariantValue::Path(path),
                AttributeValue::Lit(literal) => parse_u8_from_literal(&literal)
                    .map(|b| VariantValue::Byte(LitByte::new(b, literal.span())))
                    .ok_or_else(|| {
                        Error::new(
                            literal.span(),
                            format!("This discriminator is not a u8-convertible value"),
                        )
                    })?,
            };

            variant_ids.insert(i, id);
            continue;
        }
        if use_repr_discriminators {
            if let Some(discriminant) = &variant.discriminant {
                let expression = &discriminant.1;

                let id = match expression {
                    Expr::Lit(literal_expression) => parse_u8_from_literal(&literal_expression.lit)
                        .map(|b| VariantValue::Byte(LitByte::new(b, literal_expression.span()))),
                    Expr::Path(path_expression) => {
                        Some(VariantValue::Path(path_expression.path.clone()))
                    }
                    _ => None,
                };

                let Some(id) = id else {
                    return Err(Error::new(
                        expression.span(),
                        format!("This discriminator is not a u8-convertible value or a path. Add an #[sbor(discriminator(X))] annotation with a u8-compatible literal or path to const/static variable to fix."),
                    ));
                };

                variant_ids.insert(i, id);
                continue;
            }
        }
    }

    if variant_ids.len() > 0 {
        if variant_ids.len() < variants.len() {
            return Err(Error::new(
                Span::call_site(),
                format!("Either all or no variants must be assigned an id. Currently {} of {} variants have one.", variant_ids.len(), variants.len()),
            ));
        }
        return Ok(variant_ids
            .into_iter()
            .map(|(i, id)| {
                let expression = match id {
                    VariantValue::Byte(id) => parse_quote!(#id),
                    VariantValue::Path(id) => parse_quote!(#id),
                };
                (i, expression)
            })
            .collect());
    }
    // If no explicit indices, use default indices
    Ok(variants
        .iter()
        .enumerate()
        .map(|(i, _)| {
            let i_as_u8 = u8::try_from(i).unwrap();
            (i, parse_quote!(#i_as_u8))
        })
        .collect())
}

fn parse_u8_from_literal(literal: &Lit) -> Option<u8> {
    match literal {
        Lit::Byte(byte_literal) => Some(byte_literal.value()),
        Lit::Int(int_literal) => int_literal.base10_parse::<u8>().ok(),
        Lit::Str(str_literal) => str_literal.value().parse::<u8>().ok(),
        _ => None,
    }
}

fn get_sbor_attribute_string_value(
    attributes: &[Attribute],
    field_name: &str,
) -> Result<Option<String>> {
    extract_sbor_typed_attributes(attributes)?.get_string_value(&field_name)
}

fn get_sbor_attribute_boolean_value(attributes: &[Attribute], field_name: &str) -> Result<bool> {
    extract_sbor_typed_attributes(attributes)?.get_bool_value(&field_name)
}

pub fn get_sbor_bool_value(attributes: &[Attribute], attribute_name: &str) -> Result<bool> {
    extract_sbor_typed_attributes(&attributes)?.get_bool_value(attribute_name)
}

pub fn is_categorize_skipped(f: &Field) -> Result<bool> {
    let attributes = extract_sbor_typed_attributes(&f.attrs)?;
    Ok(attributes.get_bool_value("skip")? || attributes.get_bool_value("skip_categorize")?)
}

pub fn is_decoding_skipped(f: &Field) -> Result<bool> {
    let attributes = extract_sbor_typed_attributes(&f.attrs)?;
    Ok(attributes.get_bool_value("skip")? || attributes.get_bool_value("skip_decode")?)
}

pub fn is_encoding_skipped(f: &Field) -> Result<bool> {
    let attributes = extract_sbor_typed_attributes(&f.attrs)?;
    Ok(attributes.get_bool_value("skip")? || attributes.get_bool_value("skip_encode")?)
}

pub fn is_transparent(attributes: &[Attribute]) -> Result<bool> {
    let attributes = extract_sbor_typed_attributes(attributes)?;
    Ok(attributes.get_bool_value("transparent")?)
}

pub fn get_custom_value_kind(attributes: &[Attribute]) -> Result<Option<String>> {
    extract_sbor_typed_attributes(attributes)?.get_string_value("custom_value_kind")
}

pub fn get_custom_type_kind(attributes: &[Attribute]) -> Result<Option<String>> {
    extract_sbor_typed_attributes(attributes)?.get_string_value("custom_type_kind")
}

pub fn get_generic_types(generics: &Generics) -> Vec<Type> {
    generics
        .type_params()
        .map(|type_param| {
            let ident = &type_param.ident;
            parse_quote!(#ident)
        })
        .collect()
}

pub fn parse_comma_separated_types(source_string: &str) -> syn::Result<Vec<Type>> {
    source_string
        .split(',')
        .map(|s| s.trim().to_owned())
        .filter(|f| f.len() > 0)
        .map(|s| parse_str(&s))
        .collect()
}

fn get_child_types(attributes: &[Attribute], existing_generics: &Generics) -> Result<Vec<Type>> {
    let Some(comma_separated_types) = get_sbor_attribute_string_value(attributes, "child_types")? else {
        // If no explicit child_types list is set, we use all pre-existing generic type parameters.
        // This means (eg) that they all have to implement the relevant trait (Encode/Decode/Describe)
        // This is essentially what derived traits such as Clone do: https://github.com/rust-lang/rust/issues/26925
        // It's not perfect - but it's typically good enough!
        return Ok(get_generic_types(existing_generics));
    };

    parse_comma_separated_types(&comma_separated_types)
}

fn get_types_requiring_categorize_bound(
    attributes: &[Attribute],
    child_types: &[Type],
) -> Result<Vec<Type>> {
    let Some(comma_separated_types) = get_sbor_attribute_string_value(attributes, "categorize_types")? else {
        // A categorize bound is only needed for child types when you have a collection, eg Vec<T>
        // But if no explicit "categorize_types" is set, we assume all are needed.
        // These can be removed / overriden with the "categorize_types" field
        return Ok(child_types.to_owned());
    };

    parse_comma_separated_types(&comma_separated_types)
}

pub fn get_code_hash_const_array_token_stream(input: &TokenStream) -> TokenStream {
    let hash = get_hash_of_code(input);
    quote! {
        [#(#hash),*]
    }
}

pub fn get_hash_of_code(input: &TokenStream) -> [u8; 20] {
    const_sha1::sha1(input.to_string().as_bytes()).as_bytes()
}

pub fn get_unique_types<'a>(types: &[syn::Type]) -> Vec<syn::Type> {
    types.iter().unique().cloned().collect()
}

pub(crate) struct FieldsData {
    pub unskipped_field_names: Vec<TokenStream>,
    pub unskipped_field_name_strings: Vec<String>,
    pub unskipped_field_types: Vec<Type>,
    pub skipped_field_names: Vec<TokenStream>,
    pub skipped_field_types: Vec<Type>,
    pub fields_unpacking: TokenStream,
    pub empty_fields_unpacking: TokenStream,
    pub unskipped_unpacked_field_names: Vec<TokenStream>,
    pub unskipped_field_count: Index,
}

pub(crate) fn process_fields_for_categorize(fields: &syn::Fields) -> Result<FieldsData> {
    process_fields(fields, is_categorize_skipped)
}

pub(crate) fn process_fields_for_encode(fields: &syn::Fields) -> Result<FieldsData> {
    process_fields(fields, is_encoding_skipped)
}

pub(crate) fn process_fields_for_decode(fields: &syn::Fields) -> Result<FieldsData> {
    process_fields(fields, is_decoding_skipped)
}

pub(crate) fn process_fields_for_describe(fields: &syn::Fields) -> Result<FieldsData> {
    // Note - describe has to agree with decoding / encoding
    process_fields(fields, is_decoding_skipped)
}

fn process_fields(
    fields: &syn::Fields,
    is_skipped: impl Fn(&Field) -> Result<bool>,
) -> Result<FieldsData> {
    Ok(match fields {
        Fields::Named(fields) => {
            let mut unskipped_field_names = Vec::new();
            let mut unskipped_field_name_strings = Vec::new();
            let mut unskipped_field_types = Vec::new();
            let mut skipped_field_names = Vec::new();
            let mut skipped_field_types = Vec::new();
            for f in fields.named.iter() {
                let ident = &f.ident;
                if !is_skipped(f)? {
                    unskipped_field_names.push(quote! { #ident });
                    unskipped_field_name_strings
                        .push(ident.as_ref().map(|i| i.to_string()).unwrap_or_default());
                    unskipped_field_types.push(f.ty.clone());
                } else {
                    skipped_field_names.push(quote! { #ident });
                    skipped_field_types.push(f.ty.clone());
                }
            }

            let fields_unpacking = quote! {
                {#(#unskipped_field_names,)* ..}
            };
            let empty_fields_unpacking = quote! {
                { .. }
            };
            let unskipped_unpacked_field_names = unskipped_field_names.clone();

            let unskipped_field_count = Index::from(unskipped_field_names.len());

            FieldsData {
                unskipped_field_names,
                unskipped_field_name_strings,
                unskipped_field_types,
                skipped_field_names,
                skipped_field_types,
                fields_unpacking,
                empty_fields_unpacking,
                unskipped_unpacked_field_names,
                unskipped_field_count,
            }
        }
        Fields::Unnamed(fields) => {
            let mut unskipped_indices = Vec::new();
            let mut unskipped_field_name_strings = Vec::new();
            let mut unskipped_field_types = Vec::new();
            let mut unskipped_unpacked_field_names = Vec::new();
            let mut skipped_indices = Vec::new();
            let mut skipped_field_types = Vec::new();
            let mut unpacking_idents = Vec::new();
            let mut empty_idents = Vec::new();
            for (i, f) in fields.unnamed.iter().enumerate() {
                let index = Index::from(i);
                if !is_skipped(f)? {
                    unskipped_indices.push(quote! { #index });
                    unskipped_field_name_strings.push(i.to_string());
                    unskipped_field_types.push(f.ty.clone());
                    let unpacked_name_ident = format_ident!("a{}", i);
                    unskipped_unpacked_field_names.push(quote! { #unpacked_name_ident });
                    unpacking_idents.push(unpacked_name_ident);
                } else {
                    skipped_indices.push(quote! { #index });
                    skipped_field_types.push(f.ty.clone());
                    unpacking_idents.push(format_ident!("_"));
                }
                empty_idents.push(format_ident!("_"));
            }
            let fields_unpacking = quote! {
                (#(#unpacking_idents),*)
            };
            let empty_fields_unpacking = quote! {
                (#(#empty_idents),*)
            };

            let unskipped_field_count = Index::from(unskipped_indices.len());

            FieldsData {
                unskipped_field_names: unskipped_indices,
                unskipped_field_name_strings,
                unskipped_field_types,
                skipped_field_names: skipped_indices,
                skipped_field_types,
                fields_unpacking,
                empty_fields_unpacking,
                unskipped_unpacked_field_names,
                unskipped_field_count,
            }
        }
        Fields::Unit => FieldsData {
            unskipped_field_names: vec![],
            unskipped_field_name_strings: vec![],
            unskipped_field_types: vec![],
            skipped_field_names: vec![],
            skipped_field_types: vec![],
            fields_unpacking: quote! {},
            empty_fields_unpacking: quote! {},
            unskipped_unpacked_field_names: vec![],
            unskipped_field_count: Index::from(0),
        },
    })
}

pub fn add_where_predicate(
    optional_where: Option<&WhereClause>,
    predicate: WherePredicate,
) -> WhereClause {
    let mut where_clause = optional_where.cloned().unwrap_or(WhereClause {
        where_token: Default::default(),
        predicates: Default::default(),
    });
    where_clause.predicates.push(predicate);
    where_clause
}

pub fn build_decode_generics<'a>(
    original_generics: &'a Generics,
    attributes: &'a [Attribute],
    context_custom_value_kind: Option<&'static str>,
) -> syn::Result<(Generics, TypeGenerics<'a>, Option<WhereClause>, Path, Path)> {
    let custom_value_kind = get_custom_value_kind(&attributes)?;
    let (impl_generics, ty_generics, where_clause) = original_generics.split_for_impl();

    // Extract owned generic to allow mutation
    let mut impl_generics: Generics = parse_quote! { #impl_generics };

    let (custom_value_kind_generic, need_to_add_cvk_generic): (Path, bool) =
        if let Some(path) = custom_value_kind {
            (parse_str(path.as_str())?, false)
        } else if let Some(path) = context_custom_value_kind {
            (parse_str(path)?, false)
        } else {
            let custom_type_label = find_free_generic_name(original_generics, "X")?;
            (parse_str(&custom_type_label)?, true)
        };

    let decoder_label = find_free_generic_name(original_generics, "D")?;
    let decoder_generic: Path = parse_str(&decoder_label)?;

    let child_types = get_child_types(&attributes, &impl_generics)?;
    let categorize_types = get_types_requiring_categorize_bound(&attributes, &child_types)?;

    let mut where_clause = where_clause.cloned();
    if child_types.len() > 0 || categorize_types.len() > 0 {
        let mut new_where_clause = where_clause.unwrap_or(WhereClause {
            where_token: Default::default(),
            predicates: Default::default(),
        });
        for child_type in child_types {
            new_where_clause
                .predicates
                .push(parse_quote!(#child_type: ::sbor::Decode<#custom_value_kind_generic, #decoder_generic>));
        }
        for categorize_type in categorize_types {
            new_where_clause.predicates.push(
                parse_quote!(#categorize_type: ::sbor::Categorize<#custom_value_kind_generic>),
            );
        }
        where_clause = Some(new_where_clause);
    }

    impl_generics
        .params
        .push(parse_quote!(#decoder_generic: ::sbor::Decoder<#custom_value_kind_generic>));

    if need_to_add_cvk_generic {
        impl_generics
            .params
            .push(parse_quote!(#custom_value_kind_generic: ::sbor::CustomValueKind));
    }

    Ok((
        impl_generics,
        ty_generics,
        where_clause,
        custom_value_kind_generic,
        decoder_generic,
    ))
}

pub fn build_encode_generics<'a>(
    original_generics: &'a Generics,
    attributes: &'a [Attribute],
    context_custom_value_kind: Option<&'static str>,
) -> syn::Result<(Generics, TypeGenerics<'a>, Option<WhereClause>, Path, Path)> {
    let custom_value_kind = get_custom_value_kind(&attributes)?;
    let (impl_generics, ty_generics, where_clause) = original_generics.split_for_impl();

    // Extract owned generic to allow mutation
    let mut impl_generics: Generics = parse_quote! { #impl_generics };

    let (custom_value_kind_generic, need_to_add_cvk_generic): (Path, bool) =
        if let Some(path) = custom_value_kind {
            (parse_str(path.as_str())?, false)
        } else if let Some(path) = context_custom_value_kind {
            (parse_str(path)?, false)
        } else {
            let custom_type_label = find_free_generic_name(original_generics, "X")?;
            (parse_str(&custom_type_label)?, true)
        };

    let encoder_label = find_free_generic_name(original_generics, "E")?;
    let encoder_generic: Path = parse_str(&encoder_label)?;

    let child_types = get_child_types(&attributes, &impl_generics)?;
    let categorize_types = get_types_requiring_categorize_bound(&attributes, &child_types)?;

    let mut where_clause = where_clause.cloned();
    if child_types.len() > 0 || categorize_types.len() > 0 {
        let mut new_where_clause = where_clause.unwrap_or(WhereClause {
            where_token: Default::default(),
            predicates: Default::default(),
        });
        for child_type in child_types {
            new_where_clause
                .predicates
                .push(parse_quote!(#child_type: ::sbor::Encode<#custom_value_kind_generic, #encoder_generic>));
        }
        for categorize_type in categorize_types {
            new_where_clause.predicates.push(
                parse_quote!(#categorize_type: ::sbor::Categorize<#custom_value_kind_generic>),
            );
        }
        where_clause = Some(new_where_clause);
    }

    impl_generics
        .params
        .push(parse_quote!(#encoder_generic: ::sbor::Encoder<#custom_value_kind_generic>));

    if need_to_add_cvk_generic {
        impl_generics
            .params
            .push(parse_quote!(#custom_value_kind_generic: ::sbor::CustomValueKind));
    }

    Ok((
        impl_generics,
        ty_generics,
        where_clause,
        custom_value_kind_generic,
        encoder_generic,
    ))
}

pub fn build_describe_generics<'a>(
    original_generics: &'a Generics,
    attributes: &'a [Attribute],
    context_custom_type_kind: Option<&'static str>,
) -> syn::Result<(Generics, Generics, Option<WhereClause>, Vec<Type>, Path)> {
    let custom_type_kind = get_custom_type_kind(attributes)?;

    let (impl_generics, ty_generics, where_clause) = original_generics.split_for_impl();

    // Extract owned generic to allow mutation
    let mut impl_generics: Generics = parse_quote! { #impl_generics };

    let (custom_type_kind_generic, need_to_add_ctk_generic): (Path, bool) =
        if let Some(path) = custom_type_kind {
            (parse_str(path.as_str())?, false)
        } else if let Some(path) = context_custom_type_kind {
            (parse_str(&path)?, false)
        } else {
            let custom_type_label = find_free_generic_name(original_generics, "C")?;
            (parse_str(&custom_type_label)?, true)
        };

    let child_types = get_child_types(&attributes, &impl_generics)?;

    let mut where_clause = where_clause.cloned();
    if child_types.len() > 0 {
        let mut new_where_clause = where_clause.unwrap_or(WhereClause {
            where_token: Default::default(),
            predicates: Default::default(),
        });
        for child_type in child_types.iter() {
            new_where_clause
                .predicates
                .push(parse_quote!(#child_type: ::sbor::Describe<#custom_type_kind_generic>));
        }
        where_clause = Some(new_where_clause);
    }

    if need_to_add_ctk_generic {
        impl_generics.params.push(
            parse_quote!(#custom_type_kind_generic: ::sbor::CustomTypeKind<::sbor::GlobalTypeId>),
        );
    }

    let ty_generics: Generics = parse_quote! { #ty_generics };

    Ok((
        impl_generics,
        ty_generics,
        where_clause,
        child_types,
        custom_type_kind_generic,
    ))
}

pub fn build_custom_categorize_generic<'a>(
    original_generics: &'a Generics,
    attributes: &'a [Attribute],
    context_custom_value_kind: Option<&'static str>,
    require_categorize_on_generic_params: bool,
) -> syn::Result<(Generics, TypeGenerics<'a>, Option<&'a WhereClause>, Path)> {
    let custom_value_kind = get_custom_value_kind(&attributes)?;
    let (impl_generics, ty_generics, where_clause) = original_generics.split_for_impl();

    // Unwrap for mutation
    let mut impl_generics: Generics = parse_quote! { #impl_generics };

    let (custom_value_kind_generic, need_to_add_cvk_generic): (Path, bool) =
        if let Some(path) = custom_value_kind {
            (parse_str(path.as_str())?, false)
        } else if let Some(path) = context_custom_value_kind {
            (parse_str(path)?, false)
        } else {
            let custom_type_label = find_free_generic_name(original_generics, "X")?;
            (parse_str(&custom_type_label)?, true)
        };

    if require_categorize_on_generic_params {
        // In order to implement transparent Categorize, we need to pass through Categorize to the child field.
        // To do this, we need to ensure that type is Categorize.
        // So we add a bound that all pre-existing type parameters have to implement Categorize<X>
        // This is essentially what derived traits such as Clone do: https://github.com/rust-lang/rust/issues/26925
        // It's not perfect - but it's typically good enough!

        for param in impl_generics.params.iter_mut() {
            let GenericParam::Type(type_param) = param else {
                continue;
            };
            type_param
                .bounds
                .push(parse_quote!(::sbor::Categorize<#custom_value_kind_generic>));
        }
    }

    if need_to_add_cvk_generic {
        impl_generics
            .params
            .push(parse_quote!(#custom_value_kind_generic: ::sbor::CustomValueKind));
    }

    Ok((
        impl_generics,
        ty_generics,
        where_clause,
        custom_value_kind_generic,
    ))
}

fn find_free_generic_name(generics: &Generics, name_prefix: &str) -> syn::Result<String> {
    if !generic_already_exists(generics, name_prefix) {
        return Ok(name_prefix.to_owned());
    }
    for i in 0..100 {
        let name_attempt = format!("{}{}", name_prefix, i);
        if !generic_already_exists(generics, &name_attempt) {
            return Ok(name_attempt);
        }
    }

    return Err(Error::new(
        Span::call_site(),
        format!("Cannot find free generic name with prefix {}!", name_prefix),
    ));
}

fn generic_already_exists(generics: &Generics, name: &str) -> bool {
    generics
        .params
        .iter()
        .any(|p| &get_generic_param_name(p) == name)
}

fn get_generic_param_name(generic_param: &GenericParam) -> String {
    match generic_param {
        GenericParam::Type(type_param) => type_param.ident.to_string(),
        GenericParam::Lifetime(lifetime_param) => lifetime_param.lifetime.to_string(),
        GenericParam::Const(const_param) => const_param.ident.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_attributes() {
        let attr: Attribute = parse_quote! {
            #[sbor(skip, custom_value_kind = "NoCustomValueKind")]
        };
        let extracted = extract_typed_attributes(&[attr], "sbor").unwrap();
        assert_eq!(extracted.get_bool_value("skip").unwrap(), true);
        assert_eq!(extracted.get_bool_value("skip2").unwrap(), false);
        assert!(matches!(
            extracted.get_bool_value("custom_value_kind"),
            Err(_)
        ));
        assert_eq!(
            extracted.get_string_value("custom_value_kind").unwrap(),
            Some("NoCustomValueKind".to_string())
        );
        assert_eq!(
            extracted.get_string_value("custom_value_kind_2").unwrap(),
            None
        );
        assert!(matches!(extracted.get_string_value("skip"), Err(_)));
    }
}
