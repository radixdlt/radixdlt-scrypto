use sbor::rust::collections::*;
use sbor::rust::vec;
use sbor::rust::vec::Vec;
use scrypto::engine::types::*;
use scrypto::values::*;

use crate::engine::*;
use crate::engine::call_frame::RENodePointer;
use crate::model::*;

pub struct AuthModule;

impl AuthModule {
    pub fn auth(
        fn_ident: &str,
        input: &ScryptoValue,
        substate_id: SubstateId,
        node_pointer: RENodePointer,
        depth: usize,
        owned_heap_nodes: &mut HashMap<RENodeId, HeapRootRENode>,
        parent_heap_nodes: &mut Vec<&mut HashMap<RENodeId, HeapRootRENode>>,
        track: &mut Track
    ) -> Vec<MethodAuthorization> {
        let node_id = SubstateProperties::get_node_id(&substate_id);
        match node_id {
            RENodeId::ResourceManager(..) => {
                let resource_manager = track
                    .read_substate(substate_id)
                    .resource_manager();
                let method_auth = resource_manager.get_auth(&fn_ident, &input).clone();
                vec![method_auth]
            }
            RENodeId::System => {
                let fn_str: &str = &fn_ident;
                match fn_str {
                    "set_epoch" => {
                        vec![MethodAuthorization::Protected(HardAuthRule::ProofRule(
                            HardProofRule::Require(HardResourceOrNonFungible::Resource(
                                SYSTEM_TOKEN,
                            )),
                        ))]
                    }
                    _ => vec![],
                }
            }
            RENodeId::Component(..) => match node_pointer {
                RENodePointer::Store(..) => vec![MethodAuthorization::DenyAll],
                RENodePointer::Heap { .. } => vec![],
            },
            RENodeId::Vault(..) => {
                let resource_address = {
                    let node_ref = node_pointer.to_ref(
                        depth,
                        owned_heap_nodes,
                        parent_heap_nodes,
                        track,
                    );
                    node_ref.vault().resource_address()
                };
                let resource_manager = track
                    .read_substate(SubstateId::ResourceManager(resource_address))
                    .resource_manager();
                vec![resource_manager.get_vault_auth(&fn_ident).clone()]
            }
            _ => vec![],
        }
    }
}