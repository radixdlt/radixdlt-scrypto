use radix_engine_interface::api::node_modules::auth::{
    RoleAssignmentCreateInput, RoleAssignmentLockOwnerInput, RoleAssignmentSetInput,
    RoleAssignmentSetOwnerInput, ROLE_ASSIGNMENT_BLUEPRINT, ROLE_ASSIGNMENT_CREATE_IDENT,
    ROLE_ASSIGNMENT_SET_IDENT, ROLE_ASSIGNMENT_SET_OWNER_IDENT,
};
use radix_engine_interface::api::object_api::ObjectModuleId;
use radix_engine_interface::api::{ClientApi, ModuleId};
use radix_engine_interface::blueprints::resource::{
    AccessRule, OwnerRoleEntry, RoleAssignmentInit, RoleKey,
};
use radix_engine_interface::constants::ROLE_ASSIGNMENT_MODULE_PACKAGE;
use radix_engine_interface::data::scrypto::model::Own;
use radix_engine_interface::data::scrypto::*;
use radix_engine_interface::types::NodeId;
use sbor::rust::fmt::Debug;
use sbor::rust::prelude::*;

pub struct RoleAssignment(pub Own);

impl RoleAssignment {
    pub fn create<Y, R: Into<OwnerRoleEntry>, E: Debug + ScryptoDecode>(
        owner_role: R,
        roles: BTreeMap<ObjectModuleId, RoleAssignmentInit>,
        api: &mut Y,
    ) -> Result<Self, E>
    where
        Y: ClientApi<E>,
    {
        let rtn = api.call_function(
            ROLE_ASSIGNMENT_MODULE_PACKAGE,
            ROLE_ASSIGNMENT_BLUEPRINT,
            ROLE_ASSIGNMENT_CREATE_IDENT,
            scrypto_encode(&RoleAssignmentCreateInput {
                owner_role: owner_role.into(),
                roles,
            })
            .unwrap(),
        )?;

        let role_assignment: Own = scrypto_decode(&rtn).unwrap();

        Ok(Self(role_assignment))
    }
}

impl RoleAssignmentObject for RoleAssignment {
    fn self_id(&self) -> (&NodeId, Option<ModuleId>) {
        (&self.0 .0, None)
    }
}

pub struct AttachedRoleAssignment(pub NodeId);

impl RoleAssignmentObject for AttachedRoleAssignment {
    fn self_id(&self) -> (&NodeId, Option<ModuleId>) {
        (&self.0, Some(ModuleId::RoleAssignment))
    }
}

pub trait RoleAssignmentObject {
    fn self_id(&self) -> (&NodeId, Option<ModuleId>);

    fn set_owner_role<Y: ClientApi<E>, E: Debug + ScryptoDecode, A: Into<AccessRule>>(
        &self,
        rule: A,
        api: &mut Y,
    ) -> Result<(), E> {
        let (node_id, module_id) = self.self_id();
        match module_id {
            None => {
                api.call_method(
                    node_id,
                    ROLE_ASSIGNMENT_SET_OWNER_IDENT,
                    scrypto_encode(&RoleAssignmentSetOwnerInput { rule: rule.into() }).unwrap(),
                )?;
            }
            Some(module_id) => {
                api.call_module_method(
                    node_id,
                    module_id,
                    ROLE_ASSIGNMENT_SET_OWNER_IDENT,
                    scrypto_encode(&RoleAssignmentSetOwnerInput { rule: rule.into() }).unwrap(),
                )?;
            }
        }

        Ok(())
    }

    fn set_role<
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
        match module_id {
            None => {
                api.call_method(
                    node_id,
                    ROLE_ASSIGNMENT_SET_IDENT,
                    scrypto_encode(&RoleAssignmentSetInput {
                        module,
                        role_key: role_key.into(),
                        rule: rule.into(),
                    })
                    .unwrap(),
                )?;
            }
            Some(module_id) => {
                api.call_module_method(
                    node_id,
                    module_id,
                    ROLE_ASSIGNMENT_SET_IDENT,
                    scrypto_encode(&RoleAssignmentSetInput {
                        module,
                        role_key: role_key.into(),
                        rule: rule.into(),
                    })
                    .unwrap(),
                )?;
            }
        }

        Ok(())
    }
}
