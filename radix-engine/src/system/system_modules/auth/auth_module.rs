use super::Authorization;
use crate::blueprints::package::PackageAuthNativeBlueprint;
use crate::blueprints::resource::AuthZone;
use crate::errors::*;
use crate::kernel::actor::Actor;
use crate::kernel::call_frame::ReferenceOrigin;
use crate::kernel::kernel_api::{KernelApi, KernelInternalApi, KernelNodeApi, KernelSubstateApi};
use crate::system::module::SystemModule;
use crate::system::node_init::type_info_partition;
use crate::system::node_modules::role_assignment::RoleAssignmentNativePackage;
use crate::system::node_modules::type_info::TypeInfoSubstate;
use crate::system::system::{FieldSubstate, SystemService};
use crate::system::system_callback::{SystemConfig, SystemLockData};
use crate::system::system_callback_api::SystemCallbackObject;
use crate::types::*;
use radix_engine_interface::api::{ClientBlueprintApi, LockFlags, ModuleId, ObjectModuleId};
use radix_engine_interface::blueprints::package::{
    BlueprintVersion, BlueprintVersionKey, MethodAuthTemplate, RoleSpecification,
};
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::blueprints::transaction_processor::TRANSACTION_PROCESSOR_BLUEPRINT;
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
        role_assignment_of: GlobalAddress,
        module_id: ObjectModuleId,
        role_list: RoleList,
    },
    AccessRule(AccessRule),
    AllowAll,
}

impl AuthModule {
    pub fn on_call_function<V, Y>(
        api: &mut SystemService<Y, V>,
        blueprint_id: &BlueprintId,
        ident: &str,
    ) -> Result<NodeId, RuntimeError>
    where
        V: SystemCallbackObject,
        Y: KernelApi<SystemConfig<V>>,
    {
        // Create AuthZone
        let auth_zone = {
            // TODO: Remove special casing use of transaction processor and just have virtual resources
            // stored in root call frame
            let is_transaction_processor_blueprint = blueprint_id
                .package_address
                .eq(&TRANSACTION_PROCESSOR_PACKAGE)
                && blueprint_id
                    .blueprint_name
                    .eq(TRANSACTION_PROCESSOR_BLUEPRINT);
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

            Self::on_execution_start(api, None, virtual_resources, virtual_non_fungibles)?
        };

        // Check authorization
        {
            // Step 1: Resolve method to permission
            let permission = PackageAuthNativeBlueprint::resolve_function_permission(
                blueprint_id.package_address.as_node_id(),
                &BlueprintVersionKey::new_default(blueprint_id.blueprint_name.as_str()),
                ident,
                api.api,
            )?;

            // Step 2: Check permission
            let fn_identifier = FnIdentifier {
                blueprint_id: blueprint_id.clone(),
                ident: ident.to_string(),
            };
            Self::check_permission(&auth_zone, permission, fn_identifier, api)?;
        }

        Ok(auth_zone)
    }

    pub fn on_call_function_finish<V, Y>(
        api: &mut SystemService<Y, V>,
        auth_zone: NodeId,
    ) -> Result<(), RuntimeError>
    where
        V: SystemCallbackObject,
        Y: KernelApi<SystemConfig<V>>,
    {
        Self::on_fn_finish(api, auth_zone)
    }

    pub fn on_call_method<V, Y>(
        api: &mut SystemService<Y, V>,
        receiver: &NodeId,
        obj_module_id: ObjectModuleId,
        direct_access: bool,
        ident: &str,
        args: &IndexedScryptoValue,
    ) -> Result<NodeId, RuntimeError>
    where
        V: SystemCallbackObject,
        Y: KernelApi<SystemConfig<V>>,
    {
        let auth_zone = AuthModule::on_execution_start(
            api,
            Some((receiver, direct_access)),
            btreeset!(),
            btreeset!(),
        )?;

        // Step 1: Resolve method to permission
        let module_id = match obj_module_id {
            ObjectModuleId::Main => None,
            ObjectModuleId::Metadata => Some(ModuleId::Metadata),
            ObjectModuleId::Royalty => Some(ModuleId::Royalty),
            ObjectModuleId::RoleAssignment => Some(ModuleId::RoleAssignment),
        };

        let blueprint_id = api.get_blueprint_info(receiver, module_id)?.blueprint_id;

        let permission = Self::resolve_method_permission(
            api,
            &blueprint_id,
            receiver,
            &obj_module_id,
            ident,
            args,
        )?;

        // Step 2: Check permission
        let fn_identifier = FnIdentifier {
            blueprint_id: blueprint_id.clone(),
            ident: ident.to_string(),
        };
        Self::check_permission(&auth_zone, permission, fn_identifier, api)?;

        Ok(auth_zone)
    }

    pub fn on_call_method_finish<V, Y>(
        api: &mut SystemService<Y, V>,
        auth_zone: NodeId,
    ) -> Result<(), RuntimeError>
    where
        V: SystemCallbackObject,
        Y: KernelApi<SystemConfig<V>>,
    {
        Self::on_fn_finish(api, auth_zone)
    }

    pub fn create_mock<V, Y>(
        system: &mut SystemService<Y, V>,
        receiver: Option<(&NodeId, bool)>,
        virtual_resources: BTreeSet<ResourceAddress>,
        virtual_non_fungibles: BTreeSet<NonFungibleGlobalId>,
    ) -> Result<NodeId, RuntimeError>
    where
        V: SystemCallbackObject,
        Y: KernelApi<SystemConfig<V>>,
    {
        Self::on_execution_start(system, receiver, virtual_resources, virtual_non_fungibles)
    }

    fn copy_global_caller<V, Y>(
        system: &mut SystemService<Y, V>,
        node_id: &NodeId,
    ) -> Result<(Option<(GlobalCaller, Reference)>, Option<SubstateHandle>), RuntimeError>
    where
        V: SystemCallbackObject,
        Y: KernelApi<SystemConfig<V>>,
    {
        let handle = system.kernel_open_substate(
            node_id,
            MAIN_BASE_PARTITION,
            &AuthZoneField::AuthZone.into(),
            LockFlags::read_only(),
            SystemLockData::default(),
        )?;

        let auth_zone: FieldSubstate<AuthZone> =
            system.kernel_read_substate(handle)?.as_typed().unwrap();
        Ok((auth_zone.value.0.global_caller, Some(handle)))
    }

    fn on_execution_start<V, Y>(
        system: &mut SystemService<Y, V>,
        receiver: Option<(&NodeId, bool)>,
        virtual_resources: BTreeSet<ResourceAddress>,
        virtual_non_fungibles: BTreeSet<NonFungibleGlobalId>,
    ) -> Result<NodeId, RuntimeError>
    where
        V: SystemCallbackObject,
        Y: KernelApi<SystemConfig<V>>,
    {
        let (auth_zone, parent_lock_handle) = {
            let next_is_barrier = if let Some((receiver, direct_access)) = receiver {
                let object_info = system.get_object_info(receiver)?;
                object_info.is_global() || direct_access
            } else {
                true
            };

            let current_actor = system.current_actor();
            let local_package_address = current_actor.package_address();

            // Retrieve global caller property of next auth zone
            let (global_caller, parent_lock_handle) = match current_actor {
                Actor::Root | Actor::BlueprintHook(..) => (None, None),
                Actor::Method(current_method_actor) => {
                    let node_visibility =
                        system.kernel_get_node_visibility(&current_method_actor.node_id);
                    let current_ref_origin = node_visibility
                        .reference_origin(current_method_actor.node_id)
                        .unwrap();
                    let self_auth_zone = current_method_actor.auth_zone;
                    match (current_ref_origin, next_is_barrier) {
                        (ReferenceOrigin::Global(address), true) => {
                            let global_caller: GlobalCaller = address.into();
                            (Some((global_caller, Reference(self_auth_zone))), None)
                        }
                        (
                            ReferenceOrigin::SubstateNonGlobalReference(..)
                            | ReferenceOrigin::DirectlyAccessed,
                            _,
                        ) => (None, None),
                        (ReferenceOrigin::Global(..), false) | (ReferenceOrigin::FrameOwned, _) => {
                            Self::copy_global_caller(system, &self_auth_zone)?
                        }
                    }
                }
                Actor::Function(function_actor) => {
                    let self_auth_zone = function_actor.auth_zone;
                    let global_caller = function_actor.as_global_caller();
                    if next_is_barrier {
                        (Some((global_caller, Reference(self_auth_zone))), None)
                    } else {
                        Self::copy_global_caller(system, &self_auth_zone)?
                    }
                }
            };

            let self_auth_zone_parent = if next_is_barrier {
                None
            } else {
                system
                    .current_actor()
                    .self_auth_zone()
                    .map(|x| Reference(x))
            };

            let auth_zone = AuthZone::new(
                vec![],
                virtual_resources,
                virtual_non_fungibles,
                local_package_address,
                global_caller,
                self_auth_zone_parent,
            );

            (auth_zone, parent_lock_handle)
        };

        // Create node
        let self_auth_zone = system
            .api
            .kernel_allocate_node_id(EntityType::InternalGenericComponent)?;

        system.api.kernel_create_node(
            self_auth_zone,
            btreemap!(
                MAIN_BASE_PARTITION => btreemap!(
                    AuthZoneField::AuthZone.into() => IndexedScryptoValue::from_typed(&FieldSubstate::new_field(auth_zone))
                ),
                TYPE_INFO_FIELD_PARTITION => type_info_partition(TypeInfoSubstate::Object(ObjectInfo {
                    blueprint_info: BlueprintInfo {
                        blueprint_id: BlueprintId::new(&RESOURCE_PACKAGE, AUTH_ZONE_BLUEPRINT),
                        blueprint_version: BlueprintVersion::default(),
                        outer_obj_info: OuterObjectInfo::default(),
                        features: btreeset!(),
                        generic_substitutions: vec![],
                    },
                    object_type: ObjectType::Owned,
                }))
            ),
        )?;
        system.api.kernel_pin_node(self_auth_zone)?;

        if let Some(parent_lock_handle) = parent_lock_handle {
            system.kernel_close_substate(parent_lock_handle)?;
        }

        Ok(self_auth_zone)
    }

    pub fn on_fn_finish<V, Y>(
        api: &mut SystemService<Y, V>,
        self_auth_zone: NodeId,
    ) -> Result<(), RuntimeError>
    where
        V: SystemCallbackObject,
        Y: KernelApi<SystemConfig<V>>,
    {
        // Detach proofs from the auth zone
        let handle = api.kernel_open_substate(
            &self_auth_zone,
            MAIN_BASE_PARTITION,
            &AuthZoneField::AuthZone.into(),
            LockFlags::MUTABLE,
            SystemLockData::Default,
        )?;
        let mut substate: FieldSubstate<AuthZone> =
            api.kernel_read_substate(handle)?.as_typed().unwrap();
        let proofs = core::mem::replace(&mut substate.value.0.proofs, Vec::new());
        api.kernel_write_substate(handle, IndexedScryptoValue::from_typed(&substate.value.0))?;
        api.kernel_close_substate(handle)?;

        // Drop all proofs (previously) owned by the auth zone
        for proof in proofs {
            let object_info = api.get_object_info(proof.0.as_node_id())?;
            api.call_function(
                RESOURCE_PACKAGE,
                &object_info.blueprint_info.blueprint_id.blueprint_name,
                PROOF_DROP_IDENT,
                scrypto_encode(&ProofDropInput { proof }).unwrap(),
            )?;
        }

        // Drop the auth zone
        api.kernel_drop_node(&self_auth_zone)?;

        Ok(())
    }

    fn check_permission<Y: KernelApi<SystemConfig<V>>, V: SystemCallbackObject>(
        auth_zone: &NodeId,
        resolved_permission: ResolvedPermission,
        fn_identifier: FnIdentifier,
        api: &mut SystemService<Y, V>,
    ) -> Result<(), RuntimeError> {
        match resolved_permission {
            ResolvedPermission::AllowAll => return Ok(()),
            ResolvedPermission::AccessRule(rule) => {
                let result =
                    Authorization::check_authorization_against_access_rule(api, &auth_zone, &rule)?;

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
                    &auth_zone,
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
        api: &mut SystemService<Y, V>,
        blueprint_id: &BlueprintId,
        receiver: &NodeId,
        module_id: &ObjectModuleId,
        ident: &str,
        args: &IndexedScryptoValue,
    ) -> Result<ResolvedPermission, RuntimeError> {
        let method_key = MethodKey::new(ident);

        if let ObjectModuleId::RoleAssignment = module_id {
            // Only global objects have role assignment modules
            let global_address = GlobalAddress::new_or_panic(receiver.0);
            return RoleAssignmentNativePackage::authorization(&global_address, ident, args, api);
        }

        let auth_template = PackageAuthNativeBlueprint::get_bp_auth_template(
            blueprint_id.package_address.as_node_id(),
            &BlueprintVersionKey::new_default(blueprint_id.blueprint_name.as_str()),
            api.api,
        )?
        .method_auth;

        let receiver_object_info = api.get_object_info(&receiver)?;

        let (role_assignment_of, method_permissions) = match auth_template {
            MethodAuthTemplate::StaticRoleDefinition(static_roles) => {
                let role_assignment_of = match static_roles.roles {
                    RoleSpecification::Normal(..) => {
                        // Non-globalized objects do not have access rules module
                        if !receiver_object_info.is_global() {
                            return Ok(ResolvedPermission::AllowAll);
                        }

                        GlobalAddress::new_or_panic(receiver.0)
                    }
                    RoleSpecification::UseOuter => receiver_object_info.get_outer_object(),
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
            Some(MethodAccessibility::OuterObjectOnly) => match module_id {
                ObjectModuleId::Main => {
                    let outer_object_info = &receiver_object_info.blueprint_info.outer_obj_info;
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
                    module_id: module_id.clone(),
                })
            }
            None => {
                let fn_identifier = FnIdentifier {
                    blueprint_id: blueprint_id.clone(),
                    ident: ident.to_string(),
                };
                Err(RuntimeError::SystemModuleError(
                    SystemModuleError::AuthError(AuthError::NoMethodMapping(fn_identifier)),
                ))
            }
        }
    }
}

impl<V: SystemCallbackObject> SystemModule<SystemConfig<V>> for AuthModule {}
