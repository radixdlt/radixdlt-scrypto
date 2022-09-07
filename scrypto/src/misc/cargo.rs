use scrypto::abi::BlueprintAbi;
use scrypto::buffer::scrypto_decode;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

// Pattern matching to find the crate name. Not a sound solution but saves from
// heavy serialization frameworks.
fn extract_crate_name(mut content: &str) -> Result<String, ()> {
    let idx = content.find("name").ok_or(())?;
    content = &content[idx + 4..];

    let idx = content.find('"').ok_or(())?;
    content = &content[idx + 1..];

    let end = content.find('"').ok_or(())?;
    Ok(content[..end].to_string())
}

/// Compiles a Scrypto package.
///
/// For testing purpose only.
pub fn compile_package<P: AsRef<Path>>(package_dir: P) -> (Vec<u8>, HashMap<String, BlueprintAbi>) {
    // build
    let status = Command::new("scrypto")
        .current_dir(package_dir.as_ref())
        .args(["build"])
        .status()
        .unwrap();
    if !status.success() {
        panic!("Failed to build package: {:?}", package_dir.as_ref());
    }

    // resolve wasm name
    let mut cargo = package_dir.as_ref().to_owned();
    cargo.push("Cargo.toml");
    let bin_name = if cargo.exists() {
        let content = fs::read_to_string(cargo).expect("Failed to read the Cargo.toml file");
        extract_crate_name(&content)
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

    // path of the wasm executable
    let mut path = PathBuf::from(package_dir.as_ref());
    path.push("target");
    path.push("wasm32-unknown-unknown");
    path.push("release");
    path.push(bin_name);
    let code_path = path.with_extension("wasm");
    let abi_path = path.with_extension("abi");

    // extract ABI
    (
        fs::read(code_path).unwrap(),
        scrypto_decode(&fs::read(abi_path).unwrap()).unwrap(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_crate_name() {
        assert_eq!(
            "hello-world",
            extract_crate_name(
                r#"
                [package]
                name = "hello-world"
                version = "0.1.0"
                edition = "2021"
                "#
            )
            .unwrap()
        )
    }
}
