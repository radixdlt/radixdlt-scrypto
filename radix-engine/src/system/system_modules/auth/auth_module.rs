use super::Authorization;
use crate::blueprints::resource::{AuthZone, VaultUtil};
use crate::errors::*;
use crate::kernel::actor::{Actor, MethodActor};
use crate::kernel::call_frame::Message;
use crate::kernel::kernel_api::{KernelApi, KernelSubstateApi};
use crate::system::module::SystemModule;
use crate::system::node_init::type_info_partition;
use crate::system::node_modules::access_rules::AccessRulesNativePackage;
use crate::system::node_modules::type_info::TypeInfoSubstate;
use crate::system::system::{SubstateMutability, SubstateWrapper, SystemService};
use crate::system::system_callback::{SystemConfig, SystemLockData};
use crate::system::system_callback_api::SystemCallbackObject;
use crate::system::system_modules::auth::ActingLocation;
use crate::types::*;
use radix_engine_interface::api::field_lock_api::LockFlags;
use radix_engine_interface::api::node_modules::auth::*;
use radix_engine_interface::api::{ClientObjectApi, ObjectModuleId};
use radix_engine_interface::blueprints::package::{BlueprintVersion, FunctionAuthTemplate, PACKAGE_BLUEPRINT, PACKAGE_FUNCTION_AUTH_PARTITION_OFFSET, PACKAGE_PUBLISH_NATIVE_IDENT};
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::schema::{SchemaMethodKey, SchemaMethodPermission};
use radix_engine_interface::types::*;
use transaction::model::AuthZoneParams;
use crate::blueprints::package::PackageNativePackage;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum AuthError {
    NoFunction(FnIdentifier),
    NoMethod(FnIdentifier),
    UsedReservedRole(String),
    VisibilityError(NodeId),
    Unauthorized(Box<Unauthorized>),
    InnerBlueprintDoesNotExist(String),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum FailedAccessRules {
    AuthorityList(Vec<(RoleKey, Vec<AccessRule>)>),
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

impl AuthModule {
    fn function_auth<Y: KernelApi<SystemConfig<V>>, V: SystemCallbackObject>(
        blueprint: &BlueprintId,
        ident: &str,
        api: &mut SystemService<Y, V>,
    ) -> Result<AccessRule, RuntimeError> {
        let auth = if blueprint.package_address.eq(&PACKAGE_PACKAGE) {
            // TODO: remove
            if blueprint.blueprint_name.eq(PACKAGE_BLUEPRINT)
                && ident.eq(PACKAGE_PUBLISH_NATIVE_IDENT)
            {
                AccessRule::Protected(AccessRuleNode::ProofRule(ProofRule::Require(
                    ResourceOrNonFungible::NonFungible(AuthAddresses::system_role()),
                )))
            } else {
                AccessRule::AllowAll
            }
        } else {
            let mut auth_template = PackageNativePackage::get_bp_function_auth_template(blueprint, api)?;
            let access_rule = auth_template.rules.remove(ident);
            if let Some(access_rule) = access_rule {
                access_rule
            } else {
                return Err(RuntimeError::ModuleError(ModuleError::AuthError(
                    AuthError::NoFunction(FnIdentifier {
                        blueprint: blueprint.clone(),
                        ident: FnIdent::Application(ident.to_string()),
                    }),
                )));
            }
        };

        Ok(auth)
    }

    fn check_method_authorization<Y: KernelApi<SystemConfig<V>>, V: SystemCallbackObject>(
        auth_zone_id: &NodeId,
        callee: &MethodActor,
        args: &IndexedScryptoValue,
        api: &mut SystemService<Y, V>,
    ) -> Result<(), RuntimeError> {
        let node_id = callee.node_id;
        let module_id = callee.module_id;
        let ident = callee.ident.as_str();
        let acting_location = if callee.module_object_info.global {
            ActingLocation::AtBarrier
        } else {
            ActingLocation::AtLocalBarrier
        };

        let info = api.get_object_info(&node_id)?;
        let method_key = MethodKey::new(module_id, ident);

        if let Some(parent) = info.outer_object {
            Self::check_authorization_against_access_rules(
                callee,
                auth_zone_id,
                acting_location,
                parent.as_node_id(),
                ObjectKey::InnerBlueprint(info.blueprint_id.blueprint_name.clone()),
                method_key.clone(),
                args,
                api,
            )?;
        }

        if info.global || VaultUtil::is_vault_blueprint(&info.blueprint_id) {
            Self::check_authorization_against_access_rules(
                callee,
                auth_zone_id,
                acting_location,
                &node_id,
                ObjectKey::SELF,
                method_key,
                args,
                api,
            )?;
        }

        Ok(())
    }

    fn check_authorization_against_access_rules<
        Y: KernelApi<SystemConfig<V>>,
        V: SystemCallbackObject,
    >(
        callee: &MethodActor, // TODO: Cleanup
        auth_zone_id: &NodeId,
        acting_location: ActingLocation,
        access_rules_of: &NodeId,
        object_key: ObjectKey,
        method_key: MethodKey,
        args: &IndexedScryptoValue,
        api: &mut SystemService<Y, V>,
    ) -> Result<(), RuntimeError> {
        // TODO: Cleanup logic here
        let node_authority_rules = match &object_key {
            ObjectKey::SELF => {
                let template = api.get_bp_method_auth_template(&callee.node_object_info.blueprint_id)?;
                template.method_auth_template
            }
            ObjectKey::InnerBlueprint(_blueprint_name) => {
                let template = api.get_bp_method_auth_template(&callee.node_object_info.blueprint_id)?;
                template.outer_method_auth_template
            }
        };

        let permission = match method_key.module_id {
            ObjectModuleId::AccessRules => {
                match &object_key {
                    ObjectKey::SELF => {}
                    ObjectKey::InnerBlueprint(..) => return Ok(()),
                }
                AccessRulesNativePackage::authorization(
                    access_rules_of,
                    method_key.ident.as_str(),
                    args,
                    api,
                )?
            }
            _ => {
                let method_key = SchemaMethodKey {
                    ident: method_key.ident,
                    module_id: method_key.module_id.to_u8(),
                };
                if let Some(permission) = node_authority_rules.get(&method_key) {
                    match permission {
                        SchemaMethodPermission::Public => MethodPermission::Public,
                        SchemaMethodPermission::Protected(list) => {
                            MethodPermission::Protected(list.clone().into())
                        }
                    }
                } else {
                    match &object_key {
                        ObjectKey::SELF => {
                            return Err(RuntimeError::ModuleError(ModuleError::AuthError(
                                AuthError::NoMethod(callee.fn_identifier()),
                            )));
                        }
                        _ => return Ok(()),
                    }
                }
            }
        };

        let role_list = match permission {
            MethodPermission::Public => return Ok(()),
            MethodPermission::Protected(list) => list,
        };

        Self::check_authorization_against_role_list(
            callee.fn_identifier(),
            auth_zone_id,
            acting_location,
            access_rules_of,
            &role_list,
            api,
        )?;

        Ok(())
    }

    pub fn check_authorization_against_role_list<
        Y: KernelApi<SystemConfig<V>>,
        V: SystemCallbackObject,
    >(
        fn_identifier: FnIdentifier, // TODO: Cleanup
        auth_zone_id: &NodeId,
        acting_location: ActingLocation,
        access_rules_of: &NodeId,
        role_list: &RoleList,
        api: &mut SystemService<Y, V>,
    ) -> Result<(), RuntimeError> {
        let result = Authorization::check_authorization_against_role_list(
            acting_location,
            *auth_zone_id,
            access_rules_of,
            role_list,
            api,
        )?;
        match result {
            AuthorityListAuthorizationResult::Authorized => Ok(()),
            AuthorityListAuthorizationResult::Failed(auth_list_fail) => {
                Err(RuntimeError::ModuleError(ModuleError::AuthError(
                    AuthError::Unauthorized(Box::new(Unauthorized {
                        failed_access_rules: FailedAccessRules::AuthorityList(auth_list_fail),
                        fn_identifier,
                    })),
                )))
            }
        }
    }

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

            // Decide `authorization`, `barrier_crossing_allowed`, and `tip_auth_zone_id`
            match &callee {
                Actor::Method(actor) => {
                    Self::check_method_authorization(&auth_zone_id, actor, &args, &mut system)?;
                }
                Actor::Function { blueprint, ident } => {
                    let access_rule = Self::function_auth(blueprint, ident.as_str(), &mut system)?;
                    let acting_location = ActingLocation::AtBarrier;

                    // Verify authorization
                    let auth_result = Authorization::check_authorization_against_access_rule(
                        acting_location,
                        auth_zone_id,
                        &access_rule,
                        &mut system,
                    )?;
                    match auth_result {
                        AuthorizationCheckResult::Authorized => {}
                        AuthorizationCheckResult::Failed(access_rule_stack) => {
                            return Err(RuntimeError::ModuleError(ModuleError::AuthError(
                                AuthError::Unauthorized(Box::new(Unauthorized {
                                    failed_access_rules: FailedAccessRules::AccessRule(
                                        access_rule_stack,
                                    ),
                                    fn_identifier: callee.fn_identifier(),
                                })),
                            )));
                        }
                    }
                }
                Actor::VirtualLazyLoad { .. } | Actor::Root => {}
            };
        } else {
            // Bypass auth check for ROOT frame
        }

        Ok(())
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
        let is_transaction_processor = callee.is_transaction_processor();
        let (virtual_resources, virtual_non_fungibles) = if is_transaction_processor {
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

                    outer_object: None,
                    instance_schema: None,
                    features: btreeset!(),
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
