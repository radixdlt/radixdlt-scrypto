use radix_engine_interface::api::node_modules::auth::{
    AccessRulesCreateInput, AccessRulesUpdateRoleInput, ACCESS_RULES_BLUEPRINT,
    ACCESS_RULES_CREATE_IDENT, ACCESS_RULES_UPDATE_ROLE_IDENT,
};
use radix_engine_interface::api::object_api::ObjectModuleId;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::resource::{AccessRule, OwnerRole, RoleKey, Roles};
use radix_engine_interface::constants::ACCESS_RULES_MODULE_PACKAGE;
use radix_engine_interface::data::scrypto::model::Own;
use radix_engine_interface::data::scrypto::*;
use radix_engine_interface::types::NodeId;
use sbor::rust::fmt::Debug;
use sbor::rust::prelude::*;

pub struct AccessRules(pub Own);

impl AccessRules {
    pub fn create<Y, E: Debug + ScryptoDecode>(
        owner_role: OwnerRole,
        roles: BTreeMap<ObjectModuleId, Roles>,
        api: &mut Y,
    ) -> Result<Self, E>
    where
        Y: ClientApi<E>,
    {
        let rtn = api.call_function(
            ACCESS_RULES_MODULE_PACKAGE,
            ACCESS_RULES_BLUEPRINT,
            ACCESS_RULES_CREATE_IDENT,
            scrypto_encode(&AccessRulesCreateInput { owner_role, roles }).unwrap(),
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
    >(
        &self,
        module: ObjectModuleId,
        role_key: R,
        rule: Option<A>,
        freeze: bool,
        api: &mut Y,
    ) -> Result<(), E> {
        let (node_id, module_id) = self.self_id();
        let _rtn = api.call_method_advanced(
            node_id,
            false,
            module_id,
            ACCESS_RULES_UPDATE_ROLE_IDENT,
            scrypto_encode(&AccessRulesUpdateRoleInput {
                module,
                role_key: role_key.into(),
                rule: rule.map(|a| a.into()),
                freeze,
                //mutability: Some((entry.mutable, entry.mutable_mutable)),
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
        module: ObjectModuleId,
        role_key: R,
        rule: A,
        api: &mut Y,
    ) -> Result<(), E> {
        let (node_id, module_id) = self.self_id();
        let _rtn = api.call_method_advanced(
            node_id,
            false,
            module_id,
            ACCESS_RULES_UPDATE_ROLE_IDENT,
            scrypto_encode(&AccessRulesUpdateRoleInput {
                module,
                role_key: role_key.into(),
                rule: Some(rule.into()),
                freeze: false,
            })
            .unwrap(),
        )?;

        Ok(())
    }
}
