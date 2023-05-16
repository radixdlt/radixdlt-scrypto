use super::Authorization;
use crate::blueprints::resource::AuthZone;
use crate::errors::*;
use crate::kernel::actor::{Actor, MethodActor};
use crate::kernel::call_frame::Message;
use crate::kernel::kernel_api::{KernelApi, KernelSubstateApi};
use crate::system::module::SystemModule;
use crate::system::node_init::ModuleInit;
use crate::system::node_modules::access_rules::{
    AccessRulesConfig, AccessRulesNativePackage, CycleCheckError, FunctionAccessRulesSubstate,
    MethodAccessRulesSubstate,
};
use crate::system::node_modules::type_info::TypeInfoSubstate;
use crate::system::system::SystemService;
use crate::system::system_callback::{SystemConfig, SystemLockData};
use crate::system::system_callback_api::SystemCallbackObject;
use crate::system::system_modules::auth::ActingLocation;
use crate::types::*;
use radix_engine_interface::api::field_lock_api::LockFlags;
use radix_engine_interface::api::node_modules::auth::*;
use radix_engine_interface::api::{ClientObjectApi, ObjectModuleId};
use radix_engine_interface::blueprints::package::{
    PACKAGE_BLUEPRINT, PACKAGE_PUBLISH_NATIVE_IDENT,
};
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::types::*;
use transaction::model::AuthZoneParams;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum AuthError {
    CycleCheckError(CycleCheckError<String>),
    VisibilityError(NodeId),
    Unauthorized(Box<Unauthorized>),
    InnerBlueprintDoesNotExist(String),
}
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct Unauthorized {
    pub access_rule_stack: Vec<AccessRule>,
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
            let handle = api.kernel_lock_substate(
                blueprint.package_address.as_node_id(),
                OBJECT_BASE_PARTITION,
                &PackageField::FunctionAccessRules.into(),
                LockFlags::read_only(),
                SystemLockData::default(),
            )?;
            let package_access_rules: FunctionAccessRulesSubstate =
                api.kernel_read_substate(handle)?.as_typed().unwrap();
            let function_key = FnKey::new(blueprint.blueprint_name.to_string(), ident.to_string());
            let access_rule = package_access_rules
                .access_rules
                .get(&function_key)
                .unwrap_or(&package_access_rules.default_auth);
            access_rule.clone()
        };

        Ok(auth)
    }

    fn check_method_authorization<Y: KernelApi<SystemConfig<V>>, V: SystemCallbackObject>(
        auth_zone_id: &NodeId,
        actor: &MethodActor,
        args: &IndexedScryptoValue,
        api: &mut SystemService<Y, V>,
    ) -> Result<(), RuntimeError> {
        let node_id = actor.node_id;
        let module_id = actor.module_id;
        let ident = actor.ident.as_str();
        let acting_location = if actor.object_info.global {
            ActingLocation::AtBarrier
        } else {
            ActingLocation::AtLocalBarrier
        };

        match (node_id, module_id, ident) {
            (node_id, module_id, ident) if matches!(module_id, ObjectModuleId::AccessRules) => {
                let access_rule =
                    AccessRulesNativePackage::authorization(&node_id, ident, args, api)?;

                let auth_result = Authorization::check_authorization_against_access_rule(
                    acting_location,
                    *auth_zone_id,
                    &AccessRulesConfig::new(),
                    &access_rule,
                    api,
                )?;
                match auth_result {
                    AuthorizationCheckResult::Authorized => {}
                    AuthorizationCheckResult::Failed(access_rule_stack) => {
                        return Err(RuntimeError::ModuleError(ModuleError::AuthError(
                            AuthError::Unauthorized(Box::new(Unauthorized {
                                access_rule_stack,
                                fn_identifier: actor.fn_identifier(),
                            })),
                        )));
                    }
                }
            }
            (node_id, module_id, ..) => {
                let method_key = MethodKey::new(module_id, ident);

                let info = api.get_object_info(&node_id)?;

                if let Some(parent) = info.outer_object {
                    let method_key = MethodKey::new(module_id, ident);
                    Self::check_authorization_against_access_rules(
                        actor.fn_identifier(),
                        auth_zone_id,
                        acting_location,
                        parent.as_node_id(),
                        ObjectKey::InnerBlueprint(info.blueprint.blueprint_name),
                        method_key,
                        api,
                    )?;
                }

                if info.global {
                    Self::check_authorization_against_access_rules(
                        actor.fn_identifier(),
                        auth_zone_id,
                        acting_location,
                        &node_id,
                        ObjectKey::SELF,
                        method_key,
                        api,
                    )?;
                }
            }
        }

        Ok(())
    }

    fn check_authorization_against_access_rules<
        Y: KernelApi<SystemConfig<V>>,
        V: SystemCallbackObject,
    >(
        fn_identifier: FnIdentifier, // TODO: Cleanup
        auth_zone_id: &NodeId,
        acting_location: ActingLocation,
        receiver: &NodeId,
        object_key: ObjectKey,
        key: MethodKey,
        api: &mut SystemService<Y, V>,
    ) -> Result<(), RuntimeError> {
        let handle = api.kernel_lock_substate(
            receiver,
            ACCESS_RULES_FIELD_PARTITION,
            &AccessRulesField::AccessRules.into(),
            LockFlags::read_only(),
            SystemLockData::default(),
        )?;
        let access_rules: MethodAccessRulesSubstate =
            api.kernel_read_substate(handle)?.as_typed().unwrap();

        let access_rules_config = match object_key {
            ObjectKey::SELF => &access_rules.access_rules,
            ObjectKey::InnerBlueprint(blueprint_name) => {
                let child_rules = access_rules
                    .inner_blueprint_access_rules
                    .get(&blueprint_name)
                    .ok_or(RuntimeError::ModuleError(ModuleError::AuthError(
                        AuthError::InnerBlueprintDoesNotExist(blueprint_name),
                    )))?;
                child_rules
            }
        };

        Self::check_authorization_against_config(
            fn_identifier,
            auth_zone_id,
            acting_location,
            &access_rules_config,
            &key,
            api,
        )?;

        api.kernel_drop_lock(handle)?;

        Ok(())
    }

    pub fn check_authorization_against_config<
        Y: KernelApi<SystemConfig<V>>,
        V: SystemCallbackObject,
    >(
        fn_identifier: FnIdentifier, // TODO: Cleanup
        auth_zone_id: &NodeId,
        acting_location: ActingLocation,
        access_rules: &AccessRulesConfig,
        key: &MethodKey,
        api: &mut SystemService<Y, V>,
    ) -> Result<(), RuntimeError> {
        let authority = match access_rules.methods.get(key) {
            Some(entry) => &entry.authority,
            None => return Ok(()),
        };

        let result = Authorization::check_authorization_against_access_rule(
            acting_location,
            *auth_zone_id,
            access_rules,
            &rule!(require(authority.to_string())),
            api,
        )?;
        match result {
            AuthorizationCheckResult::Authorized => Ok(()),
            AuthorizationCheckResult::Failed(access_rule_stack) => Err(RuntimeError::ModuleError(
                ModuleError::AuthError(AuthError::Unauthorized(Box::new(Unauthorized {
                    access_rule_stack,
                    fn_identifier,
                }))),
            )),
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
                        &AccessRulesConfig::new(),
                        &access_rule,
                        &mut system,
                    )?;
                    match auth_result {
                        AuthorizationCheckResult::Authorized => {}
                        AuthorizationCheckResult::Failed(access_rule_stack) => {
                            return Err(RuntimeError::ModuleError(ModuleError::AuthError(
                                AuthError::Unauthorized(Box::new(Unauthorized {
                                    access_rule_stack,
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
                OBJECT_BASE_PARTITION => btreemap!(
                    AuthZoneField::AuthZone.into() => IndexedScryptoValue::from_typed(&auth_zone)
                ),
                TYPE_INFO_FIELD_PARTITION => ModuleInit::TypeInfo(TypeInfoSubstate::Object(ObjectInfo {
                    blueprint: BlueprintId::new(&RESOURCE_PACKAGE, AUTH_ZONE_BLUEPRINT),
                    global: false,
                    outer_object: None,
                    instance_schema: None,
                })).to_substates()
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
