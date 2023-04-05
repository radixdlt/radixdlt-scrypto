use crate::kernel::actor::Actor;
use crate::system::node_init::NodeInit;
use crate::system::node_modules::type_info::TypeInfoSubstate;
use crate::system::node_properties::NodeProperties;
use crate::types::*;
use radix_engine_interface::api::node_modules::metadata::METADATA_BLUEPRINT;
use radix_engine_interface::api::substate_api::LockFlags;
use radix_engine_interface::blueprints::resource::{BUCKET_BLUEPRINT, PROOF_BLUEPRINT};
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
pub enum RefType {
    Normal,
    DirectAccess,
}

/// A lock on a substate controlled by a call frame
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubstateLock {
    pub node_id: NodeId,
    pub module_id: TypedModuleId,
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
    SubstateNotFound,
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

impl CallFrame {
    fn get_type_info<'s>(
        node_id: &NodeId,
        heap: &mut Heap,
        track: &mut Track<'s>,
    ) -> Option<TypeInfoSubstate> {
        if let Some(substate) = heap.get_substate(
            node_id,
            TypedModuleId::TypeInfo,
            &TypeInfoOffset::TypeInfo.into(),
        ) {
            let type_info: TypeInfoSubstate = substate.as_typed().unwrap();
            Some(type_info)
        } else if let Ok(handle) = track.acquire_lock(
            node_id,
            TypedModuleId::TypeInfo.into(),
            &TypeInfoOffset::TypeInfo.into(),
            LockFlags::read_only(),
        ) {
            let type_info: TypeInfoSubstate = track.read_substate(handle).as_typed().unwrap();
            track.release_lock(handle);
            Some(type_info)
        } else {
            None
        }
    }

    pub fn acquire_lock<'s>(
        &mut self,
        heap: &mut Heap,
        track: &mut Track<'s>,
        node_id: &NodeId,
        module_id: TypedModuleId,
        substate_key: &SubstateKey,
        flags: LockFlags,
    ) -> Result<LockHandle, LockSubstateError> {
        // Check node visibility
        self.get_node_visibility(node_id)
            .ok_or(LockSubstateError::NodeNotInCallFrame(node_id.clone()))?;

        // Virtualization
        // TODO: clean up the naughty!
        let virtualization_enabled = {
            if module_id == TypedModuleId::Metadata {
                true
            } else if module_id == TypedModuleId::ObjectState {
                if let Some(type_info) = Self::get_type_info(node_id, heap, track) {
                    match type_info {
                        TypeInfoSubstate::Object(ObjectInfo { blueprint, .. }) => {
                            blueprint.package_address == METADATA_PACKAGE
                                && blueprint.blueprint_name == METADATA_BLUEPRINT
                        }
                        TypeInfoSubstate::KeyValueStore(_) => true,
                    }
                } else {
                    false
                }
            } else {
                false
            }
        };
        if virtualization_enabled {
            if heap.contains_node(node_id) {
                if heap
                    .get_substate(node_id, module_id.into(), substate_key)
                    .is_none()
                {
                    heap.put_substate(
                        node_id.clone(),
                        module_id,
                        substate_key.clone(),
                        IndexedScryptoValue::from_typed(&Option::<ScryptoValue>::None),
                    );
                }
            } else {
                match track.acquire_lock(
                    node_id,
                    module_id.into(),
                    substate_key,
                    LockFlags::read_only(),
                ) {
                    Ok(handle) => {
                        track.release_lock(handle);
                    }
                    Err(error) => {
                        if matches!(error, AcquireLockError::NotFound(_, _, _)) {
                            track.insert_substate(
                                node_id.clone(),
                                module_id.into(),
                                substate_key.clone(),
                                IndexedScryptoValue::from_typed(&Option::<ScryptoValue>::None),
                            )
                        }
                    }
                }
            };
        }

        // Lock and read the substate
        let mut store_handle = None;
        let substate_value = if heap.contains_node(node_id) {
            // TODO: make Heap more like Track?
            if flags.contains(LockFlags::UNMODIFIED_BASE) {
                return Err(LockSubstateError::LockUnmodifiedBaseOnHeapNode);
            }
            heap.get_substate(node_id, module_id.into(), substate_key)
                .ok_or(LockSubstateError::SubstateNotFound)?
        } else {
            let handle = track
                .acquire_lock(node_id, module_id.into(), substate_key, flags)
                .map_err(|x| LockSubstateError::TrackError(Box::new(x)))?;
            store_handle = Some(handle);
            track.read_substate(handle)
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

    pub fn drop_lock<'s>(
        &mut self,
        heap: &mut Heap,
        track: &mut Track<'s>,
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
                track.read_substate(handle)
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

                // TODO: Move this check into system layer
                if let Some(info) = heap.get_substate(
                    child_id,
                    TypedModuleId::TypeInfo.into(),
                    &TypeInfoOffset::TypeInfo.into(),
                ) {
                    let type_info: TypeInfoSubstate = info.as_typed().unwrap();
                    match type_info {
                        TypeInfoSubstate::Object(ObjectInfo { blueprint, .. }) => {
                            if !NodeProperties::can_own(
                                &substate_key,
                                blueprint.package_address,
                                blueprint.blueprint_name.as_str(),
                            ) {
                                return Err(UnlockSubstateError::CantOwn(child_id.clone()));
                            }
                        }
                        TypeInfoSubstate::KeyValueStore(..) => {}
                    }
                }
            }

            if !heap.contains_node(&node_id) {
                for child in &new_children {
                    Self::move_node_to_store(heap, track, child)?;
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
            track.release_lock(handle);
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

    pub fn read_substate<'f, 's>(
        &mut self,
        heap: &'f mut Heap,
        track: &'f mut Track<'s>,
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
            Ok(track.read_substate(*store_handle))
        } else {
            Ok(heap
                .get_substate(node_id, *module_id, substate_key)
                .expect("Substate missing in heap"))
        }
    }

    pub fn write_substate<'f, 's>(
        &mut self,
        heap: &'f mut Heap,
        track: &'f mut Track<'s>,
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
            track.write_substate(*store_handle, substate);
        } else {
            heap.put_substate(*node_id, *module_id, substate_key.clone(), substate);
        }
        Ok(())
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
        frame.add_ref(IDENTITY_OWNER_TOKEN.into(), RefType::Normal);
        frame.add_ref(ACCOUNT_OWNER_TOKEN.into(), RefType::Normal);
        frame.add_ref(EPOCH_MANAGER.into(), RefType::Normal);
        frame.add_ref(CLOCK.into(), RefType::Normal);
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

    pub fn drop_all_locks<'s>(
        &mut self,
        heap: &mut Heap,
        track: &mut Track<'s>,
    ) -> Result<(), UnlockSubstateError> {
        let lock_handles: Vec<LockHandle> = self.locks.keys().cloned().collect();

        for lock_handle in lock_handles {
            self.drop_lock(heap, track, lock_handle)?;
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

    pub fn create_node<'f, 's>(
        &mut self,
        node_id: NodeId,
        node_init: NodeInit,
        node_modules: BTreeMap<TypedModuleId, BTreeMap<SubstateKey, IndexedScryptoValue>>,
        heap: &mut Heap,
        track: &'f mut Track<'s>,
        push_to_store: bool,
    ) -> Result<(), UnlockSubstateError> {
        let mut substates = BTreeMap::new();
        substates.insert(TypedModuleId::ObjectState, node_init.to_substates());
        substates.extend(node_modules);

        for (_module_id, module) in &substates {
            for (substate_key, substate_value) in module {
                // FIXME there is a huge mismatch between drop_lock and create_node
                // We need to apply the same checks!

                for child_id in substate_value.owned_node_ids() {
                    self.take_node_internal(child_id)
                        .map_err(UnlockSubstateError::MoveError)?;

                    // TODO: Move this check into system layer
                    if let Some(info) = heap.get_substate(
                        child_id,
                        TypedModuleId::TypeInfo.into(),
                        &TypeInfoOffset::TypeInfo.into(),
                    ) {
                        let type_info: TypeInfoSubstate = info.as_typed().unwrap();
                        match type_info {
                            TypeInfoSubstate::Object(ObjectInfo { blueprint, .. }) => {
                                if !NodeProperties::can_own(
                                    &substate_key,
                                    blueprint.package_address,
                                    blueprint.blueprint_name.as_str(),
                                ) {
                                    return Err(UnlockSubstateError::CantOwn(child_id.clone()));
                                }
                            }
                            TypeInfoSubstate::KeyValueStore(..) => {}
                        }
                    }

                    if push_to_store {
                        Self::move_node_to_store(heap, track, child_id)?;
                    }
                }
            }
        }

        if push_to_store {
            for (module_id, module) in substates {
                for (substate_key, substate_value) in module {
                    for reference in substate_value.references() {
                        if !reference.is_global() {
                            return Err(UnlockSubstateError::CantStoreLocalReference(*reference));
                        }
                    }

                    track.insert_substate(node_id, module_id.into(), substate_key, substate_value);
                }
            }

            self.add_ref(node_id, RefType::Normal);
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
    ) -> Result<HeapNode, MoveError> {
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
                            ref_type: RefType::Normal,
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

    pub fn move_node_to_store(
        heap: &mut Heap,
        track: &mut Track,
        node_id: &NodeId,
    ) -> Result<(), UnlockSubstateError> {
        // FIXME: Clean this up
        let can_be_stored = if node_id.is_global() {
            true
        } else {
            if let Some(type_info) = Self::get_type_info(node_id, heap, track) {
                match type_info {
                    TypeInfoSubstate::Object(ObjectInfo { blueprint, .. })
                        if blueprint.package_address == RESOURCE_MANAGER_PACKAGE
                            && (blueprint.blueprint_name == BUCKET_BLUEPRINT
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

        let node = heap.remove_node(node_id);
        for (module_id, module) in node.substates {
            for (substate_key, substate_value) in module {
                for reference in substate_value.references() {
                    if !reference.is_global() {
                        return Err(UnlockSubstateError::CantStoreLocalReference(*reference));
                    }
                }

                for node in substate_value.owned_node_ids() {
                    Self::move_node_to_store(heap, track, node)?;
                }

                track.insert_substate(
                    node_id.clone(),
                    module_id.into(),
                    substate_key,
                    substate_value,
                );
            }
        }

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
