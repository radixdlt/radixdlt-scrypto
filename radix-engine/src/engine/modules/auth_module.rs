use crate::engine::*;
use crate::fee::FeeReserve;
use crate::model::*;
use crate::types::*;
use crate::wasm::{WasmEngine, WasmInstance};
use scrypto::core::NativeFunction;

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, TypeId)]
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

    pub fn on_before_frame_start<'s, Y, W, I, R>(
        actor: &REActor,
        input: &ScryptoValue, // TODO: Remove
        system_api: &mut Y,
    ) -> Result<HashSet<RENodeId>, InvokeError<AuthError>>
    where
        Y: SystemApi<'s, W, I, R>,
        W: WasmEngine<I>,
        I: WasmInstance,
        R: FeeReserve,
    {
        let mut new_refs = HashSet::new();
        if matches!(
            actor,
            REActor::Method(ResolvedReceiverMethod {
                method: ResolvedMethod::Native(NativeMethod::AuthZone(..)),
                ..
            })
        ) {
            return Ok(new_refs);
        }

        let method_auths = match actor.clone() {
            REActor::Function(function_ident) => match function_ident {
                ResolvedFunction::Native(NativeFunction::System(system_func)) => {
                    System::function_auth(&system_func)
                }
                _ => vec![],
            },
            REActor::Method(ResolvedReceiverMethod { receiver, method }) => {
                match (receiver.receiver(), method) {
                    (
                        Receiver::Ref(RENodeId::ResourceManager(resource_address)),
                        ResolvedMethod::Native(NativeMethod::ResourceManager(ref method)),
                    ) => {
                        let node_id = RENodeId::ResourceManager(resource_address);
                        let offset =
                            SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
                        let handle =
                            system_api.lock_substate(node_id, offset, LockFlags::read_only())?;
                        let substate_ref = system_api.get_ref(handle)?;
                        let resource_manager = substate_ref.resource_manager();
                        let method_auth = resource_manager.get_auth(*method, &input).clone();
                        system_api.drop_lock(handle)?;
                        let auth = vec![method_auth];
                        auth
                    }
                    (
                        Receiver::Ref(RENodeId::System(..)),
                        ResolvedMethod::Native(NativeMethod::System(ref method)),
                    ) => System::method_auth(method),
                    (
                        Receiver::Ref(RENodeId::Component(..)),
                        ResolvedMethod::Scrypto {
                            package_address,
                            blueprint_name,
                            ident,
                            ..
                        },
                    ) => {
                        let node_id = RENodeId::Package(package_address);
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

                        let component_node_id = receiver.derefed_from.unwrap_or(receiver.node_id());
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
                        Receiver::Ref(RENodeId::Vault(..)),
                        ResolvedMethod::Native(NativeMethod::Vault(ref vault_fn)),
                    ) => {
                        let vault_node_id = receiver.derefed_from.unwrap_or(receiver.node_id());
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

        let refed = system_api.get_all_referenceable_node_ids()?;
        let auth_zone_id = refed
            .into_iter()
            .find(|e| matches!(e, RENodeId::AuthZone(..)))
            .unwrap();

        let handle = system_api.lock_substate(
            auth_zone_id,
            SubstateOffset::AuthZone(AuthZoneOffset::AuthZone),
            LockFlags::MUTABLE,
        )?;
        let mut substate_mut_ref = system_api.get_ref_mut(handle)?;
        let mut raw_mut = substate_mut_ref.get_raw_mut();
        let auth_zone_ref_mut = raw_mut.auth_zone();

        // Authorization check
        auth_zone_ref_mut
            .check_auth(actor, method_auths)
            .map_err(|(authorization, error)| {
                InvokeError::Error(AuthError::Unauthorized {
                    actor: actor.clone(),
                    authorization,
                    error,
                })
            })?;

        // New auth zone frame managed by the AuthModule
        auth_zone_ref_mut.new_frame(actor);
        new_refs.insert(auth_zone_id);

        substate_mut_ref.flush()?;
        system_api.drop_lock(handle)?;

        Ok(new_refs)
    }

    pub fn on_frame_end<'s, Y, W, I, R>(system_api: &mut Y) -> Result<(), InvokeError<AuthError>>
    where
        Y: SystemApi<'s, W, I, R>,
        W: WasmEngine<I>,
        I: WasmInstance,
        R: FeeReserve,
    {
        if matches!(
            system_api.get_actor(),
            REActor::Method(ResolvedReceiverMethod {
                method: ResolvedMethod::Native(NativeMethod::AuthZone(..)),
                ..
            })
        ) {
            return Ok(());
        }

        let refed = system_api.get_all_referenceable_node_ids()?;
        let auth_zone_id = refed
            .into_iter()
            .find(|e| matches!(e, RENodeId::AuthZone(..)))
            .unwrap();
        let handle = system_api.lock_substate(
            auth_zone_id,
            SubstateOffset::AuthZone(AuthZoneOffset::AuthZone),
            LockFlags::MUTABLE,
        )?;
        {
            let mut substate_ref_mut = system_api.get_ref_mut(handle)?;
            let mut raw_mut = substate_ref_mut.get_raw_mut();
            let auth_zone = raw_mut.auth_zone();
            auth_zone.pop_frame();
            substate_ref_mut.flush()?;
        }
        system_api.drop_lock(handle)?;

        Ok(())
    }
}
