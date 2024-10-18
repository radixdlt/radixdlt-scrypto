use super::Authorization;
use crate::blueprints::package::PackageAuthNativeBlueprint;
use crate::blueprints::resource::AuthZone;
use crate::errors::*;
use crate::internal_prelude::*;
use crate::kernel::call_frame::ReferenceOrigin;
use crate::kernel::kernel_api::{KernelInternalApi, KernelNodeApi, KernelSubstateApi};
use crate::object_modules::role_assignment::RoleAssignmentNativePackage;
use crate::system::actor::Actor;
use crate::system::module::*;
use crate::system::node_init::type_info_partition;
use crate::system::system::SystemService;
use crate::system::system_callback::*;
use crate::system::type_info::TypeInfoSubstate;
use radix_engine_interface::api::{AttachedModuleId, LockFlags, ModuleId, SystemBlueprintApi};
use radix_engine_interface::blueprints::package::{
    BlueprintVersion, BlueprintVersionKey, MethodAuthTemplate, RoleSpecification,
};
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::blueprints::transaction_processor::TRANSACTION_PROCESSOR_BLUEPRINT;
use radix_engine_interface::types::*;
use radix_transactions::model::AuthZoneInit;

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
    /// Special-case the initial transaction processor function call and
    /// add virtual resources to the transaction processor call frame
    pub generate_transaction_processor_auth_zone: Option<AuthZoneInit>,
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
        module_id: ModuleId,
        role_list: RoleList,
    },
    AccessRule(AccessRule),
    AllowAll,
}

impl AuthModule {
    pub fn new() -> Self {
        Self {
            generate_transaction_processor_auth_zone: None,
        }
    }

    pub fn new_with_transaction_processor_auth_zone(auth_zone_init: AuthZoneInit) -> Self {
        Self {
            generate_transaction_processor_auth_zone: Some(auth_zone_init),
        }
    }

    fn on_call_function_auth_zone_params<Y: SystemBasedKernelApi>(
        system: &mut SystemService<Y>,
        blueprint_id: &BlueprintId,
    ) -> Result<(BTreeSet<ResourceAddress>, BTreeSet<NonFungibleGlobalId>), RuntimeError> {
        let is_root_call_frame = system
            .kernel_get_system_state()
            .current_call_frame
            .is_root();
        let is_root_thread = system.is_root_thread()?;
        if is_root_call_frame && is_root_thread {
            let auth_module = &system.kernel_get_system().modules.auth;
            if let Some(auth_zone_init) = &auth_module.generate_transaction_processor_auth_zone {
                let is_transaction_processor_blueprint = blueprint_id
                    .package_address
                    .eq(&TRANSACTION_PROCESSOR_PACKAGE)
                    && blueprint_id
                        .blueprint_name
                        .eq(TRANSACTION_PROCESSOR_BLUEPRINT);
                if is_transaction_processor_blueprint {
                    return Ok((
                        auth_zone_init.simulate_every_proof_under_resources.clone(),
                        auth_zone_init.initial_non_fungible_id_proofs.clone(),
                    ));
                }
            }
        }

        Ok((BTreeSet::new(), BTreeSet::new()))
    }

    pub fn on_call_function<Y: SystemBasedKernelApi>(
        system: &mut SystemService<Y>,
        blueprint_id: &BlueprintId,
        ident: &str,
    ) -> Result<NodeId, RuntimeError> {
        // Create AuthZone
        let auth_zone = {
            let (virtual_resources, virtual_non_fungibles) =
                Self::on_call_function_auth_zone_params(system, blueprint_id)?;
            Self::create_auth_zone(system, None, virtual_resources, virtual_non_fungibles)?
        };

        // Check authorization
        {
            // Step 1: Resolve method to permission
            let permission = PackageAuthNativeBlueprint::resolve_function_permission(
                blueprint_id.package_address.as_node_id(),
                &BlueprintVersionKey::new_default(blueprint_id.blueprint_name.as_str()),
                ident,
                system.api(),
            )?;

            // Step 2: Check permission
            let fn_identifier = FnIdentifier {
                blueprint_id: blueprint_id.clone(),
                ident: ident.to_string(),
            };
            Self::check_permission(&auth_zone, permission, fn_identifier, system)?;
        }

        Ok(auth_zone)
    }

    pub fn on_call_function_finish<Y: SystemBasedKernelApi>(
        api: &mut SystemService<Y>,
        auth_zone: NodeId,
    ) -> Result<(), RuntimeError> {
        Self::teardown_auth_zone(api, auth_zone)
    }

    pub fn on_call_method<Y: SystemBasedKernelApi>(
        api: &mut SystemService<Y>,
        receiver: &NodeId,
        module_id: ModuleId,
        direct_access: bool,
        ident: &str,
        args: &IndexedScryptoValue,
    ) -> Result<NodeId, RuntimeError> {
        let auth_zone = AuthModule::create_auth_zone(
            api,
            Some((receiver, direct_access)),
            btreeset!(),
            btreeset!(),
        )?;

        // Step 1: Resolve method to permission
        let attached_module_id = match module_id {
            ModuleId::Main => None,
            ModuleId::Metadata => Some(AttachedModuleId::Metadata),
            ModuleId::Royalty => Some(AttachedModuleId::Royalty),
            ModuleId::RoleAssignment => Some(AttachedModuleId::RoleAssignment),
        };

        let blueprint_id = api
            .get_blueprint_info(receiver, attached_module_id)?
            .blueprint_id;

        let permission =
            Self::resolve_method_permission(api, &blueprint_id, receiver, &module_id, ident, args)?;

        // Step 2: Check permission
        let fn_identifier = FnIdentifier {
            blueprint_id: blueprint_id.clone(),
            ident: ident.to_string(),
        };
        Self::check_permission(&auth_zone, permission, fn_identifier, api)?;

        Ok(auth_zone)
    }

    pub fn on_call_method_finish<Y: SystemBasedKernelApi>(
        api: &mut SystemService<Y>,
        auth_zone: NodeId,
    ) -> Result<(), RuntimeError> {
        Self::teardown_auth_zone(api, auth_zone)
    }

    /// On CALL_FUNCTION or CALL_METHOD, when auth module is disabled.
    pub fn on_call_fn_mock<Y: SystemBasedKernelApi>(
        system: &mut SystemService<Y>,
        receiver: Option<(&NodeId, bool)>,
        virtual_resources: BTreeSet<ResourceAddress>,
        virtual_non_fungibles: BTreeSet<NonFungibleGlobalId>,
    ) -> Result<NodeId, RuntimeError> {
        Self::create_auth_zone(system, receiver, virtual_resources, virtual_non_fungibles)
    }

    fn copy_global_caller<Y: SystemBasedKernelApi>(
        system: &mut SystemService<Y>,
        node_id: &NodeId,
    ) -> Result<(Option<(GlobalCaller, Reference)>, Option<SubstateHandle>), RuntimeError> {
        let handle = system.kernel_open_substate(
            node_id,
            MAIN_BASE_PARTITION,
            &AuthZoneField::AuthZone.into(),
            LockFlags::read_only(),
            SystemLockData::default(),
        )?;

        let auth_zone = system
            .kernel_read_substate(handle)?
            .as_typed::<FieldSubstate<AuthZone>>()
            .unwrap();
        Ok((auth_zone.into_payload().global_caller, Some(handle)))
    }

    pub(crate) fn create_auth_zone<Y: SystemBasedKernelApi>(
        system: &mut SystemService<Y>,
        receiver: Option<(&NodeId, bool)>,
        virtual_resources: BTreeSet<ResourceAddress>,
        virtual_non_fungibles: BTreeSet<NonFungibleGlobalId>,
    ) -> Result<NodeId, RuntimeError> {
        let (auth_zone, parent_lock_handle) = {
            let is_global_context_change = if let Some((receiver, direct_access)) = receiver {
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
                    match (current_ref_origin, is_global_context_change) {
                        // Actor is part of the global component state tree AND next actor is a global context change
                        (ReferenceOrigin::Global(address), true) => {
                            (Some((address.into(), Reference(self_auth_zone))), None)
                        }
                        // Actor is part of the global component state tree AND next actor is NOT a global context change
                        (ReferenceOrigin::Global(..), false) => {
                            Self::copy_global_caller(system, &self_auth_zone)?
                        }
                        // Actor is a direct access reference
                        (ReferenceOrigin::DirectlyAccessed, _) => (None, None),
                        // Actor is a non-global reference
                        (ReferenceOrigin::SubstateNonGlobalReference(..), _) => (None, None),
                        // Actor is a frame-owned object
                        (ReferenceOrigin::FrameOwned, _) => {
                            // In the past frame-owned objects were inheriting the AuthZone of the caller.
                            // It was a critical issue, which could allow called components to eg.
                            // withdraw resources from the signing account.
                            // To prevent this we use TRANSACTION_TRACKER NodeId as a marker, that we are dealing with a frame-owned object.
                            // It is checked later on when virtual proofs for AuthZone are verified.
                            // Approach with such marker allows to keep backward compatibility with substate database.
                            let (caller, lock_handle) =
                                Self::copy_global_caller(system, &self_auth_zone)?;
                            (
                                caller.map(|_| {
                                    (FRAME_OWNED_GLOBAL_MARKER.into(), Reference(self_auth_zone))
                                }),
                                lock_handle,
                            )
                        }
                    }
                }
                Actor::Function(function_actor) => {
                    let self_auth_zone = function_actor.auth_zone;
                    let global_caller = function_actor.as_global_caller();
                    if is_global_context_change {
                        (Some((global_caller, Reference(self_auth_zone))), None)
                    } else {
                        Self::copy_global_caller(system, &self_auth_zone)?
                    }
                }
            };

            let auth_zone_parent = if is_global_context_change {
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
                auth_zone_parent,
            );

            (auth_zone, parent_lock_handle)
        };

        // Create node
        let new_auth_zone = system
            .api()
            .kernel_allocate_node_id(EntityType::InternalGenericComponent)?;

        system.api().kernel_create_node(
            new_auth_zone,
            btreemap!(
                MAIN_BASE_PARTITION => btreemap!(
                    AuthZoneField::AuthZone.into() => IndexedScryptoValue::from_typed(&FieldSubstate::new_unlocked_field(auth_zone))
                ),
                TYPE_INFO_FIELD_PARTITION => type_info_partition(TypeInfoSubstate::Object(ObjectInfo {
                    blueprint_info: BlueprintInfo {
                        blueprint_id: BlueprintId::new(&RESOURCE_PACKAGE, AUTH_ZONE_BLUEPRINT),
                        blueprint_version: BlueprintVersion::default(),
                        outer_obj_info: OuterObjectInfo::default(),
                        features: indexset!(),
                        generic_substitutions: vec![],
                    },
                    object_type: ObjectType::Owned,
                }))
            ),
        )?;
        system.api().kernel_pin_node(new_auth_zone)?;

        if let Some(parent_lock_handle) = parent_lock_handle {
            system.kernel_close_substate(parent_lock_handle)?;
        }

        Ok(new_auth_zone)
    }

    pub fn teardown_auth_zone<Y: SystemBasedKernelApi>(
        api: &mut SystemService<Y>,
        self_auth_zone: NodeId,
    ) -> Result<(), RuntimeError> {
        // Detach proofs from the auth zone
        let handle = api.kernel_open_substate(
            &self_auth_zone,
            MAIN_BASE_PARTITION,
            &AuthZoneField::AuthZone.into(),
            LockFlags::MUTABLE,
            SystemLockData::Default,
        )?;
        let mut auth_zone = api
            .kernel_read_substate(handle)?
            .as_typed::<FieldSubstate<AuthZone>>()
            .unwrap()
            .into_payload();
        let proofs = core::mem::replace(&mut auth_zone.proofs, Vec::new());
        api.kernel_write_substate(
            handle,
            IndexedScryptoValue::from_typed(&FieldSubstate::new_unlocked_field(auth_zone)),
        )?;
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

    fn check_permission<Y: SystemBasedKernelApi>(
        auth_zone: &NodeId,
        resolved_permission: ResolvedPermission,
        fn_identifier: FnIdentifier,
        api: &mut SystemService<Y>,
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

    fn resolve_method_permission<Y: SystemBasedKernelApi>(
        system: &mut SystemService<Y>,
        blueprint_id: &BlueprintId,
        receiver: &NodeId,
        module_id: &ModuleId,
        ident: &str,
        args: &IndexedScryptoValue,
    ) -> Result<ResolvedPermission, RuntimeError> {
        let method_key = MethodKey::new(ident);

        if let ModuleId::RoleAssignment = module_id {
            // Only global objects have role assignment modules
            let global_address = GlobalAddress::new_or_panic(receiver.0);
            return RoleAssignmentNativePackage::authorization(
                &global_address,
                ident,
                args,
                system,
            );
        }

        let auth_template = PackageAuthNativeBlueprint::get_bp_auth_template(
            blueprint_id.package_address.as_node_id(),
            &BlueprintVersionKey::new_default(blueprint_id.blueprint_name.as_str()),
            system.api(),
        )?
        .method_auth;

        let receiver_object_info = system.get_object_info(&receiver)?;

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
                ModuleId::Main => {
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

impl InitSystemModule for AuthModule {}
impl ResolvableSystemModule for AuthModule {
    #[inline]
    fn resolve_from_system(system: &mut impl HasModules) -> &mut Self {
        &mut system.modules_mut().auth
    }
}
impl PrivilegedSystemModule for AuthModule {}
impl<ModuleApi: SystemModuleApiFor<Self>> SystemModule<ModuleApi> for AuthModule {}
