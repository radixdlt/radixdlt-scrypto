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

    fn check_auth(
        actor: REActor,
        method_auths: Vec<MethodAuthorization>,
        call_frames: &Vec<CallFrame>, // TODO remove this once heap is implemented
    ) -> Result<(), AuthError> {
        // This module is called with a new call frame. Get the previous one which has an authzone.
        let second_to_last_index = call_frames.len() - 2;

        let prev_call_frame = call_frames.get(second_to_last_index)
            .expect("Previous call frame does not exist");

        let auth_zone = Self::get_auth_zone(prev_call_frame);

        let mut auth_zones = vec![auth_zone];

        // FIXME: This is wrong as it allows extern component calls to use caller's auth zone
        // Also, need to add a test for this
        if let Some(frame) = call_frames.iter().rev().nth(2) {
            let auth_zone = Self::get_auth_zone(frame);
            auth_zones.push(auth_zone);
        }

        // Authorization check
        if !method_auths.is_empty() {
            for method_auth in method_auths {
                method_auth
                    .check(&auth_zones)
                    .map_err(|error| AuthError::Unauthorized {
                        actor: actor.clone(),
                        authorization: method_auth,
                        error,
                    })?;
            }
        }

        Ok(())
    }

    pub fn get_auth_zone(call_frame: &CallFrame) -> &AuthZone {
        call_frame
            .owned_heap_nodes
            .values()
            .find(|e| {
                matches!(
                    e,
                    HeapRootRENode {
                        root: HeapRENode::AuthZone(..),
                        ..
                    }
                )
            })
            .expect("Could not find auth zone")
            .root
            .auth_zone()
    }

    pub fn function_auth(
        function_ident: FunctionIdent,
        call_frames: &mut Vec<CallFrame>,
    ) -> Result<(), AuthError> {
        let auth = match &function_ident {
            FunctionIdent::Native(NativeFunction::System(system_func)) => {
                System::function_auth(system_func)
            }
            _ => vec![],
        };
        Self::check_auth(REActor::Function(function_ident), auth, call_frames)
    }

    pub fn receiver_auth<'s, R: FeeReserve>(
        next_actor: FullyQualifiedReceiverMethod,
        input: &ScryptoValue,
        node_pointer: RENodePointer,
        call_frames: &mut Vec<CallFrame>,
        track: &mut Track<'s, R>,
    ) -> Result<(), InvokeError<AuthError>> {
        let FullyQualifiedReceiverMethod { receiver, method } = next_actor.clone();

        let auth = match (receiver, method) {
            (
                Receiver::Ref(RENodeId::ResourceManager(resource_address)),
                FullyQualifiedMethod::Native(NativeMethod::ResourceManager(ref method)),
            ) => {
                let node_id = RENodeId::ResourceManager(resource_address);
                let resource_pointer = RENodePointer::Store(node_id);
                let offset =
                    SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
                resource_pointer
                    .acquire_lock(offset.clone(), LockFlags::empty(), track)
                    .map_err(RuntimeError::KernelError)?;

                let substate_ref = resource_pointer.borrow_substate(&offset, call_frames, track)?;
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
                FullyQualifiedMethod::Native(NativeMethod::System(ref method)),
            ) => System::method_auth(method),
            (
                Receiver::Ref(RENodeId::Component(..)),
                FullyQualifiedMethod::Scrypto {
                    package_address,
                    blueprint_name,
                    ident,
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

                let state = {
                    let offset = SubstateOffset::Component(ComponentOffset::State);
                    node_pointer
                        .acquire_lock(offset.clone(), LockFlags::empty(), track)
                        .map_err(RuntimeError::KernelError)?;
                    let substate_ref = node_pointer.borrow_substate(&offset, call_frames, track)?;
                    let state = substate_ref.component_state().clone();
                    node_pointer
                        .release_lock(offset, false, track)
                        .map_err(RuntimeError::KernelError)?;
                    state
                };

                {
                    let offset = SubstateOffset::Component(ComponentOffset::Info);
                    node_pointer
                        .acquire_lock(offset.clone(), LockFlags::empty(), track)
                        .map_err(RuntimeError::KernelError)?;
                    let substate_ref = node_pointer.borrow_substate(&offset, call_frames, track)?;
                    let info = substate_ref.component_info();
                    let auth = info.method_authorization(&state, &schema, &ident);
                    node_pointer
                        .release_lock(offset, false, track)
                        .map_err(RuntimeError::KernelError)?;
                    auth
                }
            }
            (
                Receiver::Ref(RENodeId::Vault(..)),
                FullyQualifiedMethod::Native(NativeMethod::Vault(ref vault_fn)),
            ) => {
                let resource_address = {
                    let offset = SubstateOffset::Vault(VaultOffset::Vault);
                    node_pointer
                        .acquire_lock(offset.clone(), LockFlags::empty(), track)
                        .map_err(RuntimeError::KernelError)?;
                    let substate_ref = node_pointer.borrow_substate(&offset, call_frames, track)?;
                    let resource_address = substate_ref.vault().resource_address();
                    node_pointer
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

                let substate_ref = resource_pointer.borrow_substate(&offset, call_frames, track)?;
                let resource_manager = substate_ref.resource_manager();
                let auth = vec![resource_manager.get_vault_auth(*vault_fn).clone()];

                resource_pointer
                    .release_lock(offset, false, track)
                    .map_err(RuntimeError::KernelError)?;

                auth
            }
            _ => vec![],
        };

        Self::check_auth(REActor::Method(next_actor), auth, call_frames).map_err(InvokeError::Error)
    }
}
