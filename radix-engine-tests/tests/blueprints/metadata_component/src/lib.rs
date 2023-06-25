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
                        metadata_admin => rule!(allow_all), locked;
                        metadata_admin_updater => rule!(deny_all), locked;
                    },
                    init {
                        key.clone() => value.clone(), locked;
                    }
                })
                .globalize();

            let metadata = global.metadata();

            assert_eq!(metadata.get_string(key).unwrap(), value);
        }

        pub fn new2(key: String, value: String) {
            let global = MetadataComponent {}
                .instantiate()
                .prepare_to_globalize(OwnerRole::Fixed(rule!(allow_all)))
                .globalize();

            let metadata = global.metadata();
            metadata.set(key.clone(), value.clone());

            assert_eq!(metadata.get_string(key).unwrap(), value);
        }

        pub fn remove_metadata(global: Global<MetadataComponent>, key: String) {
            let metadata = global.metadata();
            metadata.remove(key);
        }
    }
}
