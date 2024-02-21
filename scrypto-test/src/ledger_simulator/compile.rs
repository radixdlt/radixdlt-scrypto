use crate::prelude::*;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct Compile;

impl Compile {
    pub fn compile<P: AsRef<Path>>(package_dir: P) -> (Vec<u8>, PackageDefinition) {
        Self::compile_with_env_vars(
            package_dir,
            btreemap! {
                "RUSTFLAGS".to_owned() => "".to_owned(),
                "CARGO_ENCODED_RUSTFLAGS".to_owned() => "".to_owned(),
            },
        )
    }

    pub fn compile_with_env_vars<P: AsRef<Path>>(
        package_dir: P,
        env_vars: sbor::rust::collections::BTreeMap<String, String>,
    ) -> (Vec<u8>, PackageDefinition) {
        // Find wasm name
        let mut cargo = package_dir.as_ref().to_owned();
        cargo.push("Cargo.toml");
        let wasm_name = if cargo.exists() {
            let content = fs::read_to_string(&cargo).expect("Failed to read the Cargo.toml file");
            Self::extract_crate_name(&content)
                .expect("Failed to extract crate name from the Cargo.toml file")
                .replace('-', "_")
        } else {
            // file name
            package_dir
                .as_ref()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_owned()
                .replace('-', "_")
        };

        let mut path = PathBuf::from_str(&get_cargo_target_directory(&cargo)).unwrap();
        path.push("wasm32-unknown-unknown");
        path.push("release");
        path.push(&wasm_name);
        path.set_extension("wasm");

        #[cfg(feature = "coverage")]
        // Check if binary exists in coverage directory, if it doesn't only then build it
        {
            let mut coverage_path = PathBuf::from_str(&get_cargo_target_directory(&cargo)).unwrap();
            coverage_path.pop();
            coverage_path.push("coverage");
            coverage_path.push("wasm32-unknown-unknown");
            coverage_path.push("release");
            coverage_path.push(wasm_name);
            coverage_path.set_extension("wasm");
            if coverage_path.is_file() {
                let code = fs::read(&coverage_path).unwrap_or_else(|err| {
                    panic!(
                        "Failed to read built WASM from path {:?} - {:?}",
                        &path, err
                    )
                });
                coverage_path.set_extension("rpd");
                let definition = fs::read(&coverage_path).unwrap_or_else(|err| {
                    panic!(
                        "Failed to read package definition from path {:?} - {:?}",
                        &coverage_path, err
                    )
                });
                let definition = manifest_decode(&definition).unwrap_or_else(|err| {
                    panic!(
                        "Failed to parse package definition from path {:?} - {:?}",
                        &coverage_path, err
                    )
                });
                return (code, definition);
            }
        };

        // Build
        let features = vec![
            "scrypto/log-error",
            "scrypto/log-warn",
            "scrypto/log-info",
            "scrypto/log-debug",
            "scrypto/log-trace",
        ];

        let status = Command::new("cargo")
            .envs(env_vars)
            .current_dir(package_dir.as_ref())
            .args([
                "build",
                "--target",
                "wasm32-unknown-unknown",
                "--release",
                "--features",
                &features.join(","),
            ])
            .status()
            .unwrap_or_else(|error| {
                panic!(
                    "Compiling \"{:?}\" failed with \"{:?}\"",
                    package_dir.as_ref(),
                    error
                )
            });
        if !status.success() {
            panic!("Failed to compile package: {:?}", package_dir.as_ref());
        }

        // Extract schema
        let code = fs::read(&path).unwrap_or_else(|err| {
            panic!(
                "Failed to read built WASM from path {:?} - {:?}",
                &path, err
            )
        });
        let definition = extract_definition(&code).unwrap();

        (code, definition)
    }

    // Naive pattern matching to find the crate name.
    fn extract_crate_name(mut content: &str) -> Result<String, ()> {
        let idx = content.find("name").ok_or(())?;
        content = &content[idx + 4..];

        let idx = content.find('"').ok_or(())?;
        content = &content[idx + 1..];

        let end = content.find('"').ok_or(())?;
        Ok(content[..end].to_string())
    }
}

/// Gets the default cargo directory for the given crate.
/// This respects whether the crate is in a workspace.
pub fn get_cargo_target_directory(manifest_path: impl AsRef<OsStr>) -> String {
    let output = Command::new("cargo")
        .arg("metadata")
        .arg("--manifest-path")
        .arg(manifest_path.as_ref())
        .arg("--format-version")
        .arg("1")
        .arg("--no-deps")
        .output()
        .expect("Failed to call cargo metadata");
    if output.status.success() {
        let parsed = serde_json::from_slice::<serde_json::Value>(&output.stdout)
            .expect("Failed to parse cargo metadata");
        let target_directory = parsed
            .as_object()
            .and_then(|o| o.get("target_directory"))
            .and_then(|o| o.as_str())
            .expect("Failed to parse target_directory from cargo metadata");
        target_directory.to_owned()
    } else {
        panic!("Cargo metadata call was not successful");
    }
}
