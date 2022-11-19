use crate::engine::*;
use crate::model::*;
use crate::types::*;
use radix_engine_interface::api::types::{
    AuthZoneOffset, ComponentOffset, GlobalAddress, NativeFunction, NativeMethod, PackageOffset,
    RENodeId, ResourceManagerOffset, SubstateOffset, VaultOffset,
};

#[derive(Debug, Clone, PartialEq, Eq)]
#[scrypto(TypeId, Encode, Decode)]
pub enum AuthError {
    Unauthorized {
        actor: REActor,
        authorization: MethodAuthorization,
        error: MethodAuthorizationError,
    },
}

pub struct AuthModule;

impl AuthModule {
    pub fn supervisor_id() -> NonFungibleId {
        NonFungibleId::from_u32(0)
    }

    pub fn system_id() -> NonFungibleId {
        NonFungibleId::from_u32(1)
    }

    pub fn on_call_frame_enter<Y: SystemApi>(
        call_frame_update: &mut CallFrameUpdate,
        actor: &REActor,
        system_api: &mut Y,
    ) -> Result<(), RuntimeError> {
        let refed = system_api.get_visible_node_ids()?;
        let auth_zone_id = refed
            .into_iter()
            .find(|e| matches!(e, RENodeId::AuthZoneStack(..)))
            .unwrap();
        call_frame_update.node_refs_to_copy.insert(auth_zone_id);

        if !matches!(
            actor,
            REActor::Method(ResolvedMethod::Native(NativeMethod::AuthZone(..)), ..)
        ) {
            let handle = system_api.lock_substate(
                auth_zone_id,
                SubstateOffset::AuthZone(AuthZoneOffset::AuthZone),
                LockFlags::MUTABLE,
            )?;
            let mut substate_ref_mut = system_api.get_ref_mut(handle)?;
            let auth_zone_ref_mut = substate_ref_mut.auth_zone();

            // New auth zone frame managed by the AuthModule
            auth_zone_ref_mut.new_frame(actor);
            system_api.drop_lock(handle)?;
        }

        Ok(())
    }

    pub fn on_before_frame_start<Y, X>(
        actor: &REActor,
        executor: &X,
        system_api: &mut Y,
    ) -> Result<(), InvokeError<AuthError>>
    where
        Y: SystemApi,
        X: Executor,
    {
        if matches!(
            actor,
            REActor::Method(ResolvedMethod::Native(NativeMethod::AuthZone(..)), ..)
        ) {
            return Ok(());
        }

        let method_auths = match actor.clone() {
            REActor::Function(function_ident) => match function_ident {
                ResolvedFunction::Native(NativeFunction::EpochManager(system_func)) => {
                    EpochManager::function_auth(&system_func)
                }
                _ => vec![],
            },
            REActor::Method(method, resolved_receiver) => {
                match (method, resolved_receiver) {
                    (
                        ResolvedMethod::Native(NativeMethod::ResourceManager(ref method)),
                        ResolvedReceiver {
                            receiver: RENodeId::ResourceManager(resource_address),
                            ..
                        },
                    ) => {
                        let node_id = RENodeId::ResourceManager(resource_address);
                        let offset =
                            SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
                        let handle =
                            system_api.lock_substate(node_id, offset, LockFlags::read_only())?;
                        let substate_ref = system_api.get_ref(handle)?;
                        let resource_manager = substate_ref.resource_manager();
                        let method_auth =
                            resource_manager.get_auth(*method, executor.args()).clone();
                        system_api.drop_lock(handle)?;
                        let auth = vec![method_auth];
                        auth
                    }
                    (
                        ResolvedMethod::Native(NativeMethod::EpochManager(ref method)),
                        ResolvedReceiver {
                            receiver: RENodeId::EpochManager(..),
                            ..
                        },
                    ) => EpochManager::method_auth(method),
                    (
                        ResolvedMethod::Scrypto {
                            package_address,
                            blueprint_name,
                            ident,
                            ..
                        },
                        ResolvedReceiver {
                            receiver: RENodeId::Component(component_id),
                            ..
                        },
                    ) => {
                        let node_id = RENodeId::Global(GlobalAddress::Package(package_address));
                        let offset = SubstateOffset::Package(PackageOffset::Package);
                        let handle =
                            system_api.lock_substate(node_id, offset, LockFlags::read_only())?;

                        // Assume that package_address/blueprint is the original impl of Component for now
                        // TODO: Remove this assumption
                        let substate_ref = system_api.get_ref(handle)?;
                        let package = substate_ref.package();
                        let schema = package
                            .blueprint_abi(&blueprint_name)
                            .expect("Blueprint not found for existing component")
                            .structure
                            .clone();
                        system_api.drop_lock(handle)?;

                        let component_node_id = RENodeId::Component(component_id);
                        let state = {
                            let offset = SubstateOffset::Component(ComponentOffset::State);
                            let handle = system_api.lock_substate(
                                component_node_id,
                                offset,
                                LockFlags::read_only(),
                            )?;
                            let substate_ref = system_api.get_ref(handle)?;
                            let state = substate_ref.component_state().clone(); // TODO: Remove clone
                            system_api.drop_lock(handle)?;
                            state
                        };
                        {
                            let offset = SubstateOffset::Component(ComponentOffset::Info);
                            let handle = system_api.lock_substate(
                                component_node_id,
                                offset,
                                LockFlags::read_only(),
                            )?;
                            let substate_ref = system_api.get_ref(handle)?;
                            let info = substate_ref.component_info();
                            let auth = info.method_authorization(&state, &schema, &ident);
                            system_api.drop_lock(handle)?;
                            auth
                        }
                    }
                    (
                        ResolvedMethod::Native(NativeMethod::Vault(ref vault_fn)),
                        ResolvedReceiver {
                            receiver: RENodeId::Vault(vault_id),
                            ..
                        },
                    ) => {
                        let vault_node_id = RENodeId::Vault(vault_id);
                        let resource_address = {
                            let offset = SubstateOffset::Vault(VaultOffset::Vault);
                            let handle = system_api.lock_substate(
                                vault_node_id,
                                offset,
                                LockFlags::read_only(),
                            )?;
                            let substate_ref = system_api.get_ref(handle)?;
                            let resource_address = substate_ref.vault().resource_address();
                            system_api.drop_lock(handle)?;
                            resource_address
                        };
                        let node_id = RENodeId::Global(GlobalAddress::Resource(resource_address));
                        let offset =
                            SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
                        let handle =
                            system_api.lock_substate(node_id, offset, LockFlags::read_only())?;
                        let substate_ref = system_api.get_ref(handle)?;
                        let resource_manager = substate_ref.resource_manager();
                        let auth = vec![resource_manager.get_vault_auth(*vault_fn).clone()];
                        system_api.drop_lock(handle)?;

                        auth
                    }
                    _ => vec![],
                }
            }
        };

        let refed = system_api.get_visible_node_ids()?;
        let auth_zone_id = refed
            .into_iter()
            .find(|e| matches!(e, RENodeId::AuthZoneStack(..)))
            .unwrap();

        let handle = system_api.lock_substate(
            auth_zone_id,
            SubstateOffset::AuthZone(AuthZoneOffset::AuthZone),
            LockFlags::read_only(),
        )?;
        let substate_ref = system_api.get_ref(handle)?;
        let auth_zone_ref = substate_ref.auth_zone();

        // Authorization check
        auth_zone_ref
            .check_auth(actor, method_auths)
            .map_err(|(authorization, error)| {
                InvokeError::Error(AuthError::Unauthorized {
                    actor: actor.clone(),
                    authorization,
                    error,
                })
            })?;

        system_api.drop_lock(handle)?;

        Ok(())
    }

    pub fn on_call_frame_exit<Y>(system_api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: SystemApi,
    {
        if matches!(
            system_api.get_actor(),
            REActor::Method(ResolvedMethod::Native(NativeMethod::AuthZone(..)), ..)
        ) {
            return Ok(());
        }

        let refed = system_api.get_visible_node_ids()?;
        let auth_zone_id = refed
            .into_iter()
            .find(|e| matches!(e, RENodeId::AuthZoneStack(..)))
            .unwrap();
        let handle = system_api.lock_substate(
            auth_zone_id,
            SubstateOffset::AuthZone(AuthZoneOffset::AuthZone),
            LockFlags::MUTABLE,
        )?;
        {
            let mut substate_ref_mut = system_api.get_ref_mut(handle)?;
            let auth_zone = substate_ref_mut.auth_zone();
            auth_zone.pop_frame();
        }
        system_api.drop_lock(handle)?;

        Ok(())
    }
}
