use scrypto::prelude::*;

#[blueprint]
mod metadata {
    struct MetadataTest {}

    impl MetadataTest {
        pub fn new() -> Global<MetadataTest> {
            let (address_reservation, _) =
                Runtime::allocate_component_address(MetadataTest::blueprint_id());

            Self::new_with_address(address_reservation)
        }

        pub fn new_with_address(
            address_reservation: GlobalAddressReservation,
        ) -> Global<MetadataTest> {
            Self {}
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .metadata(metadata! {
                    roles {
                        metadata_setter => rule!(allow_all);
                        metadata_setter_updater => rule!(deny_all);
                        metadata_locker => rule!(allow_all);
                        metadata_locker_updater => rule!(deny_all);
                    },
                    init {
                        "empty_locked" => EMPTY, locked;
                    }
                })
                .with_address(address_reservation)
                .globalize()
        }

        pub fn new_with_initial_metadata(key: String, value: String) -> Global<MetadataTest> {
            Self {}
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .metadata(metadata! {
                    roles {
                        metadata_setter => rule!(allow_all);
                        metadata_setter_updater => rule!(deny_all);
                        metadata_locker => rule!(allow_all);
                        metadata_locker_updater => rule!(deny_all);
                    },
                    init {
                        key => value, locked;
                    }
                })
                .globalize()
        }

        pub fn set_string(&self, key: String, value: String) {
            let global: Global<MetadataTest> = Runtime::global_address().into();
            global.set_metadata(key, value);
        }

        pub fn set_address(&self, key: String, value: GlobalAddress) {
            let global: Global<MetadataTest> = Runtime::global_address().into();
            global.set_metadata(key, value);
        }

        pub fn set_array(&self, key: String, value: Vec<GlobalAddress>) {
            let global: Global<MetadataTest> = Runtime::global_address().into();
            global.set_metadata(key, value);
        }

        pub fn get_string(&self, key: String) -> String {
            let global: Global<MetadataTest> = Runtime::global_address().into();
            global.get_metadata(key).unwrap().unwrap()
        }

        pub fn get_address(&self, key: String) -> GlobalAddress {
            let global: Global<MetadataTest> = Runtime::global_address().into();
            global.get_metadata(key).unwrap().unwrap()
        }

        pub fn get_array(&self, key: String) -> Vec<GlobalAddress> {
            let global: Global<MetadataTest> = Runtime::global_address().into();
            global.get_metadata(key).unwrap().unwrap()
        }
    }
}
