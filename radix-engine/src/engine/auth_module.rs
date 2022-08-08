use sbor::rust::collections::*;
use sbor::rust::vec;
use sbor::rust::vec::Vec;
use scrypto::core::ScryptoActor;
use scrypto::engine::types::*;
use scrypto::values::*;

use crate::engine::call_frame::RENodePointer;
use crate::engine::*;
use crate::model::*;

pub struct AuthModule;

impl AuthModule {
    fn auth(
        fn_ident: &str,
        substate_id: &SubstateId,
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

            match &substate_id {
                // Resource auth check includes caller
                SubstateId::ComponentState(..)
                | SubstateId::ResourceManager(..)
                | SubstateId::Vault(..)
                | SubstateId::Bucket(..) => {
                    if let Some(auth_zone) = caller_auth_zone {
                        auth_zones.push(auth_zone);
                    }
                }
                // Extern call auth check
                _ => {}
            };

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
        substate_id: &SubstateId,
        node_pointer: RENodePointer,
        depth: usize,
        owned_heap_nodes: &mut HashMap<RENodeId, HeapRootRENode>,
        parent_heap_nodes: &mut Vec<&mut HashMap<RENodeId, HeapRootRENode>>,
        track: &mut Track,
        auth_zone: Option<&AuthZone>,
        caller_auth_zone: Option<&AuthZone>,
    ) -> Result<(), RuntimeError> {
        let auth = match substate_id {
            SubstateId::Bucket(..) => {
                let resource_address = {
                    let node_ref =
                        node_pointer.to_ref(depth, owned_heap_nodes, parent_heap_nodes, track);
                    node_ref.bucket().resource_address()
                };
                let resource_manager = track
                    .read_substate(SubstateId::ResourceManager(resource_address))
                    .resource_manager();
                let method_auth = resource_manager.get_consuming_bucket_auth(&fn_ident);
                vec![method_auth.clone()]
            }
            SubstateId::Proof(..) => vec![],
            _ => return Err(RuntimeError::MethodDoesNotExist(fn_ident.to_string())),
        };

        Self::auth(fn_ident, substate_id, auth, auth_zone, caller_auth_zone)
    }

    pub fn ref_auth(
        fn_ident: &str,
        input: &ScryptoValue,
        actor: &REActor,
        substate_id: SubstateId,
        node_pointer: RENodePointer,
        depth: usize,
        owned_heap_nodes: &mut HashMap<RENodeId, HeapRootRENode>,
        parent_heap_nodes: &mut Vec<&mut HashMap<RENodeId, HeapRootRENode>>,
        track: &mut Track,
        auth_zone: Option<&AuthZone>,
        caller_auth_zone: Option<&AuthZone>,
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
                if let REActor::Scrypto(ScryptoActor::Component(
                    _component_address,
                    package_address,
                    blueprint_name,
                )) = actor
                {
                    let package_substate_id = SubstateId::Package(*package_address);
                    let package = track.read_substate(package_substate_id.clone()).package();
                    let abi = package
                        .blueprint_abi(blueprint_name)
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
                        let value_ref =
                            node_pointer.to_ref(depth, owned_heap_nodes, parent_heap_nodes, track);

                        let component = value_ref.component_info();
                        let component_state = value_ref.component_state();
                        component.method_authorization(component_state, &abi.structure, &fn_ident)
                    }
                } else {
                    vec![]
                }
            }
            SubstateId::Vault(..) => {
                let resource_address = {
                    let node_ref =
                        node_pointer.to_ref(depth, owned_heap_nodes, parent_heap_nodes, track);
                    node_ref.vault().resource_address()
                };
                let resource_manager = track
                    .read_substate(SubstateId::ResourceManager(resource_address))
                    .resource_manager();
                vec![resource_manager.get_vault_auth(&fn_ident).clone()]
            }
            _ => vec![],
        };

        Self::auth(fn_ident, &substate_id, auth, auth_zone, caller_auth_zone)
    }
}
