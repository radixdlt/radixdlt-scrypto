use crate::engine::*;
use crate::fee::FeeReserve;
use crate::types::*;
use crate::wasm::*;
use scrypto::core::NativeFunction;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubstateLock {
    pub pointer: (RENodePointer, SubstateOffset),
    pub mutable: bool,
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

    pub locks: Vec<SubstateLock>,
}

impl CallFrame {
    pub fn add_substate_lock(&mut self, node_pointer: RENodePointer, offset: SubstateOffset, mutable: bool) {
        self.locks.push(SubstateLock { pointer: (node_pointer, offset), mutable, })
    }

    pub fn release_substate_lock(&mut self, node_pointer: RENodePointer, offset: SubstateOffset) {
        let p = (node_pointer, offset);
        let index = self.locks.iter().position(|s| s.pointer.eq(&p)).unwrap();
        self.locks.remove(index);
    }

    pub fn new_root() -> Self {
        Self {
            depth: 0,
            actor: REActor::Function(FunctionIdent::Native(NativeFunction::TransactionProcessor(
                TransactionProcessorFunction::Run,
            ))),
            node_refs: HashMap::new(),
            owned_heap_nodes: HashMap::new(),
            locks: Vec::new(),
        }
    }

    pub fn new_child<'s, Y, W, I, R>(
        depth: usize,
        actor: REActor,
        owned_heap_nodes: HashMap<RENodeId, HeapRootRENode>,
        node_refs: HashMap<RENodeId, RENodePointer>,
        _system_api: &mut Y,
    ) -> Self
    where
        Y: SystemApi<'s, W, I, R>,
        W: WasmEngine<I>,
        I: WasmInstance,
        R: FeeReserve,
    {
        Self {
            depth,
            actor,
            node_refs,
            owned_heap_nodes,
            locks: Vec::new(),
        }
    }

    pub fn drop_owned_values(&mut self) -> Result<(), RuntimeError> {
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
