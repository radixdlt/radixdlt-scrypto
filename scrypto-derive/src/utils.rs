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

#[allow(unused_variables)]
pub fn print_compiled_code<S: ToString>(kind: &str, code: S) {
    #[cfg(feature = "trace")]
    {
        let mut path = std::env::temp_dir();
        path.push(format!("{}.rs", uuid::Uuid::new_v4()));

        let result = write_and_fmt(path.clone(), code);
        if result.is_ok() {
            let formatted =
                fs::read_to_string(path.clone()).expect("Unable to open formatted code");
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
}
