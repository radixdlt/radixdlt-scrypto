use crate::engine::*;
use crate::fee::FeeReserve;
use crate::model::{SubstateRef, SubstateRefMut};
use crate::types::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RENodeLocation {
    Heap,
    Store,
}

/// A lock on a substate controlled by a call frame
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubstateLock {
    pub substate_pointer: (RENodeLocation, RENodeId, SubstateOffset),
    pub owned_nodes: HashSet<RENodeId>,
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

        let lock_handle = self.next_lock_handle;
        self.locks.insert(
            lock_handle,
            SubstateLock {
                substate_pointer: (location, node_id, offset),
                owned_nodes: HashSet::new(),
                flags,
            },
        );
        self.next_lock_handle = self.next_lock_handle + 1;

        let counter = self.node_lock_count.entry(node_id).or_insert(0u32);
        *counter += 1;

        Ok(lock_handle)
    }

    fn release_lock<R: FeeReserve>(
        track: &mut Track<R>,
        pointer: RENodeLocation,
        node_id: RENodeId,
        offset: SubstateOffset,
        force_write: bool,
    ) -> Result<(), KernelError> {
        match pointer {
            RENodeLocation::Store => track
                .release_lock(SubstateId(node_id, offset), force_write)
                .map_err(KernelError::TrackError),
            RENodeLocation::Heap => Ok(()),
        }
    }

    pub fn drop_lock<'s, R: FeeReserve>(
        &mut self,
        track: &mut Track<'s, R>,
        lock_handle: LockHandle,
    ) -> Result<(), KernelError> {
        let substate_lock = self
            .locks
            .remove(&lock_handle)
            .ok_or(KernelError::LockDoesNotExist(lock_handle))?;

        for refed_node in substate_lock.owned_nodes {
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

        let node_pointer = substate_lock.substate_pointer.0;
        let node_id = substate_lock.substate_pointer.1;
        let offset = substate_lock.substate_pointer.2;
        let flags = substate_lock.flags;

        if !(matches!(offset, SubstateOffset::KeyValueStore(..))
            || matches!(
                offset,
                SubstateOffset::NonFungibleStore(NonFungibleStoreOffset::Entry(..))
            ))
        {
            Self::release_lock(
                track,
                node_pointer,
                node_id,
                offset.clone(),
                flags.contains(LockFlags::FORCE_WRITE),
            )?;
        }

        Ok(())
    }

    pub fn get_lock(&self, lock_handle: LockHandle) -> Result<&SubstateLock, KernelError> {
        self.locks
            .get(&lock_handle)
            .ok_or(KernelError::LockDoesNotExist(lock_handle))
    }

    // TODO: Figure out right interface for this
    pub fn add_lock_visible_node(
        &mut self,
        lock_handle: LockHandle,
        node_id: RENodeId,
    ) -> Result<(), KernelError> {
        let lock = self
            .locks
            .get_mut(&lock_handle)
            .ok_or(KernelError::LockDoesNotExist(lock_handle))?;
        lock.owned_nodes.insert(node_id);
        Ok(())
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
        nodes_to_move: Vec<RENodeId>,
        node_refs: HashMap<RENodeId, RENodeLocation>,
    ) -> Result<Self, RuntimeError> {
        let mut owned_heap_nodes = HashSet::new();

        for node_id in nodes_to_move {
            parent.take_node_internal(node_id)?;
            owned_heap_nodes.insert(node_id);
        }

        let frame = Self {
            depth: parent.depth + 1,
            actor,
            node_refs,
            owned_root_nodes: owned_heap_nodes,
            next_lock_handle: 0u32,
            locks: HashMap::new(),
            node_lock_count: HashMap::new(),
        };

        Ok(frame)
    }

    pub fn move_nodes_upstream(
        from: &mut CallFrame,
        to: &mut CallFrame,
        node_ids: HashSet<RENodeId>,
    ) -> Result<(), RuntimeError> {
        for node_id in node_ids {
            // move re nodes to upstream call frame.
            from.take_node_internal(node_id)?;
            to.owned_root_nodes.insert(node_id);
        }

        Ok(())
    }

    pub fn copy_refs(
        from: &mut CallFrame,
        to: &mut CallFrame,
        global_addresses: HashSet<GlobalAddress>,
    ) -> Result<(), RuntimeError> {
        for global_address in global_addresses {
            let node_id = RENodeId::Global(global_address);
            if !from.node_refs.contains_key(&node_id) {
                return Err(RuntimeError::KernelError(
                    KernelError::InvalidReferenceReturn(global_address),
                ));
            }
            to.node_refs.insert(node_id, RENodeLocation::Store);
        }

        Ok(())
    }

    pub fn drop_all_locks<'s, R: FeeReserve>(
        &mut self,
        track: &mut Track<'s, R>,
    ) -> Result<(), RuntimeError> {
        let lock_ids: Vec<LockHandle> = self.locks.keys().cloned().collect();
        for lock in lock_ids {
            self.drop_lock(track, lock)?;
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

    pub fn move_owned_nodes_to_heap_node(
        &mut self,
        children: HashSet<RENodeId>,
    ) -> Result<(), RuntimeError> {
        for child_id in &children {
            self.take_node_internal(*child_id)?;
        }

        Ok(())
    }

    pub fn move_owned_nodes_to_store<'f, 's, R: FeeReserve>(
        &mut self,
        heap: &mut Heap,
        track: &'f mut Track<'s, R>,
        node_ids: HashSet<RENodeId>,
    ) -> Result<(), RuntimeError> {
        for node_id in &node_ids {
            self.take_node_internal(*node_id)?;
        }

        heap.move_nodes_to_store(track, node_ids)?;

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

    pub fn remove_node(
        &mut self,
        heap: &mut Heap,
        node_id: RENodeId,
    ) -> Result<HeapRENode, RuntimeError> {
        self.take_node_internal(node_id)?;
        let node = heap.remove_node(node_id)?;
        // TODO: Remove this
        /*
        if !node.child_nodes.is_empty() {
            return Err(RuntimeError::KernelError(KernelError::DropNodeFailure(
                node_id,
            )));
        }
         */
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

        let substate_ref = self.get_substate(heap, track, node_location, node_id, &offset)?;
        let (global_references, children) = substate_ref.references_and_owned_nodes();

        // Expand references
        {
            // TODO: Figure out how to drop these references as well on reference drop
            for global_address in global_references {
                let node_id = RENodeId::Global(global_address);
                self.node_refs.insert(node_id, RENodeLocation::Store);
            }
            for child_id in children {
                self.node_refs.insert(child_id, node_location);
                self.add_lock_visible_node(lock_handle, child_id)
                    .map_err(RuntimeError::KernelError)?;
            }
        }

        Ok(substate_ref)
    }

    pub fn get_ref_mut<'f, 's, R: FeeReserve>(
        &'f mut self,
        lock_handle: LockHandle,
        heap: &'f mut Heap,
        track: &'f mut Track<'s, R>,
    ) -> Result<SubstateRefMut<'f, 's, R>, RuntimeError> {
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

        let (global_references, children) = {
            let substate_ref = self.get_substate(heap, track, node_location, node_id, &offset)?;
            substate_ref.references_and_owned_nodes()
        };

        // Expand references
        {
            // TODO: Figure out how to drop these references as well on reference drop
            for global_address in global_references {
                let node_id = RENodeId::Global(global_address);
                self.node_refs.insert(node_id, RENodeLocation::Store);
            }
            for child_id in &children {
                self.node_refs.insert(*child_id, node_location);
                self.add_lock_visible_node(lock_handle, *child_id)
                    .map_err(RuntimeError::KernelError)?;
            }
        }

        SubstateRefMut::new(
            lock_handle,
            node_location,
            node_id,
            offset,
            children,
            self,
            heap,
            track,
        )
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
