use super::Authorization;
use crate::blueprints::package::PackageAuthNativeBlueprint;
use crate::blueprints::resource::AuthZone;
use crate::errors::*;
use crate::kernel::actor::{Actor, MethodActor};
use crate::kernel::call_frame::Message;
use crate::kernel::kernel_api::KernelApi;
use crate::system::module::SystemModule;
use crate::system::node_init::type_info_partition;
use crate::system::node_modules::access_rules::AccessRulesNativePackage;
use crate::system::node_modules::type_info::TypeInfoSubstate;
use crate::system::system::SystemService;
use crate::system::system_callback::SystemConfig;
use crate::system::system_callback_api::SystemCallbackObject;
use crate::system::system_modules::auth::ActingLocation;
use crate::types::*;
use radix_engine_interface::api::{ClientObjectApi, ObjectModuleId};
use radix_engine_interface::blueprints::package::{
    BlueprintVersion, BlueprintVersionKey, MethodAuthTemplate, RoleSpecification,
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
    pub fn_identifier: FnIdentifier,
}

#[derive(Debug, Clone)]
pub struct AuthModule {
    pub params: AuthZoneParams,
    /// Stack of auth zones
    /// Invariants:
    /// - An auth zone is created for every non-frame.
    /// - Auth zones are created by the caller frame and moved to the callee
    pub auth_zone_stack: Vec<NodeId>,
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
        access_rules_of: NodeId,
        module_id: ObjectModuleId,
        role_list: RoleList,
    },
    AccessRule(AccessRule),
    AllowAll,
}

impl AuthModule {
    pub fn last_auth_zone(&self) -> Option<NodeId> {
        self.auth_zone_stack.last().cloned()
    }

    fn check_authorization<V, Y>(
        callee: &Actor,
        args: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        V: SystemCallbackObject,
        Y: KernelApi<SystemConfig<V>>,
    {
        if let Some(auth_zone_id) = api.kernel_get_system().modules.auth.last_auth_zone() {
            let mut system = SystemService::new(api);

            // Step 1: Resolve method to permission
            // Decide `authorization`, `barrier_crossing_allowed`, and `tip_auth_zone_id`
            let (permission, acting_location) = match &callee {
                Actor::Method(actor) => {
                    let resolved_permission =
                        Self::resolve_method_permission(actor, args, &mut system)?;
                    let acting_location = if actor.module_object_info.global {
                        ActingLocation::AtBarrier
                    } else {
                        ActingLocation::AtLocalBarrier
                    };

                    (resolved_permission, acting_location)
                }
                Actor::Function {
                    blueprint_id,
                    ident,
                } => {
                    let resolved_permission =
                        PackageAuthNativeBlueprint::resolve_function_permission(
                            blueprint_id.package_address.as_node_id(),
                            &BlueprintVersionKey::new_default(blueprint_id.blueprint_name.as_str()),
                            ident.as_str(),
                            system.api,
                        )?;

                    (resolved_permission, ActingLocation::AtBarrier)
                }
                Actor::VirtualLazyLoad { .. } | Actor::Root => return Ok(()),
            };

            // Step 2: Check permission
            Self::check_permission(
                &auth_zone_id,
                acting_location,
                permission,
                callee.fn_identifier(),
                &mut system,
            )?;
        } else {
            // Bypass auth check for ROOT frame
        }

        Ok(())
    }

    fn check_permission<Y: KernelApi<SystemConfig<V>>, V: SystemCallbackObject>(
        auth_zone_id: &NodeId,
        acting_location: ActingLocation,
        resolved_permission: ResolvedPermission,
        fn_identifier: FnIdentifier,
        api: &mut SystemService<Y, V>,
    ) -> Result<(), RuntimeError> {
        match resolved_permission {
            ResolvedPermission::AllowAll => return Ok(()),
            ResolvedPermission::AccessRule(rule) => {
                let result = Authorization::check_authorization_against_access_rule(
                    acting_location,
                    auth_zone_id.clone(),
                    &rule,
                    api,
                )?;

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
                access_rules_of,
                role_list,
                module_id,
            } => {
                let result = Authorization::check_authorization_against_role_list(
                    acting_location,
                    *auth_zone_id,
                    &access_rules_of,
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

        if let ObjectModuleId::AccessRules = callee.module_id {
            return AccessRulesNativePackage::authorization(
                &callee.node_id,
                method_key.ident.as_str(),
                args,
                api,
            );
        }

        let auth_template = PackageAuthNativeBlueprint::get_bp_auth_template(
            callee
                .module_object_info
                .blueprint_id
                .package_address
                .as_node_id(),
            &BlueprintVersionKey::new_default(
                callee
                    .module_object_info
                    .blueprint_id
                    .blueprint_name
                    .as_str(),
            ),
            api.api,
        )?
        .method_auth;

        let (access_rules_of, method_permissions) = match auth_template {
            MethodAuthTemplate::StaticRoles(static_roles) => {
                let access_rules_of = match static_roles.roles {
                    RoleSpecification::Normal(..) => {
                        // Non-globalized objects do not have access rules module
                        if !callee.module_object_info.global {
                            return Ok(ResolvedPermission::AllowAll);
                        }

                        callee.node_id
                    }
                    RoleSpecification::UseOuter => {
                        let node_id = callee.node_id;
                        let info = api.get_object_info(&node_id)?;

                        let access_rules_of = info.get_outer_object();
                        access_rules_of.into_node_id()
                    }
                };

                (access_rules_of, static_roles.methods)
            }
            MethodAuthTemplate::AllowAll => return Ok(ResolvedPermission::AllowAll),
        };

        match method_permissions.get(&method_key) {
            Some(MethodAccessibility::Public) => Ok(ResolvedPermission::AllowAll),
            Some(MethodAccessibility::OuterObjectOnly) => {
                match callee.module_object_info.blueprint_info {
                    ObjectBlueprintInfo::Inner { outer_object } => Ok(
                        ResolvedPermission::AccessRule(rule!(require(global_caller(outer_object)))),
                    ),
                    ObjectBlueprintInfo::Outer { .. } => Err(RuntimeError::SystemModuleError(
                        SystemModuleError::AuthError(AuthError::InvalidOuterObjectMapping),
                    )),
                }
            }
            Some(MethodAccessibility::RoleProtected(role_list)) => {
                Ok(ResolvedPermission::RoleList {
                    access_rules_of,
                    role_list: role_list.clone(),
                    module_id: callee.module_id,
                })
            }
            None => Err(RuntimeError::SystemModuleError(
                SystemModuleError::AuthError(AuthError::NoMethodMapping(callee.fn_identifier())),
            )),
        }
    }

    /// Create a new auth zone and move it to next frame.
    ///
    /// Must be done before a new frame is created, as
    /// borrowed references must be wrapped and passed.
    ///
    fn create_auth_zone<Y: KernelApi<SystemConfig<V>>, V: SystemCallbackObject>(
        api: &mut Y,
        callee: &Actor,
        message: &mut Message,
    ) -> Result<(), RuntimeError> {
        // Add Global Object and Package Actor Auth
        let virtual_non_fungibles_non_extending = callee.get_virtual_non_extending_proofs();
        let virtual_non_fungibles_non_extending_barrier =
            callee.get_virtual_non_extending_barrier_proofs();

        // Prepare a new auth zone
        let is_barrier = callee.is_barrier();
        // TODO: Remove special casing use of transaction processor and just have virtual resources
        // stored in root call frame
        let is_transaction_processor_blueprint = callee.is_transaction_processor_blueprint();
        let is_at_root = api.kernel_get_current_depth() == 0;
        let (virtual_resources, virtual_non_fungibles) =
            if is_transaction_processor_blueprint && is_at_root {
                let auth_module = &api.kernel_get_system().modules.auth;
                (
                    auth_module.params.virtual_resources.clone(),
                    auth_module.params.initial_proofs.clone(),
                )
            } else {
                (BTreeSet::new(), BTreeSet::new())
            };
        let parent = api
            .kernel_get_system()
            .modules
            .auth
            .auth_zone_stack
            .last()
            .map(|x| Reference(x.clone().into()));
        let auth_zone = AuthZone::new(
            vec![],
            virtual_resources,
            virtual_non_fungibles,
            virtual_non_fungibles_non_extending,
            virtual_non_fungibles_non_extending_barrier,
            is_barrier,
            parent,
        );

        // Create node
        let auth_zone_node_id =
            api.kernel_allocate_node_id(EntityType::InternalGenericComponent)?;

        api.kernel_create_node(
            auth_zone_node_id,
            btreemap!(
                MAIN_BASE_PARTITION => btreemap!(
                    AuthZoneField::AuthZone.into() => IndexedScryptoValue::from_typed(&auth_zone)
                ),
                TYPE_INFO_FIELD_PARTITION => type_info_partition(TypeInfoSubstate::Object(ObjectInfo {
                    global: false,

                    blueprint_id: BlueprintId::new(&RESOURCE_PACKAGE, AUTH_ZONE_BLUEPRINT),
                    version: BlueprintVersion::default(),

                    blueprint_info: ObjectBlueprintInfo::default(),
                    features: btreeset!(),
                    instance_schema: None,
                }))
            ),
        )?;

        // Move auth zone (containing borrowed reference)!
        message.add_move_node(auth_zone_node_id);

        // Update auth zone stack
        api.kernel_get_system()
            .modules
            .auth
            .auth_zone_stack
            .push(auth_zone_node_id);

        Ok(())
    }
}

impl<V: SystemCallbackObject> SystemModule<SystemConfig<V>> for AuthModule {
    fn before_push_frame<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        callee: &Actor,
        message: &mut Message,
        args: &IndexedScryptoValue,
    ) -> Result<(), RuntimeError> {
        AuthModule::check_authorization(callee, args, api)
            .and_then(|_| AuthModule::create_auth_zone(api, callee, message))
    }

    fn after_pop_frame<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        _dropped_actor: &Actor,
    ) -> Result<(), RuntimeError> {
        // update internal state
        api.kernel_get_system().modules.auth.auth_zone_stack.pop();
        Ok(())
    }
}
