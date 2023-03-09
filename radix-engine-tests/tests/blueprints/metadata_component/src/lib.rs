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
            let metadata = global.metadata();

            assert_eq!(metadata.get_string(key).unwrap(), value);
        }

        pub fn new2(key: String, value: String) {
            let component = MetadataComponent {}.instantiate();
            let component_address = component.globalize_with_access_rules(
                AccessRulesConfig::new().default(AccessRule::AllowAll, AccessRule::DenyAll),
            );

            let global: MetadataComponentGlobalComponentRef = component_address.into();
            let metadata = global.metadata();
            metadata.set(key.clone(), value.clone());

            assert_eq!(metadata.get_string(key).unwrap(), value);
        }

        pub fn remove_metadata(address: ComponentAddress, key: String) {
            let global = GlobalComponentRef(address);
            let metadata = global.metadata();
            metadata.remove(key);
        }
    }
}
