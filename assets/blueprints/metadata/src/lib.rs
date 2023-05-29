use scrypto::prelude::*;

#[blueprint]
mod metadata {
    define_permissions! {
        main {
            set_string => PUBLIC;
            set_address => PUBLIC;
            set_array => PUBLIC;
            get_string => PUBLIC;
            get_address => PUBLIC;
            get_array => PUBLIC;
        },
        metadata {
            set => PUBLIC;
            remove => PUBLIC;
            get => PUBLIC;
        }
    }

    struct MetadataTest {}

    impl MetadataTest {
        pub fn new() -> Global<MetadataTest> {
            Self {}
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
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
