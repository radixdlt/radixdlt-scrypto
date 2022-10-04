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
    pub fn acquire_lock<'s, R: FeeReserve>(
        &self,
        substate_id: SubstateId,
        mutable: bool,
        write_through: bool,
        track: &mut Track<'s, R>,
    ) -> Result<(), KernelError> {
        match self {
            RENodePointer::Store(..) => track
                .acquire_lock(substate_id.clone(), mutable, write_through)
                .map_err(KernelError::TrackError),
            RENodePointer::Heap { .. } => Ok(()),
        }
    }

    pub fn release_lock<'s, R: FeeReserve>(
        &self,
        substate_id: SubstateId,
        write_through: bool,
        track: &mut Track<'s, R>,
    ) -> Result<(), KernelError> {
        match self {
            RENodePointer::Store(..) => track
                .release_lock(substate_id, write_through)
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
                RENodeRefMut::Stack(frame.owned_heap_nodes.get_mut(root).unwrap(), id.clone())
            }
            RENodePointer::Store(node_id) => RENodeRefMut::Track(track, node_id.clone()),
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

    pub fn system(&mut self) -> &System {
        match self {
            RENodeRef::Stack(value, id) => id
                .as_ref()
                .map_or(value.root(), |v| value.non_root(v))
                .system(),
            RENodeRef::Track(track, node_id) => track.borrow_node(node_id).system(),
        }
    }

    pub fn resource_manager(&mut self) -> &ResourceManager {
        match self {
            RENodeRef::Stack(value, id) => id
                .as_ref()
                .map_or(value.root(), |v| value.non_root(v))
                .resource_manager(),
            RENodeRef::Track(track, node_id) => track.borrow_node(node_id).resource_manager(),
        }
    }

    pub fn component(&mut self) -> &Component {
        match self {
            RENodeRef::Stack(value, id) => id
                .as_ref()
                .map_or(value.root(), |v| value.non_root(v))
                .component(),
            RENodeRef::Track(track, node_id) => track.borrow_node(node_id).component(),
        }
    }

    pub fn package(&mut self) -> &Package {
        match self {
            RENodeRef::Stack(value, id) => id
                .as_ref()
                .map_or(value.root(), |v| value.non_root(v))
                .package(),
            RENodeRef::Track(track, node_id) => track.borrow_node(node_id).package(),
        }
    }
}

pub enum RENodeRefMut<'f, 's, R: FeeReserve> {
    Stack(&'f mut HeapRootRENode, Option<RENodeId>),
    Track(&'f mut Track<'s, R>, RENodeId),
}

impl<'f, 's, R: FeeReserve> RENodeRefMut<'f, 's, R> {
    // TODO: should enable this for `RENodeRef` as well
    pub fn read_substate(
        &mut self,
        substate_id: &SubstateId,
    ) -> Result<ScryptoValue, RuntimeError> {
        match substate_id {
            SubstateId::ComponentInfo(..) => {
                Ok(ScryptoValue::from_typed(&self.component_mut().info))
            }
            SubstateId::ComponentState(..) => Ok(ScryptoValue::from_typed(
                &self
                    .component_state_get()
                    .map_err(|e| RuntimeError::KernelError(KernelError::TrackError(e)))?,
            )),
            SubstateId::NonFungible(.., id) => {
                Ok(ScryptoValue::from_typed(&self.non_fungible_get(id)))
            }
            SubstateId::KeyValueStoreEntry(.., key) => {
                Ok(ScryptoValue::from_typed(&self.key_value_store_get(&key)))
            }
            s @ _ => {
                panic!("Should never have received permissions to read {:?}.", s);
            }
        }
    }

    pub fn write_substate(
        &mut self,
        substate_id: SubstateId,
        substate: ScryptoValue,
        child_nodes: HashMap<RENodeId, HeapRootRENode>,
    ) {
        match substate_id {
            SubstateId::ComponentState(_) => {
                let actual_substate: ComponentStateSubstate =
                    scrypto_decode(&substate.raw).expect("TODO: who should check this");
                self.component_state_put(actual_substate, child_nodes);
            }
            SubstateId::NonFungible(_, id) => {
                let actual_substate: NonFungibleSubstate =
                    scrypto_decode(&substate.raw).expect("TODO: who should check this");
                self.non_fungible_put(id, actual_substate);
            }
            SubstateId::KeyValueStoreEntry(.., key) => {
                let actual_substate: KeyValueStoreEntrySubstate =
                    scrypto_decode(&substate.raw).expect("TODO: who should check this");
                self.key_value_store_put(key, actual_substate, child_nodes);
            }
            s @ _ => {
                panic!("Should never have received permissions to write {:?}.", s);
            }
        }
    }

    // TODO: can we move these substate getter and setter to the node representation?

    pub fn key_value_store_get(&mut self, key: &[u8]) -> KeyValueStoreEntrySubstate {
        if let Some(entry) = self.key_value_store_mut().get(key) {
            return entry.clone();
        }

        match self {
            RENodeRefMut::Stack(..) => {
                let substate = KeyValueStoreEntrySubstate(None); // virtualization
                self.key_value_store_mut()
                    .put(key.to_vec(), substate.clone());
                substate
            }
            RENodeRefMut::Track(track, node_id) => {
                // Read the key value
                let parent_substate_id = match node_id {
                    RENodeId::KeyValueStore(key_value_store_id) => {
                        SubstateId::KeyValueStoreSpace(*key_value_store_id)
                    }
                    _ => panic!("Unexpected"),
                };
                let substate = track.read_key_value(parent_substate_id, key.to_vec());
                let specific_substate: KeyValueStoreEntrySubstate = substate.into();

                // Store in the node
                let key_value_store = track.borrow_node_mut(node_id).key_value_store_mut();
                key_value_store.put(key.to_vec(), specific_substate.clone());

                specific_substate
            }
        }
    }

    pub fn key_value_store_put(
        &mut self,
        key: Vec<u8>,
        substate: KeyValueStoreEntrySubstate, // TODO: disallow soft deletion
        to_store: HashMap<RENodeId, HeapRootRENode>,
    ) {
        match self {
            RENodeRefMut::Stack(root_node, node_id) => {
                root_node
                    .get_node_mut(node_id.as_ref())
                    .key_value_store_mut()
                    .put(key, substate);
                for (id, val) in to_store {
                    root_node.insert_non_root_nodes(val.to_nodes(id));
                }
            }
            RENodeRefMut::Track(track, node_id) => {
                let key_value_store = track.borrow_node_mut(node_id).key_value_store_mut();
                key_value_store.put(key, substate);
                for (id, val) in to_store {
                    for (id, node) in val.to_nodes(id) {
                        track.put_node(id, node);
                    }
                }
            }
        }
    }

    pub fn non_fungible_get(&mut self, id: &NonFungibleId) -> NonFungibleSubstate {
        if let Some(non_fungible) = self.non_fungible_store_mut().get(id) {
            return non_fungible.clone();
        }

        match self {
            RENodeRefMut::Stack(..) => {
                let substate = NonFungibleSubstate(None); // virtualization
                self.non_fungible_store_mut()
                    .put(id.clone(), substate.clone());
                substate
            }
            RENodeRefMut::Track(track, node_id) => {
                // Read key value
                let parent_substate_id = match node_id {
                    RENodeId::NonFungibleStore(store_id) => SubstateId::NonFungibleSpace(*store_id),
                    _ => panic!("Unexpected"),
                };
                let substate = track.read_key_value(parent_substate_id, id.to_vec());
                let specific_substate: NonFungibleSubstate = substate.into();

                // Store it in the node
                let non_fungible_store = track.borrow_node_mut(node_id).non_fungible_store_mut();
                non_fungible_store.put(id.clone(), specific_substate.clone());

                specific_substate
            }
        }
    }

    pub fn non_fungible_put(&mut self, id: NonFungibleId, substate: NonFungibleSubstate) {
        self.non_fungible_store_mut().put(id, substate);
    }

    pub fn component_state_get(&mut self) -> Result<ComponentStateSubstate, TrackError> {
        if let Some(state) = self.component_mut().get_state() {
            return Ok(state.clone());
        }

        match self {
            RENodeRefMut::Stack(..) => panic!("Every HEAP component should contain a state"),
            RENodeRefMut::Track(track, node_id) => {
                let substate_id = match node_id {
                    RENodeId::Component(address) => SubstateId::ComponentState(*address),
                    _ => panic!("Unexpected"),
                };

                // TODO: Don't believe this is the right abstraction. Given I have `&mut Component`,
                // I should be allowed to read the `state` substate without further locking.
                // Further more, the implementation releases the WRITE LOCK even though the substate
                // is still cached in the node.

                // Load the component state substate
                track.acquire_lock(substate_id.clone(), true, false)?;
                let component_state = track
                    .borrow_substate(substate_id.clone())
                    .component_state()
                    .clone();
                track.release_lock(substate_id, false)?;

                // Put it into the component node
                let component = track.borrow_node_mut(node_id).component_mut();
                component.put_state(component_state.clone());

                Ok(component_state)
            }
        }
    }

    pub fn component_state_put(
        &mut self,
        substate: ComponentStateSubstate,
        to_store: HashMap<RENodeId, HeapRootRENode>,
    ) {
        match self {
            RENodeRefMut::Stack(root_node, node_id) => {
                let component = root_node.get_node_mut(node_id.as_ref()).component_mut();
                component.state = Some(substate);
                for (id, val) in to_store {
                    root_node.insert_non_root_nodes(val.to_nodes(id));
                }
            }
            RENodeRefMut::Track(track, node_id) => {
                let component = track.borrow_node_mut(node_id).component_mut();
                component.state = Some(substate);
                for (id, val) in to_store {
                    for (id, node) in val.to_nodes(id) {
                        track.put_node(id, node);
                    }
                }
            }
        }
    }

    pub fn bucket_mut(&mut self) -> &mut Bucket {
        match self {
            RENodeRefMut::Stack(root_node, id) => root_node.get_node_mut(id.as_ref()).bucket_mut(),
            RENodeRefMut::Track(..) => panic!("Bucket should be in stack"),
        }
    }

    pub fn proof_mut(&mut self) -> &mut Proof {
        match self {
            RENodeRefMut::Stack(root_node, id) => root_node.get_node_mut(id.as_ref()).proof_mut(),
            RENodeRefMut::Track(..) => panic!("Proof should be in stack"),
        }
    }

    pub fn auth_zone_mut(&mut self) -> &mut AuthZone {
        match self {
            RENodeRefMut::Stack(re_value, id) => re_value.get_node_mut(id.as_ref()).auth_zone_mut(),
            RENodeRefMut::Track(..) => panic!("AuthZone should be in stack"),
        }
    }

    pub fn resource_manager_mut(&mut self) -> &mut ResourceManager {
        match self {
            RENodeRefMut::Stack(root_node, id) => {
                root_node.get_node_mut(id.as_ref()).resource_manager_mut()
            }
            RENodeRefMut::Track(track, node_id) => {
                track.borrow_node_mut(node_id).resource_manager_mut()
            }
        }
    }

    pub fn non_fungible_store_mut(&mut self) -> &mut NonFungibleStore {
        match self {
            RENodeRefMut::Stack(root_node, id) => {
                root_node.get_node_mut(id.as_ref()).non_fungible_store_mut()
            }
            RENodeRefMut::Track(track, node_id) => {
                track.borrow_node_mut(node_id).non_fungible_store_mut()
            }
        }
    }

    pub fn key_value_store_mut(&mut self) -> &mut KeyValueStore {
        match self {
            RENodeRefMut::Stack(root_node, id) => {
                root_node.get_node_mut(id.as_ref()).key_value_store_mut()
            }
            RENodeRefMut::Track(track, node_id) => {
                track.borrow_node_mut(node_id).key_value_store_mut()
            }
        }
    }

    pub fn system_mut(&mut self) -> &mut System {
        match self {
            RENodeRefMut::Stack(root_node, id) => root_node.get_node_mut(id.as_ref()).system_mut(),
            RENodeRefMut::Track(track, node_id) => track.borrow_node_mut(node_id).system_mut(),
        }
    }

    pub fn worktop_mut(&mut self) -> &mut Worktop {
        match self {
            RENodeRefMut::Stack(root_node, id) => root_node.get_node_mut(id.as_ref()).worktop_mut(),
            RENodeRefMut::Track(track, node_id) => track.borrow_node_mut(node_id).worktop_mut(),
        }
    }

    pub fn vault_mut(&mut self) -> &mut Vault {
        match self {
            RENodeRefMut::Stack(root_node, id) => root_node.get_node_mut(id.as_ref()).vault_mut(),
            RENodeRefMut::Track(track, node_id) => track.borrow_node_mut(node_id).vault_mut(),
        }
    }

    pub fn component_mut(&mut self) -> &mut Component {
        match self {
            RENodeRefMut::Stack(root_node, id) => {
                root_node.get_node_mut(id.as_ref()).component_mut()
            }
            RENodeRefMut::Track(track, node_id) => track.borrow_node_mut(node_id).component_mut(),
        }
    }
}

pub fn verify_stored_value_update(
    old: &HashSet<RENodeId>,
    missing: &HashSet<RENodeId>,
) -> Result<(), RuntimeError> {
    // TODO: optimize intersection search
    for old_id in old.iter() {
        if !missing.contains(&old_id) {
            return Err(RuntimeError::KernelError(KernelError::StoredNodeRemoved(
                old_id.clone(),
            )));
        }
    }

    for missing_id in missing.iter() {
        if !old.contains(missing_id) {
            return Err(RuntimeError::KernelError(KernelError::RENodeNotFound(
                *missing_id,
            )));
        }
    }

    Ok(())
}
