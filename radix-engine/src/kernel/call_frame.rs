use crate::kernel::actor::MethodType;
use crate::system::node_modules::type_info::TypeInfoSubstate;
use crate::track::interface::{CallbackError, NodeSubstates, StoreAccess, SubstateStore, TrackedSubstateInfo, TrackGetSubstateError};
use crate::types::*;
use radix_engine_interface::api::field_api::LockFlags;
use radix_engine_interface::blueprints::resource::{
    FUNGIBLE_BUCKET_BLUEPRINT, FUNGIBLE_PROOF_BLUEPRINT, NON_FUNGIBLE_BUCKET_BLUEPRINT,
    NON_FUNGIBLE_PROOF_BLUEPRINT,
};
use radix_engine_interface::types::{LockHandle, NodeId, SubstateKey};
use radix_engine_store_interface::db_key_mapper::SubstateKeyContent;
use crate::kernel::substate_locks::SubstateLocks;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubstateStoreHandle {
    Heap(u32),
    Store(u32),
}

/// A lock on a substate controlled by a call frame
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenedSubstate<L> {
    pub non_global_references: IndexSet<NodeId>,
    pub owned_nodes: IndexSet<NodeId>,
    pub substate_store_handle: SubstateStoreHandle,
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
    LockUnmodifiedBaseOnNewSubstate(NodeId, PartitionNumber, SubstateKey),
    LockUnmodifiedBaseOnOnUpdatedSubstate(NodeId, PartitionNumber, SubstateKey)
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

    // TODO: Remove
    fn get_heap_type_info(node_id: &NodeId, heap: &mut Heap) -> Option<TypeInfoSubstate> {
        if let Some(substate) = heap.get_substate(
            node_id,
            TYPE_INFO_FIELD_PARTITION,
            &TypeInfoField::TypeInfo.into(),
        ) {
            let type_info: TypeInfoSubstate = substate.as_typed().unwrap();
            Some(type_info)
        } else {
            None
        }
    }

    pub fn open_substate<S: SubstateStore, E, F: FnMut(StoreAccess) -> Result<(), E>>(
        &mut self,
        heap: &mut Heap,
        store: &mut S,
        substate_locks: &mut SubstateLocks,
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

        let handle = match substate_locks.lock(node_id, partition_num, substate_key, flags) {
            Some(handle) => handle,
            None => {
                return Err(CallbackError::Error(OpenSubstateError::SubstateLocked(*node_id, partition_num, substate_key.clone())));
            }
        };

        // Lock and read the substate
        let (substate_value, substate_store_handle) = if heap.contains_node(node_id) {
            // FIXME: we will have to move locking logic to heap because references moves between frames.
            let value = heap
                .get_substate_or_default(node_id, partition_num, substate_key, flags, || {
                    default.map(|f| f())
                })
                .map_err(|e| CallbackError::Error(OpenSubstateError::HeapError(e)))?;

            (value, SubstateStoreHandle::Heap(handle))
        } else {

            // Check substate state
            if flags.contains(LockFlags::UNMODIFIED_BASE) {
                match store.get_substate_info(node_id, partition_num, substate_key) {
                    TrackedSubstateInfo::New => {
                        return Err(CallbackError::Error(
                            OpenSubstateError::LockUnmodifiedBaseOnNewSubstate(
                                node_id.clone(),
                                partition_num,
                                substate_key.clone(),
                            ),
                        ));
                    }
                    TrackedSubstateInfo::Updated => {
                        return Err(CallbackError::Error(
                            OpenSubstateError::LockUnmodifiedBaseOnOnUpdatedSubstate(
                                node_id.clone(),
                                partition_num,
                                substate_key.clone(),
                            ),
                        ));
                    }
                    TrackedSubstateInfo::Unmodified => {
                        // Okay
                    }
                }
            }

            let value = store
                .get_substate_or_default(
                    node_id,
                    partition_num,
                    substate_key,
                    on_store_access,
                    || default.map(|f| f()),
                )
                .map_err(|x| {
                    substate_locks.unlock(handle);
                    x.map(|e| OpenSubstateError::TrackError(Box::new(e)))
                })?;


            (value, SubstateStoreHandle::Store(handle))
        };

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
                substate_store_handle,
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

    pub fn close_substate<S: SubstateStore, E, F: FnMut(StoreAccess) -> Result<(), E>>(
        &mut self,
        heap: &mut Heap,
        store: &mut S,
        substate_locks: &mut SubstateLocks,
        lock_handle: LockHandle,
        on_store_access: &mut F,
    ) -> Result<(), CallbackError<CloseSubstateError, E>> {
        let substate_lock = self
            .locks
            .remove(&lock_handle)
            .ok_or_else(|| CallbackError::Error(CloseSubstateError::LockNotFound(lock_handle)))?;

        let handle = match substate_lock.substate_store_handle {
            SubstateStoreHandle::Store(handle)
            | SubstateStoreHandle::Heap(handle) => handle,
        };
        let (node_id, partion_num, substate_key, flags) = substate_locks.unlock(handle);

        if flags.contains(LockFlags::MUTABLE) {
            let substate_is_in_store = matches!(
                substate_lock.substate_store_handle,
                SubstateStoreHandle::Store(..)
            );
            let substate = if substate_is_in_store {
                store.get_substate::<(), _>(
                    &node_id,
                    partion_num,
                    &substate_key,
                    &mut |store_access| panic!("Getting substate on handled substate should not incur a store access."),
                ).unwrap().clone()
            } else {
                heap.get_substate(&node_id, partion_num, &substate_key).unwrap().clone()
            };

            //==============
            // Process owns
            //==============
            let mut new_owned_nodes: IndexSet<NodeId> = index_set_new();
            for own in substate.owned_nodes() {
                if !new_owned_nodes.insert(own.clone()) {
                    return Err(CallbackError::Error(
                        CloseSubstateError::ContainsDuplicatedOwns,
                    ));
                }
            }
            for own in &new_owned_nodes {
                if !substate_lock.owned_nodes.contains(own) {
                    // Node no longer owned by frame
                    self.take_node_internal(own)
                        .map_err(|e| CallbackError::Error(CloseSubstateError::TakeNodeError(e)))?;

                    // Move the node to store, if its owner is already in store
                    if substate_is_in_store {
                        Self::move_node_to_store(heap, store, own, on_store_access)
                            .map_err(|e| e.map(CloseSubstateError::PersistNodeError))?;
                    }
                }
            }
            for own in &substate_lock.owned_nodes {
                if !new_owned_nodes.contains(own) {
                    // Node detached
                    if substate_is_in_store {
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
            for own in substate.references() {
                // Deduplicate
                new_references.insert(own.clone());
            }
            for reference in &new_references {
                if !substate_lock.non_global_references.contains(reference) {
                    // handle added references

                    if !self
                        .get_node_visibility(reference)
                        .can_be_referenced_in_substate()
                    {
                        return Err(CallbackError::Error(CloseSubstateError::RefNotFound(
                            reference.clone(),
                        )));
                    }

                    if substate_is_in_store && !reference.is_global() {
                        return Err(CallbackError::Error(
                            CloseSubstateError::NonGlobalRefNotAllowed(*reference),
                        ));
                    }

                    if heap.contains_node(reference) {
                        heap.increase_borrow_count(reference);
                    } else {
                        // No op
                    }
                }
            }
            for reference in &substate_lock.non_global_references {
                if !new_references.contains(reference) {
                    // handle removed references

                    if heap.contains_node(reference) {
                        heap.decrease_borrow_count(reference);
                    }
                }
            }
        }

        // TODO: Cleanup
        if flags.contains(LockFlags::FORCE_WRITE) {
            store.force_write(&node_id, &partion_num, &substate_key);
        }

        // Shrink transient reference set
        for reference in substate_lock.non_global_references {
            let cnt = self.transient_references.remove(&reference).unwrap_or(0);
            if cnt > 1 {
                self.transient_references.insert(reference, cnt - 1);
            }
        }
        for own in substate_lock.owned_nodes {
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

    pub fn read_substate<'f, S: SubstateStore>(
        &mut self,
        heap: &'f mut Heap,
        store: &'f mut S,
        substate_locks: &mut SubstateLocks,
        lock_handle: LockHandle,
    ) -> Result<&'f IndexedScryptoValue, ReadSubstateError> {
        let OpenedSubstate {
            substate_store_handle,
            ..
        } = self
            .locks
            .get(&lock_handle)
            .ok_or(ReadSubstateError::LockNotFound(lock_handle))?;

        let handle = match substate_store_handle {
            SubstateStoreHandle::Store(handle)
            | SubstateStoreHandle::Heap(handle) => handle,
        };
        let (node_id, partition_num, substate_key, _) = substate_locks.get(*handle);

        let substate = match substate_store_handle {
            SubstateStoreHandle::Heap(handle) => heap.get_substate(node_id, *partition_num, substate_key).unwrap(),
            SubstateStoreHandle::Store(handle) => store.get_substate::<(), _>(
                node_id,
                *partition_num,
                substate_key,
                &mut |store_access| panic!("Getting substate on handled substate should not incur a store access."),
            ).unwrap(),
        };

        Ok(substate)
    }

    pub fn write_substate<'f, S: SubstateStore>(
        &mut self,
        heap: &'f mut Heap,
        store: &'f mut S,
        substate_locks: &'f mut SubstateLocks,
        lock_handle: LockHandle,
        substate: IndexedScryptoValue,
    ) -> Result<(), WriteSubstateError> {
        let OpenedSubstate {
            substate_store_handle,
            ..
        } = self
            .locks
            .get(&lock_handle)
            .ok_or(WriteSubstateError::LockNotFound(lock_handle))?;



        let handle = match substate_store_handle {
            SubstateStoreHandle::Store(handle)
            | SubstateStoreHandle::Heap(handle) => handle,
        };
        let (node_id, partition_num, substate_key, flags) = substate_locks.get(*handle);
        if !flags.contains(LockFlags::MUTABLE) {
            return Err(WriteSubstateError::NoWritePermission);
        }

        match substate_store_handle {
            SubstateStoreHandle::Heap(..) => {
                heap.set_substate(node_id.clone(), partition_num.clone(), substate_key.clone(), substate);
            },
            SubstateStoreHandle::Store(..) => {
                store.set_substate(
                    node_id.clone(),
                    partition_num.clone(),
                    substate_key.clone(),
                    substate,
                    &mut |_| Err(()),
                ).expect("Setting substate on handled substate should not incur a store access.");
            }
        }

        Ok(())
    }

    pub fn create_node<'f, S: SubstateStore, E, F: FnMut(StoreAccess) -> Result<(), E>>(
        &mut self,
        node_id: NodeId,
        node_substates: NodeSubstates,
        on_store_access: &mut F,
        heap: &mut Heap,
        store: &'f mut S,
        push_to_store: bool,
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
                        Self::move_node_to_store(heap, store, own, on_store_access)
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

                    if heap.contains_node(reference) {
                        heap.increase_borrow_count(reference);
                    } else {
                        // No op
                    }
                }
            }
        }

        if push_to_store {
            self.stable_references
                .insert(node_id, StableReferenceType::Global);
            store
                .create_node(node_id, node_substates, on_store_access)
                .map_err(CallbackError::CallbackError)?;
        } else {
            heap.create_node(node_id, node_substates);
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
        src_node_id: &NodeId,
        src_partition_number: PartitionNumber,
        dest_node_id: &NodeId,
        dest_partition_number: PartitionNumber,
        mut on_store_access: F,
        heap: &'f mut Heap,
        store: &'f mut S,
    ) -> Result<(), CallbackError<MoveModuleError, E>> {
        // Check ownership (and visibility)
        if self.owned_root_nodes.get(src_node_id) != Some(&0) {
            return Err(CallbackError::Error(MoveModuleError::NodeNotAvailable(
                src_node_id.clone(),
            )));
        }

        // Check visibility
        if !self
            .get_node_visibility(dest_node_id)
            .is_visible()
        {
            return Err(CallbackError::Error(MoveModuleError::NodeNotAvailable(
                dest_node_id.clone(),
            )));
        }

        // Move
        let module = heap
            .remove_module(src_node_id, src_partition_number)
            .map_err(|e| CallbackError::Error(MoveModuleError::HeapRemoveModuleErr(e)))?;
        let to_heap = heap.contains_node(dest_node_id);
        for (substate_key, substate_value) in module {
            if to_heap {
                heap.set_substate(
                    *dest_node_id,
                    dest_partition_number,
                    substate_key,
                    substate_value,
                );
            } else {
                // Recursively move nodes to store
                for own in substate_value.owned_nodes() {
                    Self::move_node_to_store(heap, store, own, &mut on_store_access)
                        .map_err(|e| e.map(|e| MoveModuleError::PersistNodeError(e)))?;
                }

                for reference in substate_value.references() {
                    if !reference.is_global() {
                        return Err(CallbackError::Error(
                            MoveModuleError::NonGlobalRefNotAllowed(reference.clone()),
                        ));
                    }
                }

                store
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
        node_id: &NodeId,
        partition_num: PartitionNumber,
        key: SubstateKey,
        value: IndexedScryptoValue,
        on_store_access: &mut F,
        heap: &'f mut Heap,
        store: &'f mut S,
    ) -> Result<(), CallbackError<CallFrameSetSubstateError, E>> {
        // Check node visibility
        if !self.get_node_visibility(node_id).is_visible() {
            return Err(CallbackError::Error(
                CallFrameSetSubstateError::NodeNotVisible(node_id.clone()),
            ));
        }

        // FIXME Add lock check

        if heap.contains_node(node_id) {
            heap.set_substate(*node_id, partition_num, key, value);
        } else {
            store
                .set_substate(*node_id, partition_num, key, value, on_store_access)
                .map_err(CallbackError::CallbackError)?
        };

        Ok(())
    }

    pub fn remove_substate<'f, S: SubstateStore, E, F: FnMut(StoreAccess) -> Result<(), E>>(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        key: &SubstateKey,
        on_store_access: F,
        heap: &'f mut Heap,
        store: &'f mut S,
        substate_locks: &'f mut SubstateLocks,
    ) -> Result<Option<IndexedScryptoValue>, CallbackError<CallFrameRemoveSubstateError, E>> {
        // Check node visibility
        if !self.get_node_visibility(node_id).is_visible() {
            return Err(CallbackError::Error(
                CallFrameRemoveSubstateError::NodeNotVisible(node_id.clone()),
            ));
        }

        if substate_locks.is_locked(node_id, partition_num, key) {
            return Err(CallbackError::Error(
                CallFrameRemoveSubstateError::SubstateLocked(node_id.clone(), partition_num, key.clone())
            ));
        }

        let removed = if heap.contains_node(node_id) {
            heap.remove_substate(node_id, partition_num, key)
        } else {
            store
                .remove_substate(node_id, partition_num, key, on_store_access)
                .map_err(CallbackError::CallbackError)?
        };

        Ok(removed)
    }

    pub fn scan_keys<'f, K: SubstateKeyContent, S: SubstateStore, E, F: FnMut(StoreAccess) -> Result<(), E>>(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        count: u32,
        on_store_access: F,
        heap: &'f mut Heap,
        store: &'f mut S,
    ) -> Result<Vec<SubstateKey>, CallbackError<CallFrameScanKeysError, E>> {
        // Check node visibility
        if !self.get_node_visibility(node_id).is_visible() {
            return Err(CallbackError::Error(
                CallFrameScanKeysError::NodeNotVisible(node_id.clone()),
            ));
        }

        let keys = if heap.contains_node(node_id) {
            heap.scan_keys(node_id, partition_num, count)
        } else {
            store
                .scan_keys::<K, E, F>(node_id, partition_num, count, on_store_access)
                .map_err(|e| CallbackError::CallbackError(e))?
        };

        Ok(keys)
    }

    pub fn drain_substates<'f, K: SubstateKeyContent, S: SubstateStore, E, F: FnMut(StoreAccess) -> Result<(), E>>(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        count: u32,
        on_store_access: F,
        heap: &'f mut Heap,
        store: &'f mut S,
    ) -> Result<Vec<(SubstateKey, IndexedScryptoValue)>, CallbackError<CallFrameDrainSubstatesError, E>> {
        // Check node visibility
        if !self.get_node_visibility(node_id).is_visible() {
            return Err(CallbackError::Error(
                CallFrameDrainSubstatesError::NodeNotVisible(node_id.clone()),
            ));
        }

        let substates = if heap.contains_node(node_id) {
            heap.drain_substates(node_id, partition_num, count)
        } else {
            store
                .drain_substates::<K, E, F>(node_id, partition_num, count, on_store_access)
                .map_err(|e| CallbackError::CallbackError(e))?
        };

        for (_key, substate) in &substates {
            for reference in substate.references() {
                if reference.is_global() {
                    self.stable_references
                        .insert(reference.clone(), StableReferenceType::Global);
                } else {
                    return Err(CallbackError::Error(CallFrameDrainSubstatesError::OwnedNodeNotSupported(
                        reference.clone(),
                    )));
                }
            }
        }

        Ok(substates)
    }

    // Substate Virtualization does not apply to this call
    // Should this be prevented at this layer?
    pub fn scan_sorted<'f, S: SubstateStore, E, F: FnMut(StoreAccess) -> Result<(), E>>(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        count: u32,
        on_store_access: F,
        heap: &'f mut Heap,
        store: &'f mut S,
    ) -> Result<Vec<IndexedScryptoValue>, CallbackError<CallFrameScanSortedSubstatesError, E>> {
        // Check node visibility
        if !self.get_node_visibility(node_id).is_visible() {
            return Err(CallbackError::Error(
                CallFrameScanSortedSubstatesError::NodeNotVisible(node_id.clone()),
            ));
        }

        let substates = if heap.contains_node(node_id) {
            // This should never be triggered because sorted index store is
            // used by consensus manager only.
            panic!("Unexpected code path")
        } else {
            store
                .scan_sorted_substates(node_id, partition_num, count, on_store_access)
                .map_err(|e| CallbackError::CallbackError(e))?
        };

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
        on_store_access: &mut F,
        heap: &mut Heap,
        store: &mut S,
        substate_locks: &mut SubstateLocks,
    ) -> Result<(), CallbackError<CloseSubstateError, E>> {
        let lock_handles: Vec<LockHandle> = self.locks.keys().cloned().collect();

        for lock_handle in lock_handles {
            self.close_substate(heap, store, substate_locks, lock_handle, on_store_access)?;
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

    pub fn move_node_to_store<S: SubstateStore, E, F: FnMut(StoreAccess) -> Result<(), E>>(
        heap: &mut Heap,
        store: &mut S,
        node_id: &NodeId,
        on_store_access: &mut F,
    ) -> Result<(), CallbackError<PersistNodeError, E>> {
        // FIXME: Use unified approach to node configuration
        let can_be_stored = if node_id.is_global() {
            true
        } else {
            let type_info = Self::get_heap_type_info(node_id, heap);

            if let Some(type_info) = type_info {
                match type_info {
                    TypeInfoSubstate::Object(ObjectInfo {
                        blueprint_info: BlueprintInfo { blueprint_id, .. },
                        ..
                    }) if blueprint_id.package_address == RESOURCE_PACKAGE
                        && (blueprint_id.blueprint_name == FUNGIBLE_BUCKET_BLUEPRINT
                            || blueprint_id.blueprint_name == NON_FUNGIBLE_BUCKET_BLUEPRINT
                            || blueprint_id.blueprint_name == FUNGIBLE_PROOF_BLUEPRINT
                            || blueprint_id.blueprint_name == NON_FUNGIBLE_PROOF_BLUEPRINT) =>
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
            return Err(CallbackError::Error(PersistNodeError::CantBeStored(
                node_id.clone(),
            )));
        }

        let node_substates = match heap.remove_node(node_id) {
            Ok(substates) => substates,
            Err(HeapRemoveNodeError::NodeNotFound(node_id)) => {
                panic!("Frame owned node {:?} not found in heap", node_id)
            }
            Err(HeapRemoveNodeError::NodeBorrowed(node_id, count)) => {
                return Err(CallbackError::Error(PersistNodeError::NodeBorrowed(
                    node_id, count,
                )));
            }
        };
        for (_partition_number, module_substates) in &node_substates {
            for (_substate_key, substate_value) in module_substates {
                for reference in substate_value.references() {
                    if !reference.is_global() {
                        return Err(CallbackError::Error(
                            PersistNodeError::NonGlobalRefNotAllowed(*reference),
                        ));
                    }
                }

                for node in substate_value.owned_nodes() {
                    Self::move_node_to_store(heap, store, node, on_store_access)?;
                }
            }
        }

        store
            .create_node(node_id.clone(), node_substates, on_store_access)
            .map_err(CallbackError::CallbackError)?;

        Ok(())
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
