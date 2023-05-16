use radix_engine_interface::api::node_modules::auth::{
    AccessRulesSetMethodAccessRuleAndMutabilityInput, AccessRulesSetMethodAccessRuleInput,
    ACCESS_RULES_SET_METHOD_ACCESS_RULE_AND_MUTABILITY_IDENT,
    ACCESS_RULES_SET_METHOD_ACCESS_RULE_IDENT,
};
use radix_engine_interface::api::node_modules::metadata::{
    MetadataSetInput, MetadataVal, METADATA_SET_IDENT,
};
use radix_engine_interface::api::object_api::ObjectModuleId;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::resource::{AccessRuleEntry, MethodKey, ObjectKey};
use radix_engine_interface::data::scrypto::{scrypto_decode, scrypto_encode, ScryptoDecode};
use radix_engine_interface::types::NodeId;
use sbor::rust::prelude::{Debug, ToOwned};

#[derive(Debug)]
pub struct BorrowedObject(pub NodeId);

impl BorrowedObject {
    pub fn new<T>(node_id: T) -> Self
    where
        T: Into<[u8; NodeId::LENGTH]>,
    {
        Self(NodeId(node_id.into()))
    }

    pub fn sys_set_metadata<Y, E, S, V>(&mut self, key: S, value: V, api: &mut Y) -> Result<(), E>
    where
        Y: ClientApi<E>,
        S: AsRef<str>,
        V: MetadataVal,
        E: Debug + ScryptoDecode,
    {
        api.call_module_method(
            &self.0,
            ObjectModuleId::Metadata,
            METADATA_SET_IDENT,
            scrypto_encode(&MetadataSetInput {
                key: key.as_ref().to_owned(),
                value: scrypto_decode(&scrypto_encode(&value.to_metadata_entry()).unwrap())
                    .unwrap(),
            })
            .unwrap(),
        )?;

        Ok(())
    }

    pub fn sys_set_method_access_rule<Y, E>(
        &mut self,
        method_key: MethodKey,
        rule: AccessRuleEntry,
        api: &mut Y,
    ) -> Result<(), E>
    where
        Y: ClientApi<E>,
        E: Debug + ScryptoDecode,
    {
        api.call_module_method(
            &self.0,
            ObjectModuleId::AccessRules,
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

    pub fn sys_set_method_access_rule_and_mutability<Y, E>(
        &mut self,
        method_key: MethodKey,
        rule: AccessRuleEntry,
        mutability: AccessRuleEntry,
        api: &mut Y,
    ) -> Result<(), E>
    where
        Y: ClientApi<E>,
        E: Debug + ScryptoDecode,
    {
        api.call_module_method(
            &self.0,
            ObjectModuleId::AccessRules,
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
}
