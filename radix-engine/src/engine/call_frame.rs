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
use crate::state_manager::*;
use crate::wasm::*;

/// A call frame is the basic unit that forms a transaction call stack, which keeps track of the
/// owned objects by this function.
pub struct CallFrame<
    'p, // parent lifetime
    'g, // lifetime of values outliving all frames
    's, // Substate store lifetime
    S,  // Substore store type
    W,  // WASM engine type
    I,  // WASM instance type
    C,  // Cost unit counter type
> where
    S: ReadableSubstateStore,
    W: WasmEngine<I>,
    I: WasmInstance,
    C: CostUnitCounter,
{
    /// The transaction hash
    transaction_hash: Hash,
    /// The call depth
    depth: usize,
    /// Whether to show trace messages
    trace: bool,

    /// State track
    track: &'g mut Track<'s, S>,
    /// Wasm engine
    wasm_engine: &'g mut W,
    /// Wasm Instrumenter
    wasm_instrumenter: &'g mut WasmInstrumenter,

    /// Remaining cost unit counter
    cost_unit_counter: &'g mut C,
    /// Fee table
    fee_table: &'g FeeTable,

    id_allocator: &'g mut IdAllocator,

    /// All ref values accessible by this call frame. The value may be located in one of the following:
    /// 1. borrowed values
    /// 2. track
    value_refs: HashMap<ValueId, REValueInfo>,

    /// Owned Values
    owned_values: HashMap<ValueId, REValue>,
    auth_zone: Option<RefCell<AuthZone>>,

    /// Borrowed Values from call frames up the stack
    parent_values: Vec<&'p mut HashMap<ValueId, REValue>>,
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
    old: &HashSet<ValueId>,
    missing: &HashSet<ValueId>,
) -> Result<(), RuntimeError> {
    // TODO: optimize intersection search
    for old_id in old.iter() {
        if !missing.contains(&old_id) {
            return Err(RuntimeError::StoredValueRemoved(old_id.clone()));
        }
    }

    for missing_id in missing.iter() {
        if !old.contains(missing_id) {
            return Err(RuntimeError::ValueNotFound(*missing_id));
        }
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

pub fn insert_non_root_nodes<'s, S: ReadableSubstateStore>(
    track: &mut Track<'s, S>,
    values: HashMap<ValueId, RENode>,
) {
    for (id, node) in values {
        match node {
            RENode::Vault(vault) => {
                let addr = Address::Vault(id.into());
                track.create_uuid_value(addr, vault);
            }
            RENode::Component(component) => {
                let addr = Address::LocalComponent(id.into());
                track.create_uuid_value(addr, component);
            }
            RENode::KeyValueStore(store) => {
                let id = id.into();
                let address = Address::KeyValueStore(id);
                track.create_key_space(address.clone());
                for (k, v) in store.store {
                    track.set_key_value(address.clone(), k, Some(v));
                }
            }
            _ => panic!("Invalid node being persisted: {:?}", node),
        }
    }
}

#[derive(Debug, Clone)]
pub struct REValueInfo {
    visible: bool,
    location: REValuePointer,
}

#[derive(Debug, Clone)]
pub enum REValuePointer {
    Stack {
        frame_id: Option<usize>,
        root: ValueId,
        id: Option<ValueId>,
    },
    Track(Address),
}

impl REValuePointer {
    fn child(&self, child_id: ValueId) -> REValuePointer {
        match self {
            REValuePointer::Stack { frame_id, root, .. } => REValuePointer::Stack {
                frame_id: frame_id.clone(),
                root: root.clone(),
                id: Option::Some(child_id),
            },
            REValuePointer::Track(..) => {
                let child_address = match child_id {
                    ValueId::KeyValueStore(kv_store_id) => Address::KeyValueStore(kv_store_id),
                    ValueId::Vault(vault_id) => Address::Vault(vault_id),
                    ValueId::Component(component_id) => Address::LocalComponent(component_id),
                    _ => panic!("Unexpected"),
                };
                REValuePointer::Track(child_address)
            }
        }
    }

    fn borrow_native_ref<'p, S: ReadableSubstateStore>(
        &self, // TODO: Consider changing this to self
        owned_values: &mut HashMap<ValueId, REValue>,
        borrowed_values: &mut Vec<&'p mut HashMap<ValueId, REValue>>,
        track: &mut Track<S>,
    ) -> RENativeValueRef {
        match self {
            REValuePointer::Stack { frame_id, root, id } => {
                let frame = if let Some(frame_id) = frame_id {
                    borrowed_values.get_mut(*frame_id).unwrap()
                } else {
                    owned_values
                };
                let re_value = frame.remove(root).expect("Should exist");
                RENativeValueRef::Stack(re_value, frame_id.clone(), root.clone(), id.clone())
            }
            REValuePointer::Track(address) => {
                let value = track.take_value(address.clone());
                RENativeValueRef::Track(address.clone(), value)
            }
        }
    }

    fn to_ref<'f, 'p, 's, S: ReadableSubstateStore>(
        &self,
        owned_values: &'f HashMap<ValueId, REValue>,
        borrowed_values: &'f Vec<&'p mut HashMap<ValueId, REValue>>,
        track: &'f Track<'s, S>,
    ) -> REValueRef<'f, 's, S> {
        match self {
            REValuePointer::Stack { frame_id, root, id } => {
                let frame = if let Some(frame_id) = frame_id {
                    borrowed_values.get(*frame_id).unwrap()
                } else {
                    owned_values
                };
                REValueRef::Stack(frame.get(root).unwrap(), id.clone())
            }
            REValuePointer::Track(address) => REValueRef::Track(track, address.clone()),
        }
    }

    fn to_ref_mut<'f, 'p, 's, S: ReadableSubstateStore>(
        &self,
        owned_values: &'f mut HashMap<ValueId, REValue>,
        borrowed_values: &'f mut Vec<&'p mut HashMap<ValueId, REValue>>,
        track: &'f mut Track<'s, S>,
    ) -> REValueRefMut<'f, 's, S> {
        match self {
            REValuePointer::Stack { frame_id, root, id } => {
                let frame = if let Some(frame_id) = frame_id {
                    borrowed_values.get_mut(*frame_id).unwrap()
                } else {
                    owned_values
                };
                REValueRefMut::Stack(frame.get_mut(root).unwrap(), id.clone())
            }
            REValuePointer::Track(address) => REValueRefMut::Track(track, address.clone()),
        }
    }
}

pub enum RENativeValueRef {
    Stack(REValue, Option<usize>, ValueId, Option<ValueId>),
    Track(Address, Substate),
}

impl RENativeValueRef {
    pub fn bucket(&mut self) -> &mut Bucket {
        match self {
            RENativeValueRef::Stack(root, _frame_id, _root_id, maybe_child) => {
                match root.get_node_mut(maybe_child.as_ref()) {
                    RENode::Bucket(bucket) => bucket,
                    _ => panic!("Expecting to be a bucket"),
                }
            }
            _ => panic!("Expecting to be a bucket"),
        }
    }

    pub fn proof(&mut self) -> &mut Proof {
        match self {
            RENativeValueRef::Stack(ref mut root, _frame_id, _root_id, maybe_child) => {
                match root.get_node_mut(maybe_child.as_ref()) {
                    RENode::Proof(proof) => proof,
                    _ => panic!("Expecting to be a proof"),
                }
            }
            _ => panic!("Expecting to be a proof"),
        }
    }

    pub fn worktop(&mut self) -> &mut Worktop {
        match self {
            RENativeValueRef::Stack(ref mut root, _frame_id, _root_id, maybe_child) => {
                match root.get_node_mut(maybe_child.as_ref()) {
                    RENode::Worktop(worktop) => worktop,
                    _ => panic!("Expecting to be a worktop"),
                }
            }
            _ => panic!("Expecting to be a worktop"),
        }
    }

    pub fn vault(&mut self) -> &mut Vault {
        match self {
            RENativeValueRef::Stack(root, _frame_id, _root_id, maybe_child) => {
                root.get_node_mut(maybe_child.as_ref()).vault_mut()
            }
            RENativeValueRef::Track(_address, value) => value.vault_mut(),
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
            RENativeValueRef::Stack(root, _frame_id, _root_id, maybe_child) => {
                root.get_node_mut(maybe_child.as_ref()).component_mut()
            }
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
            RENativeValueRef::Stack(value, _frame_id, _root_id, maybe_child) => value
                .get_node_mut(maybe_child.as_ref())
                .resource_manager_mut(),
            RENativeValueRef::Track(_address, value) => value.resource_manager_mut(),
        }
    }

    pub fn return_to_location<'a, 'p, S: ReadableSubstateStore>(
        self,
        owned_values: &'a mut HashMap<ValueId, REValue>,
        borrowed_values: &'a mut Vec<&'p mut HashMap<ValueId, REValue>>,
        track: &mut Track<S>,
    ) {
        match self {
            RENativeValueRef::Stack(owned, frame_id, value_id, ..) => {
                let frame = if let Some(frame_id) = frame_id {
                    borrowed_values.get_mut(frame_id).unwrap()
                } else {
                    owned_values
                };
                frame.insert(value_id, owned);
            }
            RENativeValueRef::Track(address, value) => track.write_value(address, value),
        }
    }
}

pub enum REValueRef<'f, 's, S: ReadableSubstateStore> {
    Stack(&'f REValue, Option<ValueId>),
    Track(&'f Track<'s, S>, Address),
}

impl<'f, 'p, 's, S: ReadableSubstateStore> REValueRef<'f, 's, S> {
    pub fn vault(&self) -> &Vault {
        match self {
            REValueRef::Stack(value, id) => id
                .as_ref()
                .map_or(value.root(), |v| value.non_root(v))
                .vault(),
            REValueRef::Track(track, address) => track.read_value(address.clone()).vault(),
        }
    }

    pub fn system(&self) -> &System {
        match self {
            REValueRef::Stack(value, id) => id
                .as_ref()
                .map_or(value.root(), |v| value.non_root(v))
                .system(),
            REValueRef::Track(track, address) => track.read_value(address.clone()).system(),
        }
    }

    pub fn resource_manager(&self) -> &ResourceManager {
        match self {
            REValueRef::Stack(value, id) => id
                .as_ref()
                .map_or(value.root(), |v| value.non_root(v))
                .resource_manager(),
            REValueRef::Track(track, address) => {
                track.read_value(address.clone()).resource_manager()
            }
        }
    }

    pub fn component(&self) -> &Component {
        match self {
            REValueRef::Stack(value, id) => id
                .as_ref()
                .map_or(value.root(), |v| value.non_root(v))
                .component(),
            REValueRef::Track(track, address) => track.read_value(address.clone()).component(),
        }
    }

    pub fn package(&self) -> &ValidatedPackage {
        match self {
            REValueRef::Stack(value, id) => id
                .as_ref()
                .map_or(value.root(), |v| value.non_root(v))
                .package(),
            REValueRef::Track(track, address) => track.read_value(address.clone()).package(),
        }
    }
}

pub enum REValueRefMut<'f, 's, S: ReadableSubstateStore> {
    Stack(&'f mut REValue, Option<ValueId>),
    Track(&'f mut Track<'s, S>, Address),
}

impl<'f, 's, S: ReadableSubstateStore> REValueRefMut<'f, 's, S> {
    fn kv_store_put(
        &mut self,
        key: Vec<u8>,
        value: ScryptoValue,
        to_store: HashMap<ValueId, REValue>,
    ) {
        match self {
            REValueRefMut::Stack(re_value, id) => {
                re_value
                    .get_node_mut(id.as_ref())
                    .kv_store_mut()
                    .put(key, value);
                for (id, val) in to_store {
                    re_value.insert_non_root_nodes(val.to_nodes(id));
                }
            }
            REValueRefMut::Track(track, address) => {
                track.set_key_value(
                    address.clone(),
                    key,
                    Substate::KeyValueStoreEntry(Some(value.raw)),
                );
                for (id, val) in to_store {
                    insert_non_root_nodes(track, val.to_nodes(id));
                }
            }
        }
    }

    fn kv_store_get(&mut self, key: &[u8]) -> ScryptoValue {
        let maybe_value = match self {
            REValueRefMut::Stack(re_value, id) => {
                let store = re_value.get_node_mut(id.as_ref()).kv_store_mut();
                store.get(key).map(|v| v.dom)
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
            REValueRefMut::Stack(value, re_id) => {
                let non_fungible_set = re_id
                    .as_ref()
                    .map_or(value.root(), |v| value.non_root(v))
                    .non_fungibles();
                ScryptoValue::from_typed(&non_fungible_set.get(id).cloned())
            }
            REValueRefMut::Track(track, address) => {
                let value = track.read_key_value(address.clone(), id.to_vec());
                ScryptoValue::from_typed(value.non_fungible())
            }
        }
    }

    fn non_fungible_remove(&mut self, id: &NonFungibleId) {
        match self {
            REValueRefMut::Stack(..) => {
                panic!("Not supported");
            }
            REValueRefMut::Track(track, address) => {
                track.set_key_value(address.clone(), id.to_vec(), Substate::NonFungible(None));
            }
        }
    }

    fn non_fungible_put(&mut self, id: NonFungibleId, value: ScryptoValue) {
        match self {
            REValueRefMut::Stack(re_value, re_id) => {
                let non_fungible: NonFungible =
                    scrypto_decode(&value.raw).expect("Should not fail.");

                let non_fungible_set = re_value.get_node_mut(re_id.as_ref()).non_fungibles_mut();
                non_fungible_set.insert(id, non_fungible);
            }
            REValueRefMut::Track(track, address) => {
                let non_fungible: NonFungible =
                    scrypto_decode(&value.raw).expect("Should not fail.");
                track.set_key_value(
                    address.clone(),
                    id.to_vec(),
                    Substate::NonFungible(Some(non_fungible)),
                );
            }
        }
    }

    fn component_put(&mut self, value: ScryptoValue, to_store: HashMap<ValueId, REValue>) {
        match self {
            REValueRefMut::Stack(re_value, id) => {
                let component = re_value.get_node_mut(id.as_ref()).component_mut();
                component.set_state(value.raw);
                for (id, val) in to_store {
                    re_value.insert_non_root_nodes(val.to_nodes(id));
                }
            }
            REValueRefMut::Track(track, address) => {
                track.write_component_value(address.clone(), value.raw);
                for (id, val) in to_store {
                    insert_non_root_nodes(track, val.to_nodes(id));
                }
            }
        }
    }

    fn component(&mut self) -> &Component {
        match self {
            REValueRefMut::Stack(re_value, id) => re_value.get_node_mut(id.as_ref()).component(),
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

impl<'p, 'g, 's, S, W, I, C> CallFrame<'p, 'g, 's, S, W, I, C>
where
    S: ReadableSubstateStore,
    W: WasmEngine<I>,
    I: WasmInstance,
    C: CostUnitCounter,
{
    pub fn new_root(
        verbose: bool,
        transaction_hash: Hash,
        signer_public_keys: Vec<EcdsaPublicKey>,
        is_system: bool,
        id_allocator: &'g mut IdAllocator,
        track: &'g mut Track<'s, S>,
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
            verbose,
            id_allocator,
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
            Vec::new(),
            None,
        )
    }

    pub fn new(
        transaction_hash: Hash,
        depth: usize,
        trace: bool,
        id_allocator: &'g mut IdAllocator,
        track: &'g mut Track<'s, S>,
        wasm_engine: &'g mut W,
        wasm_instrumenter: &'g mut WasmInstrumenter,
        cost_unit_counter: &'g mut C,
        fee_table: &'g FeeTable,
        auth_zone: Option<RefCell<AuthZone>>,
        owned_values: HashMap<ValueId, REValue>,
        value_refs: HashMap<ValueId, REValueInfo>,
        parent_values: Vec<&'p mut HashMap<ValueId, REValue>>,
        caller_auth_zone: Option<&'p RefCell<AuthZone>>,
    ) -> Self {
        Self {
            transaction_hash,
            depth,
            trace,
            id_allocator,
            track,
            wasm_engine,
            wasm_instrumenter,
            cost_unit_counter,
            fee_table,
            owned_values,
            value_refs,
            parent_values,
            auth_zone,
            caller_auth_zone,
            phantom: PhantomData,
        }
    }

    fn drop_owned_values(&mut self) -> Result<(), RuntimeError> {
        let values = self
            .owned_values
            .drain()
            .map(|(_id, value)| value)
            .collect();
        RENode::drop_values(values).map_err(|e| RuntimeError::DropFailure(e))
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
                    ValueId::Bucket(..) => Bucket::consuming_main(value_id, fn_ident, input, self)
                        .map_err(RuntimeError::BucketError),
                    ValueId::Proof(..) => Proof::main_consume(value_id, fn_ident, input, self)
                        .map_err(RuntimeError::ProofError),
                    ValueId::Component(..) => {
                        Component::main_consume(value_id, fn_ident, input, self)
                            .map_err(RuntimeError::ComponentError)
                    }
                    _ => panic!("Unexpected"),
                },
                SNodeExecution::AuthZone(mut auth_zone) => auth_zone
                    .main(fn_ident, input, self)
                    .map_err(RuntimeError::AuthZoneError),
                SNodeExecution::ValueRef(value_id) => match value_id {
                    ValueId::Bucket(bucket_id) => Bucket::main(bucket_id, fn_ident, input, self)
                        .map_err(RuntimeError::BucketError),
                    ValueId::Proof(..) => Proof::main(value_id, fn_ident, input, self)
                        .map_err(RuntimeError::ProofError),
                    ValueId::Worktop => Worktop::main(value_id, fn_ident, input, self)
                        .map_err(RuntimeError::WorktopError),
                    ValueId::Vault(vault_id) => Vault::main(vault_id, fn_ident, input, self)
                        .map_err(RuntimeError::VaultError),
                    ValueId::Component(..) => Component::main(value_id, fn_ident, input, self)
                        .map_err(RuntimeError::ComponentError),
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
            self.value_refs.remove(id);
            for (id, ..) in &value.non_root_nodes {
                self.value_refs.remove(id);
            }
        }

        Ok((taken, missing))
    }

    fn read_value_internal(
        &mut self,
        address: &SubstateAddress,
    ) -> Result<(REValuePointer, ScryptoValue), RuntimeError> {
        let value_id = match address {
            SubstateAddress::Component(component_address, ..) => {
                ValueId::Component(*component_address)
            }
            SubstateAddress::NonFungible(resource_address, ..) => {
                ValueId::NonFungibles(*resource_address)
            }
            SubstateAddress::KeyValueEntry(kv_store_id, ..) => ValueId::KeyValueStore(*kv_store_id),
        };

        // Get location
        // Note this must be run AFTER values are taken, otherwise there would be inconsistent readable_values state
        let (value_info, address_borrowed) = self
            .value_refs
            .get(&value_id)
            .cloned()
            .map(|v| (v, None))
            .or_else(|| {
                // Allow global read access to any component info
                if let SubstateAddress::Component(component_address, ComponentOffset::Info) =
                    address
                {
                    if self.owned_values.contains_key(&value_id) {
                        return Some((
                            REValueInfo {
                                location: REValuePointer::Stack {
                                    frame_id: Option::None,
                                    root: value_id.clone(),
                                    id: Option::None,
                                },
                                visible: true,
                            },
                            None,
                        ));
                    } else if self
                        .track
                        .take_lock(Address::GlobalComponent(*component_address), false)
                        .is_ok()
                    {
                        return Some((
                            REValueInfo {
                                location: REValuePointer::Track(Address::GlobalComponent(
                                    *component_address,
                                )),
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
        let current_value = {
            let mut value_ref = location.to_ref_mut(
                &mut self.owned_values,
                &mut self.parent_values,
                &mut self.track,
            );
            match &address {
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
            }
        };

        // TODO: Remove, currently a hack to allow for global component info retrieval
        if let Some(component_address) = address_borrowed {
            self.track
                .release_lock(Address::GlobalComponent(*component_address));
        }

        Ok((location.clone(), current_value))
    }

    /// Creates a new package ID.
    pub fn new_package_address(&mut self) -> PackageAddress {
        // Security Alert: ensure ID allocating will practically never fail
        let package_address = self
            .id_allocator
            .new_package_address(self.transaction_hash)
            .unwrap();
        package_address
    }

    /// Creates a new component address.
    pub fn new_component_address(&mut self, component: &Component) -> ComponentAddress {
        let component_address = self
            .id_allocator
            .new_component_address(
                self.transaction_hash,
                &component.package_address(),
                component.blueprint_name(),
            )
            .unwrap();
        component_address
    }

    /// Creates a new resource address.
    pub fn new_resource_address(&mut self) -> ResourceAddress {
        let resource_address = self
            .id_allocator
            .new_resource_address(self.transaction_hash)
            .unwrap();
        resource_address
    }

    /// Creates a new UUID.
    pub fn new_uuid(&mut self) -> u128 {
        self.id_allocator.new_uuid(self.transaction_hash).unwrap()
    }

    /// Creates a new bucket ID.
    pub fn new_bucket_id(&mut self) -> BucketId {
        self.id_allocator.new_bucket_id().unwrap()
    }

    /// Creates a new vault ID.
    pub fn new_vault_id(&mut self) -> VaultId {
        self.id_allocator
            .new_vault_id(self.transaction_hash)
            .unwrap()
    }

    /// Creates a new reference id.
    pub fn new_proof_id(&mut self) -> ProofId {
        self.id_allocator.new_proof_id().unwrap()
    }

    /// Creates a new map id.
    pub fn new_kv_store_id(&mut self) -> KeyValueStoreId {
        self.id_allocator
            .new_kv_store_id(self.transaction_hash)
            .unwrap()
    }
}

impl<'p, 'g, 's, S, W, I, C> SystemApi<'p, 's, W, I, S, C> for CallFrame<'p, 'g, 's, S, W, I, C>
where
    S: ReadableSubstateStore,
    W: WasmEngine<I>,
    I: WasmInstance,
    C: CostUnitCounter,
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
            match &mut value.root_mut() {
                RENode::Proof(proof) => proof.change_to_restricted(),
                _ => {}
            }
            next_owned_values.insert(id, value);
        }

        let mut locked_values = HashSet::new();
        let mut value_refs = HashMap::new();

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
                    .ok_or(RuntimeError::ValueNotFound(*value_id))?;

                let method_auths = match &value.root() {
                    RENode::Bucket(bucket) => {
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
                                location: REValuePointer::Track(Address::Resource(
                                    resource_address,
                                )),
                                visible: true,
                            },
                        );
                        value_refs.insert(
                            ValueId::NonFungibles(resource_address),
                            REValueInfo {
                                location: REValuePointer::Track(Address::NonFungibleSet(
                                    resource_address,
                                )),
                                visible: true,
                            },
                        );
                        vec![method_auth.clone()]
                    }
                    RENode::Proof(_) => vec![],
                    RENode::Component(component) => {
                        let package_address = component.package_address();
                        self.track
                            .take_lock(package_address, false)
                            .expect("Should not fail.");
                        locked_values.insert(package_address.clone().into());
                        value_refs.insert(
                            ValueId::Package(package_address),
                            REValueInfo {
                                location: REValuePointer::Track(Address::Package(package_address)),
                                visible: true,
                            },
                        );
                        vec![]
                    }
                    _ => return Err(RuntimeError::MethodDoesNotExist(fn_ident.clone())),
                };

                next_owned_values.insert(*value_id, value);

                Ok((SNodeExecution::Consumed(*value_id), method_auths))
            }
            SNodeRef::SystemRef => {
                self.track
                    .take_lock(Address::System, true)
                    .expect("System access should never fail");
                locked_values.insert(Address::System);
                value_refs.insert(
                    ValueId::System,
                    REValueInfo {
                        location: REValuePointer::Track(Address::System),
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
                                location: REValuePointer::Track(Address::Resource(
                                    resource_address.clone(),
                                )),
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
                        location: REValuePointer::Track(Address::Resource(*resource_address)),
                        visible: true,
                    },
                );
                value_refs.insert(
                    ValueId::NonFungibles(*resource_address),
                    REValueInfo {
                        location: REValuePointer::Track(Address::NonFungibleSet(*resource_address)),
                        visible: true,
                    },
                );

                Ok((SNodeExecution::ValueRef(value_id), vec![method_auth]))
            }
            SNodeRef::BucketRef(bucket_id) => {
                let value_id = ValueId::Bucket(*bucket_id);
                if !self.owned_values.contains_key(&value_id) {
                    return Err(RuntimeError::BucketNotFound(bucket_id.clone()));
                }
                value_refs.insert(
                    value_id.clone(),
                    REValueInfo {
                        location: REValuePointer::Stack {
                            frame_id: Some(self.depth),
                            root: value_id.clone(),
                            id: None,
                        },
                        visible: true,
                    },
                );

                Ok((SNodeExecution::ValueRef(value_id), vec![]))
            }
            SNodeRef::ProofRef(proof_id) => {
                let value_id = ValueId::Proof(*proof_id);
                if !self.owned_values.contains_key(&value_id) {
                    return Err(RuntimeError::ProofNotFound(proof_id.clone()));
                }
                value_refs.insert(
                    value_id.clone(),
                    REValueInfo {
                        location: REValuePointer::Stack {
                            frame_id: Some(self.depth),
                            root: value_id.clone(),
                            id: None,
                        },
                        visible: true,
                    },
                );
                Ok((SNodeExecution::ValueRef(value_id), vec![]))
            }
            SNodeRef::WorktopRef => {
                let value_id = ValueId::Worktop;
                if !self.owned_values.contains_key(&value_id) {
                    return Err(RuntimeError::ValueNotFound(value_id));
                }
                value_refs.insert(
                    value_id.clone(),
                    REValueInfo {
                        location: REValuePointer::Stack {
                            frame_id: Some(self.depth),
                            root: value_id.clone(),
                            id: None,
                        },
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
                            location: REValuePointer::Track(Address::Resource(
                                resource_address.clone(),
                            )),
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
                    let value_id = ValueId::Component(component_address);
                    let cur_location = if self.owned_values.contains_key(&value_id) {
                        REValuePointer::Stack {
                            frame_id: None,
                            root: value_id.clone(),
                            id: None,
                        }
                    } else if let Some(REValueInfo { location, .. }) =
                        self.value_refs.get(&value_id)
                    {
                        location.clone()
                    } else {
                        REValuePointer::Track(Address::GlobalComponent(component_address))
                    };

                    // Lock values and setup next frame
                    let next_location = match cur_location.clone() {
                        REValuePointer::Track(address) => {
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
                            REValuePointer::Track(address)
                        }
                        REValuePointer::Stack { frame_id, root, id } => REValuePointer::Stack {
                            frame_id: frame_id.or(Some(self.depth)),
                            root,
                            id,
                        },
                    };

                    let actor_info = {
                        let value_ref = cur_location.to_ref(
                            &self.owned_values,
                            &self.parent_values,
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
                            let value_ref = cur_location.to_ref(
                                &self.owned_values,
                                &self.parent_values,
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
                            location: next_location,
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
                let value_id = ValueId::Component(component_address);
                let cur_location = if self.owned_values.contains_key(&value_id) {
                    REValuePointer::Stack {
                        frame_id: None,
                        root: value_id.clone(),
                        id: None,
                    }
                } else {
                    return Err(RuntimeError::NotSupported);
                };

                // Setup next frame
                match cur_location {
                    REValuePointer::Stack {
                        frame_id: _,
                        root,
                        id,
                    } => {
                        let owned_ref = self.owned_values.get_mut(&root).unwrap();

                        // Lock package
                        let package_address = owned_ref.root().component().package_address();
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
                                location: REValuePointer::Track(Address::Package(package_address)),
                                visible: true,
                            },
                        );

                        value_refs.insert(
                            value_id,
                            REValueInfo {
                                location: REValuePointer::Stack {
                                    frame_id: Some(self.depth),
                                    root: root.clone(),
                                    id: id.clone(),
                                },
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
                let value_id = ValueId::Vault(*vault_id);
                let cur_location = if self.owned_values.contains_key(&value_id) {
                    REValuePointer::Stack {
                        frame_id: None,
                        root: value_id.clone(),
                        id: Option::None,
                    }
                } else {
                    let maybe_value_ref = self.value_refs.get(&value_id);
                    maybe_value_ref
                        .map(|info| &info.location)
                        .cloned()
                        .ok_or(RuntimeError::ValueNotFound(ValueId::Vault(*vault_id)))?
                };

                // Lock values and setup next frame
                let next_location = {
                    // Lock Vault
                    let next_location = match cur_location.clone() {
                        REValuePointer::Track(address) => {
                            self.track
                                .take_lock(address.clone(), true)
                                .expect("Should never fail.");
                            locked_values.insert(address.clone().into());
                            REValuePointer::Track(address)
                        }
                        REValuePointer::Stack { frame_id, root, id } => REValuePointer::Stack {
                            frame_id: frame_id.or(Some(self.depth)),
                            root,
                            id,
                        },
                    };

                    // Lock Resource
                    let resource_address = {
                        let value_ref = cur_location.to_ref(
                            &mut self.owned_values,
                            &mut self.parent_values,
                            &mut self.track,
                        );
                        value_ref.vault().resource_address()
                    };
                    self.track
                        .take_lock(resource_address, true)
                        .expect("Should never fail.");
                    locked_values.insert(resource_address.into());

                    next_location
                };

                // Retrieve Method Authorization
                let method_auth = {
                    let resource_address = {
                        let value_ref = cur_location.to_ref(
                            &mut self.owned_values,
                            &mut self.parent_values,
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
                | SNodeExecution::ValueRef(ValueId::Vault(..), ..)
                | SNodeExecution::Consumed(ValueId::Bucket(..)) => {
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
        let mut next_borrowed_values: Vec<&mut HashMap<ValueId, REValue>> = Vec::new();
        for parent_values in &mut self.parent_values {
            next_borrowed_values.push(parent_values);
        }
        next_borrowed_values.push(&mut self.owned_values);

        // start a new frame
        let mut frame = CallFrame::new(
            self.transaction_hash,
            self.depth + 1,
            self.trace,
            self.id_allocator,
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
            self.track.release_lock(l);
        }

        // move buckets and proofs to this process.
        for (id, value) in received_values {
            trace!(self, Level::Debug, "Received value: {:?}", value);
            self.owned_values.insert(id, value);
        }

        Ok(result)
    }

    fn borrow_value(
        &mut self,
        value_id: &ValueId,
    ) -> Result<REValueRef<'_, 's, S>, CostUnitCounterError> {
        self.cost_unit_counter.consume(
            self.fee_table.system_api_cost({
                match value_id {
                    ValueId::Bucket(_) => SystemApiCostingEntry::BorrowLocal,
                    ValueId::Proof(_) => SystemApiCostingEntry::BorrowLocal,
                    ValueId::Worktop => SystemApiCostingEntry::BorrowLocal,
                    ValueId::Vault(_) => SystemApiCostingEntry::BorrowGlobal {
                        // TODO: figure out loaded state and size
                        loaded: false,
                        size: 0,
                    },
                    ValueId::Component(_) => SystemApiCostingEntry::BorrowGlobal {
                        // TODO: figure out loaded state and size
                        loaded: false,
                        size: 0,
                    },
                    ValueId::KeyValueStore(_) => SystemApiCostingEntry::BorrowGlobal {
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

        Ok(info
            .location
            .to_ref(&self.owned_values, &self.parent_values, &self.track))
    }

    fn borrow_value_mut(
        &mut self,
        value_id: &ValueId,
    ) -> Result<RENativeValueRef, CostUnitCounterError> {
        self.cost_unit_counter.consume(
            self.fee_table.system_api_cost({
                match value_id {
                    ValueId::Bucket(_) => SystemApiCostingEntry::BorrowLocal,
                    ValueId::Proof(_) => SystemApiCostingEntry::BorrowLocal,
                    ValueId::Worktop => SystemApiCostingEntry::BorrowLocal,
                    ValueId::Vault(_) => SystemApiCostingEntry::BorrowGlobal {
                        // TODO: figure out loaded state and size
                        loaded: false,
                        size: 0,
                    },
                    ValueId::Component(_) => SystemApiCostingEntry::BorrowGlobal {
                        // TODO: figure out loaded state and size
                        loaded: false,
                        size: 0,
                    },
                    ValueId::KeyValueStore(_) => SystemApiCostingEntry::BorrowGlobal {
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
            &mut self.owned_values,
            &mut self.parent_values,
            &mut self.track,
        ))
    }

    fn return_value_mut(&mut self, val_ref: RENativeValueRef) -> Result<(), CostUnitCounterError> {
        self.cost_unit_counter.consume(
            self.fee_table.system_api_cost({
                match &val_ref {
                    RENativeValueRef::Stack(..) => SystemApiCostingEntry::ReturnLocal,
                    RENativeValueRef::Track(address, _) => match address {
                        Address::Vault(_) => SystemApiCostingEntry::ReturnGlobal { size: 0 },
                        Address::KeyValueStore(_) => {
                            SystemApiCostingEntry::ReturnGlobal { size: 0 }
                        }
                        Address::Resource(_) => SystemApiCostingEntry::ReturnGlobal { size: 0 },
                        Address::Package(_) => SystemApiCostingEntry::ReturnGlobal { size: 0 },
                        Address::NonFungibleSet(_) => {
                            SystemApiCostingEntry::ReturnGlobal { size: 0 }
                        }
                        Address::GlobalComponent(_) => {
                            SystemApiCostingEntry::ReturnGlobal { size: 0 }
                        }
                        Address::LocalComponent(_) => {
                            SystemApiCostingEntry::ReturnGlobal { size: 0 }
                        }
                        Address::System => SystemApiCostingEntry::ReturnGlobal { size: 0 },
                    },
                }
            }),
            "return",
        )?;

        val_ref.return_to_location(
            &mut self.owned_values,
            &mut self.parent_values,
            &mut self.track,
        );
        Ok(())
    }

    fn drop_value(&mut self, value_id: &ValueId) -> Result<REValue, CostUnitCounterError> {
        // TODO: costing

        Ok(self.owned_values.remove(&value_id).unwrap())
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
                let bucket_id = self.new_bucket_id();
                ValueId::Bucket(bucket_id)
            }
            REValueByComplexity::Primitive(REPrimitiveValue::Proof(..)) => {
                let proof_id = self.new_proof_id();
                ValueId::Proof(proof_id)
            }
            REValueByComplexity::Primitive(REPrimitiveValue::Worktop(..)) => ValueId::Worktop,
            REValueByComplexity::Primitive(REPrimitiveValue::Vault(..)) => {
                let vault_id = self.new_vault_id();
                ValueId::Vault(vault_id)
            }
            REValueByComplexity::Primitive(REPrimitiveValue::KeyValue(..)) => {
                let kv_store_id = self.new_kv_store_id();
                ValueId::KeyValueStore(kv_store_id)
            }
            REValueByComplexity::Primitive(REPrimitiveValue::Package(..)) => {
                let package_address = self.new_package_address();
                ValueId::Package(package_address)
            }
            REValueByComplexity::Primitive(REPrimitiveValue::Resource(..)) => {
                let resource_address = self.new_resource_address();
                ValueId::Resource(resource_address)
            }
            REValueByComplexity::Primitive(REPrimitiveValue::NonFungibles(
                resource_address,
                ..,
            )) => ValueId::NonFungibles(resource_address),
            REValueByComplexity::Complex(REComplexValue::Component(ref component)) => {
                let component_address = self.new_component_address(component);
                ValueId::Component(component_address)
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
        self.owned_values.insert(id, re_value);

        match id {
            ValueId::KeyValueStore(..) | ValueId::Resource(..) | ValueId::NonFungibles(..) => {
                self.value_refs.insert(
                    id.clone(),
                    REValueInfo {
                        location: REValuePointer::Stack {
                            frame_id: None,
                            root: id.clone(),
                            id: None,
                        },
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

        let (substate, maybe_non_fungibles) = match value.root {
            RENode::Component(component) => (Substate::Component(component), None),
            RENode::Package(package) => (Substate::Package(package), None),
            RENode::Resource(resource_manager) => {
                let non_fungibles =
                    if matches!(resource_manager.resource_type(), ResourceType::NonFungible) {
                        let resource_address: ResourceAddress = value_id.clone().into();
                        let re_value = self
                            .owned_values
                            .remove(&ValueId::NonFungibles(resource_address))
                            .unwrap();
                        let non_fungibles: HashMap<NonFungibleId, NonFungible> = re_value.into();
                        Some(non_fungibles)
                    } else {
                        None
                    };
                (Substate::Resource(resource_manager), non_fungibles)
            }
            _ => panic!("Not expected"),
        };

        let address = match value_id {
            ValueId::Component(component_address) => Address::GlobalComponent(*component_address),
            ValueId::Package(package_address) => Address::Package(*package_address),
            ValueId::Resource(resource_address) => Address::Resource(*resource_address),
            _ => panic!("Expected to be a component address"),
        };

        self.track.create_uuid_value(address.clone(), substate);

        let mut to_store_values = HashMap::new();
        for (id, value) in value.non_root_nodes.into_iter() {
            to_store_values.insert(id, value);
        }
        insert_non_root_nodes(self.track, to_store_values);

        if let Some(non_fungibles) = maybe_non_fungibles {
            let resource_address: ResourceAddress = address.clone().into();
            self.track
                .create_key_space(Address::NonFungibleSet(resource_address));
            let parent_address = Address::NonFungibleSet(resource_address.clone());
            for (id, non_fungible) in non_fungibles {
                self.track.set_key_value(
                    parent_address.clone(),
                    id.to_vec(),
                    Substate::NonFungible(Some(non_fungible)),
                );
            }
        }

        Ok(())
    }

    fn remove_value_data(
        &mut self,
        address: SubstateAddress,
    ) -> Result<ScryptoValue, RuntimeError> {
        let (location, current_value) = self.read_value_internal(&address)?;
        let cur_children = current_value.value_ids();
        if !cur_children.is_empty() {
            return Err(RuntimeError::ValueNotAllowed);
        }

        // Write values
        let mut value_ref = location.to_ref_mut(
            &mut self.owned_values,
            &mut self.parent_values,
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

        let (parent_location, current_value) = self.read_value_internal(&address)?;
        let cur_children = current_value.value_ids();
        for child_id in cur_children {
            let child_location = parent_location.child(child_id);

            // Extend current readable space when kv stores are found
            let visible = matches!(child_id, ValueId::KeyValueStore(..));
            let child_info = REValueInfo {
                location: child_location,
                visible,
            };
            self.value_refs.insert(child_id, child_info);
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

        let (location, current_value) = self.read_value_internal(&address)?;
        let cur_children = current_value.value_ids();

        // Fulfill method
        verify_stored_value_update(&cur_children, &missing)?;

        // TODO: verify against some schema

        // Write values
        let mut value_ref = location.to_ref_mut(
            &mut self.owned_values,
            &mut self.parent_values,
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
                self.owned_values
                    .get(&ValueId::Proof(*proof_id))
                    .map(|p| match p.root() {
                        RENode::Proof(proof) => proof.clone(),
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
