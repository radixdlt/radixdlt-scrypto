use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;

#[allow(dead_code)]
pub fn write_and_fmt<P: AsRef<Path>, S: ToString>(path: P, code: S) -> io::Result<()> {
    fs::write(&path, code.to_string())?;

    Command::new("rustfmt").arg(path.as_ref()).spawn()?.wait()?;

    Ok(())
}

#[allow(dead_code)]
pub fn print_compiled_code<S: ToString>(kind: &str, code: S) {
    let mut path = std::env::temp_dir();
    path.push(format!("{}.rs", uuid::Uuid::new_v4()));

    let result = write_and_fmt(path.clone(), code);
    if result.is_ok() {
        let formatted = fs::read_to_string(path.clone()).expect("Unable to open formatted code");
        println!(
            "{}\n{}\n{}\n{}\n",
            "-".repeat(kind.len()),
            kind,
            "-".repeat(kind.len()),
            formatted
        );
        fs::remove_file(path).unwrap();
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
