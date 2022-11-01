use std::io::Write;
use std::process::Command;
use std::process::Stdio;

use syn::parse_quote;
use syn::Generics;
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

pub fn is_skipped(f: &syn::Field) -> bool {
    let mut skipped = false;
    for att in &f.attrs {
        if att.path.is_ident("sbor")
            && att
                .parse_args::<syn::Path>()
                .map(|p| p.is_ident("skip"))
                .unwrap_or(false)
        {
            skipped = true;
        }
    }
    skipped
}

pub fn extend_generics_with_cti(
    generics: &Generics,
) -> syn::Result<(Generics, TypeGenerics, Option<&WhereClause>)> {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Note that this above logic requires no use of CTI generic param by the input type.
    // TODO: better to report error OR an alternative name if already exists
    let mut impl_generics: Generics = parse_quote! { #impl_generics };
    impl_generics
        .params
        .push(parse_quote!(CTI: ::sbor::type_id::CustomTypeId));

    Ok((impl_generics, ty_generics, where_clause))
}
