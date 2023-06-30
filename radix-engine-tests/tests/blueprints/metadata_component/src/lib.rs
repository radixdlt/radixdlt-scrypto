use scrypto::prelude::*;

#[blueprint]
mod metadata_component {
    struct MetadataComponent {}

    impl MetadataComponent {
        pub fn new(key: String, value: String) {
            let global = Self {}
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .metadata(metadata! {
                    roles {
                        metadata_setter => rule!(allow_all), locked;
                        metadata_setter_updater => rule!(deny_all), locked;
                        metadata_locker => rule!(allow_all), locked;
                        metadata_locker_updater => rule!(deny_all), locked;
                    },
                    init {
                        key.clone() => value.clone(), locked;
                    }
                })
                .globalize();

            let value0: String = global.get_metadata(key).unwrap();
            assert_eq!(value0, value);
        }

        pub fn new2(key: String, value: String) {
            let global = MetadataComponent {}
                .instantiate()
                .prepare_to_globalize(OwnerRole::Fixed(rule!(allow_all)))
                .globalize();

            global.set_metadata(key.clone(), value.clone());
            let value0: String = global.get_metadata(key).unwrap();

            assert_eq!(value0, value);
        }

        pub fn remove_metadata(global: Global<MetadataComponent>, key: String) {
            global.remove_metadata(key);
        }
    }
}
