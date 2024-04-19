use std::path::*;

pub fn new_package(
    package_name: &str,
    path: Option<PathBuf>,
    local: bool,
) -> Result<(), PackageError> {
    let wasm_name = package_name.replace('-', "_");
    let path = path.clone().unwrap_or(PathBuf::from(package_name));
    let simulator_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let (
        sbor,
        scrypto,
        transaction,
        radix_engine,
        radix_engine_interface,
        scrypto_test,
        optional_scrypto_test,
    ) = if local {
        let scrypto_dir = simulator_dir
            .parent()
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        (
            format!("{{ path = \"{}/sbor\" }}", scrypto_dir),
            format!("{{ path = \"{}/scrypto\" }}", scrypto_dir),
            format!("{{ path = \"{}/transaction\" }}", scrypto_dir),
            format!("{{ path = \"{}/radix-engine\" }}", scrypto_dir),
            format!("{{ path = \"{}/radix-engine-interface\" }}", scrypto_dir),
            format!("{{ path = \"{}/scrypto-test\" }}", scrypto_dir),
            format!(
                "{{ path = \"{}/scrypto-test\", optional = true }}",
                scrypto_dir
            ),
        )
    } else {
        let s = format!(
            "{{ git = \"https://github.com/radixdlt/radixdlt-scrypto\", tag = \"v{}\" }}",
            env!("CARGO_PKG_VERSION")
        );
        (s.clone(), s.clone(), s.clone(), s.clone(), s.clone(), s, format!(
                "{{ git = \"https://github.com/radixdlt/radixdlt-scrypto\", tag = \"v{}\", optional = true }}",
                env!("CARGO_PKG_VERSION")
            ))
    };

    if path.exists() {
        Err(PackageError::PackageAlreadyExists)
    } else {
        std::fs::create_dir_all(child_of(&path, "src")).map_err(PackageError::IOError)?;
        std::fs::create_dir_all(child_of(&path, "tests")).map_err(PackageError::IOError)?;

        std::fs::write(
            child_of(&path, "Cargo.toml"),
            include_str!("../assets/template/Cargo.toml_template")
                .replace("${package_name}", package_name)
                .replace("${sbor}", &sbor)
                .replace("${scrypto}", &scrypto)
                .replace("${transaction}", &transaction)
                .replace("${radix-engine}", &radix_engine)
                .replace("${radix-engine-interface}", &radix_engine_interface)
                .replace("${scrypto-test}", &scrypto_test)
                .replace("${optional-scrypto-test}", &optional_scrypto_test),
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
