use clap::Parser;
use regex::Regex;
use std::fs;
use std::path::PathBuf;

use crate::scrypto::*;

/// Create a Scrypto package
#[derive(Parser, Debug)]
pub struct NewPackage {
    /// The package name
    package_name: String,

    /// The package directory
    #[clap(long)]
    path: Option<PathBuf>,

    /// Use local Scrypto as dependency
    #[clap(short, long)]
    local: bool,
}

impl NewPackage {
    pub fn run(&self) -> Result<(), String> {
        let wasm_name = self.package_name.replace("-", "_");
        let path = self
            .path
            .clone()
            .unwrap_or(PathBuf::from(&self.package_name));
        let simulator_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let (sbor, scrypto, scrypto_test) = if self.local {
            let scrypto_dir = simulator_dir
                .parent()
                .unwrap()
                .to_string_lossy()
                .replace("\\", "/");
            (
                format!("{{ path = \"{}/sbor\" }}", scrypto_dir),
                format!("{{ path = \"{}/scrypto\" }}", scrypto_dir),
                format!("{{ path = \"{}/scrypto-test\" }}", scrypto_dir),
            )
        } else {
            let s = format!("{{ version = \"{}\" }}", env!("CARGO_PKG_VERSION"));
            (s.clone(), s.clone(), s.clone())
        };

        if path.exists() {
            Err(Error::PackageAlreadyExists.into())
        } else {
            fs::create_dir_all(child_of(&path, "src")).map_err(Error::IOError)?;
            fs::create_dir_all(child_of(&path, "tests")).map_err(Error::IOError)?;

            fs::write(
                child_of(&path, "Cargo.toml"),
                include_str!("../../assets/template/Cargo.toml_template")
                    .replace("${package_name}", &self.package_name)
                    .replace("${sbor}", &sbor)
                    .replace("${scrypto}", &scrypto)
                    .replace("${scrypto-test}", &scrypto_test),
            )
            .map_err(Error::IOError)?;

            // This is tested in `./tests/scrypto.sh` by verifying that a newly
            // created package can be built with --locked against the lock file.
            let cargo_lock = Self::insert_own_entry_into_cargo_lock(
                include_str!("../../assets/template/Cargo.lock_template"),
                &self.package_name,
            );
            fs::write(child_of(&path, "Cargo.lock"), cargo_lock).map_err(Error::IOError)?;

            fs::write(
                child_of(&path, ".gitignore"),
                include_str!("../../assets/template/.gitignore"),
            )
            .map_err(Error::IOError)?;

            fs::write(
                child_of(&child_of(&path, "src"), "lib.rs"),
                include_str!("../../assets/template/src/lib.rs"),
            )
            .map_err(Error::IOError)?;

            fs::write(
                child_of(&child_of(&path, "tests"), "lib.rs"),
                include_str!("../../assets/template/tests/lib.rs")
                    .replace("${wasm_name}", &wasm_name),
            )
            .map_err(Error::IOError)?;

            fs::write(
                child_of(&path, "rust-toolchain.toml"),
                include_str!("../../assets/template/rust-toolchain.toml_template"),
            )
            .map_err(Error::IOError)?;

            Ok(())
        }
    }

    fn insert_own_entry_into_cargo_lock(lock_file_contents: &str, package_name: &str) -> String {
        let name_regex = Regex::new(r#"name = "([^"]+)""#).unwrap();
        let name_line_to_inject_before = name_regex
            .captures_iter(&lock_file_contents)
            .find(|captures| {
                let dependency_name = captures.get(1).unwrap();
                dependency_name.as_str() > package_name
            })
            .map(|captures| captures.get(0).unwrap().as_str());

        if let Some(line) = name_line_to_inject_before {
            lock_file_contents.replace(
                line,
                &format!(
                    r#"name = "{package_name}"
version = "1.0.0"
dependencies = [
 "scrypto",
 "scrypto-test",
]

[[package]]
{line}"#
                ),
            )
        } else {
            let mut contents = lock_file_contents.to_string();
            contents.push_str(&format!(
                r#"
[[package]]
name = "{package_name}"
version = "1.0.0"
dependencies = [
 "scrypto",
 "scrypto-test",
]
"#
            ));
            contents
        }
    }
}

fn child_of(path: &PathBuf, name: &str) -> PathBuf {
    let mut p = path.clone();
    p.push(name);
    p
}
