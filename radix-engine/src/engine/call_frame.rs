use sbor::rust::boxed::Box;
use sbor::rust::cell::{RefCell, RefMut};
use sbor::rust::collections::*;
use sbor::rust::format;
use sbor::rust::marker::*;
use sbor::rust::ops::Deref;
use sbor::rust::string::String;
use sbor::rust::string::ToString;
use sbor::rust::vec;
use sbor::rust::vec::Vec;
use sbor::*;
use scrypto::buffer::scrypto_decode;
use scrypto::core::Receiver;
use scrypto::engine::types::*;
use scrypto::prelude::{ScryptoActor, TypeName};
use scrypto::resource::AuthZoneClearInput;
use scrypto::values::*;
use transaction::model::ExecutableInstruction;
use transaction::validation::*;

use crate::engine::*;
use crate::fee::*;
use crate::model::*;
use crate::wasm::*;

/// A call frame is the basic unit that forms a transaction call stack, which keeps track of the
/// owned objects by this function.
pub struct CallFrame<
    'p, // parent lifetime
    'g, // lifetime of values outliving all frames
    's, // Substate store lifetime
    W,  // WASM engine type
    I,  // WASM instance type
    C,  // Cost unit counter type
> where
    W: WasmEngine<I>,
    I: WasmInstance,
    C: CostUnitCounter,
{
    /// The transaction hash
    transaction_hash: Hash,
    /// The call depth
    depth: usize,
    /// The max call depth
    max_depth: usize,
    /// Whether to show trace messages
    trace: bool,

    /// State track
    track: &'g mut Track<'s>,
    /// Wasm engine
    wasm_engine: &'g mut W,
    /// Wasm Instrumenter
    wasm_instrumenter: &'g mut WasmInstrumenter,

    /// Remaining cost unit counter
    cost_unit_counter: &'g mut C,
    /// Fee table
    fee_table: &'g FeeTable,

    id_allocator: &'g mut IdAllocator,
    actor: REActor,

    /// All ref values accessible by this call frame. The value may be located in one of the following:
    /// 1. borrowed values
    /// 2. track
    node_refs: HashMap<RENodeId, RENodePointer>,

    /// Owned Values
    owned_heap_nodes: HashMap<RENodeId, HeapRootRENode>,
    auth_zone: Option<RefCell<AuthZone>>,

    /// Borrowed Values from call frames up the stack
    parent_heap_nodes: Vec<&'p mut HashMap<RENodeId, HeapRootRENode>>,
    caller_auth_zone: Option<&'p RefCell<AuthZone>>,

    phantom: PhantomData<I>,
}

#[macro_export]
macro_rules! trace {
    ( $self: expr, $level: expr, $msg: expr $( , $arg:expr )* ) => {
        #[cfg(not(feature = "alloc"))]
        if $self.trace {
            println!("{}[{:5}] {}", "  ".repeat($self.depth), $level, sbor::rust::format!($msg, $( $arg ),*));
        }
    };
}

fn verify_stored_value_update(
    old: &HashSet<RENodeId>,
    missing: &HashSet<RENodeId>,
) -> Result<(), RuntimeError> {
    // TODO: optimize intersection search
    for old_id in old.iter() {
        if !missing.contains(&old_id) {
            return Err(RuntimeError::StoredNodeRemoved(old_id.clone()));
        }
    }

    for missing_id in missing.iter() {
        if !old.contains(missing_id) {
            return Err(RuntimeError::RENodeNotFound(*missing_id));
        }
    }

    Ok(())
}

pub fn insert_non_root_nodes<'s>(track: &mut Track<'s>, values: HashMap<RENodeId, HeapRENode>) {
    for (id, node) in values {
        match node {
            HeapRENode::Vault(vault) => {
                let addr = SubstateId::Vault(id.into());
                track.create_uuid_substate(addr, vault);
            }
            HeapRENode::Component(component, component_state) => {
                let component_address = id.into();
                track.create_uuid_substate(SubstateId::ComponentInfo(component_address), component);
                track.create_uuid_substate(
                    SubstateId::ComponentState(component_address),
                    component_state,
                );
            }
            HeapRENode::KeyValueStore(store) => {
                let id = id.into();
                let substate_id = SubstateId::KeyValueStoreSpace(id);
                for (k, v) in store.store {
                    track.set_key_value(
                        substate_id.clone(),
                        k,
                        KeyValueStoreEntryWrapper(Some(v.raw)),
                    );
                }
            }
            _ => panic!("Invalid node being persisted: {:?}", node),
        }
    }
}

#[derive(Debug, Clone)]
pub enum RENodePointer {
    Heap {
        frame_id: usize,
        root: RENodeId,
        id: Option<RENodeId>,
    },
    Store(RENodeId),
}

impl RENodePointer {
    fn child(&self, child_id: RENodeId) -> RENodePointer {
        match self {
            RENodePointer::Heap { frame_id, root, .. } => RENodePointer::Heap {
                frame_id: frame_id.clone(),
                root: root.clone(),
                id: Option::Some(child_id),
            },
            RENodePointer::Store(..) => RENodePointer::Store(child_id),
        }
    }

    fn borrow_native_ref<'p, 's>(
        &self, // TODO: Consider changing this to self
        self_frame_id: usize,
        substate_id: SubstateId,
        owned_values: &mut HashMap<RENodeId, HeapRootRENode>,
        borrowed_values: &mut Vec<&'p mut HashMap<RENodeId, HeapRootRENode>>,
        track: &mut Track<'s>,
    ) -> NativeSubstateRef {
        match self {
            RENodePointer::Heap { frame_id, root, id } => {
                let frame = if self_frame_id != *frame_id {
                    borrowed_values.get_mut(*frame_id).unwrap()
                } else {
                    owned_values
                };
                let re_value = frame.remove(root).expect("Should exist");
                NativeSubstateRef::Stack(re_value, frame_id.clone(), root.clone(), id.clone())
            }
            RENodePointer::Store(..) => {
                let value = track.take_substate(substate_id.clone());
                NativeSubstateRef::Track(substate_id.clone(), value)
            }
        }
    }

    fn to_ref<'f, 'p, 's>(
        &self,
        self_frame_id: usize,
        owned_values: &'f HashMap<RENodeId, HeapRootRENode>,
        borrowed_values: &'f Vec<&'p mut HashMap<RENodeId, HeapRootRENode>>,
        track: &'f Track<'s>,
    ) -> RENodeRef<'f, 's> {
        match self {
            RENodePointer::Heap { frame_id, root, id } => {
                let frame = if self_frame_id != *frame_id {
                    borrowed_values.get(*frame_id).unwrap()
                } else {
                    owned_values
                };
                RENodeRef::Stack(frame.get(root).unwrap(), id.clone())
            }
            RENodePointer::Store(node_id) => RENodeRef::Track(track, node_id.clone()),
        }
    }

    fn to_ref_mut<'f, 'p, 's>(
        &self,
        self_frame_id: usize,
        owned_values: &'f mut HashMap<RENodeId, HeapRootRENode>,
        borrowed_values: &'f mut Vec<&'p mut HashMap<RENodeId, HeapRootRENode>>,
        track: &'f mut Track<'s>,
    ) -> RENodeRefMut<'f, 's> {
        match self {
            RENodePointer::Heap { frame_id, root, id } => {
                let frame = if self_frame_id != *frame_id {
                    borrowed_values.get_mut(*frame_id).unwrap()
                } else {
                    owned_values
                };
                RENodeRefMut::Stack(frame.get_mut(root).unwrap(), id.clone())
            }
            RENodePointer::Store(node_id) => RENodeRefMut::Track(track, node_id.clone()),
        }
    }
}

pub enum NativeSubstateRef {
    Stack(HeapRootRENode, usize, RENodeId, Option<RENodeId>),
    Track(SubstateId, Substate),
}

impl NativeSubstateRef {
    pub fn bucket(&mut self) -> &mut Bucket {
        match self {
            NativeSubstateRef::Stack(root, _frame_id, _root_id, maybe_child) => {
                match root.get_node_mut(maybe_child.as_ref()) {
                    HeapRENode::Bucket(bucket) => bucket,
                    _ => panic!("Expecting to be a bucket"),
                }
            }
            _ => panic!("Expecting to be a bucket"),
        }
    }

    pub fn proof(&mut self) -> &mut Proof {
        match self {
            NativeSubstateRef::Stack(ref mut root, _frame_id, _root_id, maybe_child) => {
                match root.get_node_mut(maybe_child.as_ref()) {
                    HeapRENode::Proof(proof) => proof,
                    _ => panic!("Expecting to be a proof"),
                }
            }
            _ => panic!("Expecting to be a proof"),
        }
    }

    pub fn worktop(&mut self) -> &mut Worktop {
        match self {
            NativeSubstateRef::Stack(ref mut root, _frame_id, _root_id, maybe_child) => {
                match root.get_node_mut(maybe_child.as_ref()) {
                    HeapRENode::Worktop(worktop) => worktop,
                    _ => panic!("Expecting to be a worktop"),
                }
            }
            _ => panic!("Expecting to be a worktop"),
        }
    }

    pub fn vault(&mut self) -> &mut Vault {
        match self {
            NativeSubstateRef::Stack(root, _frame_id, _root_id, maybe_child) => {
                root.get_node_mut(maybe_child.as_ref()).vault_mut()
            }
            NativeSubstateRef::Track(_address, value) => value.vault_mut(),
        }
    }

    pub fn system(&mut self) -> &mut System {
        match self {
            NativeSubstateRef::Track(_address, value) => value.system_mut(),
            _ => panic!("Expecting to be system"),
        }
    }

    pub fn component(&mut self) -> &mut Component {
        match self {
            NativeSubstateRef::Stack(root, _frame_id, _root_id, maybe_child) => {
                root.get_node_mut(maybe_child.as_ref()).component_mut()
            }
            _ => panic!("Expecting to be a component"),
        }
    }

    pub fn package(&mut self) -> &ValidatedPackage {
        match self {
            NativeSubstateRef::Track(_address, value) => value.package(),
            _ => panic!("Expecting to be tracked"),
        }
    }

    pub fn resource_manager(&mut self) -> &mut ResourceManager {
        match self {
            NativeSubstateRef::Stack(value, _frame_id, _root_id, maybe_child) => value
                .get_node_mut(maybe_child.as_ref())
                .resource_manager_mut(),
            NativeSubstateRef::Track(_address, value) => value.resource_manager_mut(),
        }
    }

    pub fn return_to_location<'a, 'p, 's>(
        self,
        self_frame_id: usize,
        owned_values: &'a mut HashMap<RENodeId, HeapRootRENode>,
        borrowed_values: &'a mut Vec<&'p mut HashMap<RENodeId, HeapRootRENode>>,
        track: &mut Track<'s>,
    ) {
        match self {
            NativeSubstateRef::Stack(owned, frame_id, node_id, ..) => {
                let frame = if self_frame_id != frame_id {
                    borrowed_values.get_mut(frame_id).unwrap()
                } else {
                    owned_values
                };
                frame.insert(node_id, owned);
            }
            NativeSubstateRef::Track(substate_id, value) => {
                track.write_substate(substate_id, value)
            }
        }
    }
}

pub enum RENodeRef<'f, 's> {
    Stack(&'f HeapRootRENode, Option<RENodeId>),
    Track(&'f Track<'s>, RENodeId),
}

impl<'f, 's> RENodeRef<'f, 's> {
    pub fn vault(&self) -> &Vault {
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
                track.read_substate(substate_id).vault()
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
                track.read_substate(substate_id).system()
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
                track.read_substate(substate_id).resource_manager()
            }
        }
    }

    pub fn component_state(&self) -> &ComponentState {
        match self {
            RENodeRef::Stack(value, id) => id
                .as_ref()
                .map_or(value.root(), |v| value.non_root(v))
                .component_state(),
            RENodeRef::Track(track, node_id) => {
                let substate_id = match node_id {
                    RENodeId::Component(component_address) => {
                        SubstateId::ComponentState(*component_address)
                    }
                    _ => panic!("Unexpected"),
                };
                track.read_substate(substate_id).component_state()
            }
        }
    }

    pub fn component_info(&self) -> &Component {
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
                track.read_substate(substate_id).component()
            }
        }
    }

    pub fn package(&self) -> &ValidatedPackage {
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
                track.read_substate(substate_id).package()
            }
        }
    }
}

pub enum RENodeRefMut<'f, 's> {
    Stack(&'f mut HeapRootRENode, Option<RENodeId>),
    Track(&'f mut Track<'s>, RENodeId),
}

impl<'f, 's> RENodeRefMut<'f, 's> {
    pub fn read_scrypto_value(
        &mut self,
        substate_id: &SubstateId,
    ) -> Result<ScryptoValue, RuntimeError> {
        match substate_id {
            SubstateId::ComponentInfo(..) => Ok(ScryptoValue::from_typed(&self.component().info())),
            SubstateId::ComponentState(..) => {
                Ok(ScryptoValue::from_slice(self.component_state().state())
                    .expect("Expected to decode"))
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
    ) {
        match substate_id {
            SubstateId::ComponentInfo(..) => {
                panic!("Should not get here");
            }
            SubstateId::ComponentState(..) => {
                self.component_state_set(value, child_nodes);
            }
            SubstateId::KeyValueStoreSpace(..) => {
                panic!("Should not get here");
            }
            SubstateId::KeyValueStoreEntry(.., key) => {
                self.kv_store_put(key, value, child_nodes);
            }
            SubstateId::NonFungibleSpace(..) => {
                panic!("Should not get here");
            }
            SubstateId::NonFungible(.., id) => self.non_fungible_put(id, value),
            SubstateId::Vault(..) => {
                panic!("Should not get here");
            }
            SubstateId::Package(..) => {
                panic!("Should not get here");
            }
            SubstateId::ResourceManager(..) => {
                panic!("Should not get here");
            }
            SubstateId::System => {
                panic!("Should not get here");
            }
            SubstateId::Bucket(..) => {
                panic!("Should not get here");
            }
            SubstateId::Proof(..) => {
                panic!("Should not get here");
            }
            SubstateId::Worktop => {
                panic!("Should not get here");
            }
        }
    }

    pub fn kv_store_put(
        &mut self,
        key: Vec<u8>,
        value: ScryptoValue,
        to_store: HashMap<RENodeId, HeapRootRENode>,
    ) {
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
                    Substate::KeyValueStoreEntry(KeyValueStoreEntryWrapper(Some(value.raw))),
                );
                for (id, val) in to_store {
                    insert_non_root_nodes(track, val.to_nodes(id));
                }
            }
        }
    }

    pub fn kv_store_get(&mut self, key: &[u8]) -> ScryptoValue {
        let wrapper = match self {
            RENodeRefMut::Stack(re_value, id) => {
                let store = re_value.get_node_mut(id.as_ref()).kv_store_mut();
                store
                    .get(key)
                    .map(|v| KeyValueStoreEntryWrapper(Some(v.raw)))
                    .unwrap_or(KeyValueStoreEntryWrapper(None))
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
        // For now, we have to use `Vec<u8>` within `KeyValueStoreEntryWrapper`
        // and apply the following ugly conversion.
        let value = wrapper.0.map_or(
            Value::Option {
                value: Box::new(Option::None),
            },
            |v| Value::Option {
                value: Box::new(Some(decode_any(&v).unwrap())),
            },
        );

        ScryptoValue::from_value(value).unwrap()
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
                    .map(|v| NonFungibleWrapper(Some(v)))
                    .unwrap_or(NonFungibleWrapper(None))
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
                    Substate::NonFungible(NonFungibleWrapper(None)),
                );
            }
        }
    }

    pub fn non_fungible_put(&mut self, id: NonFungibleId, value: ScryptoValue) {
        match self {
            RENodeRefMut::Stack(re_value, re_id) => {
                let wrapper: NonFungibleWrapper =
                    scrypto_decode(&value.raw).expect("Should not fail.");

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
                let wrapper: NonFungibleWrapper =
                    scrypto_decode(&value.raw).expect("Should not fail.");
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
                let component_state = re_value.get_node_mut(id.as_ref()).component_state_mut();
                component_state.set_state(value.raw);
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
                track.write_substate(substate_id, ComponentState::new(value.raw));
                for (id, val) in to_store {
                    insert_non_root_nodes(track, val.to_nodes(id));
                }
            }
        }
    }

    pub fn component(&mut self) -> &Component {
        match self {
            RENodeRefMut::Stack(re_value, id) => re_value.get_node_mut(id.as_ref()).component(),
            RENodeRefMut::Track(track, node_id) => {
                let substate_id = match node_id {
                    RENodeId::Component(component_address) => {
                        SubstateId::ComponentInfo(*component_address)
                    }
                    _ => panic!("Unexpeceted"),
                };
                let component_val = track.read_substate(substate_id);
                component_val.component()
            }
        }
    }

    pub fn component_state(&mut self) -> &ComponentState {
        match self {
            RENodeRefMut::Stack(re_value, id) => {
                re_value.get_node_mut(id.as_ref()).component_state()
            }
            RENodeRefMut::Track(track, node_id) => {
                let substate_id = match node_id {
                    RENodeId::Component(component_address) => {
                        SubstateId::ComponentState(*component_address)
                    }
                    _ => panic!("Unexpeceted"),
                };
                let component_val = track.read_substate(substate_id);
                component_val.component_state()
            }
        }
    }
}

pub enum ExecutionEntity<'a> {
    Function(TypeName),
    Method(Receiver, ExecutionState<'a>),
}

pub enum ExecutionState<'a> {
    Consumed(RENodeId),
    AuthZone(RefMut<'a, AuthZone>),
    RENodeRef(RENodeId),
    // TODO: Can remove this and replace useage with REActor
    Component(PackageAddress, String, ComponentAddress),
}

impl<'p, 'g, 's, W, I, C> CallFrame<'p, 'g, 's, W, I, C>
where
    W: WasmEngine<I>,
    I: WasmInstance,
    C: CostUnitCounter,
{
    pub fn new_root(
        verbose: bool,
        transaction_hash: Hash,
        signer_public_keys: Vec<EcdsaPublicKey>,
        is_system: bool,
        max_depth: usize,
        id_allocator: &'g mut IdAllocator,
        track: &'g mut Track<'s>,
        wasm_engine: &'g mut W,
        wasm_instrumenter: &'g mut WasmInstrumenter,
        cost_unit_counter: &'g mut C,
        fee_table: &'g FeeTable,
    ) -> Self {
        // TODO: Cleanup initialization of authzone
        let signer_non_fungible_ids: BTreeSet<NonFungibleId> = signer_public_keys
            .clone()
            .into_iter()
            .map(|public_key| NonFungibleId::from_bytes(public_key.to_vec()))
            .collect();

        let mut initial_auth_zone_proofs = Vec::new();
        if !signer_non_fungible_ids.is_empty() {
            // Proofs can't be zero amount
            let mut ecdsa_bucket = Bucket::new(ResourceContainer::new_non_fungible(
                ECDSA_TOKEN,
                signer_non_fungible_ids,
            ));
            let ecdsa_proof = ecdsa_bucket.create_proof(ECDSA_TOKEN_BUCKET_ID).unwrap();
            initial_auth_zone_proofs.push(ecdsa_proof);
        }

        if is_system {
            let id = [NonFungibleId::from_u32(0)].into_iter().collect();
            let mut system_bucket =
                Bucket::new(ResourceContainer::new_non_fungible(SYSTEM_TOKEN, id));
            let system_proof = system_bucket
                .create_proof(id_allocator.new_bucket_id().unwrap())
                .unwrap();
            initial_auth_zone_proofs.push(system_proof);
        }

        Self::new(
            transaction_hash,
            0,
            max_depth,
            verbose,
            id_allocator,
            track,
            REActor::Native,
            wasm_engine,
            wasm_instrumenter,
            cost_unit_counter,
            fee_table,
            Some(RefCell::new(AuthZone::new_with_proofs(
                initial_auth_zone_proofs,
            ))),
            HashMap::new(),
            HashMap::new(),
            Vec::new(),
            None,
        )
    }

    pub fn new(
        transaction_hash: Hash,
        depth: usize,
        max_depth: usize,
        trace: bool,
        id_allocator: &'g mut IdAllocator,
        track: &'g mut Track<'s>,
        actor: REActor,
        wasm_engine: &'g mut W,
        wasm_instrumenter: &'g mut WasmInstrumenter,
        cost_unit_counter: &'g mut C,
        fee_table: &'g FeeTable,
        auth_zone: Option<RefCell<AuthZone>>,
        owned_heap_nodes: HashMap<RENodeId, HeapRootRENode>,
        node_refs: HashMap<RENodeId, RENodePointer>,
        parent_heap_nodes: Vec<&'p mut HashMap<RENodeId, HeapRootRENode>>,
        caller_auth_zone: Option<&'p RefCell<AuthZone>>,
    ) -> Self {
        Self {
            transaction_hash,
            depth,
            max_depth,
            trace,
            id_allocator,
            track,
            actor,
            wasm_engine,
            wasm_instrumenter,
            cost_unit_counter,
            fee_table,
            owned_heap_nodes,
            node_refs,
            parent_heap_nodes,
            auth_zone,
            caller_auth_zone,
            phantom: PhantomData,
        }
    }

    fn drop_owned_values(&mut self) -> Result<(), RuntimeError> {
        let values = self
            .owned_heap_nodes
            .drain()
            .map(|(_id, value)| value)
            .collect();
        HeapRENode::drop_nodes(values).map_err(|e| RuntimeError::DropFailure(e))
    }

    fn process_call_data(validated: &ScryptoValue) -> Result<(), RuntimeError> {
        if !validated.kv_store_ids.is_empty() {
            return Err(RuntimeError::KeyValueStoreNotAllowed);
        }
        if !validated.vault_ids.is_empty() {
            return Err(RuntimeError::VaultNotAllowed);
        }
        Ok(())
    }

    fn process_return_data(&mut self, validated: &ScryptoValue) -> Result<(), RuntimeError> {
        if !validated.kv_store_ids.is_empty() {
            return Err(RuntimeError::KeyValueStoreNotAllowed);
        }

        // TODO: Should we disallow vaults to be moved?

        Ok(())
    }

    pub fn run(
        &mut self,
        execution_entity: ExecutionEntity<'p>,
        fn_ident: &str,
        input: ScryptoValue,
    ) -> Result<(ScryptoValue, HashMap<RENodeId, HeapRootRENode>), RuntimeError> {
        trace!(
            self,
            Level::Debug,
            "Run started! Depth: {}, Remaining cost units: {}",
            self.depth,
            self.cost_unit_counter.balance()
        );

        match &execution_entity {
            ExecutionEntity::Function(type_name) => {
                self.cost_unit_counter
                    .consume(
                        self.fee_table
                            .run_function_cost(&type_name, fn_ident, &input),
                        "run_function",
                    )
                    .map_err(RuntimeError::CostingError)?;
            }
            ExecutionEntity::Method(receiver, _) => {
                self.cost_unit_counter
                    .consume(
                        self.fee_table.run_method_cost(&receiver, fn_ident, &input),
                        "run_method",
                    )
                    .map_err(RuntimeError::CostingError)?;
            }
        }

        let output = {
            let rtn = match execution_entity {
                ExecutionEntity::Function(type_name) => match type_name {
                    TypeName::TransactionProcessor => TransactionProcessor::static_main(
                        fn_ident, input, self,
                    )
                    .map_err(|e| match e {
                        TransactionProcessorError::InvalidRequestData(_) => panic!("Illegal state"),
                        TransactionProcessorError::InvalidMethod => panic!("Illegal state"),
                        TransactionProcessorError::RuntimeError(e) => e,
                    }),
                    TypeName::Package => ValidatedPackage::static_main(fn_ident, input, self)
                        .map_err(RuntimeError::PackageError),
                    TypeName::ResourceManager => {
                        ResourceManager::static_main(fn_ident, input, self)
                            .map_err(RuntimeError::ResourceManagerError)
                    }
                    TypeName::Blueprint(package_address, blueprint_name) => {
                        let output = {
                            let package = self
                                .track
                                .read_substate(SubstateId::Package(package_address))
                                .package();
                            let wasm_metering_params = self.fee_table.wasm_metering_params();
                            let instrumented_code = self
                                .wasm_instrumenter
                                .instrument(package.code(), &wasm_metering_params);
                            let mut instance = self.wasm_engine.instantiate(instrumented_code);
                            let blueprint_abi = package
                                .blueprint_abi(&blueprint_name)
                                .expect("Blueprint should exist");
                            let export_name = &blueprint_abi
                                .get_fn_abi(fn_ident)
                                .unwrap()
                                .export_name
                                .to_string();
                            let mut runtime: Box<dyn WasmRuntime> =
                                Box::new(RadixEngineWasmRuntime::new(
                                    ScryptoActor::blueprint(
                                        package_address,
                                        blueprint_name.clone(),
                                    ),
                                    self,
                                ));
                            instance
                                .invoke_export(&export_name, &input, &mut runtime)
                                .map_err(|e| match e {
                                    // Flatten error code for more readable transaction receipt
                                    InvokeError::RuntimeError(e) => e,
                                    e @ _ => RuntimeError::InvokeError(e.into()),
                                })?
                        };

                        let package = self
                            .track
                            .read_substate(SubstateId::Package(package_address))
                            .package();
                        let blueprint_abi = package
                            .blueprint_abi(&blueprint_name)
                            .expect("Blueprint should exist");
                        let fn_abi = blueprint_abi.get_fn_abi(fn_ident).unwrap();
                        if !fn_abi.output.matches(&output.dom) {
                            Err(RuntimeError::InvalidFnOutput {
                                fn_ident: fn_ident.to_string(),
                                output: output.dom,
                            })
                        } else {
                            Ok(output)
                        }
                    }
                },
                ExecutionEntity::Method(_, state) => match state {
                    ExecutionState::Consumed(node_id) => match node_id {
                        RENodeId::Bucket(..) => {
                            Bucket::consuming_main(node_id, fn_ident, input, self)
                                .map_err(RuntimeError::BucketError)
                        }
                        RENodeId::Proof(..) => Proof::main_consume(node_id, fn_ident, input, self)
                            .map_err(RuntimeError::ProofError),
                        _ => panic!("Unexpected"),
                    },
                    ExecutionState::AuthZone(mut auth_zone) => auth_zone
                        .main(fn_ident, input, self)
                        .map_err(RuntimeError::AuthZoneError),
                    ExecutionState::RENodeRef(node_id) => match node_id {
                        RENodeId::Bucket(bucket_id) => {
                            Bucket::main(bucket_id, fn_ident, input, self)
                                .map_err(RuntimeError::BucketError)
                        }
                        RENodeId::Proof(proof_id) => Proof::main(proof_id, fn_ident, input, self)
                            .map_err(RuntimeError::ProofError),
                        RENodeId::Worktop => {
                            Worktop::main(fn_ident, input, self).map_err(RuntimeError::WorktopError)
                        }
                        RENodeId::Vault(vault_id) => Vault::main(vault_id, fn_ident, input, self)
                            .map_err(RuntimeError::VaultError),
                        RENodeId::Component(component_address) => {
                            Component::main(component_address, fn_ident, input, self)
                                .map_err(RuntimeError::ComponentError)
                        }
                        RENodeId::ResourceManager(resource_address) => {
                            ResourceManager::main(resource_address, fn_ident, input, self)
                                .map_err(RuntimeError::ResourceManagerError)
                        }
                        RENodeId::System => {
                            System::main(fn_ident, input, self).map_err(RuntimeError::SystemError)
                        }
                        _ => panic!("Unexpected"),
                    },
                    ExecutionState::Component(
                        package_address,
                        blueprint_name,
                        component_address,
                    ) => {
                        let output = {
                            let package = self
                                .track
                                .read_substate(SubstateId::Package(package_address))
                                .package();
                            let wasm_metering_params = self.fee_table.wasm_metering_params();
                            let instrumented_code = self
                                .wasm_instrumenter
                                .instrument(package.code(), &wasm_metering_params);
                            let mut instance = self.wasm_engine.instantiate(instrumented_code);
                            let blueprint_abi = package
                                .blueprint_abi(&blueprint_name)
                                .expect("Blueprint should exist");
                            let export_name = &blueprint_abi
                                .get_fn_abi(fn_ident)
                                .unwrap()
                                .export_name
                                .to_string();
                            let mut runtime: Box<dyn WasmRuntime> =
                                Box::new(RadixEngineWasmRuntime::new(
                                    ScryptoActor::Component(
                                        component_address,
                                        package_address.clone(),
                                        blueprint_name.clone(),
                                    ),
                                    self,
                                ));
                            instance
                                .invoke_export(&export_name, &input, &mut runtime)
                                .map_err(|e| match e {
                                    // Flatten error code for more readable transaction receipt
                                    InvokeError::RuntimeError(e) => e,
                                    e @ _ => RuntimeError::InvokeError(e.into()),
                                })?
                        };

                        let package = self
                            .track
                            .read_substate(SubstateId::Package(package_address))
                            .package();
                        let blueprint_abi = package
                            .blueprint_abi(&blueprint_name)
                            .expect("Blueprint should exist");
                        let fn_abi = blueprint_abi.get_fn_abi(fn_ident).unwrap();
                        if !fn_abi.output.matches(&output.dom) {
                            Err(RuntimeError::InvalidFnOutput {
                                fn_ident: fn_ident.to_string(),
                                output: output.dom,
                            })
                        } else {
                            Ok(output)
                        }
                    }
                },
            }?;

            rtn
        };

        // Prevent vaults/kvstores from being returned
        self.process_return_data(&output)?;

        // Take values to return
        let values_to_take = output.node_ids();
        let (taken_values, mut missing) = self.take_available_values(values_to_take, false)?;
        let first_missing_value = missing.drain().nth(0);
        if let Some(missing_node) = first_missing_value {
            return Err(RuntimeError::RENodeNotFound(missing_node));
        }

        // Check we have valid references to pass back
        for refed_component_address in &output.refed_component_addresses {
            let node_id = RENodeId::Component(*refed_component_address);
            if let Some(RENodePointer::Store(..)) = self.node_refs.get(&node_id) {
                // Only allow passing back global references
            } else {
                return Err(RuntimeError::InvokeMethodInvalidReferencePass(node_id));
            }
        }

        // drop proofs and check resource leak
        if self.auth_zone.is_some() {
            self.invoke_method(
                Receiver::AuthZoneRef,
                "clear".to_string(),
                ScryptoValue::from_typed(&AuthZoneClearInput {}),
            )?;
        }
        self.drop_owned_values()?;

        trace!(
            self,
            Level::Debug,
            "Run finished! Remaining cost units: {}",
            self.cost_unit_counter().balance()
        );

        Ok((output, taken_values))
    }

    fn take_available_values(
        &mut self,
        node_ids: HashSet<RENodeId>,
        persist_only: bool,
    ) -> Result<(HashMap<RENodeId, HeapRootRENode>, HashSet<RENodeId>), RuntimeError> {
        let (taken, missing) = {
            let mut taken_values = HashMap::new();
            let mut missing_values = HashSet::new();

            for id in node_ids {
                let maybe = self.owned_heap_nodes.remove(&id);
                if let Some(value) = maybe {
                    value.root().verify_can_move()?;
                    if persist_only {
                        value.root().verify_can_persist()?;
                    }
                    taken_values.insert(id, value);
                } else {
                    missing_values.insert(id);
                }
            }

            (taken_values, missing_values)
        };

        // Moved values must have their references removed
        for (id, value) in &taken {
            self.node_refs.remove(id);
            for (id, ..) in &value.child_nodes {
                self.node_refs.remove(id);
            }
        }

        Ok((taken, missing))
    }

    fn read_value_internal(
        &mut self,
        substate_id: &SubstateId,
    ) -> Result<(RENodePointer, ScryptoValue), RuntimeError> {
        let node_id = SubstateProperties::get_node_id(substate_id);

        // Get location
        // Note this must be run AFTER values are taken, otherwise there would be inconsistent readable_values state
        let node_pointer = self
            .node_refs
            .get(&node_id)
            .cloned()
            .ok_or_else(|| RuntimeError::SubstateReadSubstateNotFound(substate_id.clone()))?;

        if matches!(substate_id, SubstateId::ComponentInfo(..))
            && matches!(node_pointer, RENodePointer::Store(..))
        {
            self.track
                .acquire_lock(substate_id.clone(), false, false)
                .expect("Should never fail");
        }

        // Read current value
        let current_value = {
            let mut node_ref = node_pointer.to_ref_mut(
                self.depth,
                &mut self.owned_heap_nodes,
                &mut self.parent_heap_nodes,
                &mut self.track,
            );
            node_ref.read_scrypto_value(&substate_id)?
        };

        // TODO: Remove, integrate with substate borrow mechanism
        if matches!(substate_id, SubstateId::ComponentInfo(..))
            && matches!(node_pointer, RENodePointer::Store(..))
        {
            self.track.release_lock(substate_id.clone(), false);
        }

        Ok((node_pointer.clone(), current_value))
    }

    /// Creates a new UUID.
    fn new_uuid(&mut self) -> u128 {
        self.id_allocator.new_uuid(self.transaction_hash).unwrap()
    }

    fn new_node_id(&mut self, re_node: &HeapRENode) -> RENodeId {
        match re_node {
            HeapRENode::Bucket(..) => {
                let bucket_id = self.id_allocator.new_bucket_id().unwrap();
                RENodeId::Bucket(bucket_id)
            }
            HeapRENode::Proof(..) => {
                let proof_id = self.id_allocator.new_proof_id().unwrap();
                RENodeId::Proof(proof_id)
            }
            HeapRENode::Worktop(..) => RENodeId::Worktop,
            HeapRENode::Vault(..) => {
                let vault_id = self
                    .id_allocator
                    .new_vault_id(self.transaction_hash)
                    .unwrap();
                RENodeId::Vault(vault_id)
            }
            HeapRENode::KeyValueStore(..) => {
                let kv_store_id = self
                    .id_allocator
                    .new_kv_store_id(self.transaction_hash)
                    .unwrap();
                RENodeId::KeyValueStore(kv_store_id)
            }
            HeapRENode::Package(..) => {
                // Security Alert: ensure ID allocating will practically never fail
                let package_address = self
                    .id_allocator
                    .new_package_address(self.transaction_hash)
                    .unwrap();
                RENodeId::Package(package_address)
            }
            HeapRENode::Resource(..) => {
                let resource_address = self
                    .id_allocator
                    .new_resource_address(self.transaction_hash)
                    .unwrap();
                RENodeId::ResourceManager(resource_address)
            }
            HeapRENode::Component(ref component, ..) => {
                let component_address = self
                    .id_allocator
                    .new_component_address(
                        self.transaction_hash,
                        &component.package_address(),
                        component.blueprint_name(),
                    )
                    .unwrap();
                RENodeId::Component(component_address)
            }
            HeapRENode::System(..) => {
                panic!("Should not get here.");
            }
        }
    }
}

impl<'p, 'g, 's, W, I, C> SystemApi<'p, 's, W, I, C> for CallFrame<'p, 'g, 's, W, I, C>
where
    W: WasmEngine<I>,
    I: WasmInstance,
    C: CostUnitCounter,
{
    fn invoke_function(
        &mut self,
        type_name: TypeName,
        fn_ident: String,
        input: ScryptoValue,
    ) -> Result<ScryptoValue, RuntimeError> {
        trace!(
            self,
            Level::Debug,
            "Invoking function: {:?} {:?}",
            type_name,
            &fn_ident
        );

        if self.depth == self.max_depth {
            return Err(RuntimeError::MaxCallDepthLimitReached);
        }

        self.cost_unit_counter
            .consume(
                self.fee_table
                    .system_api_cost(SystemApiCostingEntry::InvokeFunction {
                        type_name: type_name.clone(),
                        input: &input,
                    }),
                "invoke_function",
            )
            .map_err(RuntimeError::CostingError)?;

        // Prevent vaults/kvstores from being moved
        Self::process_call_data(&input)?;

        // Figure out what buckets and proofs to move from this process
        let values_to_take = input.node_ids();
        let (taken_values, mut missing) = self.take_available_values(values_to_take, false)?;
        let first_missing_value = missing.drain().nth(0);
        if let Some(missing_value) = first_missing_value {
            return Err(RuntimeError::RENodeNotFound(missing_value));
        }

        let mut next_owned_values = HashMap::new();

        // Internal state update to taken values
        for (id, mut value) in taken_values {
            trace!(self, Level::Debug, "Sending value: {:?}", value);
            match &mut value.root_mut() {
                HeapRENode::Proof(proof) => proof.change_to_restricted(),
                _ => {}
            }
            next_owned_values.insert(id, value);
        }

        let mut locked_values = HashSet::<SubstateId>::new();

        // No authorization but state load
        let actor = match &type_name {
            TypeName::Blueprint(package_address, blueprint_name) => {
                self.track
                    .acquire_lock(SubstateId::Package(package_address.clone()), false, false)
                    .map_err(|e| match e {
                        TrackError::NotFound => RuntimeError::PackageNotFound(*package_address),
                        TrackError::Reentrancy => {
                            panic!("Package reentrancy error should never occur.")
                        }
                        TrackError::StateTrackError(..) => panic!("Unexpected"),
                    })?;
                locked_values.insert(SubstateId::Package(package_address.clone()));
                let package = self
                    .track
                    .read_substate(SubstateId::Package(package_address.clone()))
                    .package();
                let abi = package.blueprint_abi(blueprint_name).ok_or(
                    RuntimeError::BlueprintNotFound(
                        package_address.clone(),
                        blueprint_name.clone(),
                    ),
                )?;
                let fn_abi = abi
                    .get_fn_abi(&fn_ident)
                    .ok_or(RuntimeError::MethodDoesNotExist(fn_ident.clone()))?;
                if !fn_abi.input.matches(&input.dom) {
                    return Err(RuntimeError::InvalidFnInput {
                        fn_ident,
                        input: input.dom,
                    });
                }

                REActor::Scrypto(ScryptoActor::blueprint(
                    *package_address,
                    blueprint_name.clone(),
                ))
            }
            TypeName::Package | TypeName::ResourceManager | TypeName::TransactionProcessor => {
                REActor::Native
            }
        };

        // Move this into transaction processor
        let mut next_frame_node_refs = HashMap::new();
        if self.depth == 0 {
            let mut component_addresses = HashSet::new();

            // Collect component addresses
            for component_address in &input.refed_component_addresses {
                component_addresses.insert(*component_address);
            }
            let input: TransactionProcessorRunInput = scrypto_decode(&input.raw).unwrap();
            for instruction in &input.instructions {
                match instruction {
                    ExecutableInstruction::CallFunction { arg, .. }
                    | ExecutableInstruction::CallMethod { arg, .. } => {
                        let scrypto_value = ScryptoValue::from_slice(&arg).unwrap();
                        component_addresses.extend(scrypto_value.refed_component_addresses);
                    }
                    _ => {}
                }
            }

            // Make components visible
            for component_address in component_addresses {
                // TODO: Check if component exists
                let node_id = RENodeId::Component(component_address);
                next_frame_node_refs.insert(node_id, RENodePointer::Store(node_id));
            }
        } else {
            // Pass argument references
            for refed_component_address in &input.refed_component_addresses {
                let node_id = RENodeId::Component(refed_component_address.clone());
                if let Some(pointer) = self.node_refs.get(&node_id) {
                    let mut visible = HashSet::new();
                    visible.insert(SubstateId::ComponentInfo(*refed_component_address));
                    next_frame_node_refs.insert(node_id.clone(), pointer.clone());
                } else {
                    return Err(RuntimeError::InvokeMethodInvalidReferencePass(node_id));
                }
            }
        }

        // Setup next parent frame
        let mut next_borrowed_values: Vec<&mut HashMap<RENodeId, HeapRootRENode>> = Vec::new();
        for parent_values in &mut self.parent_heap_nodes {
            next_borrowed_values.push(parent_values);
        }
        next_borrowed_values.push(&mut self.owned_heap_nodes);

        // start a new frame
        let (result, received_values) = {
            let mut frame = CallFrame::new(
                self.transaction_hash,
                self.depth + 1,
                self.max_depth,
                self.trace,
                self.id_allocator,
                self.track,
                actor,
                self.wasm_engine,
                self.wasm_instrumenter,
                self.cost_unit_counter,
                self.fee_table,
                match type_name {
                    TypeName::TransactionProcessor | TypeName::Blueprint(_, _) => {
                        Some(RefCell::new(AuthZone::new()))
                    }
                    _ => None,
                },
                next_owned_values,
                next_frame_node_refs,
                next_borrowed_values,
                self.auth_zone.as_ref(),
            );

            // invoke the main function
            frame.run(ExecutionEntity::Function(type_name), &fn_ident, input)?
        };

        // Release locked addresses
        for l in locked_values {
            // TODO: refactor after introducing `Lock` representation.
            self.track.release_lock(l.clone(), false);
        }

        // move buckets and proofs to this process.
        for (id, value) in received_values {
            trace!(self, Level::Debug, "Received value: {:?}", value);
            self.owned_heap_nodes.insert(id, value);
        }

        // Accept component references
        for refed_component_address in &result.refed_component_addresses {
            let node_id = RENodeId::Component(*refed_component_address);
            let mut visible = HashSet::new();
            visible.insert(SubstateId::ComponentInfo(*refed_component_address));
            self.node_refs
                .insert(node_id, RENodePointer::Store(node_id));
        }

        trace!(self, Level::Debug, "Invoking finished!");
        Ok(result)
    }

    fn invoke_method(
        &mut self,
        receiver: Receiver,
        fn_ident: String,
        input: ScryptoValue,
    ) -> Result<ScryptoValue, RuntimeError> {
        trace!(
            self,
            Level::Debug,
            "Invoking method: {:?} {:?}",
            receiver,
            &fn_ident
        );

        if self.depth == self.max_depth {
            return Err(RuntimeError::MaxCallDepthLimitReached);
        }

        self.cost_unit_counter
            .consume(
                self.fee_table
                    .system_api_cost(SystemApiCostingEntry::InvokeMethod {
                        receiver: receiver.clone(),
                        input: &input,
                    }),
                "invoke_method",
            )
            .map_err(RuntimeError::CostingError)?;

        // TODO: find a better way to handle this
        let is_lock_fee = matches!(receiver, Receiver::VaultRef(..)) && &fn_ident == "lock_fee";

        // Prevent vaults/kvstores from being moved
        Self::process_call_data(&input)?;

        // Figure out what buckets and proofs to move from this process
        let values_to_take = input.node_ids();
        let (taken_values, mut missing) = self.take_available_values(values_to_take, false)?;
        let first_missing_value = missing.drain().nth(0);
        if let Some(missing_value) = first_missing_value {
            return Err(RuntimeError::RENodeNotFound(missing_value));
        }

        let mut next_owned_values = HashMap::new();

        // Internal state update to taken values
        for (id, mut value) in taken_values {
            trace!(self, Level::Debug, "Sending value: {:?}", value);
            match &mut value.root_mut() {
                HeapRENode::Proof(proof) => proof.change_to_restricted(),
                _ => {}
            }
            next_owned_values.insert(id, value);
        }

        let mut locked_values = HashSet::new();
        let mut next_frame_node_refs = HashMap::new();

        // Authorization and state load
        let (actor, execution_state, method_auths) = match &receiver {
            Receiver::Consumed(node_id) => {
                let value = self
                    .owned_heap_nodes
                    .remove(node_id)
                    .ok_or(RuntimeError::RENodeNotFound(*node_id))?;

                let method_auths = match &value.root() {
                    HeapRENode::Bucket(bucket) => {
                        let resource_address = bucket.resource_address();
                        self.track
                            .acquire_lock(
                                SubstateId::ResourceManager(resource_address),
                                true,
                                false,
                            )
                            .expect("Should not fail.");
                        locked_values.insert(SubstateId::ResourceManager(resource_address.clone()));
                        let resource_manager = self
                            .track
                            .read_substate(SubstateId::ResourceManager(resource_address))
                            .resource_manager();
                        let method_auth = resource_manager.get_consuming_bucket_auth(&fn_ident);
                        let node_id = RENodeId::ResourceManager(resource_address);
                        next_frame_node_refs.insert(node_id, RENodePointer::Store(node_id));
                        vec![method_auth.clone()]
                    }
                    HeapRENode::Proof(_) => vec![],
                    _ => return Err(RuntimeError::MethodDoesNotExist(fn_ident.clone())),
                };

                next_owned_values.insert(*node_id, value);

                Ok((
                    REActor::Native,
                    ExecutionState::Consumed(*node_id),
                    method_auths,
                ))
            }
            Receiver::AuthZoneRef => {
                if let Some(auth_zone) = &self.auth_zone {
                    for resource_address in &input.resource_addresses {
                        self.track
                            .acquire_lock(
                                SubstateId::ResourceManager(resource_address.clone()),
                                false,
                                false,
                            )
                            .map_err(|e| match e {
                                TrackError::NotFound => {
                                    RuntimeError::ResourceManagerNotFound(resource_address.clone())
                                }
                                TrackError::Reentrancy => {
                                    panic!("Package reentrancy error should never occur.")
                                }
                                TrackError::StateTrackError(..) => panic!("Unexpected"),
                            })?;
                        locked_values.insert(SubstateId::ResourceManager(resource_address.clone()));
                        let node_id = RENodeId::ResourceManager(resource_address.clone());
                        next_frame_node_refs.insert(node_id, RENodePointer::Store(node_id));
                    }
                    let borrowed = auth_zone.borrow_mut();
                    Ok((REActor::Native, ExecutionState::AuthZone(borrowed), vec![]))
                } else {
                    Err(RuntimeError::AuthZoneDoesNotExist)
                }
            }
            Receiver::NativeRENodeRef(node_id) => {
                let native_substate_id = match node_id {
                    RENodeId::Bucket(bucket_id) => SubstateId::Bucket(*bucket_id),
                    RENodeId::Proof(proof_id) => SubstateId::Proof(*proof_id),
                    RENodeId::ResourceManager(resource_address) => {
                        SubstateId::ResourceManager(*resource_address)
                    }
                    RENodeId::System => SubstateId::System,
                    RENodeId::Worktop => SubstateId::Worktop,
                    _ => return Err(RuntimeError::MethodDoesNotExist(fn_ident.clone())),
                };

                let next_node_pointer = if self.owned_heap_nodes.contains_key(&node_id) {
                    RENodePointer::Heap {
                        frame_id: self.depth,
                        root: node_id.clone(),
                        id: None,
                    }
                } else if let Some(pointer) = self.node_refs.get(&node_id) {
                    pointer.clone()
                } else {
                    match node_id {
                        // Let these be globally accessible for now
                        // TODO: Remove when references cleaned up
                        RENodeId::ResourceManager(..) | RENodeId::System => {
                            RENodePointer::Store(*node_id)
                        }
                        _ => return Err(RuntimeError::InvokeMethodInvalidReceiver(*node_id)),
                    }
                };
                next_frame_node_refs.insert(node_id.clone(), next_node_pointer.clone());

                // Lock Substate
                match next_node_pointer {
                    RENodePointer::Store(..) => {
                        self.track
                            .acquire_lock(native_substate_id.clone(), true, false)
                            .map_err(|e| match e {
                                TrackError::StateTrackError(..) => {
                                    panic!("Unexpected")
                                }
                                // TODO: Remove when references cleaned up
                                TrackError::NotFound => RuntimeError::RENodeNotFound(*node_id),
                                TrackError::Reentrancy => {
                                    RuntimeError::Reentrancy(native_substate_id.clone())
                                }
                            })?;
                        locked_values.insert(native_substate_id.clone());
                    }
                    RENodePointer::Heap { .. } => {}
                }

                // Lock Resource Managers in request
                // TODO: Remove when references cleaned up
                for resource_address in &input.resource_addresses {
                    let resource_substate_id =
                        SubstateId::ResourceManager(resource_address.clone());
                    let node_id = RENodeId::ResourceManager(resource_address.clone());
                    self.track
                        .acquire_lock(resource_substate_id.clone(), false, false)
                        .map_err(|e| match e {
                            TrackError::NotFound => RuntimeError::RENodeNotFound(node_id),
                            TrackError::Reentrancy => {
                                RuntimeError::Reentrancy(resource_substate_id)
                            }
                            TrackError::StateTrackError(..) => panic!("Unexpected"),
                        })?;

                    locked_values.insert(SubstateId::ResourceManager(resource_address.clone()));
                    next_frame_node_refs.insert(node_id, RENodePointer::Store(node_id));
                }

                // Load method authorization
                let method_auth = match node_id {
                    RENodeId::ResourceManager(..) => {
                        let resource_manager = self
                            .track
                            .read_substate(native_substate_id)
                            .resource_manager();
                        let method_auth = resource_manager.get_auth(&fn_ident, &input).clone();
                        vec![method_auth]
                    }
                    RENodeId::System => {
                        let fn_str: &str = &fn_ident;
                        match fn_str {
                            "set_epoch" => {
                                vec![MethodAuthorization::Protected(HardAuthRule::ProofRule(
                                    HardProofRule::Require(HardResourceOrNonFungible::Resource(
                                        SYSTEM_TOKEN,
                                    )),
                                ))]
                            }
                            _ => vec![],
                        }
                    }
                    _ => vec![],
                };

                Ok((
                    REActor::Native,
                    ExecutionState::RENodeRef(*node_id),
                    method_auth,
                ))
            }
            Receiver::Component(component_address) => {
                let component_address = component_address.clone();

                // Find value
                let node_id = RENodeId::Component(component_address);
                let cur_pointer = if self.owned_heap_nodes.contains_key(&node_id) {
                    RENodePointer::Heap {
                        frame_id: self.depth,
                        root: node_id.clone(),
                        id: None,
                    }
                } else if let Some(pointer) = self.node_refs.get(&node_id) {
                    pointer.clone()
                } else {
                    return Err(RuntimeError::InvokeMethodInvalidReceiver(node_id));
                };

                // Lock values and setup next frame
                let next_pointer = match cur_pointer.clone() {
                    RENodePointer::Store(node_id) => {
                        let substate_id = SubstateId::ComponentState(component_address);
                        self.track
                            .acquire_lock(substate_id.clone(), true, false)
                            .map_err(|e| match e {
                                TrackError::NotFound => {
                                    RuntimeError::ComponentNotFound(component_address)
                                }
                                TrackError::Reentrancy => {
                                    RuntimeError::ComponentReentrancy(component_address)
                                }
                                TrackError::StateTrackError(..) => {
                                    panic!("Unexpected")
                                }
                            })?;
                        locked_values.insert(substate_id.clone());
                        RENodePointer::Store(node_id)
                    }
                    RENodePointer::Heap { frame_id, root, id } => RENodePointer::Heap {
                        frame_id,
                        root,
                        id,
                    },
                };

                match cur_pointer {
                    RENodePointer::Store(..) => {
                        self.track
                            .acquire_lock(
                                SubstateId::ComponentInfo(component_address),
                                false,
                                false,
                            )
                            .expect("Component Info should not be locked for long periods of time");
                    }
                    _ => {}
                }

                let scrypto_actor = {
                    let node_ref = cur_pointer.to_ref(
                        self.depth,
                        &self.owned_heap_nodes,
                        &self.parent_heap_nodes,
                        &mut self.track,
                    );
                    let component = node_ref.component_info();
                    ScryptoActor::component(
                        component_address,
                        component.package_address(),
                        component.blueprint_name().to_string(),
                    )
                };

                // Retrieve Method Authorization
                let method_auths = {
                    let package_substate_id =
                        SubstateId::Package(scrypto_actor.package_address().clone());
                    self.track
                        .acquire_lock(package_substate_id.clone(), false, false)
                        .expect("Should never fail");
                    locked_values.insert(package_substate_id.clone());
                    let package = self
                        .track
                        .read_substate(package_substate_id.clone())
                        .package();
                    let abi = package
                        .blueprint_abi(scrypto_actor.blueprint_name())
                        .expect("Blueprint not found for existing component");
                    let fn_abi = abi
                        .get_fn_abi(&fn_ident)
                        .ok_or(RuntimeError::MethodDoesNotExist(fn_ident.clone()))?;
                    if !fn_abi.input.matches(&input.dom) {
                        return Err(RuntimeError::InvalidFnInput {
                            fn_ident,
                            input: input.dom,
                        });
                    }

                    {
                        let value_ref = cur_pointer.to_ref(
                            self.depth,
                            &self.owned_heap_nodes,
                            &self.parent_heap_nodes,
                            &self.track,
                        );

                        let component = value_ref.component_info();
                        let component_state = value_ref.component_state();
                        component.method_authorization(component_state, &abi.structure, &fn_ident)
                    }
                };

                match cur_pointer {
                    RENodePointer::Store(..) => {
                        self.track
                            .release_lock(SubstateId::ComponentInfo(component_address), false);
                    }
                    _ => {}
                }

                next_frame_node_refs.insert(node_id, next_pointer);

                let execution_state = ExecutionState::Component(
                    scrypto_actor.package_address().clone(),
                    scrypto_actor.blueprint_name().clone(),
                    component_address,
                );
                Ok((
                    REActor::Scrypto(scrypto_actor),
                    execution_state,
                    method_auths,
                ))
            }
            Receiver::ComponentMetaRef(component_address) => {
                let component_address = *component_address;

                // Find value
                let node_id = RENodeId::Component(component_address);
                let cur_pointer = if self.owned_heap_nodes.contains_key(&node_id) {
                    RENodePointer::Heap {
                        frame_id: self.depth,
                        root: node_id.clone(),
                        id: None,
                    }
                } else {
                    return Err(RuntimeError::NotSupported);
                };

                // Setup next frame
                match cur_pointer {
                    RENodePointer::Heap {
                        frame_id: _,
                        root,
                        id,
                    } => {
                        let owned_ref = self.owned_heap_nodes.get_mut(&root).unwrap();

                        // Lock package
                        let package_address = owned_ref.root().component().package_address();
                        self.track
                            .acquire_lock(SubstateId::Package(package_address), false, false)
                            .map_err(|e| match e {
                                TrackError::NotFound => panic!("Should exist"),
                                TrackError::Reentrancy => RuntimeError::PackageReentrancy,
                                TrackError::StateTrackError(..) => panic!("Unexpected"),
                            })?;
                        locked_values.insert(SubstateId::Package(package_address));
                        next_frame_node_refs.insert(
                            RENodeId::Package(package_address),
                            RENodePointer::Store(RENodeId::Package(package_address)),
                        );

                        next_frame_node_refs.insert(
                            node_id,
                            RENodePointer::Heap {
                                frame_id: self.depth,
                                root: root.clone(),
                                id: id.clone(),
                            },
                        );
                    }
                    _ => panic!("Unexpected"),
                }

                Ok((REActor::Native, ExecutionState::RENodeRef(node_id), vec![]))
            }

            Receiver::VaultRef(vault_id) => {
                // Find value
                let node_id = RENodeId::Vault(*vault_id);
                let cur_pointer = if self.owned_heap_nodes.contains_key(&node_id) {
                    RENodePointer::Heap {
                        frame_id: self.depth,
                        root: node_id.clone(),
                        id: Option::None,
                    }
                } else {
                    let maybe_pointer = self.node_refs.get(&node_id);
                    maybe_pointer
                        .cloned()
                        .ok_or(RuntimeError::RENodeNotFound(RENodeId::Vault(*vault_id)))?
                };
                if is_lock_fee && !matches!(cur_pointer, RENodePointer::Store { .. }) {
                    return Err(RuntimeError::LockFeeError(LockFeeError::RENodeNotInTrack));
                }

                // Lock values and setup next frame
                let next_pointer = {
                    // Lock Vault
                    let next_pointer = match cur_pointer.clone() {
                        RENodePointer::Store(RENodeId::Vault(vault_id)) => {
                            let substate_id = SubstateId::Vault(vault_id);
                            self.track
                                .acquire_lock(substate_id.clone(), true, is_lock_fee)
                                .map_err(|e| match e {
                                    TrackError::NotFound | TrackError::Reentrancy => {
                                        panic!("Illegal state")
                                    }
                                    TrackError::StateTrackError(e) => {
                                        RuntimeError::LockFeeError(match e {
                                            StateTrackError::RENodeAlreadyTouched => {
                                                LockFeeError::RENodeAlreadyTouched
                                            }
                                        })
                                    }
                                })?;
                            locked_values.insert(substate_id);
                            RENodePointer::Store(node_id)
                        }
                        RENodePointer::Heap { frame_id, root, id } => RENodePointer::Heap {
                            frame_id: frame_id,
                            root,
                            id,
                        },
                        _ => panic!("Unexpected pointer"),
                    };

                    // Lock Resource
                    let resource_address = {
                        let value_ref = cur_pointer.to_ref(
                            self.depth,
                            &mut self.owned_heap_nodes,
                            &mut self.parent_heap_nodes,
                            &mut self.track,
                        );
                        value_ref.vault().resource_address()
                    };
                    self.track
                        .acquire_lock(SubstateId::ResourceManager(resource_address), true, false)
                        .expect("Should never fail.");
                    locked_values.insert(SubstateId::ResourceManager(resource_address));

                    next_pointer
                };

                // Retrieve Method Authorization
                let method_auth = {
                    let resource_address = {
                        let value_ref = cur_pointer.to_ref(
                            self.depth,
                            &mut self.owned_heap_nodes,
                            &mut self.parent_heap_nodes,
                            &mut self.track,
                        );
                        value_ref.vault().resource_address()
                    };
                    let resource_manager = self
                        .track
                        .read_substate(SubstateId::ResourceManager(resource_address))
                        .resource_manager();
                    resource_manager.get_vault_auth(&fn_ident).clone()
                };

                next_frame_node_refs.insert(node_id.clone(), next_pointer);

                Ok((
                    REActor::Native,
                    ExecutionState::RENodeRef(node_id),
                    vec![method_auth],
                ))
            }
        }?;

        // Pass argument references
        for refed_component_address in &input.refed_component_addresses {
            let node_id = RENodeId::Component(refed_component_address.clone());
            if let Some(pointer) = self.node_refs.get(&node_id) {
                let mut visible = HashSet::new();
                visible.insert(SubstateId::ComponentInfo(*refed_component_address));
                next_frame_node_refs.insert(node_id.clone(), pointer.clone());
            } else {
                return Err(RuntimeError::InvokeMethodInvalidReferencePass(node_id));
            }
        }

        // Authorization check
        if !method_auths.is_empty() {
            let mut auth_zones = Vec::new();
            if let Some(self_auth_zone) = &self.auth_zone {
                auth_zones.push(self_auth_zone.borrow());
            }

            match &execution_state {
                // Resource auth check includes caller
                ExecutionState::Component(..)
                | ExecutionState::RENodeRef(RENodeId::ResourceManager(..), ..)
                | ExecutionState::RENodeRef(RENodeId::Vault(..), ..)
                | ExecutionState::Consumed(RENodeId::Bucket(..)) => {
                    if let Some(auth_zone) = self.caller_auth_zone {
                        auth_zones.push(auth_zone.borrow());
                    }
                }
                // Extern call auth check
                _ => {}
            };

            let mut borrowed = Vec::new();
            for auth_zone in &auth_zones {
                borrowed.push(auth_zone.deref());
            }
            for method_auth in method_auths {
                method_auth
                    .check(&borrowed)
                    .map_err(|error| RuntimeError::AuthorizationError {
                        function: fn_ident.clone(),
                        authorization: method_auth,
                        error,
                    })?;
            }
        }

        // Setup next parent frame
        let mut next_borrowed_values: Vec<&mut HashMap<RENodeId, HeapRootRENode>> = Vec::new();
        for parent_values in &mut self.parent_heap_nodes {
            next_borrowed_values.push(parent_values);
        }
        next_borrowed_values.push(&mut self.owned_heap_nodes);

        // start a new frame
        let (result, received_values) = {
            let mut frame = CallFrame::new(
                self.transaction_hash,
                self.depth + 1,
                self.max_depth,
                self.trace,
                self.id_allocator,
                self.track,
                actor,
                self.wasm_engine,
                self.wasm_instrumenter,
                self.cost_unit_counter,
                self.fee_table,
                match receiver {
                    Receiver::Component(_) => Some(RefCell::new(AuthZone::new())),
                    _ => None,
                },
                next_owned_values,
                next_frame_node_refs,
                next_borrowed_values,
                self.auth_zone.as_ref(),
            );

            // invoke the main function
            frame.run(
                ExecutionEntity::Method(receiver, execution_state),
                &fn_ident,
                input,
            )?
        };

        // Release locked addresses
        for l in locked_values {
            // TODO: refactor after introducing `Lock` representation.
            self.track
                .release_lock(l.clone(), is_lock_fee && matches!(l, SubstateId::Vault(..)));
        }

        // move buckets and proofs to this process.
        for (id, value) in received_values {
            trace!(self, Level::Debug, "Received value: {:?}", value);
            self.owned_heap_nodes.insert(id, value);
        }

        // Accept component references
        for refed_component_address in &result.refed_component_addresses {
            let node_id = RENodeId::Component(*refed_component_address);
            let mut visible = HashSet::new();
            visible.insert(SubstateId::ComponentInfo(*refed_component_address));
            self.node_refs
                .insert(node_id, RENodePointer::Store(node_id));
        }

        trace!(self, Level::Debug, "Invoking finished!");
        Ok(result)
    }

    fn borrow_node(
        &mut self,
        node_id: &RENodeId,
    ) -> Result<RENodeRef<'_, 's>, CostUnitCounterError> {
        trace!(self, Level::Debug, "Borrowing value: {:?}", node_id);
        self.cost_unit_counter.consume(
            self.fee_table.system_api_cost({
                match node_id {
                    RENodeId::Bucket(_) => SystemApiCostingEntry::BorrowLocal,
                    RENodeId::Proof(_) => SystemApiCostingEntry::BorrowLocal,
                    RENodeId::Worktop => SystemApiCostingEntry::BorrowLocal,
                    RENodeId::Vault(_) => SystemApiCostingEntry::BorrowGlobal {
                        // TODO: figure out loaded state and size
                        loaded: false,
                        size: 0,
                    },
                    RENodeId::Component(_) => SystemApiCostingEntry::BorrowGlobal {
                        // TODO: figure out loaded state and size
                        loaded: false,
                        size: 0,
                    },
                    RENodeId::KeyValueStore(_) => SystemApiCostingEntry::BorrowGlobal {
                        // TODO: figure out loaded state and size
                        loaded: false,
                        size: 0,
                    },
                    RENodeId::ResourceManager(_) => SystemApiCostingEntry::BorrowGlobal {
                        // TODO: figure out loaded state and size
                        loaded: false,
                        size: 0,
                    },
                    RENodeId::Package(_) => SystemApiCostingEntry::BorrowGlobal {
                        // TODO: figure out loaded state and size
                        loaded: false,
                        size: 0,
                    },
                    RENodeId::System => SystemApiCostingEntry::BorrowGlobal {
                        // TODO: figure out loaded state and size
                        loaded: false,
                        size: 0,
                    },
                }
            }),
            "borrow",
        )?;

        let node_pointer = self
            .node_refs
            .get(node_id)
            .expect(&format!("{:?} is unknown.", node_id));

        Ok(node_pointer.to_ref(self.depth, &self.owned_heap_nodes, &self.parent_heap_nodes, &self.track))
    }

    fn substate_borrow_mut(
        &mut self,
        substate_id: &SubstateId,
    ) -> Result<NativeSubstateRef, CostUnitCounterError> {
        trace!(
            self,
            Level::Debug,
            "Borrowing substate (mut): {:?}",
            substate_id
        );

        // Costing
        self.cost_unit_counter.consume(
            self.fee_table.system_api_cost({
                match substate_id {
                    SubstateId::Bucket(_) => SystemApiCostingEntry::BorrowLocal,
                    SubstateId::Proof(_) => SystemApiCostingEntry::BorrowLocal,
                    SubstateId::Worktop => SystemApiCostingEntry::BorrowLocal,
                    SubstateId::Vault(_) => SystemApiCostingEntry::BorrowGlobal {
                        // TODO: figure out loaded state and size
                        loaded: false,
                        size: 0,
                    },
                    SubstateId::ComponentState(..) => SystemApiCostingEntry::BorrowGlobal {
                        // TODO: figure out loaded state and size
                        loaded: false,
                        size: 0,
                    },
                    SubstateId::ComponentInfo(..) => SystemApiCostingEntry::BorrowGlobal {
                        // TODO: figure out loaded state and size
                        loaded: false,
                        size: 0,
                    },
                    SubstateId::KeyValueStoreSpace(_) => SystemApiCostingEntry::BorrowGlobal {
                        // TODO: figure out loaded state and size
                        loaded: false,
                        size: 0,
                    },
                    SubstateId::KeyValueStoreEntry(..) => SystemApiCostingEntry::BorrowGlobal {
                        // TODO: figure out loaded state and size
                        loaded: false,
                        size: 0,
                    },
                    SubstateId::ResourceManager(..) => SystemApiCostingEntry::BorrowGlobal {
                        // TODO: figure out loaded state and size
                        loaded: false,
                        size: 0,
                    },
                    SubstateId::NonFungibleSpace(..) => SystemApiCostingEntry::BorrowGlobal {
                        // TODO: figure out loaded state and size
                        loaded: false,
                        size: 0,
                    },
                    SubstateId::NonFungible(..) => SystemApiCostingEntry::BorrowGlobal {
                        // TODO: figure out loaded state and size
                        loaded: false,
                        size: 0,
                    },
                    SubstateId::Package(..) => SystemApiCostingEntry::BorrowGlobal {
                        // TODO: figure out loaded state and size
                        loaded: false,
                        size: 0,
                    },
                    SubstateId::System => SystemApiCostingEntry::BorrowGlobal {
                        // TODO: figure out loaded state and size
                        loaded: false,
                        size: 0,
                    },
                }
            }),
            "borrow",
        )?;

        // Authorization
        if !self.actor.is_substate_readable(substate_id) {
            panic!("Trying to read value which is not visible.")
        }

        let node_id = SubstateProperties::get_node_id(substate_id);

        let node_pointer = self
            .node_refs
            .get(&node_id)
            .expect(&format!("Node should exist {:?}", node_id));

        Ok(node_pointer.borrow_native_ref(
            self.depth,
            substate_id.clone(),
            &mut self.owned_heap_nodes,
            &mut self.parent_heap_nodes,
            &mut self.track,
        ))
    }

    fn substate_return_mut(
        &mut self,
        val_ref: NativeSubstateRef,
    ) -> Result<(), CostUnitCounterError> {
        trace!(self, Level::Debug, "Returning value");

        self.cost_unit_counter.consume(
            self.fee_table.system_api_cost({
                match &val_ref {
                    NativeSubstateRef::Stack(..) => SystemApiCostingEntry::ReturnLocal,
                    NativeSubstateRef::Track(substate_id, _) => match substate_id {
                        SubstateId::Vault(_) => SystemApiCostingEntry::ReturnGlobal { size: 0 },
                        SubstateId::KeyValueStoreSpace(_) => {
                            SystemApiCostingEntry::ReturnGlobal { size: 0 }
                        }
                        SubstateId::KeyValueStoreEntry(_, _) => {
                            SystemApiCostingEntry::ReturnGlobal { size: 0 }
                        }
                        SubstateId::ResourceManager(_) => {
                            SystemApiCostingEntry::ReturnGlobal { size: 0 }
                        }
                        SubstateId::Package(_) => SystemApiCostingEntry::ReturnGlobal { size: 0 },
                        SubstateId::NonFungibleSpace(_) => {
                            SystemApiCostingEntry::ReturnGlobal { size: 0 }
                        }
                        SubstateId::NonFungible(_, _) => {
                            SystemApiCostingEntry::ReturnGlobal { size: 0 }
                        }
                        SubstateId::ComponentInfo(..) => {
                            SystemApiCostingEntry::ReturnGlobal { size: 0 }
                        }
                        SubstateId::ComponentState(_) => {
                            SystemApiCostingEntry::ReturnGlobal { size: 0 }
                        }
                        SubstateId::System => SystemApiCostingEntry::ReturnGlobal { size: 0 },
                        SubstateId::Bucket(..) => SystemApiCostingEntry::ReturnGlobal { size: 0 },
                        SubstateId::Proof(..) => SystemApiCostingEntry::ReturnGlobal { size: 0 },
                        SubstateId::Worktop => SystemApiCostingEntry::ReturnGlobal { size: 0 },
                    },
                }
            }),
            "return",
        )?;

        val_ref.return_to_location(
            self.depth,
            &mut self.owned_heap_nodes,
            &mut self.parent_heap_nodes,
            &mut self.track,
        );
        Ok(())
    }

    fn node_drop(&mut self, node_id: &RENodeId) -> Result<HeapRootRENode, CostUnitCounterError> {
        trace!(self, Level::Debug, "Dropping value: {:?}", node_id);

        // TODO: costing

        // TODO: Authorization

        Ok(self.owned_heap_nodes.remove(&node_id).unwrap())
    }

    fn node_create(&mut self, re_node: HeapRENode) -> Result<RENodeId, RuntimeError> {
        trace!(self, Level::Debug, "Creating value");

        // Costing
        self.cost_unit_counter
            .consume(
                self.fee_table
                    .system_api_cost(SystemApiCostingEntry::Create {
                        size: 0, // TODO: get size of the value
                    }),
                "create",
            )
            .map_err(RuntimeError::CostingError)?;

        // TODO: Authorization

        // Take any required child nodes
        let children = re_node.get_child_nodes()?;
        let (taken_root_nodes, mut missing) = self.take_available_values(children, true)?;
        let first_missing_node = missing.drain().nth(0);
        if let Some(missing_node) = first_missing_node {
            return Err(RuntimeError::RENodeCreateNodeNotFound(missing_node));
        }
        let mut child_nodes = HashMap::new();
        for (id, taken_root_node) in taken_root_nodes {
            child_nodes.extend(taken_root_node.to_nodes(id));
        }

        // Insert node into heap
        let node_id = self.new_node_id(&re_node);
        let heap_root_node = HeapRootRENode {
            root: re_node,
            child_nodes,
        };
        self.owned_heap_nodes.insert(node_id, heap_root_node);

        // TODO: Clean the following up
        match node_id {
            RENodeId::KeyValueStore(..) | RENodeId::ResourceManager(..) => {
                self.node_refs.insert(
                    node_id.clone(),
                    RENodePointer::Heap {
                        frame_id: self.depth,
                        root: node_id.clone(),
                        id: None,
                    },
                );
            }
            RENodeId::Component(component_address) => {
                let mut visible = HashSet::new();
                visible.insert(SubstateId::ComponentInfo(component_address));
                self.node_refs.insert(
                    node_id.clone(),
                    RENodePointer::Heap {
                        frame_id: self.depth,
                        root: node_id.clone(),
                        id: None,
                    },
                );
            }
            _ => {}
        }

        Ok(node_id)
    }

    fn node_globalize(&mut self, node_id: RENodeId) -> Result<(), RuntimeError> {
        trace!(self, Level::Debug, "Globalizing value: {:?}", node_id);

        // Costing
        self.cost_unit_counter
            .consume(
                self.fee_table
                    .system_api_cost(SystemApiCostingEntry::Globalize {
                        size: 0, // TODO: get size of the value
                    }),
                "globalize",
            )
            .map_err(RuntimeError::CostingError)?;

        if !RENodeProperties::can_globalize(node_id) {
            return Err(RuntimeError::RENodeGlobalizeTypeNotAllowed(node_id));
        }

        // TODO: Authorization

        let mut nodes_to_take = HashSet::new();
        nodes_to_take.insert(node_id);
        let (taken_nodes, missing_nodes) = self.take_available_values(nodes_to_take, false)?;
        assert!(missing_nodes.is_empty());
        assert!(taken_nodes.len() == 1);
        let root_node = taken_nodes.into_values().nth(0).unwrap();

        let (substates, maybe_non_fungibles) = match root_node.root {
            HeapRENode::Component(component, component_state) => {
                let mut substates = HashMap::new();
                let component_address = node_id.into();
                substates.insert(
                    SubstateId::ComponentInfo(component_address),
                    Substate::Component(component),
                );
                substates.insert(
                    SubstateId::ComponentState(component_address),
                    Substate::ComponentState(component_state),
                );
                let mut visible_substates = HashSet::new();
                visible_substates.insert(SubstateId::ComponentInfo(component_address));
                (substates, None)
            }
            HeapRENode::Package(package) => {
                let mut substates = HashMap::new();
                let package_address = node_id.into();
                substates.insert(
                    SubstateId::Package(package_address),
                    Substate::Package(package),
                );
                (substates, None)
            }
            HeapRENode::Resource(resource_manager, non_fungibles) => {
                let mut substates = HashMap::new();
                let resource_address: ResourceAddress = node_id.into();
                substates.insert(
                    SubstateId::ResourceManager(resource_address),
                    Substate::Resource(resource_manager),
                );
                (substates, non_fungibles)
            }
            _ => panic!("Not expected"),
        };

        for (substate_id, substate) in substates {
            self.track
                .create_uuid_substate(substate_id.clone(), substate);
        }

        let mut to_store_values = HashMap::new();
        for (id, value) in root_node.child_nodes.into_iter() {
            to_store_values.insert(id, value);
        }
        insert_non_root_nodes(self.track, to_store_values);

        if let Some(non_fungibles) = maybe_non_fungibles {
            let resource_address: ResourceAddress = node_id.into();
            let parent_address = SubstateId::NonFungibleSpace(resource_address.clone());
            for (id, non_fungible) in non_fungibles {
                self.track.set_key_value(
                    parent_address.clone(),
                    id.to_vec(),
                    Substate::NonFungible(NonFungibleWrapper(Some(non_fungible))),
                );
            }
        }

        self.node_refs
            .insert(node_id, RENodePointer::Store(node_id));

        Ok(())
    }

    fn substate_read(&mut self, substate_id: SubstateId) -> Result<ScryptoValue, RuntimeError> {
        trace!(self, Level::Debug, "Reading value data: {:?}", substate_id);

        // Costing
        self.cost_unit_counter
            .consume(
                self.fee_table.system_api_cost(SystemApiCostingEntry::Read {
                    size: 0, // TODO: get size of the value
                }),
                "read",
            )
            .map_err(RuntimeError::CostingError)?;

        // Authorization
        if !self.actor.is_substate_readable(&substate_id) {
            return Err(RuntimeError::SubstateReadNotReadable(
                self.actor.clone(),
                substate_id.clone(),
            ));
        }

        let (parent_pointer, current_value) = self.read_value_internal(&substate_id)?;
        let cur_children = current_value.node_ids();
        for child_id in cur_children {
            let child_pointer = parent_pointer.child(child_id);
            self.node_refs.insert(child_id, child_pointer);
        }
        Ok(current_value)
    }

    fn substate_take(&mut self, substate_id: SubstateId) -> Result<ScryptoValue, RuntimeError> {
        trace!(self, Level::Debug, "Removing value data: {:?}", substate_id);

        // TODO: Costing

        // Authorization
        if !self.actor.is_substate_writeable(&substate_id) {
            return Err(RuntimeError::SubstateWriteNotWriteable(
                self.actor.clone(),
                substate_id,
            ));
        }

        let (pointer, current_value) = self.read_value_internal(&substate_id)?;
        let cur_children = current_value.node_ids();
        if !cur_children.is_empty() {
            return Err(RuntimeError::ValueNotAllowed);
        }

        // Write values
        let mut node_ref = pointer.to_ref_mut(
            self.depth,
            &mut self.owned_heap_nodes,
            &mut self.parent_heap_nodes,
            &mut self.track,
        );
        node_ref.replace_value_with_default(&substate_id);

        Ok(current_value)
    }

    fn substate_write(
        &mut self,
        substate_id: SubstateId,
        value: ScryptoValue,
    ) -> Result<(), RuntimeError> {
        trace!(self, Level::Debug, "Writing value data: {:?}", substate_id);

        // Costing
        self.cost_unit_counter
            .consume(
                self.fee_table
                    .system_api_cost(SystemApiCostingEntry::Write {
                        size: 0, // TODO: get size of the value
                    }),
                "write",
            )
            .map_err(RuntimeError::CostingError)?;

        // Authorization
        if !self.actor.is_substate_writeable(&substate_id) {
            return Err(RuntimeError::SubstateWriteNotWriteable(
                self.actor.clone(),
                substate_id,
            ));
        }

        // If write, take values from current frame
        let (taken_nodes, missing_nodes) = {
            let node_ids = value.node_ids();
            if !node_ids.is_empty() {
                if !SubstateProperties::can_own_nodes(&substate_id) {
                    return Err(RuntimeError::ValueNotAllowed);
                }

                self.take_available_values(node_ids, true)?
            } else {
                (HashMap::new(), HashSet::new())
            }
        };

        let (pointer, current_value) = self.read_value_internal(&substate_id)?;
        let cur_children = current_value.node_ids();

        // Fulfill method
        verify_stored_value_update(&cur_children, &missing_nodes)?;

        // TODO: verify against some schema

        // Write values
        let mut node_ref = pointer.to_ref_mut(
            self.depth,
            &mut self.owned_heap_nodes,
            &mut self.parent_heap_nodes,
            &mut self.track,
        );
        node_ref.write_value(substate_id, value, taken_nodes);

        Ok(())
    }

    fn transaction_hash(&mut self) -> Result<Hash, CostUnitCounterError> {
        self.cost_unit_counter.consume(
            self.fee_table
                .system_api_cost(SystemApiCostingEntry::ReadTransactionHash),
            "read_transaction_hash",
        )?;
        Ok(self.transaction_hash)
    }

    fn generate_uuid(&mut self) -> Result<u128, CostUnitCounterError> {
        self.cost_unit_counter.consume(
            self.fee_table
                .system_api_cost(SystemApiCostingEntry::GenerateUuid),
            "generate_uuid",
        )?;
        Ok(self.new_uuid())
    }

    fn emit_log(&mut self, level: Level, message: String) -> Result<(), CostUnitCounterError> {
        self.cost_unit_counter.consume(
            self.fee_table
                .system_api_cost(SystemApiCostingEntry::EmitLog {
                    size: message.len() as u32,
                }),
            "emit_log",
        )?;
        self.track.add_log(level, message);
        Ok(())
    }

    fn check_access_rule(
        &mut self,
        access_rule: scrypto::resource::AccessRule,
        proof_ids: Vec<ProofId>,
    ) -> Result<bool, RuntimeError> {
        let proofs = proof_ids
            .iter()
            .map(|proof_id| {
                self.owned_heap_nodes
                    .get(&RENodeId::Proof(*proof_id))
                    .map(|p| match p.root() {
                        HeapRENode::Proof(proof) => proof.clone(),
                        _ => panic!("Expected proof"),
                    })
                    .ok_or(RuntimeError::ProofNotFound(proof_id.clone()))
            })
            .collect::<Result<Vec<Proof>, RuntimeError>>()?;
        let mut simulated_auth_zone = AuthZone::new_with_proofs(proofs);

        let method_authorization = convert(&Type::Unit, &Value::Unit, &access_rule);
        let is_authorized = method_authorization.check(&[&simulated_auth_zone]).is_ok();
        simulated_auth_zone
            .main(
                "clear",
                ScryptoValue::from_typed(&AuthZoneClearInput {}),
                self,
            )
            .map_err(RuntimeError::AuthZoneError)?;

        Ok(is_authorized)
    }

    fn cost_unit_counter(&mut self) -> &mut C {
        self.cost_unit_counter
    }

    fn fee_table(&self) -> &FeeTable {
        self.fee_table
    }
}
