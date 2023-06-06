use scrypto::prelude::*;

#[derive(Sbor, PartialEq)]
pub struct ExtraStruct {
    field_one: String,
}

#[derive(Sbor, PartialEq)]
pub enum ExtraEnum {
    EntryOne,
    EntryTwo,
}

#[blueprint]
mod external_blueprint_caller {
    const TARGET_PACKAGE_ADDRESS: PackageAddress = PackageAddress::new_or_panic([
        13, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1,
    ]);

    extern_blueprint!(
        TARGET_PACKAGE_ADDRESS,
        ExternalBlueprintTarget {
            fn create() -> Global<ExternalBlueprintTarget>;
            fn get_value_via_package_call() -> String;
            fn get_value_via_ref(&self) -> ExtraStruct;
            fn get_value_via_mut_ref(&mut self) -> ExtraEnum;
        }
    );

    struct ExternalBlueprintCaller {}

    impl ExternalBlueprintCaller {
        pub fn create() -> Global<ExternalBlueprintCaller> {
            Self {}
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize()
        }

        pub fn run_tests_with_external_blueprint(&self) {
            // NB - These values should match those defined in ../../component/src/external_blueprint_target.rs
            assert!(
                Blueprint::<ExternalBlueprintTarget>::get_value_via_package_call() == "SUCCESS",
                "Package call failed"
            );

            let mut target: Global<ExternalBlueprintTarget> =
                Blueprint::<ExternalBlueprintTarget>::create();

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

        pub fn run_tests_with_external_component(
            &self,
            mut target: Global<ExternalBlueprintTarget>,
        ) {
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
