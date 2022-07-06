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
use scrypto::core::{SNodeRef, ScryptoActor};
use scrypto::engine::types::*;
use scrypto::prelude::ComponentOffset;
use scrypto::resource::AuthZoneClearInput;
use scrypto::values::*;
use transaction::validation::*;

use crate::engine::*;
use crate::fee::*;
use crate::ledger::*;
use crate::model::*;
use crate::wasm::*;

/// A call frame is the basic unit that forms a transaction call stack, which keeps track of the
/// owned objects by this function.
pub struct CallFrame<
    'p, // parent lifetime
    't, // Track lifetime
    's, // Substate store lifetime
    'w, // WASM engine lifetime
    'c, // Costing lifetime
    S,  // Substore store type
    W,  // WASM engine type
    I,  // WASM instance type
> where
    S: ReadableSubstateStore,
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
    track: &'t mut Track<'s, S>,
    /// Wasm engine
    wasm_engine: &'w mut W,
    /// Wasm Instrumenter
    wasm_instrumenter: &'w mut WasmInstrumenter,

    /// Remaining cost unit counter
    cost_unit_counter: &'c mut CostUnitCounter,
    /// Fee table
    fee_table: &'c FeeTable,

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

    // TODO: Remove, currently required here due to the awkwardness of the current create_resource
    locked_resmans: HashSet<Address>,

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

    fn borrow_native_ref<'borrowed, S: ReadableSubstateStore>(
        &self,
        value_id: &ValueId,
        owned_values: &mut HashMap<ValueId, RefCell<REValue>>,
        borrowed_values: &mut HashMap<ValueId, RefMut<'borrowed, REValue>>,
        track: &mut Track<S>,
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
                let stored_value_id = match value_id {
                    ValueId::Stored(stored_value_id) => stored_value_id,
                    _ => panic!("Unexpected value id"),
                };
                children.get_child(ancestors, stored_value_id)
            },
            REValueLocation::Borrowed { root, ancestors } => unsafe {
                let borrowed = borrowed_values.get(root).unwrap();
                let stored_value_id = match value_id {
                    ValueId::Stored(stored_value_id) => stored_value_id,
                    _ => panic!("Unexpected value id"),
                };
                borrowed
                    .get_children_store()
                    .unwrap()
                    .get_child(ancestors, stored_value_id)
            },
            _ => panic!("Not an owned ref"),
        }
    }

    fn to_ref<'a, 'p, 's, S: ReadableSubstateStore>(
        &self,
        value_id: &ValueId,
        owned_values: &'a HashMap<ValueId, RefCell<REValue>>,
        borrowed_values: &'a HashMap<ValueId, RefMut<'p, REValue>>,
        track: &'a Track<'s, S>,
    ) -> REValueRef<'a, 'p, 's, S> {
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
                let stored_value_id = match value_id {
                    ValueId::Stored(stored_value_id) => stored_value_id,
                    _ => panic!("Unexpected value id"),
                };
                children.get_child_mut(ancestors, stored_value_id)
            }
            REValueLocation::Borrowed { root, ancestors } => {
                let borrowed = borrowed_values.get_mut(root).unwrap();
                let stored_value_id = match value_id {
                    ValueId::Stored(stored_value_id) => stored_value_id,
                    _ => panic!("Unexpected value id"),
                };
                borrowed
                    .get_children_store_mut()
                    .unwrap()
                    .get_child_mut(ancestors, stored_value_id)
            }
            _ => panic!("Not an owned ref"),
        }
    }

    fn to_ref_mut<'a, 'borrowed, 'c, 's, S: ReadableSubstateStore>(
        &self,
        value_id: &ValueId,
        owned_values: &'a mut HashMap<ValueId, RefCell<REValue>>,
        borrowed_values: &'a mut HashMap<ValueId, RefMut<'borrowed, REValue>>,
        track: &'c mut Track<'s, S>,
    ) -> REValueRefMut<'a, 'borrowed, 'c, 's, S> {
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
                    _ => panic!("Unexpected value id"),
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
            RENativeValueRef::Track(_address, value) => value.vault_mut(),
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
            RENativeValueRef::Track(_address, value) => value.resource_manager_mut(),
            _ => panic!("Unexpected"),
        }
    }

    pub fn return_to_location<'a, S: ReadableSubstateStore>(
        self,
        value_id: ValueId,
        owned_values: &'a mut HashMap<ValueId, RefCell<REValue>>,
        borrowed_values: &mut HashMap<ValueId, RefMut<'borrowed, REValue>>,
        track: &mut Track<S>,
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

pub enum REValueRef<'f, 'p, 's, S: ReadableSubstateStore> {
    Owned(Ref<'f, REValue>),
    Borrowed(&'f RefMut<'p, REValue>),
    Track(&'f Track<'s, S>, Address),
}

impl<'f, 'p, 's, S: ReadableSubstateStore> REValueRef<'f, 'p, 's, S> {
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

pub enum REValueRefMut<'a, 'b, 'c, 's, S: ReadableSubstateStore> {
    Owned(RefMut<'a, REValue>),
    Borrowed(&'a mut RefMut<'b, REValue>),
    Track(&'c mut Track<'s, S>, Address),
}

impl<'a, 'b, 'c, 's, S: ReadableSubstateStore> REValueRefMut<'a, 'b, 'c, 's, S> {
    fn kv_store_put(
        &mut self,
        key: Vec<u8>,
        value: ScryptoValue,
        to_store: HashMap<StoredValueId, REValue>,
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

    fn component_get_state(&self) -> ScryptoValue {
        match self {
            REValueRefMut::Track(track, address) => {
                let component_val = track.read_value(address.clone());
                let component = component_val.component();
                return ScryptoValue::from_slice(component.state()).expect("Expected to decode");
            }
            REValueRefMut::Borrowed(owned) => {
                let component = owned.component();
                return ScryptoValue::from_slice(component.state()).expect("Expected to decode");
            }
            _ => panic!("Unexpected component ref"),
        }
    }

    fn component_put(&mut self, value: ScryptoValue, to_store: HashMap<StoredValueId, REValue>) {
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

    fn component_info(&mut self) -> (PackageAddress, String) {
        match self {
            REValueRefMut::Owned(owned) => {
                let component = &owned.component_mut();
                (
                    component.package_address().clone(),
                    component.blueprint_name().to_string(),
                )
            }
            REValueRefMut::Borrowed(borrowed) => {
                let component = &borrowed.component();
                (
                    component.package_address().clone(),
                    component.blueprint_name().to_string(),
                )
            }
            REValueRefMut::Track(track, address) => {
                let component_val = track.read_value(address.clone());
                let component = component_val.component();
                (
                    component.package_address().clone(),
                    component.blueprint_name().to_string(),
                )
            }
        }
    }

    fn vault_resource_address(&mut self) -> ResourceAddress {
        match self {
            REValueRefMut::Owned(re_value) => re_value.vault().resource_address(),
            REValueRefMut::Borrowed(re_value) => re_value.vault().resource_address(),
            REValueRefMut::Track(track, address) => {
                track.read_value(address.clone()).vault().resource_address()
            }
        }
    }
}

pub enum StaticSNodeState {
    Package,
    Resource,
    System,
    TransactionProcessor,
}

pub enum SNodeExecution<'a> {
    Static(StaticSNodeState),
    Consumed(ValueId),
    AuthZone(RefMut<'a, AuthZone>),
    ValueRef(ValueId),
    Scrypto(ScryptoActorInfo, PackageAddress),
}

pub enum DataInstruction {
    Read,
    Write(ScryptoValue),
}

pub enum SubstateAddress {
    KeyValueEntry(KeyValueStoreId, ScryptoValue),
    Component(ComponentAddress, ComponentOffset),
}

impl<'p, 't, 's, 'w, 'c, S, W, I> CallFrame<'p, 't, 's, 'w, 'c, S, W, I>
where
    S: ReadableSubstateStore,
    W: WasmEngine<I>,
    I: WasmInstance,
{
    pub fn new_root(
        verbose: bool,
        transaction_hash: Hash,
        signer_public_keys: Vec<EcdsaPublicKey>,
        track: &'t mut Track<'s, S>,
        wasm_engine: &'w mut W,
        wasm_instrumenter: &'w mut WasmInstrumenter,
        cost_unit_counter: &'c mut CostUnitCounter,
        fee_table: &'c FeeTable,
    ) -> Self {
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
        track: &'t mut Track<'s, S>,
        wasm_engine: &'w mut W,
        wasm_instrumenter: &'w mut WasmInstrumenter,
        cost_unit_counter: &'c mut CostUnitCounter,
        fee_table: &'c FeeTable,
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
            locked_resmans: HashSet::new(),
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
        from: Option<SNodeRef>,
        validated: &ScryptoValue,
    ) -> Result<(), RuntimeError> {
        if !validated.kv_store_ids.is_empty() {
            return Err(RuntimeError::KeyValueStoreNotAllowed);
        }

        // Allow vaults to be returned from ResourceStatic
        // TODO: Should we allow vaults to be returned by any component?
        if !matches!(from, Some(SNodeRef::ResourceRef(_))) {
            if !validated.vault_ids.is_empty() {
                return Err(RuntimeError::VaultNotAllowed);
            }
        }

        Ok(())
    }

    pub fn run(
        &mut self,
        snode_ref: Option<SNodeRef>, // TODO: Remove, abstractions between invoke_snode() and run() are a bit messy right now
        execution: SNodeExecution<'p>,
        fn_ident: &str,
        input: ScryptoValue,
    ) -> Result<(ScryptoValue, HashMap<ValueId, REValue>), RuntimeError> {
        trace!(
            self,
            Level::Debug,
            "Run started! Depth: {}, Remaining cost units: {}",
            self.depth,
            self.cost_unit_counter.remaining()
        );
        self.cost_unit_counter
            .consume(self.fee_table.engine_run_cost())
            .map_err(RuntimeError::CostingError)?;

        let output = {
            let rtn = match execution {
                SNodeExecution::Static(state) => match state {
                    StaticSNodeState::System => System::static_main(fn_ident, input, self)
                        .map_err(RuntimeError::SystemError),
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
        let (taken_values, mut missing) = self.take_available_values(values_to_take)?;
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

        // TODO: Remove
        for unlock in &self.locked_resmans {
            self.track.release_lock(unlock.clone());
        }

        trace!(
            self,
            Level::Debug,
            "Run finished! Remaining cost units: {}",
            self.cost_unit_counter().remaining()
        );

        Ok((output, taken_values))
    }

    fn take_persistent_child_values(
        &mut self,
        value_ids: HashSet<ValueId>,
    ) -> Result<(HashMap<StoredValueId, REValue>, HashSet<ValueId>), RuntimeError> {
        let (taken, missing) = {
            let mut taken_values = HashMap::new();
            let mut missing_values = HashSet::new();

            for id in value_ids {
                let maybe = self.owned_values.remove(&id);
                if let Some(celled_value) = maybe {
                    let value = celled_value.into_inner();
                    if !value.is_persistable_child() {
                        return Err(RuntimeError::ValueNotAllowed);
                    }
                    let stored_id: StoredValueId = id.into();
                    taken_values.insert(stored_id, value);
                } else {
                    missing_values.insert(id);
                }
            }

            (taken_values, missing_values)
        };

        // Moved values must have their references removed
        for (id, value) in taken.iter() {
            self.value_refs.remove(&ValueId::Stored(id.clone()));
            if let Some(children_store) = value.get_children_store() {
                for id in children_store.all_descendants() {
                    self.value_refs.remove(&ValueId::Stored(id));
                }
            }
        }

        Ok((taken, missing))
    }

    fn take_available_values(
        &mut self,
        value_ids: HashSet<ValueId>,
    ) -> Result<(HashMap<ValueId, REValue>, HashSet<ValueId>), RuntimeError> {
        let (taken, missing) = {
            let mut taken_values = HashMap::new();
            let mut missing_values = HashSet::new();

            for id in value_ids {
                let maybe = self.owned_values.remove(&id);
                if let Some(celled_value) = maybe {
                    let value = celled_value.into_inner();
                    match &value {
                        REValue::Bucket(bucket) => {
                            if bucket.is_locked() {
                                return Err(RuntimeError::CantMoveLockedBucket);
                            }
                        }
                        REValue::Proof(proof) => {
                            if proof.is_restricted() {
                                return Err(RuntimeError::CantMoveRestrictedProof(id));
                            }
                        }
                        _ => {}
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
                    self.value_refs.remove(&ValueId::Stored(id.clone()));
                }
            }
        }

        Ok((taken, missing))
    }
}

impl<'p, 't, 's, 'w, 'c, S, W, I> SystemApi<'p, 's, W, I, S>
    for CallFrame<'p, 't, 's, 'w, 'c, S, W, I>
where
    S: ReadableSubstateStore,
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
        let (taken_values, mut missing) = self.take_available_values(values_to_take)?;
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
            SNodeRef::SystemStatic => {
                Ok((SNodeExecution::Static(StaticSNodeState::System), vec![]))
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
                            .take_lock(resource_address, true)
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
                        vec![method_auth.clone()]
                    }
                    REValue::Proof(_) => vec![],
                    REValue::Component { component, .. } => {
                        let package_address = component.package_address();
                        self.track
                            .take_lock(package_address, false)
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
            SNodeRef::AuthZoneRef => {
                if let Some(auth_zone) = &self.auth_zone {
                    for resource_address in &input.resource_addresses {
                        self.track
                            .take_lock(resource_address.clone(), false)
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
                    .take_lock(address.clone(), true)
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
                        .take_lock(resource_address.clone(), false)
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
                        .take_lock(package_address.clone(), false)
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

                            self.track
                                .take_lock(address.clone(), true)
                                .map_err(|e| match e {
                                    TrackError::NotFound => {
                                        RuntimeError::ComponentNotFound(component_address)
                                    }
                                    TrackError::Reentrancy => {
                                        RuntimeError::ComponentReentrancy(component_address)
                                    }
                                })?;
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
                        let mut value_ref = next_frame_location.to_ref_mut(
                            &value_id,
                            &mut next_owned_values,
                            &mut next_borrowed_values,
                            &mut self.track,
                        );
                        let (package_address, blueprint_name) = value_ref.component_info();
                        ScryptoActorInfo::component(
                            package_address,
                            blueprint_name,
                            component_address,
                        )
                    };

                    // Retrieve Method Authorization
                    let (method_auths, package_address) = {
                        let package_address = actor_info.package_address().clone();
                        let blueprint_name = actor_info.blueprint_name().to_string();
                        self.track
                            .take_lock(package_address, false)
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
                            .take_lock(package_address, false)
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
                                .take_lock(vault_address, true)
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
                    let mut value_ref = next_location.to_ref_mut(
                        &value_id,
                        &mut next_owned_values,
                        &mut next_borrowed_values,
                        &mut self.track,
                    );
                    let resource_address = value_ref.vault_resource_address();
                    self.track
                        .take_lock(resource_address, true)
                        .expect("Should never fail.");
                    locked_values.insert(resource_address.into());

                    next_location
                };

                // Retrieve Method Authorization
                let method_auth = {
                    let mut value_ref = next_location.to_ref_mut(
                        &value_id,
                        &mut next_owned_values,
                        &mut next_borrowed_values,
                        &mut self.track,
                    );
                    let resource_address = value_ref.vault_resource_address();
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
        let (result, received_values) =
            frame.run(Some(snode_ref), loaded_snode, &fn_ident, input)?;
        drop(frame);

        // Release locked addresses
        for l in locked_values {
            self.track.release_lock(l);
        }

        // move buckets and proofs to this process.
        for (id, value) in received_values {
            trace!(self, Level::Debug, "Received value: {:?}", value);
            self.owned_values.insert(id, RefCell::new(value));
        }

        Ok(result)
    }

    fn get_non_fungible(
        &mut self,
        non_fungible_address: &NonFungibleAddress,
    ) -> Option<NonFungible> {
        let parent_address = Address::NonFungibleSet(non_fungible_address.resource_address());
        let key = non_fungible_address.non_fungible_id().to_vec();
        if let SubstateValue::NonFungible(non_fungible) =
            self.track.read_key_value(parent_address, key)
        {
            non_fungible
        } else {
            panic!("Value is not a non fungible");
        }
    }

    fn set_non_fungible(
        &mut self,
        non_fungible_address: NonFungibleAddress,
        non_fungible: Option<NonFungible>,
    ) {
        let parent_address = Address::NonFungibleSet(non_fungible_address.resource_address());
        let key = non_fungible_address.non_fungible_id().to_vec();
        self.track.set_key_value(parent_address, key, non_fungible)
    }

    fn borrow_value(&self, value_id: &ValueId) -> REValueRef<'_, 'p, 's, S> {
        let info = self
            .value_refs
            .get(value_id)
            .expect(&format!("{:?} is unknown.", value_id));
        if !info.visible {
            panic!("Trying to read value which is not visible.")
        }

        info.location.to_ref(
            value_id,
            &self.owned_values,
            &self.frame_borrowed_values,
            &self.track,
        )
    }

    fn borrow_value_mut(&mut self, value_id: &ValueId) -> RENativeValueRef<'p> {
        let info = self.value_refs.get(value_id).unwrap();
        if !info.visible {
            panic!("Trying to read value which is not visible.")
        }

        info.location.borrow_native_ref(
            value_id,
            &mut self.owned_values,
            &mut self.frame_borrowed_values,
            &mut self.track,
        )
    }

    fn return_value_mut(&mut self, value_id: ValueId, val_ref: RENativeValueRef<'p>) {
        val_ref.return_to_location(
            value_id,
            &mut self.owned_values,
            &mut self.frame_borrowed_values,
            &mut self.track,
        )
    }

    fn drop_value(&mut self, value_id: &ValueId) -> REValue {
        self.owned_values.remove(&value_id).unwrap().into_inner()
    }

    fn create_value<V: Into<REValueByComplexity>>(
        &mut self,
        v: V,
    ) -> Result<ValueId, RuntimeError> {
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
            REValueByComplexity::Complex(REComplexValue::Component(..)) => {
                let component_address = self.track.new_component_address();
                ValueId::Stored(StoredValueId::Component(component_address))
            }
        };

        let re_value = match value_by_complexity {
            REValueByComplexity::Primitive(primitive) => primitive.into(),
            REValueByComplexity::Complex(complex) => {
                let children = complex.get_children()?;
                let (child_values, mut missing) = self.take_persistent_child_values(children)?;
                let first_missing_value = missing.drain().nth(0);
                if let Some(missing_value) = first_missing_value {
                    return Err(RuntimeError::ValueNotFound(missing_value));
                }
                complex.into_re_value(child_values)
            }
        };
        self.owned_values.insert(id, RefCell::new(re_value));

        match id {
            ValueId::Stored(StoredValueId::KeyValueStoreId(..)) => {
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

    fn create_resource(&mut self, resource_manager: ResourceManager) -> ResourceAddress {
        let resource_address = self.track.create_uuid_value(resource_manager).into();

        // TODO: Remove
        self.value_refs.insert(
            ValueId::Resource(resource_address),
            REValueInfo {
                location: REValueLocation::Track { parent: None },
                visible: true,
            },
        );
        self.track
            .take_lock(resource_address, true)
            .expect("Should never fail since it was just created.");
        self.locked_resmans.insert(resource_address.into());

        resource_address
    }

    fn globalize_value(&mut self, value_id: &ValueId) {
        let mut values = HashSet::new();
        values.insert(value_id.clone());
        let (taken_values, missing) = self.take_available_values(values).unwrap();
        assert!(missing.is_empty());
        assert!(taken_values.len() == 1);
        let value = taken_values.into_values().nth(0).unwrap();

        let (substate, maybe_child_values) = match value {
            REValue::Component {
                component,
                child_values,
            } => (SubstateValue::Component(component), Some(child_values)),
            REValue::Package(package) => (SubstateValue::Package(package), None),
            _ => panic!("Not expected"),
        };

        let address = match value_id {
            ValueId::Stored(StoredValueId::Component(component_address)) => {
                Address::GlobalComponent(*component_address)
            }
            ValueId::Package(package_address) => Address::Package(*package_address),
            _ => panic!("Expected to be a component address"),
        };

        self.track.create_uuid_value_2(address.clone(), substate);

        if let Some(child_values) = maybe_child_values {
            let mut to_store_values = HashMap::new();
            for (id, cell) in child_values.into_iter() {
                to_store_values.insert(id, cell.into_inner());
            }
            self.track
                .insert_objects_into_component(to_store_values, address.into());
        }
    }

    fn data(
        &mut self,
        address: SubstateAddress,
        instruction: DataInstruction,
    ) -> Result<ScryptoValue, RuntimeError> {
        // If write, take values from current frame
        let (taken_values, missing) = match &instruction {
            DataInstruction::Write(value) => {
                let value_ids = value.value_ids();
                self.take_persistent_child_values(value_ids)?
            }
            DataInstruction::Read => (HashMap::new(), HashSet::new()),
        };

        let value_id = match address {
            SubstateAddress::Component(component_address, ..) => {
                ValueId::Stored(StoredValueId::Component(component_address))
            }
            SubstateAddress::KeyValueEntry(kv_store_id, ..) => ValueId::kv_store_id(kv_store_id),
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
                    } else if self.track.take_lock(component_address, false).is_ok() {
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
                    ComponentOffset::State => value_ref.component_get_state(),
                    ComponentOffset::Info => ScryptoValue::from_typed(&value_ref.component_info()),
                },
                SubstateAddress::KeyValueEntry(.., key) => {
                    verify_stored_key(key)?;
                    value_ref.kv_store_get(&key.raw)
                }
            };
            let cur_children = to_stored_ids(current_value.value_ids())?;
            (current_value, cur_children)
        };

        // TODO: Remove, currently a hack to allow for global component info retrieval
        if let Some(component_address) = address_borrowed {
            self.track.release_lock(component_address);
        }

        // Fulfill method
        match instruction {
            DataInstruction::Read => {
                let parent_location = location.clone();
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
            DataInstruction::Write(value) => {
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
                }

                Ok(ScryptoValue::from_typed(&()))
            }
        }
    }

    fn get_epoch(&mut self) -> u64 {
        self.track.current_epoch()
    }

    fn get_transaction_hash(&mut self) -> Hash {
        self.track.transaction_hash()
    }

    fn generate_uuid(&mut self) -> u128 {
        self.track.new_uuid()
    }

    fn user_log(&mut self, level: Level, message: String) {
        self.track.add_log(level, message);
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
}
