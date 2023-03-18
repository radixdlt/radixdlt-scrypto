use radix_engine_interface::api::node_modules::auth::{AccessRulesCreateInput, ACCESS_RULES_BLUEPRINT, ACCESS_RULES_CREATE_IDENT, ACCESS_RULES_SET_GROUP_ACCESS_RULE_AND_MUTABILITY_IDENT, AccessRulesSetGroupAccessRuleAndMutabilityInput, ACCESS_RULES_SET_METHOD_ACCESS_RULE_AND_MUTABILITY_IDENT, AccessRulesSetMethodAccessRuleAndMutabilityInput};
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::api::types::{NodeModuleId, RENodeId};
use radix_engine_interface::blueprints::resource::{AccessRule, AccessRuleEntry, AccessRulesConfig, MethodKey};
use radix_engine_interface::constants::ACCESS_RULES_PACKAGE;
use radix_engine_interface::data::scrypto::model::Own;
use radix_engine_interface::data::scrypto::*;
use sbor::rust::fmt::Debug;

pub struct AccessRules(pub Own);

impl AccessRules {
    pub fn sys_new<Y, E: Debug + ScryptoDecode>(
        access_rules: AccessRulesConfig,
        api: &mut Y,
    ) -> Result<Self, E>
    where
        Y: ClientApi<E>,
    {
        let rtn = api.call_function(
            ACCESS_RULES_PACKAGE,
            ACCESS_RULES_BLUEPRINT,
            ACCESS_RULES_CREATE_IDENT,
            scrypto_encode(&AccessRulesCreateInput { access_rules }).unwrap(),
        )?;

        let access_rules: Own = scrypto_decode(&rtn).unwrap();

        Ok(Self(access_rules))
    }
}

impl AccessRulesObject for AccessRules {
    fn self_id(&self) -> (RENodeId, NodeModuleId) {
        (RENodeId::Object(self.0.id()), NodeModuleId::SELF)
    }
}


pub struct AttachedAccessRules(pub RENodeId);

impl AccessRulesObject for AttachedAccessRules {
    fn self_id(&self) -> (RENodeId, NodeModuleId) {
        (self.0, NodeModuleId::AccessRules)
    }
}

pub trait AccessRulesObject {
    fn self_id(&self) -> (RENodeId, NodeModuleId);

    fn set_method_access_rule_and_mutability<Y: ClientApi<E>, E: Debug + ScryptoDecode>(
        &self,
        key: MethodKey,
        rule: AccessRuleEntry,
        mutability: AccessRule,
        api: &mut Y,
    ) -> Result<(), E> {
        let (node_id, module_id) = self.self_id();
        let _rtn = api.call_module_method(
            node_id,
            module_id,
            ACCESS_RULES_SET_METHOD_ACCESS_RULE_AND_MUTABILITY_IDENT,
            scrypto_encode(&AccessRulesSetMethodAccessRuleAndMutabilityInput {
                key,
                rule,
                mutability,
            })
                .unwrap(),
        )?;

        Ok(())
    }

    fn set_group_access_rule_and_mutability<Y: ClientApi<E>, E: Debug + ScryptoDecode>(
        &self,
        name: &str,
        rule: AccessRule,
        mutability: AccessRule,
        api: &mut Y,
    ) -> Result<(), E> {
        let (node_id, module_id) = self.self_id();
        let _rtn = api.call_module_method(
            node_id,
            module_id,
            ACCESS_RULES_SET_GROUP_ACCESS_RULE_AND_MUTABILITY_IDENT,
            scrypto_encode(&AccessRulesSetGroupAccessRuleAndMutabilityInput {
                name: name.to_string(),
                rule,
                mutability,
            })
                .unwrap(),
        )?;

        Ok(())
    }
}