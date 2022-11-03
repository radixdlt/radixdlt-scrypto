use std::collections::HashMap;
use std::io::Write;
use std::process::Command;
use std::process::Stdio;

use syn::parse_quote;
use syn::parse_str;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::Attribute;
use syn::Expr;
use syn::ExprLit;
use syn::Field;
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

pub fn build_generics(
    generics: &Generics,
    custom_type_id: Option<String>,
) -> syn::Result<(Generics, TypeGenerics, Option<&WhereClause>, Path)> {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Unwrap for mutation
    let mut impl_generics: Generics = parse_quote! { #impl_generics };

    let sbor_cti = if let Some(path) = custom_type_id {
        parse_str(path.as_str())?
    } else {
        // Note that this above logic requires no use of CTI generic param by the input type.
        // TODO: better to report error OR take an alternative name if already exists
        impl_generics
            .params
            .push(parse_quote!(CTI: ::sbor::type_id::CustomTypeId));
        parse_quote! { CTI }
    };

    Ok((impl_generics, ty_generics, where_clause, sbor_cti))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_attributes() {
        let attr = parse_quote! {
            #[sbor(skip, type_id = "NoCustomTypeId")]
        };
        assert_eq!(
            extract_attributes(&[attr]),
            HashMap::from([
                ("skip".to_owned(), None),
                ("type_id".to_owned(), Some("NoCustomTypeId".to_owned()))
            ])
        );
    }
}
