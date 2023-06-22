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

            fn publish_wasm_advanced(
                package_address: Option<GlobalAddressReservation>,
                code: Vec<u8>,
                setup: PackageDefinition,
                metadata: BTreeMap<String, MetadataValue>,
                owner_role: OwnerRole
            );
        }
    );

    struct PublishPackage {
    }

    impl PublishPackage {
        pub fn publish_package() {
            Blueprint::<FiFi>::publish_wasm(vec![], PackageDefinition::default(), btreemap!());
        }

        pub fn publish_package_advanced() {
            Blueprint::<FiFi>::publish_wasm_advanced(
                None,
                vec![],
                PackageDefinition::default(),
                btreemap!(),
                OwnerRole::None,
            );
        }

    }
}
