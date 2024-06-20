#[cfg(not(feature = "compile-blueprints-at-build-time"))]
fn main() {}

#[cfg(feature = "compile-blueprints-at-build-time")]
fn main() {
    use std::env;
    use std::path::PathBuf;

    use cargo_toml::{Manifest, Package};
    use scrypto::prelude::*;

    let manifest_dir = PathBuf::from_str(env!("CARGO_MANIFEST_DIR")).unwrap();
    let blueprints_dir = manifest_dir.join("assets").join("blueprints");
    println!("cargo:rerun-if-changed=\"{:?}\"", blueprints_dir);

    let mut packages = HashMap::new();
    for entry in walkdir::WalkDir::new(blueprints_dir) {
        let Ok(entry) = entry else {
            continue;
        };
        let path = entry.path();
        if !path
            .file_name()
            .map_or(false, |file_name| file_name == "Cargo.toml")
        {
            continue;
        }

        let manifest = Manifest::from_path(path).unwrap();
        if !manifest
            .dependencies
            .into_iter()
            .any(|(name, _)| name == "scrypto")
        {
            continue;
        }

        let Some(Package { name, .. }) = manifest.package else {
            continue;
        };

        let (code, definition) = scrypto_test::prelude::Compile::compile_with_env_vars(
            path.parent().unwrap(),
            btreemap! {
                "RUSTFLAGS".to_owned() => "".to_owned(),
                "CARGO_ENCODED_RUSTFLAGS".to_owned() => "".to_owned(),
                "LLVM_PROFILE_FILE".to_owned() => "".to_owned()
            },
            scrypto_test::prelude::CompileProfile::Fast,
            false,
        );
        packages.insert(name, (code, definition));
    }

    let out_dir = PathBuf::from_str(env::var("OUT_DIR").unwrap().as_str()).unwrap();
    let compilation_path = out_dir.join("compiled_packages.bin");

    let encoded_packages = scrypto_encode(&packages).unwrap();
    std::fs::write(compilation_path, encoded_packages).unwrap();
}
