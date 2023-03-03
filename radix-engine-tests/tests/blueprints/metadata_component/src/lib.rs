use scrypto::prelude::*;

#[blueprint]
mod metadata_component {
    use scrypto::prelude::*;

    struct MetadataComponent {}

    impl MetadataComponent {
        pub fn new(key: String, value: String) -> ComponentAddress {
            let component = MetadataComponent {}.instantiate();
            let metadata = Metadata::new();
            metadata.set(key, value);
            let component_address = component.globalize_with_metadata(metadata);
            component_address
        }

        pub fn new2(key: String, value: String) {
            let component = MetadataComponent {}.instantiate();
            let component_address = component.globalize_with_access_rules(
                AccessRules::new().default(AccessRule::AllowAll, AccessRule::DenyAll)
            );

            let global: MetadataComponentGlobalComponentRef = component_address.into();
            global.metadata().set(key, value);
        }
    }
}
