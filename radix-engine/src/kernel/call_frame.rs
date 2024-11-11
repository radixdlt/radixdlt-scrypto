use crate::internal_prelude::*;
use crate::kernel::kernel_api::DroppedNode;
use crate::kernel::kernel_callback_api::CallFrameReferences;
use crate::kernel::substate_io::{
    IOAccessHandler, SubstateDevice, SubstateIO, SubstateReadHandler,
};
use crate::track::interface::{CallbackError, CommitableSubstateStore, IOAccess, NodeSubstates};
use radix_engine_interface::api::field_api::LockFlags;
use radix_engine_interface::types::{NodeId, SubstateHandle, SubstateKey};
use radix_substate_store_interface::db_key_mapper::SubstateKeyContent;

use super::heap::{Heap, HeapRemovePartitionError};

/// A message used for communication between call frames.
///
/// Note that it's just an intent, not checked/allowed by kernel yet.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct CallFrameMessage {
    /// Nodes to be moved from src to dest
    pub move_nodes: Vec<NodeId>,

    /// Copy of a global ref from src to dest
    pub copy_global_references: Vec<NodeId>,

    /// Copy of a direct access ref from src to dest
    pub copy_direct_access_references: Vec<NodeId>,

    /// Create a "stable" transient in dest from src. The src node may
    /// have global or borrowed visibility
    /// TODO: Cleanup abstraction (perhaps by adding another type of visibility)
    pub copy_stable_transient_references: Vec<NodeId>,
}

impl CallFrameMessage {
    pub fn from_input<C: CallFrameReferences>(value: &IndexedScryptoValue, references: &C) -> Self {
        let mut copy_global_references = Vec::new();
        let mut copy_direct_access_references = Vec::new();

        for arg_ref in value.references().clone() {
            if arg_ref.is_global() {
                copy_global_references.push(arg_ref);
            } else {
                copy_direct_access_references.push(arg_ref);
            }
        }

        copy_global_references.extend(references.global_references());
        copy_direct_access_references.extend(references.direct_access_references());

        Self {
            move_nodes: value.owned_nodes().clone(),
            copy_global_references,
            copy_direct_access_references,
            copy_stable_transient_references: references.stable_transient_references(),
        }
    }

    pub fn from_output(value: &IndexedScryptoValue) -> Self {
        let mut copy_global_references = Vec::new();
        let mut copy_direct_access_references = Vec::new();

        for arg_ref in value.references().clone() {
            if arg_ref.is_global() {
                copy_global_references.push(arg_ref);
            } else {
                copy_direct_access_references.push(arg_ref);
            }
        }

        Self {
            move_nodes: value.owned_nodes().clone(),
            copy_global_references,
            copy_direct_access_references,
            copy_stable_transient_references: vec![],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenedSubstate<L> {
    pub references: IndexSet<NodeId>,
    pub owned_nodes: IndexSet<NodeId>,
    pub ref_origin: ReferenceOrigin,
    pub global_substate_handle: u32,
    pub device: SubstateDevice,
    pub data: L,
}

impl<L> OpenedSubstate<L> {
    fn diff_on_close(&self) -> SubstateDiff {
        SubstateDiff {
            added_owns: index_set_new(),
            added_refs: index_set_new(),
            removed_owns: self.owned_nodes.clone(),
            removed_refs: self.references.clone(),
        }
    }

    fn diff(&self, updated_value: &IndexedScryptoValue) -> Result<SubstateDiff, SubstateDiffError> {
        // Process owned nodes
        let (added_owned_nodes, removed_owned_nodes) = {
            let mut added_owned_nodes: IndexSet<NodeId> = index_set_new();
            let mut new_owned_nodes: IndexSet<NodeId> = index_set_new();
            for own in updated_value.owned_nodes() {
                if !new_owned_nodes.insert(own.clone()) {
                    return Err(SubstateDiffError::ContainsDuplicateOwns);
                }

                if !self.owned_nodes.contains(own) {
                    added_owned_nodes.insert(*own);
                }
            }

            let mut removed_owned_nodes: IndexSet<NodeId> = index_set_new();
            for own in &self.owned_nodes {
                if !new_owned_nodes.contains(own) {
                    removed_owned_nodes.insert(*own);
                }
            }

            (added_owned_nodes, removed_owned_nodes)
        };

        //====================
        // Process references
        //====================
        let (added_references, removed_references) = {
            // De-duplicate
            let updated_references: IndexSet<NodeId> =
                updated_value.references().clone().into_iter().collect();

            let mut added_references: IndexSet<NodeId> = index_set_new();
            for reference in &updated_references {
                let reference_is_new = !self.references.contains(reference);

                if reference_is_new {
                    added_references.insert(reference.clone());
                }
            }

            let mut removed_references: IndexSet<NodeId> = index_set_new();
            for old_ref in &self.references {
                if !updated_references.contains(old_ref) {
                    removed_references.insert(*old_ref);
                }
            }

            (added_references, removed_references)
        };

        Ok(SubstateDiff {
            added_owns: added_owned_nodes,
            removed_owns: removed_owned_nodes,
            added_refs: added_references,
            removed_refs: removed_references,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SubstateDiff {
    added_owns: IndexSet<NodeId>,
    removed_owns: IndexSet<NodeId>,
    added_refs: IndexSet<NodeId>,
    removed_refs: IndexSet<NodeId>,
}

impl SubstateDiff {
    pub fn from_new_substate(
        substate_value: &IndexedScryptoValue,
    ) -> Result<Self, SubstateDiffError> {
        let mut added_owns = index_set_new();
        let mut added_refs = index_set_new();

        for own in substate_value.owned_nodes() {
            if !added_owns.insert(own.clone()) {
                return Err(SubstateDiffError::ContainsDuplicateOwns);
            }
        }

        for reference in substate_value.references() {
            added_refs.insert(reference.clone());
        }

        Ok(Self {
            added_owns,
            added_refs,
            removed_owns: index_set_new(),
            removed_refs: index_set_new(),
        })
    }

    pub fn from_drop_substate(substate_value: &IndexedScryptoValue) -> Self {
        let mut removed_owns = index_set_new();
        let mut removed_refs = index_set_new();

        for own in substate_value.owned_nodes() {
            if !removed_owns.insert(own.clone()) {
                panic!("Should never have been able to create duplicate owns");
            }
        }

        for reference in substate_value.references() {
            removed_refs.insert(reference.clone());
        }

        Self {
            added_owns: index_set_new(),
            added_refs: index_set_new(),
            removed_owns,
            removed_refs,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum StableReferenceType {
    Global,
    DirectAccess,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TransientReference {
    ref_count: usize,
    ref_origin: ReferenceOrigin,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ReferenceOrigin {
    FrameOwned,
    Global(GlobalAddress),
    DirectlyAccessed,
    SubstateNonGlobalReference(SubstateDevice),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Visibility {
    StableReference(StableReferenceType),
    FrameOwned,
    Borrowed(ReferenceOrigin),
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

    pub fn is_global(&self) -> bool {
        for v in &self.0 {
            if let Visibility::StableReference(StableReferenceType::Global) = v {
                return true;
            }
        }
        return false;
    }

    // TODO: Should we return Vec<ReferenceOrigin> and not supersede global with direct access reference
    pub fn reference_origin(&self, node_id: NodeId) -> Option<ReferenceOrigin> {
        let mut found_direct_access = false;
        for v in &self.0 {
            match v {
                Visibility::StableReference(StableReferenceType::Global) => {
                    return Some(ReferenceOrigin::Global(GlobalAddress::new_or_panic(
                        node_id.0,
                    )));
                }
                Visibility::StableReference(StableReferenceType::DirectAccess) => {
                    found_direct_access = true
                }
                Visibility::Borrowed(ref_origin) => return Some(ref_origin.clone()),
                Visibility::FrameOwned => {
                    return Some(ReferenceOrigin::FrameOwned);
                }
            }
        }

        if found_direct_access {
            return Some(ReferenceOrigin::DirectlyAccessed);
        }

        return None;
    }
}

/// Callback for IO access, from call frame
pub trait CallFrameIOAccessHandler<C, L, E> {
    fn on_io_access(
        &mut self,
        current_frame: &CallFrame<C, L>,
        heap: &Heap,
        io_access: IOAccess,
    ) -> Result<(), E>;
}

/// Callback for substate read, from call frame
pub trait CallFrameSubstateReadHandler<C, L> {
    type Error;

    fn on_read_substate(
        &mut self,
        current_frame: &CallFrame<C, L>,
        heap: &Heap,
        handle: SubstateHandle,
        value: &IndexedScryptoValue,
        device: SubstateDevice,
    ) -> Result<(), Self::Error>;
}

struct CallFrameToIOAccessAdapter<'g, C, L, E, H: CallFrameIOAccessHandler<C, L, E>> {
    handler: &'g mut H,
    call_frame: &'g mut CallFrame<C, L>,
    phantom: PhantomData<E>,
}

impl<'g, C, L, E, H: CallFrameIOAccessHandler<C, L, E>> IOAccessHandler<E>
    for CallFrameToIOAccessAdapter<'g, C, L, E, H>
{
    fn on_io_access(&mut self, heap: &Heap, io_access: IOAccess) -> Result<(), E> {
        self.handler.on_io_access(self.call_frame, heap, io_access)
    }
}

struct CallFrameToIOSubstateReadAdapter<'g, C, L, H: CallFrameSubstateReadHandler<C, L>> {
    handler: &'g mut H,
    call_frame: &'g CallFrame<C, L>,
    handle: SubstateHandle,
}

impl<'g, C, L, H: CallFrameSubstateReadHandler<C, L>> SubstateReadHandler
    for CallFrameToIOSubstateReadAdapter<'g, C, L, H>
{
    type Error = H::Error;

    fn on_read_substate(
        &mut self,
        heap: &Heap,
        value: &IndexedScryptoValue,
        location: SubstateDevice,
    ) -> Result<(), Self::Error> {
        self.handler
            .on_read_substate(self.call_frame, heap, self.handle, value, location)
    }
}

/// A call frame is the basic unit that forms a transaction call stack, which keeps track of the
/// owned objects and references by this function.
pub struct CallFrame<C, L> {
    /// The stack id.
    stack_id: usize,

    /// The frame id
    depth: usize,

    /// Call frame system layer data
    call_frame_data: C,

    /// Owned nodes which by definition must live on heap
    owned_root_nodes: IndexSet<NodeId>,

    /// References to non-GLOBAL nodes, obtained from substate loading, ref counted.
    /// These references may NOT be passed between call frames as arguments
    transient_references: NonIterMap<NodeId, TransientReference>,

    /// Stable references points to nodes in track, which can't moved/deleted.
    /// Current two types: `GLOBAL` (root, stored) and `DirectAccess`.
    /// These references MAY be passed between call frames
    stable_references: BTreeMap<NodeId, StableReferenceType>,

    next_handle: SubstateHandle,
    open_substates: IndexMap<SubstateHandle, OpenedSubstate<L>>,

    /// The set of nodes that are always globally visible.
    always_visible_global_nodes: &'static IndexSet<NodeId>,
}

/// Represents an error when creating a new frame.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum CreateFrameError {
    PassMessageError(PassMessageError),
}

/// Represents an error when passing message between frames.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum PassMessageError {
    TakeNodeError(TakeNodeError),
    GlobalRefNotFound(error_models::ReferencedNodeId),
    DirectRefNotFound(error_models::ReferencedNodeId),
    TransientRefNotFound(error_models::ReferencedNodeId),
}

/// Represents an error when creating a node.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum CreateNodeError {
    ProcessSubstateError(ProcessSubstateError),
    ProcessSubstateKeyError(ProcessSubstateKeyError),
    SubstateDiffError(SubstateDiffError),
}

/// Represents an error when dropping a node.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum DropNodeError {
    TakeNodeError(TakeNodeError),
    NodeBorrowed(error_models::ReferencedNodeId),
    SubstateBorrowed(error_models::ReferencedNodeId),
    ProcessSubstateError(ProcessSubstateError),
}

/// Represents an error when persisting a node into store.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum PersistNodeError {
    ContainsNonGlobalRef(error_models::ReferencedNodeId),
    NodeBorrowed(error_models::ReferencedNodeId),
    CannotPersistPinnedNode(error_models::OwnedNodeId),
}

/// Represents an error when taking a node from current frame.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum TakeNodeError {
    OwnNotFound(error_models::OwnedNodeId),
    SubstateBorrowed(error_models::ReferencedNodeId),
}

/// Represents an error when moving modules from one node to another.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum MovePartitionError {
    NodeNotAvailable(error_models::ReferencedNodeId),
    HeapRemovePartitionError(HeapRemovePartitionError),
    NonGlobalRefNotAllowed(error_models::ReferencedNodeId),
    PersistNodeError(PersistNodeError),
    SubstateBorrowed(error_models::ReferencedNodeId),
    MoveFromStoreNotPermitted,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum PinNodeError {
    NodeNotVisible(error_models::ReferencedNodeId),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum MarkTransientSubstateError {
    NodeNotVisible(error_models::ReferencedNodeId),
}

/// Represents an error when attempting to lock a substate.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum OpenSubstateError {
    NodeNotVisible(error_models::ReferencedNodeId),
    SubstateFault,
    InvalidDefaultValue,
    ProcessSubstateKeyError(ProcessSubstateKeyError),
    SubstateLocked(error_models::OwnedNodeId, PartitionNumber, SubstateKey),
    LockUnmodifiedBaseOnHeapNode,
    LockUnmodifiedBaseOnNewSubstate(error_models::OwnedNodeId, PartitionNumber, SubstateKey),
    LockUnmodifiedBaseOnOnUpdatedSubstate(error_models::OwnedNodeId, PartitionNumber, SubstateKey),
}

/// Represents an error when reading substates.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum ReadSubstateError {
    HandleNotFound(SubstateHandle),
}

/// Represents an error when writing substates.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum WriteSubstateError {
    HandleNotFound(SubstateHandle),
    ProcessSubstateError(ProcessSubstateError),
    NoWritePermission,
    SubstateDiffError(SubstateDiffError),
}

/// Represents an error when dropping a substate lock.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum CloseSubstateError {
    HandleNotFound(SubstateHandle),
    SubstateBorrowed(error_models::ReferencedNodeId),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum CallFrameSetSubstateError {
    NodeNotVisible(error_models::ReferencedNodeId),
    SubstateLocked(error_models::OwnedNodeId, PartitionNumber, SubstateKey),
    ProcessSubstateKeyError(ProcessSubstateKeyError),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum CallFrameRemoveSubstateError {
    NodeNotVisible(error_models::ReferencedNodeId),
    SubstateLocked(error_models::OwnedNodeId, PartitionNumber, SubstateKey),
    ProcessSubstateKeyError(ProcessSubstateKeyError),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum CallFrameScanKeysError {
    NodeNotVisible(error_models::ReferencedNodeId),
    ProcessSubstateKeyError(ProcessSubstateKeyError),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum CallFrameDrainSubstatesError {
    NodeNotVisible(error_models::ReferencedNodeId),
    NonGlobalRefNotSupported(error_models::ReferencedNodeId),
    ProcessSubstateKeyError(ProcessSubstateKeyError),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum CallFrameScanSortedSubstatesError {
    NodeNotVisible(error_models::ReferencedNodeId),
    OwnedNodeNotSupported(error_models::OwnedNodeId),
    ProcessSubstateKeyError(ProcessSubstateKeyError),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum ProcessSubstateKeyError {
    NodeNotVisible(error_models::ReferencedNodeId),
    DecodeError(DecodeError),
    OwnedNodeNotSupported,
    NonGlobalRefNotSupported,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum ProcessSubstateError {
    TakeNodeError(TakeNodeError),
    CantDropNodeInStore(error_models::ReferencedNodeId),
    RefNotFound(error_models::ReferencedNodeId),
    RefCantBeAddedToSubstate(error_models::ReferencedNodeId),
    NonGlobalRefNotAllowed(error_models::ReferencedNodeId),
    PersistNodeError(PersistNodeError),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum SubstateDiffError {
    ContainsDuplicateOwns,
}

#[derive(Debug)]
pub struct CallFrameInit<C> {
    pub data: C,
    pub global_addresses: IndexSet<GlobalAddress>,
    pub direct_accesses: IndexSet<InternalAddress>,
    pub always_visible_global_nodes: &'static IndexSet<NodeId>,
    pub stack_id: usize,
}

impl<C, L: Clone> CallFrame<C, L> {
    pub fn new_root(init: CallFrameInit<C>) -> Self {
        let mut call_frame = Self {
            stack_id: init.stack_id,
            depth: 0,
            call_frame_data: init.data,
            stable_references: Default::default(),
            transient_references: NonIterMap::new(),
            owned_root_nodes: index_set_new(),
            next_handle: 0u32,
            open_substates: index_map_new(),
            always_visible_global_nodes: init.always_visible_global_nodes,
        };

        for global_ref in init.global_addresses {
            call_frame.add_global_reference(global_ref);
        }
        for direct_access in init.direct_accesses {
            call_frame.add_direct_access_reference(direct_access);
        }

        call_frame
    }

    pub fn new_child_from_parent<S: CommitableSubstateStore>(
        substate_io: &SubstateIO<S>,
        parent: &mut CallFrame<C, L>,
        call_frame_data: C,
        message: CallFrameMessage,
    ) -> Result<Self, CreateFrameError> {
        let mut frame = Self {
            stack_id: parent.stack_id,
            depth: parent.depth + 1,
            call_frame_data,
            stable_references: Default::default(),
            transient_references: NonIterMap::new(),
            owned_root_nodes: index_set_new(),
            next_handle: 0u32,
            open_substates: index_map_new(),
            always_visible_global_nodes: parent.always_visible_global_nodes,
        };

        // Copy references and move nodes
        Self::pass_message(substate_io, parent, &mut frame, message)
            .map_err(CreateFrameError::PassMessageError)?;

        Ok(frame)
    }

    pub fn pass_message<S: CommitableSubstateStore>(
        substate_io: &SubstateIO<S>,
        from: &mut CallFrame<C, L>,
        to: &mut CallFrame<C, L>,
        message: CallFrameMessage,
    ) -> Result<(), PassMessageError> {
        for node_id in message.move_nodes {
            // Note that this has no impact on the `transient_references` because
            // we don't allow move of "locked nodes".
            from.take_node_internal(substate_io, &node_id)
                .map_err(PassMessageError::TakeNodeError)?;
            to.owned_root_nodes.insert(node_id);
        }

        // Only allow copy of `Global` and `DirectAccess` references
        for node_id in message.copy_global_references {
            if from.get_node_visibility(&node_id).is_global() {
                // Note that GLOBAL and DirectAccess references are mutually exclusive,
                // so okay to overwrite
                to.stable_references
                    .insert(node_id, StableReferenceType::Global);
            } else {
                return Err(PassMessageError::GlobalRefNotFound(node_id.into()));
            }
        }

        for node_id in message.copy_direct_access_references {
            if from.get_node_visibility(&node_id).can_be_invoked(true) {
                to.stable_references
                    .insert(node_id, StableReferenceType::DirectAccess);
            } else {
                return Err(PassMessageError::DirectRefNotFound(node_id.into()));
            }
        }

        for node_id in message.copy_stable_transient_references {
            if let Some(ref_origin) = from.get_node_visibility(&node_id).reference_origin(node_id) {
                to.transient_references
                    .entry(node_id.clone())
                    .or_insert(TransientReference {
                        ref_count: 0usize,
                        ref_origin,
                    })
                    .ref_count
                    .add_assign(1);

                if let ReferenceOrigin::Global(global_address) = ref_origin {
                    to.stable_references
                        .insert(global_address.into_node_id(), StableReferenceType::Global);
                }
            } else {
                return Err(PassMessageError::TransientRefNotFound(node_id.into()));
            }
        }

        Ok(())
    }

    pub fn stack_id(&self) -> usize {
        self.stack_id
    }

    pub fn depth(&self) -> usize {
        self.depth
    }

    pub fn data(&self) -> &C {
        &self.call_frame_data
    }

    pub fn data_mut(&mut self) -> &mut C {
        &mut self.call_frame_data
    }

    pub fn pin_node<'f, S: CommitableSubstateStore>(
        &mut self,
        substate_io: &mut SubstateIO<S>,
        node_id: NodeId,
    ) -> Result<(), PinNodeError> {
        // Get device
        let (_ref_origin, device) = self
            .get_node_ref(&node_id)
            .ok_or_else(|| PinNodeError::NodeNotVisible(node_id.into()))?;

        match device {
            SubstateDevice::Heap => {
                substate_io.pinned_to_heap.insert(node_id);
            }
            SubstateDevice::Store => {
                // Nodes in store are always pinned
            }
        }

        Ok(())
    }

    pub fn mark_substate_as_transient<'f, S: CommitableSubstateStore>(
        &mut self,
        substate_io: &mut SubstateIO<S>,
        node_id: NodeId,
        partition_num: PartitionNumber,
        substate_key: SubstateKey,
    ) -> Result<(), MarkTransientSubstateError> {
        // Get device
        let (_ref_origin, device) = self
            .get_node_ref(&node_id)
            .ok_or_else(|| MarkTransientSubstateError::NodeNotVisible(node_id.into()))?;

        match device {
            SubstateDevice::Heap => {
                substate_io.heap_transient_substates.mark_as_transient(
                    node_id,
                    partition_num,
                    substate_key,
                );
            }
            SubstateDevice::Store => {
                substate_io
                    .store
                    .mark_as_transient(node_id, partition_num, substate_key);
            }
        }

        Ok(())
    }

    pub fn create_node<'f, S: CommitableSubstateStore, E>(
        &mut self,
        substate_io: &mut SubstateIO<S>,
        handler: &mut impl CallFrameIOAccessHandler<C, L, E>,
        node_id: NodeId,
        node_substates: NodeSubstates,
    ) -> Result<(), CallbackError<CreateNodeError, E>> {
        // TODO: We need to protect transient blueprints from being globalized directly
        // into store. This isn't a problem for now since only native objects are allowed
        // to be transient.

        let destination_device = if node_id.is_global() {
            SubstateDevice::Store
        } else {
            SubstateDevice::Heap
        };

        for (_partition_number, module) in &node_substates {
            for (substate_key, substate_value) in module {
                self.process_input_substate_key(substate_key).map_err(|e| {
                    CallbackError::Error(CreateNodeError::ProcessSubstateKeyError(e))
                })?;
                let diff = SubstateDiff::from_new_substate(&substate_value)
                    .map_err(|e| CallbackError::Error(CreateNodeError::SubstateDiffError(e)))?;

                self.process_substate_diff(substate_io, handler, destination_device, &diff)
                    .map_err(|e| e.map(CreateNodeError::ProcessSubstateError))?;
            }
        }

        match destination_device {
            SubstateDevice::Store => {
                self.stable_references
                    .insert(node_id, StableReferenceType::Global);
            }
            SubstateDevice::Heap => {
                self.owned_root_nodes.insert(node_id);
            }
        }

        let mut adapter = CallFrameToIOAccessAdapter {
            call_frame: self,
            handler,
            phantom: PhantomData::default(),
        };

        substate_io.create_node(destination_device, node_id, node_substates, &mut adapter)?;

        Ok(())
    }

    /// Removes node from call frame and owned nodes will be possessed by this call frame.
    pub fn drop_node<E, S: CommitableSubstateStore>(
        &mut self,
        substate_io: &mut SubstateIO<S>,
        node_id: &NodeId,
        handler: &mut impl CallFrameIOAccessHandler<C, L, E>,
    ) -> Result<DroppedNode, CallbackError<DropNodeError, E>> {
        self.take_node_internal(substate_io, node_id)
            .map_err(|e| CallbackError::Error(DropNodeError::TakeNodeError(e)))?;

        let mut adapter = CallFrameToIOAccessAdapter {
            call_frame: self,
            handler,
            phantom: PhantomData::default(),
        };
        let substates = substate_io
            .drop_node(SubstateDevice::Heap, node_id, &mut adapter)
            .map_err(|e| match e {
                CallbackError::Error(e) => CallbackError::Error(e),
                CallbackError::CallbackError(e) => CallbackError::CallbackError(e),
            })?;
        for (_partition_number, module) in &substates {
            for (_substate_key, substate_value) in module {
                let diff = SubstateDiff::from_drop_substate(&substate_value);
                adapter
                    .call_frame
                    .process_substate_diff(
                        substate_io,
                        adapter.handler,
                        SubstateDevice::Heap,
                        &diff,
                    )
                    .map_err(|e| match e {
                        CallbackError::Error(e) => {
                            CallbackError::Error(DropNodeError::ProcessSubstateError(e))
                        }
                        CallbackError::CallbackError(e) => CallbackError::CallbackError(e),
                    })?;
            }
        }

        let pinned_to_heap = substate_io.pinned_to_heap.remove(node_id);

        Ok(DroppedNode {
            substates,
            pinned_to_heap,
        })
    }

    pub fn move_partition<'f, S: CommitableSubstateStore, E>(
        &mut self,
        substate_io: &'f mut SubstateIO<S>,
        handler: &mut impl CallFrameIOAccessHandler<C, L, E>,
        src_node_id: &NodeId,
        src_partition_number: PartitionNumber,
        dest_node_id: &NodeId,
        dest_partition_number: PartitionNumber,
    ) -> Result<(), CallbackError<MovePartitionError, E>> {
        // Check src visibility
        let (_ref_origin, src_device) = self.get_node_ref(src_node_id).ok_or_else(|| {
            CallbackError::Error(MovePartitionError::NodeNotAvailable(
                src_node_id.clone().into(),
            ))
        })?;

        // Check dest visibility
        let (_ref_origin, dest_device) = self.get_node_ref(dest_node_id).ok_or_else(|| {
            CallbackError::Error(MovePartitionError::NodeNotAvailable(
                dest_node_id.clone().into(),
            ))
        })?;

        let mut adapter = CallFrameToIOAccessAdapter {
            call_frame: self,
            handler,
            phantom: PhantomData::default(),
        };
        substate_io.move_partition(
            src_device,
            src_node_id,
            src_partition_number,
            dest_device,
            dest_node_id,
            dest_partition_number,
            &mut adapter,
        )?;

        Ok(())
    }

    fn process_input_substate_key(
        &self,
        substate_key: &SubstateKey,
    ) -> Result<(), ProcessSubstateKeyError> {
        match substate_key {
            SubstateKey::Sorted((_, map_key)) | SubstateKey::Map(map_key) => {
                let key_value = IndexedScryptoValue::from_slice(map_key)
                    .map_err(|e| ProcessSubstateKeyError::DecodeError(e))?;

                // Check owns
                if !key_value.owned_nodes().is_empty() {
                    return Err(ProcessSubstateKeyError::OwnedNodeNotSupported);
                }

                // Check references
                for reference in key_value.references() {
                    if !reference.is_global() {
                        return Err(ProcessSubstateKeyError::NonGlobalRefNotSupported);
                    }

                    if !self.get_node_visibility(reference).is_visible() {
                        return Err(ProcessSubstateKeyError::NodeNotVisible(
                            reference.clone().into(),
                        ));
                    }
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn process_output_substate_key(
        &mut self,
        substate_key: &SubstateKey,
    ) -> Result<(), ProcessSubstateKeyError> {
        match substate_key {
            SubstateKey::Sorted((_, map_key)) | SubstateKey::Map(map_key) => {
                let key = IndexedScryptoValue::from_slice(map_key).unwrap();

                // Check owns
                if !key.owned_nodes().is_empty() {
                    panic!("Unexpected owns in substate key")
                }

                // Check references
                for reference in key.references() {
                    if reference.is_global() {
                        self.stable_references
                            .insert(reference.clone(), StableReferenceType::Global);
                    } else {
                        panic!("Unexpected non-global refs in substate key")
                    }
                }
            }
            _ => {}
        }

        Ok(())
    }

    pub fn open_substate<S: CommitableSubstateStore, E, F: FnOnce() -> IndexedScryptoValue>(
        &mut self,
        substate_io: &mut SubstateIO<S>,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        substate_key: &SubstateKey,
        flags: LockFlags,
        default: Option<F>,
        data: L,
        handler: &mut impl CallFrameIOAccessHandler<C, L, E>,
    ) -> Result<(SubstateHandle, usize), CallbackError<OpenSubstateError, E>> {
        let (ref_origin, device) = self.get_node_ref(node_id).ok_or_else(|| {
            CallbackError::Error(OpenSubstateError::NodeNotVisible(node_id.clone().into()))
        })?;

        self.process_input_substate_key(substate_key)
            .map_err(|e| CallbackError::Error(OpenSubstateError::ProcessSubstateKeyError(e)))?;

        let mut adapter = CallFrameToIOAccessAdapter {
            call_frame: self,
            handler,
            phantom: PhantomData::default(),
        };

        let (global_substate_handle, substate_value) = substate_io.open_substate(
            device,
            node_id,
            partition_num,
            substate_key,
            flags,
            default,
            &mut adapter,
        )?;

        let value_len = substate_value.len();
        for node_id in substate_value.references() {
            if node_id.is_global() {
                // Again, safe to overwrite because Global and DirectAccess are exclusive.
                self.stable_references
                    .insert(node_id.clone(), StableReferenceType::Global);
            }
        }

        let mut open_substate = OpenedSubstate {
            references: index_set_new(),
            owned_nodes: index_set_new(),
            ref_origin,
            global_substate_handle,
            device,
            data,
        };

        let diff = SubstateDiff::from_new_substate(substate_value)
            .expect("There should be no issues with already stored substate value");

        Self::apply_diff_to_open_substate(
            &mut self.transient_references,
            substate_io,
            &mut open_substate,
            &diff,
        );

        // Issue lock handle
        let substate_handle = self.next_handle;
        self.open_substates.insert(substate_handle, open_substate);
        self.next_handle = self.next_handle + 1;

        Ok((substate_handle, value_len))
    }

    pub fn read_substate<'f, S: CommitableSubstateStore, H: CallFrameSubstateReadHandler<C, L>>(
        &mut self,
        substate_io: &'f mut SubstateIO<S>,
        lock_handle: SubstateHandle,
        handler: &mut H,
    ) -> Result<&'f IndexedScryptoValue, CallbackError<ReadSubstateError, H::Error>> {
        let OpenedSubstate {
            global_substate_handle,
            ..
        } = self
            .open_substates
            .get(&lock_handle)
            .ok_or(CallbackError::Error(ReadSubstateError::HandleNotFound(
                lock_handle,
            )))?;

        let mut adapter = CallFrameToIOSubstateReadAdapter {
            call_frame: self,
            handler,
            handle: *global_substate_handle,
        };

        let substate = substate_io
            .read_substate(*global_substate_handle, &mut adapter)
            .map_err(|e| CallbackError::CallbackError(e))?;

        Ok(substate)
    }

    pub fn write_substate<'f, S: CommitableSubstateStore, E>(
        &mut self,
        substate_io: &'f mut SubstateIO<S>,
        lock_handle: SubstateHandle,
        substate: IndexedScryptoValue,
        handler: &mut impl CallFrameIOAccessHandler<C, L, E>,
    ) -> Result<(), CallbackError<WriteSubstateError, E>> {
        let mut opened_substate =
            self.open_substates
                .swap_remove(&lock_handle)
                .ok_or(CallbackError::Error(WriteSubstateError::HandleNotFound(
                    lock_handle,
                )))?;

        let (.., data) = substate_io
            .substate_locks
            .get(opened_substate.global_substate_handle);
        if !data.flags.contains(LockFlags::MUTABLE) {
            return Err(CallbackError::Error(WriteSubstateError::NoWritePermission));
        }

        let diff = opened_substate
            .diff(&substate)
            .map_err(|e| CallbackError::Error(WriteSubstateError::SubstateDiffError(e)))?;

        self.process_substate_diff(substate_io, handler, opened_substate.device, &diff)
            .map_err(|e| e.map(WriteSubstateError::ProcessSubstateError))?;

        Self::apply_diff_to_open_substate(
            &mut self.transient_references,
            substate_io,
            &mut opened_substate,
            &diff,
        );

        let mut adapter = CallFrameToIOAccessAdapter {
            call_frame: self,
            handler,
            phantom: PhantomData::default(),
        };

        substate_io.write_substate(
            opened_substate.global_substate_handle,
            substate,
            &mut adapter,
        )?;

        self.open_substates.insert(lock_handle, opened_substate);

        Ok(())
    }

    pub fn close_substate<S: CommitableSubstateStore>(
        &mut self,
        substate_io: &mut SubstateIO<S>,
        lock_handle: SubstateHandle,
    ) -> Result<(), CloseSubstateError> {
        let mut open_substate = self
            .open_substates
            .swap_remove(&lock_handle)
            .ok_or_else(|| CloseSubstateError::HandleNotFound(lock_handle))?;

        for node_id in open_substate.owned_nodes.iter() {
            // We must maintain the invariant that opened substates must always
            // be from a visible node. Thus, we cannot close a substate if there is a
            // child opened substate.
            if substate_io.substate_locks.node_is_locked(node_id) {
                return Err(CloseSubstateError::SubstateBorrowed(node_id.clone().into()));
            }
        }

        substate_io.close_substate(open_substate.global_substate_handle);

        let diff = open_substate.diff_on_close();
        Self::apply_diff_to_open_substate(
            &mut self.transient_references,
            substate_io,
            &mut open_substate,
            &diff,
        );

        Ok(())
    }

    pub fn open_substates(&self) -> Vec<u32> {
        self.open_substates.keys().cloned().into_iter().collect()
    }

    pub fn close_all_substates<S: CommitableSubstateStore>(
        &mut self,
        substate_io: &mut SubstateIO<S>,
    ) {
        // Closing of all substates should always be possible as no invariant needs to be maintained
        for (_lock_handle, mut open_substate) in self.open_substates.drain(..) {
            substate_io.close_substate(open_substate.global_substate_handle);
            let diff = open_substate.diff_on_close();
            Self::apply_diff_to_open_substate(
                &mut self.transient_references,
                substate_io,
                &mut open_substate,
                &diff,
            );
        }
    }

    pub fn get_handle_info(&self, lock_handle: SubstateHandle) -> Option<L> {
        self.open_substates
            .get(&lock_handle)
            .map(|substate_lock| substate_lock.data.clone())
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
    pub fn set_substate<'f, S: CommitableSubstateStore, E>(
        &mut self,
        substate_io: &'f mut SubstateIO<S>,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        key: SubstateKey,
        value: IndexedScryptoValue,
        handler: &mut impl CallFrameIOAccessHandler<C, L, E>,
    ) -> Result<(), CallbackError<CallFrameSetSubstateError, E>> {
        let (_ref_origin, device) = self.get_node_ref(node_id).ok_or_else(|| {
            CallbackError::Error(CallFrameSetSubstateError::NodeNotVisible(
                node_id.clone().into(),
            ))
        })?;

        self.process_input_substate_key(&key).map_err(|e| {
            CallbackError::Error(CallFrameSetSubstateError::ProcessSubstateKeyError(e))
        })?;

        // TODO: Should process value here (For example, not allow owned objects or references) but
        // this isn't a problem for now since only native objects are allowed to use set_substate

        let mut adapter = CallFrameToIOAccessAdapter {
            call_frame: self,
            handler,
            phantom: PhantomData::default(),
        };

        substate_io.set_substate(device, node_id, partition_num, key, value, &mut adapter)?;

        Ok(())
    }

    pub fn remove_substate<'f, S: CommitableSubstateStore, E>(
        &mut self,
        substate_io: &'f mut SubstateIO<S>,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        key: &SubstateKey,
        handler: &mut impl CallFrameIOAccessHandler<C, L, E>,
    ) -> Result<Option<IndexedScryptoValue>, CallbackError<CallFrameRemoveSubstateError, E>> {
        let (_ref_origin, device) = self.get_node_ref(node_id).ok_or_else(|| {
            CallbackError::Error(CallFrameRemoveSubstateError::NodeNotVisible(
                node_id.clone().into(),
            ))
        })?;

        self.process_input_substate_key(&key).map_err(|e| {
            CallbackError::Error(CallFrameRemoveSubstateError::ProcessSubstateKeyError(e))
        })?;

        let mut adapter = CallFrameToIOAccessAdapter {
            call_frame: self,
            handler,
            phantom: PhantomData::default(),
        };

        let removed =
            substate_io.remove_substate(device, node_id, partition_num, key, &mut adapter)?;

        Ok(removed)
    }

    pub fn scan_keys<'f, K: SubstateKeyContent, S: CommitableSubstateStore, E>(
        &mut self,
        substate_io: &'f mut SubstateIO<S>,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        limit: u32,
        handler: &mut impl CallFrameIOAccessHandler<C, L, E>,
    ) -> Result<Vec<SubstateKey>, CallbackError<CallFrameScanKeysError, E>> {
        // Check node visibility
        let (_ref_origin, device) = self.get_node_ref(node_id).ok_or_else(|| {
            CallbackError::Error(CallFrameScanKeysError::NodeNotVisible(
                node_id.clone().into(),
            ))
        })?;

        let mut adapter = CallFrameToIOAccessAdapter {
            call_frame: self,
            handler,
            phantom: PhantomData::default(),
        };

        let keys =
            substate_io.scan_keys::<K, E>(device, node_id, partition_num, limit, &mut adapter)?;

        for key in &keys {
            self.process_output_substate_key(key).map_err(|e| {
                CallbackError::Error(CallFrameScanKeysError::ProcessSubstateKeyError(e))
            })?;
        }

        Ok(keys)
    }

    pub fn drain_substates<'f, K: SubstateKeyContent, S: CommitableSubstateStore, E>(
        &mut self,
        substate_io: &'f mut SubstateIO<S>,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        limit: u32,
        handler: &mut impl CallFrameIOAccessHandler<C, L, E>,
    ) -> Result<
        Vec<(SubstateKey, IndexedScryptoValue)>,
        CallbackError<CallFrameDrainSubstatesError, E>,
    > {
        // Check node visibility
        let (_ref_origin, device) = self.get_node_ref(node_id).ok_or_else(|| {
            CallbackError::Error(CallFrameDrainSubstatesError::NodeNotVisible(
                node_id.clone().into(),
            ))
        })?;

        let mut adapter = CallFrameToIOAccessAdapter {
            call_frame: self,
            handler,
            phantom: PhantomData::default(),
        };

        let substates = substate_io.drain_substates::<K, E>(
            device,
            node_id,
            partition_num,
            limit,
            &mut adapter,
        )?;

        for (key, substate) in &substates {
            self.process_output_substate_key(key).map_err(|e| {
                CallbackError::Error(CallFrameDrainSubstatesError::ProcessSubstateKeyError(e))
            })?;

            if !substate.owned_nodes().is_empty() {
                panic!("Unexpected owns from drain_substates");
            }

            for reference in substate.references() {
                if reference.is_global() {
                    self.stable_references
                        .insert(reference.clone(), StableReferenceType::Global);
                } else {
                    panic!("Unexpected non-global ref from drain_substates");
                }
            }
        }

        Ok(substates)
    }

    // Substate Virtualization does not apply to this call
    // Should this be prevented at this layer?
    pub fn scan_sorted<'f, S: CommitableSubstateStore, E>(
        &mut self,
        substate_io: &'f mut SubstateIO<S>,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        count: u32,
        handler: &mut impl CallFrameIOAccessHandler<C, L, E>,
    ) -> Result<
        Vec<(SortedKey, IndexedScryptoValue)>,
        CallbackError<CallFrameScanSortedSubstatesError, E>,
    > {
        // Check node visibility
        let (_ref_origin, device) = self.get_node_ref(node_id).ok_or_else(|| {
            CallbackError::Error(CallFrameScanSortedSubstatesError::NodeNotVisible(
                node_id.clone().into(),
            ))
        })?;

        let mut adapter = CallFrameToIOAccessAdapter {
            call_frame: self,
            handler,
            phantom: PhantomData::default(),
        };

        let substates =
            substate_io.scan_sorted(device, node_id, partition_num, count, &mut adapter)?;

        for (key, substate) in &substates {
            self.process_output_substate_key(&SubstateKey::Sorted(key.clone()))
                .map_err(|e| {
                    CallbackError::Error(
                        CallFrameScanSortedSubstatesError::ProcessSubstateKeyError(e),
                    )
                })?;

            if !substate.owned_nodes().is_empty() {
                panic!("Unexpected owns from scan_substates");
            }

            for reference in substate.references() {
                if reference.is_global() {
                    self.stable_references
                        .insert(reference.clone(), StableReferenceType::Global);
                } else {
                    panic!("Unexpected non-global ref from scan_substates");
                }
            }
        }

        Ok(substates)
    }

    pub fn owned_nodes(&self) -> Vec<NodeId> {
        self.owned_root_nodes.clone().into_iter().collect()
    }

    fn get_node_ref(&self, node_id: &NodeId) -> Option<(ReferenceOrigin, SubstateDevice)> {
        let node_visibility = self.get_node_visibility(node_id);
        let ref_origin = node_visibility.reference_origin(node_id.clone().into())?;
        let device = match ref_origin {
            ReferenceOrigin::FrameOwned => SubstateDevice::Heap,
            ReferenceOrigin::Global(..) | ReferenceOrigin::DirectlyAccessed => {
                SubstateDevice::Store
            }
            ReferenceOrigin::SubstateNonGlobalReference(device) => device,
        };

        Some((ref_origin, device))
    }

    pub fn get_node_visibility(&self, node_id: &NodeId) -> NodeVisibility {
        let mut visibilities = BTreeSet::<Visibility>::new();

        // Stable references
        if let Some(reference_type) = self.stable_references.get(node_id) {
            visibilities.insert(Visibility::StableReference(reference_type.clone()));
        }
        if self.always_visible_global_nodes.contains(node_id) {
            visibilities.insert(Visibility::StableReference(StableReferenceType::Global));
        }

        // Frame owned nodes
        if self.owned_root_nodes.contains(node_id) {
            visibilities.insert(Visibility::FrameOwned);
        }

        // Borrowed from substate loading
        if let Some(transient_ref) = self.transient_references.get(node_id) {
            visibilities.insert(Visibility::Borrowed(transient_ref.ref_origin));
        }

        NodeVisibility(visibilities)
    }

    fn process_substate_diff<S: CommitableSubstateStore, E>(
        &mut self,
        substate_io: &mut SubstateIO<S>,
        handler: &mut impl CallFrameIOAccessHandler<C, L, E>,
        device: SubstateDevice,
        diff: &SubstateDiff,
    ) -> Result<(), CallbackError<ProcessSubstateError, E>> {
        // Verify and Update call frame state based on diff
        {
            for added_own in &diff.added_owns {
                // Node no longer owned by frame
                self.take_node_internal(substate_io, added_own)
                    .map_err(|e| CallbackError::Error(ProcessSubstateError::TakeNodeError(e)))?;
            }

            for removed_own in &diff.removed_owns {
                // Owned nodes discarded by the substate go back to the call frame,
                // and must be explicitly dropped.
                self.owned_root_nodes.insert(removed_own.clone());
            }

            for added_ref in &diff.added_refs {
                let node_visibility = self.get_node_visibility(added_ref);
                if !node_visibility.is_visible() {
                    return Err(CallbackError::Error(ProcessSubstateError::RefNotFound(
                        added_ref.clone().into(),
                    )));
                }
                if !node_visibility.can_be_referenced_in_substate() {
                    return Err(CallbackError::Error(
                        ProcessSubstateError::RefCantBeAddedToSubstate(added_ref.clone().into()),
                    ));
                }
            }

            for removed_ref in &diff.removed_refs {
                if removed_ref.is_global() {
                    self.stable_references
                        .insert(*removed_ref, StableReferenceType::Global);
                }
            }
        }

        // Update global state
        match device {
            SubstateDevice::Heap => {
                for added_ref in &diff.added_refs {
                    if !added_ref.is_global() {
                        let (_, device) = self.get_node_ref(added_ref).unwrap();
                        substate_io
                            .non_global_node_refs
                            .increment_ref_count(*added_ref, device);
                    }
                }
                for removed_ref in &diff.removed_refs {
                    if !removed_ref.is_global() {
                        substate_io
                            .non_global_node_refs
                            .decrement_ref_count(removed_ref);
                    }
                }
            }
            SubstateDevice::Store => {
                let mut adapter = CallFrameToIOAccessAdapter {
                    call_frame: self,
                    handler,
                    phantom: PhantomData::default(),
                };

                for added_own in &diff.added_owns {
                    substate_io
                        .move_node_from_heap_to_store(added_own, &mut adapter)
                        .map_err(|e| e.map(ProcessSubstateError::PersistNodeError))?;
                }

                if let Some(removed_own) = diff.removed_owns.iter().next() {
                    return Err(CallbackError::Error(
                        ProcessSubstateError::CantDropNodeInStore(removed_own.clone().into()),
                    ));
                }

                if let Some(non_global_ref) =
                    diff.added_refs.iter().filter(|r| !r.is_global()).next()
                {
                    return Err(CallbackError::Error(
                        ProcessSubstateError::NonGlobalRefNotAllowed(non_global_ref.clone().into()),
                    ));
                }

                if let Some(non_global_ref) =
                    diff.removed_refs.iter().filter(|r| !r.is_global()).next()
                {
                    panic!(
                        "Should never have contained a non global reference: {:?}",
                        non_global_ref
                    );
                }
            }
        }

        Ok(())
    }

    fn apply_diff_to_open_substate<S: CommitableSubstateStore>(
        transient_references: &mut NonIterMap<NodeId, TransientReference>,
        substate_io: &SubstateIO<S>,
        open_substate: &mut OpenedSubstate<L>,
        diff: &SubstateDiff,
    ) {
        for added_own in &diff.added_owns {
            open_substate.owned_nodes.insert(*added_own);
            transient_references
                .entry(added_own.clone())
                .or_insert(TransientReference {
                    ref_count: 0usize,
                    ref_origin: open_substate.ref_origin, // Child inherits reference origin
                })
                .ref_count
                .add_assign(1);
        }

        for removed_own in &diff.removed_owns {
            open_substate.owned_nodes.swap_remove(removed_own);
            let mut transient_ref = transient_references.remove(removed_own).unwrap();
            if transient_ref.ref_count > 1 {
                transient_ref.ref_count -= 1;
                transient_references.insert(*removed_own, transient_ref);
            }
        }

        for added_ref in &diff.added_refs {
            open_substate.references.insert(*added_ref);

            if !added_ref.is_global() {
                let device = substate_io.non_global_node_refs.get_ref_device(added_ref);

                transient_references
                    .entry(added_ref.clone())
                    .or_insert(TransientReference {
                        ref_count: 0usize,
                        ref_origin: ReferenceOrigin::SubstateNonGlobalReference(device),
                    })
                    .ref_count
                    .add_assign(1);
            }
        }

        for removed_ref in &diff.removed_refs {
            open_substate.references.swap_remove(removed_ref);

            if !removed_ref.is_global() {
                let mut transient_ref = transient_references.remove(&removed_ref).unwrap();
                if transient_ref.ref_count > 1 {
                    transient_ref.ref_count -= 1;
                    transient_references.insert(*removed_ref, transient_ref);
                }
            }
        }
    }

    fn take_node_internal<S: CommitableSubstateStore>(
        &mut self,
        substate_io: &SubstateIO<S>,
        node_id: &NodeId,
    ) -> Result<(), TakeNodeError> {
        // If there exists a non-global node-ref we still allow the node to be
        // taken. We prevent substate locked nodes from being taken though.
        // We do not need to check children of the node as a node must be
        // substate locked in order to access any of it's children.
        if substate_io.substate_locks.node_is_locked(node_id) {
            return Err(TakeNodeError::SubstateBorrowed(node_id.clone().into()));
        }

        if self.owned_root_nodes.swap_remove(node_id) {
            Ok(())
        } else {
            Err(TakeNodeError::OwnNotFound(node_id.clone().into()))
        }
    }

    #[cfg(feature = "radix_engine_tests")]
    pub fn stable_references(&self) -> &BTreeMap<NodeId, StableReferenceType> {
        &self.stable_references
    }
}

/// Non Global Node References
/// This struct should be maintained with CallFrame as the call frame should be the only
/// manipulator. Substate I/O though the "owner" only has read-access to this structure.
pub struct NonGlobalNodeRefs {
    node_refs: NonIterMap<NodeId, (SubstateDevice, usize)>,
}

impl NonGlobalNodeRefs {
    pub fn new() -> Self {
        Self {
            node_refs: NonIterMap::new(),
        }
    }

    pub fn node_is_referenced(&self, node_id: &NodeId) -> bool {
        self.node_refs
            .get(node_id)
            .map(|(_, ref_count)| ref_count.gt(&0))
            .unwrap_or(false)
    }

    fn get_ref_device(&self, node_id: &NodeId) -> SubstateDevice {
        let (device, ref_count) = self.node_refs.get(node_id).unwrap();

        if ref_count.eq(&0) {
            panic!("Reference no longer exists");
        }

        *device
    }

    fn increment_ref_count(&mut self, node_id: NodeId, device: SubstateDevice) {
        let (_, ref_count) = self.node_refs.entry(node_id).or_insert((device, 0));
        ref_count.add_assign(1);
    }

    fn decrement_ref_count(&mut self, node_id: &NodeId) {
        let (_, ref_count) = self
            .node_refs
            .get_mut(node_id)
            .unwrap_or_else(|| panic!("Node {:?} not found", node_id));
        ref_count.sub_assign(1);
    }
}

/// Structure which keeps track of all transient substates or substates
/// which are never committed though can have transaction runtime state
pub struct TransientSubstates {
    pub transient_substates: BTreeMap<NodeId, BTreeSet<(PartitionNumber, SubstateKey)>>,
}

impl TransientSubstates {
    pub fn new() -> Self {
        Self {
            transient_substates: BTreeMap::new(),
        }
    }

    pub fn mark_as_transient(
        &mut self,
        node_id: NodeId,
        partition_num: PartitionNumber,
        substate_key: SubstateKey,
    ) {
        self.transient_substates
            .entry(node_id)
            .or_insert(BTreeSet::new())
            .insert((partition_num, substate_key));
    }

    pub fn is_transient(
        &self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        substate_key: &SubstateKey,
    ) -> bool {
        match self.transient_substates.get(node_id) {
            Some(transient) => transient.contains(&(partition_num, substate_key.clone())),
            None => false,
        }
    }
}
