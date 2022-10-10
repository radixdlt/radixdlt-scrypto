use crate::engine::{KernelError, RuntimeError};
use crate::model::*;
use crate::types::*;

// TODO: still lots of unwraps

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub enum Substate {
    GlobalRENode(GlobalAddressSubstate),
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
    pub fn to_ref_mut(&mut self) -> SubstateRefMut {
        match self {
            Substate::GlobalRENode(value) => SubstateRefMut::Global(value),
            Substate::System(value) => SubstateRefMut::System(value),
            Substate::ResourceManager(value) => SubstateRefMut::ResourceManager(value),
            Substate::ComponentInfo(value) => SubstateRefMut::ComponentInfo(value),
            Substate::ComponentState(value) => SubstateRefMut::ComponentState(value),
            Substate::Package(value) => SubstateRefMut::Package(value),
            Substate::Vault(value) => SubstateRefMut::Vault(value),
            Substate::NonFungible(value) => SubstateRefMut::NonFungible(value),
            Substate::KeyValueStoreEntry(value) => SubstateRefMut::KeyValueStoreEntry(value),
        }
    }

    pub fn to_ref(&self) -> SubstateRef {
        match self {
            Substate::GlobalRENode(value) => SubstateRef::Global(value),
            Substate::System(value) => SubstateRef::System(value),
            Substate::ResourceManager(value) => SubstateRef::ResourceManager(value),
            Substate::ComponentInfo(value) => SubstateRef::ComponentInfo(value),
            Substate::ComponentState(value) => SubstateRef::ComponentState(value),
            Substate::Package(value) => SubstateRef::Package(value),
            Substate::Vault(value) => SubstateRef::Vault(value),
            Substate::NonFungible(value) => SubstateRef::NonFungible(value),
            Substate::KeyValueStoreEntry(value) => SubstateRef::KeyValueStoreEntry(value),
        }
    }

    pub fn global_re_node(&self) -> &GlobalAddressSubstate {
        if let Substate::GlobalRENode(global_re_node) = self {
            global_re_node
        } else {
            panic!("Not a global RENode");
        }
    }

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

    pub fn kv_store_entry(&self) -> &KeyValueStoreEntrySubstate {
        if let Substate::KeyValueStoreEntry(kv_store_entry) = self {
            kv_store_entry
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
        if let Substate::KeyValueStoreEntry(kv_store_entry) = self {
            kv_store_entry
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

impl Into<SystemSubstate> for Substate {
    fn into(self) -> SystemSubstate {
        if let Substate::System(system) = self {
            system
        } else {
            panic!("Not a resource manager");
        }
    }
}

impl Into<GlobalAddressSubstate> for Substate {
    fn into(self) -> GlobalAddressSubstate {
        if let Substate::GlobalRENode(substate) = self {
            substate
        } else {
            panic!("Not a global address substate");
        }
    }
}

pub enum SubstateRef<'a> {
    ComponentInfo(&'a ComponentInfoSubstate),
    ComponentState(&'a ComponentStateSubstate),
    NonFungible(&'a NonFungibleSubstate),
    KeyValueStoreEntry(&'a KeyValueStoreEntrySubstate),
    Package(&'a PackageSubstate),
    Vault(&'a VaultSubstate),
    ResourceManager(&'a ResourceManagerSubstate),
    System(&'a SystemSubstate),
    Global(&'a GlobalAddressSubstate),
}

impl<'a> SubstateRef<'a> {
    pub fn to_scrypto_value(&self) -> ScryptoValue {
        match self {
            SubstateRef::Global(value) => ScryptoValue::from_typed(*value),
            SubstateRef::System(value) => ScryptoValue::from_typed(*value),
            SubstateRef::ResourceManager(value) => ScryptoValue::from_typed(*value),
            SubstateRef::ComponentInfo(value) => ScryptoValue::from_typed(*value),
            SubstateRef::ComponentState(value) => ScryptoValue::from_typed(*value),
            SubstateRef::Package(value) => ScryptoValue::from_typed(*value),
            SubstateRef::Vault(value) => ScryptoValue::from_typed(*value),
            SubstateRef::NonFungible(value) => ScryptoValue::from_typed(*value),
            SubstateRef::KeyValueStoreEntry(value) => ScryptoValue::from_typed(*value),
        }
    }

    pub fn non_fungible(&self) -> &NonFungibleSubstate {
        match self {
            SubstateRef::NonFungible(non_fungible_substate) => *non_fungible_substate,
            _ => panic!("Not a non fungible"),
        }
    }

    pub fn system(&self) -> &SystemSubstate {
        match self {
            SubstateRef::System(system) => *system,
            _ => panic!("Not a system substate"),
        }
    }

    pub fn component_state(&self) -> &ComponentStateSubstate {
        match self {
            SubstateRef::ComponentState(state) => *state,
            _ => panic!("Not a component state"),
        }
    }

    pub fn component_info(&self) -> &ComponentInfoSubstate {
        match self {
            SubstateRef::ComponentInfo(info) => *info,
            _ => panic!("Not a component info"),
        }
    }

    pub fn references_and_owned_nodes(&self) -> (HashSet<GlobalAddress>, HashSet<RENodeId>) {
        match self {
            SubstateRef::ComponentState(substate) => {
                let scrypto_value = ScryptoValue::from_slice(&substate.raw).unwrap();
                (scrypto_value.global_references(), scrypto_value.node_ids())
            }
            SubstateRef::KeyValueStoreEntry(substate) => {
                let maybe_scrypto_value = substate
                    .0
                    .as_ref()
                    .map(|raw| ScryptoValue::from_slice(raw).unwrap());
                if let Some(scrypto_value) = maybe_scrypto_value {
                    (scrypto_value.global_references(), scrypto_value.node_ids())
                } else {
                    (HashSet::new(), HashSet::new())
                }
            }
            SubstateRef::NonFungible(substate) => {
                let maybe_scrypto_value = substate
                    .0
                    .as_ref()
                    .map(|non_fungible| ScryptoValue::from_typed(non_fungible));
                if let Some(scrypto_value) = maybe_scrypto_value {
                    (scrypto_value.global_references(), scrypto_value.node_ids())
                } else {
                    (HashSet::new(), HashSet::new())
                }
            }
            _ => (HashSet::new(), HashSet::new()),
        }
    }
}

pub enum SubstateRefMut<'a> {
    ComponentInfo(&'a mut ComponentInfoSubstate),
    ComponentState(&'a mut ComponentStateSubstate),
    NonFungible(&'a mut NonFungibleSubstate),
    KeyValueStoreEntry(&'a mut KeyValueStoreEntrySubstate),
    Package(&'a mut PackageSubstate),
    Vault(&'a mut VaultSubstate),
    ResourceManager(&'a mut ResourceManagerSubstate),
    System(&'a mut SystemSubstate),
    Global(&'a mut GlobalAddressSubstate),
}

impl<'a> SubstateRefMut<'a> {
    pub fn overwrite(&mut self, substate: Substate) -> Result<(), RuntimeError> {
        match (self, substate) {
            (SubstateRefMut::ComponentState(current), Substate::ComponentState(next)) => {
                **current = next
            }
            (SubstateRefMut::KeyValueStoreEntry(current), Substate::KeyValueStoreEntry(next)) => {
                **current = next
            }
            (SubstateRefMut::NonFungible(current), Substate::NonFungible(next)) => **current = next,
            (SubstateRefMut::System(current), Substate::System(next)) => **current = next,
            _ => return Err(RuntimeError::KernelError(KernelError::InvalidOverwrite)),
        }

        Ok(())
    }

    pub fn references_and_owned_nodes(&self) -> (HashSet<GlobalAddress>, HashSet<RENodeId>) {
        match self {
            SubstateRefMut::ComponentState(substate) => {
                let scrypto_value = ScryptoValue::from_slice(&substate.raw).unwrap();
                (scrypto_value.global_references(), scrypto_value.node_ids())
            }
            SubstateRefMut::KeyValueStoreEntry(substate) => {
                let maybe_scrypto_value = substate
                    .0
                    .as_ref()
                    .map(|raw| ScryptoValue::from_slice(raw).unwrap());
                if let Some(scrypto_value) = maybe_scrypto_value {
                    (scrypto_value.global_references(), scrypto_value.node_ids())
                } else {
                    (HashSet::new(), HashSet::new())
                }
            }
            SubstateRefMut::NonFungible(substate) => {
                let maybe_scrypto_value = substate
                    .0
                    .as_ref()
                    .map(|non_fungible| ScryptoValue::from_typed(non_fungible));
                if let Some(scrypto_value) = maybe_scrypto_value {
                    (scrypto_value.global_references(), scrypto_value.node_ids())
                } else {
                    (HashSet::new(), HashSet::new())
                }
            }
            _ => (HashSet::new(), HashSet::new()),
        }
    }
}
