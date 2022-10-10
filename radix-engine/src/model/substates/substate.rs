use crate::engine::{
    CallFrame, KernelError, RENodePointer, RuntimeError, SubstateProperties, Track,
};
use crate::fee::FeeReserve;
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
    pub fn to_ref_mut(&mut self) -> RawSubstateRefMut {
        match self {
            Substate::GlobalRENode(value) => RawSubstateRefMut::Global(value),
            Substate::System(value) => RawSubstateRefMut::System(value),
            Substate::ResourceManager(value) => RawSubstateRefMut::ResourceManager(value),
            Substate::ComponentInfo(value) => RawSubstateRefMut::ComponentInfo(value),
            Substate::ComponentState(value) => RawSubstateRefMut::ComponentState(value),
            Substate::Package(value) => RawSubstateRefMut::Package(value),
            Substate::Vault(value) => RawSubstateRefMut::Vault(value),
            Substate::NonFungible(value) => RawSubstateRefMut::NonFungible(value),
            Substate::KeyValueStoreEntry(value) => RawSubstateRefMut::KeyValueStoreEntry(value),
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

pub fn verify_stored_value_update(
    old: &HashSet<RENodeId>,
    missing: &HashSet<RENodeId>,
) -> Result<(), RuntimeError> {
    // TODO: optimize intersection search
    for old_id in old.iter() {
        if !missing.contains(&old_id) {
            return Err(RuntimeError::KernelError(KernelError::StoredNodeRemoved(
                old_id.clone(),
            )));
        }
    }

    for missing_id in missing.iter() {
        if !old.contains(missing_id) {
            return Err(RuntimeError::KernelError(KernelError::RENodeNotFound(
                *missing_id,
            )));
        }
    }

    Ok(())
}

pub struct SubstateRefMut<'f, 's, R: FeeReserve> {
    prev_children: HashSet<RENodeId>,
    node_pointer: RENodePointer,
    offset: SubstateOffset,
    call_frames: &'f mut Vec<CallFrame>,
    track: &'f mut Track<'s, R>,
}

impl<'f, 's, R: FeeReserve> SubstateRefMut<'f, 's, R> {
    pub fn new(
        node_pointer: RENodePointer,
        offset: SubstateOffset,
        call_frames: &'f mut Vec<CallFrame>,
        track: &'f mut Track<'s, R>,
    ) -> Result<Self, RuntimeError> {
        let prev_children = {
            let substate_ref_mut = node_pointer.borrow_substate(&offset, call_frames, track)?;
            let (_old_global_references, prev_children) =
                substate_ref_mut.references_and_owned_nodes();
            prev_children
        };

        let substate_ref_mut = Self {
            prev_children,
            node_pointer,
            offset,
            call_frames,
            track,
        };
        Ok(substate_ref_mut)
    }

    pub fn offset(&self) -> &SubstateOffset {
        &self.offset
    }

    pub fn overwrite(&mut self, substate: Substate) -> Result<(), RuntimeError> {
        let (new_global_references, children) = substate.to_ref().references_and_owned_nodes();
        for global_address in new_global_references {
            let node_id = RENodeId::Global(global_address);
            let current_frame = self.call_frames.last_mut().unwrap();
            if !current_frame.node_refs.contains_key(&node_id) {
                return Err(RuntimeError::KernelError(
                    KernelError::InvalidReferenceWrite(global_address),
                ));
            }
        }

        // Take values from current frame
        let (taken_nodes, missing_nodes) = {
            if !children.is_empty() {
                if !SubstateProperties::can_own_nodes(&self.offset) {
                    return Err(RuntimeError::KernelError(KernelError::ValueNotAllowed));
                }

                let current_frame = self.call_frames.last_mut().unwrap();
                current_frame.take_available_values(children, true)?
            } else {
                (HashMap::new(), HashSet::new())
            }
        };
        verify_stored_value_update(&self.prev_children, &missing_nodes)?;

        self.node_pointer
            .add_children(taken_nodes, &mut self.call_frames, &mut self.track);

        let raw_mut = self.get_ref_mut();
        match (raw_mut, substate) {
            (RawSubstateRefMut::ComponentState(current), Substate::ComponentState(next)) => {
                *current = next
            }
            (
                RawSubstateRefMut::KeyValueStoreEntry(current),
                Substate::KeyValueStoreEntry(next),
            ) => *current = next,
            (RawSubstateRefMut::NonFungible(current), Substate::NonFungible(next)) => {
                *current = next
            }
            (RawSubstateRefMut::System(current), Substate::System(next)) => *current = next,
            _ => return Err(RuntimeError::KernelError(KernelError::InvalidOverwrite)),
        }

        Ok(())
    }

    fn get_ref_mut(&mut self) -> RawSubstateRefMut {
        match self.node_pointer {
            RENodePointer::Heap { frame_id, root, id } => {
                let frame = self.call_frames.get_mut(frame_id).unwrap();
                let heap_re_node = frame
                    .owned_heap_nodes
                    .get_mut(&root)
                    .unwrap()
                    .get_node_mut(id.as_ref());
                heap_re_node.borrow_substate_mut(&self.offset).unwrap()
            }
            RENodePointer::Store(node_id) => match (node_id, &self.offset) {
                (
                    RENodeId::KeyValueStore(..),
                    SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(key)),
                ) => {
                    let parent_substate_id = SubstateId(
                        node_id,
                        SubstateOffset::KeyValueStore(KeyValueStoreOffset::Space),
                    );
                    self.track
                        .read_key_value_mut(parent_substate_id, key.to_vec())
                        .to_ref_mut()
                }
                (
                    RENodeId::ResourceManager(..),
                    SubstateOffset::ResourceManager(ResourceManagerOffset::NonFungible(
                        non_fungible_id,
                    )),
                ) => {
                    let parent_substate_id = SubstateId(
                        node_id,
                        SubstateOffset::ResourceManager(ResourceManagerOffset::NonFungibleSpace),
                    );
                    self.track
                        .read_key_value_mut(parent_substate_id, non_fungible_id.to_vec())
                        .to_ref_mut()
                }
                _ => self
                    .track
                    .borrow_substate_mut(SubstateId(node_id, self.offset.clone()))
                    .to_ref_mut(),
            },
        }
    }
}

pub enum RawSubstateRefMut<'a> {
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
