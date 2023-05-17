use scrypto::prelude::*;

#[blueprint]
mod metadata {
    struct MetadataCodeSize {}

    impl MetadataCodeSize {
        pub fn set_metadata(&self, key: String, value: String) {
            let global: MetadataCodeSizeGlobalComponentRef = Runtime::global_address().into();
            let metadata = global.metadata();
            metadata.set(key, value);
        }

        pub fn set_metadata_address(&self, key: String, value: GlobalAddress) {
            let global: MetadataCodeSizeGlobalComponentRef = Runtime::global_address().into();
            let metadata = global.metadata();
            metadata.set(key, value);
        }

        pub fn set_metadata_array(&self, key: String, value: Vec<GlobalAddress>) {
            let global: MetadataCodeSizeGlobalComponentRef = Runtime::global_address().into();
            let metadata = global.metadata();
            metadata.set(key, value);
        }

        pub fn get_metadata(&self, key: String) -> String {
            let global: MetadataCodeSizeGlobalComponentRef = Runtime::global_address().into();
            let metadata = global.metadata();
            metadata.get(key).unwrap()
        }

        pub fn get_metadata_address(&self, key: String) -> GlobalAddress {
            let global: MetadataCodeSizeGlobalComponentRef = Runtime::global_address().into();
            let metadata = global.metadata();
            metadata.get(key).unwrap()
        }

        pub fn get_metadata_array(&self, key: String) -> Vec<GlobalAddress> {
            let global: MetadataCodeSizeGlobalComponentRef = Runtime::global_address().into();
            let metadata = global.metadata();
            metadata.get(key).unwrap()
        }
    }
}
