use sbor::path::SborPath;
use sbor::rust::borrow::ToOwned;
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

use crate::engine::SNodeState::{Borrowed, Consumed, Static, Tracked};
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
    buckets: HashMap<BucketId, RefCell<Bucket>>,
    proofs: HashMap<ProofId, RefCell<Proof>>,
    owned_values: HashMap<StoredValueId, RefCell<StoredValue>>,
    worktop: Option<RefCell<Worktop>>,
    auth_zone: Option<RefCell<AuthZone>>,

    /// Referenced values
    refed_values: HashMap<StoredValueId, ValueRefType>,

    /// Caller's auth zone
    caller_auth_zone: Option<&'p RefCell<AuthZone>>,

    /// There is a single cost unit counter and a single fee table per transaction execution.
    /// When a call ocurrs, they're passed from the parent to the child, and returned
    /// after the invocation.
    cost_unit_counter: Option<CostUnitCounter>,
    fee_table: Option<FeeTable>,

    phantom: PhantomData<I>,
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
        if !missing.contains(old_id) {
            return Err(RuntimeError::StoredValueRemoved(old_id.clone()));
        }
    }

    for missing_id in missing.iter() {
        if !old.contains(missing_id) {
            return Err(RuntimeError::ValueNotFound(missing_id.clone()));
        }
    }

    Ok(())
}

fn stored_value_update(
    old: &ScryptoValue,
    new: &ScryptoValue,
) -> Result<HashSet<StoredValueId>, RuntimeError> {
    let old_ids = old.stored_value_ids();
    let new_ids = new.stored_value_ids();
    for old_id in &old_ids {
        if !new_ids.contains(old_id) {
            return Err(RuntimeError::StoredValueRemoved(old_id.clone()));
        }
    }

    let mut new_value_ids = HashSet::new();
    for new_id in new_ids {
        if !old_ids.contains(&new_id) {
            new_value_ids.insert(new_id);
        }
    }
    Ok(new_value_ids)
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

pub enum ConsumedSNodeState {
    Bucket(Bucket),
    Proof(Proof),
}

pub enum BorrowedSNodeState<'a> {
    AuthZone(RefMut<'a, AuthZone>),
    Worktop(RefMut<'a, Worktop>),
    Bucket(BucketId, RefMut<'a, Bucket>),
    Proof(ProofId, RefMut<'a, Proof>),
    Vault(RefMut<'a, StoredValue>),
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
    Consumed(ConsumedSNodeState),
    Borrowed(BorrowedSNodeState<'a>),
    Tracked(
        Address,
        SubstateValue,
        Option<(ScryptoActorInfo, ValidatedPackage)>,
    ),
}

pub enum SNodeExecution<'a> {
    Static(StaticSNodeState),
    Consumed(ConsumedSNodeState),
    AuthZone(RefMut<'a, AuthZone>),
    Worktop(RefMut<'a, Worktop>),
    Bucket(BucketId, RefMut<'a, Bucket>),
    Proof(ProofId, RefMut<'a, Proof>),
    Vault(VaultId, &'a mut Vault),
    Blueprint(ScryptoActorInfo, ValidatedPackage),
    Resource(Address, &'a mut ResourceManager),
    Component(&'a mut Component, ScryptoActorInfo, ValidatedPackage),
}

enum KVStore<'a> {
    Ref(&'a mut StoredValue),
    Tracked(ComponentAddress),
}

pub enum KVStoreMethod {
    Read,
    Write(ScryptoValue),
}

pub struct KVStoreCall {
    pub key: ScryptoValue,
    pub method: KVStoreMethod,
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
                ConsumedSNodeState::Bucket(bucket) => bucket
                    .consuming_main(fn_ident, input, system)
                    .map_err(RuntimeError::BucketError),
                ConsumedSNodeState::Proof(proof) => proof
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
                package.invoke(&info, &mut None, fn_ident, input, system)
            }
            SNodeExecution::Bucket(bucket_id, mut bucket) => bucket
                .main(bucket_id, fn_ident, input, system)
                .map_err(RuntimeError::BucketError),
            SNodeExecution::Proof(_id, mut proof) => proof
                .main(fn_ident, input, system)
                .map_err(RuntimeError::ProofError),
            SNodeExecution::Vault(vault_id, vault) => vault
                .main(vault_id, fn_ident, input, system)
                .map_err(RuntimeError::VaultError),
            SNodeExecution::Resource(address, resource_manager) => resource_manager
                .main(address.clone().into(), fn_ident, input, system)
                .map_err(RuntimeError::ResourceManagerError),
            SNodeExecution::Component(component, ref actor, ref package) => {
                let mut maybe_component = Some(component);
                package.invoke(&actor, &mut maybe_component, fn_ident, input, system)
            }
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MoveMethod {
    AsReturn,
    AsArgument,
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
        buckets: HashMap<BucketId, Bucket>,
        proofs: HashMap<ProofId, Proof>,
        caller_auth_zone: Option<&'p RefCell<AuthZone>>,
        cost_unit_counter: CostUnitCounter,
        fee_table: FeeTable,
    ) -> Self {
        let mut celled_buckets = HashMap::new();
        for (id, b) in buckets {
            celled_buckets.insert(id, RefCell::new(b));
        }

        let mut celled_proofs = HashMap::new();
        for (id, proof) in proofs {
            celled_proofs.insert(id, RefCell::new(proof));
        }

        Self {
            transaction_hash,
            depth,
            trace,
            track,
            wasm_engine,
            wasm_instrumenter,
            buckets: celled_buckets,
            proofs: celled_proofs,
            owned_values: HashMap::new(),
            refed_values: HashMap::new(),
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

        for (bucket_id, ref_bucket) in &self.buckets {
            trace!(
                self,
                Level::Warn,
                "Dangling bucket: {}, {:?}",
                bucket_id,
                ref_bucket
            );
            resource = ResourceFailure::Resource(ref_bucket.borrow().resource_address());
            success = false;
        }

        let values: HashMap<StoredValueId, StoredValue> = self
            .owned_values
            .drain()
            .map(|(id, c)| (id, c.into_inner()))
            .collect();
        for (_, value) in values {
            trace!(self, Level::Warn, "Dangling value: {:?}", value);
            resource = match value {
                StoredValue::Vault(_, vault) => ResourceFailure::Resource(vault.resource_address()),
                StoredValue::KeyValueStore { .. } => ResourceFailure::UnclaimedKeyValueStore,
            };
            success = false;
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

    /// Sends buckets to another component/blueprint, either as argument or return
    fn send_buckets(
        from: &mut HashMap<BucketId, RefCell<Bucket>>,
        bucket_ids: &HashMap<BucketId, SborPath>,
    ) -> Result<HashMap<BucketId, Bucket>, RuntimeError> {
        let mut buckets = HashMap::new();
        for (bucket_id, _) in bucket_ids {
            let bucket = from
                .remove(bucket_id)
                .ok_or(RuntimeError::BucketNotFound(*bucket_id))?
                .into_inner();
            if bucket.is_locked() {
                return Err(RuntimeError::CantMoveLockedBucket);
            }
            buckets.insert(*bucket_id, bucket);
        }
        Ok(buckets)
    }

    /// Sends proofs to another component/blueprint, either as argument or return
    fn send_vaults(
        &mut self,
        vault_ids: &HashSet<VaultId>,
    ) -> Result<HashMap<VaultId, Vault>, RuntimeError> {
        let mut vault_ids_to_take = HashSet::new();
        for vault_id in vault_ids {
            vault_ids_to_take.insert(StoredValueId::VaultId(*vault_id));
        }
        let vaults_to_take = self.take_set(&vault_ids_to_take)?;

        let mut vaults = HashMap::new();
        for vault_to_take in vaults_to_take {
            match vault_to_take {
                StoredValue::Vault(vault_id, vault) => {
                    vaults.insert(vault_id, vault);
                }
                _ => panic!("Expected vault but was {:?}", vault_to_take),
            }
        }

        Ok(vaults)
    }

    /// Sends proofs to another component/blueprint, either as argument or return
    fn send_proofs(
        from: &mut HashMap<ProofId, RefCell<Proof>>,
        proof_ids: &HashMap<ProofId, SborPath>,
        method: MoveMethod,
    ) -> Result<HashMap<ProofId, Proof>, RuntimeError> {
        let mut proofs = HashMap::new();
        for (proof_id, _) in proof_ids {
            let mut proof = from
                .remove(proof_id)
                .ok_or(RuntimeError::ProofNotFound(*proof_id))?
                .into_inner();
            if proof.is_restricted() {
                return Err(RuntimeError::CantMoveRestrictedProof(*proof_id));
            }
            if matches!(method, MoveMethod::AsArgument) {
                proof.change_to_restricted();
            }
            proofs.insert(*proof_id, proof);
        }
        Ok(proofs)
    }

    pub fn run(
        &mut self,
        snode_ref: Option<SNodeRef>, // TODO: Remove, abstractions between invoke_snode() and run() are a bit messy right now
        snode: SNodeState<'p>,
        fn_ident: &str,
        input: ScryptoValue,
    ) -> Result<
        (
            ScryptoValue,
            HashMap<BucketId, Bucket>,
            HashMap<ProofId, Proof>,
            HashMap<VaultId, Vault>,
        ),
        RuntimeError,
    > {
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
                BorrowedSNodeState::Bucket(bucket_id, bucket) => {
                    SNodeExecution::Bucket(bucket_id, bucket)
                }
                BorrowedSNodeState::Proof(proof_id, proof) => {
                    SNodeExecution::Proof(proof_id, proof)
                }
                BorrowedSNodeState::Vault(value) => {
                    ref_container = Some(value);
                    match ref_container.as_mut().unwrap().deref_mut() {
                        StoredValue::Vault(id, vault) => SNodeExecution::Vault(*id, vault),
                        _ => panic!("Should be a vault"),
                    }
                }
            },
            SNodeState::Tracked(address, value, mut meta) => {
                let initial_value = match &value {
                    SubstateValue::Component(component) => {
                        let initial_value = ScryptoValue::from_slice(component.state()).unwrap();
                        for value_id in initial_value.stored_value_ids() {
                            self.refed_values.insert(
                                value_id,
                                ValueRefType::Committed {
                                    component_address: address.clone().into(),
                                },
                            );
                        }
                        Some(initial_value)
                    }
                    _ => None,
                };

                to_return.insert(address.clone(), (value, initial_value));
                let (mut_value, _) = to_return.get_mut(&address).unwrap();
                match mut_value {
                    SubstateValue::Resource(resouce_manager) => {
                        SNodeExecution::Resource(address.clone(), resouce_manager)
                    }
                    SubstateValue::Vault(vault) => {
                        let vault_address: (ComponentAddress, VaultId) = address.clone().into();
                        SNodeExecution::Vault(vault_address.1, vault)
                    }
                    SubstateValue::Component(component) => {
                        let (info, package) = meta.take().unwrap();
                        SNodeExecution::Component(component, info, package)
                    }
                    _ => panic!("Unexpected tracked value"),
                }
            }
        };

        let output = execution.execute(fn_ident, input, self)?;

        // Update track
        for (address, (value, initial_value)) in to_return.drain() {
            match &value {
                SubstateValue::Component(component) => {
                    let value = ScryptoValue::from_slice(component.state())
                        .map_err(RuntimeError::DecodeError)?;
                    verify_stored_value(&value)?;
                    let new_value_ids = stored_value_update(&initial_value.unwrap(), &value)?;
                    // TODO: should we take values when component is actually written to rather than at the end of invocation?
                    // TODO: check if component actually mutated?
                    let new_values = self.take_values(&new_value_ids)?;
                    self.track
                        .insert_objects_into_component(new_values, address.clone().into());
                }
                _ => {}
            }

            self.track.return_borrowed_global_mut_value(address, value);
        }

        self.process_return_data(snode_ref, &output)?;

        // figure out what buckets and resources to return
        let moving_buckets = Self::send_buckets(&mut self.buckets, &output.bucket_ids)?;
        let moving_proofs =
            Self::send_proofs(&mut self.proofs, &output.proof_ids, MoveMethod::AsReturn)?;
        let moving_vaults = self.send_vaults(&output.vault_ids)?;

        // drop proofs and check resource leak
        for (_, proof) in self.proofs.drain() {
            proof.into_inner().drop();
        }

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

        Ok((output, moving_buckets, moving_proofs, moving_vaults))
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
        value_ids: HashSet<StoredValueId>,
    ) -> (HashMap<StoredValueId, StoredValue>, HashSet<StoredValueId>) {
        let (taken, missing) = self.take_available(value_ids);
        for (_, value) in &taken {
            for id in value.all_descendants() {
                self.refed_values.remove(&id);
            }
        }
        (taken, missing)
    }

    fn take_values(
        &mut self,
        value_ids: &HashSet<StoredValueId>,
    ) -> Result<Vec<StoredValue>, RuntimeError> {
        let values = self.take_set(value_ids)?;
        for value in &values {
            for id in value.all_descendants() {
                self.refed_values.remove(&id);
            }
        }
        Ok(values)
    }

    pub fn take_available(
        &mut self,
        other: HashSet<StoredValueId>,
    ) -> (HashMap<StoredValueId, StoredValue>, HashSet<StoredValueId>) {
        let mut taken_values = HashMap::new();
        let mut missing_values = HashSet::new();

        for id in other {
            let maybe = self.owned_values.remove(&id);
            if let Some(value) = maybe {
                taken_values.insert(id, value.into_inner());
            } else {
                missing_values.insert(id);
            }
        }

        (taken_values, missing_values)
    }

    pub fn take_set(
        &mut self,
        other: &HashSet<StoredValueId>,
    ) -> Result<Vec<StoredValue>, RuntimeError> {
        let mut taken_values = Vec::new();

        for id in other {
            let value = self
                .owned_values
                .remove(id)
                .ok_or(RuntimeError::ValueNotFound(*id))?
                .into_inner();
            taken_values.push(value);
        }

        Ok(taken_values)
    }

    pub fn get_owned_kv_store_mut<'a>(
        owned_values: &'a mut HashMap<StoredValueId, RefCell<StoredValue>>,
        kv_store_id: &KeyValueStoreId,
    ) -> Option<&'a mut PreCommittedKeyValueStore> {
        owned_values
            .get_mut(&StoredValueId::KeyValueStoreId(*kv_store_id))
            .map(|v| {
                let stored_value = v.get_mut();
                match stored_value {
                    StoredValue::KeyValueStore { store, .. } => store,
                    _ => panic!("Expected KV store"),
                }
            })
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

        Self::process_call_data(&input)?;

        // Figure out what buckets and proofs to move from this process
        let mut moving_buckets = HashMap::new();
        let mut moving_proofs = HashMap::new();
        moving_buckets.extend(Self::send_buckets(&mut self.buckets, &input.bucket_ids)?);
        moving_proofs.extend(Self::send_proofs(
            &mut self.proofs,
            &input.proof_ids,
            MoveMethod::AsArgument,
        )?);
        for bucket in &moving_buckets {
            trace!(self, Level::Debug, "Sending bucket: {:?}", bucket);
        }
        for proof in &moving_proofs {
            trace!(self, Level::Debug, "Sending proof: {:?}", proof);
        }

        // Authorization and state load
        let (loaded_snode, method_auths) = match &snode_ref {
            SNodeRef::TransactionProcessor => {
                // FIXME: only TransactionExecutor can invoke this function
                Ok((Static(StaticSNodeState::TransactionProcessor), vec![]))
            }
            SNodeRef::PackageStatic => Ok((Static(StaticSNodeState::Package), vec![])),
            SNodeRef::SystemStatic => Ok((Static(StaticSNodeState::System), vec![])),
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
                        Tracked(
                            component_address.into(),
                            component_value,
                            Some((actor_info, package.clone())),
                        ),
                        method_auths,
                    ))
                }
            },
            SNodeRef::ResourceStatic => Ok((Static(StaticSNodeState::Resource), vec![])),
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
                    Tracked(resource_address.clone().into(), resman_value, None),
                    vec![method_auth],
                ))
            }
            SNodeRef::Bucket(bucket_id) => {
                let bucket = self
                    .buckets
                    .remove(&bucket_id)
                    .ok_or(RuntimeError::BucketNotFound(bucket_id.clone()))?
                    .into_inner();
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
                Ok((
                    Consumed(ConsumedSNodeState::Bucket(bucket)),
                    vec![method_auth.clone()],
                ))
            }
            SNodeRef::BucketRef(bucket_id) => {
                let bucket_cell = self
                    .buckets
                    .get(&bucket_id)
                    .ok_or(RuntimeError::BucketNotFound(bucket_id.clone()))?;
                let bucket = bucket_cell.borrow_mut();
                Ok((
                    Borrowed(BorrowedSNodeState::Bucket(bucket_id.clone(), bucket)),
                    vec![],
                ))
            }
            SNodeRef::ProofRef(proof_id) => {
                let proof_cell = self
                    .proofs
                    .get(&proof_id)
                    .ok_or(RuntimeError::ProofNotFound(proof_id.clone()))?;
                let proof = proof_cell.borrow_mut();
                Ok((
                    Borrowed(BorrowedSNodeState::Proof(proof_id.clone(), proof)),
                    vec![],
                ))
            }
            SNodeRef::Proof(proof_id) => {
                let proof = self
                    .proofs
                    .remove(&proof_id)
                    .ok_or(RuntimeError::ProofNotFound(proof_id.clone()))?
                    .into_inner();
                Ok((Consumed(ConsumedSNodeState::Proof(proof)), vec![]))
            }
            SNodeRef::VaultRef(vault_id) => {
                let (resource_address, snode_state) = {
                    if let Some(value) = self.owned_values.get(&StoredValueId::VaultId(*vault_id)) {
                        let resource_address = match value.borrow().deref() {
                            StoredValue::Vault(_, vault) => vault.resource_address(),
                            _ => panic!("Expected vault"),
                        };

                        (
                            resource_address,
                            Borrowed(BorrowedSNodeState::Vault(value.borrow_mut())),
                        )
                    } else {
                        let value_id = StoredValueId::VaultId(*vault_id);
                        let maybe_value_ref = self.refed_values.get(&value_id).cloned();
                        let value_ref =
                            maybe_value_ref.ok_or(RuntimeError::ValueNotFound(value_id.clone()))?;
                        match value_ref {
                            ValueRefType::Uncommitted {
                                root,
                                ref ancestors,
                            } => {
                                let root_store = self
                                    .owned_values
                                    .get_mut(&StoredValueId::KeyValueStoreId(root))
                                    .unwrap()
                                    .get_mut();
                                let value = root_store.get_child(ancestors, &value_id);
                                let resource_address = match value.deref() {
                                    StoredValue::Vault(_, vault) => vault.resource_address(),
                                    _ => panic!("Expected vault"),
                                };
                                (resource_address, Borrowed(BorrowedSNodeState::Vault(value)))
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
                                    Tracked(vault_address.into(), vault_value, None),
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
                Tracked(..)
                | Borrowed(BorrowedSNodeState::Vault(..))
                | Borrowed(BorrowedSNodeState::Bucket(..))
                | Borrowed(BorrowedSNodeState::Blueprint(..))
                | Consumed(ConsumedSNodeState::Bucket(..)) => {
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
                | Tracked(..)
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
            moving_buckets,
            moving_proofs,
            self.auth_zone.as_ref(),
            cost_unit_counter,
            fee_table,
        );

        // invoke the main function
        let run_result = frame.run(Some(snode_ref), loaded_snode, &fn_ident, input);

        // re-gain ownership of the cost unit counter and fee table
        self.cost_unit_counter = frame.cost_unit_counter;
        self.fee_table = frame.fee_table;

        // unwrap and contine
        let (result, received_buckets, received_proofs, mut received_vaults) = run_result?;

        // move buckets and proofs to this process.
        for (bucket_id, bucket) in received_buckets {
            trace!(self, Level::Debug, "Received bucket: {:?}", bucket);
            self.buckets.insert(bucket_id, RefCell::new(bucket));
        }
        for (proof_id, proof) in received_proofs {
            trace!(self, Level::Debug, "Received proof: {:?}", proof);
            self.proofs.insert(proof_id, RefCell::new(proof));
        }
        for (vault_id, vault) in received_vaults.drain() {
            self.owned_values.insert(
                StoredValueId::VaultId(vault_id.clone()),
                RefCell::new(StoredValue::Vault(vault_id, vault)),
            );
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

    fn create_proof(&mut self, proof: Proof) -> Result<ProofId, RuntimeError> {
        let proof_id = self.track.new_proof_id();
        self.proofs.insert(proof_id, RefCell::new(proof));
        Ok(proof_id)
    }

    fn take_proof(&mut self, proof_id: ProofId) -> Result<Proof, RuntimeError> {
        let proof = self
            .proofs
            .remove(&proof_id)
            .ok_or(RuntimeError::ProofNotFound(proof_id))?
            .into_inner();

        Ok(proof)
    }

    fn create_bucket(&mut self, container: ResourceContainer) -> Result<BucketId, RuntimeError> {
        let bucket_id = self.track.new_bucket_id();
        self.buckets
            .insert(bucket_id, RefCell::new(Bucket::new(container)));
        Ok(bucket_id)
    }

    fn create_vault(&mut self, container: ResourceContainer) -> Result<VaultId, RuntimeError> {
        let vault_id = self.track.new_vault_id();
        self.owned_values.insert(
            StoredValueId::VaultId(vault_id.clone()),
            RefCell::new(StoredValue::Vault(vault_id, Vault::new(container))),
        );
        Ok(vault_id)
    }

    fn take_bucket(&mut self, bucket_id: BucketId) -> Result<Bucket, RuntimeError> {
        self.buckets
            .remove(&bucket_id)
            .map(RefCell::into_inner)
            .ok_or(RuntimeError::BucketNotFound(bucket_id))
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
        let values = self.take_values(&value.stored_value_ids())?;
        let address = self.track.create_uuid_value(component);
        self.track
            .insert_objects_into_component(values, address.clone().into());
        Ok(address.into())
    }

    fn kv_store_call(
        &mut self,
        kv_store_id: KeyValueStoreId,
        input: KVStoreCall,
    ) -> Result<ScryptoValue, RuntimeError> {
        verify_stored_key(&input.key)?;

        // If write, collect new child values
        let (taken_values, missing) = match &input.method {
            KVStoreMethod::Write(value) => {
                verify_stored_value(value)?;
                let value_ids = value.stored_value_ids();
                self.take_available_values(value_ids)
            }
            KVStoreMethod::Read => (HashMap::new(), HashSet::new()),
        };

        // Get Key Value Store
        let (store, ref_type) = if self
            .owned_values
            .contains_key(&StoredValueId::KeyValueStoreId(kv_store_id.clone()))
        {
            let ref_store = self
                .owned_values
                .get_mut(&StoredValueId::KeyValueStoreId(kv_store_id))
                .unwrap()
                .get_mut();
            (
                KVStore::Ref(ref_store),
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
                        .get_mut(&StoredValueId::KeyValueStoreId(*root))
                        .unwrap();
                    let ref_store = root_value.get_mut().get_child_mut(ancestors, &value_id);
                    (KVStore::Ref(ref_store), value_ref_type)
                }
                ValueRefType::Committed { component_address } => (
                    KVStore::Tracked(component_address.clone()),
                    ValueRefType::Committed {
                        component_address: *component_address,
                    },
                ),
            }
        };

        // Read current value
        let current_value = match &store {
            KVStore::Ref(store) => store.kv_store().get(&input.key.raw),
            KVStore::Tracked(component_address) => {
                let substate_value = self.track.read_key_value(
                    Address::KeyValueStore(*component_address, kv_store_id),
                    input.key.raw.to_vec(),
                );
                substate_value
                    .kv_entry()
                    .as_ref()
                    .map(|v| ScryptoValue::from_slice(&v).expect("Expected to decode."))
            }
        };
        let cur_children = current_value
            .as_ref()
            .map_or(HashSet::new(), |v| v.stored_value_ids());

        // Fulfill method
        match input.method {
            KVStoreMethod::Read => {
                for value_id in cur_children {
                    self.refed_values.insert(value_id, ref_type.clone());
                }

                let value = current_value.map_or(
                    Value::Option {
                        value: Box::new(Option::None),
                    },
                    |v| Value::Option {
                        value: Box::new(Some(v.dom)),
                    },
                );
                let encoded = encode_any(&value);
                Ok(ScryptoValue::from_slice(&encoded).unwrap())
            }
            KVStoreMethod::Write(value) => {
                verify_stored_value_update(&cur_children, &missing)?;

                // TODO: verify against some schema

                // Write values
                let new_values = taken_values.into_values().collect();
                match store {
                    KVStore::Ref(stored_value) => {
                        stored_value.kv_store_mut().put(input.key.raw, value);
                        stored_value.insert_children(new_values);
                    }
                    KVStore::Tracked(component_address) => {
                        self.track.set_key_value(
                            Address::KeyValueStore(component_address.clone(), kv_store_id),
                            input.key.raw,
                            SubstateValue::KeyValueStoreEntry(Some(value.raw)),
                        );
                        self.track
                            .insert_objects_into_component(new_values, component_address);
                    }
                }

                Ok(ScryptoValue::from_typed(&()))
            }
        }
    }

    fn get_component_info(
        &mut self,
        component_address: ComponentAddress,
    ) -> Result<(PackageAddress, String), RuntimeError> {
        let substate_value =
            self.track
                .borrow_global_value(component_address)
                .map_err(|e| match e {
                    TrackError::NotFound => RuntimeError::ComponentNotFound(component_address),
                    TrackError::Reentrancy => panic!("Component info reentrancy"),
                })?;

        if let SubstateValue::Component(component) = substate_value {
            Ok((
                component.package_address(),
                component.blueprint_name().to_owned(),
            ))
        } else {
            panic!("Value is not a component");
        }
    }

    fn create_kv_store(&mut self) -> KeyValueStoreId {
        let kv_store_id = self.track.new_kv_store_id();
        self.owned_values.insert(
            StoredValueId::KeyValueStoreId(kv_store_id.clone()),
            RefCell::new(StoredValue::KeyValueStore {
                id: kv_store_id,
                store: PreCommittedKeyValueStore::new(),
                child_values: HashMap::new(),
            }),
        );
        kv_store_id
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
                self.proofs
                    .get(&proof_id)
                    .map(|p| p.borrow().clone())
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
