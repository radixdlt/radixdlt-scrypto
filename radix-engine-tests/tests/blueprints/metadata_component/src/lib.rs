use scrypto::prelude::*;

#[blueprint]
mod metadata_component {
    use scrypto::prelude::*;

    struct MetadataComponent {}

    impl MetadataComponent {
        pub fn new(key: String, value: String) {
            let global = Self {}
                .instantiate()
                .prepare_to_globalize()
                .metadata(key.clone(), value.clone())
                .globalize();

            let metadata = global.metadata();

            assert_eq!(metadata.get_string(key).unwrap(), value);
        }

        pub fn new2(key: String, value: String) {
            let global = MetadataComponent {}
                .instantiate()
                .prepare_to_globalize()
                .define_roles(roles! {
                    "metadata" => rule!(allow_all), rule!(deny_all),
                })
                .protect_metadata(protect!(
                    MetadataMethod::set => vec!["metadata"],
                ))
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
