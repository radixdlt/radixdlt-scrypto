use scrypto::prelude::*;

#[blueprint]
mod blueprint {
    struct MetadataComponent {}

    impl MetadataComponent {
        pub fn new() -> ComponentAddress {
            let mut component = MetadataComponent {}.instantiate();
            component.metadata("key", "value");
            component.globalize()
        }
    }
}
