use scrypto::prelude::*;

#[derive(Sbor, Clone)]
pub struct ExtraStruct {
    field_one: String,
}

#[derive(Sbor, Clone)]
pub enum ExtraEnum {
    EntryOne,
    EntryTwo,
}

#[blueprint]
mod external_blueprint_target {
    struct ExternalBlueprintTarget {
        some_field: ExtraStruct,
    }

    impl ExternalBlueprintTarget {
        pub fn create() -> ComponentAddress {
            Self {
                some_field: ExtraStruct {
                    field_one: String::from("test_1"),
                },
            }
            .instantiate()
            .globalize()
        }

        pub fn get_value_via_package_call() -> String {
            String::from("SUCCESS")
        }

        pub fn get_value_via_ref(&self) -> ExtraStruct {
            self.some_field.clone()
        }

        pub fn get_value_via_mut_ref(&mut self) -> ExtraEnum {
            ExtraEnum::EntryOne
        }
    }
}
