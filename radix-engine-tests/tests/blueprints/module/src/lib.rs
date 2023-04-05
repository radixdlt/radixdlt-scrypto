use scrypto::api::node_modules::auth::*;
use scrypto::api::node_modules::metadata::*;
use scrypto::api::node_modules::royalty::*;
use scrypto::api::*;
use scrypto::engine::scrypto_env::*;
use scrypto::prelude::*;

#[blueprint]
mod component_module {
    use crate::{AccessRules, RoyaltyConfig};

    struct ComponentModule {}

    impl ComponentModule {
        pub fn globalize_with_mixed_up_modules() -> ComponentAddress {
            let component = ComponentModule {}.instantiate();

            let rtn = ScryptoEnv
                .call_function(
                    METADATA_PACKAGE,
                    METADATA_BLUEPRINT,
                    METADATA_CREATE_IDENT,
                    scrypto_encode(&MetadataCreateInput {}).unwrap(),
                )
                .unwrap();
            let metadata: Own = scrypto_decode(&rtn).unwrap();

            let rtn = ScryptoEnv
                .call_function(
                    ROYALTY_PACKAGE,
                    COMPONENT_ROYALTY_BLUEPRINT,
                    COMPONENT_ROYALTY_CREATE_IDENT,
                    scrypto_encode(&ComponentRoyaltyCreateInput {
                        royalty_config: RoyaltyConfig::default(),
                    })
                    .unwrap(),
                )
                .unwrap();
            let royalty: Own = scrypto_decode(&rtn).unwrap();

            let rtn = ScryptoEnv
                .call_function(
                    ACCESS_RULES_PACKAGE,
                    ACCESS_RULES_BLUEPRINT,
                    ACCESS_RULES_CREATE_IDENT,
                    scrypto_encode(&AccessRulesCreateInput {
                        access_rules: AccessRulesConfig::new(),
                        child_blueprint_rules: BTreeMap::new(),
                    })
                    .unwrap(),
                )
                .unwrap();
            let access_rules: Own = scrypto_decode(&rtn).unwrap();

            let address = ScryptoEnv
                .globalize(
                    *component.component.0.as_node_id(),
                    btreemap!(
                        TypedModuleId::AccessRules => metadata.0,
                        TypedModuleId::Metadata => royalty.0,
                        TypedModuleId::Royalty => access_rules.0,
                    ),
                )
                .unwrap();

            ComponentAddress::new_unchecked(address.into())
        }
    }
}
