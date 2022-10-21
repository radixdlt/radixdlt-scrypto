use crate::engine::RENode;
use crate::model::*;
use crate::types::*;

pub fn node_to_substates(node: RENode) -> HashMap<SubstateOffset, RuntimeSubstate> {
    let mut substates = HashMap::<SubstateOffset, RuntimeSubstate>::new();

    match node {
        RENode::Bucket(_) => panic!("Unexpected"),
        RENode::Proof(_) => panic!("Unexpected"),
        RENode::AuthZone(_) => panic!("Unexpected"),
        RENode::Global(global_node) => {
            substates.insert(
                SubstateOffset::Global(GlobalOffset::Global),
                RuntimeSubstate::Global(global_node),
            );
        }
        RENode::Vault(vault) => {
            substates.insert(SubstateOffset::Vault(VaultOffset::Vault), vault.into());
        }
        RENode::KeyValueStore(store) => {
            for (k, v) in store.loaded_entries {
                substates.insert(
                    SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(k)),
                    v.into(),
                );
            }
        }
        RENode::Component(component) => {
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
        RENode::Worktop(_) => panic!("Unexpected"),
        RENode::Package(package) => {
            let substate = package.info;
            substates.insert(
                SubstateOffset::Package(PackageOffset::Package),
                substate.into(),
            );
        }
        RENode::ResourceManager(resource_manager) => {
            let substate = resource_manager.info;
            substates.insert(
                SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager),
                substate.into(),
            );
        }
        RENode::NonFungibleStore(nf_store) => {
            for (id, non_fungible) in nf_store.loaded_non_fungibles {
                substates.insert(
                    SubstateOffset::NonFungibleStore(NonFungibleStoreOffset::Entry(id)),
                    non_fungible.into(),
                );
            }
        }
        RENode::System(system) => {
            substates.insert(
                SubstateOffset::System(SystemOffset::System),
                system.info.into(),
            );
        }
    }
    substates
}

pub fn nodes_to_substates(
    nodes: HashMap<RENodeId, RENode>,
) -> HashMap<SubstateId, RuntimeSubstate> {
    let mut substates = HashMap::new();
    for (id, node) in nodes {
        for (offset, substate) in node_to_substates(node) {
            substates.insert(SubstateId(id, offset), substate);
        }
    }
    substates
}
