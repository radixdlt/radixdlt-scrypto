use sbor::rust::vec;
use sbor::rust::vec::Vec;
use scrypto::core::{FnIdentifier, NativeFnIdentifier, Receiver, SystemFnIdentifier};
use scrypto::engine::types::*;
use scrypto::values::ScryptoValue;

use crate::engine::*;
use crate::model::*;

pub struct AuthModule;

impl AuthModule {
    fn auth(
        function: &FnIdentifier,
        method_auths: Vec<MethodAuthorization>,
        call_frames: &mut Vec<CallFrame>, // TODO remove this once heap is implemented
    ) -> Result<(), RuntimeError> {
        let mut auth_zones = vec![
            &call_frames
                .last()
                .expect("Current frame always exists")
                .auth_zone,
        ];
        // FIXME: This is wrong as it allows extern component
        // FIXME: calls to use caller's auth zone
        // FIXME: Need to add a test for this
        if let Some(frame) = call_frames.iter().rev().nth(1) {
            auth_zones.push(&frame.auth_zone);
        }

        // Authorization check
        if !method_auths.is_empty() {
            for method_auth in method_auths {
                method_auth.check(&auth_zones).map_err(|error| {
                    RuntimeError::AuthorizationError {
                        function: function.clone(),
                        authorization: method_auth,
                        error,
                    }
                })?;
            }
        }

        Ok(())
    }

    pub fn receiver_auth(
        function: &FnIdentifier,
        receiver: Receiver,
        input: &ScryptoValue,
        node_pointer: RENodePointer,
        call_frames: &mut Vec<CallFrame>,
        track: &mut Track,
    ) -> Result<(), RuntimeError> {
        let auth = match (receiver, function) {
            (
                Receiver::Consumed(RENodeId::Bucket(..)),
                FnIdentifier::Native(NativeFnIdentifier::Bucket(bucket_fn)),
            ) => {
                let resource_address = {
                    let node_ref = node_pointer.to_ref(call_frames, track);
                    node_ref.bucket().resource_address()
                };
                let resource_manager = track
                    .read_substate(SubstateId::ResourceManager(resource_address))
                    .resource_manager();
                let method_auth = resource_manager.get_bucket_auth(*bucket_fn);
                vec![method_auth.clone()]
            }
            (
                Receiver::Ref(RENodeId::ResourceManager(resource_address)),
                FnIdentifier::Native(NativeFnIdentifier::ResourceManager(fn_ident)),
            ) => {
                let substate_id = SubstateId::ResourceManager(resource_address);
                let resource_manager = track.read_substate(substate_id.clone()).resource_manager();
                let method_auth = resource_manager.get_auth(*fn_ident, &input).clone();
                vec![method_auth]
            }
            (
                Receiver::Ref(RENodeId::System),
                FnIdentifier::Native(NativeFnIdentifier::System(SystemFnIdentifier::SetEpoch)),
            ) => {
                vec![MethodAuthorization::Protected(HardAuthRule::ProofRule(
                    HardProofRule::Require(HardResourceOrNonFungible::Resource(SYSTEM_TOKEN)),
                ))]
            }
            (Receiver::Ref(RENodeId::Component(..)), FnIdentifier::Native(..)) => {
                match node_pointer {
                    RENodePointer::Store(..) => vec![MethodAuthorization::DenyAll],
                    RENodePointer::Heap { .. } => vec![],
                }
            }
            (
                Receiver::Ref(RENodeId::Component(..)),
                FnIdentifier::Scrypto {
                    package_address,
                    blueprint_name,
                    ident,
                },
            ) => {
                // Assume that package_address/blueprint is the original impl of Component for now
                // TODO: Remove this assumption

                let package_substate_id = SubstateId::Package(*package_address);
                let package = track.read_substate(package_substate_id.clone()).package();
                let abi = package
                    .blueprint_abi(blueprint_name)
                    .expect("Blueprint not found for existing component");
                let fn_abi = abi
                    .get_fn_abi(ident)
                    .ok_or(RuntimeError::MethodDoesNotExist(function.clone()))?; // TODO: Move this check into kernel
                if !fn_abi.input.matches(&input.dom) {
                    return Err(RuntimeError::InvalidFnInput {
                        fn_identifier: function.clone(),
                    });
                }

                {
                    let value_ref = node_pointer.to_ref(call_frames, track);

                    let component = value_ref.component_info();
                    let component_state = value_ref.component_state();
                    component.method_authorization(component_state, &abi.structure, ident)
                }
            }
            (
                Receiver::Ref(RENodeId::Vault(..)),
                FnIdentifier::Native(NativeFnIdentifier::Vault(vault_fn)),
            ) => {
                let resource_address = {
                    let node_ref = node_pointer.to_ref(call_frames, track);
                    node_ref.vault().resource_address()
                };
                let resource_manager = track
                    .read_substate(SubstateId::ResourceManager(resource_address))
                    .resource_manager();
                vec![resource_manager.get_vault_auth(*vault_fn).clone()]
            }
            _ => vec![],
        };

        Self::auth(function, auth, call_frames)
    }
}
