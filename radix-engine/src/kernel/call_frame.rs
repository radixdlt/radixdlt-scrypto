use crate::kernel::actor::MethodType;
use crate::kernel::substate_io;
use crate::kernel::substate_io::{SubstateIO, SubstateLocation};
use crate::kernel::substate_locks::SubstateLocks;
use crate::system::node_modules::type_info::TypeInfoSubstate;
use crate::track::interface::{
    CallbackError, NodeSubstates, StoreAccess, SubstateStore, TrackGetSubstateError,
    TrackedSubstateInfo,
};
use crate::types::*;
use radix_engine_interface::api::field_api::LockFlags;
use radix_engine_interface::blueprints::resource::{
    FUNGIBLE_BUCKET_BLUEPRINT, FUNGIBLE_PROOF_BLUEPRINT, NON_FUNGIBLE_BUCKET_BLUEPRINT,
    NON_FUNGIBLE_PROOF_BLUEPRINT,
};
use radix_engine_interface::types::{LockHandle, NodeId, SubstateKey};
use radix_engine_store_interface::db_key_mapper::SubstateKeyContent;

use super::actor::{Actor, BlueprintHookActor, FunctionActor, MethodActor};
use super::heap::{Heap, HeapOpenSubstateError, HeapRemoveModuleError, HeapRemoveNodeError};

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
/// A lock on a substate controlled by a call frame
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenedSubstate<L> {
    pub non_global_references: IndexSet<NodeId>,
    pub owned_nodes: IndexSet<NodeId>,
    pub global_lock_handle: u32,
    pub updated: bool,
    pub location: SubstateLocation,
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

pub struct NodeVisibility(pub BTreeSet<Visibility>);

impl NodeVisibility {
    /// Note that system may enforce further constraints on this.
    /// For instance, system currently only allows substates of actor,
    /// actor's outer object, and any visible key value store.
    pub fn is_visible(&self) -> bool {
        !self.0.is_empty()
    }

    pub fn can_be_invoked(&self, direct_access: bool) -> bool {
        if direct_access {
            self.0.iter().any(|x| x.is_direct_access())
        } else {
            self.0.iter().any(|x| x.is_normal())
        }
    }

    pub fn can_be_referenced_in_substate(&self) -> bool {
        self.0.iter().any(|x| x.is_normal())
    }

    pub fn can_be_reference_copied_to_frame(&self) -> Option<StableReferenceType> {
        for v in &self.0 {
            if let Visibility::StableReference(t) = v {
                return Some(t.clone());
            }
        }
        return None;
    }
}

/// A call frame is the basic unit that forms a transaction call stack, which keeps track of the
/// owned objects and references by this function.
pub struct CallFrame<L> {
    /// The frame id
    depth: usize,

    /// FIXME: redo actor generification
    actor: Actor,

    /// Owned nodes which by definition must live on heap
    /// Also keeps track of number of locks on this node, to prevent locked node from moving.
    owned_root_nodes: IndexMap<NodeId, usize>,

    /// References to non-GLOBAL nodes, obtained from substate loading, ref counted.
    transient_references: NonIterMap<NodeId, usize>,

    /// Stable references points to nodes in track, which can't moved/deleted.
    /// Current two types: `GLOBAL` (root, stored) and `DirectAccess`.
    stable_references: NonIterMap<NodeId, StableReferenceType>,

    next_lock_handle: LockHandle,
    locks: IndexMap<LockHandle, OpenedSubstate<L>>,
}

/// Represents an error when creating a new frame.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum CreateFrameError {
    ActorBeingMoved(NodeId),
    PassMessageError(PassMessageError),
}

/// Represents an error when passing message between frames.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum PassMessageError {
    TakeNodeError(TakeNodeError),
    StableRefNotFound(NodeId),
}

/// Represents an error when attempting to lock a substate.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum OpenSubstateError {
    NodeNotVisible(NodeId),
    HeapError(HeapOpenSubstateError),
    TrackError(Box<TrackGetSubstateError>),
    SubstateLocked(NodeId, PartitionNumber, SubstateKey),
    LockUnmodifiedBaseOnHeapNode,
    LockUnmodifiedBaseOnNewSubstate(NodeId, PartitionNumber, SubstateKey),
    LockUnmodifiedBaseOnOnUpdatedSubstate(NodeId, PartitionNumber, SubstateKey),
}

/// Represents an error when dropping a substate lock.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum CloseSubstateError {
    LockNotFound(LockHandle),
    ContainsDuplicatedOwns,
    TakeNodeError(TakeNodeError),
    RefNotFound(NodeId),
    NonGlobalRefNotAllowed(NodeId),
    CantDropNodeInStore(NodeId),
    PersistNodeError(PersistNodeError),
}

/// Represents an error when creating a node.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum CreateNodeError {
    TakeNodeError(TakeNodeError),
    RefNotFound(NodeId),
    NonGlobalRefNotAllowed(NodeId),
    PersistNodeError(PersistNodeError),
}

/// Represents an error when dropping a node.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum DropNodeError {
    TakeNodeError(TakeNodeError),
    NodeBorrowed(NodeId, usize),
}

/// Represents an error when persisting a node into store.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum PersistNodeError {
    CantBeStored(NodeId),
    NonGlobalRefNotAllowed(NodeId),
    NodeBorrowed(NodeId, usize),
}

/// Represents an error when taking a node from current frame.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum TakeNodeError {
    OwnNotFound(NodeId),
    OwnLocked(NodeId),
}

/// Represents an error when listing the node modules of a node.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum ListNodeModuleError {
    NodeNotVisible(NodeId),
    NodeNotInHeap(NodeId),
}

/// Represents an error when moving modules from one node to another.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum MoveModuleError {
    NodeNotAvailable(NodeId),
    HeapRemoveModuleErr(HeapRemoveModuleError),
    NonGlobalRefNotAllowed(NodeId),
    PersistNodeError(PersistNodeError),
}

/// Represents an error when reading substates.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum ReadSubstateError {
    LockNotFound(LockHandle),
}

/// Represents an error when writing substates.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum WriteSubstateError {
    LockNotFound(LockHandle),
    NoWritePermission,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum CallFrameSetSubstateError {
    NodeNotVisible(NodeId),
    SubstateLocked(NodeId, PartitionNumber, SubstateKey),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum CallFrameRemoveSubstateError {
    NodeNotVisible(NodeId),
    SubstateLocked(NodeId, PartitionNumber, SubstateKey),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum CallFrameScanKeysError {
    NodeNotVisible(NodeId),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum CallFrameDrainSubstatesError {
    NodeNotVisible(NodeId),
    OwnedNodeNotSupported(NodeId),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum CallFrameScanSortedSubstatesError {
    NodeNotVisible(NodeId),
}

impl<L: Clone> CallFrame<L> {
    pub fn new_root(actor: Actor) -> Self {
        Self {
            depth: 0,
            actor,
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
        let mut frame = Self {
            depth: parent.depth + 1,
            actor,
            stable_references: NonIterMap::new(),
            transient_references: NonIterMap::new(),
            owned_root_nodes: index_map_new(),
            next_lock_handle: 0u32,
            locks: index_map_new(),
        };

        // Copy references and move nodes
        Self::pass_message(parent, &mut frame, message)
            .map_err(CreateFrameError::PassMessageError)?;

        // Make sure actor isn't part of the owned nodes
        if let Some(node_id) = frame.actor.node_id() {
            if frame.owned_root_nodes.contains_key(&node_id) {
                return Err(CreateFrameError::ActorBeingMoved(node_id));
            }
        }

        // Additional global references
        let mut additional_global_refs = Vec::new();

        if let Some(blueprint_id) = frame.actor.blueprint_id() {
            additional_global_refs.push(blueprint_id.package_address.into());
        }

        match &frame.actor {
            Actor::Root => {}
            Actor::Method(MethodActor {
                method_type,
                object_info,
                ..
            }) => {
                if let MethodType::OnStoredObject(global_address) = method_type {
                    additional_global_refs.push(global_address.clone());
                }
                if let OuterObjectInfo::Inner { outer_object } =
                    object_info.blueprint_info.outer_obj_info
                {
                    additional_global_refs.push(outer_object.clone());
                }
            }
            Actor::Function(FunctionActor { blueprint_id, .. })
            | Actor::BlueprintHook(BlueprintHookActor { blueprint_id, .. }) => {
                additional_global_refs.push(blueprint_id.package_address.clone().into());
            }
        }

        for reference in additional_global_refs {
            frame.add_global_reference(reference);
        }

        Ok(frame)
    }

    pub fn pass_message(
        from: &mut CallFrame<L>,
        to: &mut CallFrame<L>,
        message: Message,
    ) -> Result<(), PassMessageError> {
        for node_id in message.move_nodes {
            // Note that this has no impact on the `transient_references` because
            // we don't allow move of "locked nodes".
            from.take_node_internal(&node_id)
                .map_err(PassMessageError::TakeNodeError)?;
            to.owned_root_nodes.insert(node_id, 0);
        }

        // Only allow move of `Global` and `DirectAccess` references
        for node_id in message.copy_references {
            if let Some(t) = from
                .get_node_visibility(&node_id)
                .can_be_reference_copied_to_frame()
            {
                // Note that GLOBAL and DirectAccess references are mutually exclusive,
                // so okay to overwrite
                to.stable_references.insert(node_id, t);
            } else {
                return Err(PassMessageError::StableRefNotFound(node_id));
            }
        }

        Ok(())
    }

    pub fn depth(&self) -> usize {
        self.depth
    }

    pub fn actor(&self) -> &Actor {
        &self.actor
    }

    pub fn open_substate<S: SubstateStore, E, F: FnMut(StoreAccess) -> Result<(), E>>(
        &mut self,
        substate_io: &mut SubstateIO<S>,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        substate_key: &SubstateKey,
        flags: LockFlags,
        on_store_access: F,
        default: Option<fn() -> IndexedScryptoValue>,
        data: L,
    ) -> Result<(LockHandle, usize), CallbackError<OpenSubstateError, E>> {
        // Check node visibility
        if !self.get_node_visibility(node_id).is_visible() {
            return Err(CallbackError::Error(OpenSubstateError::NodeNotVisible(
                node_id.clone(),
            )));
        }

        let (global_lock_handle, substate_value, substate_location) = substate_io.open_substate(
            node_id,
            partition_num,
            substate_key,
            flags,
            on_store_access,
            default,
        )?;

        // Analyze owns and references in the substate
        let mut non_global_references = index_set_new(); // du-duplicated
        let mut owned_nodes = index_set_new();
        for node_id in substate_value.references() {
            if node_id.is_global() {
                // Again, safe to overwrite because Global and DirectAccess are exclusive.
                self.stable_references
                    .insert(node_id.clone(), StableReferenceType::Global);
            } else {
                non_global_references.insert(node_id.clone());
            }
        }
        for node_id in substate_value.owned_nodes() {
            if !owned_nodes.insert(node_id.clone()) {
                panic!("Duplicated own found in substate");
            }
        }

        // Expand transient reference set
        for reference in &non_global_references {
            self.transient_references
                .entry(reference.clone())
                .or_default()
                .add_assign(1);
        }
        for own in &owned_nodes {
            self.transient_references
                .entry(own.clone())
                .or_default()
                .add_assign(1);
        }

        // Issue lock handle
        let lock_handle = self.next_lock_handle;
        self.locks.insert(
            lock_handle,
            OpenedSubstate {
                non_global_references,
                owned_nodes,
                global_lock_handle,
                updated: false,
                location: substate_location,
                data,
            },
        );
        self.next_lock_handle = self.next_lock_handle + 1;

        // Update lock count on the node
        if let Some(counter) = self.owned_root_nodes.get_mut(node_id) {
            *counter += 1;
        }

        Ok((lock_handle, substate_value.len()))
    }

    pub fn read_substate<'f, S: SubstateStore>(
        &mut self,
        substate_io: &'f mut SubstateIO<S>,
        lock_handle: LockHandle,
    ) -> Result<&'f IndexedScryptoValue, ReadSubstateError> {
        let OpenedSubstate {
            global_lock_handle, ..
        } = self
            .locks
            .get(&lock_handle)
            .ok_or(ReadSubstateError::LockNotFound(lock_handle))?;

        let substate = substate_io.read_substate(*global_lock_handle);
        Ok(substate)
    }

    pub fn write_substate<'f, S: SubstateStore>(
        &mut self,
        substate_io: &'f mut SubstateIO<S>,
        lock_handle: LockHandle,
        substate: IndexedScryptoValue,
    ) -> Result<(), WriteSubstateError> {
        let OpenedSubstate {
            global_lock_handle,
            updated,
            ..
        } = self
            .locks
            .get_mut(&lock_handle)
            .ok_or(WriteSubstateError::LockNotFound(lock_handle))?;

        *updated = true;
        substate_io.write_substate(*global_lock_handle, substate)?;

        Ok(())
    }

    pub fn close_substate<S: SubstateStore, E, F: FnMut(StoreAccess) -> Result<(), E>>(
        &mut self,
        substate_io: &mut SubstateIO<S>,
        lock_handle: LockHandle,
        on_store_access: &mut F,
    ) -> Result<(), CallbackError<CloseSubstateError, E>> {

        let OpenedSubstate {
            global_lock_handle,
            owned_nodes,
            non_global_references,
            updated,
            location,
            ..
        } = self
            .locks
            .remove(&lock_handle)
            .ok_or_else(|| CallbackError::Error(CloseSubstateError::LockNotFound(lock_handle)))?;

        if updated {
            let updated_substate = substate_io.read_substate(global_lock_handle);
            let mut new_owned_nodes: IndexSet<NodeId> = index_set_new();
            for node_id in updated_substate.owned_nodes() {
                new_owned_nodes.insert(*node_id);
            }

            for own in &new_owned_nodes {
                if !owned_nodes.contains(own) {
                    // Node no longer owned by frame
                    self.take_node_internal(own)
                        .map_err(|e| CallbackError::Error(CloseSubstateError::TakeNodeError(e)))?;
                }
            }

            for own in &owned_nodes {
                if !new_owned_nodes.contains(own) {
                    // Node detached
                    if location.eq(&SubstateLocation::Store) {
                        return Err(CallbackError::Error(
                            CloseSubstateError::CantDropNodeInStore(own.clone()),
                        ));
                    }
                    // Owned nodes discarded by the substate go back to the call frame,
                    // and must be explicitly dropped.
                    // FIXME(Yulong): I suspect this is buggy as one can detach a locked non-root
                    // node, move and drop; which will cause invalid lock handle in previous frames.
                    // FIXME(Josh): Would prefer removing this case entirely as this edge case
                    // means that a component's logic may or may not work depending on whether
                    // it's in the store or the heap, which I think feels very unintuitive.
                    // Rather, let's fix the specific worktop drop bucket issue
                    self.owned_root_nodes.insert(own.clone(), 0);
                }
            }

            //====================
            // Process references
            //====================
            let mut new_references: IndexSet<NodeId> = index_set_new();
            for own in updated_substate.references() {
                // Deduplicate
                new_references.insert(own.clone());
            }
            for reference in &new_references {
                if !non_global_references.contains(reference) {
                    // handle added references
                    if !self
                        .get_node_visibility(reference)
                        .can_be_referenced_in_substate()
                    {
                        return Err(CallbackError::Error(CloseSubstateError::RefNotFound(
                            reference.clone(),
                        )));
                    }
                }
            }
        }

        let (node_id, ..) = substate_io.close_substate(global_lock_handle, on_store_access)?;

        // Shrink transient reference set
        for reference in non_global_references {
            let cnt = self.transient_references.remove(&reference).unwrap_or(0);
            if cnt > 1 {
                self.transient_references.insert(reference, cnt - 1);
            }
        }
        for own in owned_nodes {
            let cnt = self.transient_references.remove(&own).unwrap_or(0);
            if cnt > 1 {
                self.transient_references.insert(own, cnt - 1);
            }
        }

        // Update node lock count
        if let Some(counter) = self.owned_root_nodes.get_mut(&node_id) {
            *counter -= 1;
        }

        Ok(())
    }

    pub fn get_lock_info(&self, lock_handle: LockHandle) -> Option<L> {
        self.locks
            .get(&lock_handle)
            .map(|substate_lock| substate_lock.data.clone())
    }

    pub fn create_node<'f, S: SubstateStore, E, F: FnMut(StoreAccess) -> Result<(), E>>(
        &mut self,
        substate_io: &mut SubstateIO<S>,
        node_id: NodeId,
        node_substates: NodeSubstates,
        push_to_store: bool,
        on_store_access: &mut F,
    ) -> Result<(), CallbackError<CreateNodeError, E>> {
        for (_partition_number, module) in &node_substates {
            for (_substate_key, substate_value) in module {
                //==============
                // Process owns
                //==============
                for own in substate_value.owned_nodes() {
                    self.take_node_internal(own)
                        .map_err(|e| CallbackError::Error(CreateNodeError::TakeNodeError(e)))?;
                    if push_to_store {
                        substate_io
                            .move_node_to_store(own, on_store_access)
                            .map_err(|e| e.map(CreateNodeError::PersistNodeError))?;
                    }
                }

                //===================
                // Process reference
                //===================
                for reference in substate_value.references() {
                    if !self
                        .get_node_visibility(reference)
                        .can_be_referenced_in_substate()
                    {
                        return Err(CallbackError::Error(CreateNodeError::RefNotFound(
                            reference.clone(),
                        )));
                    }

                    if push_to_store && !reference.is_global() {
                        return Err(CallbackError::Error(
                            CreateNodeError::NonGlobalRefNotAllowed(*reference),
                        ));
                    }

                    if substate_io.heap.contains_node(reference) {
                        substate_io.heap.increase_borrow_count(reference);
                    } else {
                        // No op
                    }
                }
            }
        }

        if push_to_store {
            self.stable_references
                .insert(node_id, StableReferenceType::Global);
            substate_io
                .store
                .create_node(node_id, node_substates, on_store_access)
                .map_err(CallbackError::CallbackError)?;
        } else {
            substate_io.heap.create_node(node_id, node_substates);
            self.owned_root_nodes.insert(node_id, 0);
        };

        Ok(())
    }

    /// Removes node from call frame and owned nodes will be possessed by this call frame.
    pub fn drop_node(
        &mut self,
        heap: &mut Heap,
        node_id: &NodeId,
    ) -> Result<NodeSubstates, DropNodeError> {
        self.take_node_internal(node_id)
            .map_err(DropNodeError::TakeNodeError)?;
        let node_substates = match heap.remove_node(node_id) {
            Ok(substates) => substates,
            Err(HeapRemoveNodeError::NodeNotFound(node_id)) => {
                panic!("Frame owned node {:?} not found in heap", node_id)
            }
            Err(HeapRemoveNodeError::NodeBorrowed(node_id, count)) => {
                return Err(DropNodeError::NodeBorrowed(node_id, count));
            }
        };
        for (_, module) in &node_substates {
            for (_, substate_value) in module {
                //=============
                // Process own
                //=============
                for own in substate_value.owned_nodes() {
                    // FIXME: This is problematic, as owned node must have been locked
                    // In general, we'd like to move node locking/borrowing to heap.
                    self.owned_root_nodes.insert(own.clone(), 0);
                }

                //====================
                // Process references
                //====================
                for reference in substate_value.references() {
                    if reference.is_global() {
                        // Expand stable references
                        // We keep all global references even if the owning substates are dropped.
                        // Revisit this if the reference model is changed.
                        self.stable_references
                            .insert(reference.clone(), StableReferenceType::Global);
                    } else {
                        if heap.contains_node(reference) {
                            // This substate is dropped and no longer borrows the heap node.
                            heap.decrease_borrow_count(reference);
                        }
                    }
                }
            }
        }
        Ok(node_substates)
    }

    pub fn move_module<'f, S: SubstateStore, E, F: FnMut(StoreAccess) -> Result<(), E>>(
        &mut self,
        substate_io: &'f mut SubstateIO<S>,
        src_node_id: &NodeId,
        src_partition_number: PartitionNumber,
        dest_node_id: &NodeId,
        dest_partition_number: PartitionNumber,
        mut on_store_access: F,
    ) -> Result<(), CallbackError<MoveModuleError, E>> {
        // Check ownership (and visibility)
        if self.owned_root_nodes.get(src_node_id) != Some(&0) {
            return Err(CallbackError::Error(MoveModuleError::NodeNotAvailable(
                src_node_id.clone(),
            )));
        }

        // Check visibility
        if !self.get_node_visibility(dest_node_id).is_visible() {
            return Err(CallbackError::Error(MoveModuleError::NodeNotAvailable(
                dest_node_id.clone(),
            )));
        }

        // Move
        let module = substate_io
            .heap
            .remove_module(src_node_id, src_partition_number)
            .map_err(|e| CallbackError::Error(MoveModuleError::HeapRemoveModuleErr(e)))?;
        let to_heap = substate_io.heap.contains_node(dest_node_id);
        for (substate_key, substate_value) in module {
            if to_heap {
                substate_io.heap.set_substate(
                    *dest_node_id,
                    dest_partition_number,
                    substate_key,
                    substate_value,
                );
            } else {
                // Recursively move nodes to store
                for own in substate_value.owned_nodes() {
                    substate_io
                        .move_node_to_store(own, &mut on_store_access)
                        .map_err(|e| e.map(|e| MoveModuleError::PersistNodeError(e)))?;
                }

                for reference in substate_value.references() {
                    if !reference.is_global() {
                        return Err(CallbackError::Error(
                            MoveModuleError::NonGlobalRefNotAllowed(reference.clone()),
                        ));
                    }
                }

                substate_io
                    .store
                    .set_substate(
                        *dest_node_id,
                        dest_partition_number,
                        substate_key,
                        substate_value,
                        &mut on_store_access,
                    )
                    .map_err(CallbackError::CallbackError)?
            }
        }

        Ok(())
    }

    pub fn add_global_reference(&mut self, address: GlobalAddress) {
        self.stable_references
            .insert(address.into_node_id(), StableReferenceType::Global);
    }

    pub fn add_direct_access_reference(&mut self, address: InternalAddress) {
        self.stable_references
            .insert(address.into_node_id(), StableReferenceType::DirectAccess);
    }

    //====================================================================================
    // Note that reference model isn't fully implemented for set/remove/scan/take APIs.
    // They're intended for internal use only and extra caution must be taken.
    //====================================================================================

    // Substate Virtualization does not apply to this call
    // Should this be prevented at this layer?
    pub fn set_substate<'f, S: SubstateStore, E, F: FnMut(StoreAccess) -> Result<(), E>>(
        &mut self,
        substate_io: &'f mut SubstateIO<S>,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        key: SubstateKey,
        value: IndexedScryptoValue,
        on_store_access: &mut F,
    ) -> Result<(), CallbackError<CallFrameSetSubstateError, E>> {
        // Check node visibility
        if !self.get_node_visibility(node_id).is_visible() {
            return Err(CallbackError::Error(
                CallFrameSetSubstateError::NodeNotVisible(node_id.clone()),
            ));
        }

        substate_io.set_substate(node_id, partition_num, key, value, on_store_access)?;

        Ok(())
    }

    pub fn remove_substate<'f, S: SubstateStore, E, F: FnMut(StoreAccess) -> Result<(), E>>(
        &mut self,
        substate_io: &'f mut SubstateIO<S>,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        key: &SubstateKey,
        on_store_access: F,
    ) -> Result<Option<IndexedScryptoValue>, CallbackError<CallFrameRemoveSubstateError, E>> {
        // Check node visibility
        if !self.get_node_visibility(node_id).is_visible() {
            return Err(CallbackError::Error(
                CallFrameRemoveSubstateError::NodeNotVisible(node_id.clone()),
            ));
        }

        let removed = substate_io.remove_substate(node_id, partition_num, key, on_store_access)?;

        Ok(removed)
    }

    pub fn scan_keys<
        'f,
        K: SubstateKeyContent,
        S: SubstateStore,
        E,
        F: FnMut(StoreAccess) -> Result<(), E>,
    >(
        &mut self,
        substate_io: &'f mut SubstateIO<S>,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        count: u32,
        on_store_access: F,
    ) -> Result<Vec<SubstateKey>, CallbackError<CallFrameScanKeysError, E>> {
        // Check node visibility
        if !self.get_node_visibility(node_id).is_visible() {
            return Err(CallbackError::Error(
                CallFrameScanKeysError::NodeNotVisible(node_id.clone()),
            ));
        }

        let keys = substate_io.scan_keys::<K, E, F>(node_id, partition_num, count, on_store_access)?;

        Ok(keys)
    }

    pub fn drain_substates<
        'f,
        K: SubstateKeyContent,
        S: SubstateStore,
        E,
        F: FnMut(StoreAccess) -> Result<(), E>,
    >(
        &mut self,
        substate_io: &'f mut SubstateIO<S>,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        count: u32,
        on_store_access: F,
    ) -> Result<
        Vec<(SubstateKey, IndexedScryptoValue)>,
        CallbackError<CallFrameDrainSubstatesError, E>,
    > {
        // Check node visibility
        if !self.get_node_visibility(node_id).is_visible() {
            return Err(CallbackError::Error(
                CallFrameDrainSubstatesError::NodeNotVisible(node_id.clone()),
            ));
        }

        let substates = substate_io.drain_substates::<K, E, F>(node_id, partition_num, count, on_store_access)?;

        for (_key, substate) in &substates {
            for reference in substate.references() {
                if reference.is_global() {
                    self.stable_references
                        .insert(reference.clone(), StableReferenceType::Global);
                } else {
                    return Err(CallbackError::Error(
                        CallFrameDrainSubstatesError::OwnedNodeNotSupported(reference.clone()),
                    ));
                }
            }
        }

        Ok(substates)
    }

    // Substate Virtualization does not apply to this call
    // Should this be prevented at this layer?
    pub fn scan_sorted<'f, S: SubstateStore, E, F: FnMut(StoreAccess) -> Result<(), E>>(
        &mut self,
        substate_io: &'f mut SubstateIO<S>,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        count: u32,
        on_store_access: F,
    ) -> Result<Vec<IndexedScryptoValue>, CallbackError<CallFrameScanSortedSubstatesError, E>> {
        // Check node visibility
        if !self.get_node_visibility(node_id).is_visible() {
            return Err(CallbackError::Error(
                CallFrameScanSortedSubstatesError::NodeNotVisible(node_id.clone()),
            ));
        }

        let substates = substate_io.scan_sorted(
            node_id,
            partition_num,
            count,
            on_store_access,
        )?;

        for substate in &substates {
            for reference in substate.references() {
                if reference.is_global() {
                    self.stable_references
                        .insert(reference.clone(), StableReferenceType::Global);
                } else {
                    // FIXME: check if non-global reference is needed
                }
            }
        }

        Ok(substates)
    }

    pub fn drop_all_locks<S: SubstateStore, E, F: FnMut(StoreAccess) -> Result<(), E>>(
        &mut self,
        substate_io: &mut SubstateIO<S>,
        on_store_access: &mut F,
    ) -> Result<(), CallbackError<CloseSubstateError, E>> {
        let lock_handles: Vec<LockHandle> = self.locks.keys().cloned().collect();

        for lock_handle in lock_handles {
            self.close_substate(substate_io, lock_handle, on_store_access)?;
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

    pub fn get_node_visibility(&self, node_id: &NodeId) -> NodeVisibility {
        let mut visibilities = BTreeSet::<Visibility>::new();

        // Stable references
        if let Some(reference_type) = self.stable_references.get(node_id) {
            visibilities.insert(Visibility::StableReference(reference_type.clone()));
        }
        if ALWAYS_VISIBLE_GLOBAL_NODES.contains(node_id) {
            visibilities.insert(Visibility::StableReference(StableReferenceType::Global));
        }

        // Frame owned nodes
        if self.owned_root_nodes.contains_key(node_id) {
            visibilities.insert(Visibility::FrameOwned);
        }

        // Actor
        if let Some(actor_node_id) = self.actor.node_id() {
            if actor_node_id == *node_id {
                visibilities.insert(Visibility::Actor);
            }
        }

        // Borrowed from substate loading
        // TODO: we may want to further split it based on the borrow origin (actor & frame owned nodes)
        if self.transient_references.contains_key(node_id) {
            visibilities.insert(Visibility::Borrowed);
        }

        NodeVisibility(visibilities)
    }
}
