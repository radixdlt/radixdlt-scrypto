use super::Authentication;
use crate::blueprints::resource::AuthZone;
use crate::errors::*;
use crate::kernel::actor::{Actor, MethodActor};
use crate::kernel::call_frame::{CallFrameUpdate, RENodeLocation};
use crate::kernel::call_frame::RefType;
use crate::kernel::kernel_api::{KernelApi, KernelInternalApi, KernelInvokeApi, KernelSubstateApi};
use crate::kernel::kernel_callback_api::KernelCallbackObject;
use crate::system::module::SystemModule;
use crate::system::node_init::ModuleInit;
use crate::system::node_modules::access_rules::{
    AccessRulesNativePackage, FunctionAccessRulesSubstate, MethodAccessRulesSubstate,
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
use radix_engine_interface::blueprints::resource::AccessRule::DenyAll;
use radix_engine_interface::types::*;
use transaction::model::AuthZoneParams;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum AuthError {
    VisibilityError(NodeId),
    Unauthorized(Box<Unauthorized>),
    InnerBlueprintDoesNotExist(String),
}
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct Unauthorized {
    pub access_rule: AccessRule,
    pub fn_identifier: FnIdentifier,
}

#[derive(Debug, Clone)]
pub struct AuthModule {
    pub params: AuthZoneParams,
    /// Stack of auth zones
    pub auth_zone_stack: Vec<NodeId>,
}

enum AuthorizationCheckResult {
    Authorized,
    Failed(AccessRule),
}

impl AuthModule {
    fn function_auth<Y: KernelApi<SystemConfig<V>>, V: SystemCallbackObject>(
        blueprint: &Blueprint,
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
                let access_rule = AccessRulesNativePackage::authorization(
                    &node_id, ident, args, api,
                )?;

                if !Authentication::verify_method_auth(
                    acting_location,
                    *auth_zone_id,
                    &AccessRulesConfig::new(),
                    &access_rule,
                    api,
                )? {
                    return Err(RuntimeError::ModuleError(ModuleError::AuthError(
                        AuthError::Unauthorized(Box::new(Unauthorized {
                            access_rule,
                            fn_identifier: actor.fn_identifier(),
                        })),
                    )));
                }
            }
            (node_id, module_id, ..) => {
                let method_key = MethodKey::new(module_id, ident);

                let info = api.get_object_info(&node_id)?;

                if let Some(parent) = info.outer_object {
                    let (ref_type, _) =
                        api.kernel_get_node_info(&node_id)
                            .ok_or(RuntimeError::ModuleError(ModuleError::AuthError(
                                AuthError::VisibilityError(node_id),
                            )))?;
                    let method_key = MethodKey::new(module_id, ident);
                    Self::check_authorization_against_access_rules(
                        actor.fn_identifier(),
                        auth_zone_id,
                        acting_location,
                        ref_type,
                        parent.as_node_id(),
                        ObjectKey::ChildBlueprint(info.blueprint.blueprint_name),
                        method_key,
                        api,
                    )?;
                }

                if info.global {
                    Self::check_authorization_against_access_rules(
                        actor.fn_identifier(),
                        auth_zone_id,
                        acting_location,
                        RefType::Normal,
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

    fn check_authorization_against_access_rules<Y: KernelApi<SystemConfig<V>>, V: SystemCallbackObject>(
        fn_identifier: FnIdentifier, // TODO: Cleanup
        auth_zone_id: &NodeId,
        acting_location: ActingLocation,
        ref_type: RefType,
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

        let is_direct_access = matches!(ref_type, RefType::DirectAccess);

        let access_rules_config = match object_key {
            ObjectKey::SELF => {
                &access_rules.access_rules
            },
            ObjectKey::ChildBlueprint(blueprint_name) => {
                let child_rules = access_rules
                    .child_blueprint_rules
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
            is_direct_access,
            &key,
            api,
        )?;

        api.kernel_drop_lock(handle)?;

        Ok(())
    }

    pub fn check_authorization_against_config<Y: KernelApi<SystemConfig<V>>, V: SystemCallbackObject>(
        fn_identifier: FnIdentifier, // TODO: Cleanup
        auth_zone_id: &NodeId,
        acting_location: ActingLocation,
        access_rules: &AccessRulesConfig,
        is_direct_access: bool,
        key: &MethodKey,
        api: &mut SystemService<Y, V>,
    ) -> Result<(), RuntimeError> {
        let auth = if is_direct_access {
            &access_rules.direct_methods
        } else {
            &access_rules.methods
        };
        let method_entry = match auth.get(key) {
            None => {
                return Ok(());
            },
            Some(entry) => entry,
        };

        let mut failed_access_rule = None;

        for authority in &method_entry.authorities {
            let result = Self::check_authorization_against_authority(
                auth_zone_id,
                acting_location,
                access_rules,
                &AccessRule::authority(authority),
                api,
            )?;
            match result {
                AuthorizationCheckResult::Authorized => return Ok(()),
                AuthorizationCheckResult::Failed(rule) => {
                    failed_access_rule.insert(rule);
                }
            }
        }

        if let Some(failed_access_rule) = failed_access_rule {
            return Err(RuntimeError::ModuleError(ModuleError::AuthError(
                AuthError::Unauthorized(Box::new(Unauthorized {
                    access_rule: failed_access_rule,
                    fn_identifier,
                })),
            )));
        }

        Ok(())
    }

    fn check_authorization_against_authority<Y: KernelApi<SystemConfig<V>>, V: SystemCallbackObject>(
        auth_zone_id: &NodeId,
        acting_location: ActingLocation,
        access_rules: &AccessRulesConfig,
        access_rule: &AccessRule,
        api: &mut SystemService<Y, V>,
    ) -> Result<AuthorizationCheckResult, RuntimeError> {
        if Authentication::verify_method_auth(
            acting_location,
            *auth_zone_id,
            access_rules,
            access_rule,
            api,
        )? {
            Ok(AuthorizationCheckResult::Authorized)
        } else {
            Ok(AuthorizationCheckResult::Failed(access_rule.clone()))
        }
    }

    pub fn last_auth_zone(&self) -> NodeId {
        self.auth_zone_stack
            .last()
            .cloned()
            .expect("Missing auth zone")
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
        let auth_zone_id = api.kernel_get_system().modules.auth.last_auth_zone();

        let mut system = SystemService::new(api);

        // Decide `authorization`, `barrier_crossing_allowed`, and `tip_auth_zone_id`
        match &callee {
            Actor::Method(actor) => {
                Self::check_method_authorization(
                    &auth_zone_id,
                    actor,
                    &args,
                    &mut system,
                )?;
            },
            Actor::Function { blueprint, ident } => {
                let access_rule = Self::function_auth(blueprint, ident.as_str(), &mut system)?;
                let acting_location = ActingLocation::AtBarrier;

                // Verify authorization
                if !Authentication::verify_method_auth(
                    acting_location,
                    auth_zone_id,
                    &AccessRulesConfig::new(),
                    &access_rule,
                    &mut system,
                )? {
                    return Err(RuntimeError::ModuleError(ModuleError::AuthError(
                        AuthError::Unauthorized(Box::new(Unauthorized {
                            access_rule,
                            fn_identifier: callee.fn_identifier(),
                        })),
                    )));
                }
            }
            Actor::VirtualLazyLoad { .. } | Actor::Root => {}
        };

        Ok(())
    }
}

impl<V: SystemCallbackObject> SystemModule<SystemConfig<V>> for AuthModule {
    fn on_init<Y: KernelApi<SystemConfig<V>>>(api: &mut Y) -> Result<(), RuntimeError> {
        // Create sentinel node
        Self::on_execution_start(api)
    }

    fn on_teardown<Y: KernelApi<SystemConfig<V>>>(api: &mut Y) -> Result<(), RuntimeError> {
        // Destroy sentinel node
        Self::on_execution_finish(api, &CallFrameUpdate::empty())
    }

    fn before_push_frame<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        callee: &Actor,
        _call_frame_update: &mut CallFrameUpdate,
        args: &IndexedScryptoValue,
    ) -> Result<(), RuntimeError> {
        Self::check_authorization(callee, args, api)
    }

    fn on_execution_start<Y: KernelApi<SystemConfig<V>>>(api: &mut Y) -> Result<(), RuntimeError> {
        let actor = api.kernel_get_system_state().current;

        // Add Global Object and Package Actor Auth
        let virtual_non_fungibles_non_extending = actor.get_virtual_non_extending_proofs();
        let virtual_non_fungibles_non_extending_barrier =
            actor.get_virtual_non_extending_barrier_proofs();

        // Prepare a new auth zone
        let is_barrier = actor.is_barrier();
        let is_transaction_processor = actor.is_transaction_processor();
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
                    blueprint: Blueprint::new(&RESOURCE_PACKAGE, AUTH_ZONE_BLUEPRINT),
                    global: false,
                    outer_object: None,
                    instance_schema: None,
                })).to_substates()
            ),
        )?;

        api.kernel_get_system()
            .modules
            .auth
            .auth_zone_stack
            .push(auth_zone_node_id);

        Ok(())
    }

    fn on_execution_finish<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        _update: &CallFrameUpdate,
    ) -> Result<(), RuntimeError> {
        let auth_zone = api
            .kernel_get_system()
            .modules
            .auth
            .auth_zone_stack
            .pop()
            .expect("Auth zone stack is broken");

        api.kernel_drop_node(&auth_zone)?;

        // Proofs in auth zone will be re-owned by the frame and auto dropped.

        Ok(())
    }
}
