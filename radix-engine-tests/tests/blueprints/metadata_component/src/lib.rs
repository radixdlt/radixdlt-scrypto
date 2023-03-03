use scrypto::prelude::*;

#[blueprint]
mod metadata_component {
    struct MetadataComponent {}

    impl MetadataComponent {
        pub fn new(key: String, value: String) -> ComponentAddress {
            let component = MetadataComponent {}.instantiate();
            let mut metadata = BTreeMap::new();
            metadata.insert(key, value);
            let component_address = component.globalize_with_metadata(metadata);
            component_address
        }
    }
}
