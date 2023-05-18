use scrypto::prelude::*;

#[blueprint]
mod metadata {
    struct MetadataTest {}

    impl MetadataTest {
        pub fn new() -> Global<MetadataTest> {
            let mut authority_rules = AuthorityRules::new();
            authority_rules.set_metadata_authority(AccessRule::AllowAll, AccessRule::DenyAll);

            Self {}
                .instantiate()
                .authority_rules(authority_rules)
                .globalize()
        }

        pub fn set_string(&self, key: String, value: String) {
            let global: Global<MetadataTest> = Runtime::global_address().into();
            let metadata = global.metadata();
            metadata.set(key, value);
        }

        pub fn set_address(&self, key: String, value: GlobalAddress) {
            let global: Global<MetadataTest> = Runtime::global_address().into();
            let metadata = global.metadata();
            metadata.set(key, value);
        }

        pub fn set_array(&self, key: String, value: Vec<GlobalAddress>) {
            let global: Global<MetadataTest> = Runtime::global_address().into();
            let metadata = global.metadata();
            metadata.set(key, value);
        }

        pub fn get_string(&self, key: String) -> String {
            let global: Global<MetadataTest> = Runtime::global_address().into();
            let metadata = global.metadata();
            metadata.get(key).unwrap()
        }

        pub fn get_address(&self, key: String) -> GlobalAddress {
            let global: Global<MetadataTest> = Runtime::global_address().into();
            let metadata = global.metadata();
            metadata.get(key).unwrap()
        }

        pub fn get_array(&self, key: String) -> Vec<GlobalAddress> {
            let global: Global<MetadataTest> = Runtime::global_address().into();
            let metadata = global.metadata();
            metadata.get(key).unwrap()
        }
    }
}
