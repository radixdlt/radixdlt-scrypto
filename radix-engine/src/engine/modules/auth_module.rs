use crate::engine::*;
use crate::fee::FeeReserve;
use crate::model::*;
use crate::types::*;
use scrypto::core::{FnIdent, MethodIdent, NativeFunction, ReceiverMethodIdent};

pub struct AuthModule;

impl AuthModule {
    pub fn supervisor_id() -> NonFungibleId {
        NonFungibleId::from_u32(0)
    }

    pub fn system_id() -> NonFungibleId {
        NonFungibleId::from_u32(1)
    }

    fn check_auth(
        fn_ident: FnIdent,
        method_auths: Vec<MethodAuthorization>,
        call_frames: &Vec<CallFrame>, // TODO remove this once heap is implemented
    ) -> Result<(), RuntimeError> {
        let cur_call_frame = call_frames
            .last()
            .expect("Current call frame does not exist");

        let auth_zone = Self::get_auth_zone(cur_call_frame);

        let mut auth_zones = vec![auth_zone];

        // FIXME: This is wrong as it allows extern component calls to use caller's auth zone
        // Also, need to add a test for this
        if let Some(frame) = call_frames.iter().rev().nth(1) {
            let auth_zone = Self::get_auth_zone(frame);
            auth_zones.push(auth_zone);
        }

        // Authorization check
        if !method_auths.is_empty() {
            for method_auth in method_auths {
                method_auth.check(&auth_zones).map_err(|error| {
                    RuntimeError::ModuleError(ModuleError::AuthError {
                        fn_ident: fn_ident.clone(),
                        authorization: method_auth,
                        error,
                    })
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
    ) -> Result<(), RuntimeError> {
        let auth = match &function_ident {
            FunctionIdent::Native(NativeFunction::System(system_func)) => {
                System::function_auth(system_func)
            }
            _ => vec![],
        };
        Self::check_auth(FnIdent::Function(function_ident), auth, call_frames)
    }

    pub fn receiver_auth<'s, R: FeeReserve>(
        method_ident: ReceiverMethodIdent,
        input: &ScryptoValue,
        node_pointer: RENodePointer,
        call_frames: &mut Vec<CallFrame>,
        track: &mut Track<'s, R>,
    ) -> Result<(), RuntimeError> {
        let auth = match &method_ident {
            ReceiverMethodIdent {
                receiver: Receiver::Consumed(RENodeId::Bucket(..)),
                method_ident: MethodIdent::Native(NativeMethod::Bucket(ref method)),
            } => {
                let resource_address = {
                    let offset = SubstateOffset::Bucket(BucketOffset::Bucket);
                    node_pointer.acquire_lock(offset.clone(), false, false, track)?;
                    let substate_ref = node_pointer.borrow_substate(&offset, call_frames, track)?;
                    let resource_address = substate_ref.bucket().resource_address();
                    node_pointer.release_lock(offset.clone(), false, track)?;
                    resource_address
                };
                let node_id = RENodeId::ResourceManager(resource_address);
                let resource_pointer = RENodePointer::Store(node_id);
                let offset =
                    SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
                resource_pointer
                    .acquire_lock(offset.clone(), false, false, track)
                    .map_err(RuntimeError::KernelError)?;

                let substate_ref = resource_pointer.borrow_substate(&offset, call_frames, track)?;
                let resource_manager = substate_ref.resource_manager();
                let auth = vec![resource_manager.get_bucket_auth(*method).clone()];

                resource_pointer
                    .release_lock(offset, false, track)
                    .map_err(RuntimeError::KernelError)?;

                auth
            }
            ReceiverMethodIdent {
                receiver: Receiver::Ref(RENodeId::ResourceManager(resource_address)),
                method_ident: MethodIdent::Native(NativeMethod::ResourceManager(ref method)),
            } => {
                let node_id = RENodeId::ResourceManager(*resource_address);
                let resource_pointer = RENodePointer::Store(node_id);
                let offset =
                    SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
                resource_pointer
                    .acquire_lock(offset.clone(), false, false, track)
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
            ReceiverMethodIdent {
                receiver: Receiver::Ref(RENodeId::System(..)),
                method_ident: MethodIdent::Native(NativeMethod::System(ref method)),
            } => System::method_auth(method),
            ReceiverMethodIdent {
                receiver: Receiver::Ref(RENodeId::Component(..)),
                method_ident: MethodIdent::Native(..),
            } => match node_pointer {
                RENodePointer::Store(..) => vec![MethodAuthorization::DenyAll],
                RENodePointer::Heap { .. } => vec![],
            },
            ReceiverMethodIdent {
                receiver: Receiver::Ref(RENodeId::Component(..)),
                method_ident: MethodIdent::Scrypto(ref ident),
            } => {
                let (package_address, blueprint_name) = {
                    let offset = SubstateOffset::Component(ComponentOffset::Info);
                    node_pointer
                        .acquire_lock(offset.clone(), false, false, track)
                        .map_err(RuntimeError::KernelError)?;
                    let substate_ref = node_pointer.borrow_substate(&offset, call_frames, track)?;
                    let info = substate_ref.component_info();
                    let package_and_blueprint =
                        (info.package_address.clone(), info.blueprint_name.clone());
                    node_pointer
                        .release_lock(offset, false, track)
                        .map_err(RuntimeError::KernelError)?;
                    package_and_blueprint
                };

                let node_id = RENodeId::Package(package_address);
                let package_pointer = RENodePointer::Store(node_id);
                let offset = SubstateOffset::Package(PackageOffset::Package);
                package_pointer
                    .acquire_lock(offset.clone(), false, false, track)
                    .map_err(RuntimeError::KernelError)?;

                // Assume that package_address/blueprint is the original impl of Component for now
                // TODO: Remove this assumption
                let package = track
                    .borrow_substate(node_id, offset.clone())
                    .package()
                    .clone(); // TODO: Remove clone
                let abi = package
                    .blueprint_abi(&blueprint_name)
                    .expect("Blueprint not found for existing component");
                let fn_abi = abi.get_fn_abi(ident).ok_or(RuntimeError::KernelError(
                    KernelError::FnIdentNotFound(FnIdent::Method(method_ident.clone())),
                ))?; // TODO: Move this check into kernel
                if !fn_abi.input.matches(&input.dom) {
                    return Err(RuntimeError::KernelError(KernelError::InvalidFnInput2(
                        FnIdent::Method(method_ident),
                    )));
                }

                package_pointer
                    .release_lock(offset, false, track)
                    .map_err(RuntimeError::KernelError)?;

                let state = {
                    let offset = SubstateOffset::Component(ComponentOffset::State);
                    node_pointer
                        .acquire_lock(offset.clone(), false, false, track)
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
                        .acquire_lock(offset.clone(), false, false, track)
                        .map_err(RuntimeError::KernelError)?;
                    let substate_ref = node_pointer.borrow_substate(&offset, call_frames, track)?;
                    let info = substate_ref.component_info();
                    let auth = info.method_authorization(&state, &abi.structure, ident);
                    node_pointer
                        .release_lock(offset, false, track)
                        .map_err(RuntimeError::KernelError)?;
                    auth
                }
            }
            ReceiverMethodIdent {
                receiver: Receiver::Ref(RENodeId::Vault(..)),
                method_ident: MethodIdent::Native(NativeMethod::Vault(ref vault_fn)),
            } => {
                let resource_address = {
                    let mut node_ref = node_pointer.to_ref(call_frames, track);
                    node_ref.vault().resource_address()
                };
                let node_id = RENodeId::ResourceManager(resource_address);
                let resource_pointer = RENodePointer::Store(node_id);
                let offset =
                    SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
                resource_pointer
                    .acquire_lock(offset.clone(), false, false, track)
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

        Self::check_auth(FnIdent::Method(method_ident), auth, call_frames)
    }
}
