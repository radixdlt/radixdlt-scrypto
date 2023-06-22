use scrypto::prelude::*;
use scrypto::blueprints::package::PackageDefinition;

#[blueprint]
mod publish_package {
    extern_blueprint!(
        "package_rdx1pkgxxxxxxxxxpackgexxxxxxxxx000726633226xxxxxxxxxpackge",
        Package as FiFi {
            fn publish_wasm(
                code: Vec<u8>,
                setup: PackageDefinition,
                metadata: BTreeMap<String, MetadataValue>
            );
        }
    );

    struct PublishPackage {
    }

    impl PublishPackage {
        pub fn publish_package() {
            Blueprint::<FiFi>::publish_wasm(vec![], PackageDefinition::default(), btreemap!());
        }
    }
}
