use scrypto::prelude::*;

#[blueprint]
mod metadata_component {
    struct MetadataComponent {}

    impl MetadataComponent {
        pub fn new() -> ComponentAddress {
            let component = MetadataComponent {}.instantiate();
            let component_address = component.globalize();
            borrow_component!(component_address).set_metadata("key", "value");
            component_address
        }
    }
}
