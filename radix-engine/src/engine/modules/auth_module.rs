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

        let auth_zone = cur_call_frame
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
            .auth_zone();

        let mut auth_zones = vec![auth_zone];

        // FIXME: This is wrong as it allows extern component calls to use caller's auth zone
        // Also, need to add a test for this
        if let Some(frame) = call_frames.iter().rev().nth(1) {
            let auth_zone = frame
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
                .auth_zone();
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
        call_frames: &Vec<CallFrame>,
        track: &mut Track<'s, R>,
    ) -> Result<(), RuntimeError> {
        let auth = match &method_ident {
            ReceiverMethodIdent {
                receiver: Receiver::Consumed(RENodeId::Bucket(..)),
                method_ident: MethodIdent::Native(NativeMethod::Bucket(ref method)),
            } => {
                let resource_address = {
                    let node_ref = node_pointer.to_ref(call_frames, track);
                    node_ref.bucket().resource_address()
                };
                let resource_pointer =
                    RENodePointer::Store(RENodeId::ResourceManager(resource_address));
                resource_pointer
                    .acquire_lock(
                        SubstateId::ResourceManager(resource_address),
                        false,
                        false,
                        track,
                    )
                    .map_err(RuntimeError::KernelError)?;
                let resource_manager = track
                    .borrow_node(&RENodeId::ResourceManager(resource_address))
                    .resource_manager();
                let method_auth = resource_manager.get_bucket_auth(*method);
                let auth = vec![method_auth.clone()];
                resource_pointer
                    .release_lock(SubstateId::ResourceManager(resource_address), false, track)
                    .map_err(RuntimeError::KernelError)?;

                auth
            }
            ReceiverMethodIdent {
                receiver: Receiver::Ref(RENodeId::ResourceManager(resource_address)),
                method_ident: MethodIdent::Native(NativeMethod::ResourceManager(ref method)),
            } => {
                let resource_manager = track
                    .borrow_node(&RENodeId::ResourceManager(*resource_address))
                    .resource_manager();
                let method_auth = resource_manager.get_auth(*method, &input).clone();
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
                    let value_ref = node_pointer.to_ref(call_frames, track);
                    let component = value_ref.component();
                    (
                        component.info.package_address.clone(),
                        component.info.blueprint_name.clone(),
                    )
                };

                let package_pointer = RENodePointer::Store(RENodeId::Package(package_address));
                package_pointer
                    .acquire_lock(SubstateId::Package(package_address), false, false, track)
                    .map_err(RuntimeError::KernelError)?;

                // Assume that package_address/blueprint is the original impl of Component for now
                // TODO: Remove this assumption
                let package = track
                    .borrow_node(&RENodeId::Package(package_address))
                    .package()
                    .clone();
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
                    .release_lock(SubstateId::Package(package_address), false, track)
                    .map_err(RuntimeError::KernelError)?;

                {
                    let node_ref = node_pointer.to_ref(call_frames, track);
                    let component = node_ref.component();
                    component
                        .info
                        .method_authorization(&component.state, &abi.structure, ident)
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
                let resource_pointer =
                    RENodePointer::Store(RENodeId::ResourceManager(resource_address));
                resource_pointer
                    .acquire_lock(
                        SubstateId::ResourceManager(resource_address),
                        false,
                        false,
                        track,
                    )
                    .map_err(RuntimeError::KernelError)?;

                let resource_manager = track
                    .borrow_node(&RENodeId::ResourceManager(resource_address))
                    .resource_manager();
                let auth = vec![resource_manager.get_vault_auth(*vault_fn).clone()];

                resource_pointer
                    .release_lock(SubstateId::ResourceManager(resource_address), false, track)
                    .map_err(RuntimeError::KernelError)?;

                auth
            }
            _ => vec![],
        };

        Self::check_auth(FnIdent::Method(method_ident), auth, call_frames)
    }
}
