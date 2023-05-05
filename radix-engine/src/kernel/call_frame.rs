use crate::kernel::actor::Actor;
use crate::system::node_modules::type_info::TypeInfoSubstate;
use crate::track::interface::{
    AcquireLockError, NodeSubstates, SetSubstateError, SubstateStore, TakeSubstateError,
};
use crate::types::*;
use radix_engine_interface::api::field_lock_api::LockFlags;
use radix_engine_interface::blueprints::resource::{
    FUNGIBLE_BUCKET_BLUEPRINT, FUNGIBLE_PROOF_BLUEPRINT, NON_FUNGIBLE_BUCKET_BLUEPRINT,
    NON_FUNGIBLE_PROOF_BLUEPRINT,
};
use radix_engine_interface::types::{LockHandle, NodeId, SubstateKey};

use super::actor::MethodActor;
use super::heap::Heap;
use super::kernel_api::LockInfo;

/// A message used for communication between call frames.
///
/// Note that it's just an intent, not checked/allowed by kernel yet.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub copy_references: Vec<NodeId>,
    pub move_nodes: Vec<NodeId>,
}

impl Message {
    pub fn from_indexed_scrypto_value(value: &IndexedScryptoValue) -> Self {
        Self {
            copy_references: value.references().clone(),
            move_nodes: value.owned_nodes().clone(),
        }
    }

    pub fn add_copy_reference(&mut self, node_id: NodeId) {
        self.copy_references.push(node_id)
    }

    pub fn add_move_node(&mut self, node_id: NodeId) {
        self.move_nodes.push(node_id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RENodeLocation {
    Heap,
    Store,
}

/// A lock on a substate controlled by a call frame
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubstateLock<L> {
    pub node_id: NodeId,
    pub module_num: ModuleNumber,
    pub substate_key: SubstateKey,
    pub initial_references: IndexSet<NodeId>,
    pub initial_owned_nodes: IndexSet<NodeId>,
    pub flags: LockFlags,
    pub store_handle: Option<u32>,
    pub data: L,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum StableReferenceType {
    Global,
    DirectAccess,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Visibility {
    StableReference(StableReferenceType),
    FrameOwned,
    Actor,
    Borrowed,
}

impl Visibility {
    pub fn is_direct_access(&self) -> bool {
        matches!(
            self,
            Self::StableReference(StableReferenceType::DirectAccess)
        )
    }

    pub fn is_normal(&self) -> bool {
        !self.is_direct_access()
    }
}

// TODO: reduce fields visibility

/// A call frame is the basic unit that forms a transaction call stack, which keeps track of the
/// owned objects by this function.
pub struct CallFrame<L> {
    /// The frame id
    depth: usize,

    /// The running application actor of this frame
    /// TODO: Move to an RENode
    actor: Option<Actor>,

    /// Owned nodes which by definition must live on heap
    /// Also keeps track of number of locks on this node, to prevent locked node from moving.
    owned_root_nodes: IndexMap<NodeId, usize>,

    /// References to non-GLOBAL nodes, obtained from substate loading, ref counted.
    transient_references: NonIterMap<NodeId, usize>,

    /// Stable references points to nodes in track, which can't moved/deleted.
    /// Current two types: `GLOBAL` (root, stored) and `DirectAccess`.
    stable_references: NonIterMap<NodeId, StableReferenceType>,

    next_lock_handle: LockHandle,
    locks: IndexMap<LockHandle, SubstateLock<L>>,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum CreateFrameError {
    ActorBeingMoved(NodeId),
    MessagePassingError(PassMessageError),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum PassMessageError {
    MoveNodeError(TakeNodeError),
    StableRefNotFound(NodeId),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum LockSubstateError {
    NodeNotVisible(NodeId),
    LockUnmodifiedBaseOnHeapNode,
    SubstateNotFound(NodeId, ModuleNumber, SubstateKey),
    TrackError(Box<AcquireLockError>),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum UnlockSubstateError {
    LockNotFound(LockHandle),
    ContainsDuplicatedOwns,
    RefNotFound(NodeId),
    TakeNodeError(TakeNodeError),
    CantDropNodeInStore(NodeId),
    CantOwn(NodeId),
    MoveToStoreError(MoveToStoreError),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum CreateNodeError {
    LockNotFound(LockHandle),
    ContainsDuplicatedOwns,
    RefNotFound(NodeId),
    TakeNodeError(TakeNodeError),
    CantDropNodeInStore(NodeId),
    CantOwn(NodeId),
    MoveToStoreError(MoveToStoreError),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum MoveToStoreError {
    CantStoreLocalReference(NodeId),
    CantBeStored(NodeId),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum TakeNodeError {
    OwnNotFound(NodeId),
    OwnLocked(NodeId),
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
    NodeNotVisible(NodeId),
    StoreError(SetSubstateError),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum CallFrameRemoveSubstateError {
    NodeNotVisible(NodeId),
    StoreError(TakeSubstateError),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum CallFrameScanSubstateError {
    NodeNotVisible(NodeId),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum CallFrameScanSortedSubstatesError {
    NodeNotVisible(NodeId),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum CallFrameTakeSortedSubstatesError {
    NodeNotVisible(NodeId),
}

impl<L: Clone> CallFrame<L> {
    pub fn new_root() -> Self {
        Self {
            depth: 0,
            actor: None,
            stable_references: NonIterMap::new(),
            transient_references: NonIterMap::new(),
            owned_root_nodes: index_map_new(),
            next_lock_handle: 0u32,
            locks: index_map_new(),
        }
    }

    pub fn new_child_from_parent(
        parent: &mut CallFrame<L>,
        actor: Actor,
        message: Message,
    ) -> Result<Self, CreateFrameError> {
        let optional_method_actor = actor.try_as_method().cloned();
        let mut frame = Self {
            depth: parent.depth + 1,
            actor: Some(actor),
            stable_references: NonIterMap::new(),
            transient_references: NonIterMap::new(),
            owned_root_nodes: index_map_new(),
            next_lock_handle: 0u32,
            locks: index_map_new(),
        };

        // Copy references and move nodes
        Self::exchange(parent, &mut frame, message)
            .map_err(CreateFrameError::MessagePassingError)?;

        // Additional logic on actor
        if let Some(method_actor) = optional_method_actor {
            if frame.owned_root_nodes.contains_key(&method_actor.node_id) {
                return Err(CreateFrameError::ActorBeingMoved(method_actor.node_id));
            }
            if let Some(outer_global_object) = method_actor.object_info.outer_object {
                frame.stable_references.insert(
                    outer_global_object.into_node_id(),
                    StableReferenceType::Global,
                );
            }
        }

        Ok(frame)
    }

    pub fn exchange(
        from: &mut CallFrame<L>,
        to: &mut CallFrame<L>,
        message: Message,
    ) -> Result<(), PassMessageError> {
        for node_id in message.move_nodes {
            // Note that this has no impact on the `transient_references` because
            // we don't allow move of "locked nodes".
            from.take_node_internal(&node_id)
                .map_err(PassMessageError::MoveNodeError)?;
            to.owned_root_nodes.insert(node_id, 0);
        }

        for node_id in message.copy_references {
            let reference_type = from
                .stable_references
                .get(&node_id)
                .ok_or_else(|| PassMessageError::StableRefNotFound(node_id))?;

            // Note that GLOBAL and DirectAccess reference can't co-exist,
            // so it's safe to overwrite.
            to.stable_references.insert(node_id, reference_type.clone());
        }

        Ok(())
    }

    pub fn depth(&self) -> usize {
        self.depth
    }

    pub fn actor(&self) -> &Option<Actor> {
        &self.actor
    }

    // TODO: Remove
    fn get_type_info<S: SubstateStore>(
        node_id: &NodeId,
        heap: &mut Heap,
        store: &mut S,
    ) -> Option<TypeInfoSubstate> {
        if let Some(substate) = heap.get_substate(
            node_id,
            TYPE_INFO_BASE_MODULE,
            &TypeInfoOffset::TypeInfo.into(),
        ) {
            let type_info: TypeInfoSubstate = substate.as_typed().unwrap();
            Some(type_info)
        } else if let Ok((handle, _)) = store.acquire_lock(
            node_id,
            TYPE_INFO_BASE_MODULE,
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
        module_num: ModuleNumber,
        substate_key: &SubstateKey,
        flags: LockFlags,
        default: Option<fn() -> IndexedScryptoValue>,
        data: L,
    ) -> Result<(LockHandle, bool), LockSubstateError> {
        // Check node visibility
        if !is_lock_substate_allowed(&self.get_node_visibility(node_id)) {
            return Err(LockSubstateError::NodeNotVisible(node_id.clone()));
        }

        // Lock and read the substate
        let mut store_handle = None;
        let mut first_time_lock = false;
        let substate_value = if heap.contains_node(node_id) {
            // TODO: make Heap more like Store?
            if flags.contains(LockFlags::UNMODIFIED_BASE) {
                return Err(LockSubstateError::LockUnmodifiedBaseOnHeapNode);
            }
            if let Some(compute_default) = default {
                heap.get_substate_virtualize(node_id, module_num, substate_key, compute_default)
            } else {
                heap.get_substate(node_id, module_num, substate_key)
                    .ok_or_else(|| {
                        LockSubstateError::SubstateNotFound(
                            node_id.clone(),
                            module_num,
                            substate_key.clone(),
                        )
                    })?
            }
        } else {
            let (handle, first_time) = store
                .acquire_lock_virtualize(node_id, module_num, substate_key, flags, || {
                    default.map(|f| f())
                })
                .map_err(|x| LockSubstateError::TrackError(Box::new(x)))?;
            store_handle = Some(handle);
            first_time_lock = first_time;
            store.read_substate(handle)
        };

        // Analyze owns and references in the substate
        let mut initial_references = index_set_new(); // du-duplicated
        let mut initial_owned_nodes = index_set_new();
        for node_id in substate_value.references() {
            if node_id.is_global() {
                // Again, safe to overwrite because Global and DirectAccess are exclusive.
                self.stable_references
                    .insert(node_id.clone(), StableReferenceType::Global);
            } else {
                initial_references.insert(node_id.clone());
            }
        }
        for node_id in substate_value.owned_nodes() {
            initial_references.insert(node_id.clone());
            if !initial_owned_nodes.insert(node_id.clone()) {
                panic!("Duplicated own found in substate");
            }
        }

        // Expand transient references with new references released from the substate
        for node_id in &initial_references {
            self.transient_references
                .entry(node_id.clone())
                .or_default()
                .add_assign(1);
        }

        let lock_handle = self.next_lock_handle;
        self.locks.insert(
            lock_handle,
            SubstateLock {
                node_id: node_id.clone(),
                module_num,
                substate_key: substate_key.clone(),
                initial_references,
                initial_owned_nodes,
                flags,
                store_handle,
                data,
            },
        );
        self.next_lock_handle = self.next_lock_handle + 1;

        // Update lock count on the node
        if let Some(counter) = self.owned_root_nodes.get_mut(node_id) {
            *counter += 1;
        }

        Ok((lock_handle, first_time_lock))
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
        let module_num = substate_lock.module_num;
        let substate_key = &substate_lock.substate_key;

        if substate_lock.flags.contains(LockFlags::MUTABLE) {
            let substate = if let Some(handle) = substate_lock.store_handle {
                store.read_substate(handle)
            } else {
                heap.get_substate(node_id, module_num, substate_key)
                    .expect("Substate locked but missing")
            };

            // Process owns
            let mut new_owned_nodes: IndexSet<NodeId> = index_set_new();
            for own in substate.owned_nodes() {
                if !new_owned_nodes.insert(own.clone()) {
                    return Err(UnlockSubstateError::ContainsDuplicatedOwns);
                }
            }
            for own in &new_owned_nodes {
                if !substate_lock.initial_owned_nodes.contains(own) {
                    // Node no longer owned by frame
                    self.take_node_internal(own)
                        .map_err(UnlockSubstateError::TakeNodeError)?;

                    // Move the taken node to store, if parent is in store
                    if !heap.contains_node(&node_id) {
                        for child in &new_owned_nodes {
                            Self::move_node_to_store(heap, store, child)?;
                        }
                    }
                }
            }
            for own in &substate_lock.initial_owned_nodes {
                if !new_owned_nodes.contains(own) {
                    // Node detached
                    if !heap.contains_node(node_id) {
                        return Err(UnlockSubstateError::CantDropNodeInStore(own.clone()));
                    }
                    // Owned nodes discarded by the substate go back to the call frame,
                    // and must be explicitly dropped.
                    // FIXME: Yulong suspects this is buggy as one can detach a locked non-root
                    // node, move and drop; which will cause invalid lock handle in previous frames.
                    self.owned_root_nodes.insert(own.clone(), 0);
                }
            }

            // Process references
            let mut new_references: IndexSet<NodeId> = index_set_new();
            for own in substate.references() {
                // Deduplicate
                new_references.insert(own.clone());
            }
            for reference in &new_references {
                if !substate_lock.initial_references.contains(reference) {
                    if !self
                        .get_node_visibility(reference)
                        .iter()
                        .any(|v| v.is_normal())
                    {
                        return Err(UnlockSubstateError::RefNotFound(reference.clone()));
                    }
                }
            }
            for reference in &substate_lock.initial_references {
                if !new_references.contains(reference) {
                    if heap.contains_node(reference) {
                        // TODO: this substate no longer borrows the node
                    }
                }
            }
        }

        // Made substate expanded owns/reference invisible.
        for refed_node in substate_lock.initial_references {
            let cnt = self.transient_references.remove(&refed_node).unwrap_or(0);
            if cnt > 1 {
                self.transient_references.insert(refed_node, cnt - 1);
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

    pub fn get_lock_info(&self, lock_handle: LockHandle) -> Option<LockInfo<L>> {
        self.locks.get(&lock_handle).map(|substate_lock| LockInfo {
            node_id: substate_lock.node_id,
            module_num: substate_lock.module_num,
            substate_key: substate_lock.substate_key.clone(),
            flags: substate_lock.flags,
            data: substate_lock.data.clone(),
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
            module_num,
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
                .get_substate(node_id, *module_num, substate_key)
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
            module_num,
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
            heap.set_substate(*node_id, *module_num, substate_key.clone(), substate);
        }
        Ok(())
    }

    pub fn create_node<'f, S: SubstateStore>(
        &mut self,
        node_id: NodeId,
        node_substates: NodeSubstates,
        heap: &mut Heap,
        store: &'f mut S,
    ) -> Result<(), CreateNodeError> {
        let push_to_store = node_id.is_global();

        for (_module_id, module) in &node_substates {
            for (_substate_key, substate_value) in module {
                // Process own
                for own in substate_value.owned_nodes() {
                    self.take_node_internal(own)
                        .map_err(CreateNodeError::TakeNodeError)?;
                    if push_to_store {
                        Self::move_node_to_store(heap, store, own)
                            .map_err(CreateNodeError::MoveToStoreError)?;
                    }
                }

                // Process reference
                for reference in substate_value.references() {
                    if !self
                        .get_node_visibility(reference)
                        .iter()
                        .any(|v| v.is_normal())
                    {
                        return Err(CreateNodeError::RefNotFound(reference.clone()));
                    }
                }
            }
        }

        if push_to_store {
            store.create_node(node_id, node_substates);
            self.stable_references
                .insert(node_id, StableReferenceType::Global);
        } else {
            heap.create_node(node_id, node_substates);
            self.owned_root_nodes.insert(node_id, 0);
        }

        Ok(())
    }

    /// Removes node from call frame and re-owns any children
    pub fn remove_node(
        &mut self,
        heap: &mut Heap,
        node_id: &NodeId,
    ) -> Result<NodeSubstates, TakeNodeError> {
        self.take_node_internal(node_id)?;
        let node_substates = heap.remove_node(node_id);
        for (_, module) in &node_substates {
            for (_, substate_value) in module {
                let refs = substate_value.references();
                let child_nodes = substate_value.owned_nodes();
                for node_ref in refs {
                    self.stable_references.insert(
                        node_ref.clone(),
                        StableReferenceType {
                            ref_type: Visibility::Normal,
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

    // Note that the set/remove/scan/take APIs aren't compatible with our reference model.
    // They're intended for internal use only and extra caution is required.

    // Substate Virtualization does not apply to this call
    // Should this be prevented at this layer?
    pub fn set_substate<'f, S: SubstateStore>(
        &mut self,
        node_id: &NodeId,
        module_num: ModuleNumber,
        key: SubstateKey,
        value: IndexedScryptoValue,
        heap: &'f mut Heap,
        store: &'f mut S,
    ) -> Result<(), CallFrameSetSubstateError> {
        // Check node visibility
        if !is_lock_substate_allowed(&self.get_node_visibility(node_id)) {
            return Err(CallFrameSetSubstateError::NodeNotVisible(node_id.clone()));
        }

        if heap.contains_node(node_id) {
            heap.set_substate(*node_id, module_num, key, value);
        } else {
            store
                .set_substate(*node_id, module_num, key, value)
                .map_err(|e| CallFrameSetSubstateError::StoreError(e))?;
        };

        Ok(())
    }

    pub fn remove_substate<'f, S: SubstateStore>(
        &mut self,
        node_id: &NodeId,
        module_num: ModuleNumber,
        key: &SubstateKey,
        heap: &'f mut Heap,
        store: &'f mut S,
    ) -> Result<Option<IndexedScryptoValue>, CallFrameRemoveSubstateError> {
        // Check node visibility
        if !is_lock_substate_allowed(&self.get_node_visibility(node_id)) {
            return Err(CallFrameRemoveSubstateError::NodeNotVisible(
                node_id.clone(),
            ));
        }

        let removed = if heap.contains_node(node_id) {
            heap.delete_substate(node_id, module_num, key)
        } else {
            store
                .take_substate(node_id, module_num, key)
                .map_err(|e| CallFrameRemoveSubstateError::StoreError(e))?
        };

        Ok(removed)
    }

    pub fn scan_substates<'f, S: SubstateStore>(
        &mut self,
        node_id: &NodeId,
        module_num: ModuleNumber,
        count: u32,
        heap: &'f mut Heap,
        store: &'f mut S,
    ) -> Result<Vec<IndexedScryptoValue>, CallFrameScanSubstateError> {
        // Check node visibility
        if !is_lock_substate_allowed(&self.get_node_visibility(node_id)) {
            return Err(CallFrameScanSubstateError::NodeNotVisible(node_id.clone()));
        }

        let substates = if heap.contains_node(node_id) {
            heap.scan_substates(node_id, module_num, count)
        } else {
            store.scan_substates(node_id, module_num, count)
        };

        for substate in &substates {
            for reference in substate.references() {
                if reference.is_global() {
                    self.stable_references
                        .insert(reference.clone(), StableReferenceType::Global);
                } else {
                    // TODO: check if non-global reference is needed
                }
            }
        }

        Ok(substates)
    }

    pub fn take_substates<'f, S: SubstateStore>(
        &mut self,
        node_id: &NodeId,
        module_num: ModuleNumber,
        count: u32,
        heap: &'f mut Heap,
        store: &'f mut S,
    ) -> Result<Vec<IndexedScryptoValue>, CallFrameTakeSortedSubstatesError> {
        // Check node visibility
        if !is_lock_substate_allowed(&self.get_node_visibility(node_id)) {
            return Err(CallFrameTakeSortedSubstatesError::NodeNotVisible(
                node_id.clone(),
            ));
        }

        let substates = if heap.contains_node(node_id) {
            heap.take_substates(node_id, module_num, count)
        } else {
            store.take_substates(node_id, module_num, count)
        };

        for substate in &substates {
            for reference in substate.references() {
                if reference.is_global() {
                    self.stable_references
                        .insert(reference.clone(), StableReferenceType::Global);
                } else {
                    // TODO: check if non-global reference is needed
                }
            }
        }

        Ok(substates)
    }

    // Substate Virtualization does not apply to this call
    // Should this be prevented at this layer?
    pub fn scan_sorted<'f, S: SubstateStore>(
        &mut self,
        node_id: &NodeId,
        module_num: ModuleNumber,
        count: u32,
        heap: &'f mut Heap,
        store: &'f mut S,
    ) -> Result<Vec<IndexedScryptoValue>, CallFrameScanSortedSubstatesError> {
        // Check node visibility
        if !is_lock_substate_allowed(&self.get_node_visibility(node_id)) {
            return Err(CallFrameScanSortedSubstatesError::NodeNotVisible(
                node_id.clone(),
            ));
        }

        let substates = if heap.contains_node(node_id) {
            todo!()
        } else {
            store.scan_sorted_substates(node_id, module_num, count)
        };

        for substate in &substates {
            for reference in substate.references() {
                if reference.is_global() {
                    self.stable_references
                        .insert(reference.clone(), StableReferenceType::Global);
                } else {
                    // TODO: check if non-global reference is needed
                }
            }
        }

        Ok(substates)
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

    fn take_node_internal(&mut self, node_id: &NodeId) -> Result<(), TakeNodeError> {
        match self.owned_root_nodes.remove(node_id) {
            None => {
                return Err(TakeNodeError::OwnNotFound(node_id.clone()));
            }
            Some(lock_count) => {
                if lock_count == 0 {
                    Ok(())
                } else {
                    Err(TakeNodeError::OwnLocked(node_id.clone()))
                }
            }
        }
    }

    pub fn owned_nodes(&self) -> Vec<NodeId> {
        self.owned_root_nodes.keys().cloned().collect()
    }

    pub fn move_node_to_store<S: SubstateStore>(
        heap: &mut Heap,
        store: &mut S,
        node_id: &NodeId,
    ) -> Result<(), MoveToStoreError> {
        // FIXME: Clean this up
        let can_be_stored = if node_id.is_global() {
            true
        } else {
            if let Some(type_info) = Self::get_type_info(node_id, heap, store) {
                match type_info {
                    TypeInfoSubstate::Object(ObjectInfo { blueprint, .. })
                        if blueprint.package_address == RESOURCE_PACKAGE
                            && (blueprint.blueprint_name == FUNGIBLE_BUCKET_BLUEPRINT
                                || blueprint.blueprint_name == NON_FUNGIBLE_BUCKET_BLUEPRINT
                                || blueprint.blueprint_name == FUNGIBLE_PROOF_BLUEPRINT
                                || blueprint.blueprint_name == NON_FUNGIBLE_PROOF_BLUEPRINT) =>
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
            return Err(MoveToStoreError::CantBeStored(node_id.clone()));
        }

        let node_substates = heap.remove_node(node_id);
        for (_module_id, module_substates) in &node_substates {
            for (_substate_key, substate_value) in module_substates {
                for reference in substate_value.references() {
                    if !reference.is_global() {
                        return Err(MoveToStoreError::CantStoreLocalReference(*reference));
                    }
                }

                for node in substate_value.owned_nodes() {
                    Self::move_node_to_store(heap, store, node)?;
                }
            }
        }

        store.create_node(node_id.clone(), node_substates);

        Ok(())
    }

    pub fn get_node_visibility(&self, node_id: &NodeId) -> BTreeSet<Visibility> {
        let mut visibilities = BTreeSet::<Visibility>::new();

        // Stable references
        if let Some(reference_type) = self.stable_references.get(node_id) {
            visibilities.insert(Visibility::StableReference(reference_type.clone()));
        }

        // Frame owned nodes
        if self.owned_root_nodes.contains_key(node_id) {
            visibilities.insert(Visibility::FrameOwned);
        }

        // Actor
        if let Some(Actor::Method(MethodActor {
            node_id: actor_node_id,
            ..
        })) = &self.actor
        {
            if actor_node_id == node_id {
                visibilities.insert(Visibility::Actor);
            }
        }

        // Borrowed from substate loading
        if self.transient_references.contains_key(node_id) {
            visibilities.insert(Visibility::Borrowed);
        }

        visibilities
    }
}

/// Note that system may enforce further constraints on this.
/// For instance, system currently only allow locking substates of actor,
/// actor's outer object, any visible key value store.
pub fn is_lock_substate_allowed(visibilities: &BTreeSet<Visibility>) -> bool {
    visibilities.iter().any(|x| x.is_normal())
}

pub fn is_invoke_allowed(visibilities: &BTreeSet<Visibility>) -> bool {
    !visibilities.is_empty()
}
