use scrypto::prelude::*;

#[blueprint]
mod metadata_component {
    struct MetadataComponent {}

    impl MetadataComponent {
        pub fn new(key: String, value: String) {
            let component = Self {}.instantiate();

            let global = component
                .set_metadata(key.clone(), value.clone())
                .globalize();

            let metadata = global.metadata();

            assert_eq!(metadata.get_string(key).unwrap(), value);
        }

        pub fn new2(key: String, value: String) {
            let component = MetadataComponent {}.instantiate();

            let global = component
                .set_authority_rule("metadata", AccessRule::AllowAll, AccessRule::DenyAll)
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
