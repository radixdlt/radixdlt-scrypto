use crate::engine::{
    CallFrame, KernelError, RENodePointer, RuntimeError, SubstateProperties, Track,
};
use crate::fee::FeeReserve;
use crate::model::*;
use crate::types::*;

// TODO: still lots of unwraps

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub enum PersistedSubstate {
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

impl PersistedSubstate {
    pub fn decode_from_buffer(
        offset: &SubstateOffset,
        buffer: &[u8],
    ) -> Result<Self, RuntimeError> {
        let substate = match offset {
            SubstateOffset::Component(ComponentOffset::State) => {
                let substate = scrypto_decode(buffer).map_err(|e| KernelError::DecodeError(e))?;
                PersistedSubstate::ComponentState(substate)
            }
            SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(..)) => {
                let substate = scrypto_decode(buffer).map_err(|e| KernelError::DecodeError(e))?;
                PersistedSubstate::KeyValueStoreEntry(substate)
            }
            SubstateOffset::NonFungibleStore(NonFungibleStoreOffset::Entry(..)) => {
                let substate = scrypto_decode(buffer).map_err(|e| KernelError::DecodeError(e))?;
                PersistedSubstate::NonFungible(substate)
            }
            offset => {
                return Err(RuntimeError::KernelError(KernelError::OffsetNotAvailable(
                    offset.clone(),
                )))
            }
        };

        Ok(substate)
    }

    pub fn to_ref_mut(&mut self) -> RawSubstateRefMut {
        match self {
            PersistedSubstate::GlobalRENode(value) => RawSubstateRefMut::Global(value),
            PersistedSubstate::System(value) => RawSubstateRefMut::System(value),
            PersistedSubstate::ResourceManager(value) => RawSubstateRefMut::ResourceManager(value),
            PersistedSubstate::ComponentInfo(value) => RawSubstateRefMut::ComponentInfo(value),
            PersistedSubstate::ComponentState(value) => RawSubstateRefMut::ComponentState(value),
            PersistedSubstate::Package(value) => RawSubstateRefMut::Package(value),
            PersistedSubstate::Vault(value) => RawSubstateRefMut::Vault(value),
            PersistedSubstate::NonFungible(value) => RawSubstateRefMut::NonFungible(value),
            PersistedSubstate::KeyValueStoreEntry(value) => {
                RawSubstateRefMut::KeyValueStoreEntry(value)
            }
        }
    }

    pub fn to_ref(&self) -> SubstateRef {
        match self {
            PersistedSubstate::GlobalRENode(value) => SubstateRef::Global(value),
            PersistedSubstate::System(value) => SubstateRef::System(value),
            PersistedSubstate::ResourceManager(value) => SubstateRef::ResourceManager(value),
            PersistedSubstate::ComponentInfo(value) => SubstateRef::ComponentInfo(value),
            PersistedSubstate::ComponentState(value) => SubstateRef::ComponentState(value),
            PersistedSubstate::Package(value) => SubstateRef::Package(value),
            PersistedSubstate::Vault(value) => SubstateRef::Vault(value),
            PersistedSubstate::NonFungible(value) => SubstateRef::NonFungible(value),
            PersistedSubstate::KeyValueStoreEntry(value) => SubstateRef::KeyValueStoreEntry(value),
        }
    }

    pub fn global_re_node(&self) -> &GlobalAddressSubstate {
        if let PersistedSubstate::GlobalRENode(global_re_node) = self {
            global_re_node
        } else {
            panic!("Not a global RENode");
        }
    }

    pub fn vault(&self) -> &VaultSubstate {
        if let PersistedSubstate::Vault(vault) = self {
            vault
        } else {
            panic!("Not a vault");
        }
    }
    pub fn vault_mut(&mut self) -> &mut VaultSubstate {
        if let PersistedSubstate::Vault(vault) = self {
            vault
        } else {
            panic!("Not a vault");
        }
    }

    pub fn package(&self) -> &PackageSubstate {
        if let PersistedSubstate::Package(package) = self {
            package
        } else {
            panic!("Not a package");
        }
    }

    pub fn non_fungible(&self) -> &NonFungibleSubstate {
        if let PersistedSubstate::NonFungible(non_fungible) = self {
            non_fungible
        } else {
            panic!("Not a NonFungible");
        }
    }

    pub fn kv_store_entry(&self) -> &KeyValueStoreEntrySubstate {
        if let PersistedSubstate::KeyValueStoreEntry(kv_store_entry) = self {
            kv_store_entry
        } else {
            panic!("Not a KVEntry");
        }
    }
}

impl Into<PersistedSubstate> for SystemSubstate {
    fn into(self) -> PersistedSubstate {
        PersistedSubstate::System(self)
    }
}

impl Into<PersistedSubstate> for PackageSubstate {
    fn into(self) -> PersistedSubstate {
        PersistedSubstate::Package(self)
    }
}

impl Into<PersistedSubstate> for ComponentInfoSubstate {
    fn into(self) -> PersistedSubstate {
        PersistedSubstate::ComponentInfo(self)
    }
}

impl Into<PersistedSubstate> for ComponentStateSubstate {
    fn into(self) -> PersistedSubstate {
        PersistedSubstate::ComponentState(self)
    }
}

impl Into<PersistedSubstate> for ResourceManagerSubstate {
    fn into(self) -> PersistedSubstate {
        PersistedSubstate::ResourceManager(self)
    }
}

impl Into<PersistedSubstate> for VaultSubstate {
    fn into(self) -> PersistedSubstate {
        PersistedSubstate::Vault(self)
    }
}

impl Into<PersistedSubstate> for NonFungibleSubstate {
    fn into(self) -> PersistedSubstate {
        PersistedSubstate::NonFungible(self)
    }
}

impl Into<PersistedSubstate> for KeyValueStoreEntrySubstate {
    fn into(self) -> PersistedSubstate {
        PersistedSubstate::KeyValueStoreEntry(self)
    }
}

impl Into<ComponentInfoSubstate> for PersistedSubstate {
    fn into(self) -> ComponentInfoSubstate {
        if let PersistedSubstate::ComponentInfo(component) = self {
            component
        } else {
            panic!("Not a component info");
        }
    }
}

impl Into<ComponentStateSubstate> for PersistedSubstate {
    fn into(self) -> ComponentStateSubstate {
        if let PersistedSubstate::ComponentState(component_state) = self {
            component_state
        } else {
            panic!("Not a component");
        }
    }
}

impl Into<ResourceManagerSubstate> for PersistedSubstate {
    fn into(self) -> ResourceManagerSubstate {
        if let PersistedSubstate::ResourceManager(resource_manager) = self {
            resource_manager
        } else {
            panic!("Not a resource manager");
        }
    }
}

impl Into<PackageSubstate> for PersistedSubstate {
    fn into(self) -> PackageSubstate {
        if let PersistedSubstate::Package(package) = self {
            package
        } else {
            panic!("Not a resource manager");
        }
    }
}

impl Into<NonFungibleSubstate> for PersistedSubstate {
    fn into(self) -> NonFungibleSubstate {
        if let PersistedSubstate::NonFungible(non_fungible) = self {
            non_fungible
        } else {
            panic!("Not a non-fungible wrapper");
        }
    }
}

impl Into<KeyValueStoreEntrySubstate> for PersistedSubstate {
    fn into(self) -> KeyValueStoreEntrySubstate {
        if let PersistedSubstate::KeyValueStoreEntry(kv_store_entry) = self {
            kv_store_entry
        } else {
            panic!("Not a key value store entry wrapper");
        }
    }
}

impl Into<VaultSubstate> for PersistedSubstate {
    fn into(self) -> VaultSubstate {
        if let PersistedSubstate::Vault(vault) = self {
            vault
        } else {
            panic!("Not a vault");
        }
    }
}

impl Into<SystemSubstate> for PersistedSubstate {
    fn into(self) -> SystemSubstate {
        if let PersistedSubstate::System(system) = self {
            system
        } else {
            panic!("Not a resource manager");
        }
    }
}

impl Into<GlobalAddressSubstate> for PersistedSubstate {
    fn into(self) -> GlobalAddressSubstate {
        if let PersistedSubstate::GlobalRENode(substate) = self {
            substate
        } else {
            panic!("Not a global address substate");
        }
    }
}

pub enum SubstateRef<'a> {
    AuthZone(&'a AuthZone),
    Worktop(&'a Worktop),
    Proof(&'a Proof),
    Bucket(&'a Bucket),
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
            _ => panic!("Unsupported scrypto value"),
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

    pub fn proof(&self) -> &Proof {
        match self {
            SubstateRef::Proof(value) => *value,
            _ => panic!("Not a proof"),
        }
    }

    pub fn auth_zone(&self) -> &AuthZone {
        match self {
            SubstateRef::AuthZone(value) => *value,
            _ => panic!("Not an authzone"),
        }
    }

    pub fn worktop(&self) -> &Worktop {
        match self {
            SubstateRef::Worktop(value) => *value,
            _ => panic!("Not a worktop"),
        }
    }

    pub fn bucket(&self) -> &Bucket {
        match self {
            SubstateRef::Bucket(value) => *value,
            _ => panic!("Not a bucket"),
        }
    }

    pub fn resource_manager(&self) -> &ResourceManagerSubstate {
        match self {
            SubstateRef::ResourceManager(value) => *value,
            _ => panic!("Not a resource manager"),
        }
    }

    pub fn package(&self) -> &PackageSubstate {
        match self {
            SubstateRef::Package(value) => *value,
            _ => panic!("Not a package"),
        }
    }

    pub fn global_address(&self) -> &GlobalAddressSubstate {
        match self {
            SubstateRef::Global(value) => *value,
            _ => panic!("Not a global address"),
        }
    }

    pub fn references_and_owned_nodes(&self) -> (HashSet<GlobalAddress>, HashSet<RENodeId>) {
        match self {
            SubstateRef::Proof(proof) => {
                let mut references = HashSet::new();
                references.insert(GlobalAddress::Resource(proof.resource_address()));
                (references, HashSet::new())
            }
            SubstateRef::Bucket(bucket) => {
                let mut references = HashSet::new();
                references.insert(GlobalAddress::Resource(bucket.resource_address()));
                (references, HashSet::new())
            }
            SubstateRef::ComponentInfo(substate) => {
                let mut references = HashSet::new();
                references.insert(GlobalAddress::Package(substate.package_address));
                (references, HashSet::new())
            }
            SubstateRef::ResourceManager(substate) => {
                let mut owned_nodes = HashSet::new();
                if let Some(non_fungible_store_id) = substate.non_fungible_store_id {
                    owned_nodes.insert(RENodeId::NonFungibleStore(non_fungible_store_id));
                }
                (HashSet::new(), owned_nodes)
            }
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
    flushed: bool,
    lock_handle: LockHandle,
    prev_children: HashSet<RENodeId>,
    node_pointer: RENodePointer,
    offset: SubstateOffset,
    call_frames: &'f mut Vec<CallFrame>,
    track: &'f mut Track<'s, R>,
}

impl<'f, 's, R: FeeReserve> Drop for SubstateRefMut<'f, 's, R> {
    fn drop(&mut self) {
        if !self.flushed {
            self.do_flush().expect("Auto-flush failure.");
        }
    }
}

impl<'f, 's, R: FeeReserve> SubstateRefMut<'f, 's, R> {
    pub fn new(
        lock_handle: LockHandle,
        node_pointer: RENodePointer,
        offset: SubstateOffset,
        prev_children: HashSet<RENodeId>,
        call_frames: &'f mut Vec<CallFrame>,
        track: &'f mut Track<'s, R>,
    ) -> Result<Self, RuntimeError> {
        let substate_ref_mut = Self {
            flushed: false,
            lock_handle,
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

    // TODO: Move logic into substate unlock
    fn do_flush(&mut self) -> Result<(), RuntimeError> {
        let (new_global_references, new_children) = {
            let substate_ref_mut = self.get_raw_mut();
            substate_ref_mut.to_ref().references_and_owned_nodes()
        };

        let current_frame = self.call_frames.last_mut().unwrap();

        for global_address in new_global_references {
            let node_id = RENodeId::Global(global_address);
            if !current_frame.node_refs.contains_key(&node_id) {
                return Err(RuntimeError::KernelError(
                    KernelError::InvalidReferenceWrite(global_address),
                ));
            }
        }

        // Take values from current frame
        let (taken_nodes, missing_nodes) = {
            if !new_children.is_empty() {
                if !SubstateProperties::can_own_nodes(&self.offset) {
                    return Err(RuntimeError::KernelError(KernelError::ValueNotAllowed));
                }

                current_frame.take_available_values(new_children, true)?
            } else {
                (HashMap::new(), HashSet::new())
            }
        };
        verify_stored_value_update(&self.prev_children, &missing_nodes)?;

        for child_node in taken_nodes.keys() {
            current_frame.add_lock_visible_node(self.lock_handle, *child_node)?;
        }
        self.node_pointer
            .add_children(taken_nodes, &mut self.call_frames, &mut self.track);

        Ok(())
    }

    pub fn flush(mut self) -> Result<(), RuntimeError> {
        self.flushed = true;
        self.do_flush()
    }

    pub fn get_raw_mut(&mut self) -> RawSubstateRefMut {
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
                    RENodeId::NonFungibleStore(..),
                    SubstateOffset::NonFungibleStore(NonFungibleStoreOffset::Entry(
                        non_fungible_id,
                    )),
                ) => {
                    let parent_substate_id = SubstateId(
                        node_id,
                        SubstateOffset::NonFungibleStore(NonFungibleStoreOffset::Space),
                    );
                    self.track
                        .read_key_value_mut(parent_substate_id, non_fungible_id.to_vec())
                        .to_ref_mut()
                }
                _ => self
                    .track
                    .borrow_substate_mut(node_id, self.offset.clone())
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
    Bucket(&'a mut Bucket),
    Proof(&'a mut Proof),
    Worktop(&'a mut Worktop),
    AuthZone(&'a mut AuthZone),
}

impl<'a> RawSubstateRefMut<'a> {
    pub fn auth_zone(&mut self) -> &mut AuthZone {
        match self {
            RawSubstateRefMut::AuthZone(value) => *value,
            _ => panic!("Not an authzone"),
        }
    }

    pub fn worktop(&mut self) -> &mut Worktop {
        match self {
            RawSubstateRefMut::Worktop(value) => *value,
            _ => panic!("Not a worktop"),
        }
    }

    pub fn bucket(&mut self) -> &mut Bucket {
        match self {
            RawSubstateRefMut::Bucket(value) => *value,
            _ => panic!("Not a bucket"),
        }
    }

    pub fn non_fungible(&mut self) -> &mut NonFungibleSubstate {
        match self {
            RawSubstateRefMut::NonFungible(value) => *value,
            _ => panic!("Not a non fungible"),
        }
    }

    pub fn resource_manager(&mut self) -> &mut ResourceManagerSubstate {
        match self {
            RawSubstateRefMut::ResourceManager(value) => *value,
            _ => panic!("Not resource manager"),
        }
    }

    pub fn kv_store_entry(&mut self) -> &mut KeyValueStoreEntrySubstate {
        match self {
            RawSubstateRefMut::KeyValueStoreEntry(value) => *value,
            _ => panic!("Not a key value store entry"),
        }
    }

    pub fn component_state(&mut self) -> &mut ComponentStateSubstate {
        match self {
            RawSubstateRefMut::ComponentState(value) => *value,
            _ => panic!("Not component state"),
        }
    }

    pub fn component_info(&mut self) -> &mut ComponentInfoSubstate {
        match self {
            RawSubstateRefMut::ComponentInfo(value) => *value,
            _ => panic!("Not system"),
        }
    }

    pub fn system(&mut self) -> &mut SystemSubstate {
        match self {
            RawSubstateRefMut::System(value) => *value,
            _ => panic!("Not system"),
        }
    }

    fn to_ref(&self) -> SubstateRef {
        match self {
            RawSubstateRefMut::Bucket(value) => SubstateRef::Bucket(value),
            RawSubstateRefMut::Global(value) => SubstateRef::Global(value),
            RawSubstateRefMut::System(value) => SubstateRef::System(value),
            RawSubstateRefMut::ResourceManager(value) => SubstateRef::ResourceManager(value),
            RawSubstateRefMut::ComponentInfo(value) => SubstateRef::ComponentInfo(value),
            RawSubstateRefMut::ComponentState(value) => SubstateRef::ComponentState(value),
            RawSubstateRefMut::Package(value) => SubstateRef::Package(value),
            RawSubstateRefMut::Vault(value) => SubstateRef::Vault(value),
            RawSubstateRefMut::NonFungible(value) => SubstateRef::NonFungible(value),
            RawSubstateRefMut::KeyValueStoreEntry(value) => SubstateRef::KeyValueStoreEntry(value),
            RawSubstateRefMut::Proof(value) => SubstateRef::Proof(value),
            RawSubstateRefMut::Worktop(value) => SubstateRef::Worktop(value),
            RawSubstateRefMut::AuthZone(value) => SubstateRef::AuthZone(value),
        }
    }
}
