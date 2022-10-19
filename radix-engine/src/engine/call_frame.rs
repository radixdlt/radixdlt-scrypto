use crate::engine::*;
use crate::types::*;

/// A lock on a substate controlled by a call frame
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubstateLock {
    pub substate_pointer: (RENodePointer, SubstateOffset),
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

    /// All ref values accessible by this call frame. The value may be located in one of the following:
    /// 1. borrowed values
    /// 2. track
    pub node_refs: HashMap<RENodeId, RENodePointer>,

    /// Owned Values
    pub owned_heap_nodes: HashMap<RENodeId, HeapRootRENode>,

    next_lock_handle: LockHandle,
    locks: HashMap<LockHandle, SubstateLock>,
    node_lock_count: HashMap<RENodeId, u32>,
}

impl CallFrame {
    pub fn create_lock(
        &mut self,
        node_pointer: RENodePointer,
        offset: SubstateOffset,
        flags: LockFlags,
    ) -> LockHandle {
        let lock_handle = self.next_lock_handle;
        self.locks.insert(
            lock_handle,
            SubstateLock {
                substate_pointer: (node_pointer, offset),
                owned_nodes: HashSet::new(),
                flags,
            },
        );
        self.next_lock_handle = self.next_lock_handle + 1;

        let counter = self
            .node_lock_count
            .entry(node_pointer.node_id())
            .or_insert(0u32);
        *counter += 1;

        lock_handle
    }

    pub fn drop_lock(
        &mut self,
        lock_handle: LockHandle,
    ) -> Result<(RENodePointer, SubstateOffset, LockFlags), KernelError> {
        let substate_lock = self
            .locks
            .remove(&lock_handle)
            .ok_or(KernelError::LockDoesNotExist(lock_handle))?;

        for refed_node in substate_lock.owned_nodes {
            self.node_refs.remove(&refed_node);
        }

        let counter = self
            .node_lock_count
            .entry(substate_lock.substate_pointer.0.node_id())
            .or_insert(0u32);
        *counter -= 1;
        if *counter == 0 {
            self.node_lock_count
                .remove(&substate_lock.substate_pointer.0.node_id());
        }

        Ok((
            substate_lock.substate_pointer.0,
            substate_lock.substate_pointer.1,
            substate_lock.flags,
        ))
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
            owned_heap_nodes: HashMap::new(),
            next_lock_handle: 0u32,
            locks: HashMap::new(),
            node_lock_count: HashMap::new(),
        }
    }

    pub fn new_child(
        depth: usize,
        actor: REActor,
        owned_heap_nodes: HashMap<RENodeId, HeapRootRENode>,
        node_refs: HashMap<RENodeId, RENodePointer>,
    ) -> Self {
        Self {
            depth,
            actor,
            node_refs,
            owned_heap_nodes,
            next_lock_handle: 0u32,
            locks: HashMap::new(),
            node_lock_count: HashMap::new(),
        }
    }

    pub fn drain_locks(&mut self) -> HashMap<LockHandle, SubstateLock> {
        self.locks.drain().collect()
    }

    pub fn drop_frame(mut self) -> Result<(), RuntimeError> {
        let values = self
            .owned_heap_nodes
            .drain()
            .map(|(_id, value)| value)
            .collect();
        HeapRENode::drop_nodes(values)
            .map_err(|e| RuntimeError::KernelError(KernelError::DropFailure(e)))
    }

    pub fn insert_owned_node(&mut self, id: RENodeId, node: HeapRootRENode) {
        self.owned_heap_nodes.insert(id, node);
    }

    pub fn take_node(&mut self, node_id: RENodeId) -> Result<HeapRootRENode, CallFrameError> {
        if self.node_lock_count.contains_key(&node_id) {
            return Err(CallFrameError::MovingLockedRENode(node_id));
        }

        let maybe = self.owned_heap_nodes.remove(&node_id);
        if let Some(root_node) = maybe {
            // Moved nodes must have their child node references removed
            self.node_refs.remove(&node_id);
            for (id, ..) in &root_node.child_nodes {
                self.node_refs.remove(id);
            }

            Ok(root_node)
        } else {
            Err(CallFrameError::RENodeNotOwned(node_id))
        }
    }

    pub fn get_owned_heap_node_mut(
        &mut self,
        node_id: RENodeId,
    ) -> Result<&mut HeapRootRENode, CallFrameError> {
        self.owned_heap_nodes
            .get_mut(&node_id)
            .ok_or(CallFrameError::RENodeNotOwned(node_id))
    }

    pub fn get_owned_heap_node(
        &self,
        node_id: RENodeId,
    ) -> Result<&HeapRootRENode, CallFrameError> {
        self.owned_heap_nodes
            .get(&node_id)
            .ok_or(CallFrameError::RENodeNotOwned(node_id))
    }

    pub fn get_node_pointer(&self, node_id: RENodeId) -> Result<RENodePointer, CallFrameError> {
        // Find node
        let node_pointer = {
            if self.owned_heap_nodes.contains_key(&node_id) {
                RENodePointer::Heap {
                    frame_id: self.depth,
                    root: node_id.clone(),
                    id: None,
                }
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
        let owned_ids: Vec<RENodeId> = self.owned_heap_nodes.keys().cloned().collect();
        node_ids.extend(owned_ids);
        node_ids
    }
}
