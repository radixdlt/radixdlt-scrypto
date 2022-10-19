use crate::engine::{
    CallFrame, Heap, KernelError, RENodeLocation, RuntimeError, SubstateProperties, Track,
};
use crate::fee::FeeReserve;
use crate::model::substates::worktop::WorktopSubstate;
use crate::model::*;
use crate::types::*;

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

impl Into<VaultSubstate> for PersistedSubstate {
    fn into(self) -> VaultSubstate {
        if let PersistedSubstate::Vault(vault) = self {
            vault
        } else {
            panic!("Not a vault");
        }
    }
}

impl PersistedSubstate {
    pub fn to_runtime(self) -> RuntimeSubstate {
        match self {
            PersistedSubstate::GlobalRENode(value) => RuntimeSubstate::GlobalRENode(value),
            PersistedSubstate::System(value) => RuntimeSubstate::System(value),
            PersistedSubstate::ResourceManager(value) => RuntimeSubstate::ResourceManager(value),
            PersistedSubstate::ComponentInfo(value) => RuntimeSubstate::ComponentInfo(value),
            PersistedSubstate::ComponentState(value) => RuntimeSubstate::ComponentState(value),
            PersistedSubstate::Package(value) => RuntimeSubstate::Package(value),
            PersistedSubstate::Vault(value) => {
                RuntimeSubstate::Vault(VaultRuntimeSubstate::new(value.0))
            }
            PersistedSubstate::NonFungible(value) => RuntimeSubstate::NonFungible(value),
            PersistedSubstate::KeyValueStoreEntry(value) => {
                RuntimeSubstate::KeyValueStoreEntry(value)
            }
        }
    }
}

pub enum PersistError {
    VaultLocked,
}

#[derive(Debug)]
pub enum RuntimeSubstate {
    GlobalRENode(GlobalAddressSubstate),
    System(SystemSubstate),
    ResourceManager(ResourceManagerSubstate),
    ComponentInfo(ComponentInfoSubstate),
    ComponentState(ComponentStateSubstate),
    Package(PackageSubstate),
    Vault(VaultRuntimeSubstate),
    NonFungible(NonFungibleSubstate),
    KeyValueStoreEntry(KeyValueStoreEntrySubstate),
}

impl RuntimeSubstate {
    pub fn clone_to_persisted(&self) -> PersistedSubstate {
        match self {
            RuntimeSubstate::GlobalRENode(value) => PersistedSubstate::GlobalRENode(value.clone()),
            RuntimeSubstate::System(value) => PersistedSubstate::System(value.clone()),
            RuntimeSubstate::ResourceManager(value) => {
                PersistedSubstate::ResourceManager(value.clone())
            }
            RuntimeSubstate::ComponentInfo(value) => {
                PersistedSubstate::ComponentInfo(value.clone())
            }
            RuntimeSubstate::ComponentState(value) => {
                PersistedSubstate::ComponentState(value.clone())
            }
            RuntimeSubstate::Package(value) => PersistedSubstate::Package(value.clone()),
            RuntimeSubstate::NonFungible(value) => PersistedSubstate::NonFungible(value.clone()),
            RuntimeSubstate::KeyValueStoreEntry(value) => {
                PersistedSubstate::KeyValueStoreEntry(value.clone())
            }
            RuntimeSubstate::Vault(value) => {
                let persisted_vault = value.clone_to_persisted();
                PersistedSubstate::Vault(persisted_vault)
            }
        }
    }

    pub fn to_persisted(self) -> PersistedSubstate {
        match self {
            RuntimeSubstate::GlobalRENode(value) => PersistedSubstate::GlobalRENode(value),
            RuntimeSubstate::System(value) => PersistedSubstate::System(value),
            RuntimeSubstate::ResourceManager(value) => PersistedSubstate::ResourceManager(value),
            RuntimeSubstate::ComponentInfo(value) => PersistedSubstate::ComponentInfo(value),
            RuntimeSubstate::ComponentState(value) => PersistedSubstate::ComponentState(value),
            RuntimeSubstate::Package(value) => PersistedSubstate::Package(value),
            RuntimeSubstate::NonFungible(value) => PersistedSubstate::NonFungible(value),
            RuntimeSubstate::KeyValueStoreEntry(value) => {
                PersistedSubstate::KeyValueStoreEntry(value)
            }
            RuntimeSubstate::Vault(value) => {
                let persisted_vault = value
                    .to_persisted()
                    .expect("Vault should be liquid at end of successful transaction");
                PersistedSubstate::Vault(persisted_vault)
            }
        }
    }

    pub fn decode_from_buffer(
        offset: &SubstateOffset,
        buffer: &[u8],
    ) -> Result<Self, RuntimeError> {
        let substate = match offset {
            SubstateOffset::Component(ComponentOffset::State) => {
                let substate = scrypto_decode(buffer).map_err(|e| KernelError::DecodeError(e))?;
                RuntimeSubstate::ComponentState(substate)
            }
            SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(..)) => {
                let substate = scrypto_decode(buffer).map_err(|e| KernelError::DecodeError(e))?;
                RuntimeSubstate::KeyValueStoreEntry(substate)
            }
            SubstateOffset::NonFungibleStore(NonFungibleStoreOffset::Entry(..)) => {
                let substate = scrypto_decode(buffer).map_err(|e| KernelError::DecodeError(e))?;
                RuntimeSubstate::NonFungible(substate)
            }
            offset => {
                return Err(RuntimeError::KernelError(KernelError::InvalidOffset(
                    offset.clone(),
                )))
            }
        };

        Ok(substate)
    }

    pub fn to_ref_mut(&mut self) -> RawSubstateRefMut {
        match self {
            RuntimeSubstate::GlobalRENode(value) => RawSubstateRefMut::Global(value),
            RuntimeSubstate::System(value) => RawSubstateRefMut::System(value),
            RuntimeSubstate::ResourceManager(value) => RawSubstateRefMut::ResourceManager(value),
            RuntimeSubstate::ComponentInfo(value) => RawSubstateRefMut::ComponentInfo(value),
            RuntimeSubstate::ComponentState(value) => RawSubstateRefMut::ComponentState(value),
            RuntimeSubstate::Package(value) => RawSubstateRefMut::Package(value),
            RuntimeSubstate::Vault(value) => RawSubstateRefMut::Vault(value),
            RuntimeSubstate::NonFungible(value) => RawSubstateRefMut::NonFungible(value),
            RuntimeSubstate::KeyValueStoreEntry(value) => {
                RawSubstateRefMut::KeyValueStoreEntry(value)
            }
        }
    }

    pub fn to_ref(&self) -> SubstateRef {
        match self {
            RuntimeSubstate::GlobalRENode(value) => SubstateRef::Global(value),
            RuntimeSubstate::System(value) => SubstateRef::System(value),
            RuntimeSubstate::ResourceManager(value) => SubstateRef::ResourceManager(value),
            RuntimeSubstate::ComponentInfo(value) => SubstateRef::ComponentInfo(value),
            RuntimeSubstate::ComponentState(value) => SubstateRef::ComponentState(value),
            RuntimeSubstate::Package(value) => SubstateRef::Package(value),
            RuntimeSubstate::Vault(value) => SubstateRef::Vault(value),
            RuntimeSubstate::NonFungible(value) => SubstateRef::NonFungible(value),
            RuntimeSubstate::KeyValueStoreEntry(value) => SubstateRef::KeyValueStoreEntry(value),
        }
    }

    pub fn global_re_node(&self) -> &GlobalAddressSubstate {
        if let RuntimeSubstate::GlobalRENode(global_re_node) = self {
            global_re_node
        } else {
            panic!("Not a global RENode");
        }
    }

    pub fn vault(&self) -> &VaultRuntimeSubstate {
        if let RuntimeSubstate::Vault(vault) = self {
            vault
        } else {
            panic!("Not a vault");
        }
    }
    pub fn vault_mut(&mut self) -> &mut VaultRuntimeSubstate {
        if let RuntimeSubstate::Vault(vault) = self {
            vault
        } else {
            panic!("Not a vault");
        }
    }

    pub fn package(&self) -> &PackageSubstate {
        if let RuntimeSubstate::Package(package) = self {
            package
        } else {
            panic!("Not a package");
        }
    }

    pub fn non_fungible(&self) -> &NonFungibleSubstate {
        if let RuntimeSubstate::NonFungible(non_fungible) = self {
            non_fungible
        } else {
            panic!("Not a NonFungible");
        }
    }

    pub fn kv_store_entry(&self) -> &KeyValueStoreEntrySubstate {
        if let RuntimeSubstate::KeyValueStoreEntry(kv_store_entry) = self {
            kv_store_entry
        } else {
            panic!("Not a KVEntry");
        }
    }
}

impl Into<RuntimeSubstate> for SystemSubstate {
    fn into(self) -> RuntimeSubstate {
        RuntimeSubstate::System(self)
    }
}

impl Into<RuntimeSubstate> for PackageSubstate {
    fn into(self) -> RuntimeSubstate {
        RuntimeSubstate::Package(self)
    }
}

impl Into<RuntimeSubstate> for ComponentInfoSubstate {
    fn into(self) -> RuntimeSubstate {
        RuntimeSubstate::ComponentInfo(self)
    }
}

impl Into<RuntimeSubstate> for ComponentStateSubstate {
    fn into(self) -> RuntimeSubstate {
        RuntimeSubstate::ComponentState(self)
    }
}

impl Into<RuntimeSubstate> for ResourceManagerSubstate {
    fn into(self) -> RuntimeSubstate {
        RuntimeSubstate::ResourceManager(self)
    }
}

impl Into<RuntimeSubstate> for VaultRuntimeSubstate {
    fn into(self) -> RuntimeSubstate {
        RuntimeSubstate::Vault(self)
    }
}

impl Into<RuntimeSubstate> for NonFungibleSubstate {
    fn into(self) -> RuntimeSubstate {
        RuntimeSubstate::NonFungible(self)
    }
}

impl Into<RuntimeSubstate> for KeyValueStoreEntrySubstate {
    fn into(self) -> RuntimeSubstate {
        RuntimeSubstate::KeyValueStoreEntry(self)
    }
}

impl Into<ComponentInfoSubstate> for RuntimeSubstate {
    fn into(self) -> ComponentInfoSubstate {
        if let RuntimeSubstate::ComponentInfo(component) = self {
            component
        } else {
            panic!("Not a component info");
        }
    }
}

impl Into<ComponentStateSubstate> for RuntimeSubstate {
    fn into(self) -> ComponentStateSubstate {
        if let RuntimeSubstate::ComponentState(component_state) = self {
            component_state
        } else {
            panic!("Not a component");
        }
    }
}

impl Into<ResourceManagerSubstate> for RuntimeSubstate {
    fn into(self) -> ResourceManagerSubstate {
        if let RuntimeSubstate::ResourceManager(resource_manager) = self {
            resource_manager
        } else {
            panic!("Not a resource manager");
        }
    }
}

impl Into<PackageSubstate> for RuntimeSubstate {
    fn into(self) -> PackageSubstate {
        if let RuntimeSubstate::Package(package) = self {
            package
        } else {
            panic!("Not a resource manager");
        }
    }
}

impl Into<NonFungibleSubstate> for RuntimeSubstate {
    fn into(self) -> NonFungibleSubstate {
        if let RuntimeSubstate::NonFungible(non_fungible) = self {
            non_fungible
        } else {
            panic!("Not a non-fungible wrapper");
        }
    }
}

impl Into<KeyValueStoreEntrySubstate> for RuntimeSubstate {
    fn into(self) -> KeyValueStoreEntrySubstate {
        if let RuntimeSubstate::KeyValueStoreEntry(kv_store_entry) = self {
            kv_store_entry
        } else {
            panic!("Not a key value store entry wrapper");
        }
    }
}

impl Into<VaultRuntimeSubstate> for RuntimeSubstate {
    fn into(self) -> VaultRuntimeSubstate {
        if let RuntimeSubstate::Vault(vault) = self {
            vault
        } else {
            panic!("Not a vault");
        }
    }
}

impl Into<SystemSubstate> for RuntimeSubstate {
    fn into(self) -> SystemSubstate {
        if let RuntimeSubstate::System(system) = self {
            system
        } else {
            panic!("Not a resource manager");
        }
    }
}

impl Into<GlobalAddressSubstate> for RuntimeSubstate {
    fn into(self) -> GlobalAddressSubstate {
        if let RuntimeSubstate::GlobalRENode(substate) = self {
            substate
        } else {
            panic!("Not a global address substate");
        }
    }
}

pub enum SubstateRef<'a> {
    AuthZone(&'a AuthZoneStackSubstate),
    Worktop(&'a WorktopSubstate),
    Proof(&'a ProofSubstate),
    Bucket(&'a BucketSubstate),
    ComponentInfo(&'a ComponentInfoSubstate),
    ComponentState(&'a ComponentStateSubstate),
    NonFungible(&'a NonFungibleSubstate),
    KeyValueStoreEntry(&'a KeyValueStoreEntrySubstate),
    Package(&'a PackageSubstate),
    Vault(&'a VaultRuntimeSubstate),
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

    pub fn proof(&self) -> &ProofSubstate {
        match self {
            SubstateRef::Proof(value) => *value,
            _ => panic!("Not a proof"),
        }
    }

    pub fn auth_zone(&self) -> &AuthZoneStackSubstate {
        match self {
            SubstateRef::AuthZone(value) => *value,
            _ => panic!("Not an authzone"),
        }
    }

    pub fn worktop(&self) -> &WorktopSubstate {
        match self {
            SubstateRef::Worktop(value) => *value,
            _ => panic!("Not a worktop"),
        }
    }

    pub fn bucket(&self) -> &BucketSubstate {
        match self {
            SubstateRef::Bucket(value) => *value,
            _ => panic!("Not a bucket"),
        }
    }

    pub fn vault(&self) -> &VaultRuntimeSubstate {
        match self {
            SubstateRef::Vault(value) => *value,
            _ => panic!("Not a vault"),
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
            SubstateRef::Global(global) => {
                let mut owned_nodes = HashSet::new();
                match global {
                    GlobalAddressSubstate::Resource(resource_address) => {
                        owned_nodes.insert(RENodeId::ResourceManager(*resource_address))
                    }
                    GlobalAddressSubstate::Component(component) => {
                        owned_nodes.insert(RENodeId::Component(component.0))
                    }
                    GlobalAddressSubstate::SystemComponent(component) => {
                        owned_nodes.insert(RENodeId::System(component.0))
                    }
                    GlobalAddressSubstate::Package(package_address) => {
                        owned_nodes.insert(RENodeId::Package(*package_address))
                    }
                };

                (HashSet::new(), owned_nodes)
            }
            SubstateRef::Vault(vault) => {
                let mut references = HashSet::new();
                references.insert(GlobalAddress::Resource(vault.resource_address()));
                (references, HashSet::new())
            }
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

pub struct SubstateRefMut<'f, 's, R: FeeReserve> {
    flushed: bool,
    _lock_handle: LockHandle,
    prev_children: HashSet<RENodeId>,
    location: RENodeLocation,
    node_id: RENodeId,
    offset: SubstateOffset,
    current_frame: &'f mut CallFrame,
    heap: &'f mut Heap,
    track: &'f mut Track<'s, R>,
}

// TODO: Remove once flush is moved into kernel substate unlock
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
        node_pointer: RENodeLocation,
        node_id: RENodeId,
        offset: SubstateOffset,
        prev_children: HashSet<RENodeId>,
        current_frame: &'f mut CallFrame,
        heap: &'f mut Heap,
        track: &'f mut Track<'s, R>,
    ) -> Result<Self, RuntimeError> {
        let substate_ref_mut = Self {
            flushed: false,
            _lock_handle: lock_handle,
            prev_children,
            location: node_pointer,
            node_id,
            offset,
            current_frame,
            heap,
            track,
        };
        Ok(substate_ref_mut)
    }

    pub fn offset(&self) -> &SubstateOffset {
        &self.offset
    }

    // TODO: Move into kernel substate unlock
    fn do_flush(&mut self) -> Result<(), RuntimeError> {
        let (new_global_references, mut new_children) = {
            let substate_ref_mut = self.get_raw_mut();
            substate_ref_mut.to_ref().references_and_owned_nodes()
        };

        for global_address in new_global_references {
            let node_id = RENodeId::Global(global_address);
            if !self.current_frame.node_refs.contains_key(&node_id) {
                return Err(RuntimeError::KernelError(
                    KernelError::InvalidReferenceWrite(global_address),
                ));
            }
        }

        for old_child in &self.prev_children {
            if !new_children.remove(old_child) {
                return Err(RuntimeError::KernelError(KernelError::StoredNodeRemoved(
                    old_child.clone(),
                )));
            }
        }

        for child_id in &new_children {
            SubstateProperties::verify_can_own(&self.offset, *child_id)?;
            // Make child visible
            //self.current_frame.add_lock_visible_node(self.lock_handle, *child_id)?;
        }

        match self.location {
            RENodeLocation::Heap => {
                self.current_frame.move_owned_nodes_to_heap_node(
                    self.heap,
                    new_children,
                    self.node_id,
                )?;
            }
            RENodeLocation::Store => {
                self.current_frame.move_owned_nodes_to_store(
                    self.heap,
                    self.track,
                    new_children,
                )?;
            }
        }

        Ok(())
    }

    pub fn flush(mut self) -> Result<(), RuntimeError> {
        self.flushed = true;
        self.do_flush()
    }

    pub fn get_raw_mut(&mut self) -> RawSubstateRefMut {
        match self.location {
            RENodeLocation::Heap => self
                .heap
                .get_substate_mut(self.node_id, &self.offset)
                .unwrap(),
            RENodeLocation::Store => match (self.node_id, &self.offset) {
                (
                    RENodeId::KeyValueStore(..),
                    SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(key)),
                ) => {
                    let parent_substate_id = SubstateId(
                        self.node_id,
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
                        self.node_id,
                        SubstateOffset::NonFungibleStore(NonFungibleStoreOffset::Space),
                    );
                    self.track
                        .read_key_value_mut(parent_substate_id, non_fungible_id.to_vec())
                        .to_ref_mut()
                }
                _ => self
                    .track
                    .borrow_substate_mut(self.node_id, self.offset.clone())
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
    Vault(&'a mut VaultRuntimeSubstate),
    ResourceManager(&'a mut ResourceManagerSubstate),
    System(&'a mut SystemSubstate),
    Global(&'a mut GlobalAddressSubstate),
    Bucket(&'a mut BucketSubstate),
    Proof(&'a mut ProofSubstate),
    Worktop(&'a mut WorktopSubstate),
    AuthZone(&'a mut AuthZoneStackSubstate),
}

impl<'a> RawSubstateRefMut<'a> {
    pub fn auth_zone(&mut self) -> &mut AuthZoneStackSubstate {
        match self {
            RawSubstateRefMut::AuthZone(value) => *value,
            _ => panic!("Not an authzone"),
        }
    }

    pub fn worktop(&mut self) -> &mut WorktopSubstate {
        match self {
            RawSubstateRefMut::Worktop(value) => *value,
            _ => panic!("Not a worktop"),
        }
    }

    pub fn vault(&mut self) -> &mut VaultRuntimeSubstate {
        match self {
            RawSubstateRefMut::Vault(value) => *value,
            _ => panic!("Not a vault"),
        }
    }

    pub fn proof(&mut self) -> &mut ProofSubstate {
        match self {
            RawSubstateRefMut::Proof(value) => *value,
            _ => panic!("Not a proof"),
        }
    }

    pub fn bucket(&mut self) -> &mut BucketSubstate {
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
