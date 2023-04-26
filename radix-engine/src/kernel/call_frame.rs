use crate::kernel::actor::Actor;
use crate::system::node_modules::type_info::TypeInfoSubstate;
use crate::types::*;
use radix_engine_interface::api::substate_lock_api::LockFlags;
use radix_engine_interface::blueprints::resource::{
    FUNGIBLE_BUCKET_BLUEPRINT, NON_FUNGIBLE_BUCKET_BLUEPRINT, PROOF_BLUEPRINT,
};
use radix_engine_interface::types::{LockHandle, NodeId, SubstateKey};
use radix_engine_stores::interface::{
    AcquireLockError, NodeSubstates, SetSubstateError, SubstateStore, TakeSubstateError,
};

use super::heap::Heap;
use super::kernel_api::LockInfo;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallFrameUpdate {
    pub nodes_to_move: Vec<NodeId>,
    pub node_refs_to_copy: IndexSet<NodeId>,
}

impl CallFrameUpdate {
    pub fn empty() -> Self {
        CallFrameUpdate {
            nodes_to_move: Vec::new(),
            node_refs_to_copy: index_set_new(),
        }
    }

    pub fn move_node(node_id: NodeId) -> Self {
        CallFrameUpdate {
            nodes_to_move: vec![node_id],
            node_refs_to_copy: index_set_new(),
        }
    }

    pub fn copy_ref(node_id: NodeId) -> Self {
        let mut node_refs_to_copy = index_set_new();
        node_refs_to_copy.insert(node_id);
        CallFrameUpdate {
            nodes_to_move: vec![],
            node_refs_to_copy,
        }
    }

    pub fn add_ref(&mut self, node_id: NodeId) {
        self.node_refs_to_copy.insert(node_id);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RENodeLocation {
    Heap,
    Store,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefType {
    Normal,
    DirectAccess,
}

/// A lock on a substate controlled by a call frame
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubstateLock {
    pub node_id: NodeId,
    pub module_id: ModuleId,
    pub substate_key: SubstateKey,
    pub initial_references: IndexSet<NodeId>,
    pub initial_owned_nodes: Vec<NodeId>,
    pub flags: LockFlags,
    pub store_handle: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RENodeRefData {
    ref_type: RefType,
}

impl RENodeRefData {
    fn new(ref_type: RefType) -> Self {
        RENodeRefData { ref_type }
    }
}

// TODO: reduce fields visibility

/// A call frame is the basic unit that forms a transaction call stack, which keeps track of the
/// owned objects by this function.
pub struct CallFrame {
    /// The frame id
    pub depth: usize,

    /// The running application actor of this frame
    /// TODO: Move to an RENode
    pub actor: Option<Actor>,

    /// Node refs which are immortal during the life time of this frame:
    /// - Any node refs received from other frames;
    /// - Global node refs obtained through substate locking.
    immortal_node_refs: NonIterMap<NodeId, RENodeRefData>,

    /// Node refs obtained through substate locking, which will be dropped upon unlocking.
    temp_node_refs: NonIterMap<NodeId, u32>,

    /// Owned nodes which by definition must live on heap
    /// Also keeps track of number of locks on this node, to prevent locked node from moving.
    owned_root_nodes: IndexMap<NodeId, u32>,

    next_lock_handle: LockHandle,
    locks: IndexMap<LockHandle, SubstateLock>,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum LockSubstateError {
    NodeNotInCallFrame(NodeId),
    LockUnmodifiedBaseOnHeapNode,
    SubstateNotFound(NodeId, ModuleId, SubstateKey),
    TrackError(Box<AcquireLockError>),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum UnlockSubstateError {
    LockNotFound(LockHandle),
    ContainsDuplicatedOwns,
    RefNotFound(NodeId),
    MoveError(MoveError),
    CantDropNodeInStore(NodeId),
    CantOwn(NodeId),
    CantStoreLocalReference(NodeId),
    CantBeStored(NodeId),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum MoveError {
    OwnNotFound(NodeId),
    RefNotFound(NodeId),
    CantMoveLockedNode(NodeId),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum ReadSubstateError {
    LockNotFound(LockHandle),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum WriteSubstateError {
    LockNotFound(LockHandle),
    NoWritePermission,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum CallFrameSetSubstateError {
    NodeNotInCallFrame(NodeId),
    StoreError(SetSubstateError),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum CallFrameRemoveSubstateError {
    NodeNotInCallFrame(NodeId),
    StoreError(TakeSubstateError),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum CallFrameScanSubstateError {
    NodeNotInCallFrame(NodeId),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum CallFrameScanSortedSubstatesError {
    NodeNotInCallFrame(NodeId),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum CallFrameTakeSortedSubstatesError {
    NodeNotInCallFrame(NodeId),
}

impl CallFrame {
    // TODO: Remove
    fn get_type_info<S: SubstateStore>(
        node_id: &NodeId,
        heap: &mut Heap,
        store: &mut S,
    ) -> Option<TypeInfoSubstate> {
        if let Some(substate) = heap.get_substate(
            node_id,
            SysModuleId::TypeInfo.into(),
            &TypeInfoOffset::TypeInfo.into(),
        ) {
            let type_info: TypeInfoSubstate = substate.as_typed().unwrap();
            Some(type_info)
        } else if let Ok(handle) = store.acquire_lock(
            node_id,
            SysModuleId::TypeInfo.into(),
            &TypeInfoOffset::TypeInfo.into(),
            LockFlags::read_only(),
        ) {
            let type_info: TypeInfoSubstate = store.read_substate(handle).as_typed().unwrap();
            store.release_lock(handle);
            Some(type_info)
        } else {
            None
        }
    }

    pub fn acquire_lock<S: SubstateStore>(
        &mut self,
        heap: &mut Heap,
        store: &mut S,
        node_id: &NodeId,
        module_id: ModuleId,
        substate_key: &SubstateKey,
        flags: LockFlags,
        default: Option<fn() -> IndexedScryptoValue>,
    ) -> Result<LockHandle, LockSubstateError> {
        // Check node visibility
        self.get_node_visibility(node_id)
            .ok_or_else(|| LockSubstateError::NodeNotInCallFrame(node_id.clone()))?;

        // Lock and read the substate
        let mut store_handle = None;
        let substate_value = if heap.contains_node(node_id) {
            // TODO: make Heap more like Store?
            if flags.contains(LockFlags::UNMODIFIED_BASE) {
                return Err(LockSubstateError::LockUnmodifiedBaseOnHeapNode);
            }
            if let Some(compute_default) = default {
                heap.get_substate_virtualize(
                    node_id,
                    module_id.into(),
                    substate_key,
                    compute_default,
                )
            } else {
                heap.get_substate(node_id, module_id.into(), substate_key)
                    .ok_or_else(|| {
                        LockSubstateError::SubstateNotFound(
                            node_id.clone(),
                            module_id,
                            substate_key.clone(),
                        )
                    })?
            }
        } else {
            let handle = store
                .acquire_lock_virtualize(node_id, module_id.into(), substate_key, flags, || {
                    default.map(|f| f())
                })
                .map_err(|x| LockSubstateError::TrackError(Box::new(x)))?;
            store_handle = Some(handle);
            store.read_substate(handle)
        };

        // Infer references and owns within the substate
        let references = substate_value.references();
        let owned_nodes = substate_value.owned_node_ids();
        let mut initial_references = index_set_new();
        for node_id in references {
            // TODO: fix this ugly condition
            if node_id.is_global() {
                // May overwrite existing node refs (for better visibility origin)
                self.immortal_node_refs.insert(
                    node_id.clone(),
                    RENodeRefData {
                        ref_type: RefType::Normal,
                    },
                );
            } else {
                initial_references.insert(node_id.clone());
            }
        }
        for node_id in owned_nodes {
            initial_references.insert(node_id.clone());
        }

        // Add initial references to ref count.
        for node_id in &initial_references {
            self.temp_node_refs
                .entry(node_id.clone())
                .or_default()
                .add_assign(1);
        }

        let lock_handle = self.next_lock_handle;
        self.locks.insert(
            lock_handle,
            SubstateLock {
                node_id: node_id.clone(),
                module_id,
                substate_key: substate_key.clone(),
                initial_references,
                initial_owned_nodes: owned_nodes.clone(),
                flags,
                store_handle,
            },
        );
        self.next_lock_handle = self.next_lock_handle + 1;

        // Update lock count on the node
        if let Some(counter) = self.owned_root_nodes.get_mut(node_id) {
            *counter += 1;
        }

        Ok(lock_handle)
    }

    pub fn drop_lock<S: SubstateStore>(
        &mut self,
        heap: &mut Heap,
        store: &mut S,
        lock_handle: LockHandle,
    ) -> Result<(), UnlockSubstateError> {
        let substate_lock = self
            .locks
            .remove(&lock_handle)
            .ok_or(UnlockSubstateError::LockNotFound(lock_handle))?;

        let node_id = &substate_lock.node_id;
        let module_id = substate_lock.module_id;
        let substate_key = &substate_lock.substate_key;

        if substate_lock.flags.contains(LockFlags::MUTABLE) {
            let substate = if let Some(handle) = substate_lock.store_handle {
                store.read_substate(handle)
            } else {
                heap.get_substate(node_id, module_id.into(), substate_key)
                    .expect("Substate locked but missing")
            };
            let references = substate.references();
            let owned_nodes = substate.owned_node_ids();

            // Reserving original Vec element order with `IndexSet`
            let mut new_children: IndexSet<NodeId> = index_set_new();
            for own in owned_nodes {
                if !new_children.insert(own.clone()) {
                    return Err(UnlockSubstateError::ContainsDuplicatedOwns);
                }
            }

            // Check references exist
            for reference in references {
                self.get_node_visibility(reference)
                    .ok_or(UnlockSubstateError::RefNotFound(reference.clone()))?;
            }

            for old_child in &substate_lock.initial_owned_nodes {
                if !new_children.remove(old_child) {
                    // TODO: revisit logic here!
                    if !heap.contains_node(node_id) {
                        return Err(UnlockSubstateError::CantDropNodeInStore(old_child.clone()));
                    }

                    // Owned nodes discarded by the substate go back to the call frame,
                    // and must be explicitly dropped.
                    self.owned_root_nodes.insert(old_child.clone(), 0);
                }
            }

            for child_id in &new_children {
                self.take_node_internal(child_id)
                    .map_err(UnlockSubstateError::MoveError)?;
            }

            if !heap.contains_node(&node_id) {
                for child in &new_children {
                    Self::move_node_to_store(heap, store, child)?;
                }
            }
        }

        // TODO: revisit this reference shrinking
        for refed_node in substate_lock.initial_references {
            let cnt = self.temp_node_refs.remove(&refed_node).unwrap_or(0);
            if cnt > 1 {
                self.temp_node_refs.insert(refed_node, cnt - 1);
            }
        }

        // Update node lock count
        if let Some(counter) = self.owned_root_nodes.get_mut(&substate_lock.node_id) {
            *counter -= 1;
        }

        // Release track lock
        if let Some(handle) = substate_lock.store_handle {
            store.release_lock(handle);
        }

        Ok(())
    }

    pub fn get_lock_info(&self, lock_handle: LockHandle) -> Option<LockInfo> {
        self.locks.get(&lock_handle).map(|substate_lock| LockInfo {
            node_id: substate_lock.node_id,
            module_id: substate_lock.module_id,
            substate_key: substate_lock.substate_key.clone(),
            flags: substate_lock.flags,
        })
    }

    pub fn read_substate<'f, S: SubstateStore>(
        &mut self,
        heap: &'f mut Heap,
        store: &'f mut S,
        lock_handle: LockHandle,
    ) -> Result<&'f IndexedScryptoValue, ReadSubstateError> {
        let SubstateLock {
            node_id,
            module_id,
            substate_key,
            store_handle,
            ..
        } = self
            .locks
            .get(&lock_handle)
            .ok_or(ReadSubstateError::LockNotFound(lock_handle))?;

        if let Some(store_handle) = store_handle {
            Ok(store.read_substate(*store_handle))
        } else {
            Ok(heap
                .get_substate(node_id, *module_id, substate_key)
                .expect("Substate missing in heap"))
        }
    }

    pub fn write_substate<'f, S: SubstateStore>(
        &mut self,
        heap: &'f mut Heap,
        store: &'f mut S,
        lock_handle: LockHandle,
        substate: IndexedScryptoValue,
    ) -> Result<(), WriteSubstateError> {
        let SubstateLock {
            node_id,
            module_id,
            substate_key,
            store_handle,
            flags,
            ..
        } = self
            .locks
            .get(&lock_handle)
            .ok_or(WriteSubstateError::LockNotFound(lock_handle))?;

        if !flags.contains(LockFlags::MUTABLE) {
            return Err(WriteSubstateError::NoWritePermission);
        }

        if let Some(store_handle) = store_handle {
            store.update_substate(*store_handle, substate);
        } else {
            heap.set_substate(*node_id, *module_id, substate_key.clone(), substate);
        }
        Ok(())
    }

    // Substate Virtualization does not apply to this call
    // Should this be prevented at this layer?
    pub fn set_substate<'f, S: SubstateStore>(
        &mut self,
        node_id: &NodeId,
        module_id: ModuleId,
        key: SubstateKey,
        value: IndexedScryptoValue,
        heap: &'f mut Heap,
        store: &'f mut S,
    ) -> Result<(), CallFrameSetSubstateError> {
        self.get_node_visibility(node_id)
            .ok_or_else(|| CallFrameSetSubstateError::NodeNotInCallFrame(node_id.clone()))?;

        if heap.contains_node(node_id) {
            heap.set_substate(*node_id, module_id.into(), key, value);
        } else {
            store
                .set_substate(*node_id, module_id, key, value)
                .map_err(|e| CallFrameSetSubstateError::StoreError(e))?;
        };

        Ok(())
    }

    pub fn remove_substate<'f, S: SubstateStore>(
        &mut self,
        node_id: &NodeId,
        module_id: ModuleId,
        key: &SubstateKey,
        heap: &'f mut Heap,
        store: &'f mut S,
    ) -> Result<Option<IndexedScryptoValue>, CallFrameRemoveSubstateError> {
        self.get_node_visibility(node_id)
            .ok_or_else(|| CallFrameRemoveSubstateError::NodeNotInCallFrame(node_id.clone()))?;

        let removed = if heap.contains_node(node_id) {
            heap.delete_substate(node_id, module_id.into(), key)
        } else {
            store
                .take_substate(node_id, module_id.into(), key)
                .map_err(|e| CallFrameRemoveSubstateError::StoreError(e))?
        };

        Ok(removed)
    }

    pub fn scan_substates<'f, S: SubstateStore>(
        &mut self,
        node_id: &NodeId,
        module_id: SysModuleId,
        count: u32,
        heap: &'f mut Heap,
        store: &'f mut S,
    ) -> Result<Vec<IndexedScryptoValue>, CallFrameScanSubstateError> {
        self.get_node_visibility(node_id)
            .ok_or_else(|| CallFrameScanSubstateError::NodeNotInCallFrame(node_id.clone()))?;

        let substates = if heap.contains_node(node_id) {
            heap.scan_substates(node_id, module_id.into(), count)
        } else {
            store.scan_substates(node_id, module_id.into(), count)
        };

        for substate in &substates {
            let refs = substate.references();
            // TODO: verify that refs does not have local refs
            for node_ref in refs {
                self.immortal_node_refs.insert(
                    node_ref.clone(),
                    RENodeRefData {
                        ref_type: RefType::Normal,
                    },
                );
            }
        }

        Ok(substates)
    }

    pub fn take_substates<'f, S: SubstateStore>(
        &mut self,
        node_id: &NodeId,
        module_id: SysModuleId,
        count: u32,
        heap: &'f mut Heap,
        store: &'f mut S,
    ) -> Result<Vec<IndexedScryptoValue>, CallFrameTakeSortedSubstatesError> {
        self.get_node_visibility(node_id).ok_or_else(|| {
            CallFrameTakeSortedSubstatesError::NodeNotInCallFrame(node_id.clone())
        })?;

        let substates = if heap.contains_node(node_id) {
            heap.take_substates(node_id, module_id.into(), count)
        } else {
            store.take_substates(node_id, module_id.into(), count)
        };

        for substate in &substates {
            let refs = substate.references();
            // TODO: verify that refs does not have local refs
            for node_ref in refs {
                self.immortal_node_refs.insert(
                    node_ref.clone(),
                    RENodeRefData {
                        ref_type: RefType::Normal,
                    },
                );
            }
        }

        Ok(substates)
    }

    // Substate Virtualization does not apply to this call
    // Should this be prevented at this layer?
    pub fn scan_sorted<'f, S: SubstateStore>(
        &mut self,
        node_id: &NodeId,
        module_id: ModuleId,
        count: u32,
        heap: &'f mut Heap,
        store: &'f mut S,
    ) -> Result<Vec<IndexedScryptoValue>, CallFrameScanSortedSubstatesError> {
        self.get_node_visibility(node_id).ok_or_else(|| {
            CallFrameScanSortedSubstatesError::NodeNotInCallFrame(node_id.clone())
        })?;

        let substates = if heap.contains_node(node_id) {
            todo!()
        } else {
            store.scan_sorted_substates(node_id, module_id, count)
        };

        for substate in &substates {
            let refs = substate.references();
            // TODO: verify that refs does not have local refs
            for node_ref in refs {
                self.immortal_node_refs.insert(
                    node_ref.clone(),
                    RENodeRefData {
                        ref_type: RefType::Normal,
                    },
                );
            }
        }

        Ok(substates)
    }

    pub fn new_root() -> Self {
        let mut frame = Self {
            depth: 0,
            actor: None,
            immortal_node_refs: NonIterMap::new(),
            temp_node_refs: NonIterMap::new(),
            owned_root_nodes: index_map_new(),
            next_lock_handle: 0u32,
            locks: index_map_new(),
        };

        // Add well-known global refs to current frame
        frame.add_ref(RADIX_TOKEN.into(), RefType::Normal);
        frame.add_ref(SYSTEM_TOKEN.into(), RefType::Normal);
        frame.add_ref(ECDSA_SECP256K1_TOKEN.into(), RefType::Normal);
        frame.add_ref(EDDSA_ED25519_TOKEN.into(), RefType::Normal);
        frame.add_ref(PACKAGE_TOKEN.into(), RefType::Normal);
        frame.add_ref(PACKAGE_OWNER_TOKEN.into(), RefType::Normal);
        frame.add_ref(VALIDATOR_OWNER_TOKEN.into(), RefType::Normal);
        frame.add_ref(IDENTITY_OWNER_TOKEN.into(), RefType::Normal);
        frame.add_ref(ACCOUNT_OWNER_TOKEN.into(), RefType::Normal);
        frame.add_ref(EPOCH_MANAGER.into(), RefType::Normal);
        frame.add_ref(CLOCK.into(), RefType::Normal);
        frame.add_ref(ACCESS_CONTROLLER_PACKAGE.into(), RefType::Normal);
        frame.add_ref(ACCOUNT_PACKAGE.into(), RefType::Normal);
        frame.add_ref(CLOCK_PACKAGE.into(), RefType::Normal);
        frame.add_ref(EPOCH_MANAGER_PACKAGE.into(), RefType::Normal);
        frame.add_ref(PACKAGE_PACKAGE.into(), RefType::Normal);
        frame.add_ref(RESOURCE_MANAGER_PACKAGE.into(), RefType::Normal);
        frame.add_ref(TRANSACTION_PROCESSOR_PACKAGE.into(), RefType::Normal);
        frame.add_ref(FAUCET_PACKAGE.into(), RefType::Normal);

        frame
    }

    pub fn new_child_from_parent(
        parent: &mut CallFrame,
        actor: Actor,
        call_frame_update: CallFrameUpdate,
    ) -> Result<Self, MoveError> {
        let mut owned_heap_nodes = index_map_new();
        let mut next_node_refs = NonIterMap::new();

        for node_id in call_frame_update.nodes_to_move {
            parent.take_node_internal(&node_id)?;
            owned_heap_nodes.insert(node_id, 0u32);
        }

        for node_id in call_frame_update.node_refs_to_copy {
            let visibility = parent
                .get_node_visibility(&node_id)
                .ok_or(MoveError::RefNotFound(node_id))?;
            next_node_refs.insert(node_id, RENodeRefData::new(visibility.0));
        }

        let frame = Self {
            depth: parent.depth + 1,
            actor: Some(actor),
            immortal_node_refs: next_node_refs,
            temp_node_refs: NonIterMap::new(),
            owned_root_nodes: owned_heap_nodes,
            next_lock_handle: 0u32,
            locks: index_map_new(),
        };

        Ok(frame)
    }

    pub fn update_upstream(
        from: &mut CallFrame,
        to: &mut CallFrame,
        update: CallFrameUpdate,
    ) -> Result<(), MoveError> {
        for node_id in update.nodes_to_move {
            // move re nodes to upstream call frame.
            from.take_node_internal(&node_id)?;
            to.owned_root_nodes.insert(node_id, 0u32);
        }

        for node_id in update.node_refs_to_copy {
            // Make sure not to allow owned nodes to be passed as references upstream
            let ref_data = from
                .immortal_node_refs
                .get(&node_id)
                .ok_or(MoveError::RefNotFound(node_id))?;

            to.immortal_node_refs
                .entry(node_id)
                .and_modify(|e| {
                    if e.ref_type == RefType::DirectAccess {
                        e.ref_type = ref_data.ref_type
                    }
                })
                .or_insert(ref_data.clone());
        }

        Ok(())
    }

    pub fn drop_all_locks<S: SubstateStore>(
        &mut self,
        heap: &mut Heap,
        store: &mut S,
    ) -> Result<(), UnlockSubstateError> {
        let lock_handles: Vec<LockHandle> = self.locks.keys().cloned().collect();

        for lock_handle in lock_handles {
            self.drop_lock(heap, store, lock_handle)?;
        }

        Ok(())
    }

    fn take_node_internal(&mut self, node_id: &NodeId) -> Result<(), MoveError> {
        match self.owned_root_nodes.remove(node_id) {
            None => {
                return Err(MoveError::OwnNotFound(node_id.clone()));
            }
            Some(lock_count) => {
                if lock_count == 0 {
                    Ok(())
                } else {
                    Err(MoveError::CantMoveLockedNode(node_id.clone()))
                }
            }
        }
    }

    pub fn create_node<'f, S: SubstateStore>(
        &mut self,
        node_id: NodeId,
        node_substates: NodeSubstates,
        heap: &mut Heap,
        store: &'f mut S,
        push_to_store: bool,
    ) -> Result<(), UnlockSubstateError> {
        for (_module_id, module) in &node_substates {
            for (_substate_key, substate_value) in module {
                // FIXME there is a huge mismatch between drop_lock and create_node
                // We need to apply the same checks!
                for child_id in substate_value.owned_node_ids() {
                    self.take_node_internal(child_id)
                        .map_err(UnlockSubstateError::MoveError)?;
                    if push_to_store {
                        Self::move_node_to_store(heap, store, child_id)?;
                    }
                }

                if push_to_store {
                    for reference in substate_value.references() {
                        if !reference.is_global() {
                            return Err(UnlockSubstateError::CantStoreLocalReference(*reference));
                        }
                    }
                }
            }
        }

        if push_to_store {
            store.create_node(node_id, node_substates);
            self.add_ref(node_id, RefType::Normal);
        } else {
            heap.create_node(node_id, node_substates);
            self.owned_root_nodes.insert(node_id, 0u32);
        }

        Ok(())
    }

    pub fn add_ref(&mut self, node_id: NodeId, visibility: RefType) {
        self.immortal_node_refs
            .insert(node_id, RENodeRefData::new(visibility));
    }

    pub fn owned_nodes(&self) -> Vec<NodeId> {
        self.owned_root_nodes.keys().cloned().collect()
    }

    /// Removes node from call frame and re-owns any children
    pub fn remove_node(
        &mut self,
        heap: &mut Heap,
        node_id: &NodeId,
    ) -> Result<NodeSubstates, MoveError> {
        self.take_node_internal(node_id)?;
        let node_substates = heap.remove_node(node_id);
        for (_, module) in &node_substates {
            for (_, substate_value) in module {
                let refs = substate_value.references();
                let child_nodes = substate_value.owned_node_ids();
                for node_ref in refs {
                    self.immortal_node_refs.insert(
                        node_ref.clone(),
                        RENodeRefData {
                            ref_type: RefType::Normal,
                        },
                    );
                }

                for child_node in child_nodes {
                    self.owned_root_nodes.insert(child_node.clone(), 0u32);
                }
            }
        }
        Ok(node_substates)
    }

    pub fn move_node_to_store<S: SubstateStore>(
        heap: &mut Heap,
        store: &mut S,
        node_id: &NodeId,
    ) -> Result<(), UnlockSubstateError> {
        // FIXME: Clean this up
        let can_be_stored = if node_id.is_global() {
            true
        } else {
            if let Some(type_info) = Self::get_type_info(node_id, heap, store) {
                match type_info {
                    TypeInfoSubstate::Object(ObjectInfo { blueprint, .. })
                        if blueprint.package_address == RESOURCE_MANAGER_PACKAGE
                            && (blueprint.blueprint_name == FUNGIBLE_BUCKET_BLUEPRINT
                                || blueprint.blueprint_name == NON_FUNGIBLE_BUCKET_BLUEPRINT
                                || blueprint.blueprint_name == PROOF_BLUEPRINT) =>
                    {
                        false
                    }
                    _ => true,
                }
            } else {
                false
            }
        };
        if !can_be_stored {
            return Err(UnlockSubstateError::CantBeStored(node_id.clone()));
        }

        let node_substates = heap.remove_node(node_id);
        for (_module_id, module_substates) in &node_substates {
            for (_substate_key, substate_value) in module_substates {
                for reference in substate_value.references() {
                    if !reference.is_global() {
                        return Err(UnlockSubstateError::CantStoreLocalReference(*reference));
                    }
                }

                for node in substate_value.owned_node_ids() {
                    Self::move_node_to_store(heap, store, node)?;
                }
            }
        }

        store.create_node(node_id.clone(), node_substates);

        Ok(())
    }

    pub fn get_node_visibility(&self, node_id: &NodeId) -> Option<(RefType, bool)> {
        if self.owned_root_nodes.contains_key(node_id) {
            Some((RefType::Normal, true))
        } else if let Some(_) = self.temp_node_refs.get(node_id) {
            Some((RefType::Normal, false))
        } else if let Some(ref_data) = self.immortal_node_refs.get(node_id) {
            Some((ref_data.ref_type, false))
        } else {
            None
        }
    }
}
