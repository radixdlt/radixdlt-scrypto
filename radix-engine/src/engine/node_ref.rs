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
        track: &'f mut Track<'s, R>,
    ) -> Result<Option<RENodePointer>, RuntimeError> {
        if let RENodeId::Global(..) = self.node_id() {
            let offset = SubstateOffset::Global(GlobalOffset::Global);
            self.acquire_lock(offset.clone(), false, false, track)
                .map_err(RuntimeError::KernelError)?;

            let substate_ref = self.borrow_substate(&offset, call_frames, track)?;
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
        mutable: bool,
        write_through: bool,
        track: &mut Track<'s, R>,
    ) -> Result<(), KernelError> {
        let substate_id = SubstateId(self.node_id(), offset);

        match self {
            RENodePointer::Store(..) => track
                .acquire_lock(substate_id, mutable, write_through)
                .map_err(KernelError::TrackError),
            RENodePointer::Heap { .. } => {
                if write_through {
                    Err(KernelError::TrackError(
                        TrackError::WriteThroughOnNewSubstate(substate_id),
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
        write_through: bool,
        track: &mut Track<'s, R>,
    ) -> Result<(), KernelError> {
        match self {
            RENodePointer::Store(..) => track
                .release_lock(SubstateId(self.node_id(), offset), write_through)
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

    pub fn to_ref<'f, 'p, 's, R: FeeReserve>(
        &self,
        call_frames: &'f Vec<CallFrame>,
        track: &'f mut Track<'s, R>,
    ) -> RENodeRef<'f, 's, R> {
        match self {
            RENodePointer::Heap { frame_id, root, id } => {
                let frame = call_frames.get(*frame_id).unwrap();
                RENodeRef::Stack(frame.owned_heap_nodes.get(root).unwrap(), id.clone())
            }
            RENodePointer::Store(node_id) => RENodeRef::Track(track, node_id.clone()),
        }
    }

    pub fn to_ref_mut<'f, 'p, 's, R: FeeReserve>(
        &self,
        call_frames: &'f mut Vec<CallFrame>,
        track: &'f mut Track<'s, R>,
    ) -> RENodeRefMut<'f, 's, R> {
        match self {
            RENodePointer::Heap { frame_id, root, id } => {
                let frame = call_frames.get_mut(*frame_id).unwrap();
                RENodeRefMut::Stack(
                    frame.owned_heap_nodes.get_mut(root).unwrap(),
                    root.clone(),
                    id.clone(),
                )
            }
            RENodePointer::Store(node_id) => RENodeRefMut::Track(track, node_id.clone()),
        }
    }

    pub fn borrow_substate<'f, 'p, 's, R: FeeReserve>(
        &self,
        offset: &SubstateOffset,
        call_frames: &'f mut Vec<CallFrame>,
        track: &'f mut Track<'s, R>,
    ) -> Result<SubstateRef<'f>, RuntimeError> {
        let substate_ref = match self {
            RENodePointer::Heap { frame_id, root, id } => {
                let frame = call_frames.get_mut(*frame_id).unwrap();
                let heap_re_node = frame
                    .owned_heap_nodes
                    .get_mut(&root)
                    .unwrap()
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

    pub fn add_children<'f, 's, R: FeeReserve>(
        &self,
        child_nodes: HashMap<RENodeId, HeapRootRENode>,
        call_frames: &'f mut Vec<CallFrame>,
        track: &'f mut Track<'s, R>,
    ) {
        match self {
            RENodePointer::Heap { frame_id, root, .. } => {
                let frame = call_frames.get_mut(*frame_id).unwrap();
                let root_node = frame.owned_heap_nodes.get_mut(&root).unwrap();

                for (id, val) in child_nodes {
                    root_node.insert_non_root_nodes(val.to_nodes(id));
                }
            }
            RENodePointer::Store(..) => {
                for (id, val) in child_nodes {
                    for (id, node) in val.to_nodes(id) {
                        track.put_node(id, node);
                    }
                }
            }
        }
    }

    // TODO: ref drop mechanism
    // TODO: concurrent refs and mut refs
}

pub enum RENodeRef<'f, 's, R: FeeReserve> {
    Stack(&'f HeapRootRENode, Option<RENodeId>),
    Track(&'f mut Track<'s, R>, RENodeId),
}

impl<'f, 's, R: FeeReserve> RENodeRef<'f, 's, R> {
    pub fn bucket(&self) -> &Bucket {
        match self {
            RENodeRef::Stack(value, id) => id
                .as_ref()
                .map_or(value.root(), |v| value.non_root(v))
                .bucket(),
            RENodeRef::Track(..) => {
                panic!("Unexpected")
            }
        }
    }

    pub fn proof(&self) -> &Proof {
        match self {
            RENodeRef::Stack(value, id) => id
                .as_ref()
                .map_or(value.root(), |v| value.non_root(v))
                .proof(),
            RENodeRef::Track(..) => {
                panic!("Unexpected")
            }
        }
    }

    pub fn vault(&mut self) -> &Vault {
        match self {
            RENodeRef::Stack(value, id) => id
                .as_ref()
                .map_or(value.root(), |v| value.non_root(v))
                .vault(),

            RENodeRef::Track(track, node_id) => track.borrow_node(node_id).vault(),
        }
    }
}

pub enum RENodeRefMut<'f, 's, R: FeeReserve> {
    Stack(&'f mut HeapRootRENode, RENodeId, Option<RENodeId>),
    Track(&'f mut Track<'s, R>, RENodeId),
}

impl<'f, 's, R: FeeReserve> RENodeRefMut<'f, 's, R> {
    pub fn bucket_mut(&mut self) -> &mut Bucket {
        match self {
            RENodeRefMut::Stack(root_node, _, id) => {
                root_node.get_node_mut(id.as_ref()).bucket_mut()
            }
            RENodeRefMut::Track(..) => panic!("Bucket should be in stack"),
        }
    }

    pub fn proof_mut(&mut self) -> &mut Proof {
        match self {
            RENodeRefMut::Stack(root_node, _, id) => {
                root_node.get_node_mut(id.as_ref()).proof_mut()
            }
            RENodeRefMut::Track(..) => panic!("Proof should be in stack"),
        }
    }

    pub fn auth_zone_mut(&mut self) -> &mut AuthZone {
        match self {
            RENodeRefMut::Stack(re_value, _, id) => {
                re_value.get_node_mut(id.as_ref()).auth_zone_mut()
            }
            RENodeRefMut::Track(..) => panic!("AuthZone should be in stack"),
        }
    }

    pub fn worktop_mut(&mut self) -> &mut Worktop {
        match self {
            RENodeRefMut::Stack(root_node, _, id) => {
                root_node.get_node_mut(id.as_ref()).worktop_mut()
            }
            RENodeRefMut::Track(track, node_id) => track.borrow_node_mut(node_id).worktop_mut(),
        }
    }

    pub fn vault_mut(&mut self) -> &mut Vault {
        match self {
            RENodeRefMut::Stack(root_node, _, id) => {
                root_node.get_node_mut(id.as_ref()).vault_mut()
            }
            RENodeRefMut::Track(track, node_id) => track.borrow_node_mut(node_id).vault_mut(),
        }
    }
}
