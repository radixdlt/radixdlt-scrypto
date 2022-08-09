use sbor::rust::collections::*;
use sbor::rust::vec;
use sbor::rust::vec::Vec;
use scrypto::core::{FnIdentifier, Receiver};
use scrypto::engine::types::*;
use scrypto::values::*;

use crate::engine::call_frame::RENodePointer;
use crate::engine::*;
use crate::model::*;

pub struct AuthModule;

impl AuthModule {
    fn auth(
        function: &FnIdentifier,
        method_auths: Vec<MethodAuthorization>,
        auth_zone: Option<&AuthZone>,
        caller_auth_zone: Option<&AuthZone>,
    ) -> Result<(), RuntimeError> {
        // Authorization check
        if !method_auths.is_empty() {
            let mut auth_zones = Vec::new();
            if let Some(self_auth_zone) = auth_zone {
                auth_zones.push(self_auth_zone);
            }

            // FIXME: This is wrong as it allows extern component
            // FIXME: calls to use caller's auth zone
            // FIXME: Need to add a test for this
            if let Some(auth_zone) = caller_auth_zone {
                auth_zones.push(auth_zone);
            }

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
        depth: usize,
        owned_heap_nodes: &mut HashMap<RENodeId, HeapRootRENode>,
        parent_heap_nodes: &mut Vec<&mut HashMap<RENodeId, HeapRootRENode>>,
        track: &mut Track,
        auth_zone: Option<&AuthZone>,
        caller_auth_zone: Option<&AuthZone>,
    ) -> Result<(), RuntimeError> {
        let auth = match receiver {
            Receiver::Consumed(RENodeId::Bucket(..)) => {
                let resource_address = {
                    let node_ref =
                        node_pointer.to_ref(depth, owned_heap_nodes, parent_heap_nodes, track);
                    node_ref.bucket().resource_address()
                };
                let resource_manager = track
                    .read_substate(SubstateId::ResourceManager(resource_address))
                    .resource_manager();
                let method_auth = resource_manager.get_consuming_bucket_auth(function.fn_ident());
                vec![method_auth.clone()]
            }
            Receiver::Consumed(RENodeId::Proof(..)) => vec![],
            Receiver::Ref(RENodeId::ResourceManager(resource_address)) => match function {
                FnIdentifier::Native(fn_ident) => {
                    let substate_id = SubstateId::ResourceManager(resource_address);
                    let resource_manager =
                        track.read_substate(substate_id.clone()).resource_manager();
                    let method_auth = resource_manager.get_auth(fn_ident, &input).clone();
                    vec![method_auth]
                }
                _ => vec![],
            },
            Receiver::Ref(RENodeId::System) => match function {
                FnIdentifier::Native(fn_ident) => match fn_ident.as_str() {
                    "set_epoch" => {
                        vec![MethodAuthorization::Protected(HardAuthRule::ProofRule(
                            HardProofRule::Require(HardResourceOrNonFungible::Resource(
                                SYSTEM_TOKEN,
                            )),
                        ))]
                    }
                    _ => vec![],
                },
                _ => vec![],
            },
            Receiver::Ref(RENodeId::Component(..)) => {
                match function {
                    FnIdentifier::Native(..) => match node_pointer {
                        RENodePointer::Store(..) => vec![MethodAuthorization::DenyAll],
                        RENodePointer::Heap { .. } => vec![],
                    },
                    FnIdentifier::Scrypto {
                        package_address,
                        blueprint_name,
                        method_name,
                    } => {
                        // Assume that package_address/blueprint is the original impl of Component for now
                        // TODO: Remove this assumption

                        let package_substate_id = SubstateId::Package(*package_address);
                        let package = track.read_substate(package_substate_id.clone()).package();
                        let abi = package
                            .blueprint_abi(blueprint_name)
                            .expect("Blueprint not found for existing component");
                        let fn_abi = abi
                            .get_fn_abi(function.fn_ident())
                            .ok_or(RuntimeError::MethodDoesNotExist(function.clone()))?; // TODO: Move this check into kernel
                        if !fn_abi.input.matches(&input.dom) {
                            return Err(RuntimeError::InvalidFnInput {
                                fn_ident: function.fn_ident().to_string(),
                            });
                        }

                        {
                            let value_ref = node_pointer.to_ref(
                                depth,
                                owned_heap_nodes,
                                parent_heap_nodes,
                                track,
                            );

                            let component = value_ref.component_info();
                            let component_state = value_ref.component_state();
                            component.method_authorization(
                                component_state,
                                &abi.structure,
                                method_name,
                            )
                        }
                    }
                }
            }
            Receiver::Ref(RENodeId::Vault(..)) => match function {
                FnIdentifier::Native(fn_ident) => {
                    let resource_address = {
                        let node_ref =
                            node_pointer.to_ref(depth, owned_heap_nodes, parent_heap_nodes, track);
                        node_ref.vault().resource_address()
                    };
                    let resource_manager = track
                        .read_substate(SubstateId::ResourceManager(resource_address))
                        .resource_manager();
                    vec![resource_manager.get_vault_auth(fn_ident).clone()]
                }
                FnIdentifier::Scrypto { .. } => vec![],
            },
            _ => vec![],
        };

        Self::auth(function, auth, auth_zone, caller_auth_zone)
    }
}
