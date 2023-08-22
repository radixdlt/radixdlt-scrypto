use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;

use radix_engine::utils::extract_definition;

use crate::prelude::*;

pub struct Package;

impl Package {
    pub fn publish(
        code: Vec<u8>,
        package_definition: PackageDefinition,
        metadata: MetadataInit,
        api: &mut TestRuntime,
    ) -> Result<(PackageAddress, Bucket), RuntimeError> {
        api.with_auth_module_disabled(|api| {
            api.call_function_typed::<PackagePublishWasmInput, PackagePublishWasmOutput>(
                PACKAGE_PACKAGE,
                PACKAGE_BLUEPRINT,
                PACKAGE_PUBLISH_WASM_IDENT,
                &PackagePublishWasmInput {
                    code,
                    definition: package_definition,
                    metadata,
                },
            )
        })
    }

    pub fn publish_advanced(
        owner_role: OwnerRole,
        definition: PackageDefinition,
        code: Vec<u8>,
        metadata: MetadataInit,
        package_address: Option<GlobalAddressReservation>,
        api: &mut TestRuntime,
    ) -> Result<PackageAddress, RuntimeError> {
        api.with_auth_module_disabled(|api| {
            api.call_function_typed::<PackagePublishWasmAdvancedInput, PackagePublishWasmAdvancedOutput>(
                PACKAGE_PACKAGE,
                PACKAGE_BLUEPRINT,
                PACKAGE_PUBLISH_WASM_ADVANCED_IDENT,
                &PackagePublishWasmAdvancedInput {
                    owner_role,
                    definition,
                    code,
                    metadata,
                    package_address
                },
            )
        })
    }

    pub fn compile_and_publish<P>(
        path: P,
        api: &mut TestRuntime,
    ) -> Result<PackageAddress, RuntimeError>
    where
        P: AsRef<Path>,
    {
        let (wasm, package_definition) = Self::compile(path);
        Self::publish_advanced(
            OwnerRole::None,
            package_definition,
            wasm,
            Default::default(),
            Default::default(),
            api,
        )
    }

    pub fn compile<P>(path: P) -> (Vec<u8>, PackageDefinition)
    where
        P: AsRef<Path>,
    {
        Compile::compile(path)
    }
}

// TODO: Deduplicate

struct Compile;

impl Compile {
    pub fn compile<P: AsRef<Path>>(package_dir: P) -> (Vec<u8>, PackageDefinition) {
        // Build
        let status = Command::new("cargo")
            .current_dir(package_dir.as_ref())
            .args(["build", "--target", "wasm32-unknown-unknown", "--release"])
            .status()
            .unwrap();
        if !status.success() {
            panic!("Failed to compile package: {:?}", package_dir.as_ref());
        }

        // Find wasm path
        let mut cargo = package_dir.as_ref().to_owned();
        cargo.push("Cargo.toml");
        let wasm_name = if cargo.exists() {
            let content =
                std::fs::read_to_string(&cargo).expect("Failed to read the Cargo.toml file");
            Self::extract_crate_name(&content)
                .expect("Failed to extract crate name from the Cargo.toml file")
                .replace("-", "_")
        } else {
            // file name
            package_dir
                .as_ref()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_owned()
                .replace("-", "_")
        };
        let mut path = PathBuf::from_str(&Self::get_cargo_target_directory(&cargo)).unwrap(); // Infallible;
        path.push("wasm32-unknown-unknown");
        path.push("release");
        path.push(wasm_name);
        path.set_extension("wasm");

        // Extract schema
        let code = std::fs::read(&path).unwrap_or_else(|err| {
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

    fn get_cargo_target_directory(manifest_path: impl AsRef<OsStr>) -> String {
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
}
