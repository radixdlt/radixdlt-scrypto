use crate::engine::HeapRENode;
use crate::model::*;
use crate::types::*;

pub fn node_to_substates(node_id: RENodeId, node: HeapRENode) -> HashMap<SubstateId, Substate> {
    let mut substates = HashMap::<SubstateId, Substate>::new();

    match node {
        HeapRENode::Bucket(_) => panic!("Unexpected"),
        HeapRENode::Proof(_) => panic!("Unexpected"),
        HeapRENode::AuthZone(_) => panic!("Unexpected"),
        HeapRENode::Global(global_node) => {
            let substate_id = match node_id {
                RENodeId::Global(global_address) => SubstateId::Global(global_address),
                _ => panic!("Unexpected"),
            };
            let substate = global_node.address;
            substates.insert(substate_id, Substate::GlobalRENode(substate));
        }
        HeapRENode::Vault(vault) => {
            let resource = vault
                .resource()
                .expect("Vault should be liquid at end of successful transaction");
            let substate = VaultSubstate(resource);
            let substate_id = match node_id {
                RENodeId::Vault(vault_id) => SubstateId::Vault(vault_id),
                _ => panic!("Unexpected"),
            };
            substates.insert(substate_id, substate.into());
        }
        HeapRENode::KeyValueStore(store) => {
            let store_id = match node_id {
                RENodeId::KeyValueStore(store_id) => store_id,
                _ => panic!("Unexpected"),
            };
            for (k, v) in store.loaded_entries {
                substates.insert(SubstateId::KeyValueStoreEntry(store_id, k), v.into());
            }
        }
        HeapRENode::Component(component) => {
            let address = match node_id {
                RENodeId::Component(address) => address,
                _ => panic!("Unexpected"),
            };
            substates.insert(SubstateId::ComponentInfo(address), component.info.into());
            if let Some(state) = component.state {
                substates.insert(SubstateId::ComponentState(address), state.into());
            }
        }
        HeapRENode::Worktop(_) => panic!("Unexpected"),
        HeapRENode::Package(package) => {
            let address = match node_id {
                RENodeId::Package(address) => address,
                _ => panic!("Unexpected"),
            };
            let substate = package.info;
            substates.insert(SubstateId::Package(address), substate.into());
        }
        HeapRENode::ResourceManager(resource_manager) => {
            let address = match node_id {
                RENodeId::ResourceManager(address) => address,
                _ => panic!("Unexpected"),
            };
            let substate = resource_manager.info;
            substates.insert(SubstateId::ResourceManager(address), substate.into());
        }
        HeapRENode::NonFungibleStore(non_fungible_store) => {
            let non_fungible_store_id = match node_id {
                RENodeId::NonFungibleStore(non_fungible_store_id) => non_fungible_store_id,
                _ => panic!("Unexpected"),
            };
            for (id, non_fungible) in non_fungible_store.loaded_non_fungibles {
                let substate_id = SubstateId::NonFungible(non_fungible_store_id.clone(), id);
                substates.insert(substate_id, non_fungible.into());
            }
        }
        HeapRENode::System(system) => {
            let address = match node_id {
                RENodeId::System(address) => address,
                _ => panic!("Unexpected"),
            };
            substates.insert(SubstateId::System(address), system.info.into());
        }
    }
    substates
}

pub fn nodes_to_substates(nodes: HashMap<RENodeId, HeapRENode>) -> HashMap<SubstateId, Substate> {
    let mut substates = HashMap::<SubstateId, Substate>::new();
    for (id, node) in nodes {
        substates.extend(node_to_substates(id, node));
    }
    substates
}
