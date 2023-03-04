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
    struct ExternalBlueprintCaller {}

    impl ExternalBlueprintCaller {
        pub fn create() -> ComponentAddress {
            Self {}.instantiate().globalize()
        }

        pub fn run_tests_with_external_blueprint(&self, package_address: PackageAddress) {
            let external_blueprint =
                ExternalBlueprintTarget::at(package_address, "ExternalBlueprintTarget");

            // NB - These values should match those defined in ../../component/src/external_blueprint_target.rs
            assert!(
                external_blueprint.get_value_via_package_call() == "SUCCESS",
                "Package call failed"
            );

            let component_address = external_blueprint.create();
            let mut target = ExternalComponentTarget::at(component_address);

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
            let mut target = ExternalComponentTarget::from(component_address);

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
