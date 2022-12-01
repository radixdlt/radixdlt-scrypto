use scrypto::prelude::*;

blueprint! {
    struct MetadataComponent {}

    impl MetadataComponent {
        pub fn new() -> ComponentAddress {
            let mut component = MetadataComponent {}.instantiate();
            component.metadata("key", "value");
            component.globalize_no_owner()
        }
    }
}
