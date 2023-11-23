#[cfg(feature = "compile-blueprints-at-build-time")]
#[allow(unused)]
mod package_loader {
    use radix_engine_common::prelude::*;
    use radix_engine_queries::typed_substate_layout::*;

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
            if let Some(rtn) = PACKAGES.get(name) {
                rtn.clone()
            } else {
                panic!("Package \"{}\" not found. Are you sure that this package is: a) in the blueprints folder, b) that this is the same as the package name in the Cargo.toml file?", name)
            }
        }
    }
}

#[cfg(not(feature = "compile-blueprints-at-build-time"))]
#[allow(unused)]
mod package_loader {
    use radix_engine_common::prelude::*;
    use radix_engine_queries::typed_substate_layout::*;
    use std::path::PathBuf;

    pub struct PackageLoader;
    impl PackageLoader {
        pub fn get(name: &str) -> (Vec<u8>, PackageDefinition) {
            let manifest_dir = PathBuf::from_str(env!("CARGO_MANIFEST_DIR")).unwrap();
            let package_dir = manifest_dir.join("assets").join("blueprints").join(name);
            scrypto_unit::Compile::compile(package_dir)
        }
    }
}

pub use package_loader::PackageLoader;
