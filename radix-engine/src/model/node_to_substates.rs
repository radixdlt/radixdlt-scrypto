use crate::engine::HeapRENode;
use crate::model::*;
use crate::types::*;

pub fn node_to_substates(node: HeapRENode) -> HashMap<SubstateOffset, RuntimeSubstate> {
    let mut substates = HashMap::<SubstateOffset, RuntimeSubstate>::new();

    match node {
        HeapRENode::Bucket(_) => panic!("Unexpected"),
        HeapRENode::Proof(_) => panic!("Unexpected"),
        HeapRENode::AuthZone(_) => panic!("Unexpected"),
        HeapRENode::Global(global_node) => {
            let substate = global_node.address;
            substates.insert(
                SubstateOffset::Global(GlobalOffset::Global),
                RuntimeSubstate::GlobalRENode(substate),
            );
        }
        HeapRENode::Vault(vault) => {
            substates.insert(SubstateOffset::Vault(VaultOffset::Vault), vault.into());
        }
        HeapRENode::KeyValueStore(store) => {
            for (k, v) in store.loaded_entries {
                substates.insert(
                    SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(k)),
                    v.into(),
                );
            }
        }
        HeapRENode::Component(component) => {
            substates.insert(
                SubstateOffset::Component(ComponentOffset::Info),
                component.info.into(),
            );
            if let Some(state) = component.state {
                substates.insert(
                    SubstateOffset::Component(ComponentOffset::State),
                    state.into(),
                );
            }
        }
        HeapRENode::Worktop(_) => panic!("Unexpected"),
        HeapRENode::Package(package) => {
            let substate = package.info;
            substates.insert(
                SubstateOffset::Package(PackageOffset::Package),
                substate.into(),
            );
        }
        HeapRENode::ResourceManager(resource_manager) => {
            let substate = resource_manager.info;
            substates.insert(
                SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager),
                substate.into(),
            );
        }
        HeapRENode::NonFungibleStore(non_fungible_store) => {
            for (id, non_fungible) in non_fungible_store.loaded_non_fungibles {
                substates.insert(
                    SubstateOffset::NonFungibleStore(NonFungibleStoreOffset::Entry(id)),
                    non_fungible.into(),
                );
            }
        }
        HeapRENode::System(system) => {
            substates.insert(
                SubstateOffset::System(SystemOffset::System),
                system.info.into(),
            );
        }
    }
    substates
}

pub fn nodes_to_substates(
    nodes: HashMap<RENodeId, HeapRENode>,
) -> HashMap<SubstateId, RuntimeSubstate> {
    let mut substates = HashMap::new();
    for (id, node) in nodes {
        for (offset, substate) in node_to_substates(node) {
            substates.insert(SubstateId(id, offset), substate);
        }
    }
    substates
}
