use crate::engine::*;
use crate::fee::FeeReserve;
use crate::model::*;
use crate::types::*;
use scrypto::core::{FnIdent, MethodFnIdent, MethodIdent};

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
        let mut auth_zones = vec![
            &call_frames
                .last()
                .expect("Current call frame does not exist")
                .auth_zone,
        ];
        // FIXME: This is wrong as it allows extern component calls to use caller's auth zone
        // Also, need to add a test for this
        if let Some(frame) = call_frames.iter().rev().nth(1) {
            auth_zones.push(&frame.auth_zone);
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
            FunctionIdent::Native(NativeFnIdentifier::System(system_fn)) => System::auth(system_fn),
            _ => vec![],
        };
        Self::check_auth(FnIdent::Function(function_ident), auth, call_frames)
    }

    pub fn receiver_auth<'s, R: FeeReserve>(
        method_ident: MethodIdent,
        input: &ScryptoValue,
        node_pointer: RENodePointer,
        call_frames: &Vec<CallFrame>,
        track: &mut Track<'s, R>,
    ) -> Result<(), RuntimeError> {
        let auth = match &method_ident {
            MethodIdent {
                receiver: Receiver::Consumed(RENodeId::Bucket(..)),
                fn_ident: MethodFnIdent::Native(NativeFnIdentifier::Bucket(ref bucket_fn)),
            } => {
                let resource_address = {
                    let node_ref = node_pointer.to_ref(call_frames, track);
                    node_ref.bucket().resource_address()
                };
                let resource_manager = track
                    .borrow_substate(SubstateId::ResourceManager(resource_address))
                    .raw()
                    .resource_manager();
                let method_auth = resource_manager.get_bucket_auth(*bucket_fn);
                vec![method_auth.clone()]
            }
            MethodIdent {
                receiver: Receiver::Ref(RENodeId::ResourceManager(resource_address)),
                fn_ident: MethodFnIdent::Native(NativeFnIdentifier::ResourceManager(ref fn_ident)),
            } => {
                let substate_id = SubstateId::ResourceManager(*resource_address);
                let resource_manager = track
                    .borrow_substate(substate_id.clone())
                    .raw()
                    .resource_manager();
                let method_auth = resource_manager.get_auth(*fn_ident, &input).clone();
                vec![method_auth]
            }
            MethodIdent {
                receiver: Receiver::Ref(RENodeId::System(..)),
                fn_ident: MethodFnIdent::Native(NativeFnIdentifier::System(ref system_fn)),
            } => System::auth(system_fn),
            MethodIdent {
                receiver: Receiver::Ref(RENodeId::Component(..)),
                fn_ident: MethodFnIdent::Native(..),
            } => match node_pointer {
                RENodePointer::Store(..) => vec![MethodAuthorization::DenyAll],
                RENodePointer::Heap { .. } => vec![],
            },
            MethodIdent {
                receiver: Receiver::Ref(RENodeId::Component(..)),
                fn_ident:
                    MethodFnIdent::Scrypto {
                        ref package_address,
                        ref blueprint_name,
                        ref ident,
                    },
            } => {
                // Assume that package_address/blueprint is the original impl of Component for now
                // TODO: Remove this assumption

                let package_substate_id = SubstateId::Package(*package_address);
                let package = track
                    .borrow_substate(package_substate_id.clone())
                    .raw()
                    .package()
                    .clone();
                let abi = package
                    .blueprint_abi(blueprint_name)
                    .expect("Blueprint not found for existing component");
                let fn_abi = abi.get_fn_abi(ident).ok_or(RuntimeError::KernelError(
                    KernelError::FnIdentNotFound(FnIdent::Method(method_ident.clone())),
                ))?; // TODO: Move this check into kernel
                if !fn_abi.input.matches(&input.dom) {
                    return Err(RuntimeError::KernelError(KernelError::InvalidFnInput2(
                        FnIdent::Method(method_ident),
                    )));
                }

                {
                    let value_ref = node_pointer.to_ref(call_frames, track);
                    let component = value_ref.component_info();
                    let component_state = value_ref.component_state();
                    component.method_authorization(component_state, &abi.structure, ident)
                }
            }
            MethodIdent {
                receiver: Receiver::Ref(RENodeId::Vault(..)),
                fn_ident: MethodFnIdent::Native(NativeFnIdentifier::Vault(ref vault_fn)),
            } => {
                let resource_address = {
                    let mut node_ref = node_pointer.to_ref(call_frames, track);
                    node_ref.vault().resource_address()
                };
                let resource_manager = track
                    .borrow_substate(SubstateId::ResourceManager(resource_address))
                    .raw()
                    .resource_manager();
                vec![resource_manager.get_vault_auth(*vault_fn).clone()]
            }
            _ => vec![],
        };

        Self::check_auth(FnIdent::Method(method_ident), auth, call_frames)
    }
}
