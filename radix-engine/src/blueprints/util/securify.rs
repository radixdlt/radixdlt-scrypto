use crate::errors::RuntimeError;
use crate::types::*;
use native_sdk::modules::access_rules::AccessRulesObject;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::resource::*;

pub enum AccessRuleState {
    Advanced(AccessRule, AccessRule),
    PreSecurifiedSingleOwner(NonFungibleGlobalId),
    SecurifiedSingleOwner(NonFungibleLocalId),
}

pub trait SecurifiedAccessRules {
    const OWNER_GROUP_NAME: &'static str;
    const SECURIFY_IDENT: &'static str;
    const PACKAGE: PackageAddress;
    const OWNER_TOKEN: ResourceAddress;

    fn update_access_rules<A: AccessRulesObject, Y: ClientApi<RuntimeError>>(
        access_rules: &A,
        to_state: AccessRuleState,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        match to_state {
            AccessRuleState::Advanced(access_rule, mutability) => {
                access_rules.set_method_access_rule_and_mutability(
                    MethodKey::new(
                        NodeModuleId::SELF,
                        Self::SECURIFY_IDENT,
                    ),
                    AccessRuleEntry::AccessRule(AccessRule::DenyAll),
                    AccessRule::DenyAll,
                    api,
                )?;
                access_rules.set_group_access_rule_and_mutability(
                    Self::OWNER_GROUP_NAME,
                    access_rule,
                    mutability,
                    api,
                )?;
            }
            AccessRuleState::PreSecurifiedSingleOwner(owner_id) => {
                let package_id = NonFungibleGlobalId::new(
                    PACKAGE_TOKEN,
                    NonFungibleLocalId::bytes(scrypto_encode(&Self::PACKAGE).unwrap()).unwrap(),
                );
                let this_package_rule = rule!(require(package_id));

                let access_rule = rule!(require(owner_id));
                access_rules.set_method_access_rule_and_mutability(
                    MethodKey::new(
                        NodeModuleId::SELF,
                        Self::SECURIFY_IDENT,
                    ),
                    AccessRuleEntry::AccessRule(access_rule.clone()),
                    this_package_rule.clone(),
                    api,
                )?;
                access_rules.set_group_access_rule_and_mutability(
                    Self::OWNER_GROUP_NAME,
                    access_rule,
                    this_package_rule,
                    api,
                )?;
            }
            AccessRuleState::SecurifiedSingleOwner(owner_local_id) => {
                access_rules.set_method_access_rule_and_mutability(
                    MethodKey::new(
                        NodeModuleId::SELF,
                        Self::SECURIFY_IDENT,
                    ),
                    AccessRuleEntry::AccessRule(AccessRule::DenyAll),
                    AccessRule::DenyAll,
                    api,
                )?;
                let global_id = NonFungibleGlobalId::new(Self::OWNER_TOKEN, owner_local_id);
                access_rules.set_group_access_rule_and_mutability(
                    Self::OWNER_GROUP_NAME,
                    rule!(require(global_id)),
                    AccessRule::DenyAll,
                    api,
                )?;
            }
        }

        Ok(())
    }
}

