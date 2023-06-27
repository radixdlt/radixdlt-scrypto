use scrypto::api::node_modules::auth::*;
use scrypto::api::node_modules::royalty::*;
use scrypto::api::*;
use scrypto::engine::scrypto_env::*;
use scrypto::prelude::*;

#[blueprint]
mod component_module {
    use crate::ComponentRoyaltyConfig;
    use std::collections::BTreeMap;

    struct ComponentModule {}

    impl ComponentModule {
        pub fn globalize_with_mixed_up_modules() -> ComponentAddress {
            let component = ComponentModule {}.instantiate();

            let rtn = ScryptoEnv
                .call_function(
                    METADATA_MODULE_PACKAGE,
                    METADATA_BLUEPRINT,
                    METADATA_CREATE_IDENT,
                    scrypto_encode(&MetadataCreateInput {}).unwrap(),
                )
                .unwrap();
            let metadata: Own = scrypto_decode(&rtn).unwrap();

            let rtn = ScryptoEnv
                .call_function(
                    ROYALTY_MODULE_PACKAGE,
                    COMPONENT_ROYALTY_BLUEPRINT,
                    COMPONENT_ROYALTY_CREATE_IDENT,
                    scrypto_encode(&ComponentRoyaltyCreateInput {
                        royalty_config: ComponentRoyaltyConfig::default(),
                    })
                    .unwrap(),
                )
                .unwrap();
            let royalty: Own = scrypto_decode(&rtn).unwrap();

            let rtn = ScryptoEnv
                .call_function(
                    ACCESS_RULES_MODULE_PACKAGE,
                    ACCESS_RULES_BLUEPRINT,
                    ACCESS_RULES_CREATE_IDENT,
                    scrypto_encode(&AccessRulesCreateInput {
                        owner_role: OwnerRole::None,
                        roles: BTreeMap::new(),
                    })
                    .unwrap(),
                )
                .unwrap();
            let access_rules: Own = scrypto_decode(&rtn).unwrap();

            let address = ScryptoEnv
                .globalize(
                    btreemap!(
                        ObjectModuleId::Main => *component.0.handle().as_node_id(),
                        ObjectModuleId::AccessRules => metadata.0,
                        ObjectModuleId::Metadata => royalty.0,
                        ObjectModuleId::Royalty => access_rules.0,
                    ),
                    None,
                )
                .unwrap();

            ComponentAddress::new_or_panic(address.into())
        }
    }
}
