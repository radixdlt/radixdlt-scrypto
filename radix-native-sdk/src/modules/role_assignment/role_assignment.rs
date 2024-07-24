use radix_common::constants::ROLE_ASSIGNMENT_MODULE_PACKAGE;
use radix_common::data::scrypto::model::Own;
use radix_common::data::scrypto::*;
use radix_engine_interface::api::object_api::ModuleId;
use radix_engine_interface::api::*;
use radix_engine_interface::blueprints::resource::{
    AccessRule, OwnerRoleEntry, RoleAssignmentInit, RoleKey,
};
use radix_engine_interface::object_modules::role_assignment::*;
use radix_engine_interface::types::NodeId;
use sbor::rust::prelude::*;

pub struct RoleAssignment(pub Own);

impl RoleAssignment {
    pub fn create<Y: SystemApi<E>, E: SystemApiError, R: Into<OwnerRoleEntry>>(
        owner_role: R,
        roles: IndexMap<ModuleId, RoleAssignmentInit>,
        api: &mut Y,
    ) -> Result<Self, E> {
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
    fn self_id(&self) -> (&NodeId, Option<AttachedModuleId>) {
        (&self.0 .0, None)
    }
}

pub struct AttachedRoleAssignment(pub NodeId);

impl RoleAssignmentObject for AttachedRoleAssignment {
    fn self_id(&self) -> (&NodeId, Option<AttachedModuleId>) {
        (&self.0, Some(AttachedModuleId::RoleAssignment))
    }
}

pub trait RoleAssignmentObject {
    fn self_id(&self) -> (&NodeId, Option<AttachedModuleId>);

    fn set_owner_role<Y: SystemApi<E>, E: SystemApiError, A: Into<AccessRule>>(
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

    fn lock_owner_role<Y: SystemApi<E>, E: SystemApiError>(&self, api: &mut Y) -> Result<(), E> {
        let (node_id, module_id) = self.self_id();
        match module_id {
            None => {
                api.call_method(
                    node_id,
                    ROLE_ASSIGNMENT_LOCK_OWNER_IDENT,
                    scrypto_encode(&RoleAssignmentLockOwnerInput {}).unwrap(),
                )?;
            }
            Some(module_id) => {
                api.call_module_method(
                    node_id,
                    module_id,
                    ROLE_ASSIGNMENT_LOCK_OWNER_IDENT,
                    scrypto_encode(&RoleAssignmentLockOwnerInput {}).unwrap(),
                )?;
            }
        }

        Ok(())
    }

    fn set_role<Y: SystemApi<E>, E: SystemApiError, R: Into<RoleKey>, A: Into<AccessRule>>(
        &self,
        module: ModuleId,
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

    fn get_role<Y: SystemApi<E>, E: SystemApiError, R: Into<RoleKey>>(
        &self,
        module: ModuleId,
        role_key: R,
        api: &mut Y,
    ) -> Result<RoleAssignmentGetOutput, E> {
        let (node_id, module_id) = self.self_id();
        match module_id {
            None => api
                .call_method(
                    node_id,
                    ROLE_ASSIGNMENT_GET_IDENT,
                    scrypto_encode(&RoleAssignmentGetInput {
                        module,
                        role_key: role_key.into(),
                    })
                    .unwrap(),
                )
                .map(|response| scrypto_decode(&response).unwrap()),
            Some(module_id) => api
                .call_module_method(
                    node_id,
                    module_id,
                    ROLE_ASSIGNMENT_GET_IDENT,
                    scrypto_encode(&RoleAssignmentGetInput {
                        module,
                        role_key: role_key.into(),
                    })
                    .unwrap(),
                )
                .map(|response| scrypto_decode(&response).unwrap()),
        }
    }

    fn get_owner_role<Y: SystemApi<E>, E: SystemApiError, R: Into<RoleKey>>(
        &self,
        api: &mut Y,
    ) -> Result<RoleAssignmentGetOwnerRoleOutput, E> {
        let (node_id, module_id) = self.self_id();
        match module_id {
            None => api
                .call_method(
                    node_id,
                    ROLE_ASSIGNMENT_GET_OWNER_ROLE_IDENT,
                    scrypto_encode(&RoleAssignmentGetOwnerRoleInput).unwrap(),
                )
                .map(|response| scrypto_decode(&response).unwrap()),
            Some(module_id) => api
                .call_module_method(
                    node_id,
                    module_id,
                    ROLE_ASSIGNMENT_GET_OWNER_ROLE_IDENT,
                    scrypto_encode(&RoleAssignmentGetOwnerRoleInput).unwrap(),
                )
                .map(|response| scrypto_decode(&response).unwrap()),
        }
    }
}
