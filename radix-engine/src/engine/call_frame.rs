use crate::engine::*;
use crate::fee::FeeReserve;
use crate::model::{SubstateRef, SubstateRefMut};
use crate::types::*;

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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RENodeLocation {
    Heap,
    Store,
}

/// A lock on a substate controlled by a call frame
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubstateLock {
    pub substate_pointer: (RENodeLocation, RENodeId, SubstateOffset),
    pub global_references: HashSet<GlobalAddress>,
    pub substate_owned_nodes: HashSet<RENodeId>,
    pub flags: LockFlags,
}

// TODO: reduce fields visibility

/// A call frame is the basic unit that forms a transaction call stack, which keeps track of the
/// owned objects by this function.
pub struct CallFrame {
    /// The frame id
    pub depth: usize,

    /// The running application actor of this frame
    pub actor: REActor,

    /// All ref nodes accessible by this call frame (does not include owned nodes).
    pub node_refs: HashMap<RENodeId, RENodeLocation>,

    /// Owned nodes which by definition must live on heap
    owned_root_nodes: HashSet<RENodeId>,

    next_lock_handle: LockHandle,
    locks: HashMap<LockHandle, SubstateLock>,
    node_lock_count: HashMap<RENodeId, u32>,
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
            // TODO: Figure out how to drop these references as well on reference drop
            for global_address in &global_references {
                let node_id = RENodeId::Global(global_address.clone());
                self.node_refs.insert(node_id, RENodeLocation::Store);
            }
            for child_id in &substate_owned_nodes {
                self.node_refs.insert(*child_id, location);
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

        let counter = self.node_lock_count.entry(node_id).or_insert(0u32);
        *counter += 1;

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
                    return Err(RuntimeError::KernelError(KernelError::StoredNodeRemoved(
                        old_child.clone(),
                    )));
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

        for refed_node in substate_lock.substate_owned_nodes {
            self.node_refs.remove(&refed_node);
        }

        let counter = self
            .node_lock_count
            .entry(substate_lock.substate_pointer.1)
            .or_insert(0u32);
        *counter -= 1;
        if *counter == 0 {
            self.node_lock_count
                .remove(&substate_lock.substate_pointer.1);
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

    fn get_lock(&self, lock_handle: LockHandle) -> Result<&SubstateLock, KernelError> {
        self.locks
            .get(&lock_handle)
            .ok_or(KernelError::LockDoesNotExist(lock_handle))
    }

    pub fn new_root() -> Self {
        Self {
            depth: 0,
            actor: REActor::Function(ResolvedFunction::Native(
                NativeFunction::TransactionProcessor(TransactionProcessorFunction::Run),
            )),
            node_refs: HashMap::new(),
            owned_root_nodes: HashSet::new(),
            next_lock_handle: 0u32,
            locks: HashMap::new(),
            node_lock_count: HashMap::new(),
        }
    }

    pub fn new_child_from_parent(
        parent: &mut CallFrame,
        actor: REActor,
        call_frame_update: CallFrameUpdate,
    ) -> Result<Self, RuntimeError> {
        let mut owned_heap_nodes = HashSet::new();
        let mut next_node_refs = HashMap::new();

        for node_id in call_frame_update.nodes_to_move {
            parent.take_node_internal(node_id)?;
            owned_heap_nodes.insert(node_id);
        }

        for node_id in call_frame_update.node_refs_to_copy {
            let location = parent.get_node_location(node_id)?;
            next_node_refs.insert(node_id, location);
        }

        let frame = Self {
            depth: parent.depth + 1,
            actor,
            node_refs: next_node_refs,
            owned_root_nodes: owned_heap_nodes,
            next_lock_handle: 0u32,
            locks: HashMap::new(),
            node_lock_count: HashMap::new(),
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
            to.owned_root_nodes.insert(node_id);
        }

        for node_id in update.node_refs_to_copy {
            // Make sure not to allow owned nodes to be passed as references upstream
            let location = from
                .node_refs
                .get(&node_id)
                .ok_or(CallFrameError::RENodeNotVisible(node_id))?;
            to.node_refs.insert(node_id, location.clone());
        }

        Ok(())
    }

    pub fn drop_all_locks<'s, R: FeeReserve>(
        &mut self,
        heap: &mut Heap,
        track: &mut Track<'s, R>,
    ) -> Result<(), RuntimeError> {
        let lock_ids: Vec<LockHandle> = self.locks.keys().cloned().collect();
        for lock in lock_ids {
            self.drop_lock(heap, track, lock)?;
        }

        Ok(())
    }

    fn take_node_internal(&mut self, node_id: RENodeId) -> Result<(), CallFrameError> {
        if self.node_lock_count.contains_key(&node_id) {
            return Err(CallFrameError::MovingLockedRENode(node_id));
        }

        if !self.owned_root_nodes.remove(&node_id) {
            return Err(CallFrameError::RENodeNotOwned(node_id));
        }

        Ok(())
    }

    pub fn create_node(
        &mut self,
        heap: &mut Heap,
        node_id: RENodeId,
        re_node: RENode,
    ) -> Result<(), RuntimeError> {
        let substates = re_node.to_substates();

        for (offset, substate) in &substates {
            let substate_ref = substate.to_ref();
            let (_, owned) = substate_ref.references_and_owned_nodes();
            for child_id in owned {
                SubstateProperties::verify_can_own(&offset, child_id)?;
                self.take_node_internal(child_id)?;
            }
        }

        // Insert node into heap
        let heap_root_node = HeapRENode {
            substates,
            //child_nodes,
        };
        heap.create_node(node_id, heap_root_node);
        self.owned_root_nodes.insert(node_id);

        Ok(())
    }

    pub fn move_owned_node_to_store<'f, 's, R: FeeReserve>(
        &mut self,
        heap: &mut Heap,
        track: &'f mut Track<'s, R>,
        node_id: RENodeId,
    ) -> Result<(), RuntimeError> {
        self.take_node_internal(node_id)?;
        heap.move_node_to_store(track, node_id)?;

        Ok(())
    }

    pub fn owned_nodes(&self) -> Vec<RENodeId> {
        self.owned_root_nodes.iter().cloned().collect()
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
                self.owned_root_nodes.insert(child_node);
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

    pub fn get_node_location(&self, node_id: RENodeId) -> Result<RENodeLocation, CallFrameError> {
        // Find node
        let node_pointer = {
            if self.owned_root_nodes.contains(&node_id) {
                RENodeLocation::Heap
            } else if let Some(pointer) = self.node_refs.get(&node_id) {
                pointer.clone()
            } else {
                return Err(CallFrameError::RENodeNotVisible(node_id));
            }
        };

        Ok(node_pointer)
    }

    pub fn get_visible_nodes(&self) -> Vec<RENodeId> {
        let mut node_ids: Vec<RENodeId> = self.node_refs.keys().cloned().collect();
        let owned_ids: Vec<RENodeId> = self.owned_root_nodes.iter().cloned().collect();
        node_ids.extend(owned_ids);
        node_ids
    }
}
