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
                .map_err(KernelError::SubstateError),
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
                .map_err(KernelError::SubstateError),
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

    pub fn vault(&mut self) -> &Vault {
        match self {
            RENodeRef::Stack(value, id) => id
                .as_ref()
                .map_or(value.root(), |v| value.non_root(v))
                .vault(),

            RENodeRef::Track(track, node_id) => track.borrow_node(node_id).vault(),
        }
    }

    pub fn system(&self) -> &System {
        match self {
            RENodeRef::Stack(value, id) => id
                .as_ref()
                .map_or(value.root(), |v| value.non_root(v))
                .system(),
            RENodeRef::Track(track, node_id) => track.borrow_node(node_id).system(),
        }
    }

    pub fn resource_manager(&self) -> &ResourceManager {
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

    pub fn package(&self) -> &Package {
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
    pub fn read_scrypto_value(
        &mut self,
        substate_id: &SubstateId,
    ) -> Result<ScryptoValue, RuntimeError> {
        match substate_id {
            SubstateId::ComponentInfo(..) => {
                Ok(ScryptoValue::from_typed(&self.component_mut().info))
            }
            SubstateId::ComponentState(..) => {
                Ok(ScryptoValue::from_slice(&self.component_mut().state.state)
                    .expect("Failed to decode component state"))
            }
            SubstateId::NonFungible(.., id) => {
                Ok(ScryptoValue::from_typed(&self.non_fungible_get(id)))
            }
            SubstateId::KeyValueStoreEntry(.., key) => {
                Ok(ScryptoValue::from_typed(&self.kv_store_get(&key)))
            }
            SubstateId::NonFungibleSpace(..)
            | SubstateId::Vault(..)
            | SubstateId::KeyValueStoreSpace(..)
            | SubstateId::Package(..)
            | SubstateId::ResourceManager(..)
            | SubstateId::System(..)
            | SubstateId::Bucket(..)
            | SubstateId::Proof(..)
            | SubstateId::AuthZone(..)
            | SubstateId::Worktop => {
                panic!("Should never have received permissions to read this native type.");
            }
        }
    }

    pub fn replace_value_with_default(&mut self, substate_id: &SubstateId) {
        match substate_id {
            SubstateId::ComponentInfo(..)
            | SubstateId::ComponentState(..)
            | SubstateId::NonFungibleSpace(..)
            | SubstateId::KeyValueStoreSpace(..)
            | SubstateId::KeyValueStoreEntry(..)
            | SubstateId::Vault(..)
            | SubstateId::Package(..)
            | SubstateId::ResourceManager(..)
            | SubstateId::System(..)
            | SubstateId::Bucket(..)
            | SubstateId::Proof(..)
            | SubstateId::AuthZone(..)
            | SubstateId::Worktop => {
                panic!("Should not get here");
            }
            SubstateId::NonFungible(.., id) => self.non_fungible_remove(&id),
        }
    }

    pub fn write_value(
        &mut self,
        substate_id: SubstateId,
        value: ScryptoValue,
        child_nodes: HashMap<RENodeId, HeapRootRENode>,
    ) -> Result<(), NodeToSubstateFailure> {
        match substate_id {
            SubstateId::ComponentState(..) => {
                self.component_state_set(value, child_nodes);
            }
            SubstateId::NonFungible(.., id) => self.non_fungible_put(id, value),
            SubstateId::KeyValueStoreEntry(.., key) => {
                self.kv_store_put(key, value, child_nodes)?;
            }
            _ => {
                panic!("Should not get here");
            }
        }
        Ok(())
    }

    pub fn kv_store_put(
        &mut self,
        key: Vec<u8>,
        value: ScryptoValue,
        to_store: HashMap<RENodeId, HeapRootRENode>,
    ) -> Result<(), NodeToSubstateFailure> {
        let substate = KeyValueStoreEntrySubstate(Some(value.raw));
        match self {
            RENodeRefMut::Stack(root_node, node_id) => {
                root_node
                    .get_node_mut(node_id.as_ref())
                    .kv_store_mut()
                    .put(key, substate);
                for (id, val) in to_store {
                    root_node.insert_non_root_nodes(val.to_nodes(id));
                }
            }
            RENodeRefMut::Track(track, node_id) => {
                let parent_substate_id = match node_id {
                    RENodeId::KeyValueStore(kv_store_id) => {
                        SubstateId::KeyValueStoreSpace(*kv_store_id)
                    }
                    _ => panic!("Unexpected"),
                };
                track.set_key_value(parent_substate_id, key, substate);
                for (id, val) in to_store {
                    for (id, node) in val.to_nodes(id) {
                        track.put_node(id, node);
                    }
                }
            }
        }

        Ok(())
    }

    pub fn kv_store_get(&mut self, key: &[u8]) -> KeyValueStoreEntrySubstate {
        match self {
            RENodeRefMut::Stack(root_node, node_id) => {
                let kv_store = root_node.get_node_mut(node_id.as_ref()).kv_store_mut();
                kv_store.get_loaded(key)
            }
            RENodeRefMut::Track(track, node_id) => {
                // TODO: use `KeyValueStore::get`

                let parent_substate_id = match node_id {
                    RENodeId::KeyValueStore(kv_store_id) => {
                        SubstateId::KeyValueStoreSpace(*kv_store_id)
                    }
                    _ => panic!("Unexpected"),
                };
                let substate_value = track.read_key_value(parent_substate_id, key.to_vec());
                substate_value.into()
            }
        }
    }

    pub fn non_fungible_get(&mut self, id: &NonFungibleId) -> NonFungibleSubstate {
        match self {
            RENodeRefMut::Stack(root_node, node_id) => {
                let resource_manager = node_id
                    .as_ref()
                    .map_or(root_node.root(), |v| root_node.non_root(v))
                    .resource_manager();
                resource_manager
                    .loaded_non_fungibles
                    .get(id)
                    .cloned()
                    .unwrap_or(NonFungibleSubstate(None))
            }
            RENodeRefMut::Track(track, node_id) => {
                let parent_substate_id = match node_id {
                    RENodeId::ResourceManager(resource_address) => {
                        SubstateId::NonFungibleSpace(*resource_address)
                    }
                    _ => panic!("Unexpected"),
                };
                let substate = track.read_key_value(parent_substate_id, id.to_vec());
                substate.into()
            }
        }
    }

    pub fn non_fungible_remove(&mut self, id: &NonFungibleId) {
        let substate = NonFungibleSubstate(None);

        match self {
            RENodeRefMut::Stack(root_node, node_id) => {
                let resource_manager = node_id
                    .as_ref()
                    .map_or(root_node.root(), |v| root_node.non_root(v))
                    .resource_manager();
                resource_manager
                    .loaded_non_fungibles
                    .insert(id.clone(), substate);
            }
            RENodeRefMut::Track(track, node_id) => {
                let parent_substate_id = match node_id {
                    RENodeId::ResourceManager(resource_address) => {
                        SubstateId::NonFungibleSpace(*resource_address)
                    }
                    _ => panic!("Unexpected"),
                };
                track.set_key_value(parent_substate_id, id.to_vec(), substate);
            }
        }
    }

    pub fn non_fungible_put(&mut self, id: NonFungibleId, non_fungible: NonFungible) {
        let substate = NonFungibleSubstate(Some(non_fungible));

        match self {
            RENodeRefMut::Stack(root_node, node_id) => {
                let resource_manager = node_id
                    .as_ref()
                    .map_or(root_node.root(), |v| root_node.non_root(v))
                    .resource_manager();
                resource_manager
                    .loaded_non_fungibles
                    .insert(id.clone(), substate);
            }
            RENodeRefMut::Track(track, node_id) => {
                let parent_substate_id = match node_id {
                    RENodeId::ResourceManager(resource_address) => {
                        SubstateId::NonFungibleSpace(*resource_address)
                    }
                    _ => panic!("Unexpected"),
                };
                track.set_key_value(parent_substate_id, id.to_vec(), substate);
            }
        }
    }

    pub fn component_state_set(
        &mut self,
        value: ScryptoValue,
        to_store: HashMap<RENodeId, HeapRootRENode>,
    ) {
        match self {
            RENodeRefMut::Stack(root_node, node_id) => {
                let component = root_node.get_node_mut(node_id.as_ref()).component_mut();
                component.state = Some(ComponentStateSubstate { raw: value.raw });
                for (id, val) in to_store {
                    root_node.insert_non_root_nodes(val.to_nodes(id));
                }
            }
            RENodeRefMut::Track(track, node_id) => {
                let component = track.borrow_node_mut(node_id).component_mut();
                component.state = Some(ComponentStateSubstate { raw: value.raw });
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
