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
    /// SystemV1 only - we special-case the initial transaction processor
    /// function call and add virtual resources to the transaction processor
    /// call frame
    pub v1_transaction_processor_proofs_for_injection: Option<AuthZoneInit>,
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
            v1_transaction_processor_proofs_for_injection: None,
        }
    }

    pub fn new_with_transaction_processor_auth_zone(auth_zone_init: AuthZoneInit) -> Self {
        Self {
            v1_transaction_processor_proofs_for_injection: Some(auth_zone_init),
        }
    }

    // In SystemV1, the transaction processor is initiated via a call_function, and we
    // used this to inject the signature proofs and resource simulation.
    //
    // In SystemV2 and later, we initialize the auth zone directly, so no longer have
    // need to do this check.
    fn system_v1_resolve_injectable_transaction_processor_proofs<Y: SystemBasedKernelApi>(
        system: &mut SystemService<Y>,
        blueprint_id: &BlueprintId,
    ) -> Result<(BTreeSet<ResourceAddress>, BTreeSet<NonFungibleGlobalId>), RuntimeError> {
        let is_root_call_frame = system
            .kernel_get_system_state()
            .current_call_frame
            .is_root();
        let is_root_thread = system.kernel_get_current_stack_id_uncosted() == 0;
        if is_root_call_frame && is_root_thread {
            let auth_module = &system.kernel_get_system().modules.auth;
            if let Some(auth_zone_init) = &auth_module.v1_transaction_processor_proofs_for_injection
            {
                // This is an extra sanity check / defense in depth which I believe isn't strictly needed.
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
            if system
                .system()
                .versioned_system_logic
                .should_inject_transaction_processor_proofs_in_call_function()
            {
                let (simulate_all_proofs_under_resources, implicit_non_fungible_proofs) =
                    Self::system_v1_resolve_injectable_transaction_processor_proofs(
                        system,
                        blueprint_id,
                    )?;
                Self::create_auth_zone(
                    system,
                    None,
                    simulate_all_proofs_under_resources,
                    implicit_non_fungible_proofs,
                )?
            } else {
                Self::create_auth_zone(system, None, Default::default(), Default::default())?
            }
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
        simulate_all_proofs_under_resources: BTreeSet<ResourceAddress>,
        implicit_non_fungible_proofs: BTreeSet<NonFungibleGlobalId>,
    ) -> Result<NodeId, RuntimeError> {
        Self::create_auth_zone(
            system,
            receiver,
            simulate_all_proofs_under_resources,
            implicit_non_fungible_proofs,
        )
    }

    fn copy_global_caller<Y: SystemBasedKernelApi>(
        system: &mut SystemService<Y>,
        direct_caller_auth_zone_id: &NodeId,
    ) -> Result<(Option<(GlobalCaller, Reference)>, Option<SubstateHandle>), RuntimeError> {
        let direct_caller_auth_zone_handle = system.kernel_open_substate(
            direct_caller_auth_zone_id,
            MAIN_BASE_PARTITION,
            &AuthZoneField::AuthZone.into(),
            LockFlags::read_only(),
            SystemLockData::default(),
        )?;

        let direct_caller_auth_zone = system
            .kernel_read_substate(direct_caller_auth_zone_handle)?
            .as_typed::<FieldSubstate<AuthZone>>()
            .unwrap();

        Ok((
            direct_caller_auth_zone.into_payload().global_caller,
            Some(direct_caller_auth_zone_handle),
        ))
    }

    pub(crate) fn create_auth_zone<Y: SystemBasedKernelApi>(
        system: &mut SystemService<Y>,
        receiver: Option<(&NodeId, bool)>,
        simulate_all_proofs_under_resources: BTreeSet<ResourceAddress>,
        implicit_non_fungible_proofs: BTreeSet<NonFungibleGlobalId>,
    ) -> Result<NodeId, RuntimeError> {
        let (auth_zone, parent_lock_handle) = {
            let is_global_context_change = if let Some((receiver, direct_access)) = receiver {
                let object_info = system.get_object_info(receiver)?;
                object_info.is_global() || direct_access
            } else {
                true
            };

            let direct_caller = system.current_actor();
            let direct_caller_package_address = direct_caller.package_address();

            // Retrieve global caller property of next auth zone
            let (global_caller, parent_lock_handle) = match direct_caller {
                Actor::Root | Actor::BlueprintHook(..) => (None, None),
                Actor::Method(direct_caller_method_actor) => {
                    let direct_caller_ancestor_visibility_origin = system
                        .kernel_get_node_visibility_uncosted(&direct_caller_method_actor.node_id)
                        .reference_origin(direct_caller_method_actor.node_id)
                        .unwrap();
                    let direct_caller_auth_zone = direct_caller_method_actor.auth_zone;

                    // The `direct_caller_ancestor_visibility_origin` is rather indirect, but it is intended to
                    // capture the concept:  "Who is the direct caller's ancestor for the purpose of auth?"
                    //
                    // In particular:
                    // * If the direct caller is a global object, then it has ReferenceOrigin::Global
                    // * If the direct caller was loaded from a substate belonging to a global object,
                    //   then it gets a Borrowed visibility, which transforms into a ReferenceOrigin::Global.
                    //   This also works transitively.
                    // * If the direct caller was made visible by being passed to the call frame, (i.e. it didn't
                    //   arise from track), then it is `ReferenceOrigin::FrameOwned`
                    //
                    // At some point we should refactor this to make this all much more explicit.
                    match (
                        direct_caller_ancestor_visibility_origin,
                        is_global_context_change,
                    ) {
                        // Direct caller's ancestor is global AND this call is a global context change
                        (ReferenceOrigin::Global(global_root_address), true) => {
                            let global_caller_address = global_root_address.into();
                            let global_caller_leaf_auth_zone_reference =
                                Reference(direct_caller_auth_zone);
                            (
                                Some((
                                    global_caller_address,
                                    global_caller_leaf_auth_zone_reference,
                                )),
                                None,
                            )
                        }
                        // Direct caller's ancestor is global AND this call is NOT a global context change
                        (ReferenceOrigin::Global(..), false) => {
                            Self::copy_global_caller(system, &direct_caller_auth_zone)?
                        }
                        // Direct caller's ancestor was directly accessed
                        (ReferenceOrigin::DirectlyAccessed, _) => (None, None),
                        // Direct caller's ancestor was borrowed from an internal referance in a substate
                        (ReferenceOrigin::SubstateNonGlobalReference(..), _) => (None, None),
                        // Direct caller's ancestor was passed into the call frame
                        (ReferenceOrigin::FrameOwned, _) => {
                            // In the past, all frame-owned direct callers copied their global caller to their callee.
                            // This was a mistake, as it could allow frame-owned objects to use proofs from e.g.
                            // the transaction processor.
                            //
                            // A fix needed to be backwards-compatible (without changing the size of substates, which would
                            // affect the fee costs), and whilst the auth zone reference could be fixed by using a `Reference`
                            // to `self_auth_zone`, the global caller was harder.
                            //
                            // As a work-around, the `FRAME_OWNED_GLOBAL_MARKER = TRANSACTION_TRACKER` was used as a marker
                            // that the global caller was invalid and shouldn't be used. It is checked used to avoid adding
                            // a global caller implicit proof in this case.

                            let (caller, lock_handle) =
                                Self::copy_global_caller(system, &direct_caller_auth_zone)?;

                            // To avoid changing the size of the substate, we need to make sure that we replace Some
                            // with Some and None with None.
                            let global_caller = match caller {
                                Some(_) => {
                                    let global_caller_address = FRAME_OWNED_GLOBAL_MARKER.into();
                                    // NOTE: This results in both the global caller stack and the parent stack being the same.
                                    // This won't cause any critical issues, but should be reworked at some point.
                                    let global_caller_leaf_auth_zone_reference =
                                        Reference(direct_caller_auth_zone);
                                    Some((
                                        global_caller_address,
                                        global_caller_leaf_auth_zone_reference,
                                    ))
                                }
                                None => None,
                            };

                            (global_caller, lock_handle)
                        }
                    }
                }
                Actor::Function(function_actor) => {
                    let direct_caller_auth_zone = function_actor.auth_zone;
                    let global_caller = function_actor.as_global_caller();
                    if is_global_context_change {
                        (
                            Some((global_caller, Reference(direct_caller_auth_zone))),
                            None,
                        )
                    } else {
                        Self::copy_global_caller(system, &direct_caller_auth_zone)?
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
                simulate_all_proofs_under_resources,
                implicit_non_fungible_proofs,
                direct_caller_package_address,
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
