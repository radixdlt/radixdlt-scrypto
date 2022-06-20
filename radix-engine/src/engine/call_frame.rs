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
use scrypto::resource::AuthZoneClearInput;
use scrypto::values::*;
use transaction::validation::*;

use crate::engine::SNodeState::{Borrowed, Consumed, Static, TrackedNative, TrackedScrypto};
use crate::engine::*;
use crate::fee::*;
use crate::ledger::*;
use crate::model::*;
use crate::wasm::*;

/// A call frame is the basic unit that forms a transaction call stack, which keeps track of the
/// owned objects by this function.
pub struct CallFrame<
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
    refed_values: HashMap<StoredValueId, ValueRefType>,
    refed_components: HashMap<ComponentAddress, Component>,

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueRefType {
    Uncommitted {
        root: KeyValueStoreId,
        ancestors: Vec<KeyValueStoreId>,
    },
    Committed {
        component_address: ComponentAddress,
    },
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

pub enum REValueRef<'a> {
    Owned(RefMut<'a, REValue>),
    Ref(RefMut<'a, StoredValue>),
}

impl<'a> REValueRef<'a> {
    fn to_stored_mut(&mut self) -> &mut StoredValue {
        match self {
            REValueRef::Owned(value) => match value.deref_mut() {
                REValue::Stored(stored_value) => stored_value,
                _ => panic!("Expecting to be stored value"),
            },
            REValueRef::Ref(stored_value) => stored_value,
        }
    }

    fn to_mut_vault(&mut self) -> &mut Vault {
        match self {
            REValueRef::Owned(value) => match value.deref_mut() {
                REValue::Stored(StoredValue::Vault(vault)) => vault,
                _ => panic!("Expecting to be a vault"),
            },
            REValueRef::Ref(stored_value) => match stored_value.deref_mut() {
                StoredValue::Vault(vault) => vault,
                _ => panic!("Expecting to be a vault"),
            },
        }
    }

    fn kv_store(&self) -> &PreCommittedKeyValueStore {
        match self {
            REValueRef::Owned(value) => match value.deref() {
                REValue::Stored(stored_value) => stored_value.kv_store(),
                _ => panic!("Expecting to be a vault"),
            },
            REValueRef::Ref(stored_value) => stored_value.kv_store(),
        }
    }

    fn kv_store_mut(&mut self) -> &mut PreCommittedKeyValueStore {
        match self {
            REValueRef::Owned(value) => match value.deref_mut() {
                REValue::Stored(stored_value) => stored_value.kv_store_mut(),
                _ => panic!("Expecting to be a vault"),
            },
            REValueRef::Ref(stored_value) => stored_value.kv_store_mut(),
        }
    }
}

pub enum BorrowedSNodeState<'a> {
    AuthZone(RefMut<'a, AuthZone>),
    Worktop(RefMut<'a, Worktop>),
    ValueRef(ValueId, RefMut<'a, REValue>),
    Vault(VaultId, REValueRef<'a>),
    Blueprint(ScryptoActorInfo, ValidatedPackage),
}

pub enum StaticSNodeState {
    Package,
    Resource,
    System,
    TransactionProcessor,
}

pub enum SNodeState<'a> {
    Static(StaticSNodeState),
    Consumed(TransientValue),
    Borrowed(BorrowedSNodeState<'a>),
    TrackedNative(
        Address,
        SubstateValue,
    ),
    TrackedScrypto(
        Address,
        SubstateValue,
        Option<(ScryptoActorInfo, ValidatedPackage)>,
    )
}

pub enum SNodeExecution<'a> {
    Static(StaticSNodeState),
    Consumed(TransientValue),
    AuthZone(RefMut<'a, AuthZone>),
    Worktop(RefMut<'a, Worktop>),
    ValueRef(ValueId, RefMut<'a, REValue>),
    Vault(VaultId, &'a mut Vault),
    Blueprint(ScryptoActorInfo, ValidatedPackage),
    Resource(Address, &'a mut ResourceManager),
    Component(ScryptoActorInfo, ValidatedPackage),
}

enum SubstateEntry<'a> {
    KeyValueStoreRef(REValueRef<'a>, ScryptoValue),
    KeyValueStoreTracked(ComponentAddress, KeyValueStoreId, ScryptoValue),
    Component(ComponentAddress, &'a mut Component),
    ComponentInfo(ComponentAddress, &'a Component),
    ComponentInfoTracked(ComponentAddress),
}

pub enum DataInstruction {
    Read,
    Write(ScryptoValue),
}

pub enum SubstateAddress {
    KeyValueEntry(KeyValueStoreId, ScryptoValue),
    Component(ComponentAddress),
    ComponentInfo(ComponentAddress),
}

impl<'a> SNodeExecution<'a> {
    fn execute<S: SystemApi<W, I>, W: WasmEngine<I>, I: WasmInstance>(
        self,
        fn_ident: &str,
        input: ScryptoValue,
        system: &mut S,
    ) -> Result<ScryptoValue, RuntimeError> {
        match self {
            SNodeExecution::Static(state) => match state {
                StaticSNodeState::System => {
                    System::static_main(fn_ident, input, system).map_err(RuntimeError::SystemError)
                }
                StaticSNodeState::TransactionProcessor => TransactionProcessor::static_main(
                    fn_ident, input, system,
                )
                .map_err(|e| match e {
                    TransactionProcessorError::InvalidRequestData(_) => panic!("Illegal state"),
                    TransactionProcessorError::InvalidMethod => panic!("Illegal state"),
                    TransactionProcessorError::RuntimeError(e) => e,
                }),
                StaticSNodeState::Package => ValidatedPackage::static_main(fn_ident, input, system)
                    .map_err(RuntimeError::PackageError),
                StaticSNodeState::Resource => ResourceManager::static_main(fn_ident, input, system)
                    .map_err(RuntimeError::ResourceManagerError),
            },
            SNodeExecution::Consumed(state) => match state {
                TransientValue::Bucket(bucket) => bucket
                    .consuming_main(fn_ident, input, system)
                    .map_err(RuntimeError::BucketError),
                TransientValue::Proof(proof) => proof
                    .main_consume(fn_ident, input)
                    .map_err(RuntimeError::ProofError),
            },
            SNodeExecution::AuthZone(mut auth_zone) => auth_zone
                .main(fn_ident, input, system)
                .map_err(RuntimeError::AuthZoneError),
            SNodeExecution::Worktop(mut worktop) => worktop
                .main(fn_ident, input, system)
                .map_err(RuntimeError::WorktopError),
            SNodeExecution::Blueprint(info, package) => {
                package.invoke(&info, fn_ident, input, system)
            }
            SNodeExecution::ValueRef(value_id, mut value) => match value.deref_mut() {
                REValue::Transient(TransientValue::Bucket(bucket)) => bucket
                    .main(value_id.into(), fn_ident, input, system)
                    .map_err(RuntimeError::BucketError),
                REValue::Transient(TransientValue::Proof(proof)) => proof
                    .main(fn_ident, input, system)
                    .map_err(RuntimeError::ProofError),
                _ => panic!("Unexpected value to execute"),
            },
            SNodeExecution::Vault(vault_id, vault) => vault
                .main(vault_id, fn_ident, input, system)
                .map_err(RuntimeError::VaultError),
            SNodeExecution::Resource(address, resource_manager) => resource_manager
                .main(address.clone().into(), fn_ident, input, system)
                .map_err(RuntimeError::ResourceManagerError),
            SNodeExecution::Component(ref actor, ref package) => {
                package.invoke(&actor, fn_ident, input, system)
            }
        }
    }
}

impl<'p, 's, 't, 'w, S, W, I> CallFrame<'p, 's, 't, 'w, S, W, I>
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
            refed_components: HashMap::new(),
            worktop,
            auth_zone,
            caller_auth_zone,
            cost_unit_counter: Some(cost_unit_counter),
            fee_table: Some(fee_table),
            phantom: PhantomData,
        }
    }

    /// Checks resource leak.
    fn check_resource(&mut self) -> Result<(), RuntimeError> {
        let mut success = true;
        let mut resource = ResourceFailure::Unknown;

        let values: HashMap<ValueId, REValue> = self
            .owned_values
            .drain()
            .map(|(id, c)| (id, c.into_inner()))
            .collect();
        for (_, value) in values {
            trace!(self, Level::Warn, "Dangling value: {:?}", value);
            let some_resource_failure = match value {
                REValue::Stored(StoredValue::Vault(vault)) => {
                    Some(ResourceFailure::Resource(vault.resource_address()))
                }
                REValue::Stored(StoredValue::KeyValueStore { .. }) => {
                    Some(ResourceFailure::UnclaimedKeyValueStore)
                }
                REValue::Transient(TransientValue::Bucket(bucket)) => {
                    Some(ResourceFailure::Resource(bucket.resource_address()))
                }
                REValue::Transient(TransientValue::Proof(proof)) => {
                    proof.drop();
                    None
                }
            };
            if let Some(resource_failure) = some_resource_failure {
                resource = resource_failure;
                success = false;
            }
        }

        if let Some(ref_worktop) = &self.worktop {
            let worktop = ref_worktop.borrow();
            if !worktop.is_empty() {
                trace!(self, Level::Warn, "Resource worktop is not empty");
                resource = ResourceFailure::Resources(worktop.resource_addresses());
                success = false;
            }
        }

        if success {
            Ok(())
        } else {
            Err(RuntimeError::ResourceCheckFailure(resource))
        }
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
        snode: SNodeState<'p>,
        fn_ident: &str,
        input: ScryptoValue,
    ) -> Result<(ScryptoValue, HashMap<ValueId, REValue>), RuntimeError> {
        let remaining_cost_units = self.cost_unit_counter().remaining();
        trace!(
            self,
            Level::Debug,
            "Run started! Remainging cost units: {}",
            remaining_cost_units
        );

        Self::cost_unit_counter_helper(&mut self.cost_unit_counter)
            .consume(Self::fee_table_helper(&mut self.fee_table).engine_run_cost())
            .map_err(RuntimeError::CostingError)?;

        let mut to_return = HashMap::new();

        // TODO: Find a better way to get rid of borrowed value does not live long enough issue
        #[allow(unused_assignments)]
        let mut ref_container = Option::None;

        let execution = match snode {
            SNodeState::Static(state) => SNodeExecution::Static(state),
            SNodeState::Consumed(consumed) => SNodeExecution::Consumed(consumed),
            SNodeState::Borrowed(borrowed) => match borrowed {
                BorrowedSNodeState::AuthZone(auth_zone) => SNodeExecution::AuthZone(auth_zone),
                BorrowedSNodeState::Worktop(worktop) => SNodeExecution::Worktop(worktop),
                BorrowedSNodeState::Blueprint(info, package) => {
                    SNodeExecution::Blueprint(info, package)
                }
                BorrowedSNodeState::ValueRef(value_id, value) => {
                    SNodeExecution::ValueRef(value_id, value)
                }
                BorrowedSNodeState::Vault(vault_id, value) => {
                    ref_container = Some(value);
                    let vault = ref_container.as_mut().unwrap().to_mut_vault();
                    SNodeExecution::Vault(vault_id, vault)
                }
            },
            SNodeState::TrackedNative(address, value) => match value {
                SubstateValue::Resource(_) => {
                    to_return.insert(address.clone(), value);
                    let resource_manager =
                        to_return.get_mut(&address).unwrap().resource_manager_mut();
                    SNodeExecution::Resource(address.clone(), resource_manager)
                }
                SubstateValue::Vault(_) => {
                    to_return.insert(address.clone(), value);
                    let vault = to_return.get_mut(&address).unwrap().vault_mut();
                    let vault_address: (ComponentAddress, VaultId) = address.clone().into();
                    SNodeExecution::Vault(vault_address.1, vault)
                }
                _ => panic!("Unexpected tracked value"),
            }
            SNodeState::TrackedScrypto(address, value, mut meta) => match value {
                SubstateValue::Component(component) => {
                    self.refed_components.insert(address.into(), component);
                    let (info, package) = meta.take().unwrap();
                    SNodeExecution::Component(info, package)
                }
                _ => panic!("Unexpected tracked value"),
            },
        };

        let output = execution.execute(fn_ident, input, self)?;

        // Update track
        for (address, value) in to_return.drain() {
            self.track.return_borrowed_global_mut_value(address, value);
        }
        for (address, value) in self.refed_components.drain() {
            self.track.return_borrowed_global_mut_value(address, value);
        }

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
        self.check_resource()?;

        let remaining_cost_units = self.cost_unit_counter().remaining();
        trace!(
            self,
            Level::Debug,
            "Run finished! Remainging cost units: {}",
            remaining_cost_units
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
        for (_, value) in &taken {
            match value {
                REValue::Stored(stored_value) => {
                    for id in stored_value.all_descendants() {
                        self.refed_values.remove(&id);
                    }
                }
                _ => {}
            }
        }

        Ok((taken, missing))
    }
}

impl<'p, 's, 't, 'w, S, W, I> SystemApi<W, I> for CallFrame<'p, 's, 't, 'w, S, W, I>
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
        let (mut taken_values, mut missing) = self.take_available_values(values_to_take)?;
        let first_missing_value = missing.drain().nth(0);
        if let Some(missing_value) = first_missing_value {
            return Err(RuntimeError::ValueNotFound(missing_value));
        }

        // Internal state update to taken values
        for (_, value) in &mut taken_values {
            trace!(self, Level::Debug, "Sending value: {:?}", value);
            match value {
                REValue::Transient(TransientValue::Proof(proof)) => proof.change_to_restricted(),
                _ => {}
            }
        }

        // Authorization and state load
        let (loaded_snode, method_auths) = match &snode_ref {
            SNodeRef::TransactionProcessor => {
                // FIXME: only TransactionExecutor can invoke this function
                Ok((Static(StaticSNodeState::TransactionProcessor), vec![]))
            }
            SNodeRef::PackageStatic => Ok((Static(StaticSNodeState::Package), vec![])),
            SNodeRef::SystemStatic => Ok((Static(StaticSNodeState::System), vec![])),
            SNodeRef::ResourceStatic => Ok((Static(StaticSNodeState::Resource), vec![])),
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
                        vec![method_auth.clone()]
                    },
                    REValue::Transient(TransientValue::Proof(_)) => vec![],
                    _ => return Err(RuntimeError::MethodDoesNotExist(fn_ident.clone())),
                };

                Ok((Consumed(value.into()), method_auths))
            }
            SNodeRef::AuthZoneRef => {
                if let Some(auth_zone) = &self.auth_zone {
                    let borrowed = auth_zone.borrow_mut();
                    Ok((Borrowed(BorrowedSNodeState::AuthZone(borrowed)), vec![]))
                } else {
                    Err(RuntimeError::AuthZoneDoesNotExist)
                }
            }
            SNodeRef::WorktopRef => {
                if let Some(worktop_ref) = &self.worktop {
                    let worktop = worktop_ref.borrow_mut();
                    Ok((Borrowed(BorrowedSNodeState::Worktop(worktop)), vec![]))
                } else {
                    Err(RuntimeError::WorktopDoesNotExist)
                }
            }
            SNodeRef::ResourceRef(resource_address) => {
                let resman_value = self
                    .track
                    .borrow_global_mut_value(resource_address.clone())
                    .map_err(|e| match e {
                        TrackError::NotFound => {
                            RuntimeError::ResourceManagerNotFound(resource_address.clone())
                        }
                        TrackError::Reentrancy => panic!("Reentrancy occurred in resource manager"),
                    })?;

                let method_auth = resman_value
                    .resource_manager()
                    .get_auth(&fn_ident, &input)
                    .clone();
                Ok((
                    TrackedNative(resource_address.clone().into(), resman_value),
                    vec![method_auth],
                ))
            }
            SNodeRef::BucketRef(bucket_id) => {
                let value_id = ValueId::Transient(TransientValueId::Bucket(*bucket_id));
                let bucket_cell = self
                    .owned_values
                    .get(&value_id)
                    .ok_or(RuntimeError::BucketNotFound(bucket_id.clone()))?;
                let bucket = bucket_cell.borrow_mut();
                Ok((
                    Borrowed(BorrowedSNodeState::ValueRef(value_id, bucket)),
                    vec![],
                ))
            }
            SNodeRef::ProofRef(proof_id) => {
                let value_id = ValueId::Transient(TransientValueId::Proof(*proof_id));
                let proof_cell = self
                    .owned_values
                    .get(&value_id)
                    .ok_or(RuntimeError::ProofNotFound(proof_id.clone()))?;
                let proof = proof_cell.borrow_mut();
                Ok((
                    Borrowed(BorrowedSNodeState::ValueRef(value_id, proof)),
                    vec![],
                ))
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
                        Borrowed(BorrowedSNodeState::Blueprint(
                            ScryptoActorInfo::blueprint(
                                package_address.clone(),
                                blueprint_name.clone(),
                            ),
                            package.clone(),
                        )),
                        vec![],
                    ))
                }
                ScryptoActor::Component(component_address) => {
                    let component_address = *component_address;

                    let component_value = self
                        .track
                        .borrow_global_mut_value(component_address)
                        .map_err(|e| match e {
                            TrackError::NotFound => {
                                RuntimeError::ComponentNotFound(component_address)
                            }
                            TrackError::Reentrancy => {
                                RuntimeError::ComponentReentrancy(component_address)
                            }
                        })?;
                    let component = component_value.component();
                    let package_address = component.package_address();
                    let blueprint_name = component.blueprint_name().to_string();

                    let package_value = self
                        .track
                        .borrow_global_value(package_address.clone())
                        .map_err(|e| match e {
                            TrackError::NotFound => RuntimeError::PackageNotFound(package_address),
                            TrackError::Reentrancy => {
                                panic!("Package reentrancy error should never occur.")
                            }
                        })?;
                    let package = package_value.package();
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
                    let (_, method_auths) =
                        component.method_authorization(&abi.structure, &fn_ident);

                    let actor_info = ScryptoActorInfo::component(
                        package_address,
                        blueprint_name,
                        component_address,
                    );

                    Ok((
                        TrackedScrypto(
                            component_address.into(),
                            component_value,
                            Some((actor_info, package.clone())),
                        ),
                        method_auths,
                    ))
                }
            },
            SNodeRef::VaultRef(vault_id) => {
                let (resource_address, snode_state) = {
                    if let Some(value) = self.owned_values.get(&ValueId::vault_id(*vault_id)) {
                        let resource_address = match value.borrow().deref() {
                            REValue::Stored(StoredValue::Vault(vault)) => vault.resource_address(),
                            _ => panic!("Expected vault"),
                        };

                        (
                            resource_address,
                            Borrowed(BorrowedSNodeState::Vault(
                                *vault_id,
                                REValueRef::Owned(value.borrow_mut()),
                            )),
                        )
                    } else {
                        let value_id = StoredValueId::VaultId(*vault_id);
                        let maybe_value_ref = self.refed_values.get(&value_id).cloned();
                        let value_ref = maybe_value_ref
                            .ok_or(RuntimeError::ValueNotFound(ValueId::vault_id(*vault_id)))?;
                        match value_ref {
                            ValueRefType::Uncommitted {
                                root,
                                ref ancestors,
                            } => {
                                let root_value = self
                                    .owned_values
                                    .get_mut(&ValueId::kv_store_id(root))
                                    .unwrap()
                                    .get_mut();
                                let root_store = match root_value {
                                    REValue::Stored(root_store) => root_store,
                                    _ => panic!("Invalid type"),
                                };
                                let value = root_store.get_child(ancestors, &value_id);
                                let resource_address = match value.deref() {
                                    StoredValue::Vault(vault) => vault.resource_address(),
                                    _ => panic!("Expected vault"),
                                };
                                (
                                    resource_address,
                                    Borrowed(BorrowedSNodeState::Vault(
                                        *vault_id,
                                        REValueRef::Ref(value),
                                    )),
                                )
                            }
                            ValueRefType::Committed { component_address } => {
                                let vault_address = (component_address, *vault_id);
                                let vault_value = self
                                    .track
                                    .borrow_global_mut_value(vault_address.clone())
                                    .map_err(|e| match e {
                                        TrackError::NotFound => panic!("Expected to find vault"),
                                        TrackError::Reentrancy => {
                                            panic!("Vault logic is causing reentrancy")
                                        }
                                    })?;
                                let resource_address = vault_value.vault().resource_address();
                                (
                                    resource_address,
                                    TrackedNative(vault_address.into(), vault_value),
                                )
                            }
                        }
                    }
                };

                let substate_value = self
                    .track
                    .borrow_global_value(resource_address.clone())
                    .unwrap();
                let resource_manager = match substate_value {
                    SubstateValue::Resource(resource_manager) => resource_manager,
                    _ => panic!("Value is not a resource manager"),
                };

                let method_auth = resource_manager.get_vault_auth(&fn_ident);
                Ok((snode_state, vec![method_auth.clone()]))
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
                TrackedNative(..)
                | TrackedScrypto(..)
                | Borrowed(BorrowedSNodeState::Vault(..))
                | Borrowed(BorrowedSNodeState::ValueRef(
                    ValueId::Transient(TransientValueId::Bucket(..)),
                    ..,
                ))
                | Borrowed(BorrowedSNodeState::Blueprint(..))
                | Consumed(TransientValue::Bucket(..)) => {
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
                Borrowed(BorrowedSNodeState::Blueprint(..))
                | TrackedScrypto(..)
                | Static(StaticSNodeState::TransactionProcessor) => {
                    Some(RefCell::new(AuthZone::new()))
                }
                _ => None,
            },
            match loaded_snode {
                Static(StaticSNodeState::TransactionProcessor) => {
                    Some(RefCell::new(Worktop::new()))
                }
                _ => None,
            },
            taken_values,
            self.auth_zone.as_ref(),
            cost_unit_counter,
            fee_table,
        );

        // invoke the main function
        let run_result = frame.run(Some(snode_ref), loaded_snode, &fn_ident, input);

        // re-gain ownership of the cost unit counter and fee table
        self.cost_unit_counter = frame.cost_unit_counter;
        self.fee_table = frame.fee_table;

        // unwrap and continue
        let (result, received_values) = run_result?;

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

    fn borrow_global_mut_resource_manager(
        &mut self,
        resource_address: ResourceAddress,
    ) -> Result<ResourceManager, RuntimeError> {
        self.track
            .borrow_global_mut_value(resource_address.clone())
            .map(|v| v.into())
            .map_err(|e| match e {
                TrackError::NotFound => {
                    RuntimeError::ResourceManagerNotFound(resource_address.clone())
                }
                TrackError::Reentrancy => panic!("Reentrancy occurred in resource manager"),
            })
    }

    fn return_borrowed_global_resource_manager(
        &mut self,
        resource_address: ResourceAddress,
        resource_manager: ResourceManager,
    ) {
        self.track
            .return_borrowed_global_mut_value(resource_address, resource_manager)
    }

    fn take_proof(&mut self, proof_id: ProofId) -> Result<Proof, RuntimeError> {
        let value = self
            .owned_values
            .remove(&ValueId::Transient(TransientValueId::Proof(
                proof_id.clone(),
            )))
            .ok_or(RuntimeError::ProofNotFound(proof_id))?
            .into_inner();

        match value {
            REValue::Transient(TransientValue::Proof(proof)) => Ok(proof),
            _ => panic!("Expected proof"),
        }
    }

    fn take_bucket(&mut self, bucket_id: BucketId) -> Result<Bucket, RuntimeError> {
        self.owned_values
            .remove(&ValueId::Transient(TransientValueId::Bucket(
                bucket_id.clone(),
            )))
            .map(|value| match value.into_inner() {
                REValue::Transient(TransientValue::Bucket(bucket)) => bucket,
                _ => panic!("Expected bucket"),
            })
            .ok_or(RuntimeError::BucketNotFound(bucket_id))
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
        self.track.create_uuid_value(resource_manager).into()
    }

    fn create_package(&mut self, package: ValidatedPackage) -> PackageAddress {
        self.track.create_uuid_value(package).into()
    }

    fn create_component(&mut self, component: Component) -> Result<ComponentAddress, RuntimeError> {
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
        let address = self.track.create_uuid_value(component);
        self.track
            .insert_objects_into_component(to_store_values, address.clone().into());
        Ok(address.into())
    }

    fn create_kv_store(&mut self) -> KeyValueStoreId {
        let kv_store_id = self.track.new_kv_store_id();
        self.owned_values.insert(
            ValueId::kv_store_id(kv_store_id.clone()),
            RefCell::new(REValue::Stored(StoredValue::KeyValueStore {
                store: PreCommittedKeyValueStore::new(),
                child_values: HashMap::new(),
            })),
        );
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

        // Get reference to data address
        let (store, ref_type) = match address {
            SubstateAddress::Component(component_address) => {
                let component = self
                    .refed_components
                    .get_mut(&component_address)
                    .ok_or(RuntimeError::ComponentNotFound(component_address.clone()))?;
                let store = SubstateEntry::Component(component_address.clone(), component);
                (
                    store,
                    ValueRefType::Committed {
                        component_address: component_address.clone(),
                    },
                )
            }
            SubstateAddress::ComponentInfo(component_address) => {
                let component = self.refed_components.get(&component_address);
                let store = if let Some(component) = component {
                    SubstateEntry::ComponentInfo(component_address.clone(), component)
                } else {
                    self.track
                        .borrow_global_value(component_address.clone())
                        .map_err(|e| match e {
                            TrackError::NotFound => {
                                RuntimeError::ComponentNotFound(component_address)
                            }
                            TrackError::Reentrancy => panic!("Should not run into reentrancy"),
                        })?;
                    SubstateEntry::ComponentInfoTracked(component_address.clone())
                };
                (
                    store,
                    ValueRefType::Committed {
                        component_address: component_address.clone(),
                    },
                )
            }
            SubstateAddress::KeyValueEntry(kv_store_id, key) => {
                verify_stored_key(&key)?;

                if self
                    .owned_values
                    .contains_key(&ValueId::kv_store_id(kv_store_id.clone()))
                {
                    let ref_store = self
                        .owned_values
                        .get_mut(&ValueId::kv_store_id(kv_store_id))
                        .unwrap()
                        .borrow_mut();
                    //.get_mut();
                    (
                        SubstateEntry::KeyValueStoreRef(REValueRef::Owned(ref_store), key),
                        ValueRefType::Uncommitted {
                            root: kv_store_id.clone(),
                            ancestors: vec![],
                        },
                    )
                } else {
                    let value_id = StoredValueId::KeyValueStoreId(kv_store_id.clone());
                    let maybe_value_ref = self.refed_values.get(&value_id).cloned();
                    let value_ref = maybe_value_ref
                        .ok_or_else(|| RuntimeError::KeyValueStoreNotFound(kv_store_id.clone()))?;
                    match &value_ref {
                        ValueRefType::Uncommitted { root, ancestors } => {
                            let mut next_ancestors = ancestors.clone();
                            next_ancestors.push(kv_store_id);
                            let value_ref_type = ValueRefType::Uncommitted {
                                root: root.clone(),
                                ancestors: next_ancestors,
                            };
                            let root_value = self
                                .owned_values
                                .get_mut(&ValueId::kv_store_id(*root))
                                .unwrap();
                            let ref_store = root_value
                                .get_mut()
                                .to_stored()
                                .get_child(ancestors, &value_id);
                            (
                                SubstateEntry::KeyValueStoreRef(REValueRef::Ref(ref_store), key),
                                value_ref_type,
                            )
                        }
                        ValueRefType::Committed { component_address } => {
                            let store = SubstateEntry::KeyValueStoreTracked(
                                component_address.clone(),
                                kv_store_id,
                                key,
                            );
                            (
                                store,
                                ValueRefType::Committed {
                                    component_address: *component_address,
                                },
                            )
                        }
                    }
                }
            }
        };

        // Read current value
        let current_value = match &store {
            SubstateEntry::Component(_, component) => {
                ScryptoValue::from_slice(component.state()).expect("Expected to decode")
            }
            SubstateEntry::ComponentInfo(_, component) => {
                let info = (
                    component.package_address().clone(),
                    component.blueprint_name().to_string(),
                );
                ScryptoValue::from_typed(&info)
            }
            SubstateEntry::ComponentInfoTracked(component_address) => {
                let component = self
                    .track
                    .borrow_global_value(component_address.clone())
                    .unwrap()
                    .component();
                let info = (
                    component.package_address().clone(),
                    component.blueprint_name().to_string(),
                );
                ScryptoValue::from_typed(&info)
            }
            SubstateEntry::KeyValueStoreRef(store, key) => {
                let value = store.kv_store().get(&key.raw).map_or(
                    Value::Option {
                        value: Box::new(Option::None),
                    },
                    |v| Value::Option {
                        value: Box::new(Some(v.dom)),
                    },
                );
                ScryptoValue::from_value(value).unwrap()
            }
            SubstateEntry::KeyValueStoreTracked(component_address, kv_store_id, key) => {
                let substate_value = self.track.read_key_value(
                    Address::KeyValueStore(*component_address, *kv_store_id),
                    key.raw.to_vec(),
                );
                let value = substate_value.kv_entry().as_ref().map_or(
                    Value::Option {
                        value: Box::new(Option::None),
                    },
                    |bytes| Value::Option {
                        value: Box::new(Some(decode_any(bytes).unwrap())),
                    },
                );
                ScryptoValue::from_value(value).unwrap()
            }
        };
        let cur_children = to_stored_ids(current_value.stored_value_ids())?;

        // Fulfill method
        match instruction {
            DataInstruction::Read => {
                for stored_value_id in cur_children {
                    self.refed_values.insert(stored_value_id, ref_type.clone());
                }
                Ok(current_value)
            }
            DataInstruction::Write(value) => {
                let missing = to_stored_ids(missing)?;
                verify_stored_value_update(&cur_children, &missing)?;

                let to_store_values = to_stored_values(taken_values)?;

                // TODO: verify against some schema

                // Write values
                match store {
                    SubstateEntry::ComponentInfo(..) | SubstateEntry::ComponentInfoTracked(..) => {
                        return Err(RuntimeError::InvalidDataWrite);
                    }
                    SubstateEntry::Component(component_address, component) => {
                        component.set_state(value.raw);
                        self.track
                            .insert_objects_into_component(to_store_values, component_address);
                    }
                    SubstateEntry::KeyValueStoreRef(mut stored_value, key) => {
                        stored_value.kv_store_mut().put(key.raw, value);
                        stored_value
                            .to_stored_mut()
                            .insert_children(to_store_values);
                    }
                    SubstateEntry::KeyValueStoreTracked(component_address, kv_store_id, key) => {
                        self.track.set_key_value(
                            Address::KeyValueStore(component_address.clone(), kv_store_id),
                            key.raw,
                            SubstateValue::KeyValueStoreEntry(Some(value.raw)),
                        );
                        self.track
                            .insert_objects_into_component(to_store_values, component_address);
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
