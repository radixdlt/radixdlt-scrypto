use crate::engine::*;
use crate::fee::FeeReserve;
use crate::model::*;
use crate::types::*;

// TODO: still lots of unwraps

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RENodePointer {
    Heap {
        frame_id: usize,
        root: RENodeId,
        id: Option<RENodeId>,
    },
    Store(RENodeId),
}

impl RENodePointer {
    pub fn node_id(&self) -> RENodeId {
        match self {
            RENodePointer::Heap { root, id, .. } => id.unwrap_or(*root),
            RENodePointer::Store(node_id) => *node_id,
        }
    }

    pub fn node_deref<'f, 's, R: FeeReserve>(
        &self,
        call_frames: &'f mut Vec<CallFrame>,
        heap: &'f mut Heap,
        track: &'f mut Track<'s, R>,
    ) -> Result<Option<RENodePointer>, RuntimeError> {
        if let RENodeId::Global(..) = self.node_id() {
            let offset = SubstateOffset::Global(GlobalOffset::Global);
            self.acquire_lock(offset.clone(), LockFlags::read_only(), track)
                .map_err(RuntimeError::KernelError)?;

            let substate_ref = self.borrow_substate(&offset, call_frames, heap, track)?;
            let node_id = substate_ref.global_address().node_deref();
            self.release_lock(offset, false, track)
                .map_err(RuntimeError::KernelError)?;
            Ok(Some(RENodePointer::Store(node_id)))
        } else {
            Ok(None)
        }
    }

    pub fn acquire_lock<'s, R: FeeReserve>(
        &self,
        offset: SubstateOffset,
        flags: LockFlags,
        track: &mut Track<'s, R>,
    ) -> Result<(), KernelError> {
        let substate_id = SubstateId(self.node_id(), offset);

        match self {
            RENodePointer::Store(..) => track
                .acquire_lock(substate_id, flags)
                .map_err(KernelError::TrackError),
            RENodePointer::Heap { .. } => {
                if flags.contains(LockFlags::UNMODIFIED_BASE) {
                    Err(KernelError::TrackError(
                        TrackError::LockUnmodifiedBaseOnNewSubstate(substate_id),
                    ))
                } else {
                    Ok(())
                }
            }
        }
    }

    pub fn release_lock<'s, R: FeeReserve>(
        &self,
        offset: SubstateOffset,
        force_write: bool,
        track: &mut Track<'s, R>,
    ) -> Result<(), KernelError> {
        match self {
            RENodePointer::Store(..) => track
                .release_lock(SubstateId(self.node_id(), offset), force_write)
                .map_err(KernelError::TrackError),
            RENodePointer::Heap { .. } => Ok(()),
        }
    }

    pub fn child(&self, child_id: RENodeId) -> RENodePointer {
        match self {
            RENodePointer::Heap { frame_id, root, .. } => RENodePointer::Heap {
                frame_id: frame_id.clone(),
                root: root.clone(),
                id: Option::Some(child_id),
            },
            RENodePointer::Store(..) => RENodePointer::Store(child_id),
        }
    }

    pub fn borrow_substate<'f, 'p, 's, R: FeeReserve>(
        &self,
        offset: &SubstateOffset,
        call_frames: &'f mut Vec<CallFrame>,
        heap: &'f mut Heap,
        track: &'f mut Track<'s, R>,
    ) -> Result<SubstateRef<'f>, RuntimeError> {
        let substate_ref = match self {
            RENodePointer::Heap { frame_id, root, id } => {
                let frame = call_frames.get_mut(*frame_id).unwrap();
                let heap_re_node = frame
                    .get_owned_heap_node_mut(heap, *root)?
                    .get_node_mut(id.as_ref());
                heap_re_node.borrow_substate(offset)?
            }
            RENodePointer::Store(node_id) => match (node_id, offset) {
                (
                    RENodeId::KeyValueStore(..),
                    SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(key)),
                ) => {
                    let parent_substate_id = SubstateId(
                        *node_id,
                        SubstateOffset::KeyValueStore(KeyValueStoreOffset::Space),
                    );
                    track
                        .read_key_value(parent_substate_id, key.to_vec())
                        .to_ref()
                }
                (
                    RENodeId::NonFungibleStore(..),
                    SubstateOffset::NonFungibleStore(NonFungibleStoreOffset::Entry(
                        non_fungible_id,
                    )),
                ) => {
                    let parent_substate_id = SubstateId(
                        *node_id,
                        SubstateOffset::NonFungibleStore(NonFungibleStoreOffset::Space),
                    );
                    track
                        .read_key_value(parent_substate_id, non_fungible_id.to_vec())
                        .to_ref()
                }
                _ => track.borrow_substate(*node_id, offset.clone()).to_ref(),
            },
        };

        Ok(substate_ref)
    }

    pub fn add_child<'f, 's, R: FeeReserve>(
        &self,
        node_id: RENodeId,
        node: HeapRootRENode,
        call_frames: &'f mut Vec<CallFrame>,
        heap: &'f mut Heap,
        track: &'f mut Track<'s, R>,
    ) {
        match self {
            RENodePointer::Heap { frame_id, root, .. } => {
                let frame = call_frames.get_mut(*frame_id).unwrap();
                let root_node = frame.get_owned_heap_node_mut(heap, *root).unwrap();

                root_node.insert_non_root_nodes(node.to_nodes(node_id));
            }
            RENodePointer::Store(..) => {
                for (id, node) in node.to_nodes(node_id) {
                    let substates = node_to_substates(node);
                    for (offset, substate) in substates {
                        track.insert_substate(SubstateId(id, offset), substate);
                    }
                }
            }
        }
    }

    // TODO: ref drop mechanism
    // TODO: concurrent refs and mut refs
}
