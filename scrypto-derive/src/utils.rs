use std::collections::HashMap;
use std::io::Write;
use std::process::Command;
use std::process::Stdio;

use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::Attribute;
use syn::Expr;
use syn::ExprLit;
use syn::Field;
use syn::Lit;

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
        if !attr.path.is_ident("scrypto") {
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

pub fn is_mutable(f: &Field) -> bool {
    let parsed = extract_attributes(&f.attrs);
    parsed.contains_key("mutable")
}
