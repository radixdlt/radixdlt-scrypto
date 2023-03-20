use super::auth_converter::convert_contextless;
use super::authorization::MethodAuthorization;
use super::Authentication;
use super::HardAuthRule;
use super::HardProofRule;
use super::HardResourceOrNonFungible;
use crate::blueprints::resource::AuthZone;
use crate::blueprints::resource::VaultInfoSubstate;
use crate::errors::*;
use crate::kernel::actor::{Actor, ActorIdentifier};
use crate::kernel::call_frame::CallFrameUpdate;
use crate::kernel::call_frame::RENodeVisibilityOrigin;
use crate::kernel::kernel_api::KernelModuleApi;
use crate::kernel::module::KernelModule;
use crate::system::kernel_modules::auth::convert;
use crate::system::node::RENodeInit;
use crate::system::node::RENodeModuleInit;
use crate::system::node_modules::access_rules::{
    AccessRulesNativePackage, FunctionAccessRulesSubstate, MethodAccessRulesSubstate,
};
use crate::system::node_modules::type_info::TypeInfoBlueprint;
use crate::system::node_modules::type_info::TypeInfoSubstate;
use crate::system::node_substates::RuntimeSubstate;
use crate::types::*;
use radix_engine_interface::api::component::ComponentStateSubstate;
use radix_engine_interface::api::node_modules::auth::*;
use radix_engine_interface::api::substate_api::LockFlags;
use radix_engine_interface::api::types::{RENodeId, SubstateOffset, VaultOffset};
use radix_engine_interface::blueprints::package::{
    PackageInfoSubstate, PACKAGE_BLUEPRINT, PACKAGE_PUBLISH_NATIVE_IDENT,
};
use radix_engine_interface::blueprints::resource::*;
use transaction::model::AuthZoneParams;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum AuthError {
    VisibilityError(RENodeId),
    Unauthorized(ActorIdentifier, MethodAuthorization),
}

#[derive(Debug, Clone)]
pub struct AuthModule {
    pub params: AuthZoneParams,
    /// Stack of auth zones, each represented by RENode ID and barrier flag
    pub auth_zone_stack: Vec<(RENodeId, bool)>,
}

impl AuthModule {
    fn is_barrier(actor: &Option<Actor>) -> bool {
        matches!(
            actor,
            Some(Actor {
                identifier: ActorIdentifier::Method(MethodIdentifier(
                    RENodeId::GlobalObject(Address::Component(..)),
                    ..
                )),
                ..
            })
        )
    }

    fn function_auth<Y: KernelModuleApi<RuntimeError>>(
        identifier: &FnIdentifier,
        api: &mut Y,
    ) -> Result<MethodAuthorization, RuntimeError> {
        let auth = if identifier.package_address.eq(&PACKAGE_PACKAGE) {
            // TODO: remove
            if identifier.blueprint_name.eq(PACKAGE_BLUEPRINT)
                && identifier.ident.eq(PACKAGE_PUBLISH_NATIVE_IDENT)
            {
                MethodAuthorization::Protected(HardAuthRule::ProofRule(HardProofRule::Require(
                    HardResourceOrNonFungible::NonFungible(AuthAddresses::system_role()),
                )))
            } else {
                MethodAuthorization::AllowAll
            }
        } else {
            let handle = api.kernel_lock_substate(
                RENodeId::GlobalObject(identifier.package_address.into()),
                NodeModuleId::SELF,
                SubstateOffset::Package(PackageOffset::FunctionAccessRules),
                LockFlags::read_only(),
            )?;
            let package_access_rules: &FunctionAccessRulesSubstate =
                api.kernel_get_substate_ref(handle)?;
            let function_key = FnKey::new(
                identifier.blueprint_name.to_string(),
                identifier.ident.to_string(),
            );
            let access_rule = package_access_rules
                .access_rules
                .get(&function_key)
                .unwrap_or(&package_access_rules.default_auth);
            convert_contextless(access_rule)
        };

        Ok(auth)
    }

    fn method_auth<Y: KernelModuleApi<RuntimeError>>(
        identifier: &MethodIdentifier,
        args: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<MethodAuthorization, RuntimeError> {
        let auth = match identifier {
            MethodIdentifier(node_id, module_id, ident)
                if matches!(
                    module_id,
                    NodeModuleId::AccessRules | NodeModuleId::AccessRules1
                ) =>
            {
                match ident.as_str() {
                    ACCESS_RULES_SET_METHOD_ACCESS_RULE_IDENT => {
                        AccessRulesNativePackage::set_method_access_rule_authorization(
                            *node_id, *module_id, args, api,
                        )?
                    }
                    ACCESS_RULES_SET_METHOD_MUTABILITY_IDENT => {
                        AccessRulesNativePackage::set_method_mutability_authorization(
                            *node_id, *module_id, args, api,
                        )?
                    }
                    ACCESS_RULES_SET_GROUP_ACCESS_RULE_IDENT => {
                        AccessRulesNativePackage::set_group_access_rule_authorization(
                            *node_id, *module_id, args, api,
                        )?
                    }
                    ACCESS_RULES_SET_GROUP_MUTABILITY_IDENT => {
                        AccessRulesNativePackage::set_group_mutability_authorization(
                            *node_id, *module_id, args, api,
                        )?
                    }
                    _ => MethodAuthorization::AllowAll,
                }
            }

            MethodIdentifier(RENodeId::Object(object_id), ..) => {
                let node_id = RENodeId::Object(*object_id);
                let (package_address, blueprint) = api.get_object_type_info(node_id)?;
                match (package_address, blueprint.as_str()) {
                    (RESOURCE_MANAGER_PACKAGE, VAULT_BLUEPRINT) => {
                        let visibility = api.kernel_get_node_visibility_origin(node_id).ok_or(
                            RuntimeError::CallFrameError(CallFrameError::RENodeNotVisible(node_id)),
                        )?;

                        let resource_address = {
                            let handle = api.kernel_lock_substate(
                                node_id,
                                NodeModuleId::SELF,
                                SubstateOffset::Vault(VaultOffset::Info),
                                LockFlags::read_only(),
                            )?;
                            let substate_ref: &VaultInfoSubstate =
                                api.kernel_get_substate_ref(handle)?;
                            let resource_address = substate_ref.resource_address;
                            api.kernel_drop_lock(handle)?;
                            resource_address
                        };

                        // TODO: Revisit what the correct abstraction is for visibility in the auth module
                        let method_key = identifier.method_key();
                        let auth = match visibility {
                            RENodeVisibilityOrigin::Normal => Self::method_authorization_stateless(
                                RENodeId::GlobalObject(resource_address.into()),
                                NodeModuleId::AccessRules1,
                                method_key,
                                api,
                            )?,
                            RENodeVisibilityOrigin::DirectAccess => {
                                let handle = api.kernel_lock_substate(
                                    RENodeId::GlobalObject(resource_address.into()),
                                    NodeModuleId::AccessRules1,
                                    SubstateOffset::AccessRules(AccessRulesOffset::AccessRules),
                                    LockFlags::read_only(),
                                )?;

                                let substate: &MethodAccessRulesSubstate =
                                    api.kernel_get_substate_ref(handle)?;

                                // TODO: Do we want to allow recaller to be able to withdraw from
                                // TODO: any visible vault?
                                let auth = if method_key.node_module_id.eq(&NodeModuleId::SELF)
                                    && (method_key.ident.eq(VAULT_RECALL_IDENT)
                                        || method_key.ident.eq(VAULT_RECALL_NON_FUNGIBLES_IDENT))
                                {
                                    let access_rule = substate.access_rules.get_group("recall");
                                    let authorization = convert_contextless(access_rule);
                                    authorization
                                } else {
                                    return Err(RuntimeError::ModuleError(ModuleError::AuthError(
                                        AuthError::VisibilityError(node_id),
                                    )));
                                };

                                api.kernel_drop_lock(handle)?;

                                auth
                            }
                        };

                        auth
                    }
                    _ => MethodAuthorization::AllowAll,
                }
            }

            MethodIdentifier(node_id, module_id, ..) => {
                let method_key = identifier.method_key();

                // TODO: Clean this up
                let auth = if matches!(
                    node_id,
                    RENodeId::GlobalObject(Address::Component(ComponentAddress::Normal(..)))
                ) && module_id.eq(&NodeModuleId::SELF)
                {
                    Self::method_authorization_stateful(
                        *node_id,
                        NodeModuleId::AccessRules,
                        method_key,
                        api,
                    )?
                } else {
                    Self::method_authorization_stateless(
                        *node_id,
                        NodeModuleId::AccessRules,
                        method_key,
                        api,
                    )?
                };

                auth
            }
        };

        Ok(auth)
    }

    fn method_authorization_stateful<Y: KernelModuleApi<RuntimeError>>(
        receiver: RENodeId,
        module_id: NodeModuleId,
        key: MethodKey,
        api: &mut Y,
    ) -> Result<MethodAuthorization, RuntimeError> {
        let (blueprint_schema, index) = {
            let type_info = TypeInfoBlueprint::get_type(receiver, api)?;
            let (package_address, blueprint_ident) = match type_info {
                TypeInfoSubstate::Object {
                    package_address,
                    blueprint_name,
                    ..
                } => (package_address, blueprint_name),
                TypeInfoSubstate::KeyValueStore(..) => {
                    return Err(RuntimeError::SystemError(SystemError::NotAnObject))
                }
            };

            let handle = api.kernel_lock_substate(
                RENodeId::GlobalObject(package_address.into()),
                NodeModuleId::SELF,
                SubstateOffset::Package(PackageOffset::Info),
                LockFlags::read_only(),
            )?;
            let package: &PackageInfoSubstate = api.kernel_get_substate_ref(handle)?;
            let schema = package
                .schema
                .blueprints
                .get(&blueprint_ident)
                .expect("Blueprint schema not found")
                .clone();
            let index = match schema.substates.get(0) {
                Some(index) => index.clone(),
                None => {
                    return Self::method_authorization_stateless(receiver, module_id, key, api);
                }
            };

            api.kernel_drop_lock(handle)?;
            (schema, index)
        };

        let state = {
            let offset = SubstateOffset::Component(ComponentOffset::State0);
            let handle = api.kernel_lock_substate(
                receiver,
                NodeModuleId::SELF,
                offset,
                LockFlags::read_only(),
            )?;
            let state: &ComponentStateSubstate = api.kernel_get_substate_ref(handle)?;
            let state = IndexedScryptoValue::from_scrypto_value(state.0.clone());
            api.kernel_drop_lock(handle)?;
            state
        };

        let handle = api.kernel_lock_substate(
            receiver,
            module_id,
            SubstateOffset::AccessRules(AccessRulesOffset::AccessRules),
            LockFlags::read_only(),
        )?;
        let access_rules: &MethodAccessRulesSubstate = api.kernel_get_substate_ref(handle)?;

        let method_auth = access_rules.access_rules.get(&key);
        let authorization = convert(&blueprint_schema.schema, index, &state, method_auth);

        api.kernel_drop_lock(handle)?;

        Ok(authorization)
    }

    fn method_authorization_stateless<Y: KernelModuleApi<RuntimeError>>(
        receiver: RENodeId,
        module_id: NodeModuleId,
        key: MethodKey,
        api: &mut Y,
    ) -> Result<MethodAuthorization, RuntimeError> {
        let handle = api.kernel_lock_substate(
            receiver,
            module_id,
            SubstateOffset::AccessRules(AccessRulesOffset::AccessRules),
            LockFlags::read_only(),
        )?;
        let access_rules: &MethodAccessRulesSubstate = api.kernel_get_substate_ref(handle)?;

        let method_auth = access_rules.access_rules.get(&key);

        // TODO: Remove
        let authorization = convert_contextless(method_auth);

        api.kernel_drop_lock(handle)?;

        Ok(authorization)
    }

    pub fn auth_zones_for_authorization(&self, callee: &Option<Actor>) -> Vec<RENodeId> {
        let mut barrier_crossing_allowed = !Self::is_barrier(callee);

        let mut auth_zones = Vec::<RENodeId>::new();
        for (node_id, is_barrier) in self.auth_zone_stack.iter().cloned().rev() {
            auth_zones.push(node_id);

            if is_barrier {
                if barrier_crossing_allowed {
                    barrier_crossing_allowed = false;
                } else {
                    break;
                }
            }
        }

        auth_zones
    }

    pub fn read_auth_zones<Y: KernelModuleApi<RuntimeError>>(
        auth_zone_ids: Vec<RENodeId>,
        api: &mut Y,
    ) -> Result<Vec<AuthZone>, RuntimeError> {
        let mut auth_zones = Vec::<AuthZone>::new();
        for auth_zone_id in auth_zone_ids {
            loop {
                let handle = api.kernel_lock_substate(
                    auth_zone_id,
                    NodeModuleId::SELF,
                    SubstateOffset::AuthZone(AuthZoneOffset::AuthZone),
                    LockFlags::read_only(),
                )?;
                let auth_zone: &AuthZone = api.kernel_get_substate_ref(handle)?;
                auth_zones.push(auth_zone.clone());
                api.kernel_drop_lock(handle)?;
            }
        }
        Ok(auth_zones)
    }
}

impl KernelModule for AuthModule {
    fn before_push_frame<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        callee: &Actor,
        _call_frame_update: &mut CallFrameUpdate,
        args: &IndexedScryptoValue,
    ) -> Result<(), RuntimeError> {
        let method_auth = match &callee.identifier {
            ActorIdentifier::Method(method) => Self::method_auth(method, &args, api)?,
            ActorIdentifier::Function(function) => Self::function_auth(function, api)?,
        };

        let auth_zone_ids = api
            .kernel_get_module_state()
            .auth
            .auth_zones_for_authorization(&Some(callee.clone()));
        let auth_zones = Self::read_auth_zones(auth_zone_ids, api)?;

        if !Authentication::verify_method_auth(&method_auth, &auth_zones, api)? {
            return Err(RuntimeError::ModuleError(ModuleError::AuthError(
                AuthError::Unauthorized(callee.identifier.clone(), method_auth),
            )));
        }

        Ok(())
    }

    fn on_execution_start<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        _caller: &Option<Actor>,
    ) -> Result<(), RuntimeError> {
        let actor = api.kernel_get_current_actor();

        // Add Package Actor Auth
        let mut virtual_non_fungibles_non_extending = BTreeSet::new();
        if let Some(actor) = &actor {
            let package_address = actor.fn_identifier.package_address();
            let id = scrypto_encode(&package_address).unwrap();
            let non_fungible_global_id =
                NonFungibleGlobalId::new(PACKAGE_TOKEN, NonFungibleLocalId::bytes(id).unwrap());
            virtual_non_fungibles_non_extending.insert(non_fungible_global_id);
        }

        // Prepare a new auth zone
        let is_barrier = Self::is_barrier(&actor);
        let auth_module = &api.kernel_get_module_state().auth;
        let auth_zone = if auth_module.auth_zone_stack.is_empty() {
            let virtual_resources = auth_module.params.virtual_resources.clone();
            let virtual_non_fungibles = auth_module.params.initial_proofs.clone();
            AuthZone::new(
                vec![],
                virtual_resources,
                virtual_non_fungibles,
                virtual_non_fungibles_non_extending,
            )
        } else {
            AuthZone::new(
                vec![],
                BTreeSet::new(),
                BTreeSet::new(),
                virtual_non_fungibles_non_extending,
            )
        };

        // Create node
        let auth_zone_node_id = api.kernel_allocate_node_id(RENodeType::Object)?;
        api.kernel_create_node(
            auth_zone_node_id,
            RENodeInit::Object(btreemap!(
                SubstateOffset::AuthZone(AuthZoneOffset::AuthZone) => RuntimeSubstate::AuthZone(auth_zone)
            )),
            btreemap!(
                NodeModuleId::TypeInfo => RENodeModuleInit::TypeInfo(TypeInfoSubstate::Object {
                    package_address: RESOURCE_MANAGER_PACKAGE,
                    blueprint_name: AUTH_ZONE_BLUEPRINT.to_owned(),
                    global: false
                })
            ),
        )?;

        api.kernel_get_module_state()
            .auth
            .auth_zone_stack
            .push((auth_zone_node_id, is_barrier));

        Ok(())
    }

    fn on_execution_finish<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        _caller: &Option<Actor>,
        _update: &CallFrameUpdate,
    ) -> Result<(), RuntimeError> {
        let auth_zone = api
            .kernel_get_module_state()
            .auth
            .auth_zone_stack
            .pop()
            .expect("Auth zone stack is broken");

        api.kernel_drop_node(auth_zone.0)?;

        // Proofs in auth zone will be re-owned by the frame and auto dropped.

        Ok(())
    }
}
