use itertools::Itertools;
use std::collections::BTreeMap;
use std::io::Write;
use std::process::Command;
use std::process::Stdio;

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

pub fn extract_attributes(
    attrs: &[Attribute],
    name: &str,
) -> Option<BTreeMap<String, Option<String>>> {
    for attr in attrs {
        if !attr.path.is_ident(name) {
            continue;
        }

        let mut fields = BTreeMap::new();
        if let Ok(meta) = attr.parse_meta() {
            if let Meta::List(MetaList { nested, .. }) = meta {
                nested.into_iter().for_each(|m| match m {
                    NestedMeta::Meta(m) => match m {
                        Meta::NameValue(name_value) => {
                            if let Some(ident) = name_value.path.get_ident() {
                                if let Lit::Str(s) = name_value.lit {
                                    fields.insert(ident.to_string(), Some(s.value()));
                                }
                            }
                        }
                        Meta::Path(path) => {
                            if let Some(ident) = path.get_ident() {
                                fields.insert(ident.to_string(), None);
                            }
                        }
                        _ => {}
                    },
                    _ => {}
                })
            }
        }
        return Some(fields);
    }

    None
}

fn get_sbor_attribute_field_value(attributes: &[Attribute], field_name: &str) -> Option<String> {
    if let Some(fields) = extract_attributes(attributes, "sbor") {
        fields.get(field_name).cloned().unwrap_or_default()
    } else {
        None
    }
}

pub fn is_categorize_skipped(f: &Field) -> bool {
    if let Some(fields) = extract_attributes(&f.attrs, "sbor") {
        fields.contains_key("skip") || fields.contains_key("skip_categorize")
    } else {
        false
    }
}

pub fn is_decoding_skipped(f: &Field) -> bool {
    if let Some(fields) = extract_attributes(&f.attrs, "sbor") {
        fields.contains_key("skip") || fields.contains_key("skip_decode")
    } else {
        false
    }
}

pub fn is_encoding_skipped(f: &Field) -> bool {
    if let Some(fields) = extract_attributes(&f.attrs, "sbor") {
        fields.contains_key("skip") || fields.contains_key("skip_encode")
    } else {
        false
    }
}

pub fn is_describing_skipped(f: &Field) -> bool {
    if let Some(fields) = extract_attributes(&f.attrs, "sbor") {
        fields.contains_key("skip") || fields.contains_key("skip_describe")
    } else {
        false
    }
}

pub fn is_transparent(attributes: &[Attribute]) -> bool {
    if let Some(fields) = extract_attributes(attributes, "sbor") {
        fields.contains_key("transparent")
    } else {
        false
    }
}

pub fn get_custom_value_kind(attributes: &[Attribute]) -> Option<String> {
    get_sbor_attribute_field_value(attributes, "custom_value_kind")
}

pub fn get_custom_type_kind(attributes: &[Attribute]) -> Option<String> {
    get_sbor_attribute_field_value(attributes, "custom_type_kind")
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

fn get_child_types(
    attributes: &[Attribute],
    existing_generics: &Generics,
) -> syn::Result<Vec<Type>> {
    let Some(comma_separated_types) = get_sbor_attribute_field_value(attributes, "child_types") else {
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
) -> syn::Result<Vec<Type>> {
    let Some(comma_separated_types) = get_sbor_attribute_field_value(attributes, "categorize_types") else {
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

pub fn get_unique_types<'a>(types: &[&'a syn::Type]) -> Vec<&'a syn::Type> {
    types.into_iter().unique().cloned().collect()
}

pub(crate) struct FieldsData {
    pub unskipped_self_field_names: Vec<TokenStream>,
    pub unskipped_field_types: Vec<Type>,
    pub skipped_self_field_names: Vec<TokenStream>,
    pub skipped_field_types: Vec<Type>,
    pub fields_unpacking: TokenStream,
    pub unskipped_unpacked_field_names: Vec<TokenStream>,
    pub unskipped_field_count: Index,
}

pub(crate) fn process_fields_for_categorize(fields: &syn::Fields) -> FieldsData {
    process_fields(fields, is_categorize_skipped)
}

pub(crate) fn process_fields_for_encode(fields: &syn::Fields) -> FieldsData {
    process_fields(fields, is_encoding_skipped)
}

pub(crate) fn process_fields_for_decode(fields: &syn::Fields) -> FieldsData {
    process_fields(fields, is_decoding_skipped)
}

pub(crate) fn process_fields_for_describe(fields: &syn::Fields) -> FieldsData {
    process_fields(fields, is_describing_skipped)
}

fn process_fields(fields: &syn::Fields, is_skipped: impl Fn(&Field) -> bool) -> FieldsData {
    match fields {
        Fields::Named(fields) => {
            let mut unskipped_self_field_names = Vec::new();
            let mut unskipped_field_types = Vec::new();
            let mut skipped_self_field_names = Vec::new();
            let mut skipped_field_types = Vec::new();
            for f in fields.named.iter() {
                let ident = &f.ident;
                if !is_skipped(f) {
                    unskipped_self_field_names.push(quote! { #ident });
                    unskipped_field_types.push(f.ty.clone());
                } else {
                    skipped_self_field_names.push(quote! { #ident });
                    skipped_field_types.push(f.ty.clone());
                }
            }

            let fields_unpacking = quote! {
                {#(#unskipped_self_field_names,)* ..}
            };
            let unskipped_unpacked_field_names = unskipped_self_field_names.clone();

            let unskipped_field_count = Index::from(unskipped_self_field_names.len());

            FieldsData {
                unskipped_self_field_names,
                unskipped_field_types,
                skipped_self_field_names,
                skipped_field_types,
                fields_unpacking,
                unskipped_unpacked_field_names,
                unskipped_field_count,
            }
        }
        Fields::Unnamed(fields) => {
            let mut unskipped_indices = Vec::new();
            let mut unskipped_field_types = Vec::new();
            let mut unskipped_unpacked_field_names = Vec::new();
            let mut skipped_indices = Vec::new();
            let mut skipped_field_types = Vec::new();
            let mut unpacking_idents = Vec::new();
            for (i, f) in fields.unnamed.iter().enumerate() {
                let index = Index::from(i);
                if !is_skipped(f) {
                    unskipped_indices.push(quote! { #index });
                    unskipped_field_types.push(f.ty.clone());
                    let unpacked_name_ident = format_ident!("a{}", i);
                    unskipped_unpacked_field_names.push(quote! { #unpacked_name_ident });
                    unpacking_idents.push(unpacked_name_ident);
                } else {
                    skipped_indices.push(quote! { #index });
                    skipped_field_types.push(f.ty.clone());
                    unpacking_idents.push(format_ident!("_"));
                }
            }
            let fields_unpacking = quote! {
                (#(#unpacking_idents),*)
            };

            let unskipped_field_count = Index::from(unskipped_indices.len());

            FieldsData {
                unskipped_self_field_names: unskipped_indices,
                unskipped_field_types,
                skipped_self_field_names: skipped_indices,
                skipped_field_types,
                fields_unpacking,
                unskipped_unpacked_field_names,
                unskipped_field_count,
            }
        }
        Fields::Unit => FieldsData {
            unskipped_self_field_names: vec![],
            unskipped_field_types: vec![],
            skipped_self_field_names: vec![],
            skipped_field_types: vec![],
            fields_unpacking: quote! {},
            unskipped_unpacked_field_names: vec![],
            unskipped_field_count: Index::from(0),
        },
    }
}

pub fn build_decode_generics<'a>(
    original_generics: &'a Generics,
    attributes: &'a [Attribute],
    context_custom_value_kind: Option<&'static str>,
) -> syn::Result<(Generics, TypeGenerics<'a>, Option<WhereClause>, Path, Path)> {
    let custom_value_kind = get_custom_value_kind(&attributes);
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
    let custom_value_kind = get_custom_value_kind(&attributes);
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
    let custom_type_kind = get_custom_type_kind(attributes);

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
    let categorize_types = get_types_requiring_categorize_bound(&attributes, &child_types)?;

    let mut where_clause = where_clause.cloned();
    if child_types.len() > 0 || categorize_types.len() > 0 {
        let mut new_where_clause = where_clause.unwrap_or(WhereClause {
            where_token: Default::default(),
            predicates: Default::default(),
        });
        for child_type in child_types.iter() {
            new_where_clause
                .predicates
                .push(parse_quote!(#child_type: ::sbor::Describe<#custom_type_kind_generic>));
        }
        for categorize_type in categorize_types {
            new_where_clause
                .predicates
                .push(parse_quote!(#categorize_type: ::sbor::Categorize<<
                    #custom_type_kind_generic as::sbor::CustomTypeKind<::sbor::GlobalTypeId>
                >::CustomValueKind>));
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
    let custom_value_kind = get_custom_value_kind(&attributes);
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
    fn test_extract_attribute_name_values() {
        let attr: Attribute = parse_quote! {
            #[sbor(skip, custom_value_kind = "NoCustomValueKind")]
        };
        assert_eq!(
            extract_attributes(&[attr.clone()], "sbor"),
            Some(BTreeMap::from([
                ("skip".to_owned(), None),
                (
                    "custom_value_kind".to_owned(),
                    Some("NoCustomValueKind".to_owned())
                )
            ]))
        );
        assert_eq!(extract_attributes(&[attr], "mutable"), None);
    }

    #[test]
    fn test_extract_attribute_path() {
        let attr: Attribute = parse_quote! {
            #[mutable]
        };
        assert_eq!(extract_attributes(&[attr.clone()], "sbor"), None);
        assert_eq!(
            extract_attributes(&[attr], "mutable"),
            Some(BTreeMap::new())
        );
    }
}
