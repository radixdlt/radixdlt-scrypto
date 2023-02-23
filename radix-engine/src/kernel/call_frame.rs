use crate::errors::{CallFrameError, KernelError, RuntimeError};
use crate::kernel::actor::ResolvedActor;
use crate::kernel::kernel_api::LockFlags;
use crate::system::node::{RENodeInit, RENodeModuleInit};
use crate::system::node_properties::SubstateProperties;
use crate::system::node_substates::{SubstateRef, SubstateRefMut};
use crate::types::*;
use radix_engine_interface::api::types::{
    Address, LockHandle, NonFungibleStoreOffset, RENodeId, SubstateId, SubstateOffset,
};

use super::heap::{Heap, HeapRENode};
use super::kernel_api::LockInfo;
use super::track::{Track, TrackError};

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
    pub node_id: RENodeId,
    pub module_id: NodeModuleId,
    pub offset: SubstateOffset,
    pub references: HashSet<RENodeId>,
    pub substate_owned_nodes: Vec<RENodeId>,
    pub flags: LockFlags,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RENodeRefData {
    visibility: RENodeVisibilityOrigin,
}

impl RENodeRefData {
    fn new(visibility: RENodeVisibilityOrigin) -> Self {
        RENodeRefData { visibility }
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
    pub actor: Option<ResolvedActor>,

    /// All ref nodes accessible by this call frame (does not include owned nodes).
    node_refs: HashMap<RENodeId, RENodeRefData>,

    /// Owned nodes which by definition must live on heap
    /// Also keeps track of number of locks on this node
    owned_root_nodes: HashMap<RENodeId, u32>,

    next_lock_handle: LockHandle,
    locks: HashMap<LockHandle, SubstateLock>,
}

impl CallFrame {
    pub fn acquire_lock<'s>(
        &mut self,
        heap: &mut Heap,
        track: &mut Track<'s>,
        node_id: RENodeId,
        module_id: NodeModuleId,
        offset: SubstateOffset,
        flags: LockFlags,
    ) -> Result<LockHandle, RuntimeError> {
        self.check_node_visibility(&node_id)?;
        if !(matches!(offset, SubstateOffset::KeyValueStore(..))
            || matches!(
                offset,
                SubstateOffset::NonFungibleStore(NonFungibleStoreOffset::Entry(..))
            ))
        {
            let substate_id = SubstateId(node_id, module_id, offset.clone());
            if heap.contains_node(&node_id) {
                if flags.contains(LockFlags::UNMODIFIED_BASE) {
                    return Err(RuntimeError::KernelError(KernelError::TrackError(
                        TrackError::LockUnmodifiedBaseOnNewSubstate(substate_id),
                    )));
                }
            } else {
                track
                    .acquire_lock(substate_id, flags)
                    .map_err(KernelError::TrackError)?;
            }
        }

        let substate_ref = self.get_substate(heap, track, node_id, module_id, &offset)?;
        let (references, substate_owned_nodes) = substate_ref.references_and_owned_nodes();

        // Expand references
        {
            for node_id in &references {
                self.node_refs.insert(
                    node_id.clone(),
                    RENodeRefData::new(RENodeVisibilityOrigin::Normal),
                );
            }
            for child_id in &substate_owned_nodes {
                self.node_refs.insert(
                    *child_id,
                    RENodeRefData::new(RENodeVisibilityOrigin::Normal),
                );
            }
        }

        let lock_handle = self.next_lock_handle;
        self.locks.insert(
            lock_handle,
            SubstateLock {
                node_id,
                module_id,
                offset,
                references,
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

    pub fn drop_lock<'s>(
        &mut self,
        heap: &mut Heap,
        track: &mut Track<'s>,
        lock_handle: LockHandle,
    ) -> Result<(), RuntimeError> {
        let substate_lock = self
            .locks
            .remove(&lock_handle)
            .ok_or(KernelError::LockDoesNotExist(lock_handle))?;

        let node_id = substate_lock.node_id;
        let module_id = substate_lock.module_id;
        let offset = substate_lock.offset;

        if substate_lock.flags.contains(LockFlags::MUTABLE) {
            let substate_ref = self.get_substate(heap, track, node_id, module_id, &offset)?;

            // Reserving original Vec element order with `IndexSet`
            let mut new_children: IndexSet<RENodeId> = index_set_new();
            for own in substate_ref.references_and_owned_nodes().1 {
                if !new_children.insert(own) {
                    return Err(RuntimeError::KernelError(
                        KernelError::ContainsDuplicatedOwns,
                    ));
                }
            }

            for old_child in &substate_lock.substate_owned_nodes {
                if !new_children.remove(old_child) {
                    if SubstateProperties::is_persisted(&offset) {
                        return Err(RuntimeError::KernelError(KernelError::StoredNodeRemoved(
                            old_child.clone(),
                        )));
                    }
                }
            }

            for child_id in &new_children {
                SubstateProperties::verify_can_own(&offset, *child_id)?;
                self.take_node_internal(*child_id)?;
            }

            if !heap.contains_node(&node_id) {
                heap.move_nodes_to_store(track, new_children.into_iter().collect())?;
            }
        }

        // TODO: revisit this
        // References need not be dropped
        // Substate Locks downstream may also continue to live
        for refed_node in substate_lock.substate_owned_nodes {
            self.node_refs.remove(&refed_node);
        }

        if let Some(counter) = self.owned_root_nodes.get_mut(&substate_lock.node_id) {
            *counter -= 1;
        }

        let flags = substate_lock.flags;

        if !(matches!(offset, SubstateOffset::KeyValueStore(..))
            || matches!(
                offset,
                SubstateOffset::NonFungibleStore(NonFungibleStoreOffset::Entry(..))
            ))
        {
            if !heap.contains_node(&node_id) {
                track
                    .release_lock(
                        SubstateId(node_id, module_id, offset.clone()),
                        flags.contains(LockFlags::FORCE_WRITE),
                    )
                    .map_err(KernelError::TrackError)?;
            }
        }

        Ok(())
    }

    pub fn get_lock_info(&self, lock_handle: LockHandle) -> Result<LockInfo, RuntimeError> {
        let substate_lock = self
            .locks
            .get(&lock_handle)
            .ok_or(KernelError::LockDoesNotExist(lock_handle))?;

        Ok(LockInfo {
            offset: substate_lock.offset.clone(),
        })
    }

    fn get_lock(&self, lock_handle: LockHandle) -> Result<&SubstateLock, KernelError> {
        self.locks
            .get(&lock_handle)
            .ok_or(KernelError::LockDoesNotExist(lock_handle))
    }

    pub fn new_root() -> Self {
        let mut frame = Self {
            depth: 0,
            actor: None,
            node_refs: HashMap::new(),
            owned_root_nodes: HashMap::new(),
            next_lock_handle: 0u32,
            locks: HashMap::new(),
        };

        // Add well-known global refs to current frame
        frame.add_ref(
            RENodeId::Global(Address::Resource(RADIX_TOKEN)),
            RENodeVisibilityOrigin::Normal,
        );
        frame.add_ref(
            RENodeId::Global(Address::Resource(SYSTEM_TOKEN)),
            RENodeVisibilityOrigin::Normal,
        );
        frame.add_ref(
            RENodeId::Global(Address::Resource(ECDSA_SECP256K1_TOKEN)),
            RENodeVisibilityOrigin::Normal,
        );
        frame.add_ref(
            RENodeId::Global(Address::Resource(EDDSA_ED25519_TOKEN)),
            RENodeVisibilityOrigin::Normal,
        );
        frame.add_ref(
            RENodeId::Global(Address::Resource(PACKAGE_TOKEN)),
            RENodeVisibilityOrigin::Normal,
        );
        frame.add_ref(
            RENodeId::Global(Address::Component(EPOCH_MANAGER)),
            RENodeVisibilityOrigin::Normal,
        );
        frame.add_ref(
            RENodeId::Global(Address::Component(CLOCK)),
            RENodeVisibilityOrigin::Normal,
        );
        frame.add_ref(
            RENodeId::Global(Address::Package(FAUCET_PACKAGE)),
            RENodeVisibilityOrigin::Normal,
        );

        frame
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
            let visibility = parent.check_node_visibility(&node_id)?;
            next_node_refs.insert(node_id, RENodeRefData::new(visibility));
        }

        let frame = Self {
            depth: parent.depth + 1,
            actor: Some(actor),
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

    pub fn drop_all_locks<'s>(
        &mut self,
        heap: &mut Heap,
        track: &mut Track<'s>,
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

    pub fn create_node<'f, 's>(
        &mut self,
        node_id: RENodeId,
        re_node: RENodeInit,
        node_modules: BTreeMap<NodeModuleId, RENodeModuleInit>,
        heap: &mut Heap,
        track: &'f mut Track<'s>,
        push_to_store: bool,
    ) -> Result<(), RuntimeError> {
        let mut substates = BTreeMap::new();
        let self_substates = re_node.to_substates();
        for (offset, substate) in self_substates {
            substates.insert((NodeModuleId::SELF, offset), substate);
        }
        for (node_module_id, module_init) in node_modules {
            for (offset, substate) in module_init.to_substates() {
                substates.insert((node_module_id, offset), substate);
            }
        }

        for ((_module_id, offset), substate) in &substates {
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
            for ((module_id, offset), substate) in substates {
                track.insert_substate(SubstateId(node_id, module_id, offset), substate);
            }

            self.add_ref(node_id, RENodeVisibilityOrigin::Normal);
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

    pub fn add_ref(&mut self, node_id: RENodeId, visibility: RENodeVisibilityOrigin) {
        self.node_refs
            .insert(node_id, RENodeRefData::new(visibility));
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

    fn get_substate<'f, 'p, 's>(
        &self,
        heap: &'f mut Heap,
        track: &'f mut Track<'s>,
        node_id: RENodeId,
        module_id: NodeModuleId,
        offset: &SubstateOffset,
    ) -> Result<SubstateRef<'f>, RuntimeError> {
        let substate_ref = if heap.contains_node(&node_id) {
            heap.get_substate(node_id, module_id, offset)?
        } else {
            track.get_substate(node_id, module_id, offset)
        };

        Ok(substate_ref)
    }

    pub fn get_ref<'f, 's>(
        &mut self,
        lock_handle: LockHandle,
        heap: &'f mut Heap,
        track: &'f mut Track<'s>,
    ) -> Result<SubstateRef<'f>, RuntimeError> {
        let SubstateLock {
            node_id,
            module_id,
            offset,
            ..
        } = self
            .get_lock(lock_handle)
            .map_err(RuntimeError::KernelError)?
            .clone();

        self.get_substate(heap, track, node_id, module_id, &offset)
    }

    pub fn get_ref_mut<'f, 's>(
        &'f mut self,
        lock_handle: LockHandle,
        heap: &'f mut Heap,
        track: &'f mut Track<'s>,
    ) -> Result<SubstateRefMut<'f>, RuntimeError> {
        let SubstateLock {
            node_id,
            module_id,
            offset,
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

        let ref_mut = if heap.contains_node(&node_id) {
            heap.get_substate_mut(node_id, module_id, &offset).unwrap()
        } else {
            track.get_substate_mut(node_id, module_id, &offset)
        };

        Ok(ref_mut)
    }

    pub fn get_node_visibility(&self, node_id: &RENodeId) -> Option<RENodeVisibilityOrigin> {
        if self.owned_root_nodes.contains_key(node_id) {
            Some(RENodeVisibilityOrigin::Normal)
        } else if let Some(ref_data) = self.node_refs.get(node_id) {
            Some(ref_data.visibility)
        } else {
            None
        }
    }

    pub fn check_node_visibility(
        &self,
        node_id: &RENodeId,
    ) -> Result<RENodeVisibilityOrigin, CallFrameError> {
        self.get_node_visibility(node_id)
            .ok_or_else(|| CallFrameError::RENodeNotVisible(node_id.clone()))
    }
}
