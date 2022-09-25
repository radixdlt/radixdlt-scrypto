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

            RENodeRef::Track(track, node_id) => {
                let substate_id = match node_id {
                    RENodeId::Vault(vault_id) => SubstateId::Vault(*vault_id),
                    _ => panic!("Unexpected"),
                };
                track.read_node(node_id).vault()
            }
        }
    }

    pub fn system(&self) -> &System {
        match self {
            RENodeRef::Stack(value, id) => id
                .as_ref()
                .map_or(value.root(), |v| value.non_root(v))
                .system(),
            RENodeRef::Track(track, node_id) => {
                let substate_id = match node_id {
                    RENodeId::System => SubstateId::System,
                    _ => panic!("Unexpected"),
                };
                track.read_node(node_id).system()
            }
        }
    }

    pub fn resource_manager(&self) -> &ResourceManager {
        match self {
            RENodeRef::Stack(value, id) => id
                .as_ref()
                .map_or(value.root(), |v| value.non_root(v))
                .resource_manager(),
            RENodeRef::Track(track, node_id) => {
                let substate_id = match node_id {
                    RENodeId::ResourceManager(resource_address) => {
                        SubstateId::ResourceManager(*resource_address)
                    }
                    _ => panic!("Unexpected"),
                };
                track.read_node(node_id).resource_manager()
            }
        }
    }

    pub fn component(&mut self) -> &Component {
        match self {
            RENodeRef::Stack(value, id) => id
                .as_ref()
                .map_or(value.root(), |v| value.non_root(v))
                .component(),
            RENodeRef::Track(track, node_id) => {
                let substate_id = match node_id {
                    RENodeId::Component(component_address) => {
                        SubstateId::ComponentInfo(*component_address)
                    }
                    _ => panic!("Unexpected"),
                };
                track.read_node(node_id).component()
            }
        }
    }

    pub fn package(&self) -> &Package {
        match self {
            RENodeRef::Stack(value, id) => id
                .as_ref()
                .map_or(value.root(), |v| value.non_root(v))
                .package(),
            RENodeRef::Track(track, node_id) => {
                let substate_id = match node_id {
                    RENodeId::Package(package_address) => SubstateId::Package(*package_address),
                    _ => panic!("Unexpected"),
                };
                track.read_node(node_id).package()
            }
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
            SubstateId::NonFungible(.., id) => Ok(self.non_fungible_get(id)),
            SubstateId::KeyValueStoreEntry(.., key) => Ok(self.kv_store_get(key)),
            SubstateId::NonFungibleSpace(..)
            | SubstateId::Vault(..)
            | SubstateId::KeyValueStoreSpace(..)
            | SubstateId::Package(..)
            | SubstateId::ResourceManager(..)
            | SubstateId::System
            | SubstateId::Bucket(..)
            | SubstateId::Proof(..)
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
            | SubstateId::System
            | SubstateId::Bucket(..)
            | SubstateId::Proof(..)
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
        match self {
            RENodeRefMut::Stack(re_value, id) => {
                re_value
                    .get_node_mut(id.as_ref())
                    .kv_store_mut()
                    .put(key, value);
                for (id, val) in to_store {
                    re_value.insert_non_root_nodes(val.to_nodes(id));
                }
            }
            RENodeRefMut::Track(track, node_id) => {
                let parent_substate_id = match node_id {
                    RENodeId::KeyValueStore(kv_store_id) => {
                        SubstateId::KeyValueStoreSpace(*kv_store_id)
                    }
                    _ => panic!("Unexpeceted"),
                };
                track.set_key_value(
                    parent_substate_id,
                    key,
                    Substate::KeyValueStoreEntry(KeyValueStoreEntrySubstate(Some(value.raw))),
                );
                for (id, val) in to_store {
                    insert_non_root_nodes(track, val.to_nodes(id))?;
                }
            }
        }

        Ok(())
    }

    pub fn kv_store_get(&mut self, key: &[u8]) -> ScryptoValue {
        let wrapper = match self {
            RENodeRefMut::Stack(re_value, id) => {
                let store = re_value.get_node_mut(id.as_ref()).kv_store_mut();
                store
                    .get(key)
                    .map(|v| KeyValueStoreEntrySubstate(Some(v.raw)))
                    .unwrap_or(KeyValueStoreEntrySubstate(None))
            }
            RENodeRefMut::Track(track, node_id) => {
                let parent_substate_id = match node_id {
                    RENodeId::KeyValueStore(kv_store_id) => {
                        SubstateId::KeyValueStoreSpace(*kv_store_id)
                    }
                    _ => panic!("Unexpeceted"),
                };
                let substate_value = track.read_key_value(parent_substate_id, key.to_vec());
                substate_value.into()
            }
        };

        // TODO: Cleanup after adding polymorphism support for SBOR
        // For now, we have to use `Vec<u8>` within `KeyValueStoreEntrySubstate`
        // and apply the following ugly conversion.
        let value = wrapper.0.map_or(
            Value::Option {
                value: Box::new(Option::None),
            },
            |v| Value::Option {
                value: Box::new(Some(
                    decode_any(&v).expect("Failed to decode the value in NonFungibleSubstate"),
                )),
            },
        );

        ScryptoValue::from_value(value)
            .expect("Failed to convert non-fungible value to Scrypto value")
    }

    pub fn non_fungible_get(&mut self, id: &NonFungibleId) -> ScryptoValue {
        let wrapper = match self {
            RENodeRefMut::Stack(value, re_id) => {
                let non_fungible_set = re_id
                    .as_ref()
                    .map_or(value.root(), |v| value.non_root(v))
                    .non_fungibles();
                non_fungible_set
                    .get(id)
                    .cloned()
                    .map(|v| NonFungibleSubstate(Some(v)))
                    .unwrap_or(NonFungibleSubstate(None))
            }
            RENodeRefMut::Track(track, node_id) => {
                let parent_substate_id = match node_id {
                    RENodeId::ResourceManager(resource_address) => {
                        SubstateId::NonFungibleSpace(*resource_address)
                    }
                    _ => panic!("Unexpeceted"),
                };
                let substate_value = track.read_key_value(parent_substate_id, id.to_vec());
                substate_value.into()
            }
        };

        ScryptoValue::from_typed(&wrapper)
    }

    pub fn non_fungible_remove(&mut self, id: &NonFungibleId) {
        match self {
            RENodeRefMut::Stack(..) => {
                panic!("Not supported");
            }
            RENodeRefMut::Track(track, node_id) => {
                let parent_substate_id = match node_id {
                    RENodeId::ResourceManager(resource_address) => {
                        SubstateId::NonFungibleSpace(*resource_address)
                    }
                    _ => panic!("Unexpeceted"),
                };
                track.set_key_value(
                    parent_substate_id,
                    id.to_vec(),
                    Substate::NonFungible(NonFungibleSubstate(None)),
                );
            }
        }
    }

    pub fn non_fungible_put(&mut self, id: NonFungibleId, value: ScryptoValue) {
        match self {
            RENodeRefMut::Stack(re_value, re_id) => {
                let wrapper: NonFungibleSubstate = scrypto_decode(&value.raw)
                    .expect("Attempted to put non-NonFungibleSubstate for non-fungible.");

                let non_fungible_set = re_value.get_node_mut(re_id.as_ref()).non_fungibles_mut();
                if let Some(non_fungible) = wrapper.0 {
                    non_fungible_set.insert(id, non_fungible);
                } else {
                    panic!("TODO: invalidate this code path and possibly consolidate `non_fungible_remove` and `non_fungible_put`")
                }
            }
            RENodeRefMut::Track(track, node_id) => {
                let parent_substate_id = match node_id {
                    RENodeId::ResourceManager(resource_address) => {
                        SubstateId::NonFungibleSpace(*resource_address)
                    }
                    _ => panic!("Unexpeceted"),
                };
                let wrapper: NonFungibleSubstate = scrypto_decode(&value.raw)
                    .expect("Attempted to put non-NonFungibleSubstate for non-fungible.");
                track.set_key_value(
                    parent_substate_id,
                    id.to_vec(),
                    Substate::NonFungible(wrapper),
                );
            }
        }
    }

    pub fn component_state_set(
        &mut self,
        value: ScryptoValue,
        to_store: HashMap<RENodeId, HeapRootRENode>,
    ) {
        match self {
            RENodeRefMut::Stack(re_value, id) => {
                let component = re_value.get_node_mut(id.as_ref()).component_mut();
                component.state.state = value.raw;
                for (id, val) in to_store {
                    re_value.insert_non_root_nodes(val.to_nodes(id));
                }
            }
            RENodeRefMut::Track(track, node_id) => {
                let substate_id = match node_id {
                    RENodeId::Component(component_address) => {
                        SubstateId::ComponentState(*component_address)
                    }
                    _ => panic!("Unexpeceted"),
                };
                *track.write_substate(substate_id).raw_mut() =
                    ComponentStateSubstate::new(value.raw).into();
                for (id, val) in to_store {
                    insert_non_root_nodes(track, val.to_nodes(id));
                }
            }
        }
    }

    pub fn vault_mut(&mut self) -> &mut Vault {
        match self {
            RENodeRefMut::Stack(re_value, id) => re_value.get_node_mut(id.as_ref()).vault_mut(),
            RENodeRefMut::Track(track, node_id) => {
                let substate_id = match node_id {
                    RENodeId::Vault(vault_id) => SubstateId::Vault(*vault_id),
                    _ => panic!("Unexpeceted"),
                };
                track.read_node(node_id).vault_mut()
            }
        }
    }

    pub fn component_mut(&mut self) -> &mut Component {
        match self {
            RENodeRefMut::Stack(re_value, id) => re_value.get_node_mut(id.as_ref()).component_mut(),
            RENodeRefMut::Track(track, node_id) => {
                let substate_id = match node_id {
                    RENodeId::Component(component_id) => SubstateId::ComponentInfo(*component_id),
                    _ => panic!("Unexpeceted"),
                };
                track.read_node(node_id).component_mut()
            }
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

pub fn insert_non_root_nodes<'s, R: FeeReserve>(
    track: &mut Track<'s, R>,
    values: HashMap<RENodeId, HeapRENode>,
) -> Result<(), NodeToSubstateFailure> {
    for (id, node) in values {
        match node {
            HeapRENode::Vault(vault) => {
                let resource = vault
                    .resource()
                    .map_err(|_| NodeToSubstateFailure::VaultPartiallyLocked)?;
                track.create_uuid_substate(
                    SubstateId::Vault(id.into()),
                    VaultSubstate(resource),
                    false,
                );
            }
            HeapRENode::Component(component) => {
                let component_address = id.into();
                track.create_uuid_substate(
                    SubstateId::ComponentInfo(component_address),
                    component.info,
                    false,
                );
                track.create_uuid_substate(
                    SubstateId::ComponentState(component_address),
                    component.state,
                    false,
                );
            }
            HeapRENode::KeyValueStore(store) => {
                let id = id.into();
                let substate_id = SubstateId::KeyValueStoreSpace(id);
                for (k, v) in store.store {
                    track.set_key_value(
                        substate_id.clone(),
                        k,
                        KeyValueStoreEntrySubstate(Some(v.raw)),
                    );
                }
            }
            _ => panic!("Invalid node being persisted: {:?}", node),
        }
    }
    Ok(())
}
