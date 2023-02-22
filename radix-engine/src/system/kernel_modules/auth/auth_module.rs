use crate::errors::*;
use crate::kernel::actor::ResolvedActor;
use crate::kernel::actor::ResolvedReceiver;
use crate::kernel::call_frame::CallFrameUpdate;
use crate::kernel::call_frame::RENodeVisibilityOrigin;
use crate::kernel::kernel_api::KernelModuleApi;
use crate::kernel::kernel_api::LockFlags;
use crate::kernel::module::KernelModule;
use crate::system::node::RENodeInit;
use crate::system::node_modules::access_rules::AuthZoneStackSubstate;
use crate::types::*;
use radix_engine_interface::api::node_modules::auth::*;
use radix_engine_interface::api::package::{
    PACKAGE_LOADER_BLUEPRINT, PACKAGE_LOADER_PUBLISH_PRECOMPILED_IDENT,
};
use radix_engine_interface::api::types::{
    Address, AuthZoneStackOffset, ComponentOffset, PackageOffset, RENodeId, SubstateOffset,
    VaultOffset,
};
use radix_engine_interface::blueprints::resource::*;
use transaction::model::AuthZoneParams;

use super::auth_converter::convert_contextless;
use super::method_authorization::MethodAuthorization;
use super::method_authorization::MethodAuthorizationError;
use super::HardAuthRule;
use super::HardProofRule;
use super::HardResourceOrNonFungible;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum AuthError {
    VisibilityError(RENodeId),
    Unauthorized {
        authorization: MethodAuthorization,
        error: MethodAuthorizationError,
    },
}

#[derive(Debug, Clone)]
pub struct AuthModule {
    pub params: AuthZoneParams,
}

impl AuthModule {
    fn is_barrier(actor: &Option<ResolvedActor>) -> bool {
        matches!(
            actor,
            Some(ResolvedActor {
                receiver: Some(ResolvedReceiver {
                    derefed_from: Some((RENodeId::Global(Address::Component(..)), _)),
                    ..
                }),
                ..
            })
        )
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
            BTreeMap::new(),
        )?;
        Ok(())
    }

    fn on_teardown<Y: KernelModuleApi<RuntimeError>>(api: &mut Y) -> Result<(), RuntimeError> {
        api.kernel_drop_node(RENodeId::AuthZoneStack)?;

        Ok(())
    }

    fn before_push_frame<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        actor: &Option<ResolvedActor>,
        call_frame_update: &mut CallFrameUpdate,
    ) -> Result<(), RuntimeError> {
        if matches!(
            actor,
            Some(ResolvedActor {
                identifier: FnIdentifier {
                    package_address: AUTH_ZONE_PACKAGE,
                    ..
                },
                ..
            })
        ) {
            return Ok(());
        }

        let method_auths = if let Some(actor) = actor {
            match &actor {
                ResolvedActor {
                    identifier,
                    receiver: None,
                } => {
                    if identifier.package_address.eq(&PACKAGE_LOADER) {
                        if identifier.blueprint_name.eq(PACKAGE_LOADER_BLUEPRINT)
                            && identifier
                                .ident
                                .eq(PACKAGE_LOADER_PUBLISH_PRECOMPILED_IDENT)
                        {
                            vec![MethodAuthorization::Protected(HardAuthRule::ProofRule(
                                HardProofRule::Require(HardResourceOrNonFungible::NonFungible(
                                    AuthAddresses::system_role(),
                                )),
                            ))]
                        } else {
                            vec![]
                        }
                    } else {
                        let handle = api.kernel_lock_substate(
                            RENodeId::Global(Address::Package(identifier.package_address)),
                            NodeModuleId::PackageAccessRules,
                            SubstateOffset::PackageAccessRules,
                            LockFlags::read_only(),
                        )?;
                        let substate_ref = api.kernel_get_substate_ref(handle)?;
                        let substate = substate_ref.package_access_rules();
                        let local_fn_identifier = (
                            identifier.blueprint_name.to_string(),
                            identifier.ident.to_string(),
                        );
                        let access_rule = substate
                            .access_rules
                            .get(&local_fn_identifier)
                            .unwrap_or(&substate.default_auth);
                        let func_auth = convert_contextless(access_rule);
                        vec![func_auth]
                    }
                }

                // TODO: Cleanup
                // SetAccessRule auth is done manually within the method
                ResolvedActor {
                    receiver:
                        Some(ResolvedReceiver {
                            receiver:
                                MethodReceiver(
                                    _,
                                    NodeModuleId::AccessRules | NodeModuleId::AccessRules1,
                                ),
                            ..
                        }),
                    ..
                } => vec![],

                // TODO: Cleanup
                ResolvedActor {
                    receiver:
                        Some(ResolvedReceiver {
                            receiver:
                                MethodReceiver(
                                    RENodeId::Proof(..)
                                    | RENodeId::Bucket(..)
                                    | RENodeId::Worktop
                                    | RENodeId::Logger
                                    | RENodeId::TransactionRuntime
                                    | RENodeId::AuthZoneStack,
                                    ..,
                                ),
                            ..
                        }),
                    ..
                } => vec![],

                ResolvedActor {
                    identifier,
                    receiver:
                        Some(ResolvedReceiver {
                            receiver: MethodReceiver(RENodeId::Vault(vault_id), module_id),
                            ..
                        }),
                } => {
                    let vault_node_id = RENodeId::Vault(*vault_id);
                    let visibility = api.kernel_get_node_visibility_origin(vault_node_id).ok_or(
                        RuntimeError::CallFrameError(CallFrameError::RENodeNotVisible(
                            vault_node_id,
                        )),
                    )?;

                    let resource_address = {
                        let offset = SubstateOffset::Vault(VaultOffset::Info);
                        let handle = api.kernel_lock_substate(
                            vault_node_id,
                            NodeModuleId::SELF,
                            offset,
                            LockFlags::read_only(),
                        )?;
                        let substate_ref = api.kernel_get_substate_ref(handle)?;
                        let resource_address = substate_ref.vault_info().resource_address;
                        api.kernel_drop_lock(handle)?;
                        resource_address
                    };
                    let handle = api.kernel_lock_substate(
                        RENodeId::Global(Address::Resource(resource_address)),
                        NodeModuleId::AccessRules1,
                        SubstateOffset::AccessRulesChain(AccessRulesChainOffset::AccessRulesChain),
                        LockFlags::read_only(),
                    )?;

                    let substate_ref = api.kernel_get_substate_ref(handle)?;
                    let substate = substate_ref.access_rules_chain();

                    // TODO: Revisit what the correct abstraction is for visibility in the auth module
                    let auth = match visibility {
                        RENodeVisibilityOrigin::Normal => {
                            substate.native_fn_authorization(*module_id, identifier.clone())
                        }
                        RENodeVisibilityOrigin::DirectAccess => {
                            // TODO: Do we want to allow recaller to be able to withdraw from
                            // TODO: any visible vault?
                            if identifier.ident.eq(VAULT_RECALL_IDENT)
                                || identifier.ident.eq(VAULT_RECALL_NON_FUNGIBLES_IDENT)
                            {
                                let access_rule =
                                    substate.access_rules_chain[0].get_group("recall");
                                let authorization = convert_contextless(access_rule);
                                vec![authorization]
                            } else {
                                return Err(RuntimeError::ModuleError(ModuleError::AuthError(
                                    AuthError::VisibilityError(vault_node_id),
                                )));
                            }
                        }
                    };

                    api.kernel_drop_lock(handle)?;
                    auth
                }

                ResolvedActor {
                    identifier,
                    receiver:
                        Some(ResolvedReceiver {
                            receiver: MethodReceiver(RENodeId::Component(component_id), module_id),
                            ..
                        }),
                } => {
                    let offset = SubstateOffset::Package(PackageOffset::Info);
                    let handle = api.kernel_lock_substate(
                        RENodeId::Global(Address::Package(identifier.package_address)),
                        NodeModuleId::SELF,
                        offset,
                        LockFlags::read_only(),
                    )?;

                    if let NodeModuleId::SELF = module_id {
                        // Assume that package_address/blueprint is the original impl of Component for now
                        // TODO: Remove this assumption
                        let substate_ref = api.kernel_get_substate_ref(handle)?;
                        let package = substate_ref.package_info();
                        let schema = package
                            .blueprint_abi(&identifier.blueprint_name)
                            .expect("Blueprint not found for existing component")
                            .structure
                            .clone();
                        api.kernel_drop_lock(handle)?;

                        let state = {
                            let offset = SubstateOffset::Component(ComponentOffset::State0);
                            let handle = api.kernel_lock_substate(
                                RENodeId::Component(*component_id),
                                NodeModuleId::SELF,
                                offset,
                                LockFlags::read_only(),
                            )?;
                            let substate_ref = api.kernel_get_substate_ref(handle)?;
                            let state = substate_ref.component_state().clone(); // TODO: Remove clone
                            api.kernel_drop_lock(handle)?;
                            state
                        };

                        {
                            let handle = api.kernel_lock_substate(
                                RENodeId::Component(*component_id),
                                NodeModuleId::AccessRules,
                                SubstateOffset::AccessRulesChain(
                                    AccessRulesChainOffset::AccessRulesChain,
                                ),
                                LockFlags::read_only(),
                            )?;
                            let substate_ref = api.kernel_get_substate_ref(handle)?;
                            let access_rules = substate_ref.access_rules_chain();
                            let auth = access_rules.method_authorization(
                                &state,
                                &schema,
                                *module_id,
                                identifier.ident.clone(),
                            );
                            api.kernel_drop_lock(handle)?;
                            auth
                        }
                    } else {
                        let handle = api.kernel_lock_substate(
                            RENodeId::Component(*component_id),
                            NodeModuleId::AccessRules,
                            SubstateOffset::AccessRulesChain(
                                AccessRulesChainOffset::AccessRulesChain,
                            ),
                            LockFlags::read_only(),
                        )?;
                        let substate_ref = api.kernel_get_substate_ref(handle)?;
                        let access_rules = substate_ref.access_rules_chain();
                        let auth =
                            access_rules.native_fn_authorization(*module_id, identifier.clone());
                        api.kernel_drop_lock(handle)?;
                        auth
                    }
                }
                ResolvedActor {
                    identifier,
                    receiver: Some(ResolvedReceiver { receiver, .. }),
                } => {
                    let handle = api.kernel_lock_substate(
                        receiver.0,
                        NodeModuleId::AccessRules,
                        SubstateOffset::AccessRulesChain(AccessRulesChainOffset::AccessRulesChain),
                        LockFlags::read_only(),
                    )?;
                    let substate_ref = api.kernel_get_substate_ref(handle)?;
                    let substate = substate_ref.access_rules_chain();
                    let auth = substate.native_fn_authorization(receiver.1, identifier.clone());
                    api.kernel_drop_lock(handle)?;
                    auth
                }
            }
        } else {
            vec![]
        };

        let handle = api.kernel_lock_substate(
            RENodeId::AuthZoneStack,
            NodeModuleId::SELF,
            SubstateOffset::AuthZoneStack(AuthZoneStackOffset::AuthZoneStack),
            LockFlags::read_only(),
        )?;
        let substate_ref = api.kernel_get_substate_ref(handle)?;
        let auth_zone_stack = substate_ref.auth_zone_stack();
        let is_barrier = Self::is_barrier(actor);

        // Authorization check
        auth_zone_stack
            .check_auth(is_barrier, method_auths)
            .map_err(|(authorization, error)| {
                RuntimeError::ModuleError(ModuleError::AuthError(AuthError::Unauthorized {
                    authorization,
                    error,
                }))
            })?;

        api.kernel_drop_lock(handle)?;

        //  Additional ref copying

        call_frame_update
            .node_refs_to_copy
            .insert(RENodeId::AuthZoneStack);

        if !matches!(
            actor,
            Some(ResolvedActor {
                identifier: FnIdentifier {
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
            let mut substate_ref_mut = api.kernel_get_substate_ref_mut(handle)?;
            let auth_zone_stack = substate_ref_mut.auth_zone_stack();

            // New auth zone frame managed by the AuthModule
            let is_barrier = Self::is_barrier(actor);

            // Add Package Actor Auth
            let mut virtual_non_fungibles = BTreeSet::new();
            if let Some(actor) = actor {
                let package_address = actor.identifier.package_address();
                let id = scrypto_encode(&package_address).unwrap();
                let non_fungible_global_id =
                    NonFungibleGlobalId::new(PACKAGE_TOKEN, NonFungibleLocalId::bytes(id).unwrap());
                virtual_non_fungibles.insert(non_fungible_global_id);
            }

            auth_zone_stack.new_frame(virtual_non_fungibles, is_barrier);
            api.kernel_drop_lock(handle)?;
        }

        Ok(())
    }

    fn on_execution_finish<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        _caller: &Option<ResolvedActor>,
        _update: &CallFrameUpdate,
    ) -> Result<(), RuntimeError> {
        if matches!(
            api.kernel_get_current_actor().unwrap().identifier,
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
            let mut substate_ref_mut = api.kernel_get_substate_ref_mut(handle)?;
            let auth_zone_stack = substate_ref_mut.auth_zone_stack();
            auth_zone_stack.pop_frame();
        }
        api.kernel_drop_lock(handle)?;

        Ok(())
    }
}
