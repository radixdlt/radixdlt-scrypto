use colored::*;
use sbor::path::SborPath;
use sbor::rust::borrow::ToOwned;
use sbor::rust::boxed::Box;
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
use scrypto::resource::AuthZoneClearInput;
use scrypto::values::*;
use transaction::validation::*;

use crate::engine::LoadedSNodeState::{Borrowed, Consumed, Static};
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
pub enum ValueType {
    Owned,
    Ref(ValueRefType),
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
    Blueprint(
        ScryptoActorInfo,
        ValidatedPackage,
    ),
    Component(ScryptoActorInfo, ValidatedPackage, Component),
    Resource(ResourceAddress, ResourceManager),
    Bucket(BucketId, RefMut<'a, Bucket>),
    Proof(ProofId, RefMut<'a, Proof>),
    Vault(VaultId, RefMut<'a, StoredValue>, ValueType),
    TrackedVault(VaultId, Vault, ValueType),
}

pub enum StaticSNodeState {
    Package,
    Resource,
    System,
    TransactionProcessor,
}

pub enum LoadedSNodeState<'a> {
    Static(StaticSNodeState),
    Consumed(Option<ConsumedSNodeState>),
    Borrowed(BorrowedSNodeState<'a>),
}

pub enum SNodeState<'a> {
    Root,
    SystemStatic,
    TransactionProcessorStatic,
    PackageStatic,
    AuthZoneRef(&'a mut AuthZone),
    WorktopRef(&'a mut Worktop),
    // TODO: use reference to the package
    Blueprint(
        ScryptoActorInfo,
        ValidatedPackage,
    ),
    Component(
        ScryptoActorInfo,
        ValidatedPackage,
        &'a mut Component,
    ),
    ResourceStatic,
    ResourceRef(ResourceAddress, &'a mut ResourceManager),
    BucketRef(BucketId, &'a mut Bucket),
    Bucket(Bucket),
    ProofRef(ProofId, &'a mut Proof),
    Proof(Proof),
    VaultRef(VaultId, &'a mut StoredValue),
    TrackedVaultRef(VaultId, &'a mut Vault),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MoveMethod {
    AsReturn,
    AsArgument,
}

impl<'a> LoadedSNodeState<'a> {
    fn to_snode_state(&mut self) -> SNodeState {
        match self {
            Static(static_state) => match static_state {
                StaticSNodeState::Package => SNodeState::PackageStatic,
                StaticSNodeState::Resource => SNodeState::ResourceStatic,
                StaticSNodeState::System => SNodeState::SystemStatic,
                StaticSNodeState::TransactionProcessor => SNodeState::TransactionProcessorStatic,
            },
            Consumed(ref mut to_consume) => match to_consume.take().unwrap() {
                ConsumedSNodeState::Proof(proof) => SNodeState::Proof(proof),
                ConsumedSNodeState::Bucket(bucket) => SNodeState::Bucket(bucket),
            },
            Borrowed(ref mut borrowed) => match borrowed {
                BorrowedSNodeState::AuthZone(s) => SNodeState::AuthZoneRef(s),
                BorrowedSNodeState::Worktop(s) => SNodeState::WorktopRef(s),
                BorrowedSNodeState::Blueprint(
                    info,
                    package,
                ) => SNodeState::Blueprint(
                    info.clone(),
                    package.clone(),
                ),
                BorrowedSNodeState::Component(
                    info,
                    package,
                    component,
                ) => SNodeState::Component(
                    info.clone(),
                    package.clone(),
                    component,
                ),
                BorrowedSNodeState::Resource(addr, s) => SNodeState::ResourceRef(*addr, s),
                BorrowedSNodeState::Bucket(id, s) => SNodeState::BucketRef(*id, s),
                BorrowedSNodeState::Proof(id, s) => SNodeState::ProofRef(*id, s),
                BorrowedSNodeState::Vault(id, vault, ..) => SNodeState::VaultRef(*id, vault),
                BorrowedSNodeState::TrackedVault(id, vault, ..) => {
                    SNodeState::TrackedVaultRef(*id, vault)
                }
            },
        }
    }

    fn cleanup<S: ReadableSubstateStore>(self, track: &mut Track<S>) {
        if let Borrowed(borrowed) = self {
            match borrowed {
                BorrowedSNodeState::AuthZone(..) => {}
                BorrowedSNodeState::Worktop(..) => {}
                BorrowedSNodeState::Bucket(..) => {}
                BorrowedSNodeState::Proof(..) => {}
                BorrowedSNodeState::Vault(..) => {}
                BorrowedSNodeState::Blueprint(..) => {}
                BorrowedSNodeState::Component(actor, _, component) => {
                    track.return_borrowed_global_mut_value(
                        actor.component_address().unwrap(),
                        component,
                    );
                }
                BorrowedSNodeState::Resource(resource_address, resource_manager) => {
                    track.return_borrowed_global_mut_value(resource_address, resource_manager);
                }
                BorrowedSNodeState::TrackedVault(vault_id, vault, value_type) => match value_type {
                    ValueType::Ref(ValueRefType::Committed { component_address }) => {
                        track
                            .return_borrowed_global_mut_value((component_address, vault_id), vault);
                    }
                    _ => panic!("Tracked vaults are owned by Track and only references are passed to call frames. Will remove this in a PR soon."),
                },
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
            self.sys_log(
                Level::Warn,
                format!("Dangling bucket: {}, {:?}", bucket_id, ref_bucket),
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
            self.sys_log(Level::Warn, format!("Dangling value: {:?}", value));
            resource = match value {
                StoredValue::Vault(_, vault) => ResourceFailure::Resource(vault.resource_address()),
                StoredValue::KeyValueStore(..) => ResourceFailure::UnclaimedKeyValueStore,
            };
            success = false;
        }

        if let Some(ref_worktop) = &self.worktop {
            let worktop = ref_worktop.borrow();
            if !worktop.is_empty() {
                self.sys_log(Level::Warn, "Resource worktop is not empty".to_string());
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
        let output = match snode {
            SNodeState::Root => {
                panic!("Root is not runnable")
            }
            SNodeState::SystemStatic => {
                System::static_main(fn_ident, input, self).map_err(RuntimeError::SystemError)
            }
            SNodeState::TransactionProcessorStatic => {
                TransactionProcessor::static_main(fn_ident, input, self).map_err(|e| match e {
                    TransactionProcessorError::InvalidRequestData(_) => panic!("Illegal state"),
                    TransactionProcessorError::InvalidMethod => panic!("Illegal state"),
                    TransactionProcessorError::RuntimeError(e) => e,
                })
            }
            SNodeState::PackageStatic => ValidatedPackage::static_main(fn_ident, input, self)
                .map_err(RuntimeError::PackageError),
            SNodeState::AuthZoneRef(auth_zone) => auth_zone
                .main(fn_ident, input, self)
                .map_err(RuntimeError::AuthZoneError),
            SNodeState::WorktopRef(worktop) => worktop
                .main(fn_ident, input, self)
                .map_err(RuntimeError::WorktopError),
            SNodeState::Blueprint(
                actor,
                package,
            ) => {
                let export_name = format!("{}_main", actor.blueprint_name());
                package.invoke(
                    &actor,
                    &mut None,
                    export_name,
                    fn_ident,
                    input,
                    self,
                )
            }
            SNodeState::Component(
                actor,
                package,
                component,
            ) => {
                let initial_value = ScryptoValue::from_slice(component.state()).unwrap();
                for value_id in initial_value.stored_value_ids() {
                    self.refed_values.insert(
                        value_id,
                        ValueRefType::Committed {
                            component_address: actor.component_address().unwrap(),
                        },
                    );
                }

                let mut maybe_component = Some(component);
                let export_name = format!("{}_main", actor.blueprint_name());
                let rtn = package.invoke(
                    &actor,
                    &mut maybe_component,
                    export_name,
                    fn_ident,
                    input,
                    self,
                )?;

                let component = maybe_component.unwrap();
                let value = ScryptoValue::from_slice(component.state())
                    .map_err(RuntimeError::DecodeError)?;
                verify_stored_value(&value)?;
                let new_value_ids = stored_value_update(&initial_value, &value)?;
                let addr = actor.component_address().unwrap();
                // TODO: should we take values when component is actually written to rather than at the end of invocation?
                let new_values = self.take_values(&new_value_ids)?;
                self.track.insert_objects_into_component(new_values, addr);

                Ok(rtn)
            }
            SNodeState::ResourceStatic => ResourceManager::static_main(fn_ident, input, self)
                .map_err(RuntimeError::ResourceManagerError),
            SNodeState::ResourceRef(resource_address, resource_manager) => {
                let return_value = resource_manager
                    .main(resource_address, fn_ident, input, self)
                    .map_err(RuntimeError::ResourceManagerError)?;

                Ok(return_value)
            }
            SNodeState::BucketRef(bucket_id, bucket) => bucket
                .main(bucket_id, fn_ident, input, self)
                .map_err(RuntimeError::BucketError),
            SNodeState::Bucket(bucket) => bucket
                .consuming_main(fn_ident, input, self)
                .map_err(RuntimeError::BucketError),
            SNodeState::ProofRef(_, proof) => proof
                .main(fn_ident, input, self)
                .map_err(RuntimeError::ProofError),
            SNodeState::Proof(proof) => proof
                .main_consume(fn_ident, input)
                .map_err(RuntimeError::ProofError),
            SNodeState::VaultRef(_vault_id, value) => match value {
                StoredValue::Vault(id, vault) => vault
                    .main(*id, fn_ident, input, self)
                    .map_err(RuntimeError::VaultError),
                _ => panic!("Should be a vault"),
            },
            SNodeState::TrackedVaultRef(vault_id, vault) => vault
                .main(vault_id, fn_ident, input, self)
                .map_err(RuntimeError::VaultError),
        }?;

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

    fn take_values(
        &mut self,
        value_ids: &HashSet<StoredValueId>,
    ) -> Result<Vec<StoredValue>, RuntimeError> {
        let values = self.take_set(value_ids)?;
        for value in &values {
            if let StoredValue::KeyValueStore(_, store) = value {
                for id in store.all_descendants() {
                    self.refed_values.remove(&id);
                }
            }
        }
        Ok(values)
    }

    fn read_kv_store_entry_internal(
        &mut self,
        kv_store_id: KeyValueStoreId,
        key: &ScryptoValue,
    ) -> Result<(Option<ScryptoValue>, ValueType), RuntimeError> {
        verify_stored_key(key)?;

        let (maybe_value, value_type) = if self
            .owned_values
            .contains_key(&StoredValueId::KeyValueStoreId(kv_store_id.clone()))
        {
            let store = Self::get_owned_kv_store_mut(&mut self.owned_values, &kv_store_id).unwrap();
            let value = store.store.get(&key.raw).cloned();
            (value, ValueType::Owned)
        } else {
            let value_id = StoredValueId::KeyValueStoreId(kv_store_id.clone());
            let maybe_value_ref = self.refed_values.get(&value_id).cloned();
            let value_ref =
                maybe_value_ref.ok_or(RuntimeError::KeyValueStoreNotFound(kv_store_id.clone()))?;
            let value = match &value_ref {
                ValueRefType::Uncommitted { root, ancestors } => {
                    let root_store =
                        Self::get_owned_kv_store_mut(&mut self.owned_values, root).unwrap();
                    let mut value = root_store.get_child(ancestors, &value_id);
                    match value.deref_mut() {
                        StoredValue::KeyValueStore(_, store) => store.store.get(&key.raw).cloned(),
                        _ => panic!("Substate value is not a KeyValueStore entry"),
                    }
                }
                ValueRefType::Committed { component_address } => {
                    let substate_value = self.track.read_key_value(
                        Address::KeyValueStore(*component_address, kv_store_id),
                        key.raw.to_vec(),
                    );
                    match substate_value {
                        SubstateValue::KeyValueStoreEntry(v) => v,
                        _ => panic!("Substate value is not a KeyValueStore entry"),
                    }
                    .map(|v| ScryptoValue::from_slice(&v).expect("Expected to decode."))
                }
            };
            (value, ValueType::Ref(value_ref))
        };

        Ok((maybe_value, value_type))
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
                    StoredValue::KeyValueStore(_, store) => store,
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
        let remaining_cost_units = self.cost_unit_counter().remaining();
        self.sys_log(
            Level::Debug,
            format!(
                "Invoking: {:?} {:?}, remainging cost units: {}",
                snode_ref, &fn_ident, remaining_cost_units
            ),
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
            self.sys_log(Level::Debug, format!("Sending bucket: {:?}", bucket));
        }
        for proof in &moving_proofs {
            self.sys_log(Level::Debug, format!("Sending proof: {:?}", proof));
        }

        // Authorization and state load
        let (mut loaded_snode, method_auths) = match &snode_ref {
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
                        .read_value(package_address.clone())
                        .ok_or(RuntimeError::PackageNotFound(*package_address))?;
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
                            package.clone()
                        )),
                        vec![],
                    ))
                }
                ScryptoActor::Component(component_address) => {
                    let component_address = *component_address;

                    let component: Component = self
                        .track
                        .borrow_global_mut_value(component_address)
                        .map_err(|e| match e {
                            TrackError::NotFound => {
                                RuntimeError::ComponentNotFound(component_address)
                            }
                            TrackError::Reentrancy => {
                                RuntimeError::ComponentReentrancy(component_address)
                            }
                        })?
                        .into();
                    let package_address = component.package_address();
                    let blueprint_name = component.blueprint_name().to_string();
                    let substate_value = self
                        .track
                        .read_value(package_address)
                        .ok_or(RuntimeError::PackageNotFound(package_address))?;
                    let package = match substate_value {
                        SubstateValue::Package(package) => package,
                        _ => panic!("Value is not a package"),
                    };

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

                    Ok((
                        Borrowed(BorrowedSNodeState::Component(
                            ScryptoActorInfo::component(
                                package_address,
                                blueprint_name,
                                component_address,
                            ),
                            package.clone(),
                            component,
                        )),
                        method_auths,
                    ))
                }
            },
            SNodeRef::ResourceStatic => Ok((Static(StaticSNodeState::Resource), vec![])),
            SNodeRef::ResourceRef(resource_address) => {
                let resource_manager: ResourceManager = self
                    .track
                    .borrow_global_mut_value(resource_address.clone())
                    .map_err(|e| match e {
                        TrackError::NotFound => {
                            RuntimeError::ResourceManagerNotFound(resource_address.clone())
                        }
                        TrackError::Reentrancy => panic!("Reentrancy occurred in resource manager"),
                    })?
                    .into();

                let method_auth = resource_manager.get_auth(&fn_ident, &input).clone();
                Ok((
                    Borrowed(BorrowedSNodeState::Resource(
                        resource_address.clone(),
                        resource_manager,
                    )),
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
                let substate_value = self.track.read_value(resource_address.clone()).unwrap();
                let resource_manager = match substate_value {
                    SubstateValue::Resource(resource_manager) => resource_manager,
                    _ => panic!("Value is not a resource manager"),
                };
                let method_auth = resource_manager.get_consuming_bucket_auth(&fn_ident);
                Ok((
                    Consumed(Some(ConsumedSNodeState::Bucket(bucket))),
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
                Ok((Consumed(Some(ConsumedSNodeState::Proof(proof))), vec![]))
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
                            Borrowed(BorrowedSNodeState::Vault(
                                vault_id.clone(),
                                value.borrow_mut(),
                                ValueType::Owned,
                            )),
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
                                let root_store =
                                    Self::get_owned_kv_store_mut(&mut self.owned_values, &root)
                                        .unwrap();
                                let value = root_store.get_child(ancestors, &value_id);
                                let resource_address = match value.deref() {
                                    StoredValue::Vault(_, vault) => vault.resource_address(),
                                    _ => panic!("Expected vault"),
                                };
                                (
                                    resource_address,
                                    Borrowed(BorrowedSNodeState::Vault(
                                        vault_id.clone(),
                                        value,
                                        ValueType::Ref(value_ref),
                                    )),
                                )
                            }
                            ValueRefType::Committed { component_address } => {
                                let vault: Vault = self
                                    .track
                                    .borrow_global_mut_value((component_address, *vault_id))
                                    .map_err(|e| match e {
                                        TrackError::NotFound => panic!("Expected to find vault"),
                                        TrackError::Reentrancy => {
                                            panic!("Vault logic is causing reentrancy")
                                        }
                                    })?
                                    .into();
                                let resource_address = vault.resource_address();
                                (
                                    resource_address,
                                    Borrowed(BorrowedSNodeState::TrackedVault(
                                        vault_id.clone(),
                                        vault,
                                        ValueType::Ref(value_ref),
                                    )),
                                )
                            }
                        }
                    }
                };

                let substate_value = self.track.read_value(resource_address.clone()).unwrap();
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
                Borrowed(BorrowedSNodeState::Resource(_, _))
                | Borrowed(BorrowedSNodeState::Vault(_, _, _))
                | Borrowed(BorrowedSNodeState::TrackedVault(..))
                | Borrowed(BorrowedSNodeState::Bucket(..))
                | Borrowed(BorrowedSNodeState::Blueprint(..))
                | Borrowed(BorrowedSNodeState::Component(..))
                | Consumed(Some(ConsumedSNodeState::Bucket(_))) => {
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
                | Borrowed(BorrowedSNodeState::Component(..))
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
        let snode = loaded_snode.to_snode_state();
        let run_result = frame.run(Some(snode_ref), snode, &fn_ident, input);

        // re-gain ownership of the cost unit counter and fee table
        self.cost_unit_counter = frame.cost_unit_counter;
        self.fee_table = frame.fee_table;

        // unwrap and contine
        let (result, received_buckets, received_proofs, mut received_vaults) = run_result?;

        // Return borrowed snodes
        loaded_snode.cleanup(&mut self.track);

        // move buckets and proofs to this process.
        for (bucket_id, bucket) in received_buckets {
            self.sys_log(Level::Debug, format!("Received bucket: {:?}", bucket));
            self.buckets.insert(bucket_id, RefCell::new(bucket));
        }
        for (proof_id, proof) in received_proofs {
            self.sys_log(Level::Debug, format!("Received proof: {:?}", proof));
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

    fn read_kv_store_entry(
        &mut self,
        kv_store_id: KeyValueStoreId,
        key: ScryptoValue,
    ) -> Result<ScryptoValue, RuntimeError> {
        verify_stored_key(&key)?;

        let (maybe_value, parent_type) =
            self.read_kv_store_entry_internal(kv_store_id.clone(), &key)?;

        let ref_type = match parent_type {
            ValueType::Owned => ValueRefType::Uncommitted {
                root: kv_store_id,
                ancestors: vec![],
            },
            ValueType::Ref(ValueRefType::Uncommitted { root, ancestors }) => {
                let mut next_ancestors = ancestors.clone();
                next_ancestors.push(kv_store_id);
                ValueRefType::Uncommitted {
                    root: root.clone(),
                    ancestors: next_ancestors,
                }
            }
            ValueType::Ref(ValueRefType::Committed { component_address }) => {
                ValueRefType::Committed { component_address }
            }
        };
        match maybe_value {
            Some(v) => {
                for value_id in v.stored_value_ids() {
                    self.refed_values.insert(value_id, ref_type.clone());
                }

                let value = Value::Option {
                    value: Box::new(Some(v.dom)),
                };
                let encoded = encode_any(&value);
                Ok(ScryptoValue::from_slice(&encoded).unwrap())
            }
            None => {
                let value = Value::Option {
                    value: Box::new(Option::None),
                };
                let encoded = encode_any(&value);
                Ok(ScryptoValue::from_slice(&encoded).unwrap())
            }
        }
    }

    fn write_kv_store_entry(
        &mut self,
        kv_store_id: KeyValueStoreId,
        key: ScryptoValue,
        value: ScryptoValue,
    ) -> Result<(), RuntimeError> {
        verify_stored_value(&value)?;

        let (old_value, parent_type) = self.read_kv_store_entry_internal(kv_store_id, &key)?;
        let new_value_ids = match old_value {
            None => value.stored_value_ids(),
            Some(old_scrypto_value) => stored_value_update(&old_scrypto_value, &value)?,
        };
        let new_values = self.take_values(&new_value_ids)?;
        match parent_type {
            ValueType::Owned => {
                let kv_store = Self::get_owned_kv_store_mut(&mut self.owned_values, &kv_store_id)
                    .ok_or(RuntimeError::CyclicKeyValueStore(kv_store_id))?;
                kv_store.store.insert(key.raw, value);
                kv_store.insert_children(new_values)
            }
            ValueType::Ref(ValueRefType::Uncommitted { root, ancestors }) => {
                if let Some(root_store) =
                    Self::get_owned_kv_store_mut(&mut self.owned_values, &root)
                {
                    let id = &StoredValueId::KeyValueStoreId(kv_store_id);
                    let mut wrapped_store = root_store.get_child(&ancestors, id);
                    match wrapped_store.deref_mut() {
                        StoredValue::KeyValueStore(_, kv_store) => {
                            kv_store.store.insert(key.raw, value);
                            kv_store.insert_children(new_values)
                        }
                        _ => panic!("Expected KV store"),
                    }
                } else {
                    return Err(RuntimeError::CyclicKeyValueStore(kv_store_id.clone()));
                }
            }
            ValueType::Ref(ValueRefType::Committed { component_address }) => {
                self.track.set_key_value(
                    Address::KeyValueStore(component_address.clone(), kv_store_id),
                    key.raw,
                    SubstateValue::KeyValueStoreEntry(Some(value.raw)),
                );
                self.track
                    .insert_objects_into_component(new_values, component_address);
            }
        }

        Ok(())
    }

    fn get_component_info(
        &mut self,
        component_address: ComponentAddress,
    ) -> Result<(PackageAddress, String), RuntimeError> {
        let substate_value = self
            .track
            .read_value(component_address)
            .ok_or(RuntimeError::ComponentNotFound(component_address))?;

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
            RefCell::new(StoredValue::KeyValueStore(
                kv_store_id,
                PreCommittedKeyValueStore::new(),
            )),
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

    #[allow(unused_variables)]
    fn sys_log(&self, level: Level, message: String) {
        let (l, m) = match level {
            Level::Error => ("ERROR".red(), message.red()),
            Level::Warn => ("WARN".yellow(), message.yellow()),
            Level::Info => ("INFO".green(), message.green()),
            Level::Debug => ("DEBUG".cyan(), message.cyan()),
            Level::Trace => ("TRACE".normal(), message.normal()),
        };

        #[cfg(not(feature = "alloc"))]
        if self.trace {
            println!("{}[{:5}] {}", "  ".repeat(self.depth), l, m);
        }
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
