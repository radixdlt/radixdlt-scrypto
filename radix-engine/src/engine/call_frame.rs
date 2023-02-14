use crate::engine::system_api::LockInfo;
use crate::engine::*;
use crate::fee::FeeReserve;
use crate::model::{SubstateRef, SubstateRefMut};
use crate::types::*;
use radix_engine_interface::api::types::{
    GlobalAddress, LockHandle, NonFungibleStoreOffset, RENodeId, SubstateId, SubstateOffset,
    TransactionProcessorFn,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallFrameUpdate {
    pub nodes_to_move: Vec<RENodeId>,
    pub node_refs_to_copy: HashSet<RENodeId>,
}

impl CallFrameUpdate {
    pub fn empty() -> Self {
        CallFrameUpdate {
            nodes_to_move: Vec::new(),
            node_refs_to_copy: HashSet::new(),
        }
    }

    pub fn move_node(node_id: RENodeId) -> Self {
        CallFrameUpdate {
            nodes_to_move: vec![node_id],
            node_refs_to_copy: HashSet::new(),
        }
    }

    pub fn copy_ref(node_id: RENodeId) -> Self {
        let mut node_refs_to_copy = HashSet::new();
        node_refs_to_copy.insert(node_id);
        CallFrameUpdate {
            nodes_to_move: vec![],
            node_refs_to_copy,
        }
    }

    pub fn add_ref(&mut self, node_id: RENodeId) {
        self.node_refs_to_copy.insert(node_id);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RENodeLocation {
    Heap,
    Store,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RENodeVisibilityOrigin {
    Normal,
    DirectAccess,
}

/// A lock on a substate controlled by a call frame
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubstateLock {
    pub substate_pointer: (RENodeLocation, RENodeId, SubstateOffset),
    pub global_references: HashSet<GlobalAddress>,
    pub substate_owned_nodes: HashSet<RENodeId>,
    pub flags: LockFlags,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RENodeRefData {
    location: RENodeLocation,
    visibility: RENodeVisibilityOrigin,
}

impl RENodeRefData {
    fn new(location: RENodeLocation, visibility: RENodeVisibilityOrigin) -> Self {
        RENodeRefData {
            location,
            visibility,
        }
    }
}

// TODO: reduce fields visibility

/// A call frame is the basic unit that forms a transaction call stack, which keeps track of the
/// owned objects by this function.
pub struct CallFrame {
    /// The frame id
    pub depth: usize,

    /// The running application actor of this frame
    pub actor: ResolvedActor,

    /// All ref nodes accessible by this call frame (does not include owned nodes).
    node_refs: HashMap<RENodeId, RENodeRefData>,

    /// Owned nodes which by definition must live on heap
    /// Also keeps track of number of locks on this node
    owned_root_nodes: HashMap<RENodeId, u32>,

    next_lock_handle: LockHandle,
    locks: HashMap<LockHandle, SubstateLock>,
}

impl CallFrame {
    pub fn acquire_lock<'s, R: FeeReserve>(
        &mut self,
        heap: &mut Heap,
        track: &mut Track<'s, R>,
        node_id: RENodeId,
        offset: SubstateOffset,
        flags: LockFlags,
    ) -> Result<LockHandle, RuntimeError> {
        let location = self.get_node_location(node_id)?;
        if !(matches!(offset, SubstateOffset::KeyValueStore(..))
            || matches!(
                offset,
                SubstateOffset::NonFungibleStore(NonFungibleStoreOffset::Entry(..))
            ))
        {
            let substate_id = SubstateId(node_id, offset.clone());
            match location {
                RENodeLocation::Store => track
                    .acquire_lock(substate_id, flags)
                    .map_err(KernelError::TrackError),
                RENodeLocation::Heap => {
                    if flags.contains(LockFlags::UNMODIFIED_BASE) {
                        Err(KernelError::TrackError(
                            TrackError::LockUnmodifiedBaseOnNewSubstate(substate_id),
                        ))
                    } else {
                        Ok(())
                    }
                }
            }?;
        }

        let substate_ref = self.get_substate(heap, track, location, node_id, &offset)?;
        let (global_references, substate_owned_nodes) = substate_ref.references_and_owned_nodes();

        // Expand references
        {
            for global_address in &global_references {
                let node_id = RENodeId::Global(global_address.clone());
                self.node_refs.insert(
                    node_id,
                    RENodeRefData::new(RENodeLocation::Store, RENodeVisibilityOrigin::Normal),
                );
            }
            for child_id in &substate_owned_nodes {
                self.node_refs.insert(
                    *child_id,
                    RENodeRefData::new(location, RENodeVisibilityOrigin::Normal),
                );
            }
        }

        let lock_handle = self.next_lock_handle;
        self.locks.insert(
            lock_handle,
            SubstateLock {
                global_references,
                substate_pointer: (location, node_id, offset),
                substate_owned_nodes,
                flags,
            },
        );
        self.next_lock_handle = self.next_lock_handle + 1;

        if let Some(counter) = self.owned_root_nodes.get_mut(&node_id) {
            *counter += 1;
        }

        Ok(lock_handle)
    }

    pub fn drop_lock<'s, R: FeeReserve>(
        &mut self,
        heap: &mut Heap,
        track: &mut Track<'s, R>,
        lock_handle: LockHandle,
    ) -> Result<(), RuntimeError> {
        let substate_lock = self
            .locks
            .remove(&lock_handle)
            .ok_or(KernelError::LockDoesNotExist(lock_handle))?;

        let location = substate_lock.substate_pointer.0;
        let node_id = substate_lock.substate_pointer.1;
        let offset = substate_lock.substate_pointer.2;

        if substate_lock.flags.contains(LockFlags::MUTABLE) {
            let substate_ref = self.get_substate(heap, track, location, node_id, &offset)?;

            let (new_global_references, mut new_children) =
                substate_ref.references_and_owned_nodes();

            for old_child in &substate_lock.substate_owned_nodes {
                if !new_children.remove(old_child) {
                    if SubstateProperties::is_persisted(&offset) {
                        return Err(RuntimeError::KernelError(KernelError::StoredNodeRemoved(
                            old_child.clone(),
                        )));
                    }
                }
            }

            for global_address in new_global_references {
                let node_id = RENodeId::Global(global_address);
                if !self.node_refs.contains_key(&node_id) {
                    return Err(RuntimeError::KernelError(
                        KernelError::InvalidReferenceWrite(global_address),
                    ));
                }
            }

            for child_id in &new_children {
                SubstateProperties::verify_can_own(&offset, *child_id)?;
                self.take_node_internal(*child_id)?;
            }

            match location {
                RENodeLocation::Heap => {}
                RENodeLocation::Store => {
                    heap.move_nodes_to_store(track, new_children)?;
                }
            }
        }

        // Global references need not be dropped
        // Substate Locks downstream may also continue to live
        for refed_node in substate_lock.substate_owned_nodes {
            self.node_refs.remove(&refed_node);
        }

        if let Some(counter) = self
            .owned_root_nodes
            .get_mut(&substate_lock.substate_pointer.1)
        {
            *counter -= 1;
        }

        let flags = substate_lock.flags;

        if !(matches!(offset, SubstateOffset::KeyValueStore(..))
            || matches!(
                offset,
                SubstateOffset::NonFungibleStore(NonFungibleStoreOffset::Entry(..))
            ))
        {
            match location {
                RENodeLocation::Store => track
                    .release_lock(
                        SubstateId(node_id, offset.clone()),
                        flags.contains(LockFlags::FORCE_WRITE),
                    )
                    .map_err(KernelError::TrackError),
                RENodeLocation::Heap => Ok(()),
            }?;
        }

        Ok(())
    }

    pub fn get_lock_info(&self, lock_handle: LockHandle) -> Result<LockInfo, RuntimeError> {
        let substate_lock = self
            .locks
            .get(&lock_handle)
            .ok_or(KernelError::LockDoesNotExist(lock_handle))?;

        Ok(LockInfo {
            offset: substate_lock.substate_pointer.2.clone(),
        })
    }

    fn get_lock(&self, lock_handle: LockHandle) -> Result<&SubstateLock, KernelError> {
        self.locks
            .get(&lock_handle)
            .ok_or(KernelError::LockDoesNotExist(lock_handle))
    }

    pub fn new_root() -> Self {
        Self {
            depth: 0,
            actor: ResolvedActor::function(FnIdentifier::Native(NativeFn::TransactionProcessor(
                TransactionProcessorFn::Run,
            ))),
            node_refs: HashMap::new(),
            owned_root_nodes: HashMap::new(),
            next_lock_handle: 0u32,
            locks: HashMap::new(),
        }
    }

    pub fn new_child_from_parent(
        parent: &mut CallFrame,
        actor: ResolvedActor,
        call_frame_update: CallFrameUpdate,
    ) -> Result<Self, RuntimeError> {
        let mut owned_heap_nodes = HashMap::new();
        let mut next_node_refs = HashMap::new();

        for node_id in call_frame_update.nodes_to_move {
            parent.take_node_internal(node_id)?;
            owned_heap_nodes.insert(node_id, 0u32);
        }

        for node_id in call_frame_update.node_refs_to_copy {
            let location = parent.get_node_location(node_id)?;
            let visibility = parent.get_node_visibility(node_id)?;
            next_node_refs.insert(node_id, RENodeRefData::new(location, visibility));
        }

        let frame = Self {
            depth: parent.depth + 1,
            actor,
            node_refs: next_node_refs,
            owned_root_nodes: owned_heap_nodes,
            next_lock_handle: 0u32,
            locks: HashMap::new(),
        };

        Ok(frame)
    }

    pub fn update_upstream(
        from: &mut CallFrame,
        to: &mut CallFrame,
        update: CallFrameUpdate,
    ) -> Result<(), RuntimeError> {
        for node_id in update.nodes_to_move {
            // move re nodes to upstream call frame.
            from.take_node_internal(node_id)?;
            to.owned_root_nodes.insert(node_id, 0u32);
        }

        for node_id in update.node_refs_to_copy {
            // Make sure not to allow owned nodes to be passed as references upstream
            let ref_data = from
                .node_refs
                .get(&node_id)
                .ok_or(CallFrameError::RENodeNotVisible(node_id))?;

            to.node_refs
                .entry(node_id)
                .and_modify(|e| {
                    if e.visibility == RENodeVisibilityOrigin::DirectAccess {
                        e.visibility = ref_data.visibility
                    }
                })
                .or_insert(ref_data.clone());
        }

        Ok(())
    }

    pub fn drop_all_locks<'s, R: FeeReserve>(
        &mut self,
        heap: &mut Heap,
        track: &mut Track<'s, R>,
    ) -> Result<(), RuntimeError> {
        let lock_handles: Vec<LockHandle> = self.locks.keys().cloned().collect();

        for lock_handle in lock_handles {
            self.drop_lock(heap, track, lock_handle)?;
        }

        Ok(())
    }

    fn take_node_internal(&mut self, node_id: RENodeId) -> Result<(), CallFrameError> {
        match self.owned_root_nodes.remove(&node_id) {
            None => Err(CallFrameError::RENodeNotOwned(node_id)),
            Some(lock_count) => {
                if lock_count == 0 {
                    Ok(())
                } else {
                    Err(CallFrameError::MovingLockedRENode(node_id))
                }
            }
        }
    }

    pub fn create_node<'f, 's, R: FeeReserve>(
        &mut self,
        node_id: RENodeId,
        re_node: RENodeInit,
        heap: &mut Heap,
        track: &'f mut Track<'s, R>,
        push_to_store: bool,
    ) -> Result<(), RuntimeError> {
        let substates = re_node.to_substates();

        for (offset, substate) in &substates {
            let substate_ref = substate.to_ref();
            let (_, owned) = substate_ref.references_and_owned_nodes();
            for child_id in owned {
                SubstateProperties::verify_can_own(&offset, child_id)?;
                self.take_node_internal(child_id)?;
                if push_to_store {
                    heap.move_node_to_store(track, child_id)?;
                }
            }
        }

        if push_to_store {
            for (offset, substate) in substates {
                track.insert_substate(SubstateId(node_id, offset), substate);
            }

            self.add_stored_ref(node_id, RENodeVisibilityOrigin::Normal);
        } else {
            // Insert node into heap
            let heap_root_node = HeapRENode {
                substates,
                //child_nodes,
            };
            heap.create_node(node_id, heap_root_node);
            self.owned_root_nodes.insert(node_id, 0u32);
        }

        Ok(())
    }

    pub fn add_stored_ref(&mut self, node_id: RENodeId, visibility: RENodeVisibilityOrigin) {
        self.node_refs.insert(
            node_id,
            RENodeRefData::new(RENodeLocation::Store, visibility),
        );
    }

    pub fn owned_nodes(&self) -> Vec<RENodeId> {
        self.owned_root_nodes.keys().cloned().collect()
    }

    /// Removes node from call frame and re-owns any children
    pub fn remove_node(
        &mut self,
        heap: &mut Heap,
        node_id: RENodeId,
    ) -> Result<HeapRENode, RuntimeError> {
        self.take_node_internal(node_id)?;
        let node = heap.remove_node(node_id)?;
        for (_, substate) in &node.substates {
            let (_, child_nodes) = substate.to_ref().references_and_owned_nodes();
            for child_node in child_nodes {
                self.owned_root_nodes.insert(child_node, 0u32);
            }
        }

        Ok(node)
    }

    fn get_substate<'f, 'p, 's, R: FeeReserve>(
        &self,
        heap: &'f mut Heap,
        track: &'f mut Track<'s, R>,
        location: RENodeLocation,
        node_id: RENodeId,
        offset: &SubstateOffset,
    ) -> Result<SubstateRef<'f>, RuntimeError> {
        let substate_ref = match location {
            RENodeLocation::Heap => heap.get_substate(node_id, offset)?,
            RENodeLocation::Store => track.get_substate(node_id, offset),
        };

        Ok(substate_ref)
    }

    pub fn get_ref<'f, 's, R: FeeReserve>(
        &mut self,
        lock_handle: LockHandle,
        heap: &'f mut Heap,
        track: &'f mut Track<'s, R>,
    ) -> Result<SubstateRef<'f>, RuntimeError> {
        let SubstateLock {
            substate_pointer: (node_location, node_id, offset),
            ..
        } = self
            .get_lock(lock_handle)
            .map_err(RuntimeError::KernelError)?
            .clone();

        self.get_substate(heap, track, node_location, node_id, &offset)
    }

    pub fn get_ref_mut<'f, 's, R: FeeReserve>(
        &'f mut self,
        lock_handle: LockHandle,
        heap: &'f mut Heap,
        track: &'f mut Track<'s, R>,
    ) -> Result<SubstateRefMut<'f>, RuntimeError> {
        let SubstateLock {
            substate_pointer: (node_location, node_id, offset),
            flags,
            ..
        } = self
            .get_lock(lock_handle)
            .map_err(RuntimeError::KernelError)?
            .clone();

        if !flags.contains(LockFlags::MUTABLE) {
            return Err(RuntimeError::KernelError(KernelError::LockNotMutable(
                lock_handle,
            )));
        }

        let ref_mut = match node_location {
            RENodeLocation::Heap => heap.get_substate_mut(node_id, &offset).unwrap(),
            RENodeLocation::Store => track.get_substate_mut(node_id, &offset),
        };

        Ok(ref_mut)
    }

    pub fn get_node_visibility(
        &self,
        node_id: RENodeId,
    ) -> Result<RENodeVisibilityOrigin, CallFrameError> {
        let visibility = if self.owned_root_nodes.contains_key(&node_id) {
            RENodeVisibilityOrigin::Normal
        } else if let Some(ref_data) = self.node_refs.get(&node_id) {
            ref_data.visibility
        } else {
            return Err(CallFrameError::RENodeNotVisible(node_id));
        };

        Ok(visibility)
    }

    pub fn get_node_location(&self, node_id: RENodeId) -> Result<RENodeLocation, CallFrameError> {
        // Find node
        let node_pointer = {
            if self.owned_root_nodes.contains_key(&node_id) {
                RENodeLocation::Heap
            } else if let Some(ref_data) = self.node_refs.get(&node_id) {
                ref_data.location.clone()
            } else {
                return Err(CallFrameError::RENodeNotVisible(node_id));
            }
        };

        Ok(node_pointer)
    }

    pub fn get_visible_nodes(&self) -> Vec<RENodeId> {
        let mut node_ids: Vec<RENodeId> = self.node_refs.keys().cloned().collect();
        let owned_ids: Vec<RENodeId> = self.owned_root_nodes.keys().cloned().collect();
        node_ids.extend(owned_ids);
        node_ids.sort(); // Required to make sure returned vector is deterministic
        node_ids
    }
}
