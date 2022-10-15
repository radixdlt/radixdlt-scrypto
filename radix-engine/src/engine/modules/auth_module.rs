use crate::engine::*;
use crate::fee::FeeReserve;
use crate::model::*;
use crate::types::*;
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

    pub fn on_new_frame<'s, R: FeeReserve>(
        actor: &REActor,
        input: &ScryptoValue, // TODO: Remove
        call_frames: &mut Vec<CallFrame>,
        track: &mut Track<'s, R>,
    ) -> Result<HashMap<RENodeId, RENodePointer>, InvokeError<AuthError>> {
        let mut new_refs = HashMap::new();
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
                        let resource_pointer = RENodePointer::Store(node_id);
                        let offset =
                            SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
                        resource_pointer
                            .acquire_lock(offset.clone(), LockFlags::empty(), track)
                            .map_err(RuntimeError::KernelError)?;

                        let substate_ref =
                            resource_pointer.borrow_substate(&offset, call_frames, track)?;
                        let resource_manager = substate_ref.resource_manager();
                        let method_auth = resource_manager.get_auth(*method, &input).clone();
                        resource_pointer
                            .release_lock(offset, false, track)
                            .map_err(RuntimeError::KernelError)?;

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
                        let package_pointer = RENodePointer::Store(node_id);
                        let offset = SubstateOffset::Package(PackageOffset::Package);
                        package_pointer
                            .acquire_lock(offset.clone(), LockFlags::empty(), track)
                            .map_err(RuntimeError::KernelError)?;

                        // Assume that package_address/blueprint is the original impl of Component for now
                        // TODO: Remove this assumption
                        let package = track.borrow_substate(node_id, offset.clone()).package();
                        let schema = package
                            .blueprint_abi(&blueprint_name)
                            .expect("Blueprint not found for existing component")
                            .structure
                            .clone();

                        package_pointer
                            .release_lock(offset, false, track)
                            .map_err(RuntimeError::KernelError)?;

                        let component_node_pointer = call_frames
                            .last()
                            .unwrap()
                            .get_node_pointer(receiver.node_id())?;

                        let state = {
                            let offset = SubstateOffset::Component(ComponentOffset::State);
                            component_node_pointer
                                .acquire_lock(offset.clone(), LockFlags::empty(), track)
                                .map_err(RuntimeError::KernelError)?;
                            let substate_ref = component_node_pointer.borrow_substate(
                                &offset,
                                call_frames,
                                track,
                            )?;
                            let state = substate_ref.component_state().clone();
                            component_node_pointer
                                .release_lock(offset, false, track)
                                .map_err(RuntimeError::KernelError)?;
                            state
                        };
                        {
                            let offset = SubstateOffset::Component(ComponentOffset::Info);
                            component_node_pointer
                                .acquire_lock(offset.clone(), LockFlags::empty(), track)
                                .map_err(RuntimeError::KernelError)?;
                            let substate_ref = component_node_pointer.borrow_substate(
                                &offset,
                                call_frames,
                                track,
                            )?;
                            let info = substate_ref.component_info();
                            let auth = info.method_authorization(&state, &schema, &ident);
                            component_node_pointer
                                .release_lock(offset, false, track)
                                .map_err(RuntimeError::KernelError)?;
                            auth
                        }
                    }
                    (
                        Receiver::Ref(RENodeId::Vault(..)),
                        ResolvedMethod::Native(NativeMethod::Vault(ref vault_fn)),
                    ) => {
                        let vault_node_pointer = call_frames
                            .last()
                            .unwrap()
                            .get_node_pointer(receiver.receiver().node_id())?;

                        let resource_address = {
                            let offset = SubstateOffset::Vault(VaultOffset::Vault);
                            vault_node_pointer
                                .acquire_lock(offset.clone(), LockFlags::empty(), track)
                                .map_err(RuntimeError::KernelError)?;
                            let substate_ref =
                                vault_node_pointer.borrow_substate(&offset, call_frames, track)?;
                            let resource_address = substate_ref.vault().resource_address();
                            vault_node_pointer
                                .release_lock(offset, false, track)
                                .map_err(RuntimeError::KernelError)?;
                            resource_address
                        };
                        let node_id = RENodeId::ResourceManager(resource_address);
                        let resource_pointer = RENodePointer::Store(node_id);
                        let offset =
                            SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
                        resource_pointer
                            .acquire_lock(offset.clone(), LockFlags::empty(), track)
                            .map_err(RuntimeError::KernelError)?;

                        let substate_ref =
                            resource_pointer.borrow_substate(&offset, call_frames, track)?;
                        let resource_manager = substate_ref.resource_manager();
                        let auth = vec![resource_manager.get_vault_auth(*vault_fn).clone()];

                        resource_pointer
                            .release_lock(offset, false, track)
                            .map_err(RuntimeError::KernelError)?;

                        auth
                    }
                    _ => vec![],
                }
            }
        };

        let frame = call_frames.last_mut().unwrap();
        let auth_zone_id = frame
            .find_ref(|e| matches!(e, RENodeId::AuthZone(..)))
            .unwrap()
            .clone();
        let node_pointer = frame.get_node_pointer(auth_zone_id).unwrap();

        let mut auth_zone_ref_mut = match node_pointer {
            RENodePointer::Heap { frame_id, root, id } => {
                let frame = call_frames.get_mut(frame_id).unwrap();
                let heap_re_node = frame
                    .get_owned_heap_node_mut(root)
                    .unwrap()
                    .get_node_mut(id.as_ref());
                heap_re_node
                    .borrow_substate_mut(&SubstateOffset::AuthZone(AuthZoneOffset::AuthZone))
                    .unwrap()
            }
            _ => panic!("Unexpected"),
        };

        // Authorization check
        auth_zone_ref_mut
            .auth_zone()
            .check_auth(actor, method_auths)
            .map_err(|(authorization, error)| {
                InvokeError::Error(AuthError::Unauthorized {
                    actor: actor.clone(),
                    authorization,
                    error,
                })
            })?;

        // New auth zone frame managed by the AuthModule
        auth_zone_ref_mut.auth_zone().new_frame(actor);
        new_refs.insert(auth_zone_id, node_pointer);

        Ok(new_refs)
    }

    pub fn on_pop_frame(
        frame: &CallFrame,
        call_frames: &mut Vec<CallFrame>,
    ) -> Result<(), InvokeError<AuthError>> {
        if matches!(
            frame.actor,
            REActor::Method(ResolvedReceiverMethod {
                method: ResolvedMethod::Native(NativeMethod::AuthZone(..)),
                ..
            })
        ) {
            return Ok(());
        }

        let auth_zone_id = frame
            .find_ref(|e| matches!(e, RENodeId::AuthZone(..)))
            .unwrap()
            .clone();
        let node_pointer = frame.get_node_pointer(auth_zone_id).unwrap();
        {
            let mut authzone = match node_pointer {
                RENodePointer::Heap { frame_id, root, id } => {
                    let frame = call_frames.get_mut(frame_id).unwrap();
                    let heap_re_node = frame
                        .get_owned_heap_node_mut(root)
                        .unwrap()
                        .get_node_mut(id.as_ref());
                    heap_re_node
                        .borrow_substate_mut(&SubstateOffset::AuthZone(AuthZoneOffset::AuthZone))
                        .unwrap()
                }
                _ => panic!("Unexpected"),
            };
            // Copy-over root frame's auth zone virtual_proofs_buckets
            authzone.auth_zone().pop_frame();
        }

        Ok(())
    }
}
