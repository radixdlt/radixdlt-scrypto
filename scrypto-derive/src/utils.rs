use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;

pub fn write_and_fmt<P: AsRef<Path>, S: ToString>(path: P, code: S) -> io::Result<()> {
    fs::write(&path, code.to_string())?;

    Command::new("rustfmt").arg(path.as_ref()).spawn()?.wait()?;

    Ok(())
}

pub fn print_compiled_code<S: ToString>(kind: &str, code: S) {
    if cfg!(trace) {
        let file = "/tmp/code.rs";
        let result = write_and_fmt(file, code);

        if result.is_ok() {
            let formatted = fs::read_to_string(file).expect("Unable to open formatted code");
            println!("{}\n{}:\n----\n{}----\n", "=".repeat(40), kind, formatted);
        }
    }
}
