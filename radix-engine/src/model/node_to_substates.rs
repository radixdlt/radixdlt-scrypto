use crate::engine::HeapRENode;
use crate::model::*;
use crate::types::*;

pub fn node_to_substates(node: HeapRENode) -> HashMap<SubstateOffset, Substate> {
    let mut substates = HashMap::<SubstateOffset, Substate>::new();

    match node {
        HeapRENode::Bucket(_) => panic!("Unexpected"),
        HeapRENode::Proof(_) => panic!("Unexpected"),
        HeapRENode::AuthZone(_) => panic!("Unexpected"),
        HeapRENode::Global(global_node) => {
            let substate = global_node.address;
            substates.insert(
                SubstateOffset::Global(GlobalOffset::Global),
                Substate::GlobalRENode(substate),
            );
        }
        HeapRENode::Vault(vault) => {
            let resource = vault
                .resource()
                .expect("Vault should be liquid at end of successful transaction");
            let substate = VaultSubstate(resource);
            substates.insert(SubstateOffset::Vault(VaultOffset::Vault), substate.into());
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
                SubstateOffset::Resource(ResourceManagerOffset::ResourceManager),
                substate.into(),
            );

            for (id, non_fungible) in resource_manager.loaded_non_fungibles {
                let offset = SubstateOffset::Resource(ResourceManagerOffset::NonFungible(id));
                substates.insert(offset, non_fungible.into());
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

pub fn nodes_to_substates(nodes: HashMap<RENodeId, HeapRENode>) -> HashMap<SubstateId, Substate> {
    let mut substates = HashMap::new();
    for (id, node) in nodes {
        for (offset, substate) in node_to_substates(node) {
            substates.insert(SubstateId(id, offset), substate);
        }
    }
    substates
}
