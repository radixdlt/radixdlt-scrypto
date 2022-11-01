use std::io::Write;
use std::process::Command;
use std::process::Stdio;

use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::Path;

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

pub fn is_skipped(f: &syn::Field, id: &str) -> bool {
    f.attrs.iter().any(|attr| {
        if attr.path.is_ident("skip") {
            if let Ok(parsed) = attr.parse_args_with(Punctuated::<Path, Comma>::parse_terminated) {
                if parsed.iter().any(|x| x.is_ident(id)) {
                    return true;
                }
            }
        }
        return false;
    })
}

pub fn is_describe_skipped(f: &syn::Field) -> bool {
    is_skipped(f, "Describe")
}
