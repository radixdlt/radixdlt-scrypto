use sbor::rust::boxed::Box;
use sbor::rust::cell::{RefCell, RefMut};
use sbor::rust::collections::*;
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
    'borrowed,
    'p, // Parent frame lifetime
    's, // Substate store lifetime
    't, // Track lifetime
    'w, // WASM engine lifetime
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

    /// Owned Values
    owned_values: HashMap<ValueId, RefCell<REValue>>,
    worktop: Option<RefCell<Worktop>>,
    auth_zone: Option<RefCell<AuthZone>>,

    /// Referenced values
    refed_values: HashMap<StoredValueId, REValueLocation>,
    // TODO: Merge with refed_values
    /// Readable values
    readable_values: HashMap<ValueId, REValueLocation>,

    borrowed_values: HashMap<ValueId, REOwnedValueRef<'borrowed>>,

    /// Caller's auth zone
    caller_auth_zone: Option<&'p RefCell<AuthZone>>,

    /// There is a single cost unit counter and a single fee table per transaction execution.
    /// When a call ocurrs, they're passed from the parent to the child, and returned
    /// after the invocation.
    cost_unit_counter: Option<CostUnitCounter>,
    fee_table: Option<FeeTable>,

    phantom: PhantomData<I>,
}

#[derive(Debug)]
pub enum TransientValue {
    Bucket(Bucket),
    Proof(Proof),
}

#[derive(Debug)]
pub enum REValue {
    Stored(StoredValue),
    Transient(TransientValue),
}

impl REValue {
    fn to_stored(&mut self) -> &mut StoredValue {
        match self {
            REValue::Stored(stored_value) => stored_value,
            _ => panic!("Expected a stored value"),
        }
    }

    fn try_drop(self) -> Result<(), DropFailure> {
        match self {
            REValue::Stored(StoredValue::Vault(..)) => Err(DropFailure::Vault),
            REValue::Stored(StoredValue::KeyValueStore { .. }) => Err(DropFailure::KeyValueStore),
            REValue::Stored(StoredValue::Component { .. }) => Err(DropFailure::Component),
            REValue::Transient(TransientValue::Bucket(..)) => Err(DropFailure::Bucket),
            REValue::Transient(TransientValue::Proof(proof)) => {
                proof.drop();
                Ok(())
            }
        }
    }
}

impl Into<Bucket> for REValue {
    fn into(self) -> Bucket {
        match self {
            REValue::Transient(TransientValue::Bucket(bucket)) => bucket,
            _ => panic!("Expected to be a bucket"),
        }
    }
}

impl Into<Proof> for REValue {
    fn into(self) -> Proof {
        match self {
            REValue::Transient(TransientValue::Proof(proof)) => proof,
            _ => panic!("Expected to be a proof"),
        }
    }
}

impl Into<StoredValue> for REValue {
    fn into(self) -> StoredValue {
        match self {
            REValue::Stored(stored_value) => stored_value,
            _ => panic!("Expected a stored value"),
        }
    }
}

impl Into<TransientValue> for REValue {
    fn into(self) -> TransientValue {
        match self {
            REValue::Transient(transient_value) => transient_value,
            _ => panic!("Expected a stored value"),
        }
    }
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

fn to_stored_values(
    values: HashMap<ValueId, REValue>,
) -> Result<HashMap<StoredValueId, StoredValue>, RuntimeError> {
    let mut stored_values = HashMap::new();
    for (id, value) in values {
        match id {
            ValueId::Stored(stored_id) => stored_values.insert(stored_id, value.into()),
            _ => return Err(RuntimeError::MovingInvalidType),
        };
    }
    Ok(stored_values)
}

fn verify_stored_value(value: &ScryptoValue) -> Result<(), RuntimeError> {
    if !value.bucket_ids.is_empty() {
        return Err(RuntimeError::BucketNotAllowed);
    }
    if !value.proof_ids.is_empty() {
        return Err(RuntimeError::ProofNotAllowed);
    }
    Ok(())
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
        borrowed_values: &mut HashMap<ValueId, REOwnedValueRef<'borrowed>>,
        track: &mut Track<S>,
    ) -> RENativeValueRef<'borrowed> {
        match self {
            REValueLocation::BorrowedRoot => {
                let owned = borrowed_values.remove(value_id).expect("Should exist");
                RENativeValueRef::Owned(owned)
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

                let value = track
                    .borrow_global_mut_value(address.clone())
                    .map(|v| v.into())
                    .unwrap();

                RENativeValueRef::Track(address, value)
            }
            _ => panic!("Unexpected"),
        }
    }

    fn to_owned_ref<'a, 'borrowed>(
        &self,
        value_id: &ValueId,
        owned_values: &'a mut HashMap<ValueId, RefCell<REValue>>,
        borrowed_values: &'a mut HashMap<ValueId, REOwnedValueRef<'borrowed>>,
    ) -> REOwnedValueRef<'a> {
        match self {
            REValueLocation::OwnedRoot => {
                let cell = owned_values.get_mut(value_id).unwrap();
                let ref_mut = cell.borrow_mut();
                REOwnedValueRef::Root(ref_mut)
            }
            REValueLocation::Owned {
                root,
                ref ancestors,
            } => {
                let root_value = owned_values.get_mut(&root).unwrap().get_mut();
                let root_store = match root_value {
                    REValue::Stored(root_store) => root_store,
                    _ => panic!("Invalid type"),
                };

                let stored_value_id = match value_id {
                    ValueId::Stored(stored_value_id) => stored_value_id,
                    _ => panic!("Unexpected value id"),
                };
                let value = root_store.get_child(ancestors, stored_value_id);
                REOwnedValueRef::Child(value)
            }
            REValueLocation::Borrowed { root, ancestors } => {
                let borrowed = borrowed_values.get_mut(root).unwrap();
                let stored_value_id = match value_id {
                    ValueId::Stored(stored_value_id) => stored_value_id,
                    _ => panic!("Unexpected value id"),
                };
                let value = borrowed
                    .mut_stored_value()
                    .get_child(ancestors, stored_value_id);
                REOwnedValueRef::Child(value)
            }
            _ => panic!("Not an owned ref"),
        }
    }

    fn to_ref<'a, 'borrowed: 'a>(
        &self,
        value_id: &ValueId,
        owned_values: &'a mut HashMap<ValueId, RefCell<REValue>>,
        borrowed_values: &'a mut HashMap<ValueId, REOwnedValueRef<'borrowed>>,
    ) -> REValueRef<'a, 'borrowed> {
        match self {
            REValueLocation::OwnedRoot
            | REValueLocation::Owned { .. }
            | REValueLocation::Borrowed { .. } => {
                REValueRef::Owned(self.to_owned_ref(value_id, owned_values, borrowed_values))
            }
            REValueLocation::BorrowedRoot => {
                REValueRef::Borrowed(borrowed_values.get_mut(value_id).unwrap())
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

                REValueRef::Track(address)
            }
        }
    }
}

pub enum RENativeValueRef<'a> {
    Owned(REOwnedValueRef<'a>),
    Track(Address, SubstateValue),
}

impl<'a> RENativeValueRef<'a> {
    pub fn bucket(&mut self) -> &mut Bucket {
        match self {
            RENativeValueRef::Owned(REOwnedValueRef::Root(ref mut root)) => {
                match root.deref_mut() {
                    REValue::Transient(TransientValue::Bucket(bucket)) => bucket,
                    _ => panic!("Expecting to be a bucket"),
                }
            }
            _ => panic!("Expecting to be a bucket"),
        }
    }

    pub fn proof(&mut self) -> &mut Proof {
        match self {
            RENativeValueRef::Owned(REOwnedValueRef::Root(ref mut root)) => {
                match root.deref_mut() {
                    REValue::Transient(TransientValue::Proof(proof)) => proof,
                    _ => panic!("Expecting to be a proof"),
                }
            }
            _ => panic!("Expecting to be a proof"),
        }
    }

    pub fn vault(&mut self) -> &mut Vault {
        match self {
            RENativeValueRef::Owned(owned) => match owned {
                REOwnedValueRef::Root(root) => match root.deref_mut() {
                    REValue::Stored(StoredValue::Vault(vault)) => vault,
                    _ => panic!("Expecting to be a vault"),
                },
                REOwnedValueRef::Child(stored_value) => match stored_value.deref_mut() {
                    StoredValue::Vault(vault) => vault,
                    _ => panic!("Expecting to be a vault"),
                },
            },
            RENativeValueRef::Track(_address, value) => value.vault_mut(),
        }
    }

    pub fn component(&mut self) -> &mut Component {
        match self {
            RENativeValueRef::Owned(owned) => match owned {
                REOwnedValueRef::Root(root) => match root.deref_mut() {
                    REValue::Stored(StoredValue::Component { component, .. }) => component,
                    _ => panic!("Expecting to be a component"),
                },
                _ => panic!("Expecting to be a component"),
            },
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
            _ => panic!("Expecting to be tracked"),
        }
    }

    pub fn return_to_location<S: ReadableSubstateStore>(
        self,
        value_id: ValueId,
        borrowed_values: &mut HashMap<ValueId, REOwnedValueRef<'a>>,
        track: &mut Track<S>,
    ) {
        match self {
            RENativeValueRef::Owned(owned) => {
                borrowed_values.insert(value_id.clone(), owned);
            }
            RENativeValueRef::Track(address, value) => {
                track.return_borrowed_global_mut_value(address, value)
            }
        }
    }
}

pub enum REOwnedValueRef<'a> {
    Root(RefMut<'a, REValue>),
    Child(RefMut<'a, StoredValue>),
}

impl<'a> REOwnedValueRef<'a> {
    fn component(&self) -> &Component {
        match self {
            REOwnedValueRef::Root(root) => match root.deref() {
                REValue::Stored(StoredValue::Component { component, .. }) => component,
                _ => panic!("Expected a component"),
            },
            REOwnedValueRef::Child(stored_value) => stored_value.component(),
        }
    }

    fn mut_stored_value(&mut self) -> &mut StoredValue {
        match self {
            REOwnedValueRef::Root(root) => match root.deref_mut() {
                REValue::Stored(stored_value) => stored_value,
                _ => panic!("Expected a stored value"),
            },
            REOwnedValueRef::Child(stored_value) => stored_value,
        }
    }
}

pub enum REValueRef<'a, 'b> {
    Owned(REOwnedValueRef<'a>),
    Borrowed(&'a mut REOwnedValueRef<'b>),
    Track(Address),
}

impl<'a, 'b> REValueRef<'a, 'b> {
    fn kv_store_put<S: ReadableSubstateStore>(
        &mut self,
        key: Vec<u8>,
        value: ScryptoValue,
        to_store: HashMap<StoredValueId, StoredValue>,
        track: &mut Track<S>,
    ) {
        match self {
            REValueRef::Owned(owned) => {
                let store = match owned {
                    REOwnedValueRef::Root(root) => match root.deref_mut() {
                        REValue::Stored(stored_value) => stored_value,
                        _ => panic!("Expecting to be stored value"),
                    },
                    REOwnedValueRef::Child(stored_value) => stored_value,
                };
                store.insert_children(to_store);
                match store {
                    StoredValue::KeyValueStore { store, .. } => {
                        store.put(key, value);
                    }
                    _ => panic!("Expecting to be kv store"),
                }
            }
            REValueRef::Borrowed(..) => {
                panic!("Not supported");
            }
            REValueRef::Track(address) => {
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

    fn kv_store_get<S: ReadableSubstateStore>(
        &self,
        key: &[u8],
        track: &mut Track<S>,
    ) -> ScryptoValue {
        let maybe_value = match self {
            REValueRef::Owned(owned) => {
                let store = match owned {
                    REOwnedValueRef::Root(root) => match root.deref() {
                        REValue::Stored(stored_value) => stored_value.kv_store(),
                        _ => panic!("Expecting to be stored value"),
                    },
                    REOwnedValueRef::Child(stored_value) => stored_value.kv_store(),
                };
                store.get(key).map(|v| v.dom)
            }
            REValueRef::Borrowed(..) => {
                panic!("Not supported");
            }
            REValueRef::Track(address) => {
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

    fn component_get_state<S: ReadableSubstateStore>(&self, track: &mut Track<S>) -> ScryptoValue {
        match self {
            REValueRef::Track(address) => {
                let component_val = track.read_value(address.clone());
                let component = component_val.component();
                return ScryptoValue::from_slice(component.state()).expect("Expected to decode");
            }
            REValueRef::Borrowed(owned) => {
                let component = &owned.component();
                return ScryptoValue::from_slice(component.state()).expect("Expected to decode");
            }
            _ => panic!("Unexpected component ref"),
        }
    }

    fn component_put<S: ReadableSubstateStore>(
        &mut self,
        value: ScryptoValue,
        to_store: HashMap<StoredValueId, StoredValue>,
        track: &mut Track<S>,
    ) {
        match self {
            REValueRef::Track(address) => {
                track
                    .write_component_value(address.clone(), value.raw)
                    .unwrap();

                let parent_address = match address {
                    Address::GlobalComponent(address) => *address,
                    Address::LocalComponent(parent, ..) => *parent,
                    _ => panic!("Expected component address"),
                };

                track.insert_objects_into_component(to_store, parent_address);
            }
            REValueRef::Borrowed(owned) => {
                let stored_value = owned.mut_stored_value();
                let component = match stored_value {
                    StoredValue::Component { component, .. } => component,
                    _ => panic!("Unexpected"),
                };
                component.set_state(value.raw);
                stored_value.insert_children(to_store);
            }
            _ => panic!("Unexpected component ref"),
        }
    }

    fn component_info<S: ReadableSubstateStore>(
        &mut self,
        track: &mut Track<S>,
    ) -> (PackageAddress, String) {
        match self {
            REValueRef::Owned(owned) => {
                let component = &owned.component();
                (
                    component.package_address().clone(),
                    component.blueprint_name().to_string(),
                )
            }
            REValueRef::Track(address) => {
                let component_val = track.borrow_global_value(address.clone()).unwrap();
                let component = component_val.component();
                (
                    component.package_address().clone(),
                    component.blueprint_name().to_string(),
                )
            }
            _ => panic!("Unexpected component ref"),
        }
    }

    fn component_authorization<S: ReadableSubstateStore>(
        &mut self,
        schema: &Type,
        fn_ident: &str,
        track: &mut Track<S>,
    ) -> Vec<MethodAuthorization> {
        match self {
            REValueRef::Owned(owned) => {
                let stored_value = owned.mut_stored_value();
                let component = match stored_value {
                    StoredValue::Component { component, .. } => component,
                    _ => panic!("Unexpected"),
                };
                component.method_authorization(schema, fn_ident)
            }
            REValueRef::Track(address) => {
                let component_val = track.borrow_global_value(address.clone()).unwrap();
                let component = component_val.component();
                component.method_authorization(schema, fn_ident)
            }
            _ => panic!("Unexpected component ref"),
        }
    }

    fn vault_address<S: ReadableSubstateStore>(&mut self, track: &mut Track<S>) -> ResourceAddress {
        match self {
            REValueRef::Owned(REOwnedValueRef::Root(re_value)) => match re_value.to_stored() {
                StoredValue::Vault(vault) => vault.resource_address(),
                _ => panic!("Unexpected value"),
            },
            REValueRef::Owned(REOwnedValueRef::Child(stored_value)) => {
                stored_value.vault().resource_address()
            }
            REValueRef::Borrowed(..) => {
                panic!("Not supported");
            }
            REValueRef::Track(address) => {
                let vault_val = track.borrow_global_value(address.clone()).unwrap();
                let vault = vault_val.vault();
                vault.resource_address()
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
    Worktop(RefMut<'a, Worktop>),
    ValueRef(ValueId),
    Scrypto(ScryptoActorInfo, ValidatedPackage),
}

pub enum DataInstruction {
    Read,
    Write(ScryptoValue),
}

pub enum SubstateAddress {
    KeyValueEntry(KeyValueStoreId, ScryptoValue),
    Component(ComponentAddress, ComponentOffset),
}

impl<'borrowed, 'p, 's, 't, 'w, S, W, I> CallFrame<'borrowed, 'p, 's, 't, 'w, S, W, I>
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
        cost_unit_counter: CostUnitCounter,
        fee_table: FeeTable,
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
            Some(RefCell::new(AuthZone::new_with_proofs(
                initial_auth_zone_proofs,
            ))),
            Some(RefCell::new(Worktop::new())),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            None,
            cost_unit_counter,
            fee_table,
        )
    }

    pub fn new(
        transaction_hash: Hash,
        depth: usize,
        trace: bool,
        track: &'t mut Track<'s, S>,
        wasm_engine: &'w mut W,
        wasm_instrumenter: &'w mut WasmInstrumenter,
        auth_zone: Option<RefCell<AuthZone>>,
        worktop: Option<RefCell<Worktop>>,
        owned_values: HashMap<ValueId, REValue>,
        readable_values: HashMap<ValueId, REValueLocation>,
        borrowed_values: HashMap<ValueId, REOwnedValueRef<'borrowed>>,
        caller_auth_zone: Option<&'p RefCell<AuthZone>>,
        cost_unit_counter: CostUnitCounter,
        fee_table: FeeTable,
    ) -> Self {
        let mut celled_owned_values = HashMap::new();
        for (id, value) in owned_values {
            celled_owned_values.insert(id, RefCell::new(value));
        }

        Self {
            transaction_hash,
            depth,
            trace,
            track,
            wasm_engine,
            wasm_instrumenter,
            owned_values: celled_owned_values,
            refed_values: HashMap::new(),
            readable_values,
            borrowed_values,
            worktop,
            auth_zone,
            caller_auth_zone,
            cost_unit_counter: Some(cost_unit_counter),
            fee_table: Some(fee_table),
            phantom: PhantomData,
        }
    }

    fn drop_owned_values(&mut self) -> Result<(), RuntimeError> {
        for (_, value) in self.owned_values.drain() {
            trace!(self, Level::Warn, "Dangling value: {:?}", value);
            value
                .into_inner()
                .try_drop()
                .map_err(|e| RuntimeError::DropFailure(e))?;
        }

        if let Some(ref_worktop) = &self.worktop {
            let worktop = ref_worktop.borrow();
            if !worktop.is_empty() {
                trace!(self, Level::Warn, "Resource worktop is not empty");
                return Err(RuntimeError::DropFailure(DropFailure::Worktop));
            }
        }

        Ok(())
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
            "Run started! Remainging cost units: {}",
            self.cost_unit_counter().remaining()
        );

        Self::cost_unit_counter_helper(&mut self.cost_unit_counter)
            .consume(Self::fee_table_helper(&mut self.fee_table).engine_run_cost())
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
                SNodeExecution::Worktop(mut worktop) => worktop
                    .main(fn_ident, input, self)
                    .map_err(RuntimeError::WorktopError),
                SNodeExecution::ValueRef(value_id) => match value_id {
                    ValueId::Transient(TransientValueId::Bucket(bucket_id)) => {
                        Bucket::main(bucket_id, fn_ident, input, self)
                            .map_err(RuntimeError::BucketError)
                    }
                    ValueId::Transient(TransientValueId::Proof(..)) => {
                        Proof::main(value_id, fn_ident, input, self)
                            .map_err(RuntimeError::ProofError)
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
                SNodeExecution::Scrypto(ref actor, ref package) => {
                    package.invoke(&actor, fn_ident, input, self)
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

        trace!(
            self,
            Level::Debug,
            "Run finished! Remainging cost units: {}",
            self.cost_unit_counter().remaining()
        );

        Ok((output, taken_values))
    }

    fn cost_unit_counter_helper(counter: &mut Option<CostUnitCounter>) -> &mut CostUnitCounter {
        counter
            .as_mut()
            .expect("Frame doens't own a cost unit counter")
    }

    pub fn cost_unit_counter(&mut self) -> &mut CostUnitCounter {
        // Use helper method to support paritial borrow of self
        // See https://users.rust-lang.org/t/how-to-partially-borrow-from-struct/32221
        Self::cost_unit_counter_helper(&mut self.cost_unit_counter)
    }

    fn fee_table_helper(fee_table: &Option<FeeTable>) -> &FeeTable {
        fee_table.as_ref().expect("Frame doens't own a fee table")
    }

    pub fn fee_table(&self) -> &FeeTable {
        // Use helper method to support paritial borrow of self
        // See https://users.rust-lang.org/t/how-to-partially-borrow-from-struct/32221
        Self::fee_table_helper(&self.fee_table)
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
                        REValue::Transient(TransientValue::Bucket(bucket)) => {
                            if bucket.is_locked() {
                                return Err(RuntimeError::CantMoveLockedBucket);
                            }
                        }
                        REValue::Transient(TransientValue::Proof(proof)) => {
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
            self.readable_values.remove(id);

            match value {
                REValue::Stored(stored_value) => {
                    for id in stored_value.all_descendants() {
                        self.refed_values.remove(&id);
                        self.readable_values.remove(&ValueId::Stored(id.clone()));
                    }
                }
                _ => {}
            }
        }

        Ok((taken, missing))
    }
}

impl<'borrowed, 'p, 's, 't, 'w, S, W, I> SystemApi<'borrowed, W, I>
    for CallFrame<'borrowed, 'p, 's, 't, 'w, S, W, I>
where
    S: ReadableSubstateStore,
    W: WasmEngine<I>,
    I: WasmInstance,
{
    fn wasm_engine(&mut self) -> &mut W {
        self.wasm_engine
    }

    fn wasm_instrumenter(&mut self) -> &mut WasmInstrumenter {
        self.wasm_instrumenter
    }

    fn invoke_snode(
        &mut self,
        snode_ref: SNodeRef,
        fn_ident: String,
        input: ScryptoValue,
    ) -> Result<ScryptoValue, RuntimeError> {
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
        let (mut next_owned_values, mut missing) = self.take_available_values(values_to_take)?;
        let first_missing_value = missing.drain().nth(0);
        if let Some(missing_value) = first_missing_value {
            return Err(RuntimeError::ValueNotFound(missing_value));
        }

        // Internal state update to taken values
        for (_, value) in &mut next_owned_values {
            trace!(self, Level::Debug, "Sending value: {:?}", value);
            match value {
                REValue::Transient(TransientValue::Proof(proof)) => proof.change_to_restricted(),
                _ => {}
            }
        }

        let mut locked_values = HashSet::new();
        let mut readable_values = HashMap::new();
        let mut borrowed_values = HashMap::new();

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
                    REValue::Transient(TransientValue::Bucket(bucket)) => {
                        let resource_address = bucket.resource_address();
                        let substate_value = self
                            .track
                            .borrow_global_value(resource_address.clone())
                            .expect("There should be no problem retrieving resource manager");
                        let resource_manager = match substate_value {
                            SubstateValue::Resource(resource_manager) => resource_manager,
                            _ => panic!("Value is not a resource manager"),
                        };
                        let method_auth = resource_manager.get_consuming_bucket_auth(&fn_ident);
                        readable_values.insert(
                            ValueId::Resource(resource_address),
                            REValueLocation::Track { parent: None },
                        );
                        vec![method_auth.clone()]
                    }
                    REValue::Transient(TransientValue::Proof(_)) => vec![],
                    REValue::Stored(StoredValue::Component {..}) => vec![],
                    _ => return Err(RuntimeError::MethodDoesNotExist(fn_ident.clone())),
                };

                next_owned_values.insert(*value_id, value);

                Ok((SNodeExecution::Consumed(*value_id), method_auths))
            }
            SNodeRef::AuthZoneRef => {
                if let Some(auth_zone) = &self.auth_zone {
                    let borrowed = auth_zone.borrow_mut();
                    Ok((SNodeExecution::AuthZone(borrowed), vec![]))
                } else {
                    Err(RuntimeError::AuthZoneDoesNotExist)
                }
            }
            SNodeRef::WorktopRef => {
                if let Some(worktop_ref) = &self.worktop {
                    let worktop = worktop_ref.borrow_mut();
                    Ok((SNodeExecution::Worktop(worktop), vec![]))
                } else {
                    Err(RuntimeError::WorktopDoesNotExist)
                }
            }
            SNodeRef::ResourceRef(resource_address) => {
                let value_id = ValueId::Resource(*resource_address);
                let address: Address = Address::Resource(*resource_address);
                let substate =
                    self.track
                        .borrow_global_value(address.clone())
                        .map_err(|e| match e {
                            TrackError::NotFound => {
                                RuntimeError::ResourceManagerNotFound(resource_address.clone())
                            }
                            TrackError::Reentrancy => {
                                panic!("Resource call has caused reentrancy")
                            }
                        })?;
                let resource_manager = substate.resource_manager();
                let method_auth = resource_manager.get_auth(&fn_ident, &input).clone();
                readable_values.insert(value_id.clone(), REValueLocation::Track { parent: None });

                Ok((SNodeExecution::ValueRef(value_id), vec![method_auth]))
            }
            SNodeRef::BucketRef(bucket_id) => {
                let value_id = ValueId::Transient(TransientValueId::Bucket(*bucket_id));
                let bucket_cell = self
                    .owned_values
                    .get(&value_id)
                    .ok_or(RuntimeError::BucketNotFound(bucket_id.clone()))?;
                let ref_mut = bucket_cell.borrow_mut();
                let value_ref = REOwnedValueRef::Root(ref_mut);
                borrowed_values.insert(value_id.clone(), value_ref);
                readable_values.insert(value_id.clone(), REValueLocation::BorrowedRoot);

                Ok((SNodeExecution::ValueRef(value_id), vec![]))
            }
            SNodeRef::ProofRef(proof_id) => {
                let value_id = ValueId::Transient(TransientValueId::Proof(*proof_id));
                let proof_cell = self
                    .owned_values
                    .get(&value_id)
                    .ok_or(RuntimeError::ProofNotFound(proof_id.clone()))?;
                let ref_mut = proof_cell.borrow_mut();
                let value_ref = REOwnedValueRef::Root(ref_mut);
                borrowed_values.insert(value_id.clone(), value_ref);
                readable_values.insert(value_id.clone(), REValueLocation::BorrowedRoot);
                Ok((SNodeExecution::ValueRef(value_id), vec![]))
            }
            SNodeRef::Scrypto(actor) => match actor {
                ScryptoActor::Blueprint(package_address, blueprint_name) => {
                    let substate_value = self
                        .track
                        .borrow_global_value(package_address.clone())
                        .map_err(|e| match e {
                            TrackError::NotFound => RuntimeError::PackageNotFound(*package_address),
                            TrackError::Reentrancy => {
                                panic!("Package reentrancy error should never occur.")
                            }
                        })?;
                    let package = match substate_value {
                        SubstateValue::Package(package) => package,
                        _ => panic!("Value is not a package"),
                    };
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
                            package.clone(),
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
                    } else if let Some(location) = self.refed_values.get(&stored_value_id) {
                        location
                    } else {
                        let address: Address = component_address.into();
                        self.track
                            .borrow_global_value(address.clone())
                            .map_err(|e| match e {
                                TrackError::NotFound => {
                                    RuntimeError::ComponentNotFound(component_address)
                                }
                                TrackError::Reentrancy => {
                                    RuntimeError::ComponentReentrancy(component_address)
                                }
                            })?;
                        &REValueLocation::Track { parent: None }
                    };

                    let actor_info = {
                        let mut value_ref = cur_location.to_ref(
                            &value_id,
                            &mut self.owned_values,
                            &mut self.borrowed_values,
                        );
                        let (package_address, blueprint_name) =
                            value_ref.component_info(&mut self.track);
                        ScryptoActorInfo::component(
                            package_address,
                            blueprint_name,
                            component_address,
                        )
                    };

                    // Retrieve Method Authorization
                    let (method_auths, package) = {
                        let package_address = actor_info.package_address().clone();
                        let blueprint_name = actor_info.blueprint_name().to_string();
                        let package_value = self
                            .track
                            .borrow_global_value(package_address)
                            .map_err(|e| match e {
                                TrackError::NotFound => {
                                    RuntimeError::PackageNotFound(package_address)
                                }
                                TrackError::Reentrancy => {
                                    panic!("Package reentrancy error should never occur.")
                                }
                            })?;
                        let package = package_value.package().clone();
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
                            let mut value_ref = cur_location.to_ref(
                                &value_id,
                                &mut self.owned_values,
                                &mut self.borrowed_values,
                            );
                            value_ref.component_authorization(
                                &abi.structure,
                                &fn_ident,
                                &mut self.track,
                            )
                        };

                        (method_auths, package)
                    };

                    // Setup next frame
                    match cur_location {
                        REValueLocation::Track { parent } => {
                            let address = if let Some(parent) = parent {
                                Address::LocalComponent(*parent, component_address)
                            } else {
                                Address::GlobalComponent(component_address)
                            };

                            self.track.take_lock(address.clone()).map_err(|e| match e {
                                TrackError::NotFound => panic!("Should exist"),
                                TrackError::Reentrancy => {
                                    RuntimeError::ComponentReentrancy(component_address)
                                }
                            })?;
                            locked_values.insert(address.clone());
                            readable_values.insert(
                                value_id,
                                REValueLocation::Track {
                                    parent: parent.clone(),
                                },
                            );
                        }
                        REValueLocation::OwnedRoot | REValueLocation::Borrowed { .. } => {
                            let owned_ref = cur_location.to_owned_ref(
                                &value_id,
                                &mut self.owned_values,
                                &mut self.borrowed_values,
                            );
                            borrowed_values.insert(value_id, owned_ref);
                            readable_values.insert(value_id, REValueLocation::BorrowedRoot);
                        }
                        _ => panic!("Unexpected"),
                    }

                    Ok((SNodeExecution::Scrypto(actor_info, package), method_auths))
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
                        let owned_ref = cur_location.to_owned_ref(
                            &value_id,
                            &mut self.owned_values,
                            &mut self.borrowed_values,
                        );
                        let package_address = owned_ref.component().package_address();

                        borrowed_values.insert(value_id, owned_ref);
                        readable_values.insert(value_id, REValueLocation::BorrowedRoot);
                        readable_values.insert(
                            ValueId::Package(package_address),
                            REValueLocation::Track { parent: None },
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
                    let stored_value_id = StoredValueId::VaultId(*vault_id);
                    let maybe_value_ref = self.refed_values.get(&stored_value_id);
                    maybe_value_ref
                        .ok_or(RuntimeError::ValueNotFound(ValueId::vault_id(*vault_id)))?
                };

                // Retrieve Method Authorization
                let method_auth = {
                    let mut value_ref = cur_location.to_ref(
                        &value_id,
                        &mut self.owned_values,
                        &mut self.borrowed_values,
                    );
                    let resource_address = value_ref.vault_address(&mut self.track);
                    let substate_value = self
                        .track
                        .borrow_global_value(resource_address.clone())
                        .unwrap();
                    let resource_manager = match substate_value {
                        SubstateValue::Resource(resource_manager) => resource_manager,
                        _ => panic!("Value is not a resource manager"),
                    };
                    resource_manager.get_vault_auth(&fn_ident).clone()
                };

                // Setup next frame
                match cur_location {
                    REValueLocation::Track { parent } => {
                        readable_values.insert(
                            value_id.clone(),
                            REValueLocation::Track {
                                parent: parent.clone(),
                            },
                        );
                    }
                    REValueLocation::OwnedRoot
                    | REValueLocation::Owned { .. }
                    | REValueLocation::Borrowed { .. } => {
                        let owned_ref = cur_location.to_owned_ref(
                            &value_id,
                            &mut self.owned_values,
                            &mut self.borrowed_values,
                        );
                        borrowed_values.insert(value_id.clone(), owned_ref);
                        readable_values.insert(value_id, REValueLocation::BorrowedRoot);
                    }
                    _ => panic!("Unexpected vault location {:?}", cur_location),
                }

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

        // Prepare moving cost unit counter and fee table
        let cost_unit_counter = self
            .cost_unit_counter
            .take()
            .expect("Frame doesn't own a cost unit counter");
        let fee_table = self
            .fee_table
            .take()
            .expect("Frame doesn't own a fee table");

        // start a new frame
        let mut frame = CallFrame::new(
            self.transaction_hash,
            self.depth + 1,
            self.trace,
            self.track,
            self.wasm_engine,
            self.wasm_instrumenter,
            match loaded_snode {
                SNodeExecution::Scrypto(..)
                | SNodeExecution::Static(StaticSNodeState::TransactionProcessor) => {
                    Some(RefCell::new(AuthZone::new()))
                }
                _ => None,
            },
            match loaded_snode {
                SNodeExecution::Static(StaticSNodeState::TransactionProcessor) => {
                    Some(RefCell::new(Worktop::new()))
                }
                _ => None,
            },
            next_owned_values,
            readable_values,
            borrowed_values,
            self.auth_zone.as_ref(),
            cost_unit_counter,
            fee_table,
        );

        // invoke the main function
        let run_result = frame.run(Some(snode_ref), loaded_snode, &fn_ident, input);

        // re-gain ownership of the cost unit counter and fee table
        self.cost_unit_counter = frame.cost_unit_counter.take();
        self.fee_table = frame.fee_table.take();
        drop(frame);

        // unwrap and continue
        let (result, received_values) = run_result?;

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

    fn borrow_global_resource_manager(
        &mut self,
        resource_address: ResourceAddress,
    ) -> Result<&ResourceManager, RuntimeError> {
        self.track
            .borrow_global_value(resource_address.clone())
            .map(SubstateValue::resource_manager)
            .map_err(|e| match e {
                TrackError::NotFound => RuntimeError::ResourceManagerNotFound(resource_address),
                TrackError::Reentrancy => panic!("Resman reentrancy should not occur."),
            })
    }

    fn borrow_native_value(&mut self, value_id: &ValueId) -> RENativeValueRef<'borrowed> {
        let location = self.readable_values.get(value_id).unwrap();
        location.borrow_native_ref(value_id, &mut self.borrowed_values, &mut self.track)
    }

    fn return_native_value(&mut self, value_id: ValueId, val_ref: RENativeValueRef<'borrowed>) {
        val_ref.return_to_location(value_id, &mut self.borrowed_values, &mut self.track)
    }

    fn take_native_value(&mut self, value_id: &ValueId) -> REValue {
        self.owned_values.remove(&value_id).unwrap().into_inner()
    }

    fn create_proof(&mut self, proof: Proof) -> Result<ProofId, RuntimeError> {
        let proof_id = self.track.new_proof_id();
        self.owned_values.insert(
            ValueId::Transient(TransientValueId::Proof(proof_id)),
            RefCell::new(REValue::Transient(TransientValue::Proof(proof))),
        );
        Ok(proof_id)
    }

    fn create_bucket(&mut self, container: ResourceContainer) -> Result<BucketId, RuntimeError> {
        let bucket_id = self.track.new_bucket_id();
        self.owned_values.insert(
            ValueId::Transient(TransientValueId::Bucket(bucket_id)),
            RefCell::new(REValue::Transient(TransientValue::Bucket(Bucket::new(
                container,
            )))),
        );
        Ok(bucket_id)
    }

    fn create_vault(&mut self, container: ResourceContainer) -> Result<VaultId, RuntimeError> {
        let vault_id = self.track.new_vault_id();
        self.owned_values.insert(
            ValueId::vault_id(vault_id.clone()),
            RefCell::new(REValue::Stored(StoredValue::Vault(Vault::new(container)))),
        );
        Ok(vault_id)
    }

    fn create_resource(&mut self, resource_manager: ResourceManager) -> ResourceAddress {
        let resource_address = self.track.create_uuid_value(resource_manager).into();

        self.readable_values.insert(
            ValueId::Resource(resource_address),
            REValueLocation::Track { parent: None },
        );

        resource_address
    }

    fn create_package(&mut self, package: ValidatedPackage) -> PackageAddress {
        self.track.create_uuid_value(package).into()
    }

    fn create_local_component(
        &mut self,
        component: Component,
    ) -> Result<ComponentAddress, RuntimeError> {
        let value =
            ScryptoValue::from_slice(component.state()).map_err(RuntimeError::DecodeError)?;
        verify_stored_value(&value)?;
        let value_ids = value.stored_value_ids();
        let (taken_values, mut missing) = self.take_available_values(value_ids)?;
        let first_missing_value = missing.drain().nth(0);
        if let Some(missing_value) = first_missing_value {
            return Err(RuntimeError::ValueNotFound(missing_value));
        }
        let to_store_values = to_stored_values(taken_values)?;

        let component_address = self.track.new_component_address();
        self.owned_values.insert(
            ValueId::Stored(StoredValueId::Component(component_address)),
            RefCell::new(REValue::Stored(StoredValue::Component {
                component,
                child_values: InMemoryChildren::with_values(to_store_values),
            })),
        );
        Ok(component_address)
    }

    fn native_globalize(&mut self, value_id: &ValueId) {
        let value = self.owned_values.remove(value_id).unwrap().into_inner();

        let (component, child_values) = match value {
            REValue::Stored(StoredValue::Component {
                component,
                child_values,
            }) => (component, child_values),
            _ => panic!("Expected to be a component"),
        };

        let component_address = match value_id {
            ValueId::Stored(StoredValueId::Component(component_address)) => *component_address,
            _ => panic!("Expected to be a component address"),
        };

        self.track.create_uuid_value_2(component_address, component);

        let mut to_store_values = HashMap::new();
        for (id, cell) in child_values.into_iter() {
            to_store_values.insert(id, cell.into_inner());
        }
        self.track
            .insert_objects_into_component(to_store_values, component_address);
    }

    fn create_kv_store(&mut self) -> KeyValueStoreId {
        let kv_store_id = self.track.new_kv_store_id();
        let value_id = ValueId::kv_store_id(kv_store_id.clone());
        self.owned_values.insert(
            value_id.clone(),
            RefCell::new(REValue::Stored(StoredValue::KeyValueStore {
                store: PreCommittedKeyValueStore::new(),
                child_values: InMemoryChildren::new(),
            })),
        );
        self.readable_values
            .insert(value_id, REValueLocation::OwnedRoot);
        kv_store_id
    }

    fn data(
        &mut self,
        address: SubstateAddress,
        instruction: DataInstruction,
    ) -> Result<ScryptoValue, RuntimeError> {
        // If write, take values from current frame
        let (taken_values, missing) = match &instruction {
            DataInstruction::Write(value) => {
                verify_stored_value(value)?;
                let value_ids = value.stored_value_ids();
                self.take_available_values(value_ids)?
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
        let location = self
            .readable_values
            .get(&value_id)
            .or_else(|| {
                // Allow global read access to any component info
                if let SubstateAddress::Component(component_address, ComponentOffset::Info) =
                    address
                {
                    if self.owned_values.contains_key(&value_id) {
                        return Some(&REValueLocation::OwnedRoot);
                    } else if self.track.borrow_global_value(component_address).is_ok() {
                        return Some(&REValueLocation::Track { parent: None });
                    }
                }

                None
            })
            .ok_or_else(|| RuntimeError::InvalidDataAccess(value_id))?;

        // Read current value
        let (current_value, cur_children) = {
            let mut value_ref =
                location.to_ref(&value_id, &mut self.owned_values, &mut self.borrowed_values);
            let current_value = match &address {
                SubstateAddress::Component(.., offset) => match offset {
                    ComponentOffset::State => value_ref.component_get_state(&mut self.track),
                    ComponentOffset::Info => {
                        ScryptoValue::from_typed(&value_ref.component_info(&mut self.track))
                    }
                },
                SubstateAddress::KeyValueEntry(.., key) => {
                    verify_stored_key(key)?;
                    value_ref.kv_store_get(&key.raw, &mut self.track)
                }
            };
            let cur_children = to_stored_ids(current_value.stored_value_ids())?;
            (current_value, cur_children)
        };

        // Fulfill method
        match instruction {
            DataInstruction::Read => {
                let parent_location = location.clone();
                for stored_value_id in cur_children {
                    let child_location = parent_location.child(value_id.clone());
                    self.refed_values
                        .insert(stored_value_id, child_location.clone());

                    // Extend current readable space when kv stores are found
                    if let StoredValueId::KeyValueStoreId(..) = stored_value_id {
                        self.readable_values
                            .insert(ValueId::Stored(stored_value_id.clone()), child_location);
                    }
                }
                Ok(current_value)
            }
            DataInstruction::Write(value) => {
                let missing = to_stored_ids(missing)?;
                verify_stored_value_update(&cur_children, &missing)?;

                let to_store_values = to_stored_values(taken_values)?;

                // TODO: verify against some schema

                // Write values
                let mut value_ref =
                    location.to_ref(&value_id, &mut self.owned_values, &mut self.borrowed_values);
                match address {
                    SubstateAddress::Component(.., offset) => match offset {
                        ComponentOffset::State => {
                            value_ref.component_put(value, to_store_values, &mut self.track)
                        }
                        ComponentOffset::Info => {
                            return Err(RuntimeError::InvalidDataWrite);
                        }
                    },
                    SubstateAddress::KeyValueEntry(.., key) => {
                        value_ref.kv_store_put(key.raw, value, to_store_values, self.track);
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
                        REValue::Transient(TransientValue::Proof(proof)) => proof.clone(),
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
        self.cost_unit_counter()
    }

    fn fee_table(&self) -> &FeeTable {
        self.fee_table()
    }
}
