use radix_engine_interface::api::node_modules::auth::{
    AccessRulesCreateInput, AccessRulesSetGroupAccessRuleAndMutabilityInput,
    AccessRulesSetGroupAccessRuleInput, AccessRulesSetMethodAccessRuleAndMutabilityInput,
    AccessRulesSetMethodAccessRuleInput, ACCESS_RULES_BLUEPRINT, ACCESS_RULES_CREATE_IDENT,
    ACCESS_RULES_SET_GROUP_ACCESS_RULE_AND_MUTABILITY_IDENT,
    ACCESS_RULES_SET_GROUP_ACCESS_RULE_IDENT,
    ACCESS_RULES_SET_METHOD_ACCESS_RULE_AND_MUTABILITY_IDENT,
    ACCESS_RULES_SET_METHOD_ACCESS_RULE_IDENT,
};
use radix_engine_interface::api::object_api::ObjectModuleId;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::resource::{
    AccessRule, AccessRuleEntry, AccessRulesConfig, MethodKey, ObjectKey,
};
use radix_engine_interface::constants::ACCESS_RULES_MODULE_PACKAGE;
use radix_engine_interface::data::scrypto::model::Own;
use radix_engine_interface::data::scrypto::*;
use radix_engine_interface::types::NodeId;
use sbor::rust::collections::BTreeMap;
use sbor::rust::fmt::Debug;
use sbor::rust::prelude::*;
use sbor::rust::string::String;
use sbor::rust::string::ToString;

pub struct AccessRules(pub Own);

impl AccessRules {
    pub fn create<Y, E: Debug + ScryptoDecode>(
        access_rules: AccessRulesConfig,
        child_blueprint_rules: BTreeMap<String, AccessRulesConfig>,
        api: &mut Y,
    ) -> Result<Self, E>
    where
        Y: ClientApi<E>,
    {
        let rtn = api.call_function(
            ACCESS_RULES_MODULE_PACKAGE,
            ACCESS_RULES_BLUEPRINT,
            ACCESS_RULES_CREATE_IDENT,
            scrypto_encode(&AccessRulesCreateInput {
                access_rules,
                child_blueprint_rules,
            })
            .unwrap(),
        )?;

        let access_rules: Own = scrypto_decode(&rtn).unwrap();

        Ok(Self(access_rules))
    }
}

impl AccessRulesObject for AccessRules {
    fn self_id(&self) -> (&NodeId, ObjectModuleId) {
        (&self.0 .0, ObjectModuleId::Main)
    }
}

pub struct AttachedAccessRules(pub NodeId);

impl AccessRulesObject for AttachedAccessRules {
    fn self_id(&self) -> (&NodeId, ObjectModuleId) {
        (&self.0, ObjectModuleId::AccessRules)
    }
}

pub trait AccessRulesObject {
    fn self_id(&self) -> (&NodeId, ObjectModuleId);

    fn set_group_access_rule<Y: ClientApi<E>, E: Debug + ScryptoDecode>(
        &self,
        name: &str,
        rule: AccessRule,
        api: &mut Y,
    ) -> Result<(), E> {
        let (node_id, module_id) = self.self_id();
        let _rtn = api.call_module_method(
            node_id,
            module_id,
            ACCESS_RULES_SET_GROUP_ACCESS_RULE_IDENT,
            scrypto_encode(&AccessRulesSetGroupAccessRuleInput {
                object_key: ObjectKey::SELF,
                name: name.into(),
                rule,
            })
            .unwrap(),
        )?;

        Ok(())
    }

    fn set_method_access_rule<Y: ClientApi<E>, E: Debug + ScryptoDecode>(
        &self,
        method_key: MethodKey,
        rule: AccessRuleEntry,
        api: &mut Y,
    ) -> Result<(), E> {
        let (node_id, module_id) = self.self_id();
        let _rtn = api.call_module_method(
            node_id,
            module_id,
            ACCESS_RULES_SET_METHOD_ACCESS_RULE_IDENT,
            scrypto_encode(&AccessRulesSetMethodAccessRuleInput {
                object_key: ObjectKey::SELF,
                method_key,
                rule,
            })
            .unwrap(),
        )?;

        Ok(())
    }

    fn set_method_access_rule_and_mutability<Y: ClientApi<E>, E: Debug + ScryptoDecode>(
        &self,
        method_key: MethodKey,
        rule: AccessRuleEntry,
        mutability: AccessRuleEntry,
        api: &mut Y,
    ) -> Result<(), E> {
        let (node_id, module_id) = self.self_id();
        let _rtn = api.call_module_method(
            &node_id,
            module_id,
            ACCESS_RULES_SET_METHOD_ACCESS_RULE_AND_MUTABILITY_IDENT,
            scrypto_encode(&AccessRulesSetMethodAccessRuleAndMutabilityInput {
                object_key: ObjectKey::SELF,
                method_key,
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
            &node_id,
            module_id,
            ACCESS_RULES_SET_GROUP_ACCESS_RULE_AND_MUTABILITY_IDENT,
            scrypto_encode(&AccessRulesSetGroupAccessRuleAndMutabilityInput {
                object_key: ObjectKey::SELF,
                name: name.to_string(),
                rule,
                mutability,
            })
            .unwrap(),
        )?;

        Ok(())
    }
}
