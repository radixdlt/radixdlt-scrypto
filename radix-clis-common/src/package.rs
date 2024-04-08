use std::path::*;

use regex::Regex;

pub fn new_package(
    package_name: &str,
    path: Option<PathBuf>,
    local: bool,
) -> Result<(), PackageError> {
    let wasm_name = package_name.replace('-', "_");
    let path = path.clone().unwrap_or(PathBuf::from(package_name));
    let simulator_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    if path.exists() {
        Err(PackageError::PackageAlreadyExists)
    } else {
        std::fs::create_dir_all(child_of(&path, "src")).map_err(PackageError::IOError)?;
        std::fs::create_dir_all(child_of(&path, "tests")).map_err(PackageError::IOError)?;

        let local_dependencies_path = if local {
            Some(PathBuf::from(
                simulator_dir
                    .parent()
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/"),
            ))
        } else {
            None
        };

        std::fs::write(
            child_of(&path, "Cargo.toml"),
            new_cargo_manifest(local_dependencies_path),
        )
        .map_err(PackageError::IOError)?;

        std::fs::write(
            child_of(&path, ".gitignore"),
            include_str!("../assets/template/.gitignore"),
        )
        .map_err(PackageError::IOError)?;

        std::fs::write(
            child_of(&child_of(&path, "src"), "lib.rs"),
            include_str!("../assets/template/src/lib.rs"),
        )
        .map_err(PackageError::IOError)?;

        std::fs::write(
            child_of(&child_of(&path, "tests"), "lib.rs"),
            include_str!("../assets/template/tests/lib.rs").replace("${wasm_name}", &wasm_name),
        )
        .map_err(PackageError::IOError)?;

        Ok(())
    }
}

fn child_of(path: &PathBuf, name: &str) -> PathBuf {
    let mut p = path.clone();
    p.push(name);
    p
}

#[derive(Debug)]
pub enum PackageError {
    PackageAlreadyExists,
    IOError(std::io::Error),
}

pub fn new_cargo_manifest(use_local_dependencies_at: Option<PathBuf>) -> String {
    let pattern = Regex::new(r"\$\{dep:([\w\d\-_]*)\}").unwrap();
    let template_file = include_str!("../assets/template/Cargo.toml_template");
    let manifest_file = if let Some(local_dependencies_path) = use_local_dependencies_at {
        pattern.replace(
            template_file,
            format!("{{ path = \"{}/$1\" }}", local_dependencies_path.display()),
        )
    } else {
        pattern.replace(
            template_file,
            format!(
                "{{ git = \"https://github.com/radixdlt/$1\", tag = \"{}\" }}",
                env!("CARGO_PKG_VERSION")
            ),
        )
    };
    (*manifest_file).to_owned()
}
