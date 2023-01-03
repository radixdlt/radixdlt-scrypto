use std::collections::HashMap;
use std::io::Write;
use std::process::Command;
use std::process::Stdio;

use proc_macro2::Span;
use syn::parse_quote;
use syn::parse_str;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::Attribute;
use syn::Error;
use syn::Expr;
use syn::ExprLit;
use syn::Field;
use syn::GenericParam;
use syn::Generics;
use syn::Lit;
use syn::Path;
use syn::TypeGenerics;
use syn::WhereClause;

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

pub fn extract_attributes(attrs: &[Attribute]) -> HashMap<String, Option<String>> {
    let mut configs = HashMap::new();

    for attr in attrs {
        if !attr.path.is_ident("sbor") {
            continue;
        }

        if let Ok(parsed) = attr.parse_args_with(Punctuated::<Expr, Comma>::parse_terminated) {
            parsed.into_iter().for_each(|s| match s {
                Expr::Assign(assign) => {
                    if let Expr::Path(path_expr) = assign.left.as_ref() {
                        if let Some(ident) = path_expr.path.get_ident() {
                            if let Expr::Lit(ExprLit {
                                lit: Lit::Str(s), ..
                            }) = assign.right.as_ref()
                            {
                                configs.insert(ident.to_string(), Some(s.value()));
                            }
                        }
                    }
                }
                Expr::Path(path_expr) => {
                    if let Some(ident) = path_expr.path.get_ident() {
                        configs.insert(ident.to_string(), None);
                    }
                }
                _ => {}
            })
        }
    }

    configs
}

pub fn is_decoding_skipped(f: &Field) -> bool {
    let parsed = extract_attributes(&f.attrs);
    parsed.contains_key("skip") || parsed.contains_key("skip_encoding")
}

pub fn is_encoding_skipped(f: &Field) -> bool {
    let parsed = extract_attributes(&f.attrs);
    parsed.contains_key("skip") || parsed.contains_key("skip_decoding")
}

pub fn custom_type_id(attrs: &[Attribute]) -> Option<String> {
    extract_attributes(attrs)
        .get("custom_type_id")
        .cloned()
        .unwrap_or(None)
}

pub fn build_decode_generics(
    original_generics: &Generics,
    custom_type_id: Option<String>,
) -> syn::Result<(Generics, TypeGenerics, Option<&WhereClause>, Path, Path)> {
    let (impl_generics, ty_generics, where_clause) = original_generics.split_for_impl();

    // Extract owned generic to allow mutation
    let mut impl_generics: Generics = parse_quote! { #impl_generics };

    let (custom_type_id_generic, need_to_add_cti_generic): (Path, bool) =
        if let Some(path) = custom_type_id {
            (parse_str(path.as_str())?, false)
        } else {
            let custom_type_label = find_free_generic_name(original_generics, "X")?;
            (parse_str(&custom_type_label)?, true)
        };

    let decoder_label = find_free_generic_name(original_generics, "D")?;
    let decoder_generic: Path = parse_str(&decoder_label)?;

    // Add a bound that all pre-existing type parameters have to implement Decode<X, D>
    // This is essentially what derived traits such as Clone do: https://github.com/rust-lang/rust/issues/26925
    // It's not perfect - but it's typically good enough!

    for param in impl_generics.params.iter_mut() {
        let GenericParam::Type(type_param) = param else {
            continue;
        };
        type_param
            .bounds
            .push(parse_quote!(::sbor::Decode<#custom_type_id_generic, #decoder_generic>));
    }

    impl_generics
        .params
        .push(parse_quote!(#decoder_generic: ::sbor::Decoder<#custom_type_id_generic>));

    if need_to_add_cti_generic {
        impl_generics
            .params
            .push(parse_quote!(#custom_type_id_generic: ::sbor::CustomTypeId));
    }

    Ok((
        impl_generics,
        ty_generics,
        where_clause,
        custom_type_id_generic,
        decoder_generic,
    ))
}

pub fn build_encode_generics(
    original_generics: &Generics,
    custom_type_id: Option<String>,
) -> syn::Result<(Generics, TypeGenerics, Option<&WhereClause>, Path, Path)> {
    let (impl_generics, ty_generics, where_clause) = original_generics.split_for_impl();

    // Extract owned generic to allow mutation
    let mut impl_generics: Generics = parse_quote! { #impl_generics };

    let (custom_type_id_generic, need_to_add_cti_generic): (Path, bool) =
        if let Some(path) = custom_type_id {
            (parse_str(path.as_str())?, false)
        } else {
            let custom_type_label = find_free_generic_name(original_generics, "X")?;
            (parse_str(&custom_type_label)?, true)
        };

    let encoder_label = find_free_generic_name(original_generics, "E")?;
    let encoder_generic: Path = parse_str(&encoder_label)?;

    // Add a bound that all pre-existing type parameters have to implement Encode<X, E>
    // This is essentially what derived traits such as Clone do: https://github.com/rust-lang/rust/issues/26925
    // It's not perfect - but it's typically good enough!

    for param in impl_generics.params.iter_mut() {
        let GenericParam::Type(type_param) = param else {
            continue;
        };
        type_param
            .bounds
            .push(parse_quote!(::sbor::Encode<#custom_type_id_generic, #encoder_generic>));
    }

    impl_generics
        .params
        .push(parse_quote!(#encoder_generic: ::sbor::Encoder<#custom_type_id_generic>));

    if need_to_add_cti_generic {
        impl_generics
            .params
            .push(parse_quote!(#custom_type_id_generic: ::sbor::CustomTypeId));
    }

    Ok((
        impl_generics,
        ty_generics,
        where_clause,
        custom_type_id_generic,
        encoder_generic,
    ))
}

pub fn build_custom_type_id_generic(
    original_generics: &Generics,
    custom_type_id: Option<String>,
) -> syn::Result<(Generics, TypeGenerics, Option<&WhereClause>, Path)> {
    let (impl_generics, ty_generics, where_clause) = original_generics.split_for_impl();

    // Unwrap for mutation
    let mut impl_generics: Generics = parse_quote! { #impl_generics };

    let sbor_cti = if let Some(path) = custom_type_id {
        parse_str(path.as_str())?
    } else {
        let custom_type_label = find_free_generic_name(original_generics, "X")?;
        let custom_type_id_generic = parse_str(&custom_type_label)?;
        impl_generics
            .params
            .push(parse_quote!(#custom_type_id_generic: ::sbor::CustomTypeId));
        custom_type_id_generic
    };

    Ok((impl_generics, ty_generics, where_clause, sbor_cti))
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
        let attr = parse_quote! {
            #[sbor(skip, custom_type_id = "NoCustomTypeId")]
        };
        assert_eq!(
            extract_attributes(&[attr]),
            HashMap::from([
                ("skip".to_owned(), None),
                (
                    "custom_type_id".to_owned(),
                    Some("NoCustomTypeId".to_owned())
                )
            ])
        );
    }
}
