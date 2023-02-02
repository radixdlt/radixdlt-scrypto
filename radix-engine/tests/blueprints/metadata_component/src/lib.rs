use scrypto::prelude::*;

blueprint! {
    struct MetadataComponent {}

    impl MetadataComponent {
        pub fn new() -> ComponentAddress {
            let component = MetadataComponent {}.instantiate();
            component.set_metadata("key", "value");
            component.globalize()
        }
    }
}
