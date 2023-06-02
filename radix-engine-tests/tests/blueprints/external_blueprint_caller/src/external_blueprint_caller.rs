use scrypto::prelude::*;

#[derive(Sbor, PartialEq)]
struct ExtraStruct {
    field_one: String,
}

#[derive(Sbor, PartialEq)]
enum ExtraEnum {
    EntryOne,
    EntryTwo,
}

external_blueprint! {
    ExternalBlueprintTarget {
        fn create() -> ComponentAddress;
        fn get_value_via_package_call() -> String;
    }
}

external_component! {
    ExternalComponentTarget {
        fn get_value_via_ref(&self) -> ExtraStruct;
        fn get_value_via_mut_ref(&mut self) -> ExtraEnum;
    }
}

#[blueprint]
mod external_blueprint_caller {
    const TARGET_PACKAGE_ADDRESS: PackageAddress = PackageAddress::new_or_panic([
        13, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1,
    ]);

    struct ExternalBlueprintCaller {}

    impl ExternalBlueprintCaller {
        pub fn create() -> Global<ExternalBlueprintCaller> {
            Self {}
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize()
        }

        pub fn run_tests_with_external_blueprint(&self) {
            let external_blueprint =
                ExternalBlueprintTarget::at(TARGET_PACKAGE_ADDRESS, "ExternalBlueprintTarget");

            // NB - These values should match those defined in ../../component/src/external_blueprint_target.rs
            assert!(
                external_blueprint.get_value_via_package_call() == "SUCCESS",
                "Package call failed"
            );

            let component_address = external_blueprint.create();
            let mut target: Global<ExternalComponentTarget> = component_address.into();

            assert!(
                target.get_value_via_ref()
                    == ExtraStruct {
                        field_one: String::from("test_1")
                    },
                "Ref call failed"
            );
            assert!(
                target.get_value_via_mut_ref() == ExtraEnum::EntryOne,
                "Mut Ref call failed"
            );
        }

        pub fn run_tests_with_external_component(&self, component_address: ComponentAddress) {
            // NB - These values should match those defined in ../../component/src/external_blueprint_target.rs
            let mut target: Global<ExternalComponentTarget> = component_address.into();

            assert!(
                target.get_value_via_ref()
                    == ExtraStruct {
                        field_one: String::from("test_1")
                    },
                "Ref call failed"
            );
            assert!(
                target.get_value_via_mut_ref() == ExtraEnum::EntryOne,
                "Mut Ref call failed"
            );
        }
    }
}
