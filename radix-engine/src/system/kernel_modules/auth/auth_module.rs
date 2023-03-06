use super::auth_converter::convert_contextless;
use super::method_authorization::MethodAuthorization;
use super::HardAuthRule;
use super::HardProofRule;
use super::HardResourceOrNonFungible;
use crate::blueprints::resource::VaultInfoSubstate;
use crate::errors::*;
use crate::kernel::actor::{Actor, ActorIdentifier};
use crate::kernel::call_frame::CallFrameUpdate;
use crate::kernel::call_frame::RENodeVisibilityOrigin;
use crate::kernel::kernel_api::KernelModuleApi;
use crate::kernel::module::KernelModule;
use crate::system::kernel_modules::auth::convert;
use crate::system::node::{RENodeInit, RENodeModuleInit};
use crate::system::node_modules::access_rules::{
    AccessRulesNativePackage, AuthZoneStackSubstate, FunctionAccessRulesSubstate,
    MethodAccessRulesSubstate,
};
use crate::system::node_modules::type_info::{TypeInfoBlueprint, TypeInfoSubstate};
use crate::types::*;
use radix_engine_interface::api::component::ComponentStateSubstate;
use radix_engine_interface::api::node_modules::auth::*;
use radix_engine_interface::api::package::{
    PackageInfoSubstate, PACKAGE_LOADER_BLUEPRINT, PACKAGE_LOADER_PUBLISH_NATIVE_IDENT,
};
use radix_engine_interface::api::substate_api::LockFlags;
use radix_engine_interface::api::types::{
    AuthZoneStackOffset, RENodeId, SubstateOffset, VaultOffset,
};
use radix_engine_interface::blueprints::resource::*;
use transaction::model::AuthZoneParams;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum AuthError {
    VisibilityError(RENodeId),
    Unauthorized(Option<ActorIdentifier>, MethodAuthorization),
}

#[derive(Debug, Clone)]
pub struct AuthModule {
    pub params: AuthZoneParams,
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
        let auth = if identifier.package_address.eq(&PACKAGE_LOADER) {
            if identifier.blueprint_name.eq(PACKAGE_LOADER_BLUEPRINT)
                && identifier.ident.eq(PACKAGE_LOADER_PUBLISH_NATIVE_IDENT)
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
                NodeModuleId::FunctionAccessRules,
                SubstateOffset::PackageAccessRules,
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
        args: &ScryptoValue,
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
            MethodIdentifier(RENodeId::TransactionRuntime | RENodeId::AuthZoneStack, ..) => {
                MethodAuthorization::AllowAll
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
                            RENodeVisibilityOrigin::Normal => {
                                Self::method_authorization_contextless(
                                    RENodeId::GlobalObject(resource_address.into()),
                                    NodeModuleId::AccessRules1,
                                    method_key,
                                    api,
                                )?
                            }
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
                    Self::normal_component_method_authorization(
                        *node_id,
                        NodeModuleId::AccessRules,
                        method_key,
                        api,
                    )?
                } else {
                    Self::method_authorization_contextless(
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

    fn normal_component_method_authorization<Y: KernelModuleApi<RuntimeError>>(
        receiver: RENodeId,
        module_id: NodeModuleId,
        key: MethodKey,
        api: &mut Y,
    ) -> Result<MethodAuthorization, RuntimeError> {
        let (blueprint_schema, index) = {
            let (package_address, blueprint_ident) = TypeInfoBlueprint::get_type(receiver, api)?;

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
            let index = schema
                .substates
                .get(&0)
                .expect("Substate schema [offset: 0] not found")
                .clone();
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
            let state = IndexedScryptoValue::from_slice(&state.raw)
                .expect("Failed to decode component state");
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

    pub fn method_authorization_contextless<Y: KernelModuleApi<RuntimeError>>(
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
}

impl KernelModule for AuthModule {
    fn on_init<Y: KernelModuleApi<RuntimeError>>(api: &mut Y) -> Result<(), RuntimeError> {
        let auth_zone_params = api.kernel_get_module_state().auth.params.clone();
        let auth_zone = AuthZoneStackSubstate::new(
            vec![],
            auth_zone_params.virtualizable_proofs_resource_addresses,
            auth_zone_params.initial_proofs.into_iter().collect(),
        );
        let node_id = api.kernel_allocate_node_id(RENodeType::AuthZoneStack)?;
        api.kernel_create_node(
            node_id,
            RENodeInit::AuthZoneStack(auth_zone),
            btreemap!(
                NodeModuleId::TypeInfo => RENodeModuleInit::TypeInfo(TypeInfoSubstate {
                        package_address: AUTH_ZONE_PACKAGE,
                        blueprint_name: AUTH_ZONE_BLUEPRINT.to_string(),
                        global: false,
                    })
            ),
        )?;
        Ok(())
    }

    fn on_teardown<Y: KernelModuleApi<RuntimeError>>(api: &mut Y) -> Result<(), RuntimeError> {
        // Proofs in authzone will get auto-dropped when frame exits
        api.kernel_drop_node(RENodeId::AuthZoneStack)?;

        Ok(())
    }

    fn before_push_frame<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        next_actor: &Option<Actor>,
        call_frame_update: &mut CallFrameUpdate,
        args: &ScryptoValue,
    ) -> Result<(), RuntimeError> {
        if matches!(
            next_actor,
            Some(Actor {
                fn_identifier: FnIdentifier {
                    package_address: AUTH_ZONE_PACKAGE,
                    ..
                },
                ..
            })
        ) {
            return Ok(());
        }

        let method_auth = if let Some(actor) = next_actor {
            match &actor.identifier {
                ActorIdentifier::Method(method) => Self::method_auth(method, &args, api)?,
                ActorIdentifier::Function(function) => Self::function_auth(function, api)?,
            }
        } else {
            MethodAuthorization::AllowAll
        };

        let handle = api.kernel_lock_substate(
            RENodeId::AuthZoneStack,
            NodeModuleId::SELF,
            SubstateOffset::AuthZoneStack(AuthZoneStackOffset::AuthZoneStack),
            LockFlags::read_only(),
        )?;
        let substate_ref: &AuthZoneStackSubstate = api.kernel_get_substate_ref(handle)?;
        let auth_zone_stack = substate_ref.clone();
        let is_barrier = Self::is_barrier(next_actor);

        // Authorization check
        if !auth_zone_stack.check_auth(is_barrier, &method_auth, api)? {
            return Err(RuntimeError::ModuleError(ModuleError::AuthError(
                AuthError::Unauthorized(
                    next_actor.as_ref().map(|a| a.identifier.clone()),
                    method_auth,
                ),
            )));
        }

        api.kernel_drop_lock(handle)?;

        //  Additional ref copying

        call_frame_update
            .node_refs_to_copy
            .insert(RENodeId::AuthZoneStack);

        if !matches!(
            next_actor,
            Some(Actor {
                fn_identifier: FnIdentifier {
                    package_address: ACCESS_RULES_PACKAGE | AUTH_ZONE_PACKAGE,
                    ..
                },
                ..
            })
        ) {
            let handle = api.kernel_lock_substate(
                RENodeId::AuthZoneStack,
                NodeModuleId::SELF,
                SubstateOffset::AuthZoneStack(AuthZoneStackOffset::AuthZoneStack),
                LockFlags::MUTABLE,
            )?;
            let auth_zone_stack: &mut AuthZoneStackSubstate =
                api.kernel_get_substate_ref_mut(handle)?;

            // New auth zone frame managed by the AuthModule
            let is_barrier = Self::is_barrier(next_actor);

            // Add Package Actor Auth
            let mut virtual_non_fungibles = BTreeSet::new();
            if let Some(actor) = next_actor {
                let package_address = actor.fn_identifier.package_address();
                let id = scrypto_encode(&package_address).unwrap();
                let non_fungible_global_id =
                    NonFungibleGlobalId::new(PACKAGE_TOKEN, NonFungibleLocalId::bytes(id).unwrap());
                virtual_non_fungibles.insert(non_fungible_global_id);
            }

            auth_zone_stack.push_auth_zone(virtual_non_fungibles, is_barrier);
            api.kernel_drop_lock(handle)?;
        }

        Ok(())
    }

    fn on_execution_finish<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        _caller: &Option<Actor>,
        _update: &CallFrameUpdate,
    ) -> Result<(), RuntimeError> {
        if matches!(
            api.kernel_get_current_actor().unwrap().fn_identifier,
            FnIdentifier {
                package_address: ACCESS_RULES_PACKAGE | AUTH_ZONE_PACKAGE,
                ..
            }
        ) {
            return Ok(());
        }

        let handle = api.kernel_lock_substate(
            RENodeId::AuthZoneStack,
            NodeModuleId::SELF,
            SubstateOffset::AuthZoneStack(AuthZoneStackOffset::AuthZoneStack),
            LockFlags::MUTABLE,
        )?;
        {
            let auth_zone_stack: &mut AuthZoneStackSubstate =
                api.kernel_get_substate_ref_mut(handle)?;
            auth_zone_stack.pop_auth_zone();
        }
        api.kernel_drop_lock(handle)?;

        Ok(())
    }
}
