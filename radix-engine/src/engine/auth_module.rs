use sbor::rust::vec;
use sbor::rust::vec::Vec;
use scrypto::engine::types::*;
use scrypto::values::*;

use crate::engine::*;
use crate::model::*;

pub struct AuthModule;

impl AuthModule {
    fn auth(
        fn_ident: &str,
        method_auths: Vec<MethodAuthorization>,
        call_frames: &mut Vec<CallFrame>, // TODO remove this once heap is implemented
    ) -> Result<(), RuntimeError> {
        let mut auth_zones = vec![
            &call_frames
                .last()
                .expect("Current frame always exists")
                .auth_zone,
        ];
        if let Some(frame) = call_frames.get(call_frames.len() - 2) {
            auth_zones.push(&frame.auth_zone);
        }

        // Authorization check
        if !method_auths.is_empty() {
            for method_auth in method_auths {
                method_auth.check(&auth_zones).map_err(|error| {
                    RuntimeError::AuthorizationError {
                        function: fn_ident.to_string(),
                        authorization: method_auth,
                        error,
                    }
                })?;
            }
        }

        Ok(())
    }

    pub fn consumed_auth(
        fn_ident: &str,
        node: &HeapRENode,
        call_frames: &mut Vec<CallFrame>,
        track: &mut Track,
    ) -> Result<(), RuntimeError> {
        let auth = match node {
            HeapRENode::Bucket(bucket) => {
                let resource_address = bucket.resource_address();
                let resource_manager = track
                    .read_substate(SubstateId::ResourceManager(resource_address))
                    .resource_manager();
                let method_auth = resource_manager.get_consuming_bucket_auth(&fn_ident);

                vec![method_auth.clone()]
            }
            HeapRENode::Proof(_) => vec![],
            _ => return Err(RuntimeError::MethodDoesNotExist(fn_ident.to_string())),
        };

        Self::auth(fn_ident, auth, call_frames)
    }

    pub fn ref_auth(
        fn_ident: &str,
        input: &ScryptoValue,
        substate_id: SubstateId,
        node_pointer: RENodePointer,
        call_frames: &mut Vec<CallFrame>,
        track: &mut Track,
    ) -> Result<(), RuntimeError> {
        let auth = match &substate_id {
            SubstateId::ResourceManager(..) => {
                let resource_manager = track.read_substate(substate_id.clone()).resource_manager();
                let method_auth = resource_manager.get_auth(&fn_ident, &input).clone();
                vec![method_auth]
            }
            SubstateId::System => match fn_ident {
                "set_epoch" => {
                    vec![MethodAuthorization::Protected(HardAuthRule::ProofRule(
                        HardProofRule::Require(HardResourceOrNonFungible::Resource(SYSTEM_TOKEN)),
                    ))]
                }
                _ => vec![],
            },
            SubstateId::ComponentInfo(..) => match node_pointer {
                RENodePointer::Store(..) => vec![MethodAuthorization::DenyAll],
                RENodePointer::Heap { .. } => vec![],
            },
            SubstateId::ComponentState(..) => {
                let node_ref = node_pointer.to_ref(call_frames, track);
                let component = node_ref.component_info();
                let package_substate_id = SubstateId::Package(component.package_address().clone());

                let package = track.read_substate(package_substate_id.clone()).package();
                let abi = package
                    .blueprint_abi(component.blueprint_name())
                    .expect("Blueprint not found for existing component");
                let fn_abi = abi
                    .get_fn_abi(&fn_ident)
                    .ok_or(RuntimeError::MethodDoesNotExist(fn_ident.to_string()))?;
                if !fn_abi.input.matches(&input.dom) {
                    return Err(RuntimeError::InvalidFnInput {
                        fn_ident: fn_ident.to_string(),
                    });
                }

                {
                    let value_ref = node_pointer.to_ref(call_frames, track);

                    let component = value_ref.component_info();
                    let component_state = value_ref.component_state();
                    component.method_authorization(component_state, &abi.structure, &fn_ident)
                }
            }
            SubstateId::Vault(..) => {
                let resource_address = {
                    let node_ref = node_pointer.to_ref(call_frames, track);
                    node_ref.vault().resource_address()
                };
                let resource_manager = track
                    .read_substate(SubstateId::ResourceManager(resource_address))
                    .resource_manager();
                vec![resource_manager.get_vault_auth(&fn_ident).clone()]
            }
            _ => vec![],
        };

        Self::auth(fn_ident, auth, call_frames)
    }
}
