use scrypto::prelude::*;

#[blueprint]
mod metadata_component {
    struct MetadataComponent {}

    impl MetadataComponent {
        pub fn new() -> ComponentAddress {
            let mut component = MetadataComponent {}.instantiate();
            component.metadata("key", "value");
            component.globalize()
        }
    }
}
