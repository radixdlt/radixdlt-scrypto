use scrypto::prelude::*;

#[blueprint]
mod metadata_component {
    struct MetadataComponent {}

    impl MetadataComponent {
        pub fn new(key: String, value: String) {
            let component = MetadataComponent {}.instantiate();
            let metadata = Metadata::new();
            metadata.set(key.clone(), value.clone());
            let component_address = component.globalize_with_metadata(metadata);
            let global: MetadataComponentGlobalComponentRef = component_address.into();

            assert_eq!(global.metadata().get(key), Some(value));
        }

        pub fn new2(key: String, value: String) {
            let component = MetadataComponent {}.instantiate();
            let component_address = component.globalize_with_access_rules(
                AccessRules::new().default(AccessRule::AllowAll, AccessRule::DenyAll)
            );

            let global: MetadataComponentGlobalComponentRef = component_address.into();
            global.metadata().set(key.clone(), value.clone());

            assert_eq!(global.metadata().get(key), Some(value));
        }

        pub fn remove_metadata(address: ComponentAddress, key: String) {
            let global = GlobalComponentRef(address);
            global.metadata().remove(key);
        }
    }
}
