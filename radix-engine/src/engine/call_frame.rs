use crate::engine::*;
use crate::types::*;
use scrypto::core::NativeFunction;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubstateLock {
    pub pointer: (RENodePointer, SubstateOffset),
    pub mutable: bool,
    pub refed_nodes: HashSet<RENodeId>,
}

// TODO: reduce fields visibility

/// A call frame is the basic unit that forms a transaction call stack, which keeps track of the
/// owned objects by this function.
pub struct CallFrame {
    /// The frame id
    pub depth: usize,
    /// The running actor of this frame
    pub actor: REActor,

    /// All ref values accessible by this call frame. The value may be located in one of the following:
    /// 1. borrowed values
    /// 2. track
    pub node_refs: HashMap<RENodeId, RENodePointer>,

    /// Owned Values
    pub owned_heap_nodes: HashMap<RENodeId, HeapRootRENode>,

    cur_lock_handle: LockHandle,
    locks: HashMap<LockHandle, SubstateLock>,
    node_lock_index: HashMap<RENodeId, u32>,
}

impl CallFrame {
    pub fn create_lock(
        &mut self,
        node_pointer: RENodePointer,
        offset: SubstateOffset,
        mutable: bool,
    ) -> LockHandle {
        self.cur_lock_handle = self.cur_lock_handle + 1;
        self.locks.insert(
            self.cur_lock_handle,
            SubstateLock {
                pointer: (node_pointer, offset),
                mutable,
                refed_nodes: HashSet::new(),
            },
        );

        let counter = self
            .node_lock_index
            .entry(node_pointer.node_id())
            .or_insert(0u32);
        *counter += 1;

        self.cur_lock_handle
    }

    pub fn drop_lock(
        &mut self,
        lock_handle: LockHandle,
    ) -> Result<(RENodePointer, SubstateOffset), KernelError> {
        let substate_lock = self
            .locks
            .remove(&lock_handle)
            .ok_or(KernelError::LockDoesNotExist(lock_handle))?;

        for refed_node in substate_lock.refed_nodes {
            self.node_refs.remove(&refed_node);
        }

        let counter = self
            .node_lock_index
            .entry(substate_lock.pointer.0.node_id())
            .or_insert(0u32);
        *counter -= 1;
        if *counter == 0 {
            self.node_lock_index
                .remove(&substate_lock.pointer.0.node_id());
        }

        Ok(substate_lock.pointer)
    }

    pub fn get_lock(&self, lock_handle: LockHandle) -> Result<&SubstateLock, KernelError> {
        self.locks
            .get(&lock_handle)
            .ok_or(KernelError::LockDoesNotExist(lock_handle))
    }

    pub fn get_lock_mut(
        &mut self,
        lock_handle: LockHandle,
    ) -> Result<&mut SubstateLock, KernelError> {
        self.locks
            .get_mut(&lock_handle)
            .ok_or(KernelError::LockDoesNotExist(lock_handle))
    }

    pub fn new_root() -> Self {
        Self {
            depth: 0,
            actor: REActor::Function(FunctionIdent::Native(NativeFunction::TransactionProcessor(
                TransactionProcessorFunction::Run,
            ))),
            node_refs: HashMap::new(),
            owned_heap_nodes: HashMap::new(),
            cur_lock_handle: 0u32,
            locks: HashMap::new(),
            node_lock_index: HashMap::new(),
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
            cur_lock_handle: 0u32,
            locks: HashMap::new(),
            node_lock_index: HashMap::new(),
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

    pub fn take_available_values(
        &mut self,
        node_ids: HashSet<RENodeId>,
        persist_only: bool,
    ) -> Result<(HashMap<RENodeId, HeapRootRENode>, HashSet<RENodeId>), RuntimeError> {
        let (taken, missing) = {
            let mut taken_values = HashMap::new();
            let mut missing_values = HashSet::new();

            for id in node_ids {
                if self.node_lock_index.contains_key(&id) {
                    return Err(RuntimeError::KernelError(KernelError::MovingLockedRENode(
                        id,
                    )));
                }

                let maybe = self.owned_heap_nodes.remove(&id);
                if let Some(value) = maybe {
                    value.root().verify_can_move()?;
                    if persist_only {
                        value.root().verify_can_persist()?;
                    }
                    taken_values.insert(id, value);
                } else {
                    missing_values.insert(id);
                }
            }

            (taken_values, missing_values)
        };

        // Moved values must have their references removed
        for (id, value) in &taken {
            self.node_refs.remove(id);
            for (id, ..) in &value.child_nodes {
                self.node_refs.remove(id);
            }
        }

        Ok((taken, missing))
    }

    pub fn take_node(&mut self, node_id: RENodeId) -> Result<HeapRootRENode, RuntimeError> {
        if self.node_lock_index.contains_key(&node_id) {
            return Err(RuntimeError::KernelError(KernelError::MovingLockedRENode(
                node_id,
            )));
        }

        let maybe = self.owned_heap_nodes.remove(&node_id);
        if let Some(root_node) = maybe {
            root_node.root().verify_can_move()?;
            Ok(root_node)
        } else {
            Err(RuntimeError::KernelError(KernelError::RENodeNotFound(
                node_id,
            )))
        }
    }

    pub fn get_node_pointer(&self, node_id: RENodeId) -> Result<RENodePointer, RuntimeError> {
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
                return Err(RuntimeError::KernelError(KernelError::RENodeNotVisible(
                    node_id,
                )));
            }
        };

        Ok(node_pointer)
    }
}
