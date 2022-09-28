use crate::engine::HeapRENode;
use crate::model::*;
use crate::types::*;

// TODO: still lots of unwraps

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub enum Substate {
    System(SystemSubstate),
    ResourceManager(ResourceManagerSubstate),
    ComponentInfo(ComponentInfoSubstate),
    ComponentState(ComponentStateSubstate),
    Package(PackageSubstate),
    Vault(VaultSubstate),
    NonFungible(NonFungibleSubstate),
    KeyValueStoreEntry(KeyValueStoreEntrySubstate),
}

impl Substate {
    pub fn vault(&self) -> &VaultSubstate {
        if let Substate::Vault(vault) = self {
            vault
        } else {
            panic!("Not a vault");
        }
    }
    pub fn vault_mut(&mut self) -> &mut VaultSubstate {
        if let Substate::Vault(vault) = self {
            vault
        } else {
            panic!("Not a vault");
        }
    }

    pub fn resource_manager(&mut self) -> &mut ResourceManagerSubstate {
        if let Substate::ResourceManager(resource_manager) = self {
            resource_manager
        } else {
            panic!("Not a resource manager");
        }
    }

    pub fn resource_manager_mut(&mut self) -> &mut ResourceManagerSubstate {
        if let Substate::ResourceManager(resource_manager) = self {
            resource_manager
        } else {
            panic!("Not a resource manager");
        }
    }

    pub fn system(&self) -> &SystemSubstate {
        if let Substate::System(system) = self {
            system
        } else {
            panic!("Not a system value");
        }
    }

    pub fn system_mut(&mut self) -> &mut SystemSubstate {
        if let Substate::System(system) = self {
            system
        } else {
            panic!("Not a system value");
        }
    }

    pub fn component_state(&self) -> &ComponentStateSubstate {
        if let Substate::ComponentState(state) = self {
            state
        } else {
            panic!("Not a component state");
        }
    }

    pub fn component_state_mut(&mut self) -> &mut ComponentStateSubstate {
        if let Substate::ComponentState(component) = self {
            component
        } else {
            panic!("Not a component state");
        }
    }

    pub fn component_info(&self) -> &ComponentInfoSubstate {
        if let Substate::ComponentInfo(info) = self {
            info
        } else {
            panic!("Not a component info");
        }
    }

    pub fn component_info_mut(&mut self) -> &mut ComponentInfoSubstate {
        if let Substate::ComponentInfo(component) = self {
            component
        } else {
            panic!("Not a component info");
        }
    }

    pub fn package(&self) -> &PackageSubstate {
        if let Substate::Package(package) = self {
            package
        } else {
            panic!("Not a package");
        }
    }

    pub fn non_fungible(&self) -> &NonFungibleSubstate {
        if let Substate::NonFungible(non_fungible) = self {
            non_fungible
        } else {
            panic!("Not a NonFungible");
        }
    }

    pub fn kv_entry(&self) -> &KeyValueStoreEntrySubstate {
        if let Substate::KeyValueStoreEntry(kv_entry) = self {
            kv_entry
        } else {
            panic!("Not a KVEntry");
        }
    }
}

impl Into<Substate> for SystemSubstate {
    fn into(self) -> Substate {
        Substate::System(self)
    }
}

impl Into<Substate> for PackageSubstate {
    fn into(self) -> Substate {
        Substate::Package(self)
    }
}

impl Into<Substate> for ComponentInfoSubstate {
    fn into(self) -> Substate {
        Substate::ComponentInfo(self)
    }
}

impl Into<Substate> for ComponentStateSubstate {
    fn into(self) -> Substate {
        Substate::ComponentState(self)
    }
}

impl Into<Substate> for ResourceManagerSubstate {
    fn into(self) -> Substate {
        Substate::ResourceManager(self)
    }
}

impl Into<Substate> for VaultSubstate {
    fn into(self) -> Substate {
        Substate::Vault(self)
    }
}

impl Into<Substate> for NonFungibleSubstate {
    fn into(self) -> Substate {
        Substate::NonFungible(self)
    }
}

impl Into<Substate> for KeyValueStoreEntrySubstate {
    fn into(self) -> Substate {
        Substate::KeyValueStoreEntry(self)
    }
}

impl Into<ComponentInfoSubstate> for Substate {
    fn into(self) -> ComponentInfoSubstate {
        if let Substate::ComponentInfo(component) = self {
            component
        } else {
            panic!("Not a component info");
        }
    }
}

impl Into<ComponentStateSubstate> for Substate {
    fn into(self) -> ComponentStateSubstate {
        if let Substate::ComponentState(component_state) = self {
            component_state
        } else {
            panic!("Not a component");
        }
    }
}

impl Into<ResourceManagerSubstate> for Substate {
    fn into(self) -> ResourceManagerSubstate {
        if let Substate::ResourceManager(resource_manager) = self {
            resource_manager
        } else {
            panic!("Not a resource manager");
        }
    }
}

impl Into<PackageSubstate> for Substate {
    fn into(self) -> PackageSubstate {
        if let Substate::Package(package) = self {
            package
        } else {
            panic!("Not a resource manager");
        }
    }
}

impl Into<NonFungibleSubstate> for Substate {
    fn into(self) -> NonFungibleSubstate {
        if let Substate::NonFungible(non_fungible) = self {
            non_fungible
        } else {
            panic!("Not a non-fungible wrapper");
        }
    }
}

impl Into<KeyValueStoreEntrySubstate> for Substate {
    fn into(self) -> KeyValueStoreEntrySubstate {
        if let Substate::KeyValueStoreEntry(kv_entry) = self {
            kv_entry
        } else {
            panic!("Not a key value store entry wrapper");
        }
    }
}

impl Into<VaultSubstate> for Substate {
    fn into(self) -> VaultSubstate {
        if let Substate::Vault(vault) = self {
            vault
        } else {
            panic!("Not a vault");
        }
    }
}

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
        HeapRENode::KeyValueStore(_) => {
            // TODO: do we need a substate for key-value store?
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
        HeapRENode::Resource(resource_manager, maybe_non_fungibles) => {
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
