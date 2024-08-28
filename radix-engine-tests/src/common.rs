#[cfg(feature = "compile-blueprints-at-build-time")]
#[allow(unused)]
pub mod package_loader {
    use radix_common::prelude::*;
    use radix_substate_store_queries::typed_substate_layout::*;
    use std::path::PathBuf;

    const PACKAGES_BINARY: &[u8] =
        include_bytes!(concat!(env!("OUT_DIR"), "/compiled_packages.bin"));

    lazy_static::lazy_static! {
        static ref PACKAGES: HashMap<String, (Vec<u8>, PackageDefinition)> = {
            scrypto_decode(PACKAGES_BINARY).unwrap()
        };
    }

    pub struct PackageLoader;
    impl PackageLoader {
        pub fn get(name: &str) -> (Vec<u8>, PackageDefinition) {
            // Extract package file name if specified package name contains also a path.
            let file_name = PathBuf::from(name)
                .file_name()
                .unwrap()
                .to_os_string()
                .into_string()
                .unwrap();
            if let Some(rtn) = PACKAGES.get(&file_name) {
                rtn.clone()
            } else {
                panic!("Package \"{}\" not found. Are you sure that this package is: a) in the blueprints folder, b) that this is the same as the package name in the Cargo.toml file?", name)
            }
        }
    }
}

#[cfg(not(feature = "compile-blueprints-at-build-time"))]
#[allow(unused)]
pub mod package_loader {
    use radix_common::prelude::*;
    use radix_substate_store_queries::typed_substate_layout::*;
    use scrypto_test::ledger_simulator::CompileProfile;
    use std::path::PathBuf;

    pub struct PackageLoader;
    impl PackageLoader {
        pub fn get(name: &str) -> (Vec<u8>, PackageDefinition) {
            let manifest_dir = PathBuf::from_str(env!("CARGO_MANIFEST_DIR")).unwrap();
            let package_dir = manifest_dir.join("assets").join("blueprints").join(name);
            scrypto_test::prelude::Compile::compile(package_dir, CompileProfile::FastWithTraceLogs)
        }
    }
}

pub use package_loader::PackageLoader;

/// Defines globally for all tests paths for various assets used during the tests and benches.
/// To use it in a test definition file include following statement:
/// use radix_engine_tests::common::*;
pub mod path_macros {

    #[macro_export]
    macro_rules! include_workspace_asset_bytes {
        ($package: expr, $name: expr) => {
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../",
                $package,
                "/assets/",
                $name
            ))
        };
    }

    #[macro_export]
    macro_rules! include_workspace_transaction_examples_str {
        ($name: expr) => {
            include_str!(path_workspace_transaction_examples!($name))
        };
    }

    #[macro_export]
    macro_rules! path_workspace_transaction_examples {
        ($name: expr) => {
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../radix-transactions/examples/",
                $name
            )
        };
    }

    #[macro_export]
    macro_rules! include_local_wasm_str {
        ($name: expr) => {
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/wasm/", $name))
        };
    }

    #[macro_export]
    macro_rules! path_local_blueprint {
        ($name: expr) => {
            concat!(env!("CARGO_MANIFEST_DIR"), "/assets/blueprints/", $name)
        };
    }

    #[macro_export]
    macro_rules! path_local_metering_receipts {
        () => {
            concat!(env!("CARGO_MANIFEST_DIR"), "/assets/metering/")
        };
    }

    pub use crate::include_local_wasm_str;
    pub use crate::include_workspace_asset_bytes;
    pub use crate::include_workspace_transaction_examples_str;
    pub use crate::path_local_blueprint;
    pub use crate::path_workspace_transaction_examples;
}

pub use path_macros::*;
