use crate::kernel::kernel_callback_api::CallFrameReferences;
use crate::kernel::substate_io::{
    SubstateDevice, SubstateIO, SubstateIOHandler, SubstateReadHandler,
};
use crate::track::interface::{
    CallbackError, NodeSubstates, StoreAccess, SubstateStore, TrackGetSubstateError,
};
use crate::types::*;
use radix_engine_interface::api::field_api::LockFlags;
use radix_engine_interface::types::{NodeId, OpenSubstateHandle, SubstateKey};
use radix_engine_store_interface::db_key_mapper::SubstateKeyContent;

use super::heap::{Heap, HeapOpenSubstateError, HeapRemoveModuleError};

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
    pub copy_to_stable_transient_references: Vec<NodeId>,
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
            copy_to_stable_transient_references: references.stable_transient_references(),
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
            copy_to_stable_transient_references: vec![],
        }
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
    pub ref_origin: ReferenceOrigin,
    pub global_lock_handle: u32,
    pub location: SubstateDevice,
    pub data: L,
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
    Heap,
    Global(GlobalAddress),
    DirectlyAccessed,
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

    // TODO: Should we return Vec<ReferenceOrigin> and not supercede global with direct access reference
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
                    return Some(ReferenceOrigin::Heap);
                }
            }
        }

        if found_direct_access {
            return Some(ReferenceOrigin::DirectlyAccessed);
        }

        return None;
    }
}

pub trait CallFrameEventHandler<C, L, E> {
    fn on_persist_node(&mut self, heap: &Heap, node_id: &NodeId) -> Result<(), E>;

    fn on_store_access(
        &mut self,
        current_frame: &CallFrame<C, L>,
        heap: &Heap,
        store_access: StoreAccess,
    ) -> Result<(), E>;
}

pub trait CallFrameSubstateReadHandler<C, L> {
    type Error;

    fn on_read_substate(
        &mut self,
        current_frame: &CallFrame<C, L>,
        heap: &Heap,
        handle: OpenSubstateHandle,
        value: &IndexedScryptoValue,
        device: SubstateDevice,
    ) -> Result<(), Self::Error>;
}

struct WrapperHandler<'g, C, L, E, H: CallFrameEventHandler<C, L, E>> {
    handler: &'g mut H,
    call_frame: &'g CallFrame<C, L>,
    phantom: PhantomData<E>,
}

impl<'g, C, L, E, H: CallFrameEventHandler<C, L, E>> SubstateIOHandler<E>
    for WrapperHandler<'g, C, L, E, H>
{
    fn on_persist_node(&mut self, heap: &Heap, node_id: &NodeId) -> Result<(), E> {
        self.handler.on_persist_node(heap, node_id)
    }

    fn on_store_access(&mut self, heap: &Heap, store_access: StoreAccess) -> Result<(), E> {
        self.handler
            .on_store_access(self.call_frame, heap, store_access)
    }
}

struct WrapperHandler2<'g, C, L, H: CallFrameSubstateReadHandler<C, L>> {
    handler: &'g mut H,
    call_frame: &'g CallFrame<C, L>,
    handle: OpenSubstateHandle,
}

impl<'g, C, L, H: CallFrameSubstateReadHandler<C, L>> SubstateReadHandler
    for WrapperHandler2<'g, C, L, H>
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
    stable_references: NonIterMap<NodeId, StableReferenceType>,

    next_handle: OpenSubstateHandle,
    open_substates: IndexMap<OpenSubstateHandle, OpenedSubstate<L>>,
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
    GlobalRefNotFound(NodeId),
    DirectRefNotFound(NodeId),
    TransientRefNotFound(NodeId),
}

/// Represents an error when creating a node.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum CreateNodeError {
    ProcessSubstateError(ProcessSubstateError),
    NonGlobalRefNotAllowed(NodeId),
    PersistNodeError(PersistNodeError),
    ProcessSubstateKeyError(ProcessSubstateKeyError),
}

/// Represents an error when dropping a node.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum DropNodeError {
    TakeNodeError(TakeNodeError),
    NodeBorrowed(NodeId, usize),
    SubstateBorrowed(NodeId),
}

/// Represents an error when persisting a node into store.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum PersistNodeError {
    ContainsNonGlobalRef(NodeId),
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
    SubstateBorrowed(NodeId),
}

/// Represents an error when attempting to lock a substate.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum OpenSubstateError {
    NodeNotVisible(NodeId),
    HeapError(HeapOpenSubstateError),
    TrackError(Box<TrackGetSubstateError>),
    ProcessSubstateKeyError(ProcessSubstateKeyError),
    SubstateLocked(NodeId, PartitionNumber, SubstateKey),
    LockUnmodifiedBaseOnHeapNode,
    LockUnmodifiedBaseOnNewSubstate(NodeId, PartitionNumber, SubstateKey),
    LockUnmodifiedBaseOnOnUpdatedSubstate(NodeId, PartitionNumber, SubstateKey),
}

/// Represents an error when reading substates.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum ReadSubstateError {
    LockNotFound(OpenSubstateHandle),
}

/// Represents an error when writing substates.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum WriteSubstateError {
    LockNotFound(OpenSubstateHandle),
    ProcessSubstateError(ProcessSubstateError),
    NoWritePermission,
    PersistNodeError(PersistNodeError),
    NonGlobalRefNotAllowed(NodeId),
    ContainsDuplicatedOwns,
}

/// Represents an error when dropping a substate lock.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum CloseSubstateError {
    LockNotFound(OpenSubstateHandle),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum CallFrameSetSubstateError {
    NodeNotVisible(NodeId),
    SubstateLocked(NodeId, PartitionNumber, SubstateKey),
    ProcessSubstateKeyError(ProcessSubstateKeyError),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum CallFrameRemoveSubstateError {
    NodeNotVisible(NodeId),
    SubstateLocked(NodeId, PartitionNumber, SubstateKey),
    ProcessSubstateKeyError(ProcessSubstateKeyError),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum CallFrameScanKeysError {
    NodeNotVisible(NodeId),
    OwnedNodeNotSupported(NodeId),
    ProcessSubstateKeyError(ProcessSubstateKeyError),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum CallFrameDrainSubstatesError {
    NodeNotVisible(NodeId),
    OwnedNodeNotSupported(NodeId),
    ProcessSubstateKeyError(ProcessSubstateKeyError),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum CallFrameScanSortedSubstatesError {
    NodeNotVisible(NodeId),
    OwnedNodeNotSupported(NodeId),
    ProcessSubstateKeyError(ProcessSubstateKeyError),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum ProcessSubstateKeyError {
    DecodeError(DecodeError),
    NodeNotVisible(NodeId),
    OwnedNodeNotSupported,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum ProcessSubstateError {
    TakeNodeError(TakeNodeError),
    CantDropNodeInStore(NodeId),
    RefNotFound(NodeId),
}

impl<C, L: Clone> CallFrame<C, L> {
    pub fn new_root(call_frame_data: C) -> Self {
        Self {
            depth: 0,
            call_frame_data,
            stable_references: NonIterMap::new(),
            transient_references: NonIterMap::new(),
            owned_root_nodes: index_set_new(),
            next_handle: 0u32,
            open_substates: index_map_new(),
        }
    }

    pub fn new_child_from_parent(
        parent: &mut CallFrame<C, L>,
        call_frame_data: C,
        message: CallFrameMessage,
    ) -> Result<Self, CreateFrameError> {
        let mut frame = Self {
            depth: parent.depth + 1,
            call_frame_data,
            stable_references: NonIterMap::new(),
            transient_references: NonIterMap::new(),
            owned_root_nodes: index_set_new(),
            next_handle: 0u32,
            open_substates: index_map_new(),
        };

        // Copy references and move nodes
        Self::pass_message(parent, &mut frame, message)
            .map_err(CreateFrameError::PassMessageError)?;

        Ok(frame)
    }

    pub fn pass_message(
        from: &mut CallFrame<C, L>,
        to: &mut CallFrame<C, L>,
        message: CallFrameMessage,
    ) -> Result<(), PassMessageError> {
        for node_id in message.move_nodes {
            // Note that this has no impact on the `transient_references` because
            // we don't allow move of "locked nodes".
            from.take_node_internal(&node_id)
                .map_err(PassMessageError::TakeNodeError)?;
            to.owned_root_nodes.insert(node_id);
        }

        // Only allow move of `Global` and `DirectAccess` references
        for node_id in message.copy_global_references {
            if from.get_node_visibility(&node_id).is_global() {
                // Note that GLOBAL and DirectAccess references are mutually exclusive,
                // so okay to overwrite
                to.stable_references
                    .insert(node_id, StableReferenceType::Global);
            } else {
                return Err(PassMessageError::GlobalRefNotFound(node_id));
            }
        }

        for node_id in message.copy_direct_access_references {
            if from.get_node_visibility(&node_id).can_be_invoked(true) {
                to.stable_references
                    .insert(node_id, StableReferenceType::DirectAccess);
            } else {
                return Err(PassMessageError::DirectRefNotFound(node_id));
            }
        }

        for node_id in message.copy_to_stable_transient_references {
            if from.depth >= to.depth {
                panic!("Transient references only supported for downstream calls.");
            }

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
                return Err(PassMessageError::TransientRefNotFound(node_id));
            }
        }

        Ok(())
    }

    pub fn depth(&self) -> usize {
        self.depth
    }

    pub fn data(&self) -> &C {
        &self.call_frame_data
    }

    pub fn create_node<'f, S: SubstateStore, E>(
        &mut self,
        substate_io: &mut SubstateIO<S>,
        handler: &mut impl CallFrameEventHandler<C, L, E>,
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
                self.process_substate(substate_value, destination_device, None)
                    .map_err(|e| CallbackError::Error(CreateNodeError::ProcessSubstateError(e)))?;

                self.process_input_substate_key(substate_key).map_err(|e| {
                    CallbackError::Error(CreateNodeError::ProcessSubstateKeyError(e))
                })?;
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

        let mut handler = WrapperHandler {
            call_frame: self,
            handler,
            phantom: PhantomData::default(),
        };

        substate_io.create_node(&mut handler, node_id, node_substates, destination_device)?;

        Ok(())
    }

    /// Removes node from call frame and owned nodes will be possessed by this call frame.
    pub fn drop_node<S: SubstateStore>(
        &mut self,
        substate_io: &mut SubstateIO<S>,
        node_id: &NodeId,
    ) -> Result<NodeSubstates, DropNodeError> {
        self.take_node_internal(node_id)
            .map_err(DropNodeError::TakeNodeError)?;

        let node_substates = substate_io.drop_node(node_id)?;

        for (_, module) in &node_substates {
            for (_, substate_value) in module {
                //=============
                // Process own
                //=============
                for own in substate_value.owned_nodes() {
                    self.owned_root_nodes.insert(own.clone());
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
                    }
                }
            }
        }

        Ok(node_substates)
    }

    pub fn move_partition<'f, S: SubstateStore, E>(
        &mut self,
        substate_io: &'f mut SubstateIO<S>,
        handler: &mut impl CallFrameEventHandler<C, L, E>,
        src_node_id: &NodeId,
        src_partition_number: PartitionNumber,
        dest_node_id: &NodeId,
        dest_partition_number: PartitionNumber,
    ) -> Result<(), CallbackError<MoveModuleError, E>> {
        // Check ownership (and visibility)
        if !self.owned_root_nodes.contains(src_node_id) {
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

        let mut handler = WrapperHandler {
            call_frame: self,
            handler,
            phantom: PhantomData::default(),
        };

        // Move
        substate_io.move_partition(
            &mut handler,
            src_node_id,
            src_partition_number,
            dest_node_id,
            dest_partition_number,
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
                if !key_value.owned_nodes().is_empty() {
                    return Err(ProcessSubstateKeyError::OwnedNodeNotSupported);
                }

                for reference in key_value.references() {
                    if !self.get_node_visibility(reference).is_global() {
                        return Err(ProcessSubstateKeyError::NodeNotVisible(reference.clone()));
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
                for reference in key.references() {
                    if reference.is_global() {
                        self.stable_references
                            .insert(reference.clone(), StableReferenceType::Global);
                    } else {
                        return Err(ProcessSubstateKeyError::OwnedNodeNotSupported);
                    }
                }
            }
            _ => {}
        }

        Ok(())
    }

    pub fn open_substate<
        S: SubstateStore,
        E,
        F: FnMut(&Self, &Heap, StoreAccess) -> Result<(), E>,
    >(
        &mut self,
        substate_io: &mut SubstateIO<S>,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        substate_key: &SubstateKey,
        flags: LockFlags,
        on_store_access: &mut F,
        default: Option<fn() -> IndexedScryptoValue>,
        data: L,
    ) -> Result<(OpenSubstateHandle, usize), CallbackError<OpenSubstateError, E>> {
        let node_visibility = self.get_node_visibility(node_id);
        let ref_origin = if let Some(ref_origin) = node_visibility.reference_origin(node_id.clone())
        {
            ref_origin
        } else {
            return Err(CallbackError::Error(OpenSubstateError::NodeNotVisible(
                node_id.clone(),
            )));
        };

        self.process_input_substate_key(substate_key)
            .map_err(|e| CallbackError::Error(OpenSubstateError::ProcessSubstateKeyError(e)))?;

        let (global_lock_handle, substate_value, substate_location) = substate_io.open_substate(
            node_id,
            partition_num,
            substate_key,
            flags,
            &mut |heap, store_access| on_store_access(self, heap, store_access),
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
                .or_insert(TransientReference {
                    ref_count: 0usize,
                    ref_origin,
                })
                .ref_count
                .add_assign(1);
        }

        for own in &owned_nodes {
            self.transient_references
                .entry(own.clone())
                .or_insert(TransientReference {
                    ref_count: 0usize,
                    ref_origin,
                })
                .ref_count
                .add_assign(1);
        }

        // Issue lock handle
        let lock_handle = self.next_handle;
        self.open_substates.insert(
            lock_handle,
            OpenedSubstate {
                non_global_references,
                owned_nodes,
                ref_origin,
                global_lock_handle,
                location: substate_location,
                data,
            },
        );
        self.next_handle = self.next_handle + 1;

        Ok((lock_handle, substate_value.len()))
    }

    pub fn read_substate<'f, S: SubstateStore, H: CallFrameSubstateReadHandler<C, L>>(
        &mut self,
        substate_io: &'f mut SubstateIO<S>,
        lock_handle: OpenSubstateHandle,
        handler: &mut H,
    ) -> Result<&'f IndexedScryptoValue, CallbackError<ReadSubstateError, H::Error>> {
        let OpenedSubstate {
            global_lock_handle, ..
        } = self
            .open_substates
            .get(&lock_handle)
            .ok_or(CallbackError::Error(ReadSubstateError::LockNotFound(
                lock_handle,
            )))?;

        let mut handler = WrapperHandler2 {
            call_frame: self,
            handler,
            handle: *global_lock_handle,
        };

        let substate = substate_io
            .read_substate(*global_lock_handle, &mut handler)
            .map_err(|e| CallbackError::CallbackError(e))?;

        Ok(substate)
    }

    pub fn write_substate<'f, S: SubstateStore, E>(
        &mut self,
        substate_io: &'f mut SubstateIO<S>,
        handler: &mut impl CallFrameEventHandler<C, L, E>,
        lock_handle: OpenSubstateHandle,
        substate: IndexedScryptoValue,
    ) -> Result<(), CallbackError<WriteSubstateError, E>> {
        let mut opened_substate =
            self.open_substates
                .remove(&lock_handle)
                .ok_or(CallbackError::Error(WriteSubstateError::LockNotFound(
                    lock_handle,
                )))?;

        {
            let (new_owned_nodes, new_non_global_references) = self
                .process_substate(
                    &substate,
                    opened_substate.location,
                    Some((
                        &opened_substate.owned_nodes,
                        &opened_substate.non_global_references,
                    )),
                )
                .map_err(|e| CallbackError::Error(WriteSubstateError::ProcessSubstateError(e)))?;

            for new_owned_node in &new_owned_nodes {
                self.transient_references
                    .entry(new_owned_node.clone())
                    .or_insert(TransientReference {
                        ref_count: 0usize,
                        ref_origin: opened_substate.ref_origin,
                    })
                    .ref_count
                    .add_assign(1);
            }

            for new_non_global_reference in &new_non_global_references {
                self.transient_references
                    .entry(new_non_global_reference.clone())
                    .or_insert(TransientReference {
                        ref_count: 0usize,
                        ref_origin: opened_substate.ref_origin,
                    })
                    .ref_count
                    .add_assign(1);
            }

            opened_substate.owned_nodes.extend(new_owned_nodes);
            opened_substate
                .non_global_references
                .extend(new_non_global_references);
        }

        let mut handler = WrapperHandler {
            call_frame: self,
            handler,
            phantom: PhantomData::default(),
        };

        substate_io.write_substate(&mut handler, opened_substate.global_lock_handle, substate)?;

        self.open_substates.insert(lock_handle, opened_substate);

        Ok(())
    }

    pub fn close_substate<S: SubstateStore>(
        &mut self,
        substate_io: &mut SubstateIO<S>,
        lock_handle: OpenSubstateHandle,
    ) -> Result<(), CloseSubstateError> {
        let OpenedSubstate {
            global_lock_handle,
            owned_nodes,
            non_global_references,
            ..
        } = self
            .open_substates
            .remove(&lock_handle)
            .ok_or_else(|| CloseSubstateError::LockNotFound(lock_handle))?;

        substate_io.close_substate(global_lock_handle)?;

        // Shrink transient reference set
        for reference in non_global_references {
            let mut transient_ref = self.transient_references.remove(&reference).unwrap();
            if transient_ref.ref_count > 1 {
                transient_ref.ref_count -= 1;
                self.transient_references.insert(reference, transient_ref);
            }
        }
        for own in owned_nodes {
            let mut transient_ref = self.transient_references.remove(&own).unwrap();
            if transient_ref.ref_count > 1 {
                transient_ref.ref_count -= 1;
                self.transient_references.insert(own, transient_ref);
            }
        }

        Ok(())
    }

    pub fn get_handle_info(&self, lock_handle: OpenSubstateHandle) -> Option<L> {
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

        self.process_input_substate_key(&key).map_err(|e| {
            CallbackError::Error(CallFrameSetSubstateError::ProcessSubstateKeyError(e))
        })?;

        substate_io.set_substate(node_id, partition_num, key, value, on_store_access)?;

        Ok(())
    }

    pub fn remove_substate<'f, S: SubstateStore, E, F: FnMut(StoreAccess) -> Result<(), E>>(
        &mut self,
        substate_io: &'f mut SubstateIO<S>,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        key: &SubstateKey,
        on_store_access: &mut F,
    ) -> Result<Option<IndexedScryptoValue>, CallbackError<CallFrameRemoveSubstateError, E>> {
        // Check node visibility
        if !self.get_node_visibility(node_id).is_visible() {
            return Err(CallbackError::Error(
                CallFrameRemoveSubstateError::NodeNotVisible(node_id.clone()),
            ));
        }

        self.process_input_substate_key(&key).map_err(|e| {
            CallbackError::Error(CallFrameRemoveSubstateError::ProcessSubstateKeyError(e))
        })?;

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
        limit: u32,
        on_store_access: &mut F,
    ) -> Result<Vec<SubstateKey>, CallbackError<CallFrameScanKeysError, E>> {
        // Check node visibility
        if !self.get_node_visibility(node_id).is_visible() {
            return Err(CallbackError::Error(
                CallFrameScanKeysError::NodeNotVisible(node_id.clone()),
            ));
        }

        let keys =
            substate_io.scan_keys::<K, E, F>(node_id, partition_num, limit, on_store_access)?;

        for key in &keys {
            self.process_output_substate_key(key).map_err(|e| {
                CallbackError::Error(CallFrameScanKeysError::ProcessSubstateKeyError(e))
            })?;
        }

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
        limit: u32,
        on_store_access: &mut F,
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

        let substates = substate_io.drain_substates::<K, E, F>(
            node_id,
            partition_num,
            limit,
            on_store_access,
        )?;

        for (key, substate) in &substates {
            self.process_output_substate_key(key).map_err(|e| {
                CallbackError::Error(CallFrameDrainSubstatesError::ProcessSubstateKeyError(e))
            })?;

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
        on_store_access: &mut F,
    ) -> Result<Vec<(SortedU16Key, IndexedScryptoValue)>, CallbackError<CallFrameScanSortedSubstatesError, E>> {
        // Check node visibility
        if !self.get_node_visibility(node_id).is_visible() {
            return Err(CallbackError::Error(
                CallFrameScanSortedSubstatesError::NodeNotVisible(node_id.clone()),
            ));
        }

        let substates = substate_io.scan_sorted(node_id, partition_num, count, on_store_access)?;

        for (key, substate) in &substates {
            self.process_output_substate_key(&SubstateKey::Sorted(key.clone()))
                .map_err(|e| {
                    CallbackError::Error(
                        CallFrameScanSortedSubstatesError::ProcessSubstateKeyError(e),
                    )
                })?;

            for reference in substate.references() {
                if reference.is_global() {
                    self.stable_references
                        .insert(reference.clone(), StableReferenceType::Global);
                } else {
                    return Err(CallbackError::Error(
                        CallFrameScanSortedSubstatesError::OwnedNodeNotSupported(reference.clone()),
                    ));
                }
            }
        }

        Ok(substates)
    }

    pub fn close_all_substates<S: SubstateStore>(
        &mut self,
        substate_io: &mut SubstateIO<S>,
    ) -> Result<(), CloseSubstateError> {
        let lock_handles: Vec<OpenSubstateHandle> = self.open_substates.keys().cloned().collect();

        for lock_handle in lock_handles {
            self.close_substate(substate_io, lock_handle)?;
        }

        Ok(())
    }

    pub fn owned_nodes(&self) -> Vec<NodeId> {
        self.owned_root_nodes.clone().into_iter().collect()
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
        if self.owned_root_nodes.contains(node_id) {
            visibilities.insert(Visibility::FrameOwned);
        }

        // Borrowed from substate loading
        if let Some(transient_ref) = self.transient_references.get(node_id) {
            visibilities.insert(Visibility::Borrowed(transient_ref.ref_origin));
        }

        NodeVisibility(visibilities)
    }

    fn process_substate(
        &mut self,
        updated_substate: &IndexedScryptoValue,
        device: SubstateDevice,
        prev: Option<(&IndexSet<NodeId>, &IndexSet<NodeId>)>,
    ) -> Result<(IndexSet<NodeId>, IndexSet<NodeId>), ProcessSubstateError> {
        // Process owned nodes
        let new_owned_nodes = {
            let mut new_owned_nodes: IndexSet<NodeId> = index_set_new();
            let mut updated_owned_nodes: IndexSet<NodeId> = index_set_new();
            for own in updated_substate.owned_nodes() {
                let node_is_new = if let Some((old_owned_nodes, _)) = prev {
                    !old_owned_nodes.contains(own)
                } else {
                    true
                };

                if node_is_new {
                    // Node no longer owned by frame
                    self.take_node_internal(own)
                        .map_err(ProcessSubstateError::TakeNodeError)?;
                    new_owned_nodes.insert(*own);
                }
                updated_owned_nodes.insert(*own);
            }

            if let Some((old_owned_nodes, _)) = prev {
                for own in old_owned_nodes {
                    if !updated_owned_nodes.contains(own) {
                        // Node detached
                        if device.eq(&SubstateDevice::Store) {
                            return Err(ProcessSubstateError::CantDropNodeInStore(own.clone()));
                        }
                        // Owned nodes discarded by the substate go back to the call frame,
                        // and must be explicitly dropped.
                        self.owned_root_nodes.insert(own.clone());
                    }
                }
            }

            new_owned_nodes
        };

        //====================
        // Process references
        //====================
        let new_non_global_references = {
            let mut updated_references: IndexSet<NodeId> = index_set_new();
            let mut new_non_global_references: IndexSet<NodeId> = index_set_new();
            for node_id in updated_substate.references() {
                // Deduplicate
                updated_references.insert(node_id.clone());
            }

            for reference in &updated_references {
                let reference_is_new = if let Some((_, old_references)) = &prev {
                    !old_references.contains(reference)
                } else {
                    true
                };

                if reference_is_new {
                    // handle added references
                    if !self
                        .get_node_visibility(reference)
                        .can_be_referenced_in_substate()
                    {
                        return Err(ProcessSubstateError::RefNotFound(reference.clone()));
                    }

                    if !reference.is_global() {
                        new_non_global_references.insert(reference.clone());
                    }
                }
            }

            new_non_global_references
        };

        Ok((new_owned_nodes, new_non_global_references))
    }

    fn take_node_internal(&mut self, node_id: &NodeId) -> Result<(), TakeNodeError> {
        if self.owned_root_nodes.remove(node_id) {
            Ok(())
        } else {
            Err(TakeNodeError::OwnNotFound(node_id.clone()))
        }
    }
}
