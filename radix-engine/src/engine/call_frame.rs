use sbor::rust::boxed::Box;
use sbor::rust::cell::Ref;
use sbor::rust::cell::{RefCell, RefMut};
use sbor::rust::collections::*;
use sbor::rust::format;
use sbor::rust::marker::*;
use sbor::rust::ops::Deref;
use sbor::rust::ops::DerefMut;
use sbor::rust::string::String;
use sbor::rust::string::ToString;
use sbor::rust::vec;
use sbor::rust::vec::Vec;
use sbor::*;
use scrypto::buffer::scrypto_decode;
use scrypto::core::{Network, SNodeRef, ScryptoActor};
use scrypto::engine::types::*;
use scrypto::prelude::ComponentOffset;
use scrypto::resource::AuthZoneClearInput;
use scrypto::values::*;
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
    W,  // WASM engine type
    I,  // WASM instance type
> where
    W: WasmEngine<I>,
    I: WasmInstance,
{
    /// The transaction hash
    transaction_hash: Hash,
    /// The call depth
    depth: usize,
    /// Whether to show trace messages
    trace: bool,

    /// State track
    track: &'g mut Track,
    /// Wasm engine
    wasm_engine: &'g mut W,
    /// Wasm Instrumenter
    wasm_instrumenter: &'g mut WasmInstrumenter,

    /// Remaining cost unit counter
    cost_unit_counter: &'g mut CostUnitCounter,
    /// Fee table
    fee_table: &'g FeeTable,

    /// All ref values accessible by this call frame. The value may be located in one of the following:
    /// 1. borrowed values
    /// 2. track
    value_refs: HashMap<ValueId, REValueInfo>,

    /// Owned Values
    owned_values: HashMap<ValueId, RefCell<REValue>>,
    auth_zone: Option<RefCell<AuthZone>>,

    /// Borrowed Values from call frames up the stack
    frame_borrowed_values: HashMap<ValueId, RefMut<'p, REValue>>,
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
    old: &HashSet<StoredValueId>,
    missing: &HashSet<StoredValueId>,
) -> Result<(), RuntimeError> {
    // TODO: optimize intersection search
    for old_id in old.iter() {
        if !missing.contains(&old_id) {
            return Err(RuntimeError::StoredValueRemoved(old_id.clone()));
        }
    }

    for missing_id in missing.iter() {
        if !old.contains(missing_id) {
            return Err(RuntimeError::ValueNotFound(ValueId::Stored(*missing_id)));
        }
    }

    Ok(())
}

fn to_stored_ids(ids: HashSet<ValueId>) -> Result<HashSet<StoredValueId>, RuntimeError> {
    let mut stored_ids = HashSet::new();
    for id in ids {
        match id {
            ValueId::Stored(stored_id) => stored_ids.insert(stored_id),
            _ => return Err(RuntimeError::MovingInvalidType),
        };
    }
    Ok(stored_ids)
}

fn verify_stored_key(value: &ScryptoValue) -> Result<(), RuntimeError> {
    if !value.bucket_ids.is_empty() {
        return Err(RuntimeError::BucketNotAllowed);
    }
    if !value.proof_ids.is_empty() {
        return Err(RuntimeError::ProofNotAllowed);
    }
    if !value.vault_ids.is_empty() {
        return Err(RuntimeError::VaultNotAllowed);
    }
    if !value.kv_store_ids.is_empty() {
        return Err(RuntimeError::KeyValueStoreNotAllowed);
    }
    Ok(())
}

#[derive(Debug, Clone)]
pub struct REValueInfo {
    visible: bool,
    location: REValueLocation,
}

#[derive(Debug, Clone)]
pub enum REValueLocation {
    OwnedRoot,
    Owned {
        root: ValueId,
        ancestors: Vec<KeyValueStoreId>,
    },
    BorrowedRoot,
    Borrowed {
        root: ValueId,
        ancestors: Vec<KeyValueStoreId>,
    },
    Track {
        parent: Option<ComponentAddress>,
    },
}

impl REValueLocation {
    fn child(&self, value_id: ValueId) -> REValueLocation {
        match self {
            REValueLocation::OwnedRoot => REValueLocation::Owned {
                root: value_id,
                ancestors: vec![],
            },
            REValueLocation::Owned { root, ancestors } => {
                let mut next_ancestors = ancestors.clone();
                let kv_store_id = value_id.into();
                next_ancestors.push(kv_store_id);
                REValueLocation::Owned {
                    root: root.clone(),
                    ancestors: next_ancestors,
                }
            }
            REValueLocation::BorrowedRoot => REValueLocation::Borrowed {
                root: value_id,
                ancestors: vec![],
            },
            REValueLocation::Borrowed { root, ancestors } => {
                let mut next_ancestors = ancestors.clone();
                let kv_store_id = value_id.into();
                next_ancestors.push(kv_store_id);
                REValueLocation::Borrowed {
                    root: root.clone(),
                    ancestors: next_ancestors,
                }
            }
            REValueLocation::Track { parent } => REValueLocation::Track {
                parent: Some(parent.unwrap_or_else(|| value_id.into())),
            },
        }
    }

    fn borrow_native_ref<'borrowed>(
        &self,
        value_id: &ValueId,
        owned_values: &mut HashMap<ValueId, RefCell<REValue>>,
        borrowed_values: &mut HashMap<ValueId, RefMut<'borrowed, REValue>>,
        track: &mut Track,
    ) -> RENativeValueRef<'borrowed> {
        match self {
            REValueLocation::BorrowedRoot => {
                let owned = borrowed_values.remove(value_id).expect("Should exist");
                RENativeValueRef::OwnedRef(owned)
            }
            REValueLocation::Track { parent } => {
                let address = match value_id {
                    ValueId::Stored(StoredValueId::VaultId(vault_id)) => {
                        Address::Vault(parent.unwrap(), *vault_id)
                    }
                    ValueId::Resource(resouce_address) => Address::Resource(*resouce_address),
                    ValueId::Package(package_address) => Address::Package(*package_address),
                    ValueId::System => Address::System,
                    _ => panic!("Unexpected"),
                };

                let value = track.take_value(address.clone());

                RENativeValueRef::Track(address, value)
            }
            REValueLocation::OwnedRoot => {
                let cell = owned_values.remove(value_id).unwrap();
                let value = cell.into_inner();
                RENativeValueRef::Owned(value)
            }
            _ => panic!("Unexpected {:?} {:?}", self, value_id),
        }
    }

    fn to_owned_ref<'a, 'borrowed>(
        &self,
        value_id: &ValueId,
        owned_values: &'a HashMap<ValueId, RefCell<REValue>>,
        borrowed_values: &'a HashMap<ValueId, RefMut<'borrowed, REValue>>,
    ) -> Ref<'a, REValue> {
        match self {
            REValueLocation::OwnedRoot => {
                let cell = owned_values.get(value_id).unwrap();
                cell.borrow()
            }
            REValueLocation::Owned {
                root,
                ref ancestors,
            } => unsafe {
                let root_value = owned_values
                    .get(&root)
                    .unwrap()
                    .try_borrow_unguarded()
                    .unwrap();
                let children = root_value
                    .get_children_store()
                    .expect("Should have children");
                children.get_child(ancestors, value_id)
            },
            REValueLocation::Borrowed { root, ancestors } => unsafe {
                let borrowed = borrowed_values.get(root).unwrap();
                borrowed
                    .get_children_store()
                    .unwrap()
                    .get_child(ancestors, value_id)
            },
            _ => panic!("Not an owned ref"),
        }
    }

    fn to_ref<'a, 'p>(
        &self,
        value_id: &ValueId,
        owned_values: &'a HashMap<ValueId, RefCell<REValue>>,
        borrowed_values: &'a HashMap<ValueId, RefMut<'p, REValue>>,
        track: &'a Track,
    ) -> REValueRef<'a, 'p> {
        match self {
            REValueLocation::OwnedRoot
            | REValueLocation::Owned { .. }
            | REValueLocation::Borrowed { .. } => {
                REValueRef::Owned(self.to_owned_ref(value_id, owned_values, borrowed_values))
            }
            REValueLocation::BorrowedRoot => {
                REValueRef::Borrowed(borrowed_values.get(value_id).unwrap())
            }
            REValueLocation::Track { parent } => {
                let address = match value_id {
                    ValueId::Stored(StoredValueId::VaultId(vault_id)) => {
                        Address::Vault(parent.unwrap(), *vault_id)
                    }
                    ValueId::Stored(StoredValueId::KeyValueStoreId(kv_store_id)) => {
                        Address::KeyValueStore(parent.unwrap(), *kv_store_id)
                    }
                    ValueId::Stored(StoredValueId::Component(component_address)) => {
                        if let Some(parent) = parent {
                            Address::LocalComponent(*parent, *component_address)
                        } else {
                            Address::GlobalComponent(*component_address)
                        }
                    }
                    ValueId::Package(package_address) => Address::Package(*package_address),
                    ValueId::Resource(resource_address) => Address::Resource(*resource_address),
                    ValueId::System => Address::System,
                    _ => panic!("Unexpected value id"),
                };

                REValueRef::Track(track, address)
            }
        }
    }

    fn to_owned_ref_mut<'a, 'borrowed>(
        &self,
        value_id: &ValueId,
        owned_values: &'a mut HashMap<ValueId, RefCell<REValue>>,
        borrowed_values: &'a mut HashMap<ValueId, RefMut<'borrowed, REValue>>,
    ) -> RefMut<'a, REValue> {
        match self {
            REValueLocation::OwnedRoot => {
                let cell = owned_values.get_mut(value_id).unwrap();
                cell.borrow_mut()
            }
            REValueLocation::Owned {
                root,
                ref ancestors,
            } => {
                let root_value = owned_values.get_mut(&root).unwrap().get_mut();
                let children = root_value
                    .get_children_store_mut()
                    .expect("Should have children");
                children.get_child_mut(ancestors, value_id)
            }
            REValueLocation::Borrowed { root, ancestors } => {
                let borrowed = borrowed_values.get_mut(root).unwrap();
                borrowed
                    .get_children_store_mut()
                    .unwrap()
                    .get_child_mut(ancestors, value_id)
            }
            _ => panic!("Not an owned ref"),
        }
    }

    fn to_ref_mut<'a, 'borrowed, 'c>(
        &self,
        value_id: &ValueId,
        owned_values: &'a mut HashMap<ValueId, RefCell<REValue>>,
        borrowed_values: &'a mut HashMap<ValueId, RefMut<'borrowed, REValue>>,
        track: &'c mut Track,
    ) -> REValueRefMut<'a, 'borrowed, 'c> {
        match self {
            REValueLocation::OwnedRoot
            | REValueLocation::Owned { .. }
            | REValueLocation::Borrowed { .. } => {
                REValueRefMut::Owned(self.to_owned_ref_mut(value_id, owned_values, borrowed_values))
            }
            REValueLocation::BorrowedRoot => {
                REValueRefMut::Borrowed(borrowed_values.get_mut(value_id).unwrap())
            }
            REValueLocation::Track { parent } => {
                let address = match value_id {
                    ValueId::Stored(StoredValueId::VaultId(vault_id)) => {
                        Address::Vault(parent.unwrap(), *vault_id)
                    }
                    ValueId::Stored(StoredValueId::KeyValueStoreId(kv_store_id)) => {
                        Address::KeyValueStore(parent.unwrap(), *kv_store_id)
                    }
                    ValueId::Stored(StoredValueId::Component(component_address)) => {
                        if let Some(parent) = parent {
                            Address::LocalComponent(*parent, *component_address)
                        } else {
                            Address::GlobalComponent(*component_address)
                        }
                    }
                    ValueId::Resource(resource_address) => Address::Resource(*resource_address),
                    ValueId::NonFungibles(resource_address) => {
                        Address::NonFungibleSet(*resource_address)
                    }
                    _ => panic!("Unexpected value id {:?}", value_id),
                };

                REValueRefMut::Track(track, address)
            }
        }
    }
}

pub enum RENativeValueRef<'borrowed> {
    Owned(REValue),
    OwnedRef(RefMut<'borrowed, REValue>),
    Track(Address, SubstateValue),
}

impl<'borrowed> RENativeValueRef<'borrowed> {
    pub fn bucket(&mut self) -> &mut Bucket {
        match self {
            RENativeValueRef::OwnedRef(root) => match root.deref_mut() {
                REValue::Bucket(bucket) => bucket,
                _ => panic!("Expecting to be a bucket"),
            },
            _ => panic!("Expecting to be a bucket"),
        }
    }

    pub fn proof(&mut self) -> &mut Proof {
        match self {
            RENativeValueRef::OwnedRef(ref mut root) => match root.deref_mut() {
                REValue::Proof(proof) => proof,
                _ => panic!("Expecting to be a proof"),
            },
            _ => panic!("Expecting to be a proof"),
        }
    }

    pub fn worktop(&mut self) -> &mut Worktop {
        match self {
            RENativeValueRef::OwnedRef(ref mut root) => match root.deref_mut() {
                REValue::Worktop(worktop) => worktop,
                _ => panic!("Expecting to be a worktop"),
            },
            _ => panic!("Expecting to be a worktop"),
        }
    }

    pub fn vault(&mut self) -> &mut Vault {
        match self {
            RENativeValueRef::Owned(..) => panic!("Unexpected"),
            RENativeValueRef::OwnedRef(owned) => owned.vault_mut(),
            RENativeValueRef::Track(_address, value) => value.vault_mut().0,
        }
    }

    pub fn system(&mut self) -> &mut System {
        match self {
            RENativeValueRef::Track(_address, value) => value.system_mut(),
            _ => panic!("Expecting to be system"),
        }
    }

    pub fn component(&mut self) -> &mut Component {
        match self {
            RENativeValueRef::OwnedRef(owned) => owned.component_mut(),
            _ => panic!("Expecting to be a component"),
        }
    }

    pub fn package(&mut self) -> &ValidatedPackage {
        match self {
            RENativeValueRef::Track(_address, value) => value.package(),
            _ => panic!("Expecting to be tracked"),
        }
    }

    pub fn resource_manager(&mut self) -> &mut ResourceManager {
        match self {
            RENativeValueRef::Owned(owned) => owned.resource_manager_mut(),
            RENativeValueRef::Track(_address, value) => value.resource_manager_mut(),
            _ => panic!("Unexpected"),
        }
    }

    pub fn return_to_location<'a>(
        self,
        value_id: ValueId,
        owned_values: &'a mut HashMap<ValueId, RefCell<REValue>>,
        borrowed_values: &mut HashMap<ValueId, RefMut<'borrowed, REValue>>,
        track: &mut Track,
    ) {
        match self {
            RENativeValueRef::Owned(value) => {
                owned_values.insert(value_id, RefCell::new(value));
            }
            RENativeValueRef::OwnedRef(owned) => {
                borrowed_values.insert(value_id.clone(), owned);
            }
            RENativeValueRef::Track(address, value) => track.write_value(address, value),
        }
    }
}

pub enum REValueRef<'f, 'p> {
    Owned(Ref<'f, REValue>),
    Borrowed(&'f RefMut<'p, REValue>),
    Track(&'f Track, Address),
}

impl<'f, 'p> REValueRef<'f, 'p> {
    pub fn vault(&self) -> &Vault {
        match self {
            REValueRef::Owned(owned) => owned.vault(),
            REValueRef::Track(track, address) => track.read_value(address.clone()).vault().0,
            REValueRef::Borrowed(borrowed) => borrowed.vault(),
        }
    }

    pub fn system(&self) -> &System {
        match self {
            REValueRef::Owned(owned) => owned.system(),
            REValueRef::Track(track, address) => track.read_value(address.clone()).system(),
            _ => panic!("Unexpected system ref"),
        }
    }

    pub fn resource_manager(&self) -> &ResourceManager {
        match self {
            REValueRef::Owned(owned) => owned.resource_manager(),
            REValueRef::Track(track, address) => {
                track.read_value(address.clone()).resource_manager()
            }
            REValueRef::Borrowed(borrowed) => borrowed.resource_manager(),
        }
    }

    pub fn component(&self) -> &Component {
        match self {
            REValueRef::Owned(owned) => owned.component(),
            REValueRef::Track(track, address) => track.read_value(address.clone()).component(),
            REValueRef::Borrowed(borrowed) => borrowed.component(),
        }
    }

    pub fn package(&self) -> &ValidatedPackage {
        match self {
            REValueRef::Owned(owned) => owned.package(),
            REValueRef::Track(track, address) => track.read_value(address.clone()).package(),
            _ => panic!("Unexpected component ref"),
        }
    }
}

pub enum REValueRefMut<'a, 'b, 'c> {
    Owned(RefMut<'a, REValue>),
    Borrowed(&'a mut RefMut<'b, REValue>),
    Track(&'c mut Track, Address),
}

impl<'a, 'b, 'c> REValueRefMut<'a, 'b, 'c> {
    fn kv_store_put(
        &mut self,
        key: Vec<u8>,
        value: ScryptoValue,
        to_store: HashMap<ValueId, REValue>,
    ) {
        match self {
            REValueRefMut::Owned(owned) => {
                let children = owned.get_children_store_mut();
                children.unwrap().insert_children(to_store);
                owned.kv_store_mut().put(key, value);
            }
            REValueRefMut::Borrowed(..) => {
                panic!("Not supported");
            }
            REValueRefMut::Track(track, address) => {
                let component_address =
                    if let Address::KeyValueStore(component_address, _) = &address {
                        component_address
                    } else {
                        panic!("Expected KV Store address");
                    };

                track.set_key_value(
                    address.clone(),
                    key,
                    SubstateValue::KeyValueStoreEntry(Some(value.raw)),
                );
                track.insert_objects_into_component(to_store, *component_address);
            }
        }
    }

    fn kv_store_get(&mut self, key: &[u8]) -> ScryptoValue {
        let maybe_value = match self {
            REValueRefMut::Owned(owned) => {
                let store = owned.kv_store_mut();
                store.get(key).map(|v| v.dom)
            }
            REValueRefMut::Borrowed(..) => {
                panic!("Not supported");
            }
            REValueRefMut::Track(track, address) => {
                let substate_value = track.read_key_value(address.clone(), key.to_vec());
                substate_value
                    .kv_entry()
                    .as_ref()
                    .map(|bytes| decode_any(bytes).unwrap())
            }
        };

        // TODO: Cleanup
        let value = maybe_value.map_or(
            Value::Option {
                value: Box::new(Option::None),
            },
            |v| Value::Option {
                value: Box::new(Some(v)),
            },
        );
        ScryptoValue::from_value(value).unwrap()
    }

    fn non_fungible_get(&mut self, id: &NonFungibleId) -> ScryptoValue {
        match self {
            REValueRefMut::Owned(owned) => {
                ScryptoValue::from_typed(&owned.non_fungibles().get(id).cloned())
            }
            REValueRefMut::Borrowed(..) => {
                panic!("Not supported");
            }
            REValueRefMut::Track(track, address) => {
                let value = track.read_key_value(address.clone(), id.to_vec());
                ScryptoValue::from_typed(value.non_fungible())
            }
        }
    }

    fn non_fungible_remove(&mut self, id: &NonFungibleId) {
        match self {
            REValueRefMut::Owned(..) => {
                panic!("Not supported");
            }
            REValueRefMut::Borrowed(..) => {
                panic!("Not supported");
            }
            REValueRefMut::Track(track, address) => {
                track.set_key_value(
                    address.clone(),
                    id.to_vec(),
                    SubstateValue::NonFungible(None),
                );
            }
        }
    }

    fn non_fungible_put(&mut self, id: NonFungibleId, value: ScryptoValue) {
        match self {
            REValueRefMut::Owned(owned) => {
                let non_fungible: NonFungible =
                    scrypto_decode(&value.raw).expect("Should not fail.");
                owned.non_fungibles_mut().insert(id, non_fungible);
            }
            REValueRefMut::Borrowed(..) => {
                panic!("Not supported");
            }
            REValueRefMut::Track(track, address) => {
                let non_fungible: NonFungible =
                    scrypto_decode(&value.raw).expect("Should not fail.");
                track.set_key_value(
                    address.clone(),
                    id.to_vec(),
                    SubstateValue::NonFungible(Some(non_fungible)),
                );
            }
        }
    }

    fn component_put(&mut self, value: ScryptoValue, to_store: HashMap<ValueId, REValue>) {
        match self {
            REValueRefMut::Track(track, address) => {
                track.write_component_value(address.clone(), value.raw);

                let parent_address = match address {
                    Address::GlobalComponent(address) => *address,
                    Address::LocalComponent(parent, ..) => *parent,
                    _ => panic!("Expected component address"),
                };

                track.insert_objects_into_component(to_store, parent_address);
            }
            REValueRefMut::Borrowed(owned) => {
                let component = owned.component_mut();
                component.set_state(value.raw);
                let children = owned.get_children_store_mut();
                children.unwrap().insert_children(to_store);
            }
            _ => panic!("Unexpected component ref"),
        }
    }

    fn component(&mut self) -> &Component {
        match self {
            REValueRefMut::Owned(owned) => owned.component(),
            REValueRefMut::Borrowed(borrowed) => borrowed.component(),
            REValueRefMut::Track(track, address) => {
                let component_val = track.read_value(address.clone());
                component_val.component()
            }
        }
    }
}

pub enum StaticSNodeState {
    Package,
    Resource,
    TransactionProcessor,
}

pub enum SNodeExecution<'a> {
    Static(StaticSNodeState),
    Consumed(ValueId),
    AuthZone(RefMut<'a, AuthZone>),
    ValueRef(ValueId),
    Scrypto(ScryptoActorInfo, PackageAddress),
}

pub enum SubstateAddress {
    KeyValueEntry(KeyValueStoreId, ScryptoValue),
    NonFungible(ResourceAddress, NonFungibleId),
    Component(ComponentAddress, ComponentOffset),
}

impl<'p, 'g, W, I> CallFrame<'p, 'g, W, I>
where
    W: WasmEngine<I>,
    I: WasmInstance,
{
    pub fn new_root(
        verbose: bool,
        transaction_hash: Hash,
        signer_public_keys: Vec<EcdsaPublicKey>,
        is_system: bool,
        track: &'g mut Track,
        wasm_engine: &'g mut W,
        wasm_instrumenter: &'g mut WasmInstrumenter,
        cost_unit_counter: &'g mut CostUnitCounter,
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
            let system_proof = system_bucket.create_proof(track.new_bucket_id()).unwrap();
            initial_auth_zone_proofs.push(system_proof);
        }

        Self::new(
            transaction_hash,
            0,
            verbose,
            track,
            wasm_engine,
            wasm_instrumenter,
            cost_unit_counter,
            fee_table,
            Some(RefCell::new(AuthZone::new_with_proofs(
                initial_auth_zone_proofs,
            ))),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            None,
        )
    }

    pub fn new(
        transaction_hash: Hash,
        depth: usize,
        trace: bool,
        track: &'g mut Track,
        wasm_engine: &'g mut W,
        wasm_instrumenter: &'g mut WasmInstrumenter,
        cost_unit_counter: &'g mut CostUnitCounter,
        fee_table: &'g FeeTable,
        auth_zone: Option<RefCell<AuthZone>>,
        owned_values: HashMap<ValueId, RefCell<REValue>>,
        value_refs: HashMap<ValueId, REValueInfo>,
        frame_borrowed_values: HashMap<ValueId, RefMut<'p, REValue>>,
        caller_auth_zone: Option<&'p RefCell<AuthZone>>,
    ) -> Self {
        Self {
            transaction_hash,
            depth,
            trace,
            track,
            wasm_engine,
            wasm_instrumenter,
            cost_unit_counter,
            fee_table,
            owned_values,
            value_refs,
            frame_borrowed_values,
            auth_zone,
            caller_auth_zone,
            phantom: PhantomData,
        }
    }

    fn drop_owned_values(&mut self) -> Result<(), RuntimeError> {
        let values = self
            .owned_values
            .drain()
            .map(|(_, cell)| cell.into_inner())
            .collect();
        REValue::drop_values(values).map_err(|e| RuntimeError::DropFailure(e))
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

    fn process_return_data(
        &mut self,
        from: SNodeRef,
        validated: &ScryptoValue,
    ) -> Result<(), RuntimeError> {
        if !validated.kv_store_ids.is_empty() {
            return Err(RuntimeError::KeyValueStoreNotAllowed);
        }

        // Allow vaults to be returned from ResourceStatic
        // TODO: Should we allow vaults to be returned by any component?
        if !matches!(from, SNodeRef::ResourceRef(_)) {
            if !validated.vault_ids.is_empty() {
                return Err(RuntimeError::VaultNotAllowed);
            }
        }

        Ok(())
    }

    pub fn run(
        &mut self,
        snode_ref: SNodeRef, // TODO: Remove, abstractions between invoke_snode() and run() are a bit messy right now
        execution: SNodeExecution<'p>,
        fn_ident: &str,
        input: ScryptoValue,
    ) -> Result<(ScryptoValue, HashMap<ValueId, REValue>), RuntimeError> {
        trace!(
            self,
            Level::Debug,
            "Run started! Depth: {}, Remaining cost units: {}",
            self.depth,
            self.cost_unit_counter.balance()
        );

        self.cost_unit_counter
            .consume(
                self.fee_table.function_cost(&snode_ref, fn_ident, &input),
                "run_function",
            )
            .map_err(RuntimeError::CostingError)?;

        let output = {
            let rtn = match execution {
                SNodeExecution::Static(state) => match state {
                    StaticSNodeState::TransactionProcessor => TransactionProcessor::static_main(
                        fn_ident, input, self,
                    )
                    .map_err(|e| match e {
                        TransactionProcessorError::InvalidRequestData(_) => panic!("Illegal state"),
                        TransactionProcessorError::InvalidMethod => panic!("Illegal state"),
                        TransactionProcessorError::RuntimeError(e) => e,
                    }),
                    StaticSNodeState::Package => {
                        ValidatedPackage::static_main(fn_ident, input, self)
                            .map_err(RuntimeError::PackageError)
                    }
                    StaticSNodeState::Resource => {
                        ResourceManager::static_main(fn_ident, input, self)
                            .map_err(RuntimeError::ResourceManagerError)
                    }
                },
                SNodeExecution::Consumed(value_id) => match value_id {
                    ValueId::Transient(TransientValueId::Bucket(..)) => {
                        Bucket::consuming_main(value_id, fn_ident, input, self)
                            .map_err(RuntimeError::BucketError)
                    }
                    ValueId::Transient(TransientValueId::Proof(..)) => {
                        Proof::main_consume(value_id, fn_ident, input, self)
                            .map_err(RuntimeError::ProofError)
                    }
                    ValueId::Stored(StoredValueId::Component(..)) => {
                        Component::main_consume(value_id, fn_ident, input, self)
                            .map_err(RuntimeError::ComponentError)
                    }
                    _ => panic!("Unexpected"),
                },
                SNodeExecution::AuthZone(mut auth_zone) => auth_zone
                    .main(fn_ident, input, self)
                    .map_err(RuntimeError::AuthZoneError),
                SNodeExecution::ValueRef(value_id) => match value_id {
                    ValueId::Transient(TransientValueId::Bucket(bucket_id)) => {
                        Bucket::main(bucket_id, fn_ident, input, self)
                            .map_err(RuntimeError::BucketError)
                    }
                    ValueId::Transient(TransientValueId::Proof(..)) => {
                        Proof::main(value_id, fn_ident, input, self)
                            .map_err(RuntimeError::ProofError)
                    }
                    ValueId::Transient(TransientValueId::Worktop) => {
                        Worktop::main(value_id, fn_ident, input, self)
                            .map_err(RuntimeError::WorktopError)
                    }
                    ValueId::Stored(StoredValueId::VaultId(vault_id)) => {
                        Vault::main(vault_id, fn_ident, input, self)
                            .map_err(RuntimeError::VaultError)
                    }
                    ValueId::Stored(StoredValueId::Component(..)) => {
                        Component::main(value_id, fn_ident, input, self)
                            .map_err(RuntimeError::ComponentError)
                    }
                    ValueId::Resource(resource_address) => {
                        ResourceManager::main(resource_address, fn_ident, input, self)
                            .map_err(RuntimeError::ResourceManagerError)
                    }
                    ValueId::System => {
                        System::main(fn_ident, input, self).map_err(RuntimeError::SystemError)
                    }
                    _ => panic!("Unexpected"),
                },
                SNodeExecution::Scrypto(ref actor, package_address) => {
                    let output = {
                        let package = self.track.read_value(package_address).package();
                        let wasm_metering_params = self.fee_table.wasm_metering_params();
                        let instrumented_code = self
                            .wasm_instrumenter
                            .instrument(package.code(), &wasm_metering_params);
                        let mut instance = self.wasm_engine.instantiate(instrumented_code);
                        let blueprint_abi = package
                            .blueprint_abi(actor.blueprint_name())
                            .expect("Blueprint should exist");
                        let export_name = &blueprint_abi
                            .get_fn_abi(fn_ident)
                            .unwrap()
                            .export_name
                            .to_string();
                        let mut runtime: Box<dyn WasmRuntime> =
                            Box::new(RadixEngineWasmRuntime::new(actor.clone(), self));
                        instance
                            .invoke_export(&export_name, &input, &mut runtime)
                            .map_err(|e| match e {
                                // Flatten error code for more readable transaction receipt
                                InvokeError::RuntimeError(e) => e,
                                e @ _ => RuntimeError::InvokeError(e.into()),
                            })?
                    };

                    let package = self.track.read_value(package_address).package();
                    let blueprint_abi = package
                        .blueprint_abi(actor.blueprint_name())
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
            }?;

            rtn
        };

        // Prevent vaults/kvstores from being returned
        self.process_return_data(snode_ref, &output)?;

        // Take values to return
        let values_to_take = output.value_ids();
        let (taken_values, mut missing) = self.take_available_values(values_to_take, false)?;
        let first_missing_value = missing.drain().nth(0);
        if let Some(missing_value) = first_missing_value {
            return Err(RuntimeError::ValueNotFound(missing_value));
        }

        // drop proofs and check resource leak
        if self.auth_zone.is_some() {
            self.invoke_snode(
                SNodeRef::AuthZoneRef,
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
        value_ids: HashSet<ValueId>,
        persist_only: bool,
    ) -> Result<(HashMap<ValueId, REValue>, HashSet<ValueId>), RuntimeError> {
        let (taken, missing) = {
            let mut taken_values = HashMap::new();
            let mut missing_values = HashSet::new();

            for id in value_ids {
                let maybe = self.owned_values.remove(&id);
                if let Some(celled_value) = maybe {
                    let value = celled_value.into_inner();
                    value.verify_can_move()?;
                    if persist_only {
                        value.verify_can_persist()?;
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
            self.value_refs.remove(id);
            if let Some(children) = value.get_children_store() {
                for id in children.all_descendants() {
                    self.value_refs.remove(&id);
                }
            }
        }

        Ok((taken, missing))
    }

    fn read_value_internal(
        &mut self,
        address: &SubstateAddress,
    ) -> Result<
        (
            ValueId,
            REValueLocation,
            ScryptoValue,
            HashSet<StoredValueId>,
        ),
        RuntimeError,
    > {
        let value_id = match address {
            SubstateAddress::Component(component_address, ..) => {
                ValueId::Stored(StoredValueId::Component(*component_address))
            }
            SubstateAddress::NonFungible(resource_address, ..) => {
                ValueId::NonFungibles(*resource_address)
            }
            SubstateAddress::KeyValueEntry(kv_store_id, ..) => ValueId::kv_store_id(*kv_store_id),
        };

        // Get location
        // Note this must be run AFTER values are taken, otherwise there would be inconsistent readable_values state
        let (value_info, address_borrowed) = self
            .value_refs
            .get(&value_id)
            .map(|v| (v, None))
            .or_else(|| {
                // Allow global read access to any component info
                if let SubstateAddress::Component(component_address, ComponentOffset::Info) =
                    address
                {
                    if self.owned_values.contains_key(&value_id) {
                        return Some((
                            &REValueInfo {
                                location: REValueLocation::OwnedRoot,
                                visible: true,
                            },
                            None,
                        ));
                    } else if self
                        .track
                        .take_lock(*component_address, false, false)
                        .is_ok()
                    {
                        return Some((
                            &REValueInfo {
                                location: REValueLocation::Track { parent: None },
                                visible: true,
                            },
                            Some(component_address),
                        ));
                    }
                }

                None
            })
            .ok_or_else(|| RuntimeError::InvalidDataAccess(value_id))?;
        if !value_info.visible {
            return Err(RuntimeError::InvalidDataAccess(value_id));
        }
        let location = &value_info.location;

        // Read current value
        let (current_value, cur_children) = {
            let mut value_ref = location.to_ref_mut(
                &value_id,
                &mut self.owned_values,
                &mut self.frame_borrowed_values,
                &mut self.track,
            );
            let current_value = match &address {
                SubstateAddress::Component(.., offset) => match offset {
                    ComponentOffset::State => {
                        ScryptoValue::from_slice(value_ref.component().state())
                            .expect("Expected to decode")
                    }
                    ComponentOffset::Info => {
                        ScryptoValue::from_typed(&value_ref.component().info())
                    }
                },
                SubstateAddress::KeyValueEntry(.., key) => {
                    verify_stored_key(key)?;
                    value_ref.kv_store_get(&key.raw)
                }
                SubstateAddress::NonFungible(.., id) => value_ref.non_fungible_get(id),
            };
            let cur_children = to_stored_ids(current_value.value_ids())?;
            (current_value, cur_children)
        };

        // TODO: Remove, currently a hack to allow for global component info retrieval
        if let Some(component_address) = address_borrowed {
            self.track.release_lock(*component_address, false);
        }

        Ok((value_id, location.clone(), current_value, cur_children))
    }
}

impl<'p, 'g, W, I> SystemApi<'p, W, I> for CallFrame<'p, 'g, W, I>
where
    W: WasmEngine<I>,
    I: WasmInstance,
{
    fn invoke_snode(
        &mut self,
        snode_ref: SNodeRef,
        fn_ident: String,
        input: ScryptoValue,
    ) -> Result<ScryptoValue, RuntimeError> {
        if self.depth == MAX_CALL_DEPTH {
            return Err(RuntimeError::MaxCallDepthLimitReached);
        }

        self.cost_unit_counter
            .consume(
                self.fee_table
                    .system_api_cost(SystemApiCostingEntry::InvokeFunction {
                        receiver: &snode_ref,
                        input: &input,
                    }),
                "invoke_function",
            )
            .map_err(RuntimeError::CostingError)?;

        trace!(
            self,
            Level::Debug,
            "Invoking: {:?} {:?}",
            snode_ref,
            &fn_ident
        );

        // Prevent vaults/kvstores from being moved
        Self::process_call_data(&input)?;

        // Figure out what buckets and proofs to move from this process
        let values_to_take = input.value_ids();
        let (taken_values, mut missing) = self.take_available_values(values_to_take, false)?;
        let first_missing_value = missing.drain().nth(0);
        if let Some(missing_value) = first_missing_value {
            return Err(RuntimeError::ValueNotFound(missing_value));
        }

        let mut next_owned_values = HashMap::new();

        // Internal state update to taken values
        for (id, mut value) in taken_values {
            trace!(self, Level::Debug, "Sending value: {:?}", value);
            match &mut value {
                REValue::Proof(proof) => proof.change_to_restricted(),
                _ => {}
            }
            next_owned_values.insert(id, RefCell::new(value));
        }

        let mut locked_values = HashSet::new();
        let mut value_refs = HashMap::new();
        let mut next_borrowed_values = HashMap::new();

        // Authorization and state load
        let (loaded_snode, method_auths) = match &snode_ref {
            SNodeRef::TransactionProcessor => {
                // FIXME: only TransactionExecutor can invoke this function
                Ok((
                    SNodeExecution::Static(StaticSNodeState::TransactionProcessor),
                    vec![],
                ))
            }
            SNodeRef::PackageStatic => {
                Ok((SNodeExecution::Static(StaticSNodeState::Package), vec![]))
            }
            SNodeRef::ResourceStatic => {
                Ok((SNodeExecution::Static(StaticSNodeState::Resource), vec![]))
            }
            SNodeRef::Consumed(value_id) => {
                let value = self
                    .owned_values
                    .remove(value_id)
                    .ok_or(RuntimeError::ValueNotFound(*value_id))?
                    .into_inner();

                let method_auths = match &value {
                    REValue::Bucket(bucket) => {
                        let resource_address = bucket.resource_address();
                        self.track
                            .take_lock(resource_address, true, false)
                            .expect("Should not fail.");
                        locked_values.insert(resource_address.clone().into());
                        let resource_manager =
                            self.track.read_value(resource_address).resource_manager();
                        let method_auth = resource_manager.get_consuming_bucket_auth(&fn_ident);
                        value_refs.insert(
                            ValueId::Resource(resource_address),
                            REValueInfo {
                                location: REValueLocation::Track { parent: None },
                                visible: true,
                            },
                        );
                        value_refs.insert(
                            ValueId::NonFungibles(resource_address),
                            REValueInfo {
                                location: REValueLocation::Track { parent: None },
                                visible: true,
                            },
                        );
                        vec![method_auth.clone()]
                    }
                    REValue::Proof(_) => vec![],
                    REValue::Component { component, .. } => {
                        let package_address = component.package_address();
                        self.track
                            .take_lock(package_address, false, false)
                            .expect("Should not fail.");
                        locked_values.insert(package_address.clone().into());
                        value_refs.insert(
                            ValueId::Package(package_address),
                            REValueInfo {
                                location: REValueLocation::Track { parent: None },
                                visible: true,
                            },
                        );
                        vec![]
                    }
                    _ => return Err(RuntimeError::MethodDoesNotExist(fn_ident.clone())),
                };

                next_owned_values.insert(*value_id, RefCell::new(value));

                Ok((SNodeExecution::Consumed(*value_id), method_auths))
            }
            SNodeRef::SystemRef => {
                self.track
                    .take_lock(Address::System, true, false)
                    .expect("System access should never fail");
                locked_values.insert(Address::System);
                value_refs.insert(
                    ValueId::System,
                    REValueInfo {
                        location: REValueLocation::Track { parent: None },
                        visible: true,
                    },
                );
                let fn_str: &str = &fn_ident;
                let access_rules = match fn_str {
                    "set_epoch" => {
                        vec![MethodAuthorization::Protected(HardAuthRule::ProofRule(
                            HardProofRule::Require(HardResourceOrNonFungible::Resource(
                                SYSTEM_TOKEN,
                            )),
                        ))]
                    }
                    _ => vec![],
                };
                Ok((SNodeExecution::ValueRef(ValueId::System), access_rules))
            }
            SNodeRef::AuthZoneRef => {
                if let Some(auth_zone) = &self.auth_zone {
                    for resource_address in &input.resource_addresses {
                        self.track
                            .take_lock(resource_address.clone(), false, false)
                            .map_err(|e| match e {
                                TrackError::NotFound => {
                                    RuntimeError::ResourceManagerNotFound(resource_address.clone())
                                }
                                TrackError::Reentrancy => {
                                    panic!("Package reentrancy error should never occur.")
                                }
                            })?;
                        locked_values.insert(resource_address.clone().into());
                        value_refs.insert(
                            ValueId::Resource(resource_address.clone()),
                            REValueInfo {
                                location: REValueLocation::Track { parent: None },
                                visible: true,
                            },
                        );
                    }
                    let borrowed = auth_zone.borrow_mut();
                    Ok((SNodeExecution::AuthZone(borrowed), vec![]))
                } else {
                    Err(RuntimeError::AuthZoneDoesNotExist)
                }
            }
            SNodeRef::ResourceRef(resource_address) => {
                let value_id = ValueId::Resource(*resource_address);
                let address: Address = Address::Resource(*resource_address);
                self.track
                    .take_lock(address.clone(), true, false)
                    .map_err(|e| match e {
                        TrackError::NotFound => {
                            RuntimeError::ResourceManagerNotFound(resource_address.clone())
                        }
                        TrackError::Reentrancy => {
                            panic!("Resource call has caused reentrancy")
                        }
                    })?;
                locked_values.insert(address.clone());
                let resource_manager = self.track.read_value(address).resource_manager();
                let method_auth = resource_manager.get_auth(&fn_ident, &input).clone();
                value_refs.insert(
                    value_id.clone(),
                    REValueInfo {
                        location: REValueLocation::Track { parent: None },
                        visible: true,
                    },
                );
                value_refs.insert(
                    ValueId::NonFungibles(*resource_address),
                    REValueInfo {
                        location: REValueLocation::Track { parent: None },
                        visible: true,
                    },
                );

                Ok((SNodeExecution::ValueRef(value_id), vec![method_auth]))
            }
            SNodeRef::BucketRef(bucket_id) => {
                let value_id = ValueId::Transient(TransientValueId::Bucket(*bucket_id));
                let bucket_cell = self
                    .owned_values
                    .get(&value_id)
                    .ok_or(RuntimeError::BucketNotFound(bucket_id.clone()))?;
                let ref_mut = bucket_cell.borrow_mut();
                next_borrowed_values.insert(value_id.clone(), ref_mut);
                value_refs.insert(
                    value_id.clone(),
                    REValueInfo {
                        location: REValueLocation::BorrowedRoot,
                        visible: true,
                    },
                );

                Ok((SNodeExecution::ValueRef(value_id), vec![]))
            }
            SNodeRef::ProofRef(proof_id) => {
                let value_id = ValueId::Transient(TransientValueId::Proof(*proof_id));
                let proof_cell = self
                    .owned_values
                    .get(&value_id)
                    .ok_or(RuntimeError::ProofNotFound(proof_id.clone()))?;
                let ref_mut = proof_cell.borrow_mut();
                next_borrowed_values.insert(value_id.clone(), ref_mut);
                value_refs.insert(
                    value_id.clone(),
                    REValueInfo {
                        location: REValueLocation::BorrowedRoot,
                        visible: true,
                    },
                );
                Ok((SNodeExecution::ValueRef(value_id), vec![]))
            }
            SNodeRef::WorktopRef => {
                let value_id = ValueId::Transient(TransientValueId::Worktop);
                let worktop_cell = self
                    .owned_values
                    .get(&value_id)
                    .ok_or(RuntimeError::ValueNotFound(value_id))?;

                let ref_mut = worktop_cell.borrow_mut();
                next_borrowed_values.insert(value_id.clone(), ref_mut);
                value_refs.insert(
                    value_id.clone(),
                    REValueInfo {
                        location: REValueLocation::BorrowedRoot,
                        visible: true,
                    },
                );

                for resource_address in &input.resource_addresses {
                    self.track
                        .take_lock(resource_address.clone(), false, false)
                        .map_err(|e| match e {
                            TrackError::NotFound => {
                                RuntimeError::ResourceManagerNotFound(resource_address.clone())
                            }
                            TrackError::Reentrancy => {
                                panic!("Package reentrancy error should never occur.")
                            }
                        })?;
                    locked_values.insert(resource_address.clone().into());
                    value_refs.insert(
                        ValueId::Resource(resource_address.clone()),
                        REValueInfo {
                            location: REValueLocation::Track { parent: None },
                            visible: true,
                        },
                    );
                }

                Ok((SNodeExecution::ValueRef(value_id), vec![]))
            }
            SNodeRef::Scrypto(actor) => match actor {
                ScryptoActor::Blueprint(package_address, blueprint_name) => {
                    self.track
                        .take_lock(package_address.clone(), false, false)
                        .map_err(|e| match e {
                            TrackError::NotFound => RuntimeError::PackageNotFound(*package_address),
                            TrackError::Reentrancy => {
                                panic!("Package reentrancy error should never occur.")
                            }
                        })?;
                    locked_values.insert(package_address.clone().into());
                    let package = self.track.read_value(package_address.clone()).package();
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
                    Ok((
                        SNodeExecution::Scrypto(
                            ScryptoActorInfo::blueprint(
                                package_address.clone(),
                                blueprint_name.clone(),
                            ),
                            package_address.clone(),
                        ),
                        vec![],
                    ))
                }
                ScryptoActor::Component(component_address) => {
                    let component_address = *component_address;

                    // Find value
                    let stored_value_id = StoredValueId::Component(component_address);
                    let value_id = ValueId::Stored(stored_value_id.clone());
                    let cur_location = if self.owned_values.contains_key(&value_id) {
                        &REValueLocation::OwnedRoot
                    } else if let Some(REValueInfo { location, .. }) =
                        self.value_refs.get(&value_id)
                    {
                        location
                    } else {
                        &REValueLocation::Track { parent: None }
                    };

                    // Lock values and setup next frame
                    let next_frame_location = match cur_location {
                        REValueLocation::Track { parent } => {
                            let address = if let Some(parent) = parent {
                                Address::LocalComponent(*parent, component_address)
                            } else {
                                Address::GlobalComponent(component_address)
                            };

                            self.track.take_lock(address.clone(), true, false).map_err(
                                |e| match e {
                                    TrackError::NotFound => {
                                        RuntimeError::ComponentNotFound(component_address)
                                    }
                                    TrackError::Reentrancy => {
                                        RuntimeError::ComponentReentrancy(component_address)
                                    }
                                },
                            )?;
                            locked_values.insert(address.clone());
                            REValueLocation::Track {
                                parent: parent.clone(),
                            }
                        }
                        REValueLocation::OwnedRoot | REValueLocation::Borrowed { .. } => {
                            let owned_ref = cur_location.to_owned_ref_mut(
                                &value_id,
                                &mut self.owned_values,
                                &mut self.frame_borrowed_values,
                            );
                            next_borrowed_values.insert(value_id, owned_ref);
                            REValueLocation::BorrowedRoot
                        }
                        _ => panic!("Unexpected"),
                    };

                    let actor_info = {
                        let value_ref = next_frame_location.to_ref(
                            &value_id,
                            &mut next_owned_values,
                            &mut next_borrowed_values,
                            &mut self.track,
                        );
                        let component = value_ref.component();
                        ScryptoActorInfo::component(
                            component.package_address(),
                            component.blueprint_name().to_string(),
                            component_address,
                        )
                    };

                    // Retrieve Method Authorization
                    let (method_auths, package_address) = {
                        let package_address = actor_info.package_address().clone();
                        let blueprint_name = actor_info.blueprint_name().to_string();
                        self.track
                            .take_lock(package_address, false, false)
                            .expect("Should never fail");
                        locked_values.insert(package_address.clone().into());
                        let package = self.track.read_value(package_address).package();
                        let abi = package
                            .blueprint_abi(&blueprint_name)
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

                        let method_auths = {
                            let value_ref = next_frame_location.to_ref(
                                &value_id,
                                &next_owned_values,
                                &next_borrowed_values,
                                &self.track,
                            );
                            value_ref
                                .component()
                                .method_authorization(&abi.structure, &fn_ident)
                        };

                        (method_auths, package_address)
                    };

                    value_refs.insert(
                        value_id,
                        REValueInfo {
                            location: next_frame_location,
                            visible: true,
                        },
                    );

                    Ok((
                        SNodeExecution::Scrypto(actor_info, package_address),
                        method_auths,
                    ))
                }
            },
            SNodeRef::Component(component_address) => {
                let component_address = *component_address;

                // Find value
                let value_id = ValueId::Stored(StoredValueId::Component(component_address));
                let cur_location = if self.owned_values.contains_key(&value_id) {
                    REValueLocation::OwnedRoot
                } else {
                    return Err(RuntimeError::NotSupported);
                };

                // Setup next frame
                match cur_location {
                    REValueLocation::OwnedRoot => {
                        let owned_ref = cur_location.to_owned_ref_mut(
                            &value_id,
                            &mut self.owned_values,
                            &mut self.frame_borrowed_values,
                        );

                        // Lock package
                        let package_address = owned_ref.component().package_address();
                        self.track
                            .take_lock(package_address, false, false)
                            .map_err(|e| match e {
                                TrackError::NotFound => panic!("Should exist"),
                                TrackError::Reentrancy => RuntimeError::PackageReentrancy,
                            })?;
                        locked_values.insert(package_address.into());
                        value_refs.insert(
                            ValueId::Package(package_address),
                            REValueInfo {
                                location: REValueLocation::Track { parent: None },
                                visible: true,
                            },
                        );

                        next_borrowed_values.insert(value_id, owned_ref);
                        value_refs.insert(
                            value_id,
                            REValueInfo {
                                location: REValueLocation::BorrowedRoot,
                                visible: true,
                            },
                        );
                    }
                    _ => panic!("Unexpected"),
                }

                Ok((SNodeExecution::ValueRef(value_id), vec![]))
            }

            SNodeRef::VaultRef(vault_id) => {
                // Find value
                let value_id = ValueId::vault_id(*vault_id);
                let cur_location = if self.owned_values.contains_key(&value_id) {
                    &REValueLocation::OwnedRoot
                } else {
                    let maybe_value_ref = self.value_refs.get(&value_id);
                    maybe_value_ref
                        .map(|info| &info.location)
                        .ok_or(RuntimeError::ValueNotFound(ValueId::vault_id(*vault_id)))?
                };

                // Lock values and setup next frame
                let next_location = {
                    // Lock Vault
                    let next_location = match cur_location {
                        REValueLocation::Track { parent } => {
                            let vault_address = (parent.unwrap().clone(), *vault_id);
                            self.track
                                .take_lock(vault_address, true, false)
                                .expect("Should never fail.");
                            locked_values.insert(vault_address.into());
                            REValueLocation::Track {
                                parent: parent.clone(),
                            }
                        }
                        REValueLocation::OwnedRoot
                        | REValueLocation::Owned { .. }
                        | REValueLocation::Borrowed { .. } => {
                            let owned_ref = cur_location.to_owned_ref_mut(
                                &value_id,
                                &mut self.owned_values,
                                &mut self.frame_borrowed_values,
                            );
                            next_borrowed_values.insert(value_id.clone(), owned_ref);
                            REValueLocation::BorrowedRoot
                        }
                        _ => panic!("Unexpected vault location {:?}", cur_location),
                    };

                    // Lock Resource
                    let resource_address = {
                        let value_ref = next_location.to_ref(
                            &value_id,
                            &mut next_owned_values,
                            &mut next_borrowed_values,
                            &mut self.track,
                        );
                        value_ref.vault().resource_address()
                    };
                    self.track
                        .take_lock(resource_address, true, false)
                        .expect("Should never fail.");
                    locked_values.insert(resource_address.into());

                    next_location
                };

                // Retrieve Method Authorization
                let method_auth = {
                    let resource_address = {
                        let value_ref = next_location.to_ref(
                            &value_id,
                            &mut next_owned_values,
                            &mut next_borrowed_values,
                            &mut self.track,
                        );
                        value_ref.vault().resource_address()
                    };
                    let resource_manager =
                        self.track.read_value(resource_address).resource_manager();
                    resource_manager.get_vault_auth(&fn_ident).clone()
                };

                value_refs.insert(
                    value_id.clone(),
                    REValueInfo {
                        location: next_location,
                        visible: true,
                    },
                );

                Ok((SNodeExecution::ValueRef(value_id), vec![method_auth]))
            }
        }?;

        // Authorization check
        if !method_auths.is_empty() {
            let mut auth_zones = Vec::new();
            if let Some(self_auth_zone) = &self.auth_zone {
                auth_zones.push(self_auth_zone.borrow());
            }

            match &loaded_snode {
                // Resource auth check includes caller
                SNodeExecution::Scrypto(..)
                | SNodeExecution::ValueRef(ValueId::Resource(..), ..)
                | SNodeExecution::ValueRef(ValueId::Stored(StoredValueId::VaultId(..)), ..)
                | SNodeExecution::Consumed(ValueId::Transient(TransientValueId::Bucket(..))) => {
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

        // start a new frame
        let mut frame = CallFrame::new(
            self.transaction_hash,
            self.depth + 1,
            self.trace,
            self.track,
            self.wasm_engine,
            self.wasm_instrumenter,
            self.cost_unit_counter,
            self.fee_table,
            match loaded_snode {
                SNodeExecution::Scrypto(..)
                | SNodeExecution::Static(StaticSNodeState::TransactionProcessor) => {
                    Some(RefCell::new(AuthZone::new()))
                }
                _ => None,
            },
            next_owned_values,
            value_refs,
            next_borrowed_values,
            self.auth_zone.as_ref(),
        );

        // invoke the main function
        let (result, received_values) = frame.run(snode_ref, loaded_snode, &fn_ident, input)?;
        drop(frame);

        // Release locked addresses
        for l in locked_values {
            self.track.release_lock(l, false);
        }

        // move buckets and proofs to this process.
        for (id, value) in received_values {
            trace!(self, Level::Debug, "Received value: {:?}", value);
            self.owned_values.insert(id, RefCell::new(value));
        }

        Ok(result)
    }

    fn borrow_value(
        &mut self,
        value_id: &ValueId,
    ) -> Result<REValueRef<'_, 'p>, CostUnitCounterError> {
        self.cost_unit_counter.consume(
            self.fee_table.system_api_cost({
                match value_id {
                    ValueId::Transient(_) => SystemApiCostingEntry::BorrowLocal,
                    ValueId::Stored(_) => SystemApiCostingEntry::BorrowGlobal {
                        // TODO: figure out loaded state and size
                        loaded: false,
                        size: 0,
                    },
                    ValueId::Resource(_) => SystemApiCostingEntry::BorrowGlobal {
                        // TODO: figure out loaded state and size
                        loaded: false,
                        size: 0,
                    },
                    ValueId::Package(_) => SystemApiCostingEntry::BorrowGlobal {
                        // TODO: figure out loaded state and size
                        loaded: false,
                        size: 0,
                    },
                    ValueId::System => SystemApiCostingEntry::BorrowGlobal {
                        // TODO: figure out loaded state and size
                        loaded: false,
                        size: 0,
                    },
                    ValueId::NonFungibles(..) => SystemApiCostingEntry::BorrowGlobal {
                        // TODO: figure out loaded state and size
                        loaded: false,
                        size: 0,
                    },
                }
            }),
            "borrow",
        )?;

        let info = self
            .value_refs
            .get(value_id)
            .expect(&format!("{:?} is unknown.", value_id));
        if !info.visible {
            panic!("Trying to read value which is not visible.")
        }

        Ok(info.location.to_ref(
            value_id,
            &self.owned_values,
            &self.frame_borrowed_values,
            &self.track,
        ))
    }

    fn borrow_value_mut(
        &mut self,
        value_id: &ValueId,
    ) -> Result<RENativeValueRef<'p>, CostUnitCounterError> {
        self.cost_unit_counter.consume(
            self.fee_table.system_api_cost({
                match value_id {
                    ValueId::Transient(_) => SystemApiCostingEntry::BorrowLocal,
                    ValueId::Stored(_) => SystemApiCostingEntry::BorrowGlobal {
                        // TODO: figure out loaded state and size
                        loaded: false,
                        size: 0,
                    },
                    ValueId::Resource(_) => SystemApiCostingEntry::BorrowGlobal {
                        // TODO: figure out loaded state and size
                        loaded: false,
                        size: 0,
                    },
                    ValueId::Package(_) => SystemApiCostingEntry::BorrowGlobal {
                        // TODO: figure out loaded state and size
                        loaded: false,
                        size: 0,
                    },
                    ValueId::System => SystemApiCostingEntry::BorrowGlobal {
                        // TODO: figure out loaded state and size
                        loaded: false,
                        size: 0,
                    },
                    ValueId::NonFungibles(..) => SystemApiCostingEntry::BorrowGlobal {
                        // TODO: figure out loaded state and size
                        loaded: false,
                        size: 0,
                    },
                }
            }),
            "borrow",
        )?;

        let info = self
            .value_refs
            .get(value_id)
            .expect(&format!("Value should exist {:?}", value_id));
        if !info.visible {
            panic!("Trying to read value which is not visible.")
        }

        Ok(info.location.borrow_native_ref(
            value_id,
            &mut self.owned_values,
            &mut self.frame_borrowed_values,
            &mut self.track,
        ))
    }

    fn return_value_mut(
        &mut self,
        value_id: ValueId,
        val_ref: RENativeValueRef<'p>,
    ) -> Result<(), CostUnitCounterError> {
        self.cost_unit_counter.consume(
            self.fee_table.system_api_cost({
                match value_id {
                    // TODO: get size of the value
                    ValueId::Transient(_) => SystemApiCostingEntry::ReturnLocal,
                    ValueId::Stored(_) => SystemApiCostingEntry::ReturnGlobal { size: 0 },
                    ValueId::Resource(_) => SystemApiCostingEntry::ReturnGlobal { size: 0 },
                    ValueId::Package(_) => SystemApiCostingEntry::ReturnGlobal { size: 0 },
                    ValueId::System => SystemApiCostingEntry::ReturnGlobal { size: 0 },
                    ValueId::NonFungibles(..) => SystemApiCostingEntry::ReturnGlobal { size: 0 },
                }
            }),
            "return",
        )?;

        val_ref.return_to_location(
            value_id,
            &mut self.owned_values,
            &mut self.frame_borrowed_values,
            &mut self.track,
        );
        Ok(())
    }

    fn drop_value(&mut self, value_id: &ValueId) -> Result<REValue, CostUnitCounterError> {
        // TODO: costing

        Ok(self.owned_values.remove(&value_id).unwrap().into_inner())
    }

    fn create_value<V: Into<REValueByComplexity>>(
        &mut self,
        v: V,
    ) -> Result<ValueId, RuntimeError> {
        self.cost_unit_counter
            .consume(
                self.fee_table
                    .system_api_cost(SystemApiCostingEntry::Create {
                        size: 0, // TODO: get size of the value
                    }),
                "create",
            )
            .map_err(RuntimeError::CostingError)?;

        let value_by_complexity = v.into();
        let id = match value_by_complexity {
            REValueByComplexity::Primitive(REPrimitiveValue::Bucket(..)) => {
                let bucket_id = self.track.new_bucket_id();
                ValueId::Transient(TransientValueId::Bucket(bucket_id))
            }
            REValueByComplexity::Primitive(REPrimitiveValue::Proof(..)) => {
                let proof_id = self.track.new_proof_id();
                ValueId::Transient(TransientValueId::Proof(proof_id))
            }
            REValueByComplexity::Primitive(REPrimitiveValue::Worktop(..)) => {
                ValueId::Transient(TransientValueId::Worktop)
            }
            REValueByComplexity::Primitive(REPrimitiveValue::Vault(..)) => {
                let vault_id = self.track.new_vault_id();
                ValueId::Stored(StoredValueId::VaultId(vault_id))
            }
            REValueByComplexity::Primitive(REPrimitiveValue::KeyValue(..)) => {
                let kv_store_id = self.track.new_kv_store_id();
                ValueId::Stored(StoredValueId::KeyValueStoreId(kv_store_id))
            }
            REValueByComplexity::Primitive(REPrimitiveValue::Package(..)) => {
                let package_address = self.track.new_package_address();
                ValueId::Package(package_address)
            }
            REValueByComplexity::Primitive(REPrimitiveValue::Resource(..)) => {
                let resource_address = self.track.new_resource_address();
                ValueId::Resource(resource_address)
            }
            REValueByComplexity::Primitive(REPrimitiveValue::NonFungibles(
                resource_address,
                ..,
            )) => ValueId::NonFungibles(resource_address),
            REValueByComplexity::Complex(REComplexValue::Component(ref component)) => {
                let component_address = self.track.new_component_address(component);
                ValueId::Stored(StoredValueId::Component(component_address))
            }
        };

        let re_value = match value_by_complexity {
            REValueByComplexity::Primitive(primitive) => primitive.into(),
            REValueByComplexity::Complex(complex) => {
                let children = complex.get_children()?;
                let (child_values, mut missing) = self.take_available_values(children, true)?;
                let first_missing_value = missing.drain().nth(0);
                if let Some(missing_value) = first_missing_value {
                    return Err(RuntimeError::ValueNotFound(missing_value));
                }
                complex.into_re_value(child_values)
            }
        };
        self.owned_values.insert(id, RefCell::new(re_value));

        match id {
            ValueId::Stored(StoredValueId::KeyValueStoreId(..))
            | ValueId::Resource(..)
            | ValueId::NonFungibles(..) => {
                self.value_refs.insert(
                    id.clone(),
                    REValueInfo {
                        location: REValueLocation::OwnedRoot,
                        visible: true,
                    },
                );
            }
            _ => {}
        }

        Ok(id)
    }

    fn globalize_value(&mut self, value_id: &ValueId) -> Result<(), CostUnitCounterError> {
        self.cost_unit_counter.consume(
            self.fee_table
                .system_api_cost(SystemApiCostingEntry::Globalize {
                    size: 0, // TODO: get size of the value
                }),
            "globalize",
        )?;

        let mut values = HashSet::new();
        values.insert(value_id.clone());
        let (taken_values, missing) = self.take_available_values(values, false).unwrap();
        assert!(missing.is_empty());
        assert!(taken_values.len() == 1);
        let value = taken_values.into_values().nth(0).unwrap();

        let (substate, maybe_child_values, maybe_non_fungibles) = match value {
            REValue::Component {
                component,
                child_values,
            } => (
                SubstateValue::Component(component),
                Some(child_values),
                None,
            ),
            REValue::Package(package) => (SubstateValue::Package(package), None, None),
            REValue::Resource(resource_manager) => {
                let non_fungibles =
                    if matches!(resource_manager.resource_type(), ResourceType::NonFungible) {
                        let resource_address: ResourceAddress = value_id.clone().into();
                        let re_value = self
                            .owned_values
                            .remove(&ValueId::NonFungibles(resource_address))
                            .unwrap()
                            .into_inner();
                        let non_fungibles: HashMap<NonFungibleId, NonFungible> = re_value.into();
                        Some(non_fungibles)
                    } else {
                        None
                    };
                (
                    SubstateValue::Resource(resource_manager),
                    None,
                    non_fungibles,
                )
            }
            _ => panic!("Not expected"),
        };

        let address = match value_id {
            ValueId::Stored(StoredValueId::Component(component_address)) => {
                Address::GlobalComponent(*component_address)
            }
            ValueId::Package(package_address) => Address::Package(*package_address),
            ValueId::Resource(resource_address) => Address::Resource(*resource_address),
            _ => panic!("Expected to be a component address"),
        };

        self.track.create_uuid_value(address.clone(), substate);

        if let Some(child_values) = maybe_child_values {
            let mut to_store_values = HashMap::new();
            for (id, cell) in child_values.into_iter() {
                to_store_values.insert(id, cell.into_inner());
            }
            self.track
                .insert_objects_into_component(to_store_values, address.clone().into());
        }

        if let Some(non_fungibles) = maybe_non_fungibles {
            let resource_address: ResourceAddress = address.clone().into();
            self.track
                .create_non_fungible_space(resource_address.clone());
            let parent_address = Address::NonFungibleSet(resource_address.clone());
            for (id, non_fungible) in non_fungibles {
                self.track.set_key_value(
                    parent_address.clone(),
                    id.to_vec(),
                    SubstateValue::NonFungible(Some(non_fungible)),
                );
            }
        }

        Ok(())
    }

    fn remove_value_data(
        &mut self,
        address: SubstateAddress,
    ) -> Result<ScryptoValue, RuntimeError> {
        let (value_id, location, current_value, cur_children) =
            self.read_value_internal(&address)?;
        if !cur_children.is_empty() {
            return Err(RuntimeError::ValueNotAllowed);
        }

        // Write values
        let mut value_ref = location.to_ref_mut(
            &value_id,
            &mut self.owned_values,
            &mut self.frame_borrowed_values,
            &mut self.track,
        );
        match address {
            SubstateAddress::Component(..) => {
                panic!("Should not get here");
            }
            SubstateAddress::KeyValueEntry(..) => {
                panic!("Should not get here");
            }
            SubstateAddress::NonFungible(.., id) => value_ref.non_fungible_remove(&id),
        }

        Ok(current_value)
    }

    fn read_value_data(&mut self, address: SubstateAddress) -> Result<ScryptoValue, RuntimeError> {
        self.cost_unit_counter
            .consume(
                self.fee_table.system_api_cost(SystemApiCostingEntry::Read {
                    size: 0, // TODO: get size of the value
                }),
                "read",
            )
            .map_err(RuntimeError::CostingError)?;

        let (value_id, parent_location, current_value, cur_children) =
            self.read_value_internal(&address)?;
        for stored_value_id in cur_children {
            let child_location = parent_location.child(value_id.clone());

            // Extend current readable space when kv stores are found
            let visible = matches!(stored_value_id, StoredValueId::KeyValueStoreId(..));
            let child_info = REValueInfo {
                location: child_location,
                visible,
            };
            self.value_refs
                .insert(ValueId::Stored(stored_value_id), child_info);
        }
        Ok(current_value)
    }

    fn write_value_data(
        &mut self,
        address: SubstateAddress,
        value: ScryptoValue,
    ) -> Result<(), RuntimeError> {
        self.cost_unit_counter
            .consume(
                self.fee_table
                    .system_api_cost(SystemApiCostingEntry::Write {
                        size: 0, // TODO: get size of the value
                    }),
                "write",
            )
            .map_err(RuntimeError::CostingError)?;

        // If write, take values from current frame
        let (taken_values, missing) = {
            let value_ids = value.value_ids();
            match address {
                SubstateAddress::KeyValueEntry(..)
                | SubstateAddress::Component(_, ComponentOffset::State) => {
                    self.take_available_values(value_ids, true)?
                }
                SubstateAddress::Component(_, ComponentOffset::Info) => {
                    return Err(RuntimeError::InvalidDataWrite)
                }
                SubstateAddress::NonFungible(..) => {
                    if !value_ids.is_empty() {
                        return Err(RuntimeError::ValueNotAllowed);
                    }
                    (HashMap::new(), HashSet::new())
                }
            }
        };

        let (value_id, location, _current_value, cur_children) =
            self.read_value_internal(&address)?;

        // Fulfill method
        let missing = to_stored_ids(missing)?;
        verify_stored_value_update(&cur_children, &missing)?;

        // TODO: verify against some schema

        // Write values
        let mut value_ref = location.to_ref_mut(
            &value_id,
            &mut self.owned_values,
            &mut self.frame_borrowed_values,
            &mut self.track,
        );
        match address {
            SubstateAddress::Component(.., offset) => match offset {
                ComponentOffset::State => value_ref.component_put(value, taken_values),
                ComponentOffset::Info => {
                    return Err(RuntimeError::InvalidDataWrite);
                }
            },
            SubstateAddress::KeyValueEntry(.., key) => {
                value_ref.kv_store_put(key.raw, value, taken_values);
            }
            SubstateAddress::NonFungible(.., id) => value_ref.non_fungible_put(id, value),
        }

        Ok(())
    }

    fn transaction_hash(&mut self) -> Result<Hash, CostUnitCounterError> {
        self.cost_unit_counter.consume(
            self.fee_table
                .system_api_cost(SystemApiCostingEntry::ReadTransactionHash),
            "read_transaction_hash",
        )?;
        Ok(self.track.transaction_hash())
    }

    fn transaction_network(&mut self) -> Result<Network, CostUnitCounterError> {
        self.cost_unit_counter.consume(
            self.fee_table
                .system_api_cost(SystemApiCostingEntry::ReadTransactionHash),
            "read_transaction_network",
        )?;
        Ok(self.track.transaction_network())
    }

    fn generate_uuid(&mut self) -> Result<u128, CostUnitCounterError> {
        self.cost_unit_counter.consume(
            self.fee_table
                .system_api_cost(SystemApiCostingEntry::GenerateUuid),
            "generate_uuid",
        )?;
        Ok(self.track.new_uuid())
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
                self.owned_values
                    .get(&ValueId::Transient(TransientValueId::Proof(*proof_id)))
                    .map(|p| match p.borrow().deref() {
                        REValue::Proof(proof) => proof.clone(),
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

    fn cost_unit_counter(&mut self) -> &mut CostUnitCounter {
        self.cost_unit_counter
    }

    fn fee_table(&self) -> &FeeTable {
        self.fee_table
    }

    fn pay_fee(&mut self, vault_id: VaultId, amount: Decimal) -> Result<(), RuntimeError> {
        let value_id = ValueId::vault_id(vault_id);
        if self.owned_values.contains_key(&value_id) {
            Err(RuntimeError::PayFeeFailure(
                "Attempted to locked fee on a local vault".to_owned(),
            ))
        } else if let Some(r) = self.value_refs.get_mut(&value_id) {
            match r.location {
                REValueLocation::Track { parent } => {
                    // 1. Update the substate
                    let address =
                        Address::Vault(parent.expect("TODO: is this a safe unwrap?"), vault_id);
                    self.track
                        .take_lock(address.clone(), true, true)
                        .map_err(|e| match e {
                            TrackError::NotFound => RuntimeError::ValueNotFound(value_id),
                            TrackError::Reentrancy => panic!("Vault reentrancy should never occur"),
                        })?;
                    let mut value = self.track.take_value(address.clone());
                    let (liquid, locked) = value.vault_mut();
                    if liquid.resource_address() != RADIX_TOKEN {
                        return Err(RuntimeError::PayFeeFailure(
                            "Attempted to pay non-XRD as fee".to_owned(),
                        ));
                    }
                    let fee = liquid.take(amount).map_err(RuntimeError::VaultError)?;
                    match locked {
                        Some(existing) => {
                            existing
                                .put(fee)
                                .expect("Combining XRD fees should always succeed");
                        }
                        None => {
                            *locked = Some(fee);
                        }
                    }
                    self.track.write_value(address.clone(), value);
                    self.track.release_lock(address.clone(), true);

                    // 2. Credit cost units
                    // TODO: add xrd/cost unit conversion
                    self.cost_unit_counter
                        .repay(100)
                        .map_err(RuntimeError::CostingError)?;

                    Ok(())
                }
                _ => Err(RuntimeError::PayFeeFailure(
                    "Referenced vault is not in track".to_owned(),
                )),
            }
        } else {
            Err(RuntimeError::PayFeeFailure("Vault not found".to_owned()))
        }
    }
}
