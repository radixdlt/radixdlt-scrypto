use super::auth_converter::convert_contextless;
use super::authorization::MethodAuthorization;
use super::Authentication;
use super::HardAuthRule;
use super::HardProofRule;
use super::HardResourceOrNonFungible;
use crate::blueprints::resource::AuthZone;
use crate::errors::*;
use crate::kernel::actor::{Actor, MethodActor};
use crate::kernel::call_frame::Message;
use crate::kernel::call_frame::RefType;
use crate::kernel::kernel_api::KernelApi;
use crate::kernel::kernel_callback_api::KernelCallbackObject;
use crate::system::module::SystemModule;
use crate::system::node_init::ModuleInit;
use crate::system::node_modules::access_rules::{
    AccessRulesNativePackage, FunctionAccessRulesSubstate, MethodAccessRulesSubstate,
};
use crate::system::node_modules::type_info::TypeInfoBlueprint;
use crate::system::node_modules::type_info::TypeInfoSubstate;
use crate::system::system::SystemService;
use crate::system::system_callback::SystemConfig;
use crate::system::system_callback_api::SystemCallbackObject;
use crate::system::system_modules::auth::convert;
use crate::types::*;
use radix_engine_interface::api::component::ComponentStateSubstate;
use radix_engine_interface::api::field_lock_api::LockFlags;
use radix_engine_interface::api::node_modules::auth::*;
use radix_engine_interface::api::{ClientObjectApi, ObjectModuleId};
use radix_engine_interface::blueprints::package::{
    PackageInfoSubstate, PACKAGE_BLUEPRINT, PACKAGE_PUBLISH_NATIVE_IDENT,
};
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::blueprints::transaction_processor::TRANSACTION_PROCESSOR_BLUEPRINT;
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
    pub authorization: MethodAuthorization,
}

#[derive(Debug, Clone)]
pub struct AuthModule {
    pub params: AuthZoneParams,
    /// Stack of auth zones
    pub auth_zone_stack: Vec<NodeId>,
}

impl AuthModule {
    fn is_barrier(actor: &Actor) -> bool {
        // FIXME update the rule to be consistent with internal design
        match actor {
            Actor::Method(MethodActor { node_id, .. }) => {
                node_id.is_global_component() || node_id.is_global_resource()
            }
            Actor::Function { .. } => false,
            Actor::VirtualLazyLoad { .. } => false,
        }
    }

    fn is_transaction_processor(actor: Option<&Actor>) -> bool {
        match actor {
            Some(actor) => {
                let blueprint = actor.blueprint();
                blueprint.eq(&Blueprint::new(
                    &TRANSACTION_PROCESSOR_PACKAGE,
                    TRANSACTION_PROCESSOR_BLUEPRINT,
                ))
            }
            None => false,
        }
    }

    fn function_auth<Y: KernelApi<M>, M: KernelCallbackObject>(
        blueprint: &Blueprint,
        ident: &str,
        api: &mut Y,
    ) -> Result<MethodAuthorization, RuntimeError> {
        let auth = if blueprint.package_address.eq(&PACKAGE_PACKAGE) {
            // TODO: remove
            if blueprint.blueprint_name.eq(PACKAGE_BLUEPRINT)
                && ident.eq(PACKAGE_PUBLISH_NATIVE_IDENT)
            {
                MethodAuthorization::Protected(HardAuthRule::ProofRule(HardProofRule::Require(
                    HardResourceOrNonFungible::NonFungible(AuthAddresses::system_role()),
                )))
            } else {
                MethodAuthorization::AllowAll
            }
        } else {
            let handle = api.kernel_lock_substate(
                blueprint.package_address.as_node_id(),
                OBJECT_BASE_MODULE,
                &PackageOffset::FunctionAccessRules.into(),
                LockFlags::read_only(),
                M::LockData::default(),
            )?;
            let package_access_rules: FunctionAccessRulesSubstate =
                api.kernel_read_substate(handle)?.as_typed().unwrap();
            let function_key = FnKey::new(blueprint.blueprint_name.to_string(), ident.to_string());
            let access_rule = package_access_rules
                .access_rules
                .get(&function_key)
                .unwrap_or(&package_access_rules.default_auth);
            convert_contextless(access_rule)
        };

        Ok(auth)
    }

    fn method_auth<Y: KernelApi<SystemConfig<V>>, V: SystemCallbackObject>(
        node_id: &NodeId,
        module_id: &ObjectModuleId,
        ident: &str,
        args: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<Vec<MethodAuthorization>, RuntimeError> {
        let auths = match (node_id, module_id, ident) {
            (node_id, module_id, ident) if matches!(module_id, ObjectModuleId::AccessRules) => {
                vec![AccessRulesNativePackage::authorization(
                    node_id, ident, args, api,
                )?]
            }
            (node_id, module_id, ..) => {
                let method_key = MethodKey::new(*module_id, ident);

                let info = {
                    let mut system = SystemService::new(api);
                    system.get_object_info(node_id)?
                };

                let mut auths = Vec::new();

                if let Some(parent) = info.outer_object {
                    let (ref_type, _) =
                        api.kernel_get_node_info(node_id)
                            .ok_or(RuntimeError::ModuleError(ModuleError::AuthError(
                                AuthError::VisibilityError(node_id.clone()),
                            )))?;
                    let method_key = MethodKey::new(*module_id, ident);
                    let auth = Self::method_authorization_stateless(
                        ref_type,
                        parent.as_node_id(),
                        ObjectKey::ChildBlueprint(info.blueprint.blueprint_name),
                        method_key,
                        api,
                    )?;

                    auths.push(auth);
                }

                if info.global {
                    // TODO: Clean this up
                    let auth = if module_id.eq(&ObjectModuleId::SELF) {
                        Self::method_authorization_stateful(
                            &node_id,
                            ObjectKey::SELF,
                            method_key,
                            api,
                        )?
                    } else {
                        Self::method_authorization_stateless(
                            RefType::Normal,
                            &node_id,
                            ObjectKey::SELF,
                            method_key,
                            api,
                        )?
                    };
                    auths.push(auth);
                }

                auths
            }
        };

        Ok(auths)
    }

    fn method_authorization_stateful<Y: KernelApi<M>, M: KernelCallbackObject>(
        receiver: &NodeId,
        object_key: ObjectKey,
        method_key: MethodKey,
        api: &mut Y,
    ) -> Result<MethodAuthorization, RuntimeError> {
        let (blueprint_schema, index) = {
            let type_info = TypeInfoBlueprint::get_type(receiver, api)?;
            let blueprint = match type_info {
                TypeInfoSubstate::Object(ObjectInfo { blueprint, .. }) => blueprint,
                TypeInfoSubstate::KeyValueStore(..)
                | TypeInfoSubstate::SortedIndex
                | TypeInfoSubstate::Index => {
                    return Err(RuntimeError::SystemError(SystemError::NotAnObject))
                }
            };

            let handle = api.kernel_lock_substate(
                blueprint.package_address.as_node_id(),
                OBJECT_BASE_MODULE,
                &PackageOffset::Info.into(),
                LockFlags::read_only(),
                M::LockData::default(),
            )?;
            let package: PackageInfoSubstate =
                api.kernel_read_substate(handle)?.as_typed().unwrap();
            let schema = package
                .schema
                .blueprints
                .get(&blueprint.blueprint_name)
                .expect("Blueprint schema not found")
                .clone();
            let index = match &schema.tuple_module {
                Some((_, index)) => index.get(0).unwrap().clone(),
                None => {
                    return Self::method_authorization_stateless(
                        RefType::Normal,
                        receiver,
                        object_key,
                        method_key,
                        api,
                    );
                }
            };

            api.kernel_drop_lock(handle)?;
            (schema, index)
        };

        // TODO: Remove
        let state = {
            let substate_key = ComponentOffset::State0.into();
            let handle = api.kernel_lock_substate(
                receiver,
                OBJECT_BASE_MODULE,
                &substate_key,
                LockFlags::read_only(),
                M::LockData::default(),
            )?;
            let state: ComponentStateSubstate =
                api.kernel_read_substate(handle)?.as_typed().unwrap();
            let state = IndexedScryptoValue::from_scrypto_value(state.0.clone());
            api.kernel_drop_lock(handle)?;
            state
        };

        let handle = api.kernel_lock_substate(
            receiver,
            ACCESS_RULES_BASE_MODULE,
            &AccessRulesOffset::AccessRules.into(),
            LockFlags::read_only(),
            M::LockData::default(),
        )?;
        let access_rules: MethodAccessRulesSubstate =
            api.kernel_read_substate(handle)?.as_typed().unwrap();

        let method_auth = access_rules
            .access_rules
            .get_access_rule(false, &method_key);
        let authorization = convert(&blueprint_schema.schema, index, &state, &method_auth);

        api.kernel_drop_lock(handle)?;

        Ok(authorization)
    }

    fn method_authorization_stateless<Y: KernelApi<M>, M: KernelCallbackObject>(
        ref_type: RefType,
        receiver: &NodeId,
        object_key: ObjectKey,
        key: MethodKey,
        api: &mut Y,
    ) -> Result<MethodAuthorization, RuntimeError> {
        let handle = api.kernel_lock_substate(
            receiver,
            ACCESS_RULES_BASE_MODULE,
            &AccessRulesOffset::AccessRules.into(),
            LockFlags::read_only(),
            M::LockData::default(),
        )?;
        let access_rules: MethodAccessRulesSubstate =
            api.kernel_read_substate(handle)?.as_typed().unwrap();

        let is_direct_access = matches!(ref_type, RefType::DirectAccess);

        let method_auth = match object_key {
            ObjectKey::SELF => access_rules
                .access_rules
                .get_access_rule(is_direct_access, &key),
            ObjectKey::ChildBlueprint(blueprint_name) => {
                let child_rules = access_rules
                    .child_blueprint_rules
                    .get(&blueprint_name)
                    .ok_or(RuntimeError::ModuleError(ModuleError::AuthError(
                        AuthError::InnerBlueprintDoesNotExist(blueprint_name),
                    )))?;
                child_rules.get_access_rule(is_direct_access, &key)
            }
        };

        // TODO: Remove
        let authorization = convert_contextless(&method_auth);

        api.kernel_drop_lock(handle)?;

        Ok(authorization)
    }

    pub fn last_auth_zone(&self) -> NodeId {
        self.auth_zone_stack
            .last()
            .cloned()
            .expect("Missing auth zone")
    }
}

impl<V: SystemCallbackObject> SystemModule<SystemConfig<V>> for AuthModule {
    fn on_init<Y: KernelApi<SystemConfig<V>>>(api: &mut Y) -> Result<(), RuntimeError> {
        // Create sentinel node
        Self::on_execution_start(api)
    }

    fn on_teardown<Y: KernelApi<SystemConfig<V>>>(api: &mut Y) -> Result<(), RuntimeError> {
        // Destroy sentinel node
        Self::on_execution_finish(api, &Message::default())
    }

    fn before_push_frame<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        callee: &Actor,
        _call_frame_update: &mut Message,
        args: &IndexedScryptoValue,
    ) -> Result<(), RuntimeError> {
        // Decide `authorization`, `barrier_crossing_allowed`, and `tip_auth_zone_id`
        let authorizations = match &callee {
            Actor::Method(MethodActor {
                node_id,
                module_id,
                ident,
                ..
            }) => Self::method_auth(node_id, module_id, ident.as_str(), &args, api)?,
            Actor::Function { blueprint, ident } => {
                vec![Self::function_auth(blueprint, ident.as_str(), api)?]
            }
            Actor::VirtualLazyLoad { .. } => return Ok(()),
        };
        let barrier_crossings_required = 0;
        let barrier_crossings_allowed = if Self::is_barrier(callee) { 0 } else { 1 };
        let auth_zone_id = api.kernel_get_system().modules.auth.last_auth_zone();

        let mut system = SystemService::new(api);

        // Authenticate
        for authorization in authorizations {
            if !Authentication::verify_method_auth(
                barrier_crossings_required,
                barrier_crossings_allowed,
                auth_zone_id,
                &authorization,
                &mut system,
            )? {
                return Err(RuntimeError::ModuleError(ModuleError::AuthError(
                    AuthError::Unauthorized(Box::new(Unauthorized { authorization })),
                )));
            }
        }

        Ok(())
    }

    fn on_execution_start<Y: KernelApi<SystemConfig<V>>>(api: &mut Y) -> Result<(), RuntimeError> {
        let actor = api.kernel_get_system_state().current;

        // Add Global Object and Package Actor Auth
        let mut virtual_non_fungibles_non_extending = BTreeSet::new();
        if let Some(actor) = actor {
            let package_address = actor.package_address();
            let id = scrypto_encode(&package_address).unwrap();
            let non_fungible_global_id = NonFungibleGlobalId::new(
                PACKAGE_VIRTUAL_BADGE,
                NonFungibleLocalId::bytes(id).unwrap(),
            );
            virtual_non_fungibles_non_extending.insert(non_fungible_global_id);

            if let Some(method) = actor.try_as_method() {
                if let Some(address) = method.global_address {
                    let id = scrypto_encode(&address).unwrap();
                    let non_fungible_global_id = NonFungibleGlobalId::new(
                        GLOBAL_ACTOR_VIRTUAL_BADGE,
                        NonFungibleLocalId::bytes(id).unwrap(),
                    );
                    virtual_non_fungibles_non_extending.insert(non_fungible_global_id);
                }
            }
        }

        // Prepare a new auth zone
        let is_barrier = if let Some(actor) = actor {
            Self::is_barrier(actor)
        } else {
            false
        };
        let is_transaction_processor = Self::is_transaction_processor(actor);
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
            is_barrier,
            parent,
        );

        // Create node
        let auth_zone_node_id =
            api.kernel_allocate_node_id(EntityType::InternalGenericComponent)?;
        api.kernel_create_node(
            auth_zone_node_id,
            btreemap!(
                OBJECT_BASE_MODULE => btreemap!(
                    AuthZoneOffset::AuthZone.into() => IndexedScryptoValue::from_typed(&auth_zone)
                ),
                TYPE_INFO_BASE_MODULE => ModuleInit::TypeInfo(TypeInfoSubstate::Object(ObjectInfo {
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
        _update: &Message,
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
