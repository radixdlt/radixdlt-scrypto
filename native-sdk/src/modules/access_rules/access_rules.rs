use radix_engine_interface::api::node_modules::auth::{
    AccessRulesCreateInput, AccessRulesUpdateMethod, AccessRulesUpdateRoleInput,
    ACCESS_RULES_BLUEPRINT, ACCESS_RULES_CREATE_IDENT, ACCESS_RULES_UPDATE_METHOD_IDENT,
    ACCESS_RULES_UPDATE_ROLE_IDENT,
};
use radix_engine_interface::api::object_api::ObjectModuleId;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::resource::{AccessRule, MethodEntry, MethodKey, MethodPermission, ObjectKey, RoleKey, RoleList, Roles};
use radix_engine_interface::constants::ACCESS_RULES_MODULE_PACKAGE;
use radix_engine_interface::data::scrypto::model::Own;
use radix_engine_interface::data::scrypto::*;
use radix_engine_interface::types::NodeId;
use sbor::rust::collections::BTreeMap;
use sbor::rust::fmt::Debug;
use sbor::rust::prelude::*;
use sbor::rust::string::String;

pub struct AccessRules(pub Own);

impl AccessRules {
    pub fn create<Y, E: Debug + ScryptoDecode>(
        method_permissions: BTreeMap<MethodKey, MethodEntry>,
        roles: Roles,
        inner_blueprint_rules: BTreeMap<String, BTreeMap<MethodKey, MethodEntry>>,
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
                method_permissions,
                roles,
                inner_blueprint_rules,
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

    fn update_role<
        Y: ClientApi<E>,
        E: Debug + ScryptoDecode,
        R: Into<RoleKey>,
        A: Into<AccessRule>,
        L: Into<RoleList>,
    >(
        &self,
        role_key: R,
        rule: A,
        mutability: L,
        api: &mut Y,
    ) -> Result<(), E> {
        let (node_id, module_id) = self.self_id();
        let _rtn = api.call_method_advanced(
            node_id,
            false,
            module_id,
            ACCESS_RULES_UPDATE_ROLE_IDENT,
            scrypto_encode(&AccessRulesUpdateRoleInput {
                role_key: role_key.into(),
                rule: Some(rule.into()),
                mutability: Some(mutability.into()),
            })
            .unwrap(),
        )?;

        Ok(())
    }

    fn update_role_rules<
        Y: ClientApi<E>,
        E: Debug + ScryptoDecode,
        R: Into<RoleKey>,
        A: Into<AccessRule>,
    >(
        &self,
        role_key: R,
        entry: A,
        api: &mut Y,
    ) -> Result<(), E> {
        let (node_id, module_id) = self.self_id();
        let _rtn = api.call_method_advanced(
            node_id,
            false,
            module_id,
            ACCESS_RULES_UPDATE_ROLE_IDENT,
            scrypto_encode(&AccessRulesUpdateRoleInput {
                role_key: role_key.into(),
                rule: Some(entry.into()),
                mutability: None,
            })
            .unwrap(),
        )?;

        Ok(())
    }

    fn update_role_mutability<
        Y: ClientApi<E>,
        E: Debug + ScryptoDecode,
        R: Into<RoleKey>,
        A: Into<RoleList>,
    >(
        &self,
        role_key: R,
        mutability: A,
        api: &mut Y,
    ) -> Result<(), E> {
        let (node_id, module_id) = self.self_id();
        let _rtn = api.call_method_advanced(
            node_id,
            false,
            module_id,
            ACCESS_RULES_UPDATE_ROLE_IDENT,
            scrypto_encode(&AccessRulesUpdateRoleInput {
                role_key: role_key.into(),
                mutability: Some(mutability.into()),
                rule: None,
            })
            .unwrap(),
        )?;

        Ok(())
    }

    fn update_method<
        Y: ClientApi<E>,
        E: Debug + ScryptoDecode,
        P: Into<MethodPermission>,
        L: Into<RoleList>,
    >(
        &self,
        method_key: MethodKey,
        permission: P,
        mutability: L,
        api: &mut Y,
    ) -> Result<(), E> {
        let (node_id, module_id) = self.self_id();
        let _rtn = api.call_method_advanced(
            &node_id,
            false,
            module_id,
            ACCESS_RULES_UPDATE_METHOD_IDENT,
            scrypto_encode(&AccessRulesUpdateMethod {
                object_key: ObjectKey::SELF,
                method_key,
                permission: Some(permission.into()),
                mutability: Some(mutability.into()),
            })
            .unwrap(),
        )?;

        Ok(())
    }
}
