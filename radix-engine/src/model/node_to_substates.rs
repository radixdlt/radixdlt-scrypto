use crate::engine::HeapRENode;
use crate::model::*;
use crate::types::*;

pub fn node_to_substates(node_id: RENodeId, node: HeapRENode) -> HashMap<SubstateId, Substate> {
    let mut substates = HashMap::<SubstateId, Substate>::new();

    match node {
        HeapRENode::Bucket(_) => panic!("Unexpected"),
        HeapRENode::Proof(_) => panic!("Unexpected"),
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
            for (k, v) in store.store {
                substates.insert(
                    SubstateId::KeyValueStoreEntry(store_id, k),
                    Substate::KeyValueStoreEntry(KeyValueStoreEntrySubstate(Some(v.raw))),
                );
            }
        }
        HeapRENode::Component(component) => {
            let address = match node_id {
                RENodeId::Component(address) => address,
                _ => panic!("Unexpected"),
            };
            substates.insert(SubstateId::ComponentInfo(address), component.info.into());
            substates.insert(SubstateId::ComponentState(address), component.state.into());
        }
        HeapRENode::Worktop(_) => panic!("Unexpected"),
        HeapRENode::Package(package) => {
            let address = match node_id {
                RENodeId::Package(address) => address,
                _ => panic!("Unexpected"),
            };
            let substate = PackageSubstate {
                code: package.code,
                blueprint_abis: package.blueprint_abis,
            };
            substates.insert(SubstateId::Package(address), substate.into());
        }
        HeapRENode::ResourceManager(resource_manager, maybe_non_fungibles) => {
            let address = match node_id {
                RENodeId::ResourceManager(address) => address,
                _ => panic!("Unexpected"),
            };
            let substate = ResourceManagerSubstate {
                resource_type: resource_manager.resource_type,
                metadata: resource_manager.metadata,
                method_table: resource_manager.method_table,
                vault_method_table: resource_manager.vault_method_table,
                bucket_method_table: resource_manager.bucket_method_table,
                authorization: resource_manager.authorization,
                total_supply: resource_manager.total_supply,
            };
            substates.insert(SubstateId::ResourceManager(address), substate.into());

            if let Some(non_fungibles) = maybe_non_fungibles {
                for (id, non_fungible) in non_fungibles {
                    let substate_id = SubstateId::NonFungible(address.clone(), id);
                    let substate = Substate::NonFungible(NonFungibleSubstate(Some(non_fungible)));
                    substates.insert(substate_id, substate);
                }
            }
        }
        HeapRENode::System(system) => {
            let address = match node_id {
                RENodeId::System(address) => address,
                _ => panic!("Unexpected"),
            };
            substates.insert(
                SubstateId::System(address),
                SystemSubstate {
                    epoch: system.epoch,
                }
                .into(),
            );
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
