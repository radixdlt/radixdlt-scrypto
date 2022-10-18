use crate::engine::*;
use crate::fee::FeeReserve;
use crate::model::*;
use crate::types::*;

// TODO: still lots of unwraps

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RENodePointer {
    Heap {
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
            RENodePointer::Heap { root, .. } => RENodePointer::Heap {
                root: root.clone(),
                id: Option::Some(child_id),
            },
            RENodePointer::Store(..) => RENodePointer::Store(child_id),
        }
    }

    pub fn borrow_substate<'f, 'p, 's, R: FeeReserve>(
        &self,
        offset: &SubstateOffset,
        heap: &'f mut Heap,
        track: &'f mut Track<'s, R>,
    ) -> Result<SubstateRef<'f>, RuntimeError> {
        let substate_ref = match self {
            RENodePointer::Heap { root, id } => {
                let heap_re_node = heap.get_node_mut(*root)?.get_node_mut(id.as_ref());
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

    // TODO: ref drop mechanism
    // TODO: concurrent refs and mut refs
}
