use super::Authorization;
use crate::blueprints::package::PackageAuthNativeBlueprint;
use crate::errors::*;
use crate::kernel::actor::{Actor, AuthInfo, FunctionActor, MethodActor};
use crate::kernel::kernel_api::{KernelApi, KernelInvocation};
use crate::system::module::SystemModule;
use crate::system::node_modules::role_assignment::RoleAssignmentNativePackage;
use crate::system::system::SystemService;
use crate::system::system_callback::SystemConfig;
use crate::system::system_callback_api::SystemCallbackObject;
use crate::types::*;
use radix_engine_interface::api::ObjectModuleId;
use radix_engine_interface::blueprints::package::{
    BlueprintVersionKey, MethodAuthTemplate, RoleSpecification,
};
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::types::*;
use transaction::model::AuthZoneParams;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum AuthError {
    NoFunction(FnIdentifier),
    NoMethodMapping(FnIdentifier),
    VisibilityError(NodeId),
    Unauthorized(Box<Unauthorized>),
    InnerBlueprintDoesNotExist(String),
    InvalidOuterObjectMapping,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum FailedAccessRules {
    RoleList(Vec<(RoleKey, Vec<AccessRule>)>),
    AccessRule(Vec<AccessRule>),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct Unauthorized {
    pub failed_access_rules: FailedAccessRules,
    pub fn_identifier: Option<FnIdentifier>,
}

#[derive(Debug, Clone)]
pub struct AuthModule {
    pub params: AuthZoneParams,
}

pub enum AuthorizationCheckResult {
    Authorized,
    Failed(Vec<AccessRule>),
}

pub enum AuthorityListAuthorizationResult {
    Authorized,
    Failed(Vec<(RoleKey, Vec<AccessRule>)>),
}

pub enum ResolvedPermission {
    RoleList {
        role_assignment_of: NodeId,
        module_id: ObjectModuleId,
        role_list: RoleList,
    },
    AccessRule(AccessRule),
    AllowAll,
}

impl AuthModule {
    fn check_authorization<V, Y>(
        callee: &Actor,
        args: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        V: SystemCallbackObject,
        Y: KernelApi<SystemConfig<V>>,
    {
        if let Some(auth_info) = callee.auth_info() {
            let mut system = SystemService::new(api);

            // Step 1: Resolve method to permission
            // Decide `authorization`, `barrier_crossing_allowed`, and `tip_auth_zone_id`
            let permission = match &callee {
                Actor::Method(actor) => Self::resolve_method_permission(actor, args, &mut system)?,
                Actor::Function(FunctionActor {
                    blueprint_id,
                    ident,
                    ..
                }) => PackageAuthNativeBlueprint::resolve_function_permission(
                    blueprint_id.package_address.as_node_id(),
                    &BlueprintVersionKey::new_default(blueprint_id.blueprint_name.as_str()),
                    ident.as_str(),
                    system.api,
                )?,
                // TODO: Remove (aka move check_authorization into system)
                Actor::BlueprintHook(..) | Actor::Root => return Ok(()),
            };

            // Step 2: Check permission
            Self::check_permission(auth_info, permission, callee.fn_identifier(), &mut system)?;
        } else {
            // Bypass auth check for ROOT frame
        }

        Ok(())
    }

    fn check_permission<Y: KernelApi<SystemConfig<V>>, V: SystemCallbackObject>(
        auth_info: AuthInfo,
        resolved_permission: ResolvedPermission,
        fn_identifier: Option<FnIdentifier>,
        api: &mut SystemService<Y, V>,
    ) -> Result<(), RuntimeError> {
        match resolved_permission {
            ResolvedPermission::AllowAll => return Ok(()),
            ResolvedPermission::AccessRule(rule) => {
                let result =
                    Authorization::check_authorization_against_access_rule(&auth_info, &rule, api)?;

                match result {
                    AuthorizationCheckResult::Authorized => Ok(()),
                    AuthorizationCheckResult::Failed(access_rule_stack) => Err(
                        RuntimeError::SystemModuleError(SystemModuleError::AuthError(
                            AuthError::Unauthorized(Box::new(Unauthorized {
                                failed_access_rules: FailedAccessRules::AccessRule(
                                    access_rule_stack,
                                ),
                                fn_identifier,
                            })),
                        )),
                    ),
                }
            }
            ResolvedPermission::RoleList {
                role_assignment_of,
                role_list,
                module_id,
            } => {
                let result = Authorization::check_authorization_against_role_list(
                    &auth_info,
                    &role_assignment_of,
                    module_id,
                    &role_list,
                    api,
                )?;

                match result {
                    AuthorityListAuthorizationResult::Authorized => Ok(()),
                    AuthorityListAuthorizationResult::Failed(auth_list_fail) => Err(
                        RuntimeError::SystemModuleError(SystemModuleError::AuthError(
                            AuthError::Unauthorized(Box::new(Unauthorized {
                                failed_access_rules: FailedAccessRules::RoleList(auth_list_fail),
                                fn_identifier,
                            })),
                        )),
                    ),
                }
            }
        }
    }

    fn resolve_method_permission<Y: KernelApi<SystemConfig<V>>, V: SystemCallbackObject>(
        callee: &MethodActor,
        args: &IndexedScryptoValue,
        api: &mut SystemService<Y, V>,
    ) -> Result<ResolvedPermission, RuntimeError> {
        let method_key = MethodKey::new(callee.ident.as_str());

        if let ObjectModuleId::RoleAssignment = callee.module_id {
            return RoleAssignmentNativePackage::authorization(
                &callee.node_id,
                method_key.ident.as_str(),
                args,
                api,
            );
        }

        let blueprint_id = api
            .get_blueprint_info(&callee.node_id, callee.module_id)?
            .blueprint_id;

        let auth_template = PackageAuthNativeBlueprint::get_bp_auth_template(
            blueprint_id.package_address.as_node_id(),
            &BlueprintVersionKey::new_default(blueprint_id.blueprint_name.as_str()),
            api.api,
        )?
        .method_auth;

        let (role_assignment_of, method_permissions) = match auth_template {
            MethodAuthTemplate::StaticRoles(static_roles) => {
                let role_assignment_of = match static_roles.roles {
                    RoleSpecification::Normal(..) => {
                        // Non-globalized objects do not have access rules module
                        if !callee.object_info.global {
                            return Ok(ResolvedPermission::AllowAll);
                        }

                        callee.node_id
                    }
                    RoleSpecification::UseOuter => {
                        let node_id = callee.node_id;
                        let info = api.get_object_info(&node_id)?;

                        let role_assignment_of = info.get_outer_object();
                        role_assignment_of.into_node_id()
                    }
                };

                (role_assignment_of, static_roles.methods)
            }
            MethodAuthTemplate::AllowAll => return Ok(ResolvedPermission::AllowAll),
        };

        match method_permissions.get(&method_key) {
            Some(MethodAccessibility::Public) => Ok(ResolvedPermission::AllowAll),
            Some(MethodAccessibility::OwnPackageOnly) => {
                let package = blueprint_id.package_address;
                Ok(ResolvedPermission::AccessRule(rule!(require(
                    package_of_direct_caller(package)
                ))))
            }
            Some(MethodAccessibility::OuterObjectOnly) => match callee.module_id {
                ObjectModuleId::Main => {
                    let outer_object_info = &callee.object_info.blueprint_info.outer_obj_info;
                    match outer_object_info {
                        OuterObjectInfo::Some { outer_object } => {
                            Ok(ResolvedPermission::AccessRule(rule!(require(
                                global_caller(*outer_object)
                            ))))
                        }
                        OuterObjectInfo::None { .. } => Err(RuntimeError::SystemModuleError(
                            SystemModuleError::AuthError(AuthError::InvalidOuterObjectMapping),
                        )),
                    }
                }
                _ => Err(RuntimeError::SystemModuleError(
                    SystemModuleError::AuthError(AuthError::InvalidOuterObjectMapping),
                )),
            },
            Some(MethodAccessibility::RoleProtected(role_list)) => {
                Ok(ResolvedPermission::RoleList {
                    role_assignment_of,
                    role_list: role_list.clone(),
                    module_id: callee.module_id,
                })
            }
            None => Err(RuntimeError::SystemModuleError(
                SystemModuleError::AuthError(AuthError::NoMethodMapping(callee.fn_identifier())),
            )),
        }
    }
}

impl<V: SystemCallbackObject> SystemModule<SystemConfig<V>> for AuthModule {
    fn before_invoke<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        invocation: &KernelInvocation<Actor>,
    ) -> Result<(), RuntimeError> {
        AuthModule::check_authorization(&invocation.call_frame_data, &invocation.args, api)
    }
}
