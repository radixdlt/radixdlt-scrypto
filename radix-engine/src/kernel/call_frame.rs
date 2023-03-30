use crate::errors::{CallFrameError, KernelError, RuntimeError};
use crate::kernel::actor::Actor;
use crate::system::node::{ModuleInit, NodeInit};
use crate::system::node_modules::type_info::TypeInfoSubstate;
use crate::system::node_properties::SubstateProperties;
use crate::system::node_substates::{SubstateRef, SubstateRefMut};
use crate::types::*;
use radix_engine_interface::api::substate_api::LockFlags;
use radix_engine_interface::types::{LockHandle, NodeId, SubstateKey};
use radix_engine_stores::interface::{AcquireLockError, SubstateStore};

use super::heap::{Heap, HeapNode};
use super::kernel_api::LockInfo;
use super::track::Track;

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
pub enum RENodeVisibilityOrigin {
    Normal,
    DirectAccess,
}

/// A lock on a substate controlled by a call frame
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubstateLock {
    pub node_id: NodeId,
    pub module_id: TypedModuleId,
    pub substate_key: SubstateKey,
    pub temp_references: IndexSet<NodeId>,
    pub substate_owned_nodes: Vec<NodeId>,
    pub flags: LockFlags,
    pub track_handle: Option<u32>,
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
    pub actor: Option<Actor>,

    /// Node refs which are immortal during the life time of this frame:
    /// - Any node refs received from other frames;
    /// - Global node refs obtained through substate locking.
    immortal_node_refs: NonIterMap<NodeId, RENodeRefData>,

    /// Node refs obtained through substate locking, which will be dropped upon unlocking.
    temp_node_refs: NonIterMap<NodeId, u32>,

    /// Owned nodes which by definition must live on heap
    /// Also keeps track of number of locks on this node
    owned_root_nodes: IndexMap<NodeId, u32>,

    next_lock_handle: LockHandle,
    locks: IndexMap<LockHandle, SubstateLock>,
}

pub enum CallFrameAcquireLockError {
    NodeNotInCallFrame(NodeId),
    LockUnmodifiedBaseOnHeapNode,
    SubstateNotFound,
    TrackAcquireLockError(AcquireLockError),
}

pub enum CallFrameDropLockError {
    LockNotFound(LockHandle),
}

pub enum FrameUpdateError {
    OwnNotFound(NodeId),
    RefNotFound(NodeId),
    NodeLocked(NodeId),
}

impl CallFrame {
    pub fn acquire_lock<'s>(
        &mut self,
        heap: &mut Heap,
        track: &mut Track<'s>,
        node_id: &NodeId,
        module_id: TypedModuleId,
        substate_key: &SubstateKey,
        flags: LockFlags,
    ) -> Result<LockHandle, RuntimeError> {
        self.check_node_visibility(&node_id)?;
        if !(matches!(offset, SubstateKey::KeyValueStore(..))) {
            let substate_id = SubstateId(node_id.clone(), module_id, offset.clone());
            if heap.contains_node(&node_id) {
                if flags.contains(LockFlags::UNMODIFIED_BASE) {
                    return Err(RuntimeError::KernelError(KernelError::TrackError(
                        Box::new(TrackError::LockUnmodifiedBaseOnNewSubstate(substate_id)),
                    )));
                }
            } else {
                track
                    .acquire_lock(substate_id, flags)
                    .map_err(|e| KernelError::TrackError(Box::new(e)))?;
            }
            heap.get_substate(node_id, module_id, substate_key)
                .ok_or(CallFrameAcquireLockError::SubstateNotFound)?
        } else {
            let handle = track
                .acquire_lock(node_id, module_id.into(), substate_key, flags)
                .map_err(CallFrameAcquireLockError::TrackAcquireLockError)?;
            track_handle = Some(handle);
            track.get_substate(handle)
        };

        let substate_ref = self.get_substate(heap, track, node_id, module_id, &offset)?;
        let (references, substate_owned_nodes) = substate_ref.references_and_owned_nodes();

        // Expand references
        let mut temp_references = index_set_new();
        for node_id in references {
            // TODO: fix this ugly condition
            if EntityType::is_global_node(&node_id) {
                // May overwrite existing node refs (for better visibility origin)
                self.immortal_node_refs.insert(
                    node_id.clone(),
                    RENodeRefData {
                        visibility: RENodeVisibilityOrigin::Normal,
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
                track_handle,
            },
        );
        self.next_lock_handle = self.next_lock_handle + 1;

        // Update lock count on the node
        if let Some(counter) = self.owned_root_nodes.get_mut(node_id) {
            *counter += 1;
        }

        Ok(lock_handle)
    }

    pub fn drop_lock<'s>(
        &mut self,
        heap: &mut Heap,
        track: &mut Track<'s>,
        lock_handle: LockHandle,
    ) -> Result<(), CallFrameDropLockError> {
        let substate_lock = self
            .locks
            .remove(&lock_handle)
            .ok_or(CallFrameDropLockError::LockNotFound(lock_handle))?;

        let node_id = &substate_lock.node_id;
        let module_id = substate_lock.module_id;
        let substate_key = &substate_lock.substate_key;

        if substate_lock.flags.contains(LockFlags::MUTABLE) {
            let substate_ref = self.get_substate(heap, track, node_id, module_id, substate_key)?;
            let (references, owned_nodes) = substate_ref.references_and_owned_nodes();

            // Reserving original Vec element order with `IndexSet`
            let mut new_children: IndexSet<NodeId> = index_set_new();
            for own in owned_nodes {
                if !new_children.insert(own) {
                    return Err(RuntimeError::KernelError(
                        KernelError::ContainsDuplicatedOwns,
                    ));
                }
            }

            // Check references exist
            for reference in references {
                self.check_node_visibility(&reference)?;
            }

            for old_child in &substate_lock.initial_owned_nodes {
                if !new_children.remove(old_child) {
                    // TODO: revisit logic here!
                    if SubstateProperties::is_persisted(&substate_key) {
                        return Err(RuntimeError::KernelError(KernelError::StoredNodeRemoved(
                            old_child.clone(),
                        )));
                    }

                    // Owned nodes discarded by the substate go back to the call frame,
                    // and must be explicitly dropped.
                    self.owned_root_nodes.insert(old_child.clone(), 0);
                }
            }

            for child_id in &new_children {
                self.take_node_internal(child_id)?;

                // TODO: Move this check into system layer
                if let Ok(info) = heap.get_substate(
                    child_id,
                    TypedModuleId::TypeInfo,
                    &TypeInfosubstate_key::TypeInfo.into(),
                ) {
                    let type_info: &TypeInfoSubstate = info.into();
                    match type_info {
                        TypeInfoSubstate::Object {
                            package_address,
                            blueprint_name,
                            ..
                        } => {
                            SubstateProperties::verify_can_own(
                                &substate_key,
                                *package_address,
                                blueprint_name.as_str(),
                            )?;
                        }
                        TypeInfoSubstate::KeyValueStore(..) => {}
                    }
                }
            }

            if !heap.contains_node(&node_id) {
                heap.move_nodes_to_store(track, new_children.into_iter().collect())?;
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

        let flags = substate_lock.flags;

        if !(matches!(offset, SubstateKey::KeyValueStore(..))) {
            if !heap.contains_node(&node_id) {
                track
                    .release_lock(
                        SubstateId(node_id.clone(), module_id, offset.clone()),
                        flags.contains(LockFlags::FORCE_WRITE),
                    )
                    .map_err(|e| KernelError::TrackError(Box::new(e)))?;
            }
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
        frame.add_ref(
            NodeId::GlobalObject(RADIX_TOKEN.into()),
            RENodeVisibilityOrigin::Normal,
        );
        frame.add_ref(
            NodeId::GlobalObject(SYSTEM_TOKEN.into()),
            RENodeVisibilityOrigin::Normal,
        );
        frame.add_ref(
            NodeId::GlobalObject(ECDSA_SECP256K1_TOKEN.into()),
            RENodeVisibilityOrigin::Normal,
        );
        frame.add_ref(
            NodeId::GlobalObject(EDDSA_ED25519_TOKEN.into()),
            RENodeVisibilityOrigin::Normal,
        );
        frame.add_ref(
            NodeId::GlobalObject(PACKAGE_TOKEN.into()),
            RENodeVisibilityOrigin::Normal,
        );
        frame.add_ref(
            NodeId::GlobalObject(PACKAGE_OWNER_TOKEN.into()),
            RENodeVisibilityOrigin::Normal,
        );
        frame.add_ref(
            NodeId::GlobalObject(IDENTITY_OWNER_TOKEN.into()),
            RENodeVisibilityOrigin::Normal,
        );
        frame.add_ref(
            NodeId::GlobalObject(ACCOUNT_OWNER_TOKEN.into()),
            RENodeVisibilityOrigin::Normal,
        );
        frame.add_ref(
            NodeId::GlobalObject(EPOCH_MANAGER.into()),
            RENodeVisibilityOrigin::Normal,
        );
        frame.add_ref(
            NodeId::GlobalObject(CLOCK.into()),
            RENodeVisibilityOrigin::Normal,
        );
        frame.add_ref(
            NodeId::GlobalObject(Address::Package(FAUCET_PACKAGE)),
            RENodeVisibilityOrigin::Normal,
        );

        frame
    }

    pub fn new_child_from_parent(
        parent: &mut CallFrame,
        actor: Actor,
        call_frame_update: CallFrameUpdate,
    ) -> Result<Self, RuntimeError> {
        let mut owned_heap_nodes = index_map_new();
        let mut next_node_refs = NonIterMap::new();

        for node_id in call_frame_update.nodes_to_move {
            parent.take_node_internal(&node_id)?;
            owned_heap_nodes.insert(node_id, 0u32);
        }

        for node_id in call_frame_update.node_refs_to_copy {
            let visibility = parent
                .get_node_visibility(&node_id)
                .ok_or(FrameUpdateError::RefNotFound(node_id))?;
            next_node_refs.insert(node_id, RENodeRefData::new(visibility));
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
    ) -> Result<(), FrameUpdateError> {
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
                .ok_or(FrameUpdateError::RefNotFound(node_id))?;

            to.immortal_node_refs
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
    ) -> Result<(), CallFrameDropLockError> {
        let lock_handles: Vec<LockHandle> = self.locks.keys().cloned().collect();

        for lock_handle in lock_handles {
            self.drop_lock(heap, track, lock_handle)?;
        }

        Ok(())
    }

    fn take_node_internal(&mut self, node_id: &NodeId) -> Result<(), FrameUpdateError> {
        match self.owned_root_nodes.remove(node_id) {
            None => {
                return Err(FrameUpdateError::OwnNotFound(node_id.clone()));
            }
            Some(lock_count) => {
                if lock_count == 0 {
                    Ok(())
                } else {
                    Err(FrameUpdateError::NodeLocked(node_id.clone()))
                }
            }
        }
    }

    pub fn create_node<'f, 's>(
        &mut self,
        node_id: NodeId,
        re_node: NodeInit,
        node_modules: BTreeMap<TypedModuleId, ModuleInit>,
        heap: &mut Heap,
        track: &'f mut Track<'s>,
        push_to_store: bool,
    ) -> Result<(), RuntimeError> {
        let mut substates = BTreeMap::new();
        let self_substates = re_node.to_substates();
        for (substate_key, substate) in self_substates {
            substates.insert((TypedModuleId::ObjectState, substate_key), substate);
        }
        for (node_module_id, module_init) in node_modules {
            for (substate_key, substate) in module_init.to_substates() {
                substates.insert((node_module_id, substate_key), substate);
            }
        }

        for ((_module_id, substate_key), substate) in &substates {
            let substate_ref = substate.to_ref();
            let (_, owned) = substate_ref.references_and_owned_nodes();
            for child_id in owned {
                self.take_node_internal(&child_id)?;

                // TODO: Move this logic into system layer
                if let Ok(info) = heap.get_substate(
                    &child_id,
                    TypedModuleId::TypeInfo,
                    &TypeInfosubstate_key::TypeInfo.into(),
                ) {
                    let type_info: &TypeInfoSubstate = info.into();
                    match type_info {
                        TypeInfoSubstate::Object {
                            package_address,
                            blueprint_name,
                            ..
                        } => {
                            SubstateProperties::verify_can_own(
                                &substate_key,
                                *package_address,
                                blueprint_name.as_str(),
                            )?;
                        }
                        TypeInfoSubstate::KeyValueStore(..) => {}
                    }
                }

                if push_to_store {
                    heap.move_node_to_store(track, child_id)?;
                }
            }
        }

        if push_to_store {
            for ((module_id, substate_key), substate) in substates {
                track
                    .insert_substate(SubstateId(node_id, module_id, offset), substate)
                    .map_err(|e| KernelError::TrackError(Box::new(e)))?;
            }

            self.add_ref(node_id, RENodeVisibilityOrigin::Normal);
        } else {
            // Insert node into heap
            let heap_root_node = HeapNode {
                substates,
                //child_nodes,
            };
            heap.insert_node(node_id, heap_root_node);
            self.owned_root_nodes.insert(node_id, 0u32);
        }

        Ok(())
    }

    pub fn add_ref(&mut self, node_id: NodeId, visibility: RENodeVisibilityOrigin) {
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
    ) -> Result<HeapNode, FrameUpdateError> {
        self.take_node_internal(node_id)?;
        let node = heap.remove_node(node_id);
        for (_, module) in &node.substates {
            for (_, substate_value) in module {
                let refs = substate_value.references();
                let child_nodes = substate_value.owned_node_ids();
                for node_ref in refs {
                    self.immortal_node_refs.insert(
                        node_ref.clone(),
                        RENodeRefData {
                            visibility: RENodeVisibilityOrigin::Normal,
                        },
                    );
                }

                for child_node in child_nodes {
                    self.owned_root_nodes.insert(child_node.clone(), 0u32);
                }
            }
        }
        Ok(node)
    }

    pub fn get_node_visibility(&self, node_id: &NodeId) -> Option<RENodeVisibilityOrigin> {
        if self.owned_root_nodes.contains_key(node_id) {
            Some(RENodeVisibilityOrigin::Normal)
        } else if let Some(_) = self.temp_node_refs.get(node_id) {
            Some(RENodeVisibilityOrigin::Normal)
        } else if let Some(ref_data) = self.immortal_node_refs.get(node_id) {
            Some(ref_data.visibility)
        } else {
            None
        }
    }
}
